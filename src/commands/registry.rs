use anyhow::{bail, Result};
use clap::{Args as ClapArgs, Subcommand};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Input, Select};

use super::CommandRunner;
use crate::config::{
    ThirdPartyRegistry,
    has_project_config, read_global_registries, read_project_config,
    write_global_registries, write_project_config,
};
use crate::help;
use crate::registry::helpers;

#[derive(Debug, Clone, ClapArgs)]
#[command(about = help::REGISTRY_ABOUT)]
#[command(after_help = help::REGISTRY_AFTER_HELP)]
pub struct Args {
    #[command(subcommand)]
    pub action: Action,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Action {
    /// Add a third-party registry
    #[command(about = "Add a third-party registry")]
    Add {
        /// Registry namespace (e.g., @acme)
        #[arg(help = "Registry namespace (e.g., @acme)")]
        name: Option<String>,

        /// Source (e.g., github:user/repo)
        #[arg(help = "Source (e.g., github:user/repo)")]
        source: Option<String>,

        /// Configure authentication
        #[arg(long, help = "Configure authentication")]
        auth: bool,

        /// Save to global config instead of project
        #[arg(long, short, help = "Save to global config instead of project")]
        global: bool,
    },

    /// Remove a third-party registry
    #[command(about = "Remove a third-party registry")]
    Remove {
        /// Registry namespace to remove (e.g., @acme)
        #[arg(help = "Registry namespace to remove (e.g., @acme)")]
        name: String,

        /// Remove from global config
        #[arg(long, short, help = "Remove from global config")]
        global: bool,
    },

    /// List all configured registries
    #[command(about = "List all configured registries")]
    List,

    /// Browse items in a third-party registry
    #[command(about = "Browse items in a third-party registry")]
    Browse {
        /// Registry namespace to browse (e.g., @acme)
        #[arg(help = "Registry namespace to browse (e.g., @acme)")]
        name: String,

        /// Search by name, description, or tags
        #[arg(long, short, help = "Search by name, description, or tags")]
        search: Option<String>,

        /// Project root directory
        #[arg(long, default_value = ".", help = "Project root directory")]
        project_root: String,
    },
}

impl CommandRunner for Args {
    fn run(&self) -> Result<()> {
        match &self.action {
            Action::Add { name, source, auth, global } => {
                run_registry_add(name.as_deref(), source.as_deref(), *auth, *global)
            }
            Action::Remove { name, global } => run_registry_remove(name, *global),
            Action::List => run_registry_list(),
            Action::Browse { name, search, project_root } => {
                helpers::run_browse(name, project_root, search.as_deref())
            }
        }
    }
}

fn normalize_namespace(name: &str) -> String {
    if name.starts_with('@') {
        name.to_string()
    } else {
        format!("@{}", name)
    }
}

fn run_registry_add(
    name: Option<&str>,
    source: Option<&str>,
    auth: bool,
    global: bool,
) -> Result<()> {
    let theme = ColorfulTheme::default();

    let namespace = match name {
        Some(n) => normalize_namespace(n),
        None => {
            let input: String = Input::with_theme(&theme)
                .with_prompt("Registry namespace")
                .interact_text()?;
            normalize_namespace(&input)
        }
    };

    let source_str = match source {
        Some(s) => s.to_string(),
        None => {
            Input::with_theme(&theme)
                .with_prompt("Source (github:user/repo)")
                .interact_text()?
        }
    };

    let _ = crate::registry::client::GitHubClient::from_source(&source_str, None)?;

    let auth_method = if auth {
        let options = ["Environment variable", "None"];
        let sel = Select::with_theme(&theme)
            .with_prompt("Authentication method")
            .items(&options)
            .default(0)
            .interact()?;

        match sel {
            0 => {
                let var: String = Input::with_theme(&theme)
                    .with_prompt("Environment variable name")
                    .interact_text()?;
                Some(format!("env:{}", var))
            }
            _ => None,
        }
    } else {
        None
    };

    let registry = ThirdPartyRegistry {
        source: source_str,
        auth: auth_method.clone(),
    };

    let save_to_project = !global && has_project_config(".");

    if save_to_project {
        let mut config = read_project_config(".")?;
        config.registries.insert(namespace.clone(), registry);
        write_project_config(".", &config)?;
        out!(success, "Added {} to project config", namespace);
    } else {
        let mut regs = read_global_registries();
        regs.insert(namespace.clone(), registry);
        write_global_registries(&regs)?;
        out!(success, "Added {} to global config", namespace);
    }

    if let Some(ref auth_ref) = auth_method {
        out!(step, "Auth: {}", auth_ref);
    }
    out!(blank);
    out!(hint, "mdigitalcn registry browse {}", namespace);
    Ok(())
}

fn run_registry_remove(name: &str, global: bool) -> Result<()> {
    let namespace = normalize_namespace(name);

    if global {
        let mut regs = read_global_registries();
        if regs.remove(&namespace).is_some() {
            write_global_registries(&regs)?;
            out!(success, "Removed {} from global config", namespace);
        } else {
            bail!("Registry '{}' not found in global config", namespace);
        }
        return Ok(());
    }

    if has_project_config(".") {
        let mut config = read_project_config(".")?;
        if config.registries.remove(&namespace).is_some() {
            write_project_config(".", &config)?;
            out!(success, "Removed {} from project config", namespace);
            return Ok(());
        }
    }

    let mut regs = read_global_registries();
    if regs.remove(&namespace).is_some() {
        write_global_registries(&regs)?;
        out!(success, "Removed {} from global config", namespace);
        return Ok(());
    }

    bail!("Registry '{}' not found", namespace);
}

fn run_registry_list() -> Result<()> {
    let global = read_global_registries();
    let project = if has_project_config(".") {
        read_project_config(".")
            .map(|c| c.registries)
            .unwrap_or_default()
    } else {
        std::collections::HashMap::new()
    };

    out!(header, "Configured registries");
    out!(blank);

    println!("    {:<16} {}", "mdigitalcn".bold(), "(default)".dimmed());

    if !project.is_empty() {
        out!(blank);
        println!("  {}", "PROJECT".dimmed());
        for (name, reg) in &project {
            let auth = if reg.auth.is_some() { " [auth]" } else { "" };
            println!("    {:<16} {}{}", name.bold(), reg.source.dimmed(), auth);
        }
    }

    if !global.is_empty() {
        out!(blank);
        println!("  {}", "GLOBAL".dimmed());
        for (name, reg) in &global {
            if project.contains_key(name) {
                continue;
            }
            let auth = if reg.auth.is_some() { " [auth]" } else { "" };
            println!("    {:<16} {}{}", name.bold(), reg.source.dimmed(), auth);
        }
    }

    if global.is_empty() && project.is_empty() {
        out!(blank);
        out!(info, "No third-party registries configured");
    }

    out!(blank);
    out!(hint, "mdigitalcn registry add @name github:user/repo");
    Ok(())
}
