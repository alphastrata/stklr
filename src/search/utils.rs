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
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::io::Write;
use std::io::{self, BufRead};
use std::ops::Deref;
use std::ops::DerefMut;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

#[derive(Debug, Clone)]
pub enum Linked {
    Complete,
    Loaded,
    Partial,
    Irrelevant,
    Unprocessed,
}
#[derive(Debug, Clone)]
pub enum Flavour {
    Docstring,
    Declare,
    Tasteless,
}
#[derive(Debug, Clone)]
pub struct RawLine {
    pub all_linked: Linked,
    pub contents_modified: Option<String>,
    pub contents_original: String,
    pub flavour: Flavour,
    pub hits: Vec<String>, //NOTE: expended data (exists temporarily to populate the SourceCode struct(s))
    pub line_num: usize,   //NOTE: indentionally duplicate data
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

#[derive(Default, Debug, Clone)]
pub struct CodeBase {
    source_files: Vec<RawSourceCode>,
    idents: HashMap<String, String>,
}

#[derive(Default, Debug, Clone)]
pub struct RawSourceCode {
    pub m: HashMap<usize, RawLine>,
    pub file: PathBuf,
    pub ident_locs: Vec<usize>,
    pub doc_locs: Vec<usize>,
    pub total_lines: usize,
    pub named_idents: Vec<String>,
}

impl Deref for RawSourceCode {
    type Target = HashMap<usize, RawLine>;
    fn deref(&self) -> &Self::Target {
        &self.m
    }
}
impl DerefMut for RawSourceCode {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.m
    }
}

impl RawSourceCode {
    pub fn new_from_file<P>(file: P) -> Self
    where
        PathBuf: From<P>,
        P: AsRef<Path> + Copy,
    {
        let mut sc = RawSourceCode {
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
                        let mut lm = RawLine {
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
    pub fn preview_changes(&mut self) {
        let filepath = self.file.clone();
        self.named_idents.iter().for_each(|i| {
            self.m.iter_mut().for_each(|(_k, lm)| match lm.flavour {
                Flavour::Docstring => {
                    //BUG: this will actually fail when we have say [`Ident`] and Ident in the same line.
                    //BUG: what if 'i' is logically a part of a word, say 'read' within 'readable'
                    if lm.contents_original.contains(i)
                        && !lm.contents_original.contains(&format!("`{}`", i))
                    {
                        let needle = &format!("[`{}`]", i);
                        lm.contents_modified = Some(lm.contents_original.replace(i, needle));
                        //TODO: move this out into a feedback.rs module

                        println!("PREVIEW CHANGE FOR:");
                        println!("{}::{}", filepath.display(), lm.line_num);
                        println!("{}", &lm.contents_original);
                        println!("{}\n", lm.contents_modified.clone().unwrap());
                    }
                    lm.all_linked = Linked::Complete;
                }
                _ => {}
            })
        });
    }
    pub fn execute(&mut self) {
        let mut write_buf = HashMap::new();
        let (tx, rx) = mpsc::channel();

        self.named_idents.iter().for_each(|i| {
            let tx_c = tx.clone();
            self.m.iter().for_each(|(_, lm)| {
                match lm.flavour {
                    Flavour::Docstring => {
                        //BUG: this will actually fail when we have say [`Ident`] and Ident in the same line.
                        if lm.contents_original.contains(i)
                            && !lm.contents_original.contains(&format!("`{}`", i))
                        {
                            let needle = &format!("[`{}`]", i);
                            if let Err(e) = tx_c.send((
                                lm.line_num,
                                lm.contents_original.replace(i, needle).clone(),
                            )) {
                                eprintln!("Failure to send:{}", e);
                            }
                        }
                    }
                    _ => {}
                }
            })
        });

        drop(tx);

        while let Ok((e, lm)) = rx.recv() {
            dbg!(&lm);
            write_buf.insert(e, lm);
        }

        self.write(write_buf)
    }
    fn write(&self, write_buf: HashMap<usize, String>) {
        let mut output = String::new();

        (0..self.total_lines).into_iter().for_each(|i| {
            if let Some(m) = write_buf.get(&i) {
                output.push_str(&format!("{}\n", m))
            } else {
                output.push_str(&format!("{}\n", self.get(&i).unwrap().contents_original))
            }
        });

        std::fs::write(&self.file, output).unwrap();
    }

    fn peek_context(&self, pivot_point: usize) -> Vec<RawLine> {
        todo!()
    }
}

impl RawLine {
    fn process(&mut self) {
        self.find_docs();
        self.find_idents();
    }
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
    }

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
    use super::RawSourceCode;
    use super::*;
    #[test]
    fn read_sourcecode() {
        let sc = RawSourceCode::new_from_file("src/main.rs");
        dbg!(sc);
    }

    #[test]
    fn read_lines_of_source() {
        let mut sc = RawSourceCode::new_from_file("src/search/utils.rs");
        //let mut sc = SourceCode::new_from_file("src/main.rs");
        //sc.preview_changes();
        sc.execute();
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
