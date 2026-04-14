use anyhow::{bail, Result};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, MultiSelect};
use std::collections::HashMap;
use std::path::Path;

use super::cache::{CacheState, RegistryCache};
use super::client::GitHubClient;
use super::{Registry, RegistryKind};
use crate::config::{read_project_config, write_project_config, RegistryConfig};

pub fn get_client_and_cache() -> (GitHubClient, RegistryCache) {
    let config = read_project_config(".")
        .map(|c| c.effective_registry())
        .unwrap_or_else(|_| RegistryConfig::default_mdigitalcn());

    (GitHubClient::from_config(&config), RegistryCache::new())
}

pub fn fetch_registry(
    client: &GitHubClient,
    cache: &RegistryCache,
    kind: RegistryKind,
    no_cache: bool,
) -> Result<Registry> {
    if !no_cache {
        match cache.get(kind) {
            CacheState::Fresh(reg) => return Ok(reg),
            CacheState::Stale(reg) => {
                let sp = crate::output::spinner(&format!("Updating {} registry...", kind));
                match client.fetch_registry(kind) {
                    Ok(fresh) => {
                        let _ = cache.set(kind, &fresh);
                        sp.finish_and_clear();
                        return Ok(fresh);
                    }
                    Err(_) => {
                        sp.finish_and_clear();
                        out!(warning, "Using cached {} registry (network unavailable)", kind);
                        return Ok(reg);
                    }
                }
            }
            CacheState::Miss => {}
        }
    }

    let sp = crate::output::spinner(&format!("Fetching {} registry...", kind));
    let registry = client.fetch_registry(kind)?;
    let _ = cache.set(kind, &registry);
    sp.finish_and_clear();
    Ok(registry)
}

/// Fetch available templates for a given framework prefix (e.g. "vite", "nextjs").
/// Returns Vec<(name, display_name)> — e.g. [("vite/dashboard", "Admin Dashboard (Vite)")].
pub fn list_templates_for_framework(framework: &str, no_cache: bool) -> Result<Vec<(String, String)>> {
    let (client, cache) = get_client_and_cache();
    let registry = fetch_registry(&client, &cache, RegistryKind::Template, no_cache)?;
    let prefix = format!("{}/", framework);
    let items: Vec<(String, String)> = registry
        .items(RegistryKind::Template)
        .iter()
        .filter(|i| i.name.starts_with(&prefix))
        .map(|i| (i.name.clone(), i.display_name.clone()))
        .collect();
    Ok(items)
}

pub fn run_add(
    kind: RegistryKind,
    names: &[String],
    no_cache: bool,
    project_root: &str,
    overwrite: bool,
) -> Result<()> {
    if names.is_empty() {
        return add_items(kind, names, no_cache, project_root, overwrite, None);
    }

    let (default_names, namespaced) = super::group_by_namespace(names);

    if !default_names.is_empty() {
        add_items(kind, &default_names, no_cache, project_root, overwrite, None)?;
    }

    for (ns, ns_names) in &namespaced {
        add_items(kind, ns_names, false, project_root, overwrite, Some(ns))?;
    }

    Ok(())
}

