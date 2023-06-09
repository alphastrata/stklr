use crate::search::consts::*;

use anyhow::Result;
use core::fmt::Display;
use glob::glob;
use log::{debug, error};
use std::{
    collections::HashMap,
    fs,
    hash::Hash,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};

/// Are all the available changes to a line done? not done? etc.
#[derive(Default, Debug, Clone, Hash)]
pub enum Linked {
    Complete,
    Loaded,
    Partial,
    Irrelevant,
    #[default]
    Unprocessed,
}
/// A way to describe lines of code based on what they are/do etc.
#[derive(PartialEq, Default, Debug, Clone, Hash)]
#[allow(non_camel_case_types)]
pub enum Flavour {
    RUST_DOCS,
    RUST_FN,
    RUST_TY,
    RUST_ENUM,
    RUST_STRUCT,
    RUST_IMPORT,
    RUST_USE,
    RUST_TRAIT,
    #[default]
    Tasteless,
}
/// A line from a source file exactly as is.
#[derive(Default, Debug, Clone, Hash)]
pub struct RawLine {
    pub line_num: usize, //NOTE: indentionally duplicate data
    pub all_linked: Linked,
    pub contents: String,
    pub flavour: Flavour,
    pub idents: Vec<String>,
    pub source_file: PathBuf,
}

/// A line from a source file with its contents modified by this app.
#[derive(PartialEq, Eq, PartialOrd, Debug, Clone, Hash)]
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
    /// Populates the idents
    fn populate_idents(mut self) -> Self {
        self.source_files.iter().for_each(|sf| {
            sf.named_idents
                .iter()
                .for_each(|e| self.named_idents.push(e.to_string()));
        });
        self
    }

    pub fn setup_tree(paths: &Option<Vec<String>>) -> SourceTree {
        if let Some(paths) = paths {
            SourceTree::new_from_paths(paths)
        } else {
            SourceTree::new_from_cwd()
        }
    }
    /// Creates a new [`SourceTree`] from Path.
    pub fn new_from_paths(paths: &[String]) -> Self {
        SourceTree {
            source_files: paths.iter().map(RawSourceCode::new_from_file).collect(),
            named_idents: Vec::new(),
        }
        .populate_idents()
    }
    /// Creates a new [`SourceTree`] from cwd.
    pub fn new_from_cwd() -> Self {
        let path = std::env::current_dir().expect("Unable to ascertain current working directory, this is likely a permissions error with your OS.");

        Self::new_from_dir(format!("{}", path.as_path().display()))
    }
    /// Creates a new [`SourceTree`] from a given directory.
    pub fn new_from_dir<P>(dir: P) -> Self
    where
        P: Display + AsRef<Path>,
    {
        let search_path = format!("{}/**/*.rs", dir);

        SourceTree {
            source_files: {
                glob(&search_path)
                    .unwrap()
                    .filter_map(Result::ok)
                    .filter(|f| !f.display().to_string().contains("target/"))
                    .map(|p| RawSourceCode::new_from_file(&p))
                    .collect::<Vec<RawSourceCode>>()
            },
            named_idents: Vec::new(),
        }
        .populate_idents()
    }
    /// Commits changes to disk, essentially writing the [`AdjustedLine`]
    pub fn write_changes(file: PathBuf, changes: &mut [AdjustedLine], write_flag: bool) {
        debug!("SourceTree::write_changes was called");
        changes.sort_by(|a, b| a.line_num.cmp(&b.line_num));

        let output: Vec<String> = changes
            .iter_mut()
            .map(|adj_line| adj_line.contents.to_owned())
            .collect();

        if write_flag {
            if let Ok(_) = fs::write(&file, output.join("\n")) {
                debug!("Write successful.")
            } else {
                error!("Write unsuccessful for:{}", file.display());
            }
        } else {
            changes.iter().for_each(|e| println!("{}", e));
        }
    }
}

/// ~Most of the info we require/use about a file of raw rust source code.
#[derive(Default, Debug, Clone)]
pub struct RawSourceCode {
    /// Holding line-number, line-of-code pairs
    pub m: HashMap<usize, RawLine>,
    /// The Path on disk this all comes from
    pub file: PathBuf,
    /// Information about the file, like ctime/mtime etc
    // pub file_info: FileInfo,
    /// line-numbers of found idents
    pub ident_locs: Vec<usize>,
    /// line-numbers that we found /// docstrings on
    pub doc_locs: Vec<usize>,
    /// Total number of loc in this file.
    pub total_lines: usize,
    /// All the idents we've found in String form.
    pub named_idents: Vec<String>,
}

