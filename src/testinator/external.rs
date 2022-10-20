//!
//! calls to external binaries and the filesystem for ctime/mtime etc.
//!
//use anyhow::Result;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::SystemTime;

/// Helper struct to hold info about any and all files we access.
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub file: PathBuf,
    pub ctime: Option<SystemTime>, // not supported on linux ><
    pub now: SystemTime,
    pub mtime: SystemTime,
}

impl FileInfo {
    /// Initilise a `FileInfo` with it's ctime, mtime, and a timestamp for *now*.
    pub fn init(p: &PathBuf) -> Result<Self> {
        let metadata = fs::metadata(p)?;
        let ctime = {
            if let Ok(ctime) = metadata.created() {
                Some(ctime)
            } else {
                None
            }
        };

        Ok(Self {
            file: p.clone(),
            ctime,
            now: SystemTime::now(),
            mtime: metadata.modified()?,
        })
    }
}

// Need this because there's no trivial Default impl for `SystemTime`. (which makes sense..)
impl Default for FileInfo {
    fn default() -> Self {
        Self::init(&file!().into())
            .expect("Unexpected failure, this constructor should never be called.")
    }
}

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

/// Has a file been modified since we last looked at it?
pub fn should_process(f: &mut FileInfo) -> bool {
    if f.now > f.mtime {
        f.mtime = f.now;
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::file;

    #[test]
    fn ctime_of_this() {
        let fi = FileInfo::init(&file!().into()).unwrap();
        dbg!(fi);
    }
}
