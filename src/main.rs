#![allow(non_snake_case)]
//
//! STKLR
//
use ansi_term::Colour;
use STKLR::green;
use STKLR::red;
use STKLR::search::utils::SourceTree;
use STKLR::termite;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::collections::HashMap;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Minimum [`std`] prints.
    #[arg(short, long, default_value_t = false)]
    quiet: bool,

    /// Turns on [`the`] [`log`] it's verbose -- not reccommended.
    #[arg(short, long, default_value_t = false)]
    debug: bool,

    /// Show a report on % of stuff that's public etc.
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generates a report, containing: x, y, z.
    Report { path: Option<Vec<String>> },
    /// [`Preview`] mode will print changes to [`std`] but not [`write`] anything to disk.
    Preview { path: Option<Vec<String>> },
    /// Run [`STKLR`] and [`write`] all changes found to disk.
    Fix { path: Option<Vec<String>> },
}

fn main() {
    let cli = Cli::parse();

    if cli.debug {
        termite::setup_logger().unwrap();
    }

    _ = match &cli.command {
        Commands::Report { path } => run_report(path),
        Commands::Preview { path } => run(path, &cli, false, true),
        Commands::Fix { path } => run(path, &cli, true, false),
    };
}
// TODO: what even is this?
#[allow(dead_code)]
fn run_report(_paths: &Option<Vec<String>>) -> Result<()> {
    Ok(())
}
// TODO: break this up. run_report, Preview, Fix
fn run(paths: &Option<Vec<String>>, cli: &Cli, write_mode: bool, preview: bool) -> Result<()> {
    let t1 = std::time::Instant::now();

    let st = {
        if let Some(paths) = paths {
            SourceTree::new_from_paths(paths)
        } else {
            SourceTree::new_from_cwd()
        }
    };

    //NOTE: maybe par iter..?
    for rsc in st.source_files.iter() {
        if !cli.quiet {
            println!(
                "\nProcessing: {}",
                Colour::Blue.paint(rsc.file.display().to_string())
            );
        }
        let new_m = rsc
            .make_adjustments(&rsc.named_idents)
            .into_iter()
            .map(|adj| (adj.line_num, adj.contents))
            // to ignore the fact the adjusted lines come back out-of-order
            .collect::<HashMap<usize, String>>();

        let output = (0..rsc.total_lines)
            .into_iter()
            .map(|n| -> String {
                if let Some(new) = new_m.get(&n) {
                    if !cli.quiet {
                        green!(new, n);
                    }
                    new.to_owned()
                } else {
                    let new = rsc.get(&n).unwrap().contents.to_owned();
                    if !cli.quiet {
                        red!(new, n);
                    }
                    new
                }
            })
            .collect::<Vec<String>>();

        // Write each file
        if write_mode {
            _ = std::fs::write(&rsc.file, output.join("\n"));
        } else if !cli.quiet {
            //TODO: pretty print
        }
    }

    println!(
        "\n\nCOMPLETE!\n{} FILES IN: {}s",
        st.source_files.len(),
        t1.elapsed().as_secs_f64()
    );

    Ok(())
}
