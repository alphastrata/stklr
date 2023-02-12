use std::{
    collections::HashMap,
    fmt::Display,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};

use crate::utils::read_lines;
use glob::glob;

/// A line from a source file exactly as is.
#[derive(Default, Debug, Clone, Hash)]
pub struct RawLine {
    pub line_num: usize,
    pub contents: String,
    pub source_file: PathBuf,
}

/// All the source code combined!
#[derive(Default, Debug, Clone)]
pub struct SourceTree {
    pub source_files: Vec<RawSourceCode>,
    pub named_idents: Vec<String>,
}

impl SourceTree {
    /// Creates a new [`SourceTree`] from a slice/vec of paths.
    pub fn new_from_paths(paths: &[String]) -> Self {
        SourceTree {
            source_files: paths.iter().map(RawSourceCode::new_from_file).collect(),
            named_idents: Vec::new(),
        }
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
    }
}

#[derive(Default, Debug, Clone)]
pub struct RawSourceCode {
    pub m: HashMap<usize, RawLine>,
    pub file: PathBuf,
    pub total_lines: usize,
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
            total_lines: 0,
        };

        if let Ok(lines) = read_lines(file) {
            lines
                .collect::<Vec<_>>()
                .iter()
                .enumerate()
                .for_each(|(e, l)| {
                    if let Ok(l) = l {
                        let raw_line = RawLine {
                            contents: l.into(),
                            line_num: e,
                            source_file: file.into(),
                            ..Default::default()
                        };
                        raw_source_file.m.insert(e, raw_line);
                    }
                });
        }
        raw_source_file.total_lines = raw_source_file.m.len();
        raw_source_file
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
#[derive(Debug)]
struct SourceCode {
    source_files: Vec<SourceFile>,
}

#[derive(Debug)]
struct SourceFile {
    raw_source_code_lines: Vec<RawSourceCodeLine>,
    idents: HashMap<usize, Ident>,
}

#[derive(Debug)]
struct RawSourceCodeLine {
    line_num: usize,
    line: String,
}

#[derive(Debug)]
enum Ident {
    Constant(String),
    Enum(String),
    ExternCrate(String),
    Function(String),
    Impl,
    Macro,
    Macro2,
    Module,
    Static(String),
    Structure(String),
    Trait(String),
    TraitAlias(String),
    Type(String),
    Union(String),
    Use,
    Verbatim(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_tree() {
        let source_tree = SourceTree::new_from_cwd();
        dbg!(source_tree);
    }
}
