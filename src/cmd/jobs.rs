use crate::cmd::cli::*;
use crate::search::utils::SourceTree;
use crate::{green, red};

use ansi_term::Colour;
use anyhow::Result;
use std::collections::HashMap;

#[allow(dead_code)]
pub fn run_report(_paths: &Option<Vec<String>>) -> Result<()> {
    Ok(())
}
// TODO: break this up. run_report, Preview, Fix
pub fn run(paths: &Option<Vec<String>>, cli: &Cli) -> Result<()> {
    let t1 = std::time::Instant::now();
    let mut change_count = 0;

    let st = {
        if let Some(paths) = paths {
            SourceTree::new_from_paths(paths)
        } else {
            SourceTree::new_from_cwd()
        }
    };

    for rsc in st.source_files.iter() {
        let new_m = rsc
            .make_adjustments(&rsc.named_idents)
            .into_iter()
            .map(|adj| (adj.line_num, adj.contents))
            .collect::<HashMap<usize, String>>();

        let output = (0..rsc.total_lines)
            .into_iter()
            .map(|n| -> String {
                if let Some(new) = new_m.get(&n) {
                    change_count += 1;
                    if !cli.quiet {
                        green!(new.clone(), n)
                    }
                    new.to_owned()
                } else {
                    let new = rsc.get(&n).unwrap().contents.to_owned();
                    if !cli.quiet {
                        red!(new.clone(), n);
                    }
                    new
                }
            })
            .collect::<Vec<String>>();

        _ = std::fs::write(&rsc.file, output.join("\n"));
    }

    println!(
        "\n\nCOMPLETE!\n{} CHANGES ON {} FILES IN: {}s",
        change_count,
        st.source_files.len(),
        t1.elapsed().as_secs_f64()
    );

    Ok(())
}
