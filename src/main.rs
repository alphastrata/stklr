use anyhow::Result;
use glob::glob;

use STKLR::search::utils::CodeBase;
use STKLR::search::utils::SourceCode;

/// doesn't relate at all to CodeBase
struct UnDocumented {}

// our program begins here
fn main() -> Result<()> {
    for path in glob("./**/*.rs").unwrap().filter_map(Result::ok) {
        let p = path;
        let mut sc = SourceCode::new_from_file(&p);
        //let mut sc = SourceCode::new_from_file("src/main.rs");
        //sc.preview_changes();
        sc.preview_changes();
        //sc.execute();
    }

    Ok(())
}