fn add_items(
    kind: RegistryKind,
    names: &[String],
    no_cache: bool,
    project_root: &str,
    overwrite: bool,
    namespace: Option<&str>,
) -> Result<()> {
    let start = std::time::Instant::now();

    let (client, registry) = if let Some(ns) = namespace {
        let registries = crate::config::resolve_all_registries(project_root);
        let reg_config = registries.get(ns)
            .ok_or_else(|| anyhow::anyhow!(
                "Registry '{}' not configured. Run: mdigitalcn registry add {} <source>", ns, ns
            ))?;
        let token = crate::config::resolve_auth(reg_config.auth.as_deref());
        let client = GitHubClient::from_source(&reg_config.source, token)?;
        let sp = crate::output::spinner(&format!("Fetching {} registry...", ns));
        let registry = client.fetch_registry(kind)?;
        sp.finish_and_clear();
        (client, registry)
    } else {
        let (client, cache) = get_client_and_cache();
        let registry = fetch_registry(&client, &cache, kind, no_cache)?;
        (client, registry)
    };

    if registry.items(kind).is_empty() {
        out!(info, "No {}s available", kind);
        return Ok(());
    }

    let selected: Vec<String> = if names.is_empty() {
        interactive_select(&registry, kind)?
    } else {
        for name in names {
            if name.starts_with('_') { continue; }
            if registry.find(kind, name).is_none() {
                let suggestions = registry.search(kind, name);
                if suggestions.is_empty() {
                    if let Some(ns) = namespace {
                        bail!("'{}/{}' not found in {} registry", ns, name, ns);
                    } else {
                        bail!("{} '{}' not found. Run `mdigitalcn {} list`.", kind, name, kind);
                    }
                } else {
                    let s: Vec<&str> = suggestions.iter().map(|i| i.name.as_str()).collect();
                    bail!("'{}' not found. Did you mean: {}?", name, s.join(", "));
                }
            }
        }
        names.to_vec()
    };

    if selected.is_empty() {
        out!(info, "No {}s selected", kind);
        return Ok(());
    }

    let all_names = registry.resolve_deps(kind, &selected);
    let all_names = if kind == RegistryKind::Component {
        ensure_foundation(all_names, &registry, kind)
    } else {
        all_names
    };

    let extra: Vec<&String> = all_names.iter()
        .filter(|n| !selected.contains(n) && !n.starts_with('_'))
        .collect();
    if !extra.is_empty() {
        out!(info, "Also adding: {}", extra.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "));
    }

    let target_path = resolve_target_path(kind, project_root);
    let target = Path::new(project_root).join(&target_path);
    let mut all_peer_deps: HashMap<String, String> = registry.collect_npm_deps(kind, &all_names);
    let mut added: Vec<String> = Vec::new();
    let ns_label = namespace.map(|ns| format!(" (from {})", ns)).unwrap_or_default();

    for item_name in &all_names {
        let Some(item) = registry.find(kind, item_name) else { continue };
        let is_foundation = item_name.starts_with('_');

        if is_foundation {
            let check_path = if item_name == "_hooks" {
                target.join("hooks")
            } else {
                target.join(item.files.first().unwrap_or(&String::new()))
            };
            if check_path.exists() && !overwrite { continue; }

            let sp = crate::output::spinner(&format!("Setting up {}...", item.display_name));
            let written = client.download_foundation(kind, item, &target)?;
            sp.finish_and_clear();

            out!(success, "Added {}{}", item.display_name, ns_label);
            for file in &written {
                out!(step, "{}", file.strip_prefix(project_root).unwrap_or(file).display());
            }
        } else {
            let item_dir = target.join(&item.name);
            if item_dir.exists() && !overwrite {
                out!(warning, "Skipping {} (exists, use --overwrite)", item_name);
                continue;
            }

            let sp = crate::output::spinner(&format!("Downloading {}...", item.display_name));
            let written = client.download_item(kind, item, &target)?;
            sp.finish_and_clear();

            out!(success, "Added {}{}", item.display_name, ns_label);
            for file in &written {
                out!(step, "{}", file.strip_prefix(project_root).unwrap_or(file).display());
            }
        }

        for (dep, ver) in &item.peer_dependencies {
            all_peer_deps.insert(dep.clone(), ver.clone());
        }
        added.push(item_name.clone());
    }

    if let Ok(mut config) = read_project_config(project_root) {
        let tracked = config.generated.tracked_mut(kind);
        for name in &added {
            let tracked_name = if let Some(ns) = namespace {
                format!("{}/{}", ns, name)
            } else {
                name.clone()
            };
            if !tracked.contains(&tracked_name) {
                tracked.push(tracked_name);
            }
        }
        let _ = write_project_config(project_root, &config);
    }

    let user_added: Vec<&String> = added.iter().filter(|n| !n.starts_with('_')).collect();
    if user_added.is_empty() {
        out!(info, "No new {}s added", kind);
        return Ok(());
    }

    let elapsed = start.elapsed().as_secs_f64();
    out!(blank);
    out!(success, "Added {} {}(s) in {:.1}s: {}", user_added.len(), kind, elapsed,
        user_added.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "));
    for name in &user_added {
        out!(step, "{}/{}", target_path, name);
    }
    print_peer_deps(&all_peer_deps, project_root);
    Ok(())
}

fn ensure_foundation(mut names: Vec<String>, registry: &Registry, kind: RegistryKind) -> Vec<String> {
    let mut foundation: Vec<String> = Vec::new();

    for f in ["_utils", "_types", "_variants"] {
        if !names.contains(&f.to_string()) {
            foundation.push(f.to_string());
        }
    }

    for extra in ["_hooks", "_shared"] {
        let needed = names.iter().any(|n| {
            registry.find(kind, n)
                .map(|item| item.dependencies.iter().any(|d| d == extra))
                .unwrap_or(false)
        });
        if needed && !names.contains(&extra.to_string()) {
            foundation.push(extra.to_string());
        }
    }

    foundation.append(&mut names);
    foundation
}

pub fn run_list(
    kind: RegistryKind,
    no_cache: bool,
    category: Option<&str>,
    search: Option<&str>,
) -> Result<()> {
    let (client, cache) = get_client_and_cache();
    let registry = fetch_registry(&client, &cache, kind, no_cache)?;

    let items: Vec<_> = if let Some(query) = search {
        registry.search(kind, query)
    } else {
        registry.items(kind).iter().collect()
    };

    let items: Vec<_> = items.into_iter()
        .filter(|item| !item.name.starts_with('_'))
        .collect();

    if items.is_empty() {
        out!(info, "No {}s found", kind);
        return Ok(());
    }

    out!(header, "Available {}s", kind);

    let mut categories: HashMap<&str, Vec<&&super::RegistryItem>> = HashMap::new();
    for item in &items {
        if let Some(cat) = category {
            if item.category != cat { continue; }
        }
        categories.entry(item.category.as_str()).or_default().push(item);
    }

    let mut sorted: Vec<_> = categories.into_iter().collect();
    sorted.sort_by_key(|(cat, _)| *cat);

    let mut total = 0;
    for (cat, entries) in &sorted {
        out!(blank);
        println!("  {}", cat.to_uppercase().dimmed());
        for entry in entries {
            let extra = if entry.dependencies.is_empty() {
                String::new()
            } else {
                format!(" {}", format!("[needs: {}]", entry.dependencies.join(", ")).dimmed())
            };
            println!("    {:<20} {}{}", entry.name.bold(), entry.description.dimmed(), extra);
            total += 1;
        }
    }

    out!(blank);
    out!(info, "{} {}(s) available", total, kind);
    out!(blank);
    if kind == RegistryKind::Template {
        out!(hint, "mdigitalcn init --template <name> --name my-app");
    } else {
        out!(hint, "mdigitalcn {} add <name>", kind);
    }
    Ok(())
}

pub fn run_info(kind: RegistryKind, name: &str, no_cache: bool) -> Result<()> {
    let (client, cache) = get_client_and_cache();
    let registry = fetch_registry(&client, &cache, kind, no_cache)?;

    let item = registry.find(kind, name)
        .ok_or_else(|| anyhow::anyhow!("{} '{}' not found", kind, name))?;

    out!(header, "{}", item.display_name);
    pf("Name", &item.name);
    pf("Category", &item.category);
    pf("Description", &item.description);
    if let Some(fw) = &item.framework {
        pf("Framework", fw);
    }
    if let Some(arch) = &item.architecture {
        pf("Architecture", arch);
    }
    pf("Path", &item.path);
    if !item.files.is_empty() {
        pf("Files", &item.files.join(", "));
    }
    if !item.dependencies.is_empty() {
        pf("Dependencies", &item.dependencies.join(", "));
    }
    if !item.peer_dependencies.is_empty() {
        let peers: Vec<String> = item.peer_dependencies.iter()
            .map(|(k, v)| format!("{}@{}", k, v)).collect();
        pf("Peer deps", &peers.join(", "));
    }
    if !item.tags.is_empty() {
        pf("Tags", &item.tags.join(", "));
    }

    out!(blank);
    if kind == RegistryKind::Template {
        out!(hint, "mdigitalcn init --template {} --name my-app", name);
    } else {
        out!(hint, "mdigitalcn {} add {}", kind, name);
    }
    Ok(())
}

pub fn run_status(kind: RegistryKind, project_root: &str) -> Result<()> {
    let config = read_project_config(project_root)?;
    let tracked = config.generated.tracked(kind);
    let target_path = resolve_target_path(kind, project_root);

    if tracked.is_empty() {
        out!(info, "No {}s installed yet", kind);
        out!(hint, "mdigitalcn {} add <name>", kind);
        return Ok(());
    }

    out!(header, "Installed {}s", kind);
    for name in tracked {
        let path = Path::new(project_root).join(&target_path).join(name);
        if path.exists() {
            out!(success, "{} -> {}/{}", name, target_path, name);
        } else {
            out!(error, "{} (missing from {}/{})", name, target_path, name);
        }
    }
    out!(blank);
    out!(info, "{} {}(s) tracked", tracked.len(), kind);
    Ok(())
}

pub fn run_browse(namespace: &str, project_root: &str, search: Option<&str>) -> Result<()> {
    let registries = crate::config::resolve_all_registries(project_root);
    let reg_config = registries.get(namespace)
        .ok_or_else(|| anyhow::anyhow!(
            "Registry '{}' not configured. Run: mdigitalcn registry add {} <source>", namespace, namespace
        ))?;

    let token = crate::config::resolve_auth(reg_config.auth.as_deref());
    let client = GitHubClient::from_source(&reg_config.source, token)?;

    let sp = crate::output::spinner(&format!("Fetching {} registry...", namespace));
    let registry = client.fetch_registry(RegistryKind::Component)?;
    sp.finish_and_clear();

    let items: Vec<_> = if let Some(query) = search {
        registry.search(RegistryKind::Component, query)
    } else {
        registry.items(RegistryKind::Component).iter().collect()
    };

    if items.is_empty() {
        out!(info, "No items found in {}", namespace);
        return Ok(());
    }

    out!(header, "{} registry ({} items)", namespace, items.len());

    let mut categories: HashMap<&str, Vec<&&super::RegistryItem>> = HashMap::new();
    for item in &items {
        categories.entry(item.category.as_str()).or_default().push(item);
    }

    let mut sorted: Vec<_> = categories.into_iter().collect();
    sorted.sort_by_key(|(cat, _)| *cat);

    for (cat, entries) in &sorted {
        out!(blank);
        println!("  {}", cat.to_uppercase().dimmed());
        for entry in entries {
            let extra = if entry.dependencies.is_empty() {
                String::new()
            } else {
                format!(" {}", format!("[needs: {}]", entry.dependencies.join(", ")).dimmed())
            };
            println!("    {:<24} {}{}", entry.name.bold(), entry.description.dimmed(), extra);
        }
    }

    out!(blank);
    out!(hint, "mdigitalcn <type> add {}/{}", namespace, "<name>");
    Ok(())
}

/// Overlay a registry template onto an already-scaffolded project directory.
///
/// Behavior depends on `overwrite`:
/// - `false` (default, fresh scaffold): Wipes src/ and app/, replaces with template.
/// - `true` (re-apply / existing project): Merges template files on top.
///   Writes template files, keeps user files that don't conflict.
///   Inspired by Nx's `generateFiles()` overwrite strategy.
pub fn overlay_template(
    template_name: &str,
    project_dir: &str,
    no_cache: bool,
    overwrite: bool,
) -> Result<super::TemplateManifest> {
    let kind = RegistryKind::Template;
    let (client, cache) = get_client_and_cache();
    let registry = fetch_registry(&client, &cache, kind, no_cache)?;

    let item = registry.find(kind, template_name)
        .ok_or_else(|| {
            let available: Vec<&str> = registry.items(kind).iter()
                .map(|i| i.name.as_str()).collect();
            anyhow::anyhow!(
                "Template '{}' not found.\nAvailable: {}\nRun `mdigitalcn template list` to see all.",
                template_name, available.join(", ")
            )
        })?;

    let template_path = item.path.clone();
    let display = item.display_name.clone();

    out!(info, "Applying template: {}", display);

    let sp = crate::output::spinner("Fetching template manifest...");
    let manifest = client.fetch_template_manifest(kind, &template_path)?;
    sp.finish_and_clear();

    let sp = crate::output::spinner("Listing template files...");
    let files = client.list_remote_tree(kind, &template_path)?;
    sp.finish_and_clear();

    // Download template files to a staging dir first (atomic)
    let staging = format!("{}/.mdigitalcn_staging", project_dir);
    let staging_path = Path::new(&staging);
    if staging_path.exists() {
        std::fs::remove_dir_all(staging_path).ok();
    }

    let sp = crate::output::spinner(&format!("Downloading {} template files...", files.len()));
    let written = client.download_template(kind, &template_path, &files, staging_path)?;
    sp.finish_and_clear();

    let project_path = Path::new(project_dir);

    if overwrite {
        // Merge mode: write template files on top, keep non-conflicting user files.
        // Only overwrite files that exist in the template.
        let mut overwrote = 0usize;
        let mut created = 0usize;
        let mut skipped_dirs: std::collections::HashSet<String> = std::collections::HashSet::new();

        merge_staged_into(staging_path, project_path, &mut overwrote, &mut created, &mut skipped_dirs)?;
        std::fs::remove_dir_all(staging_path).ok();

        out!(success, "Merged {} — {} new, {} overwritten, user files preserved",
            display, created, overwrote);
    } else {
        // Fresh mode (default): wipe scaffold app code, replace with template
        let src_dir = project_path.join("src");
        let app_dir = project_path.join("app");
        if src_dir.exists() {
            std::fs::remove_dir_all(&src_dir)?;
        }
        if app_dir.exists() {
            std::fs::remove_dir_all(&app_dir)?;
        }

        move_dir_contents(staging_path, project_path)?;
        std::fs::remove_dir_all(staging_path).ok();

        out!(success, "Applied {} ({} files)", display, written.len());
    }

    // Merge template deps into existing package.json
    merge_template_deps(project_path, &manifest)?;

    // Patch index.html entry point: scaffold uses src/main.tsx, templates use src/app/main.tsx
    patch_entry_point(project_path)?;

    if !manifest.requires.layouts.is_empty()
        || !manifest.requires.widgets.is_empty()
        || !manifest.requires.components.is_empty()
    {
        out!(blank);
        out!(info, "Recommended registry items:");
        if !manifest.requires.layouts.is_empty() {
            out!(step, "Layouts: {}", manifest.requires.layouts.join(", "));
        }
        if !manifest.requires.widgets.is_empty() {
            out!(step, "Widgets: {}", manifest.requires.widgets.join(", "));
        }
        if !manifest.requires.components.is_empty() {
            out!(step, "Components: {}", manifest.requires.components.join(", "));
        }
    }

    Ok(manifest)
}

/// Recursively merge staged template files into project, preserving user files.
fn merge_staged_into(
    from: &Path,
    to: &Path,
    overwrote: &mut usize,
    created: &mut usize,
    _skipped_dirs: &mut std::collections::HashSet<String>,
) -> Result<()> {
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());

        if entry.file_type()?.is_dir() {
            if !target.exists() {
                std::fs::create_dir_all(&target)?;
            }
            merge_staged_into(&entry.path(), &target, overwrote, created, _skipped_dirs)?;
            std::fs::remove_dir_all(entry.path()).ok();
        } else {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            if target.exists() {
                *overwrote += 1;
            } else {
                *created += 1;
            }
            std::fs::rename(entry.path(), &target)?;
        }
    }
    Ok(())
}

