//
//! STKLR
//
#![allow(non_snake_case)]
use anyhow::Result;

use STKLR::search::utils::CodeBase;

///
/// We want a CodeBase to get linked
/// A [`search`] to be skipped
/// a preview_changes to be linked, and another preview_changes here linked too.
/// the Result<()> would be a bonus.
/// and the main
/// write_changes should be picked up.
fn main() -> Result<()> {
    let cb: CodeBase = CodeBase::new_from_cwd();
    cb.write_changes();
    Ok(())
}
