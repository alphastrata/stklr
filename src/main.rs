use anyhow::Result;
use glob::glob;

/// use STKLR::search::utils::CodeBase;
use STKLR::search::utils::RawSourceCode;

/// doesn't relate at all to CodeBase <<< SHOULD BE PICKED UP.
struct UnDocumented {}

// our program begins here
fn main() -> Result<()> {
    for path in glob("./**/*.rs").unwrap().filter_map(Result::ok) {
        let p = path;
        //let mut rsc = RawSourceCode::new_from_file(&p);
        let mut rsc = RawSourceCode::new_from_file("src/main.rs");
        //rsc.preview_changes();
        rsc.write();
    }

    Ok(())
}