fn patch_entry_point(project_path: &Path) -> Result<()> {
    let index_html = project_path.join("index.html");
    if !index_html.exists() { return Ok(()); }

    let content = std::fs::read_to_string(&index_html)?;
    if content.contains("src/app/main.tsx") { return Ok(()); }

    // Replace src/main.tsx with src/app/main.tsx (FSD template convention)
    let patched = content.replace(
        "src=\"/src/main.tsx\"",
        "src=\"/src/app/main.tsx\"",
    );
    if patched != content {
        std::fs::write(&index_html, &patched)?;
    }
    Ok(())
}

fn move_dir_contents(from: &Path, to: &Path) -> Result<()> {
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            if !target.exists() {
                std::fs::create_dir_all(&target)?;
            }
            move_dir_contents(&entry.path(), &target)?;
            std::fs::remove_dir_all(entry.path()).ok();
        } else {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::rename(entry.path(), &target)?;
        }
    }
    Ok(())
}

fn merge_template_deps(project_path: &Path, manifest: &super::TemplateManifest) -> Result<()> {
    let pkg_path = project_path.join("package.json");
    if !pkg_path.exists() { return Ok(()); }

    let content = std::fs::read_to_string(&pkg_path)?;
    let mut pkg: serde_json::Value = serde_json::from_str(&content)?;

    // Merge npmDependencies into dependencies (skip if scaffold already has the dep)
    if !manifest.npm_dependencies.is_empty() {
        let deps = pkg.get_mut("dependencies")
            .and_then(|d| d.as_object_mut());
        if let Some(deps) = deps {
            for (k, v) in &manifest.npm_dependencies {
                if !deps.contains_key(k) {
                    deps.insert(k.clone(), serde_json::Value::String(v.clone()));
                }
            }
        } else {
            pkg["dependencies"] = serde_json::Value::Object(
                manifest.npm_dependencies.iter()
                    .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                    .collect(),
            );
        }
    }

    // Merge devDependencies (skip if scaffold already has the dep)
    if !manifest.dev_dependencies.is_empty() {
        let dev_deps = pkg.get_mut("devDependencies")
            .and_then(|d| d.as_object_mut());
        if let Some(dev_deps) = dev_deps {
            for (k, v) in &manifest.dev_dependencies {
                if !dev_deps.contains_key(k) {
                    dev_deps.insert(k.clone(), serde_json::Value::String(v.clone()));
                }
            }
        } else {
            pkg["devDependencies"] = serde_json::Value::Object(
                manifest.dev_dependencies.iter()
                    .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                    .collect(),
            );
        }
    }

    std::fs::write(&pkg_path, serde_json::to_string_pretty(&pkg)?)?;
    Ok(())
}

