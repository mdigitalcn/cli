use crate::config::{write_project_config, ProjectConfig};
use anyhow::Result;
use clap::ValueEnum;
use colored::Colorize;
use include_dir::{Dir, DirEntry, include_dir};
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Instant;
use tera::{Context as TeraContext, Tera};

static VITE_TEMPLATES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/scaffold/vite");
static NEXTJS_TEMPLATES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/scaffold/nextjs");
static WEBVIEW_TEMPLATES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/scaffold/webview");
static PWA_TEMPLATES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/scaffold/pwa");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserChoices {
    pub project_name: String,
    pub template: Template,
    pub tanstack_router: bool,
    pub query: QueryChoice,
    pub cicd: CiCdChoice,
    pub uikit: bool,
    pub git_repo: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum)]
pub enum Template {
    Vite,
    #[value(name = "nextjs")]
    NextJs,
    Webview,
    Pwa,
}

#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum)]
pub enum CiCdChoice {
    #[value(name = "gitlab")]
    GitlabCicd,
    #[value(name = "github")]
    GithubCicd,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum)]
pub enum QueryChoice {
    #[value(name = "rtk")]
    RtkQuery,
    #[value(name = "tanstack")]
    TanstackQuery,
    None,
}

pub fn generate(choices: &UserChoices) -> Result<()> {
    generate_inner(choices, false)
}

pub fn generate_quiet(choices: &UserChoices) -> Result<()> {
    generate_inner(choices, true)
}

fn generate_inner(choices: &UserChoices, quiet: bool) -> Result<()> {
    let start = Instant::now();
    let template_name = format!("{:?}", choices.template).to_lowercase();

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb.set_message(format!("Loading {} template...", template_name));

    let engine = create_template_engine(&template_name)?;
    let context = build_context(choices);
    let file_count = scaffold_atomic(&engine, &context, &choices.project_name, &pb)?;

    pb.set_message("Writing project config...");
    let cicd = match choices.cicd {
        CiCdChoice::GitlabCicd => Some("gitlab"),
        CiCdChoice::GithubCicd => Some("github"),
        CiCdChoice::None => None,
    };
    let mut config = ProjectConfig::new(&template_name, cicd);
    if choices.uikit {
        config.features.push("uikit".to_string());
    }
    write_project_config(&choices.project_name, &config)?;

    pb.finish_and_clear();

    if quiet {
        out!(success, "Base scaffold ready ({} files)", file_count + 1);
        return Ok(());
    }

    let total_files = file_count + 1;
    let elapsed = start.elapsed().as_secs_f64();

    out!(blank);
    out!(success, "Created {} in {:.2}s", choices.project_name, elapsed);
    out!(blank);

    let template_label = match choices.template {
        Template::Vite => "Vite + React",
        Template::NextJs => "Next.js",
        Template::Webview => "Webview (Tauri)",
        Template::Pwa => "PWA",
    };
    let query_label = match choices.query {
        QueryChoice::RtkQuery => "RTK Query",
        QueryChoice::TanstackQuery => "TanStack Query",
        QueryChoice::None => "None",
    };
    let cicd_label = match choices.cicd {
        CiCdChoice::GithubCicd => "GitHub Actions",
        CiCdChoice::GitlabCicd => "GitLab CI/CD",
        CiCdChoice::None => "None",
    };

    println!("  {:<14} {}", "Template:".dimmed(), template_label);
    println!("  {:<14} {}", "Query:".dimmed(), query_label);
    if choices.tanstack_router {
        println!("  {:<14} TanStack Router", "Router:".dimmed());
    }
    if choices.uikit {
        println!("  {:<14} mdigitalcn Kit (82 components)", "UIKit:".dimmed());
    }
    println!("  {:<14} {}", "CI/CD:".dimmed(), cicd_label);
    println!("  {:<14} {}", "Files:".dimmed(), total_files);

    out!(blank);
    out!(info, "Next steps:");
    println!();
    println!("   {} {}", "$".dimmed(), format!("cd {}", choices.project_name).bold());
    println!("   {} {}", "$".dimmed(), "pnpm install".bold());
    println!("   {} {}", "$".dimmed(), "pnpm dev".bold());
    out!(blank);

    Ok(())
}

fn build_context(choices: &UserChoices) -> TeraContext {
    let mut context = TeraContext::new();
    let template_name = format!("{:?}", choices.template).to_lowercase();

    let cicd = match choices.cicd {
        CiCdChoice::GitlabCicd => Some("gitlab"),
        CiCdChoice::GithubCicd => Some("github"),
        CiCdChoice::None => None,
    };

    let query = match choices.query {
        QueryChoice::RtkQuery => Some("rtk"),
        QueryChoice::TanstackQuery => Some("tanstack"),
        QueryChoice::None => None,
    };

    context.insert("project_name", &choices.project_name);
    context.insert("template", &template_name);
    context.insert("use_tailwind", &true);
    context.insert("use_tanstack_router", &choices.tanstack_router);
    context.insert("query", &query);
    context.insert("use_cicd", &cicd);
    context.insert("use_uikit", &choices.uikit);

    context
}

