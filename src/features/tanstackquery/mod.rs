use anyhow::{Context, Result};
use include_dir::{Dir, include_dir};
use std::fs;
use std::path::Path;
use tera;

static TEMPLATES: Dir<'_> =
    include_dir!("$CARGO_MANIFEST_DIR/src/features/tanstackquery/templates");

/// Default TanStack React Query version to add to package.json.
const TANSTACK_QUERY_VERSION: &str = "^5.90.0";

/// Output paths — identical to scaffold's `mdigitalcn init --query tanstack`
const CLIENT_PATH: &str = "src/shared/services/tanstack_query/client.ts";
const API_PATH: &str = "src/shared/services/tanstack_query/api.ts";

pub fn create(project_root: &str) -> Result<()> {
    let project_path = Path::new(project_root);

    // Detect if scaffold already set up tanstack query (user ran --query tanstack)
    if project_path.join(CLIENT_PATH).exists() && project_path.join(API_PATH).exists() {
        out!(step, "TanStack Query already configured (from scaffold)");
        out!(step, "Skipping — files already at src/shared/services/tanstack_query/");
        return Ok(());
    }

    // Verify axios service exists (required dependency)
    let axios_path = project_path.join("src/shared/services/axios/index.ts");
    if !axios_path.exists() {
        out!(warning, "src/shared/services/axios/ not found");
        out!(warning, "TanStack Query API layer imports from '../axios'. Create the axios service first or fix imports manually.");
    }

    let mut tera = tera::Tera::default();

    for file in TEMPLATES.files() {
        if let (Some(path), Some(content)) = (file.path().to_str(), file.contents_utf8()) {
            if path.ends_with(".tera") {
                let name = file
                    .path()
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(path);
                tera.add_raw_template(name, content)
                    .with_context(|| format!("Failed to parse template '{}'", name))?;
            }
        }
    }

    let context = tera::Context::new();

    // Render to same paths as scaffold's --query tanstack
    render_to(&tera, &context, "client.ts.tera", &project_path.join(CLIENT_PATH))?;
    render_to(&tera, &context, "api.ts.tera", &project_path.join(API_PATH))?;

    // Update services/index.ts barrel export if it exists
    update_services_barrel(project_path)?;

    add_dependency(project_path)?;
    Ok(())
}

fn render_to(tera: &tera::Tera, context: &tera::Context, template: &str, output: &Path) -> Result<()> {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory '{}'", parent.display()))?;
    }

    let content = tera
        .render(template, context)
        .with_context(|| format!("Failed to render template '{}'", template))?;

    fs::write(output, content)
        .with_context(|| format!("Failed to write '{}'", output.display()))?;

    out!(step, "Created {}", output.display());
    Ok(())
}

/// Add tanstack_query export to services/index.ts if it exists and doesn't already have it
fn update_services_barrel(project_path: &Path) -> Result<()> {
    let barrel = project_path.join("src/shared/services/index.ts");
    if !barrel.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&barrel)?;
    if content.contains("tanstack_query") {
        return Ok(());
    }

    let updated = format!("{}\nexport * from './tanstack_query'\n", content.trim_end());
    fs::write(&barrel, updated)?;
    out!(step, "Updated src/shared/services/index.ts");
    Ok(())
}

fn add_dependency(project_path: &Path) -> Result<()> {
    let pkg_path = project_path.join("package.json");

    if !pkg_path.exists() {
        out!(
            warning,
            "package.json not found - add @tanstack/react-query manually"
        );
        return Ok(());
    }

    let content = fs::read_to_string(&pkg_path)
        .with_context(|| format!("Failed to read '{}'", pkg_path.display()))?;

    let mut pkg: serde_json::Value =
        serde_json::from_str(&content).context("Failed to parse package.json")?;

    // Check if already installed
    let already_installed = pkg
        .get("dependencies")
        .and_then(|d| d.get("@tanstack/react-query"))
        .is_some()
        || pkg
            .get("devDependencies")
            .and_then(|d| d.get("@tanstack/react-query"))
            .is_some();

    if already_installed {
        out!(step, "@tanstack/react-query already in package.json");
        return Ok(());
    }

    if let Some(deps) = pkg.get_mut("dependencies").and_then(|d| d.as_object_mut()) {
        deps.insert(
            "@tanstack/react-query".to_string(),
            serde_json::json!(TANSTACK_QUERY_VERSION),
        );
    } else {
        pkg["dependencies"] = serde_json::json!({
            "@tanstack/react-query": TANSTACK_QUERY_VERSION
        });
    }

    let updated = serde_json::to_string_pretty(&pkg).context("Failed to serialize package.json")?;

    fs::write(&pkg_path, updated)
        .with_context(|| format!("Failed to write '{}'", pkg_path.display()))?;

    out!(step, "Added @tanstack/react-query to package.json");
    Ok(())
}
