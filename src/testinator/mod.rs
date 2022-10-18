//! Utils make the world go round..
//!

use super::search::utils::{Flavour, RawLine, RawSourceCode, SourceTree};
use crate::{blue, green, red};

use anyhow::Result;
use std::{fmt::Display, path::PathBuf};

/// Holding everything you could possibly want to know about the location of a #[test]
#[derive(Debug, Default, Clone)]
pub struct TestLocation {
    pub file: PathBuf,
    pub line: RawLine,
    pub name: Option<String>,
    pub closing_bracket: usize,
}

impl Display for TestLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //.for_each(|ft| green!(ft.line.line_num, ft.name.clone().unwrap()));
        let path = self.file.file_name().unwrap().to_string_lossy();
        let file = ansi_term::Colour::Blue.paint(path);
        let ln = ansi_term::Colour::Yellow.paint(self.line.line_num.to_string());
        let name = ansi_term::Colour::Green.paint(self.name.clone().unwrap_or_default());
        write!(f, "{}\n {}:{}\n", file, ln, name)
    }
}

impl TestLocation {
    /// Begins the process of working out where a test is, and its scope.
    pub fn new(rsc: &RawSourceCode, rl: &RawLine) -> Option<Self> {
        if rl.contents.contains("test]") {
            // I'm taking the RHS of the #[test] macro here to
            // account for things like #[tokio::test]
            return Some(
                TestLocation {
                    file: rsc.file.clone(),
                    line: rl.clone(),
                    closing_bracket: 0, //NOTE: the linecounter begins with 1 so we're good here.
                    name: None,
                }
                .namify(rsc)
                .refine(rsc),
            );
        }
        None
    }
    /// Name the test you've captured! (useful if we want to call `cargo test <testname>`),
    /// it uses the same RawLine::find_idents() that STKLR uses to match docstrings.
    /// NOTE: this function looks at the line BELOW what you load it with i.e: self.line_num + 1
    fn namify(&mut self, rsc: &RawSourceCode) -> Self {
        // Get the line ahead.
        if let Some(rl) = rsc.get(&(self.line.line_num + 1)) {
            let mut rl = rl.clone(); //NOTE: get_mut is a thing... maybe change/upgrade to that?
            rl.find_idents();
            match rl.flavour {
                Flavour::RUST_FN => self.name = Some(rl.idents[0].clone()),
                Flavour::RUST_DOCS => self.name = None,
                _ => {}
            }
        }
        self.clone()
    }

    /// Goes over the area around the test (using whitespace) to work out its scope.
    fn refine(&mut self, rsc: &RawSourceCode) -> Self {
        //NOTE: ITM here is ok because we cannot know the starting index, and potentially the size
        //of a map on a large enough codebase makes this a more effective use of our looptime.
        let mut idx = self.line.line_num;
        loop {
            if let Some(ln) = rsc.m.get(&idx) {
                // this is what the last line of any test/function/method/impl
                // *should* be in cargo fmtted rust...
                if ln.contents.contains("}\n") {
                    // NOTE: in the unlikely event someone has #[test] in a comment, like here...
                    match ln.flavour {
                        Flavour::RUST_DOCS => {}
                        _ => {
                            self.closing_bracket = idx;
                            return self.clone();
                        }
                    };
                }
                idx += 1;
            } else {
                break;
            }
        }
        self.clone()
    }
}

/// Given a source_file, go through it and find tests!
pub fn grep_tests(_st: &SourceTree) -> Result<Vec<TestLocation>> {
    Ok(SourceTree::new_from_cwd()
        .source_files
        .iter()
        .flat_map(|rsc| {
            rsc.iter()
                .flat_map(|(_, rl)| TestLocation::new(rsc, rl))
                .collect::<Vec<TestLocation>>()
        })
        .collect::<Vec<TestLocation>>())
}

//TODO: use the Vec<TestMap> against the Vec<RawSource> from the SourceTree.
