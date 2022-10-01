#![allow(unused_imports)]
#![allow(dead_code)]
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
    //m: HashMap<usize, LineMatch>,
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

impl LineMatch {
    /// Processes the docs/idents that may be present in lines
    fn process(&mut self) {
        self.find_idents();
        self.find_docs();
    }

    //TODO: you cannot mutate the lines directly in SC with THESE, because you're stripping stuff
    //out.
    //These entries should be populating the SearchBuf so that it can check/get these bad-bois
    //later
    //TODO: LineMatch.is_ident() should be has ident/docs
    //TODO: global mutexed vec of the idents -- names only
    //TODO: implement the link_ident for LineMatch (if docs && if ident_is_present &&
    //ident_is_naked)
    //TODO: can the SearchBuf take refs to the sc?
    fn find_idents(&mut self) {
        let text = self.contents.to_owned();
        for caps in RUST_FN.captures_iter(&text) {
            if let Some(v) = caps.name("ident") {
                self.contents = v.as_str().to_string();
                self.is_ident = true;
            }
        }
        for caps in RUST_ENUM.captures_iter(&text) {
            if let Some(v) = caps.name("ident") {
                self.contents = v.as_str().to_string();
                self.is_ident = true;
            }
        }
        for caps in RUST_STRUCT.captures_iter(&text) {
            if let Some(v) = caps.name("ident") {
                self.contents = v.as_str().to_string();
                self.is_ident = true;
            }
        }
        for caps in RUST_TRAIT.captures_iter(&text) {
            if let Some(v) = caps.name("ident") {
                self.contents = v.as_str().to_string();
                self.is_ident = true;
            }
        }
        for caps in RUST_TY.captures_iter(&text) {
            if let Some(v) = caps.name("ident") {
                self.contents = v.as_str().to_string();
                self.is_ident = true;
            }
        }
    }
    fn find_docs(&mut self) {
        let text = self.contents.to_owned();
        for caps in RUST_DOCSTRING.captures_iter(&text) {
            if let Some(v) = caps.name("ident") {
                self.contents = v.as_str().to_string();
                self.is_ident = false;
            }
        }
    }
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
    fn read_lines_of_source() {
        let mut sc = SourceCode::new_from_file("src/main.rs");
        for (_, v) in sc.iter_mut() {
            v.process();
            if !v.is_ident {
                dbg!(&v);
            }
        }
    }

    #[test]
    fn write_source_back() {
        let mut sc = SourceCode::new_from_file("src/main.rs");
        for (_, v) in sc.iter_mut() {
            v.process();
        }
        (0..sc.total_lines)
            .into_iter()
            .enumerate()
            .for_each(|(e, l)| println!("{:?}", sc.get(&e).unwrap()))
    }
}
// impl SearchBuf {
//     /// Initialise a new `SearchBbuf` from a given file.
//     // NOTE: mpsc pattern used here, modify with care.
//     fn init<P>(filename: P) -> Self
//     where
//         P: AsRef<Path>,
//     {
//         let mut sb: SearchBuf = Self::default();
//         let (tx, rx) = mpsc::channel();
//
//         // Search docstrings, and build collection of them.
//         let file_buf_read = read_lines(filename);
//
//         if let Ok(lines) = file_buf_read {
//             lines
//                 .collect::<Vec<_>>()
//                 .iter() //NOTE: indentionally not par_iter, as we'll save that for multiple files.
//                 .enumerate()
//                 .for_each(|(e, l)| {
//                     let tx_c = tx.clone();
//                     //TODO: A macro for all .contains(tag) pls
//                     if let Ok(l) = l {
//                         if l.contains("///") {
//                             let lm = LineMatch {
//                                 line_num: e,
//                                 contents: l.into(),
//                                 is_ident: false,
//                                 hits: 0, //NOTE: this is updated later
//                             };
//                             if let Err(e) = tx_c.send(lm) {
//                                 eprintln!("Error tx_c.send() -> {}", e)
//                             }
//                         }
//                     }
//                 });
//             // close the sender. (otherwise the rx listens indefinitely)
//         }
//
//         // // Search file for $idents
//         // if let Ok(lines) = file_buf_read{
//         //     lines.collect::<Vec<_>>().iter().enumerate().for_each((e,l){
//         //         let tx_c = tx.clone();
//         //
//         //         if let Ok(l) = l{
//         //             // do regex matching and send
//         //             let (caps, hits) = idents_from_regex(&l);
//         //             let lm LineMatch{
//         //                 line_num: e,
//         //                 is_ident: true,
//         //                 contents: caps.name("ident"),
//         //                 hits: 0,
//         //             }.
//         //
//         //         }
//         //     });
//         //     drop(tx);
//         // }
//
//         while let Ok(lm) = rx.recv() {
//             if lm.is_ident {
//                 sb.idents.push(lm.line_num)
//             } else {
//                 sb.docstrings.push(lm.line_num)
//             }
//             sb.m.insert(lm.line_num, lm);
//         }
//         sb
//     }
// }
