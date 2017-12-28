//! Crate used by the client and daemon, avoid to rewrite code in both

#![deny(missing_docs)]

pub mod config;
pub mod ffi;
pub mod log;
pub mod process;

/// Taskmaster's default port
pub const DEFAULT_PORT: u16 = 7089;
