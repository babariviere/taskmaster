//! Module to enable logging.

use std::fmt;

/// Default logger
static mut LOGGER: &'static Logger = &NopLogger;

/// Set level of logging.
#[repr(u8)]
#[derive(Debug)]
pub enum Level {
    /// Message that requires immediate user attentions
    Critical = 1,
    /// Potential ignorable error
    Error,
    /// Anomalous condition
    Warn,
    /// Information output
    Info,
    /// Information for developper
    Debug,
    /// Deep information for developper
    Trace,
    /// Really deep information for developper
    Blather,
}

/// Set level filter for logging.
#[repr(u8)]
#[derive(Debug)]
pub enum LevelFilter {
    /// Disabled
    Off,
    /// Only critical
    Critical,
    /// Error and above
    Error,
    /// Warn and above
    Warn,
    /// Info and above
    Info,
    /// Debug and above
    Debug,
    /// Trace and above
    Trace,
    /// Blather and above
    Blather,
}

impl Default for LevelFilter {
    fn default() -> LevelFilter {
        LevelFilter::Info
    }
}

/// Metadata for logging.
#[derive(Default)]
pub struct Metadata<'a> {
    filter: LevelFilter,
    app: Option<&'a str>,
}

impl<'a> Metadata<'a> {
    /// Get filter
    pub fn filter(&self) -> &LevelFilter {
        &self.filter
    }

    /// Get app name
    pub fn app(&self) -> Option<&'a str> {
        self.app
    }
}

/// Builder for Metadata struct
///
/// Example:
/// ```
/// use taskmaster::log::*;
///
/// let meta = MetadataBuilder::new()
///                 .filter(LevelFilter::Blather)
///                 .app("testing app")
///                 .build();
/// ```
pub struct MetadataBuilder<'a> {
    metadata: Metadata<'a>,
}

impl<'a> MetadataBuilder<'a> {
    /// Create a builder
    pub fn new() -> MetadataBuilder<'a> {
        MetadataBuilder {
            metadata: Metadata::default(),
        }
    }

    /// Set filter
    pub fn filter(mut self, filter: LevelFilter) -> MetadataBuilder<'a> {
        self.metadata.filter = filter;
        self
    }

    /// Set app name
    pub fn app(mut self, app: &'a str) -> MetadataBuilder<'a> {
        self.metadata.app = Some(app);
        self
    }

    /// Build it
    pub fn build(self) -> Metadata<'a> {
        self.metadata
    }
}

/// All data needed to log
pub struct Log<'a> {
    meta: Metadata<'a>,
    args: fmt::Arguments<'a>,
    module: Option<&'a str>,
    file: Option<&'a str>,
    line: Option<usize>,
}

impl<'a> Log<'a> {}

/// Trait to determine if a type can log or not.
pub trait Logger {
    /// Check if logger is enabled
    fn enabled(&self, metadata: &Metadata) -> bool;
}

struct NopLogger;

impl Logger for NopLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        false
    }
}
