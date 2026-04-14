use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;

use super::CommandRunner;
use crate::help;

#[derive(Debug, Clone, clap::Args)]
#[command(about = help::LIST_ABOUT)]
#[command(after_help = help::LIST_AFTER_HELP)]
pub struct ListCommand {
    #[command(subcommand)]
    pub command: Option<ListSubcommand>,

    // Issue #15: add --no-cache for consistency
    /// Bypass cache and fetch fresh data
    #[arg(long, help = "Bypass cache and fetch fresh data")]
    pub no_cache: bool,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ListSubcommand {
    /// Show available features and configs
    #[command(about = "Show available features and configs")]
    Features,
}

impl CommandRunner for ListCommand {
    fn run(&self) -> Result<()> {
        match &self.command {
            Some(ListSubcommand::Features) => list_features(),
            None => {
                print_registry_types()?;
                out!(blank);
                print_configured_registries()?;
                out!(blank);
                list_features()?;
                Ok(())
            }
        }
    }
}

fn print_registry_types() -> Result<()> {
    out!(header, "Registry commands");
    out!(blank);

    let types = [
        ("component", "UI primitives"),
        ("widget", "Composed widgets"),
        ("page", "Page templates"),
        ("module", "Module templates"),
        ("layout", "Layout templates"),
        ("template", "Starter projects"),
    ];

    for (name, desc) in types {
        println!("    {:<14} {}", name.bold(), desc.dimmed());
    }

    out!(blank);
    out!(hint, "mdigitalcn <type> list");
    out!(hint, "mdigitalcn <type> add <name>");
    out!(hint, "mdigitalcn init --template vite/saas --name my-app");
    Ok(())
}

fn print_configured_registries() -> Result<()> {
    let registries = crate::config::resolve_all_registries(".");

    out!(header, "Registries");
    out!(blank);

    println!("    {:<16} {}", "mdigitalcn".bold(), "(default)".dimmed());

    for (name, reg) in &registries {
        let auth = if reg.auth.is_some() { " [auth]" } else { "" };
        println!("    {:<16} {}{}", name.bold(), reg.source.dimmed(), auth);
    }

    if !registries.is_empty() {
        out!(blank);
        out!(hint, "mdigitalcn registry browse @name");
    }
    Ok(())
}

fn list_features() -> Result<()> {
    out!(header, "Features and configs");

    out!(blank);
    println!("  {}", "FEATURES".dimmed());
    println!("    {:<14} {}", "tanstackquery".bold(), "TanStack Query".dimmed());

    out!(blank);
    println!("  {}", "CONFIGS".dimmed());
    let configs = [
        ("commitlint", "Conventional commit linting"),
        ("dockerfile", "Docker setup"),
        ("eslint", "ESLint config"),
        ("githubcicd", "GitHub Actions CI/CD"),
        ("gitlabcicd", "GitLab CI/CD"),
        ("husky", "Git hooks + lint-staged"),
        ("nginx", "Nginx config"),
        ("prettier", "Prettier config"),
        ("sentry", "Sentry error tracking"),
        ("tsconfig", "TypeScript config"),
    ];

    for (id, desc) in configs {
        println!("    {:<14} {}", id.bold(), desc.dimmed());
    }

    out!(blank);
    out!(info, "{} items available", 1 + configs.len());
    out!(hint, "mdigitalcn add prettier eslint dockerfile");
    Ok(())
}
