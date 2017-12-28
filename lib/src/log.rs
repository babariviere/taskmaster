//! Module for logging

/// Log level
#[derive(Debug)]
pub enum Level {
    /// Requires user attention
    Critical,
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