fn create_template_engine(template: &str) -> Result<Tera> {
    let mut tera = Tera::default();

    let template_dir = match template {
        "vite" => Some(&VITE_TEMPLATES),
        "nextjs" => Some(&NEXTJS_TEMPLATES),
        "webview" => Some(&WEBVIEW_TEMPLATES),
        "pwa" => Some(&PWA_TEMPLATES),
        _ => None,
    };

    if let Some(dir) = template_dir {
        load_templates_from_dir(&mut tera, dir)?;
    }

    Ok(tera)
}

fn load_templates_from_dir(tera: &mut Tera, dir: &Dir) -> Result<()> {
    for entry in dir.entries() {
        match entry {
            DirEntry::File(file) => {
                if let Some((path_str, content)) = file.path().to_str()
                    .filter(|p| p.ends_with(".tera"))
                    .zip(file.contents_utf8())
                {
                    tera.add_raw_template(path_str, content)?;
                }
            }
            DirEntry::Dir(subdir) => {
                load_templates_from_dir(tera, subdir)?;
            }
        }
    }
    Ok(())
}

fn scaffold_atomic(
    engine: &Tera,
    context: &TeraContext,
    project_name: &str,
    pb: &ProgressBar,
) -> Result<usize> {
    let final_path = Path::new(project_name);
    let temp_path_str = format!("{}.mdigitalcn_tmp", project_name);
    let temp_path = Path::new(&temp_path_str);

    if temp_path.exists() {
        std::fs::remove_dir_all(temp_path).ok();
    }

    let result = scaffold(engine, context, &temp_path_str, pb);

    match result {
        Ok(file_count) => {
            if final_path.exists() {
                std::fs::remove_dir_all(final_path)?;
            }
            std::fs::rename(temp_path, final_path)?;
            Ok(file_count)
        }
        Err(e) => {
            if temp_path.exists() {
                std::fs::remove_dir_all(temp_path).ok();
            }
            Err(e)
        }
    }
}

fn scaffold(
    engine: &Tera,
    context: &TeraContext,
    project_name: &str,
    pb: &ProgressBar,
) -> Result<usize> {
    let output_path = Path::new(project_name);
    std::fs::create_dir_all(output_path)?;

    let template_names: Vec<_> = engine
        .get_template_names()
        .filter(|name| !should_skip_template(name, context))
        .collect();

    let total = template_names.len();

    pb.set_length(total as u64);
    pb.set_position(0);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:35.cyan/blue}] {pos}/{len} {msg:.dim}")
            .unwrap()
            .progress_chars("##-"),
    );

    for template_name in &template_names {
        let display_name = template_name.trim_end_matches(".tera");
        pb.set_message(display_name.to_string());
        render_file(engine, context, template_name, output_path)?;
        pb.inc(1);
    }

    Ok(total)
}

fn should_skip_template(template_name: &str, context: &TeraContext) -> bool {
    let use_cicd = context.get("use_cicd").and_then(|v| v.as_str());
    let use_tanstack_router = context
        .get("use_tanstack_router")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let query = context.get("query").and_then(|v| v.as_str());

    match template_name {
        ".github/workflows/ci-cd.yml.tera" => use_cicd != Some("github"),
        ".gitlab-ci.yml.tera" => use_cicd != Some("gitlab"),
        "src/routes/__root.tsx.tera" | "src/routes/index.tsx.tera" => !use_tanstack_router,
        "src/shared/services/rtk_query/index.ts.tera"
        | "src/shared/services/rtk_query/store.ts.tera"
        | "src/modules/example_module/model/slice.ts.tera" => query != Some("rtk"),
        "src/shared/services/tanstack_query/client.ts.tera"
        | "src/shared/services/tanstack_query/api.ts.tera"
        | "src/shared/services/tanstack_query/index.ts.tera" => query != Some("tanstack"),
        _ => false,
    }
}

fn render_file(
    engine: &Tera,
    context: &TeraContext,
    template_name: &str,
    output_dir: &Path,
) -> Result<()> {
    let output_name = template_name.trim_end_matches(".tera");
    let output_path = output_dir.join(output_name);

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = engine.render(template_name, context)?;
    std::fs::write(&output_path, content)?;

    Ok(())
}
