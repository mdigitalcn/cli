pub mod tanstackquery;

use anyhow::{Context as AnyhowContext, Result};
use dialoguer::{theme::ColorfulTheme, MultiSelect};
use include_dir::{Dir, DirEntry, include_dir};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tera::{Context, Tera};

static CONFIGS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/features/configs");

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConfigManifest {
    description: String,
    #[serde(default)]
    npm_deps: Vec<String>,
    #[serde(default)]
    dev_deps: Vec<String>,
}

fn config_metadata() -> HashMap<&'static str, (&'static str, &'static [&'static str])> {
    let mut m = HashMap::new();
    m.insert("prettier", ("Code formatter", &["prettier"][..]));
    m.insert("eslint", ("Linter", &["eslint", "eslint-plugin-react-hooks", "eslint-plugin-react-refresh", "@eslint/js"]));
    m.insert("tsconfig", ("TypeScript configuration", &[]));
    m.insert("husky", ("Git hooks + lint-staged", &["husky", "lint-staged"]));
    m.insert("commitlint", ("Conventional commits", &["@commitlint/cli", "@commitlint/config-conventional"]));
    m.insert("dockerfile", ("Docker setup", &[]));
    m.insert("nginx", ("Nginx config", &[]));
    m.insert("githubcicd", ("GitHub Actions CI/CD", &[]));
    m.insert("gitlabcicd", ("GitLab CI/CD", &[]));
    m.insert("sentry", ("Error tracking", &["@sentry/react"]));
    m
}

pub fn add(project_root: &str, name: &str) -> Result<()> {
    match name {
        "tanstackquery" => tanstackquery::create(project_root)?,
        _ => add_config(project_root, name)?,
    }

    out!(success, "Added {}", name);

    let meta = config_metadata();
    if let Some((_, deps)) = meta.get(name) {
        if !deps.is_empty() {
            let pm = crate::utils::detect_package_manager(project_root);
            let install_flag = if pm.contains("pnpm") || pm.contains("bun") { " -D" } else { " --save-dev" };
            out!(blank);
            out!(info, "Install dependencies:");
            out!(step, "{}{} {}", pm, install_flag, deps.join(" "));
        }
    }

    Ok(())
}

pub fn add_interactive(project_root: &str) -> Result<()> {
    let meta = config_metadata();
    let mut entries: Vec<(&str, &str)> = meta.iter()
        .map(|(name, (desc, _))| (*name, *desc))
        .collect();
    entries.sort_by_key(|(name, _)| *name);

    entries.insert(0, ("tanstackquery", "TanStack Query setup"));

    let labels: Vec<String> = entries.iter()
        .map(|(name, desc)| format!("{:<16} {}", name, desc))
        .collect();

    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select configs to add (Space to select, Enter to confirm)")
        .items(&labels)
        .interact()?;

    if selections.is_empty() {
        out!(info, "Nothing selected");
        return Ok(());
    }

    for idx in selections {
        let (name, _) = entries[idx];
        let sp = crate::output::spinner(&format!("Adding {}...", name));
        add(project_root, name)?;
        sp.finish_and_clear();
    }

    Ok(())
}

fn add_config(project_root: &str, config_name: &str) -> Result<()> {
    let Some(config_dir) = CONFIGS_DIR.get_dir(config_name) else {
        let available: Vec<_> = CONFIGS_DIR
            .dirs()
            .filter_map(|d| d.path().file_name()?.to_str())
            .filter(|name| *name != "git")
            .collect();
        anyhow::bail!("Unknown config '{}'. Available: {}", config_name, available.join(", "));
    };

    let project_path = Path::new(project_root);
    let context = create_context(project_path)?;
    let mut tera = Tera::default();

    load_templates_recursive(&mut tera, config_dir)?;

    for name in tera.get_template_names() {
        let output_path = project_path.join(
            name.strip_prefix(&format!("{}/", config_name))
                .unwrap_or(name)
                .trim_end_matches(".tera"),
        );
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let rendered = tera.render(name, &context)?;
        if !rendered.trim().is_empty() {
            fs::write(&output_path, &rendered)?;
            out!(step, "Created {}", output_path.display());
        }
    }

    Ok(())
}

fn load_templates_recursive(tera: &mut Tera, dir: &Dir) -> Result<()> {
    for entry in dir.entries() {
        match entry {
            DirEntry::File(file) => {
                if let (Some(path), Some(content)) = (file.path().to_str(), file.contents_utf8()) {
                    if path.ends_with(".tera") {
                        tera.add_raw_template(path, content)
                            .with_context(|| format!("Failed to parse template '{}'", path))?;
                    }
                }
            }
            DirEntry::Dir(subdir) => {
                load_templates_recursive(tera, subdir)?;
            }
        }
    }
    Ok(())
}

