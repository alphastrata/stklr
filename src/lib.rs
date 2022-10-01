use anyhow;
use regex::RegexSet;
use std::fs;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

/// `Hey` has docs!
/// not all of them indicate that Hey can be linked back too
enum Hey {
    Hi,
    Heya,
    /// Docs can be on a variant, like here for Bonjour, but it too is not linked.
    Bonjour,
}

enum Without {
    // no docs :(
    With,
    Without,
}

struct Line {
    num: usize,
    contents: String,
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_lines_test() {
        if let Ok(lines) = read_lines("src/lib.rs") {
            // Consumes the iterator, returns an (Optional) String
            for (e, l) in lines.enumerate() {
                if let Ok(l) = l {
                    if l.contains("///") {
                        println!("{}:\t{}", e, l); //NOTE: RG has line numbers in green, search
                        //match in red
                    }
                }
            }
        }
    }
}
