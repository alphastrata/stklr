use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    pub static ref RUST_DOCSTRING: Regex = Regex::new(r#"(?P<ident>(///))"#).unwrap();
    pub static ref RUST_ENUM: Regex = Regex::new(r"(enum\s{1}(?P<ident>\w*))").unwrap();
    pub static ref RUST_FN: Regex = Regex::new(r"(fn\s{1}(?P<ident>\w*))").unwrap();
    pub static ref RUST_STRUCT: Regex = Regex::new(r"(struct\s{1}(?P<ident>\w*))").unwrap();
    pub static ref RUST_TRAIT: Regex = Regex::new(r"(trait\s{1}(?P<ident>\w*))").unwrap();
    pub static ref RUST_TY: Regex = Regex::new(r"(type\s{1}(?P<ident>\w*))").unwrap();
    pub static ref RUST_USE: Regex = Regex::new(r"(use\s{1}(?P<ident>\w*))").unwrap();
    pub static ref RUST_IMPORT: Regex = Regex::new(r"(::{1}(?P<ident>\w*))").unwrap();
}

/// A store of keywords we NEVER want to match on.
pub const NEVERS: Vec<&str> = vec!["to", "path", "file", "from", "into", "self", "Self"];

// rust std types, we only want to syntax highlight these with ``s.
pub const ALWAYS: Vec<&str> = vec![
    "i8", "u8", "i16", "u16", "i32", "u32", "i64", "u64", "i128", "u128", "isize", "usize",
];
