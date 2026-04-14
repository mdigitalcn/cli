use crate::help;
use crate::scaffold::{self, CiCdChoice, QueryChoice, Template, UserChoices};
use crate::registry::TemplateManifest;
use anyhow::Result;
use clap::Args as ClapArgs;
use colored::*;
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

use super::CommandRunner;

static VALID_CHARS_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap());

const RESERVED_NAMES: &[&str] = &[
    "con", "prn", "aux", "nul",
    "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8", "com9",
    "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
    "node_modules", ".git", ".gitignore", "package", "test",
];

#[derive(Debug, Clone, ClapArgs)]
#[command(about = help::CREATE_ABOUT)]
#[command(long_about = help::CREATE_LONG_ABOUT)]
#[command(after_help = help::CREATE_AFTER_HELP)]
pub struct Args {
    /// Project name (required for non-interactive mode)
    #[arg(short, long, help = "Project name (required for non-interactive mode)")]
    pub name: Option<String>,

    /// Use a starter template (e.g., vite/dashboard)
    #[arg(long, value_name = "TEMPLATE", help = "Use a starter template (e.g., vite/dashboard)")]
    pub template: Option<String>,

    /// Framework template to use
    #[arg(long, value_enum, help = "Framework template to use")]
    pub framework: Option<Template>,

    /// Add TanStack Router
    #[arg(long, default_value = "false", help = "Add TanStack Router")]
    pub router: bool,

    // Issue #7: default changed from "rtk" to "none"
    /// API query library (rtk, tanstack, or none)
    #[arg(long, value_enum, default_value = "none", help = "API query library (rtk, tanstack, or none)")]
    pub query: QueryChoice,

    /// CI/CD platform (github, gitlab, or none)
    #[arg(short, long, value_enum, default_value = "none", help = "CI/CD platform (github, gitlab, or none)")]
    pub cicd: CiCdChoice,

    /// Include mdigitalcn Kit (82 ready-to-use components)
    #[arg(long, help = "Include mdigitalcn Kit (82 ready-to-use components)")]
    pub uikit: bool,

    /// Overwrite existing directory
    #[arg(long, help = "Overwrite existing directory")]
    pub force: bool,

    /// Bypass cache and fetch fresh data
    #[arg(long, help = "Bypass cache and fetch fresh data")]
    pub no_cache: bool,

    /// Merge template files on top of existing code (keep non-conflicting user files)
    #[arg(long, help = "Merge template on top of existing code instead of replacing")]
    pub overwrite: bool,
}

enum InitResult {
    Scaffold(UserChoices),
    Done, // Template flow already handled everything
}

impl CommandRunner for Args {
    fn run(&self) -> Result<()> {
        if let Some(template_name) = &self.template {
            return run_with_template(self, template_name);
        }

        let result = if let Some(name) = &self.name {
            InitResult::Scaffold(from_args(name.clone(), self)?)
        } else if self.framework.is_some() {
            // Issue #12: user passed flags but forgot --name
            anyhow::bail!(
                "--name is required in non-interactive mode.\n\n\
                Example: mdigitalcn init --name my-app --framework vite"
            );
        } else {
            prompt_for_choices()?
        };

        if let InitResult::Scaffold(choices) = result {
            scaffold::generate(&choices)?;
        }
        Ok(())
    }
}

fn run_with_template(args: &Args, template_name: &str) -> Result<()> {
    let start = std::time::Instant::now();

    // Infer framework from template path prefix
    let framework = if let Some(fw) = &args.framework {
        fw.clone()
    } else {
        infer_framework(template_name)?
    };

    // Determine project name
    let project_name = args.name.clone().unwrap_or_else(|| {
        template_name.split('/').last().unwrap_or("my-app").to_string()
    });

    validate_non_interactive(&project_name, args.force)?;

    out!(blank);
    out!(header, "mdigitalcn Project Builder");

    // Step 1: Scaffold base project
    let choices = UserChoices {
        project_name: project_name.clone(),
        template: framework,
        tanstack_router: false,
        query: QueryChoice::None,
        cicd: args.cicd.clone(),
        uikit: args.uikit,
        git_repo: None,
    };

    out!(blank);
    scaffold::generate_quiet(&choices)?;

    // Step 2: Overlay template
    let manifest = crate::registry::helpers::overlay_template(
        template_name,
        &project_name,
        args.no_cache,
        args.overwrite,
    )?;

    // Step 3: Update .mdigitalcn.json with template info
    if let Ok(mut config) = crate::config::read_project_config(&project_name) {
        config.features.push(format!("template:{}", template_name));
        let _ = crate::config::write_project_config(&project_name, &config);
    }

    // Issue #2: extracted summary
    let elapsed = start.elapsed().as_secs_f64();
    print_template_result(&choices.template, &manifest, &project_name, elapsed);

    Ok(())
}

