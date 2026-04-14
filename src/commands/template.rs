use anyhow::Result;
use clap::{Args as ClapArgs, Subcommand};

use super::CommandRunner;
use crate::help;
use crate::registry::helpers;
use crate::registry::RegistryKind;

const KIND: RegistryKind = RegistryKind::Template;

#[derive(Debug, Clone, ClapArgs)]
#[command(about = help::TEMPLATE_ABOUT)]
#[command(after_help = help::TEMPLATE_AFTER_HELP)]
pub struct Args {
    #[command(subcommand)]
    pub action: Option<Action>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Action {
    /// List available starter templates
    #[command(about = "List available starter templates")]
    List {
        #[arg(long, help = "Bypass cache and fetch fresh data")]
        no_cache: bool,

        #[arg(long, short, help = "Filter by category")]
        category: Option<String>,

        #[arg(long, short, help = "Search by name, description, or tags")]
        search: Option<String>,
    },

    /// Show detailed info about a template
    #[command(about = "Show detailed info about a template")]
    Info {
        #[arg(help = "Template name to inspect (e.g., vite/dashboard)")]
        name: String,

        #[arg(long, help = "Bypass cache and fetch fresh data")]
        no_cache: bool,
    },
}

impl CommandRunner for Args {
    fn run(&self) -> Result<()> {
        match &self.action {
            Some(Action::List { no_cache, category, search }) => {
                helpers::run_list(KIND, *no_cache, category.as_deref(), search.as_deref())
            }
            Some(Action::Info { name, no_cache }) => {
                helpers::run_info(KIND, name, *no_cache)
            }
            // No subcommand: default to listing
            None => {
                helpers::run_list(KIND, false, None, None)
            }
        }
    }
}
