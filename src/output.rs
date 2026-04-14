use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::io::IsTerminal;
use std::sync::OnceLock;
use std::time::Duration;

#[derive(Clone, Copy, PartialEq)]
pub enum Verbosity {
    Quiet,
    Normal,
    Verbose,
}

#[derive(Clone, Copy)]
pub enum Level {
    Success,
    Error,
    Warning,
    Info,
    Step,
    Header,
    Hint,
    Blank,
}


static VERBOSITY: OnceLock<Verbosity> = OnceLock::new();

pub fn init(verbosity: Verbosity) {
    if std::env::var("NO_COLOR").is_ok() || !std::io::stdout().is_terminal() {
        colored::control::set_override(false);
    }
    let _ = VERBOSITY.set(verbosity);
}

fn verbosity() -> Verbosity {
    VERBOSITY.get().copied().unwrap_or(Verbosity::Normal)
}

pub fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.enable_steady_tick(Duration::from_millis(80));
    pb.set_message(msg.to_string());
    pb
}

pub fn _print(level: Level, msg: &str) {
    let v = verbosity();

    match level {
        Level::Error => eprintln!("{} {}", "x".red(), msg.red()),
        Level::Success if v != Verbosity::Quiet => println!("{} {}", "+".green(), msg),
        Level::Warning if v != Verbosity::Quiet => println!("{} {}", "!".yellow(), msg.yellow()),
        Level::Info if v != Verbosity::Quiet => println!("{} {}", ">".cyan(), msg),
        Level::Step if v != Verbosity::Quiet => {
            println!("  {} {}", "-".dimmed(), msg.dimmed())
        }
        Level::Header if v != Verbosity::Quiet => {
            println!("\n  {}", msg.bold())
        }
        Level::Hint if v != Verbosity::Quiet => {
            println!("  {} {}", ">".dimmed(), msg.dimmed())
        }
        Level::Blank if v != Verbosity::Quiet => println!(),
        _ => {}
    }
}

#[macro_export]
macro_rules! out {
    (success, $($arg:tt)*) => {
        $crate::output::_print($crate::output::Level::Success, &format!($($arg)*))
    };
    (error, $($arg:tt)*) => {
        $crate::output::_print($crate::output::Level::Error, &format!($($arg)*))
    };
    (warning, $($arg:tt)*) => {
        $crate::output::_print($crate::output::Level::Warning, &format!($($arg)*))
    };
    (info, $($arg:tt)*) => {
        $crate::output::_print($crate::output::Level::Info, &format!($($arg)*))
    };
    (step, $($arg:tt)*) => {
        $crate::output::_print($crate::output::Level::Step, &format!($($arg)*))
    };
    (header, $($arg:tt)*) => {
        $crate::output::_print($crate::output::Level::Header, &format!($($arg)*))
    };
    (hint, $($arg:tt)*) => {
        $crate::output::_print($crate::output::Level::Hint, &format!($($arg)*))
    };
    (blank) => {
        $crate::output::_print($crate::output::Level::Blank, "")
    };
}
