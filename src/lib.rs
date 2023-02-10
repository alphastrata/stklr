#![allow(non_snake_case)]
#![allow(unused_must_use)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
use anyhow::Result;
use glob::glob;
use std::collections::HashMap;
use std::path::Path;

/// Gives you a wrapped Arc<RwLock<Node>>
#[macro_export]
macro_rules! quick_node {
    ($val:expr) => {{
        let node = Node::new($val.to_string());
        Arc::new(RwLock::new(node))
    }};
}

#[macro_export]
/// Unwraps unsafely a .read() lock
macro_rules! open {
    ($e:expr) => {{
        let inner = $e.read().unwrap();
        *inner
    }};
}

#[macro_export]
/// Unsafely unwraps a .write() lock
macro_rules! take {
    ($x:expr) => {
        (*$x.write().unwrap())
    };
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

pub fn search_for_rust_files(dir: impl AsRef<Path>) -> Result<Vec<std::path::PathBuf>> {
    let pattern = dir.as_ref().join("**").join("*.rs");
    let files = glob(&pattern.to_string_lossy())
        .map_err(|e| anyhow::Error::new(e))?
        .map(|entry| entry.map_err(|e| anyhow::Error::new(e)))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(files)
}