fn resolve_target_path(kind: RegistryKind, project_root: &str) -> String {
    read_project_config(project_root)
        .map(|c| c.paths.path_for(kind).to_string())
        .unwrap_or_else(|_| kind.default_path().to_string())
}

fn interactive_select(registry: &Registry, kind: RegistryKind) -> Result<Vec<String>> {
    let items = registry.items(kind);
    if items.is_empty() {
        return Ok(Vec::new());
    }

    let groups: Vec<(&str, Vec<(String, String)>)> = registry
        .by_category(kind)
        .into_iter()
        .filter(|(cat, _)| !cat.starts_with('_'))
        .map(|(cat, entries)| {
            let labels: Vec<(String, String)> = entries.iter()
                .map(|i| (i.name.clone(), format!("{} -- {}", i.display_name, i.description)))
                .collect();
            (cat, labels)
        })
        .collect();

    let mut display_items: Vec<String> = Vec::new();
    let mut name_map: Vec<Option<String>> = Vec::new();

    for (category, entries) in &groups {
        display_items.push(format!("-- {} --", category.to_uppercase()));
        name_map.push(None);
        for (name, label) in entries {
            display_items.push(format!("  {}", label));
            name_map.push(Some(name.clone()));
        }
    }

    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Select {}s (Space to select, Enter to confirm)", kind))
        .items(&display_items)
        .interact()?;

    Ok(selections.into_iter().filter_map(|i| name_map[i].clone()).collect())
}

