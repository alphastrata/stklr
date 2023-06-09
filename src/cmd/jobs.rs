//!
//! everything related to handling the sorts of 'jobs'/i.e the 'work' this app does.
//!
#![allow(unreachable_code)]
use crate::cmd::cli::{Cli, Commands};
use crate::green;
use crate::search::utils::{ReportCard, SourceTree};

use anyhow::Result;
use std::collections::HashMap;
use std::process::Command;

/// Runs preview/fix functionality.
pub fn run(paths: &Option<Vec<String>>, cli: &Cli) -> Result<()> {
    let t1 = std::time::Instant::now();
    let mut change_count = 0;

    let mut st = SourceTree::setup_tree(paths);

    for rsc in st.source_files.iter_mut() {
        let named_idents = &rsc.named_idents.clone(); // double borrow solve.
        let new_m = rsc
            .make_adjustments(named_idents)
            .map(|adj| (adj.line_num, adj.contents))
            .collect::<HashMap<usize, String>>();

        let output = (0..rsc.total_lines)
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
                        //TODO: --show-changes and --show-in-context
                        // red!(new, n);
                    }
                    new
                }
            })
            .collect::<Vec<String>>();

        if let Commands::Fix { path: _ } = &cli.command {
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
mod tests {}
