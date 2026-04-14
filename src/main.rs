#[macro_use]
mod output;
mod commands;
mod config;
mod features;
mod help;
mod registry;
mod scaffold;
mod utils;

use crate::commands::CommandRunner;
use crate::output::Verbosity;
use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

#[derive(Parser)]
#[command(name = "mdigitalcn")]
#[command(about = help::MAIN_ABOUT)]
#[command(long_about = help::MAIN_LONG_ABOUT)]
#[command(version)]
#[command(styles = help::STYLES)]
#[command(help_template = help::MAIN_HELP_TEMPLATE)]
#[command(after_help = help::MAIN_AFTER_HELP)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Suppress all output except errors
    #[arg(short, long, global = true, help = "Suppress all output except errors")]
    pub quiet: bool,

    /// Show detailed output
    #[arg(short, long, global = true, help = "Show detailed output")]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(about = help::CREATE_ABOUT)]
    Init(commands::init::Args),

    #[command(about = "Manage UI components from the registry")]
    Component(commands::component::Args),

    #[command(about = "Manage composed widgets from the registry")]
    Widget(commands::widget::Args),

    #[command(about = "Manage page templates from the registry")]
    Page(commands::page::Args),

    #[command(about = "Manage module templates from the registry")]
    Module(commands::module::Args),

    #[command(about = "Manage layout templates from the registry")]
    Layout(commands::layout::Args),

    #[command(about = help::TEMPLATE_ABOUT)]
    Template(commands::template::Args),

    #[command(about = help::REGISTRY_ABOUT)]
    Registry(commands::registry::Args),

    #[command(about = help::ADD_ABOUT)]
    Add(commands::add::Args),

    #[command(about = help::LIST_ABOUT)]
    List(commands::list::ListCommand),
}

impl CommandRunner for Command {
    fn run(&self) -> Result<()> {
        match self {
            Command::Init(args) => args.run(),
            Command::Component(args) => args.run(),
            Command::Widget(args) => args.run(),
            Command::Page(args) => args.run(),
            Command::Module(args) => args.run(),
            Command::Layout(args) => args.run(),
            Command::Template(args) => args.run(),
            Command::Registry(args) => args.run(),
            Command::Add(args) => args.run(),
            Command::List(cmd) => cmd.run(),
        }
    }
}

fn main() {
    let cli = Cli::parse();

    let verbosity = match (cli.quiet, cli.verbose) {
        (true, _) => Verbosity::Quiet,
        (_, true) => Verbosity::Verbose,
        _ => Verbosity::Normal,
    };
    output::init(verbosity);

    if let Err(e) = cli.command.run() {
        eprintln!();
        out!(error, "{}", e);

        let causes: Vec<_> = e.chain().skip(1).collect();
        if !causes.is_empty() {
            eprintln!();
            for cause in &causes {
                eprintln!("  {} {}", "Caused by:".dimmed(), cause);
            }
        }

        if !cli.verbose && causes.is_empty() {
            eprintln!();
            eprintln!("  {} Run with {} for more details", "->".dimmed(), "--verbose".bold());
        }

        eprintln!();
        std::process::exit(1);
    }
}
