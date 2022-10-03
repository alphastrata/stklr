use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    pub static ref RUST_DOCSTRING: Regex = Regex::new(r#"(?P<ident>(///))"#).unwrap();
    pub static ref RUST_ENUM: Regex = Regex::new(r"(enum\s{1}(?P<ident>\w*))").unwrap();
    pub static ref RUST_FN: Regex = Regex::new(r"(fn\s{1}(?P<ident>\w*))").unwrap();
    pub static ref RUST_STRUCT: Regex = Regex::new(r"(struct\s{1}(?P<ident>\w*))").unwrap();
    pub static ref RUST_TRAIT: Regex = Regex::new(r"(trait\s{1}(?P<ident>\w*))").unwrap();
    pub static ref RUST_TY: Regex = Regex::new(r"(type\s{1}(?P<ident>\w*))").unwrap();
}
    /// Creates a new [`CodeBase`] from the glob searching the current working directory the app is run
    /// makes adjustments to RawLines from within [`RawSourceCode`]'s RawLines
    /// Process preview_changes [`RawSourceCode`] [`find_docs`]
