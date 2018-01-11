//! Crate used by the client and daemon, avoid to rewrite code in both

#![deny(missing_docs)]
#![feature(libc)]

extern crate libc as _libc;

#[macro_use]
pub mod log;
pub mod config;
pub mod ffi;
pub mod parser;
pub mod process;
pub mod signal;

/// Libc
pub mod libc {
    pub use _libc::*;
}

/// Taskmaster's default port
pub const DEFAULT_PORT: u16 = 9450;
