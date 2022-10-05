//
//! STKLR
//

#![allow(unused_imports)]
use anyhow::Result;
use log::{debug, error, info, trace, warn};
use STKLR::search::utils::SourceTree;

fn main() -> Result<()> {
    let t1 = std::time::Instant::now();

    // A st= SourceTree which is an abstraction over all the .rs files' contents
    let st = SourceTree::new_from_cwd();

    // rsc = RawSourceCode
    for rsc in st.source_files.iter() {
        println!("{}", rsc.file.display());
        rsc.make_adjustments(&rsc.named_idents)
            .iter()
            .for_each(|al| info!("{}", al));
    }

    println!(
        "{} FILES IN: {}s",
        st.source_files.len(),
        t1.elapsed().as_secs_f64()
    );
    Ok(())
}