fn infer_framework(template_name: &str) -> Result<Template> {
    if template_name.starts_with("vite/") {
        Ok(Template::Vite)
    } else if template_name.starts_with("nextjs/") {
        Ok(Template::NextJs)
    } else if template_name.starts_with("webview/") {
        Ok(Template::Webview)
    } else if template_name.starts_with("pwa/") {
        Ok(Template::Pwa)
    } else {
        anyhow::bail!(
            "Cannot infer framework from template '{}'. Use --framework vite|nextjs|webview|pwa",
            template_name
        )
    }
}

fn from_args(project_name: String, args: &Args) -> Result<UserChoices> {
    validate_non_interactive(&project_name, args.force)?;

    let template = args.framework.clone().unwrap_or(Template::Vite);

    let tanstack_router = if matches!(template, Template::NextJs) {
        false
    } else {
        args.router
    };

    Ok(UserChoices {
        project_name,
        template,
        tanstack_router,
        query: args.query.clone(),
        cicd: args.cicd.clone(),
        uikit: args.uikit,
        git_repo: None,
    })
}

fn prompt_for_choices() -> Result<InitResult> {
    out!(blank);
    out!(header, "mdigitalcn Project Builder");

    let theme = ColorfulTheme::default();

    out!(blank);
    println!("  {}", "PROJECT".dimmed());
    let project_name = prompt_project_name(&theme)?;

    out!(blank);
    println!("  {}", "FRAMEWORK".dimmed());
    let template = prompt_template(&theme)?;

    // Offer starter templates for the chosen framework
    out!(blank);
    println!("  {}", "STARTER TEMPLATE".dimmed());
    let fw_key = match template {
        Template::Vite => "vite",
        Template::NextJs => "nextjs",
        Template::Webview => "webview",
        Template::Pwa => "pwa",
    };

    if let Some(chosen_template) = prompt_starter_template(&theme, fw_key)? {
        validate_interactive(&project_name).map_err(|e| anyhow::anyhow!("{}", e))?;
        run_interactive_with_template(&project_name, &chosen_template, template)?;
        return Ok(InitResult::Done);
    }

    out!(blank);
    println!("  {}", "FEATURES".dimmed());
    let tanstack_router = if !matches!(template, Template::NextJs) {
        Confirm::with_theme(&theme)
            .with_prompt(help::TANSTACK_ROUTER_PROMPT)
            .default(false)
            .interact()?
    } else {
        false
    };
    let query = prompt_query(&theme)?;
    let uikit = Confirm::with_theme(&theme)
        .with_prompt("Include mdigitalcn Kit? (82 ready-to-use components)")
        .default(false)
        .interact()?;
    let cicd = prompt_cicd(&theme)?;

    out!(blank);

    Ok(InitResult::Scaffold(UserChoices {
        project_name,
        template,
        tanstack_router,
        query,
        cicd,
        uikit,
        git_repo: None,
    }))
}

fn prompt_project_name(theme: &ColorfulTheme) -> Result<String> {
    loop {
        let name: String = Input::with_theme(theme)
            .with_prompt(help::PROJECT_NAME_PROMPT)
            .default("my_app".to_string())
            .interact()?;

        match validate_interactive(&name) {
            Ok(_) => return Ok(name),
            Err(e) => {
                println!("{} {}", "x".bright_red(), e.bright_red());
                println!("{} Please try a different name\n", ">".bright_yellow());
            }
        }
    }
}

fn prompt_template(theme: &ColorfulTheme) -> Result<Template> {
    let options = vec![
        help::VITE_DESC,
        help::NEXTJS_DESC,
        help::WEBVIEW_DESC,
        help::PWA_DESC,
    ];

    let selection = Select::with_theme(theme)
        .with_prompt(help::TEMPLATE_PROMPT)
        .items(&options)
        .default(0)
        .interact()?;

    Ok(match selection {
        1 => Template::NextJs,
        2 => Template::Webview,
        3 => Template::Pwa,
        _ => Template::Vite,
    })
}

/// Show a picker with available starter templates for the framework.
/// Returns None if user picks "Blank project" (no template).
fn prompt_starter_template(theme: &ColorfulTheme, framework: &str) -> Result<Option<String>> {
    let templates = match crate::registry::helpers::list_templates_for_framework(framework, false) {
        Ok(t) if !t.is_empty() => t,
        _ => {
            println!("  {}", "Starter templates unavailable (offline or no cache). Continuing with blank project.".dimmed());
            return Ok(None);
        }
    };

    let mut options: Vec<String> = vec!["Blank project (no starter)".to_string()];
    for (_, display) in &templates {
        options.push(display.clone());
    }

    let selection = Select::with_theme(theme)
        .with_prompt("Choose a starter template")
        .items(&options)
        .default(0)
        .interact()?;

    if selection == 0 {
        return Ok(None);
    }

    Ok(Some(templates[selection - 1].0.clone()))
}