fn pf(label: &str, value: &str) {
    out!(step, "{:<16} {}", format!("{}:", label), value);
}

fn print_peer_deps(peer_deps: &HashMap<String, String>, project_root: &str) {
    if peer_deps.is_empty() { return; }

    let pkg_path = Path::new(project_root).join("package.json");
    if !pkg_path.exists() {
        let deps_str: Vec<String> = peer_deps.iter()
            .map(|(k, v)| if v.is_empty() || v == "*" { k.clone() } else { format!("{}@{}", k, v) })
            .collect();
        let pm = crate::utils::detect_package_manager(project_root);
        out!(blank);
        out!(info, "Install peer dependencies:");
        out!(step, "{} add {}", pm, deps_str.join(" "));
        return;
    }

    let content = match std::fs::read_to_string(&pkg_path) {
        Ok(c) => c,
        Err(_) => return,
    };
    let mut pkg: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return,
    };

    let mut added: Vec<String> = Vec::new();

    // Ensure dependencies object exists
    if pkg.get("dependencies").is_none() {
        pkg["dependencies"] = serde_json::Value::Object(serde_json::Map::new());
    }

    if let Some(deps) = pkg.get_mut("dependencies").and_then(|d| d.as_object_mut()) {
        for (k, v) in peer_deps {
            if !deps.contains_key(k) {
                let version = if v.is_empty() || v == "*" {
                    "latest".to_string()
                } else {
                    v.clone()
                };
                deps.insert(k.clone(), serde_json::Value::String(version));
                added.push(k.clone());
            }
        }
    }

    if added.is_empty() { return; }

    match serde_json::to_string_pretty(&pkg) {
        Ok(json) => {
            if std::fs::write(&pkg_path, json).is_ok() {
                out!(blank);
                out!(info, "Added peer dependencies to package.json:");
                for dep in &added {
                    let ver = peer_deps.get(dep).map(|s| s.as_str()).unwrap_or("");
                    if ver.is_empty() || ver == "*" {
                        out!(step, "{}", dep);
                    } else {
                        out!(step, "{}@{}", dep, ver);
                    }
                }
                let pm = crate::utils::detect_package_manager(project_root);
                let pm_bin = pm.split_whitespace().next().unwrap_or("npm");
                out!(hint, "Run: {} install", pm_bin);
            }
        }
        Err(_) => {
            // Fallback: print hint only
            let deps_str: Vec<String> = added.iter()
                .map(|k| {
                    let v = peer_deps.get(k).map(|s| s.as_str()).unwrap_or("");
                    if v.is_empty() || v == "*" { k.clone() } else { format!("{}@{}", k, v) }
                })
                .collect();
            let pm = crate::utils::detect_package_manager(project_root);
            out!(blank);
            out!(info, "Install peer dependencies:");
            out!(step, "{} add {}", pm, deps_str.join(" "));
        }
    }
}
