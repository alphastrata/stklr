use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    //TODO: can you be DRYer?
    pub static ref RUST_DOCSTRING: Regex = Regex::new(r#"(?P<ident>(///))"#).unwrap();
    pub static ref RUST_ENUM: Regex = Regex::new(r"(enum\s{1}(?P<ident>\w*))").unwrap();
    pub static ref RUST_FN: Regex = Regex::new(r"(fn\s{1}(?P<ident>\w*))").unwrap();
    pub static ref RUST_STRUCT: Regex = Regex::new(r"(struct\s{1}(?P<ident>\w*))").unwrap();
    pub static ref RUST_TRAIT: Regex = Regex::new(r"(trait\s{1}(?P<ident>\w*))").unwrap();
    pub static ref RUST_TY: Regex = Regex::new(r"(type\s{1}(?P<ident>\w*))").unwrap();
    pub static ref RUST_USE: Regex = Regex::new(r"(use\s{1}(?P<ident>\w*))").unwrap();
    pub static ref RUST_IMPORT: Regex = Regex::new(r"(::{1}(?P<ident>\w*;))").unwrap();

    // Stuff we never want linked.
    pub static ref NEVERS: Vec<&'static str> = vec!["log", "std", "core", "super","io","crate",
        "to", "path", "for","from", "into", "that", "we", "so","of", "new", "file", "from", "into", "self", "Self"];

    // Stuff we only want ` tik wrapped.
    pub static ref ALWAYS: Vec<&'static str> = vec!["std", "core", "io", "self", "Result", "()", "Self", "String", "str", "&str", "bool", "true", "false", "float", "f8", "f16", "f32", "f64", "f128", "i8", "u8", "i16", "u16", "i32", "u32", "i64", "u64", "i128", "u128", "isize", "usize", "path","from","into", "n"];
}
