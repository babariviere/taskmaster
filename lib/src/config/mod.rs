//! Module to parse and get config

mod ctl;
mod daemon;
mod parser;
mod process;
mod util;

pub use self::ctl::*;
pub use self::daemon::*;
pub use self::parser::*;
pub use self::process::*;
pub use self::util::*;

use log::Level;
use signal::StopSignal;
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;

/// Condition to auto restart a program
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum AutoRestartCondition {
    /// When the program exit and don't match exit codes
    Unexpected,
    /// Auto restart on exit
    True,
    /// Do not auto restart
    False,
}

impl Default for AutoRestartCondition {
    fn default() -> Self {
        AutoRestartCondition::Unexpected
    }
}

/// Logging output
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum OutputLog {
    /// No output
    None,
    /// Output to specified file
    File(PathBuf),
    /// Output to an automatic path
    Auto,
}

impl Default for OutputLog {
    fn default() -> Self {
        OutputLog::Auto
    }
}

/// Global config
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    daemon: Option<DaemonConfig>,
    ctl: Option<CtlConfig>,
    processes: Vec<ProcessConfig>,
}

impl Config {
    /// Get daemon config
    pub fn daemon(&self) -> Option<&DaemonConfig> {
        self.daemon.as_ref()
    }

    /// Get ctl config
    pub fn ctl(&self) -> Option<&CtlConfig> {
        self.ctl.as_ref()
    }

    /// Get processes config
    pub fn processes(&self) -> &Vec<ProcessConfig> {
        &self.processes
    }
}
