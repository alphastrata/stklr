#![allow(non_snake_case)]
///   _____ _______ _  ___      _____  
///  / ____|__   __| |/ / |    |  __ \
/// | (___    | |  | ' /| |    | |__) |
///  \___ \   | |  |  < | |    |  _  /
///  ____) |  | |  | . \| |____| | \ \
/// |_____/   |_|  |_|\_\______|_|  \_\
///                                    
///                                    
use STKLR::cmd::cli::Cli;
use STKLR::cmd::cli::Commands;
use STKLR::cmd::jobs::{run, run_report};
use STKLR::termite;

use anyhow::Result;
use std::process::Command;

fn main() -> Result<()> {
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
