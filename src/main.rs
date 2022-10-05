//
//! STKLR
//

#![allow(unused_imports)]
use anyhow::Result;
use log::{debug, error, info, trace, warn};
use std::collections::HashMap;
use STKLR::search::utils::SourceTree;

fn main() -> Result<()> {
    let t1 = std::time::Instant::now();

    //let st = SourceTree::new_from_dir("/media/jer/ARCHIVE/scrapers/rustwari");

    let st = SourceTree::new_from_cwd();

    //NOTE: maybe par iter..?
    for rsc in st.source_files.iter() {
        println!("Processing: {}", rsc.file.display());
        let new_m = rsc
            .make_adjustments(&rsc.named_idents)
            .into_iter()
            .map(|adj| (adj.line_num, adj.contents))
            // to ignore the fact the adjusted lines come back out-of-order
            .collect::<HashMap<usize, String>>();

        // use the indexes of the lines as ground-truth, unchanged since ingest
        let output = (0..rsc.total_lines)
            .into_iter()
            .map(|n| -> String {
                if let Some(new) = new_m.get(&n) {
                    new.to_owned()
                } else {
                    rsc.get(&n).unwrap().contents.to_owned()
                }
            })
            .collect::<Vec<String>>();

        // Write each file
        _ = std::fs::write(&rsc.file, output.join("\n"));
        println!("Completed in {}s\n", t1.elapsed().as_secs_f64())
    }

    println!(
        "{} FILES IN: {}s",
        st.source_files.len(),
        t1.elapsed().as_secs_f64()
    );

    Ok(())
}