impl RawSourceCode {
    /// Produce a new RawSourceCode from a file.
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
            // file_info: FileInfo::init(&file.into()).unwrap(),
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
                            source_file: file.into(),
                            ..Default::default()
                        };

                        raw_line.find_docs();
                        raw_line.find_idents();

                        raw_source_file
                            .named_idents
                            .extend(raw_line.idents.iter().cloned());

                        raw_source_file.m.insert(e, raw_line);
                    }
                });
        }
        raw_source_file.total_lines = raw_source_file.m.len();
        raw_source_file.named_idents.dedup();
        raw_source_file.named_idents.retain(|x| !x.is_empty());
        raw_source_file
    }

    /// Checks whether `self` [`should_be_modified`] and if so, [`process`] is called.
    pub fn make_adjustments<'a>(
        &'a mut self,
        idents: &'a [String],
    ) -> impl Iterator<Item = AdjustedLine> + '_ {
        self.m
            .iter_mut()
            .filter(|(_, raw_line)| raw_line.should_be_modified(idents))
            .map(|(_, f)| {
                f.process_changes(idents);
                f
            })
            .map(|f| -> RawLine { f.to_owned() })
            .map(move |rpl| -> AdjustedLine { rpl.into() })
    }
}

#[derive(Debug, Default)]
pub struct ReportCard {
    pub source_files: Vec<RawSourceCode>,
    pub named_idents: Vec<String>,
    pub num_funcs: usize,
    pub num_pub_funcs: usize,

    pub num_structs: usize,
    pub num_pub_structs: usize,

    pub num_enums: usize,
    pub num_pub_enums: usize,

    pub num_types: usize,
    pub num_pub_types: usize,

    pub num_traits: usize,
    pub num_pub_traits: usize,

    pub num_macros: usize,
}

impl ReportCard {
    /// Produce a ReportCard from A SourceTree
    pub fn from_source_tree(st: SourceTree) -> Self {
        let mut rc = ReportCard::default();
        st
            .source_files
            .iter()
            .map(|rsc| rc.process(rsc))
            .collect::<()>();;

        rc.source_files = st.source_files;
        rc
    }

    /// Process a ReportCard from the &RawSourceCode in Self
    pub fn process(&mut self, rsc: &RawSourceCode) {
        rsc
            .iter()
            .flat_map(|(_k, v)| v.report(self))
            .collect::<()>();;
    }

    //TODO: DRY this up...
    //TODO: colourise output.
    pub fn pretty_print(&self) {
        println!("REPORT:");
        println!(" enums     : {}", self.num_enums + self.num_pub_enums);
        println!(" functions : {}", self.num_funcs + self.num_pub_funcs);
        println!(" structs   : {}", self.num_structs + self.num_pub_structs);
        println!(" traits    : {}", self.num_traits + self.num_pub_traits);
        println!(" types     : {}", self.num_types + self.num_pub_types);

        println!("\nPUBLIC:");
        //TODO: this is awful!
        //TODO: colourise output.
        //TODO: macro this!
        if self.num_funcs > 1 {
            let percentage_public = self.num_funcs / self.num_pub_funcs;
            println!(" functions : {}", percentage_public);
        }

        if self.num_structs > 1 {
            let percentage_public = self.num_structs / self.num_pub_structs;
            println!(" structs   : {}", percentage_public);
        }

        if self.num_traits > 1 {
            let percentage_public = self.num_traits / self.num_pub_traits;
            println!(" traits    : {}", percentage_public);
        }

        if self.num_types > 1 {
            let percentage_public = self.num_types / self.num_pub_types;
            println!(" types     : {}", percentage_public);
        }

        // println!("FILE:");
        // self.source_files.iter().for_each(|rsc| {
        //     println!("FILE: {}", rsc.file.display());
        //     println!(
        //         "last checked vs mtime: {:?}s",
        //         rsc.file_info
        //             .last_checked
        //             .duration_since(rsc.file_info.mtime)
        //             .unwrap()
        //             .as_secs_f64()
        //     );
        // })
    }
}

impl RawLine {
    /// Will return true for a SINGLE instance of when a modification should be made, how many may
    /// really be in there is the domain of [`process`]
    fn should_be_modified(&self, idents: &[String]) -> bool {
        if matches!(self.flavour, Flavour::RUST_DOCS) {
            for i in idents {
                if self.contents.contains(i)
                    || self.contents.contains(&format!("{}s", i))
                    || self.contents.contains(&format!("{}.", i))
                    || self.contents.contains(&format!("{}'s", i)) && self.contents.contains("///")
                    || self.contents.contains("//!")
                {
                    return true;
                }
            }
        }
        false
    }

