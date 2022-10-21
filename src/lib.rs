pub mod cmd;
mod feedback;
mod search;
pub mod termite;
pub mod testinator;

use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(BufReader::new(file).lines())
}

