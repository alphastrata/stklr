//!
//! calls to external binaries and the filesystem for ctime/mtime etc.
//!
use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use std::time::SystemTime;

/// Helper struct to hold info about any and all files we access.
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub file: PathBuf,
    pub ctime: Option<SystemTime>, // not supported on linux ><
    pub now: SystemTime,
    pub mtime: SystemTime,
    pub last_checked: SystemTime,
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
            last_checked: SystemTime::now(),
        })
    }

    pub fn refresh(self) -> Self {
        //FIXME: will fail should a user rename/delete a file, so ...
        Self::init(&self.file).unwrap()
    }
    /// Has a file been modified since we last looked at it?
    pub fn should_process(&self) -> bool {
        if let Ok(t) = self.mtime.duration_since(self.last_checked) {
            if t > Duration::from_millis(10) {
                eprintln!("SHOULD PROCESS=true {}", t.as_secs_f64());
                return true;
            }
        }
        false
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
        .expect("failed to execute process");

    out.success()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::file;

    #[test]
    #[ignore]
    fn file_info() {
        let fi = FileInfo::init(&file!().into()).unwrap();
        dbg!(fi);
    }
}
