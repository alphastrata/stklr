#![allow(unused_imports)]
#![allow(dead_code)]
use std::fmt::Display;

use crate::search::utils::Flavour;
use crate::search::utils::RawLine;
use crate::search::utils::RawSourceCode;

pub mod external;
pub mod testinate;

/// The 'thing' contained within the outermost~ish set of {} brackets, so:
/// `enum`s `fn`s `struct`s `#[test]`s and so on.
pub struct CodeScope<'rsc> {
    start: usize,
    end: usize,
    flavour: Flavour,
    complete: Vec<&'rsc RawLine>,
}
impl CodeScope<'_> {}

impl Display for CodeScope<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self)
    }
}

pub struct SourceWithScope<'rsc> {
    rsc: &'rsc RawSourceCode,
    scopes: Vec<CodeScope<'rsc>>,
}

//pub m: HashMap<usize, RawLine>,
//pub file: PathBuf,
//pub file_info: FileInfo,
//pub ident_locs: Vec<usize>,
//pub doc_locs: Vec<usize>,
//pub total_lines: usize,
//pub named_idents: Vec<String>,
impl SourceWithScope<'_> {
    fn init(rsc: RawSourceCode) -> Self {
        //rsc.m.iter().map(|(line_num, rl)  )
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore]
    fn capture_scopes() {
        let _sample = r#"impl SourceTree {
    /// Populates the idents we care about...
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
    /// Creates a new [`SourceTree`] from a slice/vec of paths.
    pub fn new_from_paths(paths: &[String]) -> Self {
        SourceTree {
            source_files: paths.iter().map(RawSourceCode::new_from_file).collect(),
            named_idents: Vec::new(),
        }
        .populate_idents()
    }
    /// Creates a new [`SourceTree`] `Result` the glob search the current working directory the app is run
    /// in.
    pub fn new_from_cwd() -> Self {
        let path = std::env::current_dir().expect("Unable to ascertain current working directory, this is likely a permissions error with your OS.");

        Self::new_from_dir(format!("{}", path.as_path().display()))
    }
    /// Creates a new [`SourceTree`] `Result` a given directory.
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
    /// Commits changes to disk, essentially writing the [`AdjustedLine`] back to a `Result` of
    /// the same name, line-by-line.
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


// Boilerplates....
impl Display for AdjustedLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", &self.line_num, &self.contents,)
    }
}"#;
    }
}
