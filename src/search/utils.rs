#![allow(unused_imports)]
#![allow(dead_code)]

use super::consts::*;
use crate::termite;

use core::fmt::Display;
use glob::glob;
use log::{debug, error, info, trace, warn};
use std::collections::HashMap;
use std::fs;
use std::hash::Hash;
use std::ops::Deref;
use std::ops::DerefMut;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Default, Debug, Clone, Hash)]
pub enum Linked {
    Complete,
    Loaded,
    Partial,
    Irrelevant,
    #[default]
    Unprocessed,
}
#[derive(Default, Debug, Clone, Hash)]
pub enum Flavour {
    Docstring,
    Declare,
    #[default]
    Tasteless,
}
#[derive(Default, Debug, Clone, Hash)]
pub struct RawLine {
    pub line_num: usize, //NOTE: indentionally duplicate data
    pub all_linked: Linked,
    pub contents_original: String,
    pub flavour: Flavour,
    pub idents: Vec<String>, //NOTE: expended data (exists temporarily to populate the SourceCode struct(s))
    pub source_file: PathBuf,
}

#[derive(Default, Debug, Clone, Hash)]
struct AdjustedLine {
    pub line_num: usize,
    pub contents_original: String,
    pub source_file: PathBuf,
}

impl From<RawLine> for AdjustedLine {
    fn from(line: RawLine) -> Self {
        Self {
            line_num: line.line_num,
            contents_original: line.contents_original,
            source_file: line.source_file,
        }
    }
}

/// All the source code combined!
#[derive(Default, Debug, Clone)]
pub struct SourceTree {
    pub source_files: Vec<RawSourceCode>,
    pub named_idents: Vec<String>,
}

