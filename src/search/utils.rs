#![allow(unused_imports)]
#![allow(dead_code)]

use super::consts::*;
use crate::termite;
use log::{debug, error, info, trace, warn};

use core::fmt::Display;
use glob::glob;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::ops::Deref;
use std::ops::DerefMut;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

#[derive(Default, Debug, Clone)]
pub enum Linked {
    Complete,
    Loaded,
    Partial,
    Irrelevant,
    #[default]
    Unprocessed,
}
#[derive(Default, Debug, Clone)]
pub enum Flavour {
    Docstring,
    Declare,
    #[default]
    Tasteless,
}
#[derive(Default, Debug, Clone)]
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
    pub source_files: Vec<RawSourceCode>,
    pub named_idents: Vec<String>,
}

impl CodeBase {
    /// Populates the idents we care about...
    fn populate_idents(mut self) -> Self {
        self.source_files.iter().for_each(|sf| {
            sf.named_idents.iter().for_each(|e| {
                info!("{}::{}", sf.file.display(), e);
                self.named_idents.push(e.to_string())
            });
        });
        self
    }
    /// Creates a new CodeBase from the glob searching the current working directory the app is run
    /// in.
    pub fn new_from_cwd() -> Self {
        Self::new_from_dir(".")
    }
    /// Creates a new CodeBase from a given directory.
    pub fn new_from_dir<P>(dir: P) -> Self
    where
        P: Display + AsRef<Path>,
    {
        _ = termite::setup_logger().unwrap();

        let search_path = format!("{}/**/*.rs", dir);
        CodeBase {
            source_files: {
                glob(&search_path)
                    .unwrap()
                    .filter_map(Result::ok)
                    .map(|p| RawSourceCode::new_from_file(&p))
                    .collect::<Vec<RawSourceCode>>()
            },
            named_idents: Vec::new(),
        }
        .populate_idents()
    }
    /// makes adjustments to RawLines from within RawSourceCode's RawLines
    fn make_adjustments(&self, tx: Sender<(usize, String)>) {
        let tx_c = tx.clone();
        self.source_files.iter().for_each(|sf| {
            sf.named_idents.iter().for_each(|i| {
                sf.m.iter()
                    .for_each(|(_, raw_line)| match raw_line.flavour {
                        Flavour::Docstring => {
                            //BUG: haven't accounted for multiple occurences, or a nested
                            //occurence.
                            if raw_line.should_be_modified(i) {

                                tx_c.send((
                                    raw_line.line_num,
                                    raw_line.process_changes(i)
                                ))
                                .expect("Unable to send line on channel, something has gone HORRIBLY WRONG!");
                                info!("Send success!")
                            }
                        }

                        _ => {}
                    })
            });
        })
    }

    pub fn preview_changes(self) {
        let tsrsc = self.clone();

        let write_buf: Arc<Mutex<HashMap<usize, String>>> = Arc::new(Mutex::new(HashMap::new()));
        let (tx, rx) = mpsc::channel();
        let tx_c = tx.clone();

        let txer = thread::spawn(move || tsrsc.make_adjustments(tx_c));

        let wb_c = write_buf.clone();
        let rxer = thread::spawn(move || {
            while let Ok((e, ml)) = rx.recv() {
                wb_c.lock().unwrap().insert(e, ml);
            }
        });

        if let Ok(_) = txer.join() {
            drop(tx);
        }

        rxer.join().expect("unexpected threat failure.");

        let write_buf_complete = write_buf.lock().unwrap();

        let total_lines = self.source_files.iter().map(|sf| sf.total_lines).sum();
        self.source_files
            .iter()
            .for_each(|e| info!("Working on: {:?}", e.file.display()));

        for sf in self.source_files.iter() {
            let mut output = String::new();
            (0..total_lines).into_iter().for_each(|i| {
                if let Some(m) = write_buf_complete.get(&i) {
                    println!("{}", sf.file.display());
                    println!("{}{}", i, m.replace("\t", ""));
                    output.push_str(&format!("{}\n", m))
                } else {
                    if let Some(sf) = sf.get(&i) {
                        output.push_str(&format!("{}\n", sf.contents_original))
                    }
                }
            });
        }
    }

    pub fn write_changes(self) {
        let tsrsc = self.clone();

        let write_buf: Arc<Mutex<HashMap<usize, String>>> = Arc::new(Mutex::new(HashMap::new()));
        let (tx, rx) = mpsc::channel();
        let tx_c = tx.clone();

        let txer = thread::spawn(move || tsrsc.make_adjustments(tx_c));

        let wb_c = write_buf.clone();
        let rxer = thread::spawn(move || {
            while let Ok((e, ml)) = rx.recv() {
                wb_c.lock().unwrap().insert(e, ml);
            }
        });

        if let Ok(_) = txer.join() {
            drop(tx);
        }

        rxer.join().expect("unexpected threat failure.");

        let write_buf_complete = write_buf.lock().unwrap();

        let total_lines = self.source_files.iter().map(|sf| sf.total_lines).sum();

        self.source_files
            .iter()
            .for_each(|e| info!("Working on: {:?}", e.file.display()));

        for sf in self.source_files.iter() {
            let mut output = String::new();
            (0..total_lines).into_iter().for_each(|i| {
                if let Some(m) = write_buf_complete.get(&i) {
                    info!("Sent a modified {}::{}\n", sf.file.display(), i);
                    output.push_str(&format!("{}\n", m))
                } else {
                    if let Some(sf) = sf.get(&i) {
                        output.push_str(&format!("{}\n", sf.contents_original))
                    }
                }
            });
            output.split('\n').for_each(|l| println!("{}", l));

            let tmp = format!(
                "./results/{}",
                sf.file.display().to_string().split('/').last().unwrap()
            );

            _ = std::fs::File::create(&tmp).unwrap();
            std::fs::write(&tmp, output).unwrap();
        }
    }
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
        sc.named_idents.retain(|x| x != ""); //TODO: can you initialise this better so avoid this?
        sc
    }
}

