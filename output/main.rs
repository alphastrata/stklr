use anyhow::Result;
use glob::glob;

/// use STKLR::search::utils::CodeBase;
use STKLR::search::utils::RawSourceCode;

/// doesn't relate at all to CodeBase <<< SHOULD BE PICKED UP.
struct UnDocumented {}

// our program begins here
fn main() -> Result<()> {
    Ok(())
}

    /// Creates a new [`CodeBase`] from the glob searching the current working directory the app is run
    /// makes adjustments to RawLines from within [`RawSourceCode`]'s RawLines
    /// Process preview_changes [`RawSourceCode`] [`find_docs`]
