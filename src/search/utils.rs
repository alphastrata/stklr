use super::consts::*;
use crate::termite;

use core::fmt::Display;
use glob::glob;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use std::collections::HashMap;
use std::fs;
use std::hash::Hash;
use std::ops::Deref;
use std::ops::DerefMut;
use std::path::{Path, PathBuf};

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
    pub contents: String,
    pub flavour: Flavour,
    pub idents: Vec<String>, //NOTE: expended data (exists temporarily to populate the SourceCode struct(s))
    pub source_file: PathBuf,
}

#[derive(Default, Debug, Clone, Hash)]
pub struct AdjustedLine {
    pub line_num: usize,
    pub contents: String,
    pub source_file: PathBuf,
}

impl From<RawLine> for AdjustedLine {
    fn from(line: RawLine) -> Self {
        Self {
            line_num: line.line_num,
            contents: line.contents,
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
                debug!("{}::{}", sf.file.display(), e);
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
    /// Commits changes to disk, essentially writing the `Vec<AdjustedLine>` back to a file_path of
    /// the same name, line-by-line.
    pub fn write_changes(file: PathBuf, changes: &mut Vec<AdjustedLine>, write_flag: bool) {
        changes.sort_by(|a, b| a.line_num.cmp(&b.line_num));
        let output: Vec<String> = changes
            .into_iter()
            .map(|adj_line| adj_line.contents.to_owned())
            .collect();

        if write_flag {
            _ = fs::write(&file, output.join("\n")).unwrap(); //FIXME:
        } else {
            changes.iter().for_each(|e| println!("{}", e));
        }
    }
}

impl Display for AdjustedLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", &self.line_num, &self.contents,)
    }
}
impl Display for RawLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", &self.line_num, &self.contents)
    }
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

#[derive(Default, Debug, Clone)]
pub struct RawSourceCode {
    pub m: HashMap<usize, RawLine>,
    pub file: PathBuf,
    pub ident_locs: Vec<usize>,
    pub doc_locs: Vec<usize>,
    pub total_lines: usize,
    pub named_idents: Vec<String>,
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
                            contents: l.into(),
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

    /// Checks whether `self` [`should_be_modified`] and if so, [`process_changes`] from the passed
    /// `idents` is called.
    fn make_adjustments(&self, idents: &[String]) -> Vec<AdjustedLine> {
        self.m
            .iter()
            .filter(|(_, raw_line)| raw_line.should_be_modified(idents))
            .map(|(_, f)| f.clone().process_changes(idents))
            .map(|rpl| rpl.clone().into())
            .collect::<Vec<AdjustedLine>>()
    }
}

