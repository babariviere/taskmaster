use super::*;

use std::io::{self, Write};
use std::path::Path;

/// TODO: format

/// Output kind
enum OutputKind {
    Stdout(io::Stdout),
    Stderr(io::Stderr),
    File(File),
}

/// Output
pub struct Output {
    kind: OutputKind,
    lvl: LevelFilter,
}

impl Output {
    /// Create output from stdout
    pub fn stdout(lvl: LevelFilter) -> Output {
        Output {
            kind: OutputKind::Stdout(io::stdout()),
            lvl: lvl,
        }
    }

    /// Create output from stderr
    pub fn stderr(lvl: LevelFilter) -> Output {
        Output {
            kind: OutputKind::Stderr(io::stderr()),
            lvl: lvl,
        }
    }

    /// Create output from file
    pub fn file<P: AsRef<Path>>(path: P, lvl: LevelFilter) -> Result<Output, io::Error> {
        Ok(Output {
            kind: OutputKind::File(File::create(path.as_ref())?),
            lvl: lvl,
        })
    }

    /// Log to output
    pub fn log(&mut self, log: &Log) {
        if log.level() as u8 > self.lvl as u8 {
            return;
        }
        match self.kind {
            OutputKind::Stdout(ref mut s) => {
                let _ = s.write(log.message().as_bytes());
            }
            OutputKind::Stderr(ref mut s) => {
                let _ = s.write(log.message().as_bytes());
            }
            OutputKind::File(ref mut s) => {
                let _ = s.write(log.message().as_bytes());
            }
        }
    }
}
