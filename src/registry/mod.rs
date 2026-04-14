pub mod cache;
pub mod client;
pub mod helpers;

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegistryKind {
    Component,
    Widget,
    Page,
    Module,
    Layout,
    Template,
}

impl RegistryKind {
    pub fn repo(&self) -> &str {
        match self {
            Self::Component => "uikit",
            Self::Widget => "widgets",
            Self::Page => "pages",
            Self::Module => "modules",
            Self::Layout => "layouts",
            Self::Template => "templates",
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Component => "component",
            Self::Widget => "widget",
            Self::Page => "page",
            Self::Module => "module",
            Self::Layout => "layout",
            Self::Template => "template",
        }
    }
}

impl fmt::Display for RegistryKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

pub fn group_by_namespace(names: &[String]) -> (Vec<String>, HashMap<String, Vec<String>>) {
    let mut defaults = Vec::new();
    let mut namespaced: HashMap<String, Vec<String>> = HashMap::new();
    for name in names {
        if let Some((ns, item)) = name.strip_prefix('@').and_then(|rest| rest.split_once('/')) {
            namespaced.entry(format!("@{}", ns)).or_default().push(item.to_string());
            continue;
        }
        defaults.push(name.clone());
    }
    (defaults, namespaced)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryItem {
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub description: String,
    pub category: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub path: String,
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default, rename = "peerDependencies")]
    pub peer_dependencies: HashMap<String, String>,
    #[serde(default)]
    pub framework: Option<String>,
    #[serde(default)]
    pub architecture: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateManifest {
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub description: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub framework: String,
    #[serde(default)]
    pub architecture: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub requires: TemplateRequires,
    #[serde(default, rename = "npmDependencies")]
    pub npm_dependencies: HashMap<String, String>,
    #[serde(default, rename = "devDependencies")]
    pub dev_dependencies: HashMap<String, String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TemplateRequires {
    #[serde(default)]
    pub layouts: Vec<String>,
    #[serde(default)]
    pub widgets: Vec<String>,
    #[serde(default)]
    pub components: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry {
    #[serde(default)]
    pub version: String,
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    #[serde(default)]
    items: Vec<RegistryItem>,
    #[serde(default)]
    components: Vec<RegistryItem>,
}

impl Registry {
    pub fn items(&self, kind: RegistryKind) -> &[RegistryItem] {
        if !self.items.is_empty() {
            return &self.items;
        }
        if kind == RegistryKind::Component {
            return &self.components;
        }
        &self.items
    }

    pub fn find(&self, kind: RegistryKind, name: &str) -> Option<&RegistryItem> {
        self.items(kind).iter().find(|i| i.name == name)
    }

    pub fn search(&self, kind: RegistryKind, query: &str) -> Vec<&RegistryItem> {
        let q = query.to_lowercase();
        self.items(kind).iter()
            .filter(|item| {
                item.name.contains(&q)
                    || item.display_name.to_lowercase().contains(&q)
                    || item.description.to_lowercase().contains(&q)
                    || item.tags.iter().any(|t| t.to_lowercase().contains(&q))
                    || item.category.to_lowercase().contains(&q)
            })
            .collect()
    }

    pub fn by_category(&self, kind: RegistryKind) -> Vec<(&str, Vec<&RegistryItem>)> {
        let mut categories: HashMap<&str, Vec<&RegistryItem>> = HashMap::new();
        for item in self.items(kind) {
            categories.entry(item.category.as_str()).or_default().push(item);
        }
        let mut sorted: Vec<_> = categories.into_iter().collect();
        sorted.sort_by_key(|(cat, _)| *cat);
        sorted
    }

    pub fn resolve_deps(&self, kind: RegistryKind, names: &[String]) -> Vec<String> {
        let mut resolved: Vec<String> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        for name in names {
            self.resolve_recursive(kind, name, &mut resolved, &mut seen);
        }
        resolved
    }

    pub fn collect_npm_deps(&self, kind: RegistryKind, names: &[String]) -> HashMap<String, String> {
        let mut npm_deps = HashMap::new();
        for name in names {
            if let Some(item) = self.find(kind, name) {
                for dep in &item.dependencies {
                    if self.find(kind, dep).is_none() {
                        npm_deps.insert(dep.clone(), "*".to_string());
                    }
                }
            }
        }
        npm_deps
    }

    fn resolve_recursive(
        &self,
        kind: RegistryKind,
        name: &str,
        resolved: &mut Vec<String>,
        seen: &mut HashSet<String>,
    ) {
        if seen.contains(name) { return; }
        seen.insert(name.to_string());

        if let Some(item) = self.find(kind, name) {
            for dep in &item.dependencies {
                self.resolve_recursive(kind, dep, resolved, seen);
            }
            resolved.push(name.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item(name: &str, deps: &[&str]) -> RegistryItem {
        RegistryItem {
            name: name.to_string(),
            display_name: name.to_string(),
            description: format!("A {} item", name),
            category: "test".to_string(),
            tags: vec!["test".to_string()],
            path: format!("src/{}", name),
            files: vec!["index.tsx".to_string()],
            dependencies: deps.iter().map(|s| s.to_string()).collect(),
            peer_dependencies: HashMap::new(),
            framework: None,
            architecture: None,
        }
    }

    fn make_registry(items: Vec<RegistryItem>) -> Registry {
        Registry {
            version: "1.0".to_string(),
            schema: None,
            items,
            components: Vec::new(),
        }
    }

    #[test]
    fn test_resolve_deps_chain() {
        let reg = make_registry(vec![
            make_item("a", &["b"]),
            make_item("b", &["c"]),
            make_item("c", &[]),
        ]);
        let result = reg.resolve_deps(RegistryKind::Widget, &["a".to_string()]);
        assert_eq!(result, vec!["c", "b", "a"]);
    }

    #[test]
    fn test_resolve_deps_circular() {
        let reg = make_registry(vec![
            make_item("a", &["b"]),
            make_item("b", &["a"]),
        ]);
        let result = reg.resolve_deps(RegistryKind::Widget, &["a".to_string()]);
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"a".to_string()));
        assert!(result.contains(&"b".to_string()));
    }

    #[test]
    fn test_resolve_deps_dedup() {
        let reg = make_registry(vec![
            make_item("a", &["c"]),
            make_item("b", &["c"]),
            make_item("c", &[]),
        ]);
        let result = reg.resolve_deps(RegistryKind::Widget, &["a".to_string(), "b".to_string()]);
        assert_eq!(result.iter().filter(|n| n.as_str() == "c").count(), 1);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_collect_npm_deps() {
        let mut item = make_item("datepicker", &["date-fns", "helper"]);
        item.peer_dependencies.insert("react".to_string(), "^19".to_string());
        let reg = make_registry(vec![
            item,
            make_item("helper", &[]),
        ]);
        let npm = reg.collect_npm_deps(RegistryKind::Widget, &["datepicker".to_string()]);
        // date-fns is not in registry → npm dep
        assert!(npm.contains_key("date-fns"));
        // helper IS in registry → not an npm dep
        assert!(!npm.contains_key("helper"));
    }

    #[test]
    fn test_search() {
        let mut item = make_item("date-picker", &[]);
        item.description = "A date selection component".to_string();
        item.tags = vec!["date".to_string(), "form".to_string()];
        let reg = make_registry(vec![item, make_item("button", &[])]);

        assert_eq!(reg.search(RegistryKind::Widget, "date").len(), 1);
        assert_eq!(reg.search(RegistryKind::Widget, "selection").len(), 1);
        assert_eq!(reg.search(RegistryKind::Widget, "form").len(), 1);
        assert_eq!(reg.search(RegistryKind::Widget, "xyz").len(), 0);
    }

    #[test]
    fn test_by_category() {
        let mut a = make_item("a", &[]);
        a.category = "forms".to_string();
        let mut b = make_item("b", &[]);
        b.category = "display".to_string();
        let mut c = make_item("c", &[]);
        c.category = "forms".to_string();
        let reg = make_registry(vec![a, b, c]);

        let cats = reg.by_category(RegistryKind::Widget);
        assert_eq!(cats.len(), 2);
        assert_eq!(cats[0].0, "display");
        assert_eq!(cats[1].0, "forms");
        assert_eq!(cats[1].1.len(), 2);
    }

    #[test]
    fn test_group_by_namespace() {
        let names = vec![
            "button".to_string(),
            "@acme/card".to_string(),
            "input".to_string(),
            "@acme/modal".to_string(),
            "@other/badge".to_string(),
        ];
        let (defaults, namespaced) = group_by_namespace(&names);
        assert_eq!(defaults, vec!["button", "input"]);
        assert_eq!(namespaced["@acme"], vec!["card", "modal"]);
        assert_eq!(namespaced["@other"], vec!["badge"]);
    }

    #[test]
    fn test_find_components_key() {
        let reg = Registry {
            version: "1.0".to_string(),
            schema: None,
            items: Vec::new(),
            components: vec![make_item("button", &[])],
        };
        assert!(reg.find(RegistryKind::Component, "button").is_some());
        assert!(reg.find(RegistryKind::Widget, "button").is_none());
    }
}
