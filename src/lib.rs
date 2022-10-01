#![allow(unused_imports)]
#![allow(dead_code)]
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

#[derive(Debug, Clone)]
struct LineMatch {
    line_num: usize, //NOTE indentionally duplicate data
    /// Contains either an entire line's worth of docstring, or a _single_ `$ident`
    contents: String,
    /// We assume `false` to mean a docstring
    is_ident: bool,
    /// `hits` should be 0, if none of the `$ident`s we're hoping for are in `self.contents`
    hits: usize,
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

/// `SearchBuf` is a buffer holding all lines with matches.
#[derive(Default, Debug, Clone)]
struct SearchBuf {
    m: HashMap<usize, LineMatch>,
    file: PathBuf,
    idents: Vec<usize>,
    docstrings: Vec<usize>,
}

#[derive(Default, Debug, Clone)]
struct CodeBase {
    inner: Vec<SourceCode>,
}

/// In-memory representation of a rust-code source file.
#[derive(Default, Debug, Clone)]
struct SourceCode {
    m: HashMap<usize, LineMatch>,
    file: PathBuf,
    ident_locs: Vec<usize>,
    doc_locs: Vec<usize>,
    total_lines: usize,
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
        };
        if let Ok(lines) = read_lines(file) {
            lines
                .collect::<Vec<_>>()
                .iter()
                .enumerate()
                .for_each(|(e, l)| {
                    if let Ok(l) = l {
                        sc.m.insert(
                            e,
                            LineMatch {
                                line_num: e,
                                contents: l.into(),
                                is_ident: false,
                                hits: 0,
                            },
                        );
                    }
                });
        }
        sc.total_lines = sc.m.len();
        sc
    }
}

fn idents_from_regex(text: &str) -> String {
    let rust_fn = Regex::new(r"(fn\s{1}(?P<ident>\w*))").unwrap();
    let rust_enum = Regex::new(r"(enum\s{1}(?P<ident>\w*))").unwrap();
    let rust_struct = Regex::new(r"(struct\s{1}(?P<ident>\w*))").unwrap();
    let rust_ty = Regex::new(r"(type\s{1}(?P<ident>\w*))").unwrap();
    let rust_trait = Regex::new(r"(trait\s{1}(?P<ident>\w*))").unwrap();
    //let rust_docstring = Regex::new(r#"(?P<ident>(///))"#).unwrap();

    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn read_sourcecode() {
        let sc = SourceCode::new_from_file("src/main.rs");
        dbg!(sc);
    }

    #[test]
    fn read_lines_test() {
        let sb = SearchBuf::init("src/lib.rs");
        eprintln!("{:#?}", sb);
    }

    #[test]
    fn reg_test() {
        let rust_fn = Regex::new(r"(fn\s{1}(?P<ident>\w*))").unwrap();
        let rust_enum = Regex::new(r"(enum\s{1}(?P<ident>\w*))").unwrap();
        let rust_struct = Regex::new(r"(struct\s{1}(?P<ident>\w*))").unwrap();
        let rust_ty = Regex::new(r"(type\s{1}(?P<ident>\w*))").unwrap();
        let rust_trait = Regex::new(r"(trait\s{1}(?P<ident>\w*))").unwrap();
        let rust_docstring = Regex::new(r#"(?P<ident>(///))"#).unwrap();

        let text = std::fs::read_to_string("src/main.rs").unwrap();
        for caps in rust_docstring.captures_iter(&text) {
            eprintln!("doc: {}", caps.name("ident").unwrap().as_str());
        }
        for caps in rust_fn.captures_iter(&text) {
            eprintln!("fn:{}", caps.name("ident").unwrap().as_str());
        }
        for caps in rust_enum.captures_iter(&text) {
            eprintln!("enum:{}", caps.name("ident").unwrap().as_str());
        }
        for caps in rust_struct.captures_iter(&text) {
            eprintln!("struct:{}", caps.name("ident").unwrap().as_str());
        }
        for caps in rust_trait.captures_iter(&text) {
            eprintln!("struct:{}", caps.name("ident").unwrap().as_str());
        }
        for caps in rust_ty.captures_iter(&text) {
            eprintln!("struct:{}", caps.name("ident").unwrap().as_str());
        }
    }
}

impl SearchBuf {
    /// Initialise a new `SearchBbuf` from a given file.
    // NOTE: mpsc pattern used here, modify with care.
    fn init<P>(filename: P) -> Self
    where
        P: AsRef<Path>,
    {
        let mut sb: SearchBuf = Self::default();
        let (tx, rx) = mpsc::channel();

        // Search docstrings, and build collection of them.
        let file_buf_read = read_lines(filename);

        if let Ok(lines) = file_buf_read {
            lines
                .collect::<Vec<_>>()
                .iter() //NOTE: indentionally not par_iter, as we'll save that for multiple files.
                .enumerate()
                .for_each(|(e, l)| {
                    let tx_c = tx.clone();
                    //TODO: A macro for all .contains(tag) pls
                    if let Ok(l) = l {
                        if l.contains("///") {
                            let lm = LineMatch {
                                line_num: e,
                                contents: l.into(),
                                is_ident: false,
                                hits: 0, //NOTE: this is updated later
                            };
                            if let Err(e) = tx_c.send(lm) {
                                eprintln!("Error tx_c.send() -> {}", e)
                            }
                        }
                    }
                });
            // close the sender. (otherwise the rx listens indefinitely)
        }

        // // Search file for $idents
        // if let Ok(lines) = file_buf_read{
        //     lines.collect::<Vec<_>>().iter().enumerate().for_each((e,l){
        //         let tx_c = tx.clone();
        //
        //         if let Ok(l) = l{
        //             // do regex matching and send
        //             let (caps, hits) = idents_from_regex(&l);
        //             let lm LineMatch{
        //                 line_num: e,
        //                 is_ident: true,
        //                 contents: caps.name("ident"),
        //                 hits: 0,
        //             }.
        //
        //         }
        //     });
        //     drop(tx);
        // }

        while let Ok(lm) = rx.recv() {
            if lm.is_ident {
                sb.idents.push(lm.line_num)
            } else {
                sb.docstrings.push(lm.line_num)
            }
            sb.m.insert(lm.line_num, lm);
        }
        sb
    }
}
