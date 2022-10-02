#![allow(dead_code)]
#![allow(unused_imports)]
use super::consts::*;

use anyhow::Error;
use rayon::iter::Enumerate;
use rayon::prelude::*;
use regex::Regex;
use regex::RegexSet;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use std::fs::File;
use std::io::{self, BufRead};
use std::ops::Deref;
use std::ops::DerefMut;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

/// Representing the levels of 'Completeness' a line's docstrings may be in.
#[derive(Debug, Clone)]
enum Linked {
    Complete,
    /// Ready to write out to stdout or the file actual
    Loaded,
    /// Some linking opportunities are not being acted upon
    Partial,
    /// No linking opportunities are present
    Irrelevant,
    Unprocessed,
}

/// Indicating whether the keyword, regex match we're dealing with is for docs, or the declaration
/// of something we need to know about.
#[derive(Debug, Clone)]
enum Flavour {
    Docstring,
    Declare,
    Tasteless,
}
/// All the information you could possibly need to do this app's job.
#[derive(Debug, Clone)]
struct LineMatch {
    all_linked: Linked,
    contents_modified: Option<String>,
    contents_original: String,
    flavour: Flavour,
    hits: Vec<String>, //NOTE: expended data (exists temporarily to populate the SourceCode struct(s))
    line_num: usize,   //NOTE: indentionally duplicate data
}

/// Helper to read the lines of a file and give you back an easy-iterable (from the cookbook).
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

/// A collection of source files, with their $idents.
#[derive(Default, Debug, Clone)]
struct CodeBase {
    source_files: Vec<SourceCode>,
    idents: HashMap<String, String>,
}

/// In-memory representation of a rust-code source file.
#[derive(Default, Debug, Clone)]
struct SourceCode {
    m: HashMap<usize, LineMatch>,
    file: PathBuf,
    ident_locs: Vec<usize>,
    doc_locs: Vec<usize>,
    total_lines: usize,
    named_idents: Vec<String>,
}

impl Deref for SourceCode {
    type Target = HashMap<usize, LineMatch>;
    fn deref(&self) -> &Self::Target {
        &self.m
    }
}
impl DerefMut for SourceCode {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.m
    }
}

impl SourceCode {
    /// Create a new `SourceCode` from a given file, holding all lines of said file as `String`s.
    /// Deref is implemented so that you can treat the internal `HashMap` that backs the struct, as
    /// what-it-is.
    fn new_from_file<P>(file: P) -> Self
    where
        PathBuf: From<P>,
        P: AsRef<Path> + Copy,
    {
        let mut sc = SourceCode {
            m: HashMap::new(),
            file: PathBuf::from(file),
            ident_locs: Vec::new(),
            doc_locs: Vec::new(),
            total_lines: 0,
            named_idents: Vec::new(),
        };
        //TODO: MPSC this.
        if let Ok(lines) = read_lines(file) {
            lines
                .collect::<Vec<_>>()
                .iter()
                .enumerate()
                .for_each(|(e, l)| {
                    if let Ok(l) = l {
                        let mut lm = LineMatch {
                            all_linked: Linked::Unprocessed,
                            contents_modified: None,
                            contents_original: l.into(),
                            flavour: Flavour::Tasteless,
                            hits: Vec::new(),
                            line_num: e,
                        };
                        lm.find_docs();
                        lm.find_idents();
                        if lm.hits.len() > 0 {
                            sc.named_idents.extend(lm.hits.iter().cloned())
                        }
                        sc.m.insert(e, lm);
                    }
                });
        }
        sc.total_lines = sc.m.len();
        sc.named_idents.dedup();
        sc
    }
    /// Imposes docstring linking changes on any and all idents that're contained within self.
    fn prepare_lines(&mut self, preview: bool) {
        _ = self
            .named_idents
            .iter()
            .map(|i| {
                _ = self
                    .m
                    .iter_mut()
                    .map(|(_k, lm)| match lm.flavour {
                        Flavour::Docstring => {
                            //BUG: this will actually fail when we have say [`Ident`] and Ident in the same line.
                            if lm.contents_original.contains(i)
                                && !lm.contents_original.contains(&format!("`{}`", i))
                            {
                                let needle = &format!("[`{}`]", i);
                                lm.contents_modified =
                                    Some(lm.contents_original.replace(i, needle));
                                if preview {
                                    println!("PREVIEW CHANGE FOR: Line Number::{}", lm.line_num);
                                    println!("{}", &lm.contents_original);
                                    println!("{}\n", lm.contents_modified.clone().unwrap());
                                }
                            }
                            lm.all_linked = Linked::Complete;
                        }
                        _ => {}
                    })
                    .collect::<()>();
            })
            .collect::<()>();
    }
    /// Preview the pending changes, line-by-line
    fn preview(&mut self) {
        let t1 = std::time::Instant::now();
        self.prepare_lines(true);
        dbg!(t1.elapsed().as_nanos());
    }

