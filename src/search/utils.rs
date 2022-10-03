#![allow(dead_code)]
#![allow(unused_imports)]
use super::consts::*;

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
    pub source_files: Vec<RawSourceCode>,
    pub named_idents: Vec<String>,
}

impl CodeBase {
    /// Populates the idents we care about...
    fn populate_idents(mut self) -> Self {
        self.source_files.iter().for_each(|sf| {
            sf.named_idents
                .iter()
                .for_each(|e| self.named_idents.push(e.to_string()));
        });
        self
    }
    /// Creates a new CodeBase from the glob searching the current working directory the app is run
    /// in.
    fn new_from_cwd() -> Self {
        CodeBase {
            source_files: {
                glob("./**/*.rs")
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
                            if raw_line.contents_original.contains(i)
                                && !raw_line.contents_original.contains(&format!("`{}`", i))
                            {
                                let needle = &format!("[`{}`]", i);
                                if let Err(e) = tx_c.send((
                                    raw_line.line_num,
                                    raw_line.contents_original.replace(i, needle).clone(),
                                )) {
                                    eprintln!("Failure to send:{}", e);
                                }
                            }
                        }
                        _ => {}
                    })
            });
        })
    }
    pub fn write(self) {
        let write_buf: Arc<Mutex<HashMap<usize, String>>> = Arc::new(Mutex::new(HashMap::new()));

        let tsrsc = self.clone();

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

        let mut output = String::new();
        let write_buf_complete = write_buf.lock().unwrap();

        let total_lines = self.source_files.iter().map(|sf| sf.total_lines).sum();
        for sf in self.source_files.iter() {
            (0..total_lines).into_iter().for_each(|i| {
                if let Some(m) = write_buf_complete.get(&i) {
                    dbg!(&m);
                    output.push_str(&format!("{}\n", m))
                } else {
                    output.push_str(&format!("{}\n", sf.get(&i).unwrap().contents_original))
                }
            });
        }

        // Assuming the buffers are ordered at this stage...
        eprintln!("{}", "-".repeat(80));
        output.split('\n').for_each(|l| println!("{}", l));
        eprintln!("{}", "-".repeat(80));

        _ = std::fs::File::create("output.rs").unwrap();

        std::fs::write("output.rs", output).unwrap();
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
    use anyhow::Result;
    use glob::glob;

    #[test]
    fn read_sourcecode() {
        _ = RawSourceCode::new_from_file("src/main.rs");
    }

    #[test]
    fn read_lines_of_source() {
        let mut rsc = RawSourceCode::new_from_file("src/search/utils.rs");
        rsc.preview_changes();
    }
    #[test]
    fn write_to_main() {
        let cb: CodeBase = CodeBase {
            source_files: {
                glob("./**/*.rs")
                    .unwrap()
                    .filter_map(Result::ok)
                    .map(|p| RawSourceCode::new_from_file(&p))
                    .collect::<Vec<RawSourceCode>>()
            },
            named_idents: Vec::new(),
        };

        dbg!(&cb.named_idents);

        let write_buf: Arc<Mutex<HashMap<usize, String>>> = Arc::new(Mutex::new(HashMap::new()));
        let wb_c = write_buf.clone();
        let tsrsc = cb.clone();

        let (tx, rx) = mpsc::channel();
        let tx_c = tx.clone();

        let txer = thread::spawn(move || tsrsc.make_adjustments(tx_c));

        let rxer = thread::spawn(move || {
            while let Ok((e, ml)) = rx.recv() {
                wb_c.lock().unwrap().insert(e, ml);
            }
        });

        if let Ok(_) = txer.join() {
            drop(tx);
        }
        rxer.join().expect("unexpected threat failure.");

        // We'll write this...
        let mut output = String::new();

        let write_buf_complete = write_buf.lock().unwrap();

        let total_lines = cb.source_files.iter().map(|sf| sf.total_lines).sum();
        for sf in cb.source_files.iter() {
            (0..total_lines).into_iter().for_each(|i| {
                if let Some(m) = write_buf_complete.get(&i) {
                    output.push_str(&format!("{}\n", m))
                } else {
                    if let Some(co) = sf.get(&i) {
                        output.push_str(&format!("{}\n", co.contents_original))
                    }
                }
            });
        }

        // Assuming the buffers are ordered at this stage...
        eprintln!("{}", "-".repeat(80));
        output.split('\n').for_each(|l| println!("{}", l));
        eprintln!("{}", "-".repeat(80));

        // _ = std::fs::File::create("output.rs").unwrap();
        //
        // std::fs::write("output.rs", output).unwrap();
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
