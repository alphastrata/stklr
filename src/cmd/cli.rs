use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Minimum [`std`] prints.
    #[arg(short, long, default_value_t = false)]
    pub quiet: bool,

    /// Turns on [`the`] [`log`] it's verbose -- not reccommended.
    #[arg(short, long, default_value_t = false)]
    pub debug: bool,

    /// Show a report on % of stuff that's public etc.
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generates a report, containing: x, y, z.
    Report { path: Option<Vec<String>> },
    /// [`Preview`] mode will print changes to [`std`] but not [`write`] anything to disk.
    Preview { path: Option<Vec<String>> },
    /// Run [`STKLR`] and [`write`] all changes found to disk.
    Fix { path: Option<Vec<String>> },
}

impl Cli {
    pub fn init() -> Self {
        Cli::parse()
    }
}
