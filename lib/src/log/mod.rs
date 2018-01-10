//! Module for logging

#[macro_use]
mod macros;
mod output;

pub use self::output::*;
use std::fmt;
use std::fs::File;
use std::str::FromStr;

/// Logger
static mut LOGGER: Logger = Logger {
    outputs: None,
    max_lvl: LevelFilter::Blather,
};

/// Log level
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Level {
    /// Requires user attention
    Critical = 1,
    /// Potential dangerous error
    Error,
    /// Something is abnormal
    Warn,
    /// Information
    Info,
    /// Debugging info
    Debug,
    /// Tracing code
    Trace,
    /// Useless talk
    Blather,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Level::Critical => write!(f, "critical"),
            &Level::Error => write!(f, "error"),
            &Level::Warn => write!(f, "warn"),
            &Level::Info => write!(f, "info"),
            &Level::Debug => write!(f, "debug"),
            &Level::Trace => write!(f, "trace"),
            &Level::Blather => write!(f, "blather"),
        }
    }
}

impl Default for Level {
    fn default() -> Level {
        Level::Info
    }
}

impl FromStr for Level {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "critical" => Ok(Level::Critical),
            "error" => Ok(Level::Error),
            "warn" => Ok(Level::Warn),
            "info" => Ok(Level::Info),
            "debug" => Ok(Level::Debug),
            "trace" => Ok(Level::Trace),
            "blather" => Ok(Level::Blather),
            _ => Err(()),
        }
    }
}

/// Log level filter
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum LevelFilter {
    /// No log
    Off,
    /// Critical
    Critical,
    /// Error
    Error,
    /// Warn
    Warn,
    /// Info
    Info,
    /// Debug
    Debug,
    /// Trace
    Trace,
    /// Blather
    Blather,
}

/// Logging metadata
pub struct Metadata<'a> {
    level: Level,
    module: &'a str,
    file: &'a str,
    line: u32,
}

impl<'a> Metadata<'a> {
    /// Get level
    #[inline]
    pub fn level(&self) -> Level {
        self.level
    }

    /// Get module
    #[inline]
    pub fn module(&self) -> &'a str {
        self.module
    }

    /// Get file
    #[inline]
    pub fn file(&self) -> &'a str {
        self.file
    }

    /// Get line
    #[inline]
    pub fn line(&self) -> u32 {
        self.line
    }
}

impl<'a> Default for Metadata<'a> {
    fn default() -> Metadata<'a> {
        Metadata {
            level: Level::default(),
            module: "",
            file: "",
            line: 0,
        }
    }
}

/// Metadata builder
pub struct MetadataBuilder<'a>(Metadata<'a>);

impl<'a> MetadataBuilder<'a> {
    /// Create builder
    #[inline]
    pub fn new() -> MetadataBuilder<'a> {
        MetadataBuilder(Metadata::default())
    }

    /// Set log level
    #[inline]
    pub fn level(mut self, lvl: Level) -> MetadataBuilder<'a> {
        self.0.level = lvl;
        self
    }

    /// Set module path
    #[inline]
    pub fn module(mut self, module: &'a str) -> MetadataBuilder<'a> {
        self.0.module = module;
        self
    }

    /// Set file name
    #[inline]
    pub fn file(mut self, file: &'a str) -> MetadataBuilder<'a> {
        self.0.file = file;
        self
    }

    /// Set line
    #[inline]
    pub fn line(mut self, line: u32) -> MetadataBuilder<'a> {
        self.0.line = line;
        self
    }

    /// Build metadata
    #[inline]
    pub fn build(self) -> Metadata<'a> {
        self.0
    }
}

/// A log message
pub struct Log<'a> {
    msg: fmt::Arguments<'a>,
    meta: Metadata<'a>,
}

impl<'a> Log<'a> {
    /// Get log message
    #[inline]
    pub fn message(&self) -> &fmt::Arguments<'a> {
        &self.msg
    }

    /// Get log meta
    #[inline]
    pub fn meta(&self) -> &Metadata<'a> {
        &self.meta
    }

    /// Get log level
    #[inline]
    pub fn level(&self) -> Level {
        self.meta.level()
    }

    /// Get log module path
    #[inline]
    pub fn module(&self) -> &'a str {
        self.meta.module()
    }

    /// Get log file name
    #[inline]
    pub fn file(&self) -> &'a str {
        self.meta.file()
    }

    /// Get log line
    #[inline]
    pub fn line(&self) -> u32 {
        self.meta.line()
    }
}

impl<'a> Default for Log<'a> {
    fn default() -> Log<'a> {
        Log {
            msg: format_args!(""),
            meta: Metadata::default(),
        }
    }
}

/// Log builder
pub struct LogBuilder<'a>(Log<'a>);

impl<'a> LogBuilder<'a> {
    /// Create builder
    pub fn new() -> LogBuilder<'a> {
        LogBuilder(Log::default())
    }

    /// Set message
    #[inline]
    pub fn message(mut self, msg: fmt::Arguments<'a>) -> LogBuilder<'a> {
        self.0.msg = msg;
        self
    }

    /// Set meta
    #[inline]
    pub fn meta(mut self, meta: Metadata<'a>) -> LogBuilder<'a> {
        self.0.meta = meta;
        self
    }

    /// Set log level
    #[inline]
    pub fn level(mut self, lvl: Level) -> LogBuilder<'a> {
        self.0.meta.level = lvl;
        self
    }

    /// Set log module
    #[inline]
    pub fn module(mut self, module: &'a str) -> LogBuilder<'a> {
        self.0.meta.module = module;
        self
    }

    /// Set log file
    #[inline]
    pub fn file(mut self, file: &'a str) -> LogBuilder<'a> {
        self.0.meta.file = file;
        self
    }

    /// Set log line
    #[inline]
    pub fn line(mut self, line: u32) -> LogBuilder<'a> {
        self.0.meta.line = line;
        self
    }

    /// Build log
    #[inline]
    pub fn build(self) -> Log<'a> {
        self.0
    }
}

/// Allow logging
pub struct Logger {
    outputs: Option<Vec<Output>>,
    max_lvl: LevelFilter,
}

impl Logger {
    /// Create a new logger
    pub fn new() -> Logger {
        Logger {
            outputs: None,
            max_lvl: LevelFilter::Blather,
        }
    }

    /// Add an output
    pub fn add_output(&mut self, out: Output) {
        match self.outputs {
            Some(ref mut outs) => outs.push(out),
            None => {
                let mut outputs = Vec::new();
                outputs.push(out);
                self.outputs = Some(outputs);
            }
        }
    }

    /// Remove all outputs
    pub fn clear_outputs(&mut self) {
        if let Some(ref mut outs) = self.outputs {
            outs.clear();
        }
    }

    /// Set max level
    pub fn set_max_level(&mut self, lvl: LevelFilter) {
        self.max_lvl = lvl;
    }

    /// Do logging
    pub fn log(&mut self, log: Log) {
        let ref mut outputs = match self.outputs {
            Some(ref mut out) => out,
            None => return,
        };
        if log.level() as u8 > self.max_lvl as u8 {
            return;
        }
        for output in outputs.iter_mut() {
            output.log(&log);
        }
    }
}

/// Get mutable ref to global logger
pub fn logger() -> &'static mut Logger {
    unsafe { &mut LOGGER }
}

/// Initialize logger
pub fn init_logger<F>(init: F)
where
    F: FnOnce(&'static mut Logger),
{
    let logger = logger();
    logger.clear_outputs();
    init(logger);
}
