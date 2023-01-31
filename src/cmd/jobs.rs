//!
//! everything related to handling the sorts of 'jobs'/i.e the 'work' this app does.
//!
use crate::cmd::cli::{Cli, Commands};
use crate::search::utils::{ReportCard, SourceTree};
use crate::testinator::external::test;
use crate::testinator::testinate::grep_tests;
use crate::{green, red, show};

use anyhow::Result;
#[allow(unused_imports)]
use log::{debug, error, info};
use std::collections::HashMap;
use std::iter::zip;
use std::path::PathBuf;
use std::process::Command;

fn control_loop(st: &SourceTree) -> Vec<PathBuf> {
    loop {
        // listen to see if any of our files have been modified
        let modified: Vec<PathBuf> = st
            .source_files
            .iter()
            .filter(|rsc| rsc.file_info.should_process())
            .map(|rsc| rsc.file.clone())
            .collect();

        if !modified.is_empty() {
            return modified;
        }
    }
}
/// WIP:
/// Looks at the current working directory, finds tests within `.rs` files then tries to run those
/// tests, everytime the file they're contained in is modified.
pub fn testinate(_cwd: &Option<Vec<String>>, _cli: &Cli) -> Result<()> {
    let t1 = std::time::Instant::now();

    let mut st = SourceTree::new_from_cwd();

    loop {
        let found_tests = grep_tests(&st).unwrap();

        st = SourceTree::new_from_cwd(); // We need newer timestamps
        eprintln!("rebuild sourcetree");

        let modified = control_loop(&st);

        let failures = found_tests
            .iter()
            .filter(|ft| modified.contains(&ft.file))
            .flat_map(|ft| ft.name.clone())
            .filter_map(|name| Some(test(&name)))
            .collect::<Vec<bool>>();

        let z = zip(found_tests, failures);

        z.for_each(|(ft, failure)| println!("{} {}", ft, show!(failure)));

        println!("\n\nCOMPLETED in {}s", t1.elapsed().as_secs_f64());
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    Ok(())
}

/// Runs preview/fix functionality.
pub fn run(paths: &Option<Vec<String>>, cli: &Cli) -> Result<()> {
    let t1 = std::time::Instant::now();
    let mut change_count = 0;

    let st = SourceTree::setup_tree(paths);

    for rsc in st.source_files.iter() {
        let new_m = rsc
            .make_adjustments(&rsc.named_idents)
            .into_iter()
            .map(|adj| (adj.line_num, adj.contents))
            .collect::<HashMap<usize, String>>();

        //TODO: too many loops.
        let output = (0..rsc.total_lines)
            .into_iter()
            .map(|n| -> String {
                if let Some(new) = new_m.get(&n) {
                    change_count += 1;
                    if !cli.quiet {
                        green!(new, n)
                    }
                    new.to_owned()
                } else {
                    let new = rsc.get(&n).unwrap().contents.to_owned(); //safe unwrap
                    if !cli.quiet {
                        red!(new, n);
                    }
                    new
                }
            })
            .collect::<Vec<String>>();

        if let Commands::Fix { path } = &cli.command {
            std::fs::write(&rsc.file, output.join("\n"))?;
        }
    }
    _ = cargo_fmt();

    println!(
        "\n\nCOMPLETE!\n{} CHANGES ON {} FILES IN: {}s",
        change_count,
        st.source_files.len(),
        t1.elapsed().as_secs_f64()
    );

    Ok(())
}

/// WIP:
/// Runs the report functionality.
pub fn run_report(paths: &Option<Vec<String>>, _cli: &Cli) -> Result<()> {
    let t1 = std::time::Instant::now();

    let st = SourceTree::setup_tree(paths);
    let rc = ReportCard::from_source_tree(st);

    rc.pretty_print();

    println!("\n\nCOMPLETED in {}s", t1.elapsed().as_secs_f64());
    Ok(())
}

pub fn cargo_fmt() -> Result<()> {
    let cmd = Command::new("cargo fmt").output()?;
    dbg!("cargo fmt exit code {}", cmd.status);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::cli::Cli;
    //TODO:
}