impl SourceTree {
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
    /// Creates a new SourceTree from the glob searching the current working directory the app is run
    /// in.
    pub fn new_from_cwd() -> Self {
        Self::new_from_dir(".")
    }
    /// Creates a new SourceTree from a given directory.
    pub fn new_from_dir<P>(dir: P) -> Self
    where
        P: Display + AsRef<Path>,
    {
        _ = termite::setup_logger().unwrap();

        let search_path = format!("{}/**/*.rs", dir);
        SourceTree {
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
    fn make_adjustments(&self, tx: Sender<(RawLine, String)>) {
        let tx_c = tx.clone();
        self.source_files.iter().for_each(|sf| {
            sf.named_idents.iter().cloned().for_each(|i| {
                sf.iter()
                    .for_each(|(_, raw_line)| match raw_line.flavour {
                        Flavour::Docstring => {
                            //BUG: haven't accounted for multiple occurences, or a nested
                            //occurence.
                            if raw_line.should_be_modified(&i) {
                                let mut raw_line = raw_line.clone();
                                    raw_line.contents_original = raw_line.process_changes(&i);

                                tx_c.send(
                                    (raw_line.to_owned(),
                                    sf.file.display().to_string())
                                )
                                .expect("Unable to send line on channel, something has gone HORRIBLY WRONG!");
                            }
                        }
                        _ => {
                            panic!("This shoudn'd be reachable...");
                        }
                    });
            });
        })
    }

    pub fn write_changes(self, write_flag: bool) {
        // Adjustment thread -> sends to builder thread
        let tsrsc = self.clone();
        let (tx_adj_2_build, rx_build) = mpsc::channel();
        let tx_adj_c = tx_adj_2_build.clone();
        let adjust_t = thread::spawn(move || tsrsc.make_adjustments(tx_adj_c));

        // Builder thread -> sends to writer
        let (tx_build, rx_writer) = mpsc::channel();

        // THE RX_BUILD gets values here, from tsrsc.make_adjustments(tx_c)
        let builder_t = thread::spawn(move || {
            let mut sf_hm: HashMap<String, Vec<AdjustedLine>> = HashMap::new();
            while let Ok((rl, filepath)) = rx_build.recv() {
                sf_hm.entry(filepath).and_modify(|v| v.push(rl.into()));
            }
            tx_build
                .send(sf_hm)
                .expect("unable to send complete sf_hm to writer!")
        });

        if let Ok(_) = adjust_t.join() {
            drop(tx_adj_2_build);
        }

        if let Ok(_) = builder_t.join() {
            // ALL THE LINES ARE IN SF_HM
        }

        // THE WRITER thread is actually *this* one, the main one.
        while let Ok(sf_hm) = rx_writer.recv() {
            // sf_hm is a hashmap of filepath, adjusted_line

            sf_hm.iter().for_each(|(sf_path, v)| {
                let mut output: Vec<&str> = Vec::new();
                for (e, _) in v.iter().enumerate() {
                    let current = &v[e];
                    output.push(&current.contents_original)
                }

                // Write 'output'
                if write_flag {
                    fs::write(&sf_path, output.join("\n"))
                        .expect("problem writing output to filepath")
                }
            });
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
        let mut raw_source_file = RawSourceCode {
            m: HashMap::new(),
            file: PathBuf::from(file),
            ident_locs: Vec::new(),
            doc_locs: Vec::new(),
            total_lines: 0,
            named_idents: Vec::new(),
        };

        if let Ok(lines) = crate::read_lines(file) {
            lines
                .collect::<Vec<_>>()
                .iter()
                .enumerate()
                .for_each(|(e, l)| {
                    if let Ok(l) = l {
                        let mut raw_line = RawLine {
                            all_linked: Linked::Unprocessed,
                            contents_original: l.into(),
                            line_num: e,
                            ..Default::default()
                        };
                        raw_line.find_docs();
                        raw_line.find_idents();
                        if raw_line.idents.len() > 0 {
                            raw_source_file
                                .named_idents
                                .extend(raw_line.idents.iter().cloned())
                        }
                        raw_source_file.m.insert(e, raw_line);
                    }
                });
        }
        raw_source_file.total_lines = raw_source_file.m.len();
        raw_source_file.named_idents.dedup();
        raw_source_file.named_idents.retain(|x| x != ""); //TODO: can you initialise this better so avoid this?
        raw_source_file
    }
}

impl RawLine {
    /// Will return true for a SINGLE instance of when a modification should be made, how many may
    /// really be in there is the domain of [`process_changes`]
    fn should_be_modified(&self, i: &str) -> bool {
        if self.contents_original.contains(i)
            || self.contents_original.contains(&format!("{i}s"))
            || self.contents_original.contains(&format!("{i}."))
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
        // Account for the `matches` call's not handling out-of-bounds
        let padded_i = format!(" {} ", i);
        let padded_self = format!(" {} ", self.contents_original);

        // Account for abnormal docstring end chars like '.'
        let mut changes_to_make: Vec<&str> = padded_self.matches(&padded_i).collect();
        if changes_to_make.is_empty() {
            changes_to_make = self.contents_original.matches(i).collect();
        }

        let needle = &format!("[`{}`]", i);

        dbg!(&padded_i);
        dbg!(&padded_self);
        dbg!(&changes_to_make);
        dbg!(&needle);

        self.contents_original
            .clone()
            .replacen(i, needle, changes_to_make.len())
    }
    fn process(&mut self) {
        self.find_docs();
        self.find_idents();
        self.idents.dedup();
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
                        self.idents.push(cap);
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
                        use STKLR::search::utils::SourceTree;
                        use STKLR::search::utils::RawSourceCode;
                        }"#;

        let mut raw_line = RawLine {
            contents_original: example.into(),
            ..Default::default()
        };
        raw_line.process();
        assert_eq!(raw_line.idents.len(), 9);
    }
    #[test]
    fn handle_lifetime_annotations() {
        let example = r#"async fn servitude<'a>(a: &'a str) -> bool{}"#;
        let mut raw_line = RawLine {
            contents_original: example.to_string(),
            ..Default::default()
        };
        raw_line.process();
        assert_eq!("servitude".to_string(), raw_line.idents[0])
    }
    #[test]
    fn handle_traits() {
        let example = r#"pub trait Bless { 
        fn bless(&P) -> Blessing 
        where P: Clone + Send + Sync {
        };
        }"#;
        let mut raw_line = RawLine {
            contents_original: example.into(),
            ..Default::default()
        };
        raw_line.process();
        assert_eq!(raw_line.idents[1], "Bless");
        assert_eq!(raw_line.idents[0], "bless");
    }
    #[test]
    fn multi_matches_single_line() {
        let example =
            r#"a preview_changes to be linked, and another preview_changes here linked too."#;
        let raw_line = RawLine {
            contents_original: example.to_string(),
            idents: vec!["preview_changes".into()],
            ..Default::default()
        };
        let res = raw_line.process_changes(&raw_line.idents[0]);
        let expected =
            "a [`preview_changes`] to be linked, and another [`preview_changes`] here linked too.";
        assert_eq!(expected, &res);
    }
    #[test]
    fn trial_on_this_source() {
        let st = SourceTree::new_from_cwd();
        st.write_changes(false);
    }

    #[test]
    fn fullstop_after_ident() {
        let example = r#"a preview_changes to the mighty SourceTree."#;
        let mut raw_line = RawLine {
            contents_original: example.to_string(),
            idents: vec!["preview_changes".into(), "SourceTree".into()],
            ..Default::default()
        };
        for id in raw_line.idents.iter() {
            dbg!("loop", &id);
            if raw_line.should_be_modified(id) {
                raw_line.contents_original = raw_line.clone().process_changes(&id).to_string();
                dbg!(&raw_line.contents_original);
            }
        }
    }
    #[test]
    fn ident_has_apos_s() {}

    // fn show_matched_idents() {
    //     for path in glob("./**/*.rs").unwrap().filter_map(Result::ok) {
    //         let p = path;
    //         let rsc = RawSourceCode::new_from_file(&p);
    //         if !rsc.named_idents.is_empty() {
    //             dbg!(rsc.named_idents);
    //         }
    //     }
    // }
}
