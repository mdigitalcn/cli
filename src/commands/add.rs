use anyhow::Result;
use clap::Args as ClapArgs;

use super::CommandRunner;
use crate::help;
use crate::registry::RegistryKind;

#[derive(Debug, Clone, ClapArgs)]
#[command(about = help::ADD_ABOUT)]
#[command(long_about = help::ADD_LONG_ABOUT)]
#[command(after_help = help::ADD_AFTER_HELP)]
pub struct Args {
    /// Feature, config, or registry item names to add
    ///
    /// Supports: config names (prettier, eslint), features (tanstackquery),
    /// typed items (component:button), or plain names searched across all registries.
    #[arg(value_name = "NAME", num_args = 0.., help = "Names to add (e.g., prettier, component:button, hero-section)")]
    pub names: Vec<String>,

    /// Project root directory
    #[arg(long, default_value = ".", help = "Project root directory")]
    pub project_root: String,

    /// Bypass cache and fetch fresh data
    #[arg(long, help = "Bypass cache and fetch fresh data")]
    pub no_cache: bool,

    /// Overwrite existing files
    #[arg(long, help = "Overwrite existing files")]
    pub overwrite: bool,
}

/// Known config/feature names (embedded, no network needed)
const KNOWN_CONFIGS: &[&str] = &[
    "prettier", "eslint", "tsconfig", "husky", "commitlint",
    "dockerfile", "nginx", "githubcicd", "gitlabcicd", "sentry",
    "tanstackquery",
];

/// Type prefixes for explicit registry item targeting
fn parse_typed_name(input: &str) -> Option<(RegistryKind, String)> {
    let (prefix, name) = input.split_once(':')?;
    let kind = match prefix {
        "component" | "c" => RegistryKind::Component,
        "widget" | "w" => RegistryKind::Widget,
        "page" | "p" => RegistryKind::Page,
        "module" | "m" => RegistryKind::Module,
        "layout" | "l" => RegistryKind::Layout,
        _ => return None,
    };
    if name.is_empty() {
        return None;
    }
    Some((kind, name.to_string()))
}

impl CommandRunner for Args {
    fn run(&self) -> Result<()> {
        if self.names.is_empty() {
            return crate::features::add_interactive(&self.project_root);
        }

        // Partition names into configs/features vs registry items
        let mut configs: Vec<String> = Vec::new();
        let mut typed_items: Vec<(RegistryKind, Vec<String>)> = Vec::new();
        let mut unresolved: Vec<String> = Vec::new();

        for name in &self.names {
            let normalized = name.to_lowercase();

            // 1. Explicit type prefix: component:button, w:hero-section
            if let Some((kind, item_name)) = parse_typed_name(&normalized) {
                push_to_kind(&mut typed_items, kind, item_name);
                continue;
            }

            // 2. Known config/feature name
            if KNOWN_CONFIGS.contains(&normalized.as_str()) {
                configs.push(normalized);
                continue;
            }

            // 3. Namespace prefix (@acme/button) — delegate to registry
            if normalized.starts_with('@') {
                // Treat as component by default for namespaced items
                unresolved.push(normalized);
                continue;
            }

            // 4. Unknown name — try to find in registries
            unresolved.push(normalized);
        }

        // Process configs/features first (no network needed)
        for config_name in &configs {
            let sp = crate::output::spinner(&format!("Adding {}...", config_name));
            crate::features::add(&self.project_root, config_name)?;
            sp.finish_and_clear();
        }

        // Process explicitly typed registry items
        for (kind, names) in &typed_items {
            crate::registry::helpers::run_add(
                *kind,
                names,
                self.no_cache,
                &self.project_root,
                self.overwrite,
            )?;
        }

        // Process unresolved names — search across all registries
        if !unresolved.is_empty() {
            resolve_and_add(&unresolved, self.no_cache, &self.project_root, self.overwrite)?;
        }

        Ok(())
    }
}

fn push_to_kind(items: &mut Vec<(RegistryKind, Vec<String>)>, kind: RegistryKind, name: String) {
    if let Some(entry) = items.iter_mut().find(|(k, _)| *k == kind) {
        entry.1.push(name);
    } else {
        items.push((kind, vec![name]));
    }
}

/// Search all registry types for unresolved names.
/// shadcn-style: single flat lookup, type is embedded in each item.
fn resolve_and_add(
    names: &[String],
    no_cache: bool,
    project_root: &str,
    overwrite: bool,
) -> Result<()> {
    use crate::registry::helpers::{get_client_and_cache, fetch_registry};
    use std::collections::HashMap;

    let search_kinds = [
        RegistryKind::Component,
        RegistryKind::Widget,
        RegistryKind::Page,
        RegistryKind::Module,
        RegistryKind::Layout,
    ];

    // Fetch all registries (uses cache, fast if warm)
    let (client, cache) = get_client_and_cache();
    let sp = crate::output::spinner("Searching registries...");

    let mut registry_map = HashMap::new();
    for kind in &search_kinds {
        if let Ok(reg) = fetch_registry(&client, &cache, *kind, no_cache) {
            registry_map.insert(*kind, reg);
        }
    }
    sp.finish_and_clear();

    // Resolve each name: find which registry it belongs to
    let mut grouped: HashMap<RegistryKind, Vec<String>> = HashMap::new();
    let mut not_found: Vec<String> = Vec::new();

    for name in names {
        // Handle @namespace/item — route to component add with namespace
        if name.starts_with('@') {
            grouped.entry(RegistryKind::Component).or_default().push(name.clone());
            continue;
        }

        let mut found_in: Vec<RegistryKind> = Vec::new();
        for (kind, reg) in &registry_map {
            if reg.find(*kind, name).is_some() {
                found_in.push(*kind);
            }
        }

        match found_in.len() {
            0 => not_found.push(name.clone()),
            1 => {
                grouped.entry(found_in[0]).or_default().push(name.clone());
            }
            _ => {
                // Ambiguous — found in multiple registries
                let locations: Vec<String> = found_in.iter().map(|k| k.label().to_string()).collect();
                out!(warning, "'{}' found in multiple registries: {}", name, locations.join(", "));
                out!(info, "Use explicit prefix: {}:{}", found_in[0].label(), name);
                // Default to first match (component > widget > page > module > layout)
                grouped.entry(found_in[0]).or_default().push(name.clone());
            }
        }
    }

    // Report not found
    if !not_found.is_empty() {
        // Try fuzzy search for suggestions
        let mut suggestions: Vec<String> = Vec::new();
        for name in &not_found {
            for (kind, reg) in &registry_map {
                let matches = reg.search(*kind, name);
                for m in matches.iter().take(2) {
                    suggestions.push(format!("{}:{}", kind.label(), m.name));
                }
            }
        }

        if suggestions.is_empty() {
            anyhow::bail!(
                "Not found: {}. Not a config, feature, or registry item.\n\nRun `mdigitalcn list` to see available configs.\nRun `mdigitalcn component list` to see registry items.",
                not_found.join(", ")
            );
        } else {
            anyhow::bail!(
                "Not found: {}.\n\nDid you mean: {}?",
                not_found.join(", "),
                suggestions.join(", ")
            );
        }
    }

    // Add items grouped by kind
    for (kind, items) in &grouped {
        crate::registry::helpers::run_add(*kind, items, no_cache, project_root, overwrite)?;
    }

    Ok(())
}
