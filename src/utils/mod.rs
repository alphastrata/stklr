use anyhow::Result;
use glob::glob;
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

#[inline(always)]
/// Read the lines of a file into `Lines`
pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(BufReader::new(file).lines())
}

/// Grep/Glob all the rust files recursively from a target directory downward.
pub fn search_for_rust_files(dir: impl AsRef<Path>) -> Result<Vec<std::path::PathBuf>> {
    let pattern = dir.as_ref().join("**").join("*.rs");
    let files = glob(&pattern.to_string_lossy())
        .map_err(|e| anyhow::Error::new(e))?
        .map(|entry| entry.map_err(|e| anyhow::Error::new(e)))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(files)
}
