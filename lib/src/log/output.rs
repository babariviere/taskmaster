use super::*;

use std::io::{self, Write};
use std::path::Path;

/// TODO: format
pub type Formatter = Fn(&Log) -> String;

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
    format: Option<Box<Formatter>>,
}

impl Output {
    /// Create output from stdout
    pub fn stdout(lvl: LevelFilter, format: Option<Box<Formatter>>) -> Output {
        Output {
            kind: OutputKind::Stdout(io::stdout()),
            lvl: lvl,
            format: format,
        }
    }

    /// Create output from stderr
    pub fn stderr(lvl: LevelFilter, format: Option<Box<Formatter>>) -> Output {
        Output {
            kind: OutputKind::Stderr(io::stderr()),
            lvl: lvl,
            format: format,
        }
    }

    /// Create output from file
    pub fn file<P: AsRef<Path>>(
        path: P,
        lvl: LevelFilter,
        format: Option<Box<Formatter>>,
    ) -> Result<Output, io::Error> {
        Ok(Output {
            kind: OutputKind::File(File::create(path.as_ref())?),
            lvl: lvl,
            format: format,
        })
    }

    /// Log to output
    pub fn log(&mut self, log: &Log) {
        if log.level() as u8 > self.lvl as u8 {
            return;
        }
        match self.kind {
            OutputKind::Stdout(ref mut s) => {
                if let Some(ref f) = self.format {
                    let _ = writeln!(s, "{}", f(log));
                } else {
                    let _ = writeln!(s, "{}", log.message());
                }
            }
            OutputKind::Stderr(ref mut s) => {
                if let Some(ref f) = self.format {
                    let _ = writeln!(s, "{}", f(log));
                } else {
                    let _ = writeln!(s, "{}", log.message());
                }
            }
            OutputKind::File(ref mut s) => {
                if let Some(ref f) = self.format {
                    let _ = writeln!(s, "{}", f(log));
                } else {
                    let _ = writeln!(s, "{}", log.message());
                }
            }
        }
    }
}
