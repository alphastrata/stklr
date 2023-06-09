//! Main controls for the CLI.
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Minimum `std` prints.
    #[arg(short, long, default_value_t = false)]
    pub quiet: bool,

    /// Turns on the log it's verbose -- not reccommended.
    #[arg(short, long, default_value_t = false)]
    pub debug: bool,

    #[command(subcommand)]
    pub command: Commands,
}

impl Default for Cli {
    fn default() -> Self {
        Self {
            quiet: false,
            debug: false,
            command: Commands::Report { path: None },
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// WIP: Generates a report, containing: x, y, z.
    Report { path: Option<Vec<String>> },
    /// Will print changes to the terminal, but not write anything.
    Preview { path: Option<Vec<String>> },
    /// Run the app and write changes found.
    Fix { path: Option<Vec<String>> },
}

impl Cli {
    pub fn init() -> Self {
        Cli::parse()
    }
}
