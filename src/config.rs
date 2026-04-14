use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::registry::RegistryKind;

const CONFIG_FILE_NAME: &str = ".mdigitalcn.json";
const CONFIG_VERSION: &str = "1.0";
const GLOBAL_REGISTRIES_FILE: &str = "registries.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub version: String,
    pub framework: String,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cicd: Option<String>,
    #[serde(default)]
    pub paths: ProjectPaths,
    #[serde(default)]
    pub generated: GeneratedAssets,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry: Option<RegistryConfig>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub registries: HashMap<String, ThirdPartyRegistry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThirdPartyRegistry {
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    pub owner: String,
    #[serde(default = "default_branch")]
    pub branch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
}

fn default_branch() -> String { "main".to_string() }

impl RegistryConfig {
    pub fn default_mdigitalcn() -> Self {
        Self {
            owner: "mdigitalcn".to_string(),
            branch: "main".to_string(),
            repo: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectPaths {
    #[serde(default = "default_layouts_path")]
    pub layouts: String,
    #[serde(default = "default_pages_path")]
    pub pages: String,
    #[serde(default = "default_widgets_path")]
    pub widgets: String,
    #[serde(default = "default_components_path")]
    pub components: String,
    #[serde(default = "default_modules_path")]
    pub modules: String,
    #[serde(default = "default_templates_path")]
    pub templates: String,
    #[serde(default = "default_shared_path")]
    pub shared: String,
}

impl ProjectPaths {
    pub fn path_for(&self, kind: RegistryKind) -> &str {
        match kind {
            RegistryKind::Component => &self.components,
            RegistryKind::Widget => &self.widgets,
            RegistryKind::Page => &self.pages,
            RegistryKind::Module => &self.modules,
            RegistryKind::Layout => &self.layouts,
            RegistryKind::Template => &self.templates,
        }
    }
}

impl RegistryKind {
    pub fn default_path(&self) -> &str {
        match self {
            Self::Component => "src/components/ui",
            Self::Widget => "src/widgets",
            Self::Page => "src/pages",
            Self::Module => "src/modules",
            Self::Layout => "src/layouts",
            Self::Template => "src/templates",
        }
    }
}

impl Default for ProjectPaths {
    fn default() -> Self {
        Self {
            layouts: default_layouts_path(),
            pages: default_pages_path(),
            widgets: default_widgets_path(),
            components: default_components_path(),
            modules: default_modules_path(),
            templates: default_templates_path(),
            shared: default_shared_path(),
        }
    }
}

fn default_layouts_path() -> String { "src/layouts".to_string() }
fn default_pages_path() -> String { "src/pages".to_string() }
fn default_widgets_path() -> String { "src/widgets".to_string() }
fn default_components_path() -> String { "src/components/ui".to_string() }
fn default_modules_path() -> String { "src/modules".to_string() }
fn default_templates_path() -> String { "src/templates".to_string() }
fn default_shared_path() -> String { "src/shared".to_string() }

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GeneratedAssets {
    #[serde(default)]
    pub layouts: Vec<String>,
    #[serde(default)]
    pub pages: Vec<String>,
    #[serde(default)]
    pub widgets: Vec<String>,
    #[serde(default)]
    pub components: Vec<String>,
    #[serde(default)]
    pub modules: Vec<String>,
    #[serde(default)]
    pub templates: Vec<String>,
}

impl GeneratedAssets {
    pub fn tracked(&self, kind: RegistryKind) -> &[String] {
        match kind {
            RegistryKind::Component => &self.components,
            RegistryKind::Widget => &self.widgets,
            RegistryKind::Page => &self.pages,
            RegistryKind::Module => &self.modules,
            RegistryKind::Layout => &self.layouts,
            RegistryKind::Template => &self.templates,
        }
    }

    pub fn tracked_mut(&mut self, kind: RegistryKind) -> &mut Vec<String> {
        match kind {
            RegistryKind::Component => &mut self.components,
            RegistryKind::Widget => &mut self.widgets,
            RegistryKind::Page => &mut self.pages,
            RegistryKind::Module => &mut self.modules,
            RegistryKind::Layout => &mut self.layouts,
            RegistryKind::Template => &mut self.templates,
        }
    }
}

impl ProjectConfig {
    pub fn new(framework: &str, cicd: Option<&str>) -> Self {
        Self {
            version: CONFIG_VERSION.to_string(),
            framework: framework.to_string(),
            features: Vec::new(),
            cicd: cicd.map(|s| s.to_string()),
            paths: ProjectPaths::default(),
            generated: GeneratedAssets::default(),
            registry: Some(RegistryConfig::default_mdigitalcn()),
            registries: HashMap::new(),
        }
    }

    pub fn effective_registry(&self) -> RegistryConfig {
        self.registry.clone().unwrap_or_else(RegistryConfig::default_mdigitalcn)
    }
}

pub fn read_project_config(project_root: &str) -> Result<ProjectConfig> {
    let config_path = Path::new(project_root).join(CONFIG_FILE_NAME);
    let content = std::fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read {}", config_path.display()))?;
    let config: ProjectConfig = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse {}", config_path.display()))?;

    let major = config.version.split('.').next()
        .and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);
    let expected_major = CONFIG_VERSION.split('.').next()
        .and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);

    if major != expected_major {
        anyhow::bail!(
            "Config version mismatch: found '{}', expected '{}.x'. Re-run `mdigitalcn init`.",
            config.version, expected_major,
        );
    }

    Ok(config)
}

pub fn write_project_config(project_root: &str, config: &ProjectConfig) -> Result<()> {
    let config_path = Path::new(project_root).join(CONFIG_FILE_NAME);
    let content = serde_json::to_string_pretty(config)?;
    std::fs::write(&config_path, content)
        .with_context(|| format!("Failed to write {}", config_path.display()))?;
    Ok(())
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct GlobalRegistries {
    #[serde(default)]
    pub registries: HashMap<String, ThirdPartyRegistry>,
}

fn global_registries_path() -> std::path::PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    Path::new(&home).join(".mdigitalcn").join(GLOBAL_REGISTRIES_FILE)
}

pub fn read_global_registries() -> HashMap<String, ThirdPartyRegistry> {
    let path = global_registries_path();
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return HashMap::new(),
    };
    serde_json::from_str::<GlobalRegistries>(&content)
        .map(|d| d.registries)
        .unwrap_or_default()
}

