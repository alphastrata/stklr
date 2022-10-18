#![allow(non_snake_case)]
///   _____ _______ _  ___      _____  
///  / ____|__   __| |/ / |    |  __ \
/// | (___    | |  | ' /| |    | |__) |
///  \___ \   | |  |  < | |    |  _  /
///  ____) |  | |  | . \| |____| | \ \
/// |_____/   |_|  |_|\_\______|_|  \_\
///                                    
///                                    
use STKLR::{
    cmd::{
        cli::{Cli, Commands},
        jobs::{run, run_report, testinate},
    },
    termite,
};

use anyhow::Result;

fn main() -> Result<()> {
    let cli = Cli::init();

    if cli.debug {
        termite::setup_logger().unwrap();
    }

    _ = match &cli.command {
        Commands::Report { path } => run_report(path, &cli),
        Commands::Preview { path } => run(path, &cli),
        Commands::Fix { path } => run(path, &cli),
        Commands::Test{ path } => testinate(path, &cli),
    };

    Ok(())
}