    /// Execute the changes by mem-swapping the contents_original with the contents_modified and
    fn execute(&mut self) {
        self.prepare_lines(true);
    }
    /// Writes the contents_modified field of all LineMatches to their original source files.
    fn write(&self) {
        todo!()
    }
}

impl LineMatch {
    /// Processes the docs/idents that may be present in lines
    fn process(&mut self) {
        self.find_docs();
        self.find_idents();
    }
    /// Find the identifying keywords we're interested in.
    //TODO: Make this DRY with a macro
    fn find_idents(&mut self) {
        let text = self.contents_original.to_owned();
        macro_rules! generate_ident_find_loops {
            ($($CONST:ident),*) => {
                $(for caps in $CONST.captures_iter(&text) {
                    if let Some(v) = caps.name("ident") {
                        let cap = v.as_str().to_string();
                        self.flavour = Flavour::Declare;
                        self.hits.push(cap);
                    }
                })*
            };
        }
        generate_ident_find_loops!(RUST_FN, RUST_TY, RUST_ENUM, RUST_STRUCT, RUST_TRAIT);
        // for caps in RUST_FN.captures_iter(&text) {
        //     if let Some(v) = caps.name("ident") {
        //         let cap = v.as_str().to_string();
        //         self.flavour = Flavour::Declare;
        //         self.hits.push(cap);
        //     }
        // }
        // for caps in RUST_ENUM.captures_iter(&text) {
        //     if let Some(v) = caps.name("ident") {
        //         let cap = v.as_str().to_string();
        //         self.flavour = Flavour::Declare;
        //         self.hits.push(cap);
        //     }
        // }
        // for caps in RUST_STRUCT.captures_iter(&text) {
        //     if let Some(v) = caps.name("ident") {
        //         let cap = v.as_str().to_string();
        //         self.flavour = Flavour::Declare;
        //         self.hits.push(cap);
        //     }
        // }
        // for caps in RUST_TRAIT.captures_iter(&text) {
        //     if let Some(v) = caps.name("ident") {
        //         let cap = v.as_str().to_string();
        //         self.flavour = Flavour::Declare;
        //         self.hits.push(cap);
        //     }
        // }
        // for caps in RUST_TY.captures_iter(&text) {
        //     if let Some(v) = caps.name("ident") {
        //         let cap = v.as_str().to_string();
        //         self.flavour = Flavour::Declare;
        //         self.hits.push(cap);
        //     }
        // }
    }

    /// marks `Self` as having, or not having a `///` docstring at the beginning of the line.
    fn find_docs(&mut self) {
        let text = self.contents_original.to_owned();
        for caps in RUST_DOCSTRING.captures_iter(&text) {
            if let Some(_) = caps.name("ident") {
                //let docstring_type = v.as_str().to_string();
                self.flavour = Flavour::Docstring;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Flavour;
    use super::SourceCode;
    use super::*;
    #[test]
    fn read_sourcecode() {
        let sc = SourceCode::new_from_file("src/main.rs");
        dbg!(sc);
    }

    #[test]
    fn read_lines_of_source() {
        let mut sc = SourceCode::new_from_file("src/search/utils.rs");
        sc.preview();
    }

    // #[test]
    // fn write_source_back() {
    //     let mut sc = SourceCode::new_from_file("src/main.rs");
    //     for (_, v) in sc.iter_mut() {
    //         v.process();
    //         //v.impose(&mut sc);
    //         dbg!(v);
    //     }
    //     (0..sc.total_lines)
    //         .into_iter()
    //         .enumerate()
    //         .for_each(|(e, l)| println!("{:?}", sc.get(&e).unwrap()))
    // }
}
