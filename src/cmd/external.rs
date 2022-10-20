//use anyhow::Result;
use std::process::Command;

/// Runs `cargo test $s`, `bool` on the return is false for failures.
pub fn test(s: &str) -> bool {
    let out = Command::new("cargo")
        .arg("test")
        .arg("-q")
        .arg(s)
        .status()
        //.output()
        .expect("failed to execute process");

    out.success()
}