    /// Used by the [`report`] functionality.
    fn pub_or_private(&self) -> bool {
        self.contents.contains("pub")
    }
    /// WIP!
    /// Produce a [`report`] on the source at hand..
    fn report(&self, rc: &mut ReportCard) -> Result<()> {
        //TODO: macro this
        match self.flavour {
            Flavour::RUST_FN => {
                if self.pub_or_private() {
                    rc.num_pub_funcs += 1
                } else {
                    rc.num_funcs += 1
                }
            }
            Flavour::RUST_TY => {
                if self.pub_or_private() {
                    rc.num_pub_types += 1
                } else {
                    rc.num_types += 1
                }
            }

            Flavour::RUST_ENUM => {
                if self.pub_or_private() {
                    rc.num_pub_enums += 1
                } else {
                    rc.num_enums += 1
                }
            }
            Flavour::RUST_TRAIT => {
                if self.pub_or_private() {
                    rc.num_pub_types += 1
                } else {
                    rc.num_traits += 1
                }
            }
            Flavour::RUST_STRUCT => {
                if self.pub_or_private() {
                    rc.num_pub_structs += 1
                } else {
                    rc.num_structs += 1
                }
            }
            _ => (),
        }

        Ok(())
    }
    /// Actually [`process`] the modifications to a [`RawLine`] contents.
    fn process_changes(&mut self, idents: &[String]) {
        idents.iter().for_each(|id| {
            self.contents = self.contents.replace(';', "");
            self.contents = self
                .contents
                .split_whitespace()
                .map(|sp| {
                    if ALWAYS.contains(&sp) {
                        debug!("{} found always in: {}", id, sp);
                        format!(" `{}`", id)
                    }
                    // Handle the captures.
                    else if sp.contains(id) && sp.len() > 3 {
                        debug!("{} found cap in: {}", id, sp);
                        format!(" [`{}`]", id)
                    }
                    // Unchanged...
                    else {
                        debug!("No change to: {} ", sp);
                        format!(" {}", sp)
                    }
                })
                .collect::<String>()
                ;
        });
    }
    /// Finds the things we're interested in.
    pub fn find_idents(&mut self) {
        let text = self.contents.to_owned();

        // DRYness, is goodness.
        macro_rules! generate_ident_find_loops {
            ($($CONST:ident),*) => {
                $(for caps in $CONST.captures_iter(&text) {
                    if let Some(v) = caps.name("ident") {
                        let cap = v.as_str().to_string();
                        //TODO: You've got more flavours, use them...
                        self.flavour = Flavour::$CONST;
                        if cap != "" && !NEVERS.iter().any(|c| c == &cap) && cap.len() > 2{
                            self.idents.push(cap.clone());
                        }
                    }
                })*
            };
        }

        // generate a matcher for all of these.
        generate_ident_find_loops!(
            RUST_FN,
            RUST_TY,
            RUST_ENUM,
            RUST_STRUCT,
            RUST_IMPORT,
            RUST_USE,
            RUST_TRAIT
        );
    }

    /// Find and classify docstrings.
    // NOTE: also regex controlled, see ./src/search/consts.rs
    // TODO: add support for '//!' module docstrings.
    fn find_docs(&mut self) {
        let text = self.contents.to_owned();
        for caps in RUST_DOCSTRING.captures_iter(&text) {
            if caps.name("ident").is_some() {
                self.flavour = Flavour::RUST_DOCS;
            }
        }
    }
}

// Boilerplates....
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
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::cli::Cli;
    use crate::cmd::jobs::run_report;

    #[test]
    #[ignore]
    fn trial_on_source() {
        let t1 = std::time::Instant::now();

        let mut st = SourceTree::new_from_dir("/media/jer/ARCHIVE/scrapers/rustwari");

        for rsc in st.source_files.iter_mut() {
            debug!("{}", rsc.file.display());
            let named_idents = &rsc.named_idents.clone();
            let new_m = rsc
                .make_adjustments(named_idents)
                .map(|adj| (adj.line_num, adj.contents))
                .collect::<HashMap<usize, String>>();

            let output = (0..rsc.total_lines)
                .map(|n| -> String {
                    if let Some(new) = new_m.get(&n) {
                        new.to_owned()
                    } else {
                        let new = rsc.get(&n).unwrap().contents.to_owned();
                        new
                    }
                })
                .collect::<Vec<String>>();

            _ = std::fs::write(&rsc.file, output.join("\n"));
        }

        debug!(
            "{} FILES IN: {}s",
            st.source_files.len(),
            t1.elapsed().as_secs_f64()
        );
    }

    #[test]
    fn report_on_rustwari() {
        let t1 = std::time::Instant::now();

        let st = SourceTree::new_from_dir("/media/jer/ARCHIVE/scrapers/rustwari");
        _ = run_report(&None, &Cli::default());

        debug!(
            "{} FILES IN: {}s",
            st.source_files.len(),
            t1.elapsed().as_secs_f64()
        );
    }
}