fn create_context(project_path: &Path) -> Result<Context> {
    let mut context = Context::new();

    let pkg_path = project_path.join("package.json");
    if let Ok(content) = fs::read_to_string(&pkg_path) {
        if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(name) = pkg.get("name").and_then(|n| n.as_str()) {
                context.insert("project_name", name);
            }

            let has_dep = |name: &str| {
                ["dependencies", "devDependencies"]
                    .iter()
                    .any(|k| pkg.get(k).and_then(|d| d.get(name)).is_some())
            };

            let has_react = has_dep("react");
            let has_typescript = has_dep("typescript");
            let has_nextjs = has_dep("next");
            let has_vite = has_dep("vite");

            context.insert("has_react", &has_react);
            context.insert("has_typescript", &has_typescript);
            context.insert("has_nextjs", &has_nextjs);
            context.insert("has_vite", &has_vite);

            let mut project = HashMap::new();
            project.insert("has_react", has_react);
            project.insert("has_typescript", has_typescript);
            project.insert("has_nextjs", has_nextjs);
            project.insert("has_vite", has_vite);
            context.insert("project", &project);
        }
    }

    if let Ok(config) = crate::config::read_project_config(project_path.to_str().unwrap_or(".")) {
        context.insert("framework", &config.framework);
    }

    Ok(context)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write_package_json(dir: &std::path::Path, content: &str) {
        fs::write(dir.join("package.json"), content).unwrap();
    }

    #[test]
    fn test_context_with_react_vite() {
        let dir = tempdir().unwrap();
        write_package_json(dir.path(), r#"{
            "name": "test-app",
            "dependencies": { "react": "^19.0.0" },
            "devDependencies": { "vite": "^6.0.0", "typescript": "^5.7.0" }
        }"#);
        let ctx = create_context(dir.path()).unwrap();
        assert_eq!(ctx.get("has_react").unwrap(), &tera::Value::Bool(true));
        assert_eq!(ctx.get("has_vite").unwrap(), &tera::Value::Bool(true));
        assert_eq!(ctx.get("has_typescript").unwrap(), &tera::Value::Bool(true));
        assert_eq!(ctx.get("has_nextjs").unwrap(), &tera::Value::Bool(false));
        assert_eq!(ctx.get("project_name").unwrap(), &tera::Value::String("test-app".to_string()));
    }

    #[test]
    fn test_context_without_package_json() {
        let dir = tempdir().unwrap();
        let ctx = create_context(dir.path()).unwrap();
        // No package.json → no flags set
        assert!(ctx.get("has_react").is_none());
    }

    #[test]
    fn test_context_nested_project_object() {
        let dir = tempdir().unwrap();
        write_package_json(dir.path(), r#"{
            "name": "test",
            "dependencies": { "react": "^19.0.0", "next": "^15.0.0" }
        }"#);
        let ctx = create_context(dir.path()).unwrap();
        let project = ctx.get("project").unwrap();
        // project is a map with has_react, has_nextjs, etc.
        assert!(project.get("has_react").is_some());
        assert_eq!(project.get("has_react").unwrap(), &tera::Value::Bool(true));
        assert_eq!(project.get("has_nextjs").unwrap(), &tera::Value::Bool(true));
    }

    #[test]
    fn test_add_config_prettier() {
        let dir = tempdir().unwrap();
        write_package_json(dir.path(), r#"{
            "name": "test",
            "dependencies": { "react": "^19.0.0" },
            "devDependencies": { "typescript": "^5.7.0" }
        }"#);
        add_config(dir.path().to_str().unwrap(), "prettier").unwrap();
        assert!(dir.path().join(".prettierrc").exists());
        assert!(dir.path().join(".prettierignore").exists());
        let content = fs::read_to_string(dir.path().join(".prettierrc")).unwrap();
        assert!(content.contains("jsxSingleQuote"));
    }

    #[test]
    fn test_add_config_husky_nested() {
        let dir = tempdir().unwrap();
        write_package_json(dir.path(), r#"{ "name": "test" }"#);
        add_config(dir.path().to_str().unwrap(), "husky").unwrap();
        // Nested dir: .husky/pre-commit must exist
        assert!(dir.path().join(".husky/pre-commit").exists());
        assert!(dir.path().join(".lintstagedrc").exists());
    }

    #[test]
    fn test_add_config_unknown() {
        let dir = tempdir().unwrap();
        let result = add_config(dir.path().to_str().unwrap(), "nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown config"));
    }

    #[test]
    fn test_config_metadata_complete() {
        let meta = config_metadata();
        let expected = ["prettier", "eslint", "tsconfig", "husky", "commitlint",
            "dockerfile", "nginx", "githubcicd", "gitlabcicd", "sentry"];
        for name in expected {
            assert!(meta.contains_key(name), "Missing config: {}", name);
        }
    }
}
