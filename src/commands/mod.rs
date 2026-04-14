// ─── Registry command macro (Issue #1) ───────────────────────
// Generates Args, Action, and CommandRunner impl for registry-type commands.
// Replaces 5 identical 64-line files with a single macro invocation each.
macro_rules! define_registry_command {
    ($kind_variant:ident, $about:expr, $after_help:expr) => {
        use anyhow::Result;
        use clap::{Args as ClapArgs, Subcommand};

        use super::CommandRunner;
        use crate::registry::helpers;
        use crate::registry::RegistryKind;

        const KIND: RegistryKind = RegistryKind::$kind_variant;

        #[derive(Debug, Clone, ClapArgs)]
        #[command(about = $about)]
        #[command(after_help = $after_help)]
        pub struct Args {
            #[command(subcommand)]
            pub action: Option<Action>,
        }

        #[derive(Debug, Clone, Subcommand)]
        pub enum Action {
            #[command(about = "Add items from the registry")]
            Add {
                #[arg(value_name = "NAME", help = "Names of items to add")]
                names: Vec<String>,

                #[arg(long, help = "Bypass cache and fetch fresh data")]
                no_cache: bool,

                #[arg(long, default_value = ".", help = "Project root directory")]
                project_root: String,

                #[arg(long, help = "Overwrite existing files")]
                overwrite: bool,
            },

            #[command(about = "List available items in the registry")]
            List {
                #[arg(long, help = "Bypass cache and fetch fresh data")]
                no_cache: bool,

                #[arg(long, short, help = "Filter by category")]
                category: Option<String>,

                #[arg(long, short, help = "Search by name, description, or tags")]
                search: Option<String>,
            },

            #[command(about = "Show detailed info about an item")]
            Info {
                #[arg(help = "Item name to inspect")]
                name: String,

                #[arg(long, help = "Bypass cache and fetch fresh data")]
                no_cache: bool,
            },

            #[command(about = "Show installed items in the current project")]
            Status {
                #[arg(long, default_value = ".", help = "Project root directory")]
                project_root: String,
            },
        }

        impl CommandRunner for Args {
            fn run(&self) -> Result<()> {
                match &self.action {
                    Some(Action::Add { names, no_cache, project_root, overwrite }) => {
                        helpers::run_add(KIND, names, *no_cache, project_root, *overwrite)
                    }
                    Some(Action::List { no_cache, category, search }) => {
                        helpers::run_list(KIND, *no_cache, category.as_deref(), search.as_deref())
                    }
                    Some(Action::Info { name, no_cache }) => {
                        helpers::run_info(KIND, name, *no_cache)
                    }
                    Some(Action::Status { project_root }) => {
                        helpers::run_status(KIND, project_root)
                    }
                    // Issue #9: no subcommand → default to listing
                    None => {
                        helpers::run_list(KIND, false, None, None)
                    }
                }
            }
        }
    };
}

pub mod add;
pub mod component;
pub mod init;
pub mod layout;
pub mod list;
pub mod module;
pub mod page;
pub mod registry;
pub mod template;
pub mod widget;

use anyhow::Result;

pub trait CommandRunner {
    fn run(&self) -> Result<()>;
}