impl RawLine {
    /// Will return true for a SINGLE instance of when a modification should be made, how many may
    /// really be in there is the domain of [`process_changes`]
    fn should_be_modified(&self, i: &str) -> bool {
        if self.contents_original.contains(i)
            || self.contents_original.contains(&format!("{i}s"))
            || self.contents_original.contains(&format!("{i}'s"))
                && !self.contents_original.contains(&format!("`{}`", i))
                && self.contents_original.contains("///")
        {
            return true;
        }
        false
    }
    /// Actually processes the modifications to a RawLine's contents_modified
    fn process_changes(&self, i: &str) -> String {
        let padded_i = format!(" {} ", i);
        let padded_self = format!("{} ", self.contents_original);
        let changes_to_make: Vec<&str> = padded_self.matches(&padded_i).collect();
        let needle = &format!("[`{}`]", i);

        self.contents_original
            .clone()
            .replacen(i, needle, changes_to_make.len())
    }
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
                        trace!("{}::{}", self.line_num, cap);
                        self.flavour = Flavour::Declare;
                        self.hits.push(cap);
                    }
                })*
            };
        }
        generate_ident_find_loops!(
            RUST_FN,
            RUST_TY,
            RUST_ENUM,
            RUST_STRUCT,
            RUST_TRAIT,
            RUST_USE,
            RUST_IMPORT
        );
    }

    /// Process preview_changes RawSourceCode [`find_docs`]
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
    use anyhow::Result;
    use glob::glob;

    #[test]
    fn handle_imports() {
        let example = r#"
                        use anyhow::Result;
                        use STKLR::search::utils::CodeBase;
                        use STKLR::search::utils::RawSourceCode;
                        }"#;

        let mut rl = RawLine {
            all_linked: Linked::Unprocessed,
            contents_modified: None,
            contents_original: example.into(),
            flavour: Flavour::Tasteless,
            hits: Vec::new(),
            line_num: 0,
        };

        rl.find_idents();
        rl.hits.dedup();
        assert_eq!(rl.hits.len(), 9);
        let answers = [
            "anyhow",
            "STKLR",
            "Result",
            "search",
            "utils",
            "CodeBase",
            "search",
            "utils",
            "RawSourceCode",
        ];
        rl.hits
            .iter()
            .enumerate()
            .for_each(|(e, hit)| assert_eq!(answers[e], hit));
    }
    #[test]
    fn handle_lifetime_annotations() {
        let example = r#"async fn servitude<'a>(a: &'a str) -> bool{}"#;
        let mut rl = RawLine {
            contents_original: example.into(),
            ..Default::default()
        };
        rl.find_idents();
        rl.hits.dedup();
        assert_eq!(rl.hits.len(), 1);
        assert_eq!(rl.hits[0], "servitude");
    }
    #[test]
    fn handle_traits() {
        let example = r#"pub trait Bless { 
        fn bless(&P) -> Blessing 
        where P: Clone + Send + Sync {
        };
        }"#;
        let mut rl = RawLine {
            contents_original: example.into(),
            ..Default::default()
        };
        rl.find_idents();
        rl.hits.dedup();
        assert_eq!(rl.hits.len(), 2);
    }
    #[test]
    fn multi_edit_single_line() {
        let cb = CodeBase::new_from_dir("/media/jer/ARCHIVE/scrapers/rustwari");

        cb.write_changes();
    }

    //TODO: work out this nonsense, with 'to' there's no function for that in there!
    // Helper [`to`] return the x and y values from a given path

    //TODO: you need a test for nested vec<T>s...
    // * `range` a Vector<[`Mat`]> of the images you want to concat,...

    #[test]
    fn read_sourcecode() {
        _ = RawSourceCode::new_from_file("src/main.rs");
    }

    #[test]
    fn write_to_main() {
        //let cb = CodeBase::new_from_cwd();
        let cb: CodeBase = CodeBase {
            source_files: {
                glob("./src/main.rs")
                    .unwrap()
                    .filter_map(Result::ok)
                    .map(|p| RawSourceCode::new_from_file(&p))
                    .collect::<Vec<RawSourceCode>>()
            },
            named_idents: Vec::new(),
        };

        let cb = cb.populate_idents();
        cb.write_changes();
    }
    #[test]
    fn show_matched_idents() {
        for path in glob("./**/*.rs").unwrap().filter_map(Result::ok) {
            let p = path;
            let rsc = RawSourceCode::new_from_file(&p);
            if !rsc.named_idents.is_empty() {
                dbg!(rsc.named_idents);
            }
        }
    }
}