pub fn write_global_registries(regs: &HashMap<String, ThirdPartyRegistry>) -> Result<()> {
    let path = global_registries_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let data = GlobalRegistries { registries: regs.clone() };
    let content = serde_json::to_string_pretty(&data)?;
    std::fs::write(&path, content)
        .with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

pub fn resolve_all_registries(project_root: &str) -> HashMap<String, ThirdPartyRegistry> {
    let mut result = read_global_registries();
    if let Ok(config) = read_project_config(project_root) {
        result.extend(config.registries);
    }
    result
}

pub fn resolve_auth(auth: Option<&str>) -> Option<String> {
    auth.and_then(|s| {
        s.strip_prefix("env:").and_then(|var| std::env::var(var).ok())
    })
}

pub fn has_project_config(project_root: &str) -> bool {
    Path::new(project_root).join(CONFIG_FILE_NAME).exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_config() {
        let config = ProjectConfig::new("vite", Some("github"));
        assert_eq!(config.version, "1.0");
        assert_eq!(config.framework, "vite");
        assert_eq!(config.cicd, Some("github".to_string()));
        assert!(config.registry.is_some());
    }

    #[test]
    fn test_default_registry() {
        let reg = RegistryConfig::default_mdigitalcn();
        assert_eq!(reg.owner, "mdigitalcn");
        assert_eq!(reg.branch, "main");
        assert!(reg.repo.is_none());
    }

    #[test]
    fn test_write_and_read_config() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_str().unwrap();
        let config = ProjectConfig::new("vite", None);
        write_project_config(root, &config).unwrap();
        let read = read_project_config(root).unwrap();
        assert_eq!(read.framework, "vite");
    }

    #[test]
    fn test_tracked_assets() {
        let mut assets = GeneratedAssets::default();
        assets.tracked_mut(RegistryKind::Component).push("button".to_string());
        assert_eq!(assets.tracked(RegistryKind::Component), &["button"]);
        assert!(assets.tracked(RegistryKind::Widget).is_empty());
    }

    #[test]
    fn test_paths_for_kind() {
        let paths = ProjectPaths::default();
        assert_eq!(paths.path_for(RegistryKind::Component), "src/components/ui");
        assert_eq!(paths.path_for(RegistryKind::Page), "src/pages");
    }
}
