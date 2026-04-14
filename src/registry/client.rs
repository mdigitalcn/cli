use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::{Registry, RegistryItem, RegistryKind, TemplateManifest};

fn make_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(10))
        .timeout_read(Duration::from_secs(30))
        .build()
}

pub struct GitHubClient {
    owner: String,
    branch: String,
    repo_override: Option<String>,
    token: Option<String>,
    agent: ureq::Agent,
}

impl GitHubClient {
    pub fn from_config(config: &crate::config::RegistryConfig) -> Self {
        let token = std::env::var("MDIGITALCN_GITHUB_TOKEN").ok();
        Self {
            owner: config.owner.clone(),
            branch: config.branch.clone(),
            repo_override: config.repo.clone(),
            token,
            agent: make_agent(),
        }
    }

    pub fn from_source(source: &str, token: Option<String>) -> anyhow::Result<Self> {
        let effective_token = token.or_else(|| std::env::var("MDIGITALCN_GITHUB_TOKEN").ok());

        if let Some(gh) = source.strip_prefix("github:") {
            let (repo_path, branch) = match gh.split_once('#') {
                Some((path, br)) => (path, br.to_string()),
                None => (gh, "main".to_string()),
            };

            let (owner, repo) = repo_path.split_once('/')
                .ok_or_else(|| anyhow::anyhow!(
                    "Invalid source: '{}'. Expected github:owner/repo", source
                ))?;

            Ok(Self {
                owner: owner.to_string(),
                branch,
                repo_override: Some(repo.to_string()),
                token: effective_token,
                agent: make_agent(),
            })
        } else {
            anyhow::bail!("Unsupported source: '{}'. Use github:owner/repo", source)
        }
    }

    fn repo_for(&self, kind: RegistryKind) -> String {
        self.repo_override.clone().unwrap_or_else(|| kind.repo().to_string())
    }

    pub fn fetch_registry(&self, kind: RegistryKind) -> Result<Registry> {
        let repo = self.repo_for(kind);
        let url = format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}",
            self.owner, repo, self.branch, "registry.json"
        );

        let body = self.get_text(&url)
            .with_context(|| format!("Failed to fetch {} registry", kind))?;

        let registry: Registry = serde_json::from_str(&body)
            .with_context(|| format!("Failed to parse {} registry", kind))?;

        Ok(registry)
    }

    pub fn download_item(
        &self,
        kind: RegistryKind,
        item: &RegistryItem,
        target_dir: &Path,
    ) -> Result<Vec<PathBuf>> {
        let repo = self.repo_for(kind);
        let item_dir = target_dir.join(&item.name);
        std::fs::create_dir_all(&item_dir)
            .with_context(|| format!("Failed to create dir: {}", item_dir.display()))?;

        let mut written = Vec::new();

        for file_name in &item.files {
            let url = format!(
                "https://raw.githubusercontent.com/{}/{}/{}/{}/{}",
                self.owner, repo, self.branch, item.path, file_name
            );

            let content = self.get_text(&url)
                .with_context(|| format!("Failed to fetch {}/{}", item.path, file_name))?;

            let dest = item_dir.join(file_name);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&dest, &content)
                .with_context(|| format!("Failed to write: {}", dest.display()))?;

            written.push(dest);
        }

        Ok(written)
    }

    pub fn download_foundation(
        &self,
        kind: RegistryKind,
        item: &RegistryItem,
        target_dir: &Path,
    ) -> Result<Vec<PathBuf>> {
        let repo = self.repo_for(kind);
        let mut written = Vec::new();

        let dest_base = if item.name == "_hooks" {
            target_dir.join("hooks")
        } else {
            target_dir.to_path_buf()
        };
        std::fs::create_dir_all(&dest_base)
            .with_context(|| format!("Failed to create dir: {}", dest_base.display()))?;

        for file_name in &item.files {
            let url = format!(
                "https://raw.githubusercontent.com/{}/{}/{}/{}/{}",
                self.owner, repo, self.branch, item.path, file_name
            );

            let content = self.get_text(&url)
                .with_context(|| format!("Failed to fetch {}/{}", item.path, file_name))?;

            let dest = dest_base.join(file_name);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&dest, &content)
                .with_context(|| format!("Failed to write: {}", dest.display()))?;

            written.push(dest);
        }

        Ok(written)
    }

    pub fn fetch_template_manifest(
        &self,
        kind: RegistryKind,
        template_path: &str,
    ) -> Result<TemplateManifest> {
        let repo = self.repo_for(kind);
        let url = format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}/template.json",
            self.owner, repo, self.branch, template_path
        );
        let body = self.get_text(&url)
            .with_context(|| format!("Failed to fetch template.json for {}", template_path))?;
        let manifest: TemplateManifest = serde_json::from_str(&body)
            .with_context(|| "Failed to parse template.json")?;
        Ok(manifest)
    }

    pub fn list_remote_tree(
        &self,
        kind: RegistryKind,
        path_prefix: &str,
    ) -> Result<Vec<String>> {
        let repo = self.repo_for(kind);
        let url = format!(
            "https://api.github.com/repos/{}/{}/git/trees/{}?recursive=1",
            self.owner, repo, self.branch
        );
        let body = self.get_text(&url)?;
        let tree: serde_json::Value = serde_json::from_str(&body)?;

        let prefix = format!("{}/", path_prefix.trim_end_matches('/'));
        let files: Vec<String> = tree["tree"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter(|entry| entry["type"].as_str() == Some("blob"))
                    .filter_map(|entry| entry["path"].as_str())
                    .filter(|p| p.starts_with(&prefix))
                    .map(|p| p.strip_prefix(&prefix).unwrap_or(p).to_string())
                    .filter(|p| p != "template.json")
                    .collect()
            })
            .unwrap_or_default();

        if files.is_empty() {
            anyhow::bail!("No files found in template path: {}", path_prefix);
        }

        Ok(files)
    }

    pub fn download_template(
        &self,
        kind: RegistryKind,
        template_path: &str,
        files: &[String],
        target_dir: &Path,
    ) -> Result<Vec<PathBuf>> {
        let repo = self.repo_for(kind);
        std::fs::create_dir_all(target_dir)?;

        let mut written = Vec::new();

        for file_name in files {
            let url = format!(
                "https://raw.githubusercontent.com/{}/{}/{}/{}/{}",
                self.owner, repo, self.branch, template_path, file_name
            );

            let content = self.get_text(&url)
                .with_context(|| format!("Failed to fetch {}/{}", template_path, file_name))?;

            let dest = target_dir.join(file_name);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&dest, &content)?;
            written.push(dest);
        }

        Ok(written)
    }

    fn get_text(&self, url: &str) -> Result<String> {
        let mut req = self.agent.get(url).set("User-Agent", "mdigitalcn-cli");

        if let Some(token) = &self.token {
            req = req.set("Authorization", &format!("Bearer {}", token));
        }

        let response = req.call().map_err(|e| match e {
            ureq::Error::Status(404, _) => anyhow::anyhow!("Not found: {}", url),
            ureq::Error::Status(403, _) => anyhow::anyhow!(
                "Rate limited. Set MDIGITALCN_GITHUB_TOKEN for higher limits"
            ),
            ureq::Error::Status(code, _) => anyhow::anyhow!("HTTP {}: {}", code, url),
            other => anyhow::anyhow!("Request failed: {}", other),
        })?;

        response
            .into_string()
            .with_context(|| format!("Failed to read response from: {}", url))
    }
}
