#![allow(non_snake_case)]
//
//! STKLR
//
use STKLR::cmd::cli::Cli;
use STKLR::cmd::cli::Commands;
use STKLR::cmd::jobs::{run, run_report};
use STKLR::termite;

use anyhow::Result;

fn main() -> Result<()> {
    //let cli = Cli::parse();
    let cli = Cli::init();

    if cli.debug {
        termite::setup_logger().unwrap();
    }

    _ = match &cli.command {
        Commands::Report { path } => run_report(path),
        Commands::Preview { path } => run(path, &cli),
        Commands::Fix { path } => run(path, &cli),
    };

    Ok(())
}
