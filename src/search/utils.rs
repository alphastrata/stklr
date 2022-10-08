use super::consts::*;

use core::fmt::Display;
use glob::glob;
use log::debug;
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
    pub idents: Vec<String>,
    pub source_file: PathBuf,
}

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

    let alwayses = [
        "i8", "u8", "i16", "u16", "i32", "u32", "i64", "u64", "i128", "u128", "isize", "usize",
    ];

impl SourceTree {
    /// Populates the idents we care about...
    fn populate_idents(mut self) -> Self {
        self.source_files.iter().for_each(|sf| {
            sf.named_idents.iter().for_each(|e| {
                if !NEVERS.contains(&&e[..]) {
                    debug!("{}::{}", sf.file.display(), e);
                    self.named_idents.push(e.to_string())
                }
            });
        });
        self
    }

    /// Creates a [`new`] [`SourceTree`] [`from`] a collection of [`path`] to source files.
    pub fn new_from_paths(paths: &[String]) -> Self {
        SourceTree {
            source_files: paths
                .iter()
                .map(|p| RawSourceCode::new_from_file(p))
                .collect(),
            named_idents: Vec::new(),
        }
        .populate_idents()
    }
    /// Creates a [`new`] [`SourceTree`] [`from`] the [`glob`] [`search`] the current working directory the app is run
    /// in.
    pub fn new_from_cwd() -> Self {
        let path = std::env::current_dir().expect("Unable to ascertain current working directory, this is likely a permissions error with your OS.");

        Self::new_from_dir(format!("{}", path.as_path().display()))
    }
    /// Creates a [`new`] [`SourceTree`] [`from`] a given directory.
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
    /// Commits changes to disk, essentially writing the [`AdjustedLine`] back to a [`path`] of
    /// the same name, line-by-line.
    pub fn write_changes(file: PathBuf, changes: &mut [AdjustedLine], write_flag: bool) {
        debug!("SourceTree::write_changes was called");
        changes.sort_by(|a, b| a.line_num.cmp(&b.line_num));
        let output: Vec<String> = changes
            .iter_mut()
            .map(|adj_line| adj_line.contents.to_owned())
            .collect();

        if write_flag {
            fs::write(&file, output.join("\n")).unwrap(); //FIXME:
            debug!("Write successful.")
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
                        raw_source_file
                            .named_idents
                            .extend(raw_line.idents.iter().cloned());
                        raw_source_file.m.insert(e, raw_line);
                    }
                });
        }
        raw_source_file.total_lines = raw_source_file.m.len();
        raw_source_file.named_idents.dedup();
        raw_source_file.named_idents.retain(|x| x != "");
        raw_source_file
    }

    /// Checks whether `self` [`should_be_modified`] and if so, [`process`] [`from`] the passed
    /// `idents` is called.
    pub fn make_adjustments(&self, idents: &[String]) -> Vec<AdjustedLine> {
        self.m
            .iter()
            .filter(|(_, raw_line)| raw_line.should_be_modified(idents))
            .map(|(_, f)| -> RawLine { f.to_owned().process_changes(idents) })
            .map(|rpl| -> AdjustedLine { rpl.into() })
            .collect::<Vec<AdjustedLine>>()
    }
}

impl RawLine {
    /// Will return true for a SINGLE instance of when a modification should be made, how many may
    /// really be in there is the domain of [`process`]
    #[inline(always)]
    fn should_be_modified(&self, idents: &[String]) -> bool {
        // NOTE: this isn't as bad as you'd initially think, you're out at the first branch if it's
        // not a docstring, or, out at the first 'hit'.
        if matches!(self.flavour, Flavour::Docstring) {
            for i in idents {
                if self.contents.contains(i)
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
    /// Actually process the modifications to a [`RawLine`] contents_modified
    fn process_changes(mut self, idents: &[String]) -> Self {
        for id in idents {
            let split_n_proc = &self
                .contents
                .split_whitespace()
                .map(|sp| {
                    if sp.contains(id) {
                        debug!("{} found in: {}", id, sp);
                        format!(" [`{}`]", id)
                    } else {
                        debug!("No change to: {} ", sp);
                        format!(" {}", sp)
                    }
                })
                .collect::<String>();
            self.contents = split_n_proc.to_string();
        }
        self
    }
    /// Finds the things we're interested in.
    fn find_idents(&mut self) {
        let text = self.contents.to_owned();

        // DRYness, is goodness.
        macro_rules! generate_ident_find_loops {
            ($($CONST:ident),*) => {
                $(for caps in $CONST.captures_iter(&text) {
                    if let Some(v) = caps.name("ident") {
                        let cap = v.as_str().to_string();
                        //TODO: You've got more flavours, use them...
                        self.flavour = Flavour::Declare;
                        if cap != ""{
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
            RUST_TRAIT,
            RUST_USE,
            RUST_IMPORT
        );
    }

    /// Find and classify docstrings.
    // NOTE: also regex controlled, see ./src/search/consts.rs
    // TODO: add support for '//!' module docstrings.
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
        //let st = SourceTree::new_from_cwd();
        for rsc in st.source_files.iter() {
            //rsc.make_adjustments(&rsc.named_idents);
            debug!("{}", rsc.file.display());
            let new_m = rsc
                .make_adjustments(&rsc.named_idents)
                .into_iter()
                .map(|adj| (adj.line_num, adj.contents))
                .collect::<HashMap<usize, String>>();

            let output = (0..rsc.total_lines)
                .into_iter()
                .map(|n| -> String {
                    if let Some(new) = new_m.get(&n) {
                        //dbg!(&new);
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
}