/// Run the template overlay flow from interactive mode.
fn run_interactive_with_template(
    project_name: &str,
    template_name: &str,
    framework: Template,
) -> Result<()> {
    let start = std::time::Instant::now();

    let theme = ColorfulTheme::default();
    let uikit = Confirm::with_theme(&theme)
        .with_prompt("Include mdigitalcn Kit? (82 ready-to-use components)")
        .default(false)
        .interact()?;

    // Issue #8: ask CI/CD in template flow too
    let cicd = prompt_cicd(&theme)?;

    let choices = UserChoices {
        project_name: project_name.to_string(),
        template: framework,
        tanstack_router: false,
        query: QueryChoice::None,
        cicd,
        uikit,
        git_repo: None,
    };

    out!(blank);
    scaffold::generate_quiet(&choices)?;

    let manifest = crate::registry::helpers::overlay_template(
        template_name,
        project_name,
        false,
        false, // fresh scaffold, not overwrite mode
    )?;

    if let Ok(mut config) = crate::config::read_project_config(project_name) {
        config.features.push(format!("template:{}", template_name));
        let _ = crate::config::write_project_config(project_name, &config);
    }

    // Issue #2: extracted summary
    let elapsed = start.elapsed().as_secs_f64();
    print_template_result(&choices.template, &manifest, project_name, elapsed);

    Ok(())
}

// ─── Issue #2: Extracted shared summary ──────────────────────

fn print_template_result(
    template: &Template,
    manifest: &TemplateManifest,
    project_name: &str,
    elapsed: f64,
) {
    out!(blank);
    out!(success, "Created {} in {:.1}s", project_name, elapsed);
    out!(blank);

    let fw_label = match template {
        Template::Vite => "Vite + React",
        Template::NextJs => "Next.js",
        Template::Webview => "Webview (Tauri)",
        Template::Pwa => "PWA",
    };
    println!("  {:<16} {}", "Framework:".dimmed(), fw_label);
    println!("  {:<16} {}", "Template:".dimmed(), manifest.display_name);
    println!("  {:<16} {}", "Architecture:".dimmed(), manifest.architecture);

    print_next_steps(project_name);
}

fn print_next_steps(project_name: &str) {
    out!(blank);
    out!(info, "Next steps:");
    println!();
    println!("   {} {}", "$".dimmed(), format!("cd {}", project_name).bold());
    println!("   {} {}", "$".dimmed(), "pnpm install".bold());
    println!("   {} {}", "$".dimmed(), "pnpm dev".bold());
    out!(blank);
}

fn prompt_query(theme: &ColorfulTheme) -> Result<QueryChoice> {
    let options = vec![
        help::RTK_QUERY_DESC,
        help::TANSTACK_QUERY_DESC,
        help::NO_QUERY_DESC,
    ];

    let selection = Select::with_theme(theme)
        .with_prompt(help::QUERY_PROMPT)
        .items(&options)
        .default(0)
        .interact()?;

    Ok(match selection {
        1 => QueryChoice::TanstackQuery,
        2 => QueryChoice::None,
        _ => QueryChoice::RtkQuery,
    })
}

fn prompt_cicd(theme: &ColorfulTheme) -> Result<CiCdChoice> {
    let options = vec![
        help::GITHUB_CICD_DESC,
        help::GITLAB_CICD_DESC,
        help::NO_CICD_DESC,
    ];

    let selection = Select::with_theme(theme)
        .with_prompt(help::CICD_PROMPT)
        .items(&options)
        .default(2)
        .interact()?;

    Ok(match selection {
        0 => CiCdChoice::GithubCicd,
        1 => CiCdChoice::GitlabCicd,
        _ => CiCdChoice::None,
    })
}

fn check_project_name(name: &str) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Project name cannot be empty".to_string());
    }
    if name.len() > 42 {
        return Err("Project name is too long (max 42 characters)".to_string());
    }
    if !VALID_CHARS_REGEX.is_match(name) {
        return Err("Only letters, numbers, hyphens, and underscores allowed".to_string());
    }
    if !name.chars().next().is_some_and(|c| c.is_alphanumeric()) {
        return Err("Must start with a letter or number".to_string());
    }
    if RESERVED_NAMES.contains(&name.to_lowercase().as_str()) {
        return Err(format!("'{}' is a reserved name", name));
    }
    Ok(())
}

fn validate_interactive(name: &str) -> Result<(), String> {
    check_project_name(name)?;

    if Path::new(name).exists() {
        let theme = ColorfulTheme::default();
        println!("Directory '{}' already exists", name.bright_yellow());

        let overwrite = Confirm::with_theme(&theme)
            .with_prompt("Overwrite existing directory?")
            .default(false)
            .interact()
            .map_err(|e| format!("Confirmation failed: {}", e))?;

        if !overwrite {
            return Err("Cancelled".to_string());
        }
    }

    Ok(())
}

fn validate_non_interactive(name: &str, force: bool) -> Result<()> {
    check_project_name(name).map_err(|e| anyhow::anyhow!("{}", e))?;

    if Path::new(name).exists() && !force {
        anyhow::bail!("Directory '{}' already exists. Use --force to overwrite.", name);
    }

    Ok(())
}