//-------
impl RawLine {
    /// Will return true for a SINGLE instance of when a modification should be made, how many may
    /// really be in there is the domain of [`process_changes`]
    fn should_be_modified(&self, idents: &[String]) -> bool {
        // NOTE: this isn't as bad as you'd initially think, you're out at the first branch if it's
        // not a docstring, or, out at the first 'hit'.
        if matches!(self.flavour, Flavour::Docstring) {
            for i in idents {
                if self.contents.contains(i) && !self.contents.contains(&format!("`{}`", i))
                    || self.contents.contains(&format!("{}s", i))
                    || self.contents.contains(&format!("{}.", i))
                    || self.contents.contains(&format!("{}'s", i)) && self.contents.contains("///")
                {
                    return true;
                }
            }
        }
        false
    }
    /// Actually processes the modifications to a RawLine's contents_modified
    fn process_changes(mut self, idents: &[String]) -> Self {
        // // Account for the `matches` call's not handling out-of-bounds
        // let padded_i = format!(" {} ", i);
        // let padded_self = format!(" {} ", self.contents);
        //
        // // Account for abnormal docstring end chars like '.'
        // let mut changes_to_make: Vec<&str> = padded_self.matches(&padded_i).collect();
        // if changes_to_make.is_empty() {
        //     changes_to_make = self.contents.matches(i).collect();
        // }
        //
        // let needle = &format!("[`{}`]", i);
        //
        // self.contents = self
        //     .contents
        //     .clone()
        //     .replacen(i, needle, changes_to_make.len());
        // self
        ////v2:
        for id in idents {
            let split_n_proc = &self
                .contents
                .split_whitespace()
                .map(|sp| {
                    if sp.contains(id) {
                        format!(" [`{}`]", id)
                    } else {
                        format!(" {}", sp)
                    }
                })
                .collect::<String>();
            self.contents = split_n_proc.to_string();
        }
        self
    }
    pub fn process(&mut self) {
        self.find_docs();
        self.find_idents();
        self.idents.dedup();
    }
    //TODO: Make this DRY with a macro
    fn find_idents(&mut self) {
        let text = self.contents.to_owned();
        macro_rules! generate_ident_find_loops {
            ($($CONST:ident),*) => {
                $(for caps in $CONST.captures_iter(&text) {
                    if let Some(v) = caps.name("ident") {
                        let cap = v.as_str().to_string();
                        //trace!("{}::{}", self.line_num, cap);
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
        let text = self.contents.to_owned();
        for caps in RUST_DOCSTRING.captures_iter(&text) {
            if let Some(_) = caps.name("ident") {
                self.flavour = Flavour::Docstring;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn trial_on_source() {
        let t1 = std::time::Instant::now();

        let st = SourceTree::new_from_dir("/media/jer/ARCHIVE/scrapers/rustwari");
        for rsc in st.source_files.iter() {
            //rsc.make_adjustments(&rsc.named_idents);
            info!("{}", rsc.file.display());
            rsc.make_adjustments(&rsc.named_idents)
                .iter()
                .for_each(|al| info!("{}", al));
        }

        info!(
            "{} FILES IN: {}s",
            st.source_files.len(),
            t1.elapsed().as_secs_f64()
        );
    }

    // #[test]
    // fn handle_imports() {
    //     let example = r#"
    //                     use anyhow::Result;
    //                     use STKLR::search::utils::SourceTree;
    //                     use STKLR::search::utils::RawSourceCode;
    //                     }"#;
    //
    //     let mut raw_line = RawLine {
    //         contents: example.into(),
    //         ..Default::default()
    //     };
    //     raw_line.process();
    //     assert_eq!(raw_line.idents.len(), 9);
    // }
    // #[test]
    // fn handle_lifetime_annotations() {
    //     let example = r#"async fn servitude<'a>(a: &'a str) -> bool{}"#;
    //     let mut raw_line = RawLine {
    //         contents: example.to_string(),
    //         ..Default::default()
    //     };
    //     raw_line.process();
    //     assert_eq!("servitude".to_string(), raw_line.idents[0])
    // }
    //#[test]
    //fn handle_traits() {
    //    let example = r#"pub trait Bless {
    //    fn bless(&P) -> Blessing
    //    where P: Clone + Send + Sync {
    //    };
    //    }"#;
    //    let mut raw_line = RawLine {
    //        contents: example.into(),
    //        ..Default::default()
    //    };
    //    raw_line.process();
    //    assert_eq!(raw_line.idents[1], "Bless");
    //    assert_eq!(raw_line.idents[0], "bless");
    //}
    //#[test]
    //fn multi_matches_single_line() {
    //    let example =
    //        r#"a preview_changes to be linked, and another preview_changes here linked too."#;
    //    let raw_line = RawLine {
    //        contents: example.to_string(),
    //        idents: vec!["preview_changes".into()],
    //        ..Default::default()
    //    };

    //    let expected =
    //        "a [`preview_changes`] to be linked, and another [`preview_changes`] here linked too.";
    //}
    // #[test]
    // fn fullstop_after_ident() {
    //     let example = r#"a preview_changes to the mighty SourceTree."#;
    //     let mut raw_line = RawLine {
    //         contents: example.to_string(),
    //         idents: vec!["preview_changes".into(), "SourceTree".into()],
    //         ..Default::default()
    //     };
    // }
    // #[test]
    // fn ident_has_apos_s() {}
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
