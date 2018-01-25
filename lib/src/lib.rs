//! Crate used by the client and daemon, avoid to rewrite code in both

#![deny(missing_docs)]

extern crate failure;
extern crate nix;
extern crate serde;
#[macro_use]
extern crate serde_derive;

#[macro_use]
pub mod log;
pub mod api;
pub mod config;
pub mod ffi;
pub mod parser;
pub mod signal;

/// Taskmaster's default port
pub const DEFAULT_PORT: u16 = 9450;
