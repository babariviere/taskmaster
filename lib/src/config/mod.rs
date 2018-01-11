//! Module to parse and get config

mod parser;

pub use self::parser::*;

use log::Level;
use signal::StopSignal;
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;

/// Condition to auto restart a program
#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
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

/// Configuration for taskmasterd
#[derive(Clone, Debug)]
pub struct DaemonConfig {
    /// Daemon log output
    pub logfile: PathBuf,
    /// Max log output
    pub logfile_maxbytes: usize,
    /// Log backups
    pub logfile_backups: u16,
    /// Log level
    pub loglevel: Level,
    /// Pid file path
    pub pidfile: PathBuf,
    /// Umask
    pub umask: u16,
    /// Disable daemon
    pub nodaemon: bool,
    /// Set min file descriptors
    pub minfds: i32,
    /// Disable cleanup
    pub nocleanup: bool,
    /// Log dir for children
    pub child_log_dir: PathBuf,
}

impl Default for DaemonConfig {
    fn default() -> DaemonConfig {
        let cwd = env::current_dir().expect("unable to get current dir");
        DaemonConfig {
            logfile: cwd.join("taskmasterd.log"),
            logfile_maxbytes: 50000,
            logfile_backups: 10,
            loglevel: Level::Info,
            pidfile: cwd.join("taskmasterd.pid"),
            umask: 0o022,
            nodaemon: false,
            minfds: 1024,
            nocleanup: false,
            child_log_dir: cwd.join("tmp.5321"),
        }
    }
}

/// Configuration for taskmasterctl
#[derive(Clone, Debug)]
pub struct CtlConfig {
    /// Server ip
    pub server_ip: SocketAddr,
    /// Prompt
    pub prompt: String,
    /// Path to file history
    pub history_file: Option<PathBuf>,
}

impl Default for CtlConfig {
    fn default() -> Self {
        CtlConfig {
            server_ip: (
                IpAddr::from(Ipv4Addr::new(127, 0, 0, 1)),
                super::DEFAULT_PORT,
            ).into(),
            prompt: "taskmaster> ".into(),
            history_file: None,
        }
    }
}

/// Configuration for one process (program)
#[derive(Clone, Debug)]
pub struct ProcessConfig {
    /// Name
    pub name: String,
    /// Command
    pub command: String,
    /// Process name
    pub proc_name: String,
    /// Num of procs
    pub num_procs: u16,
    /// Num procs to start
    pub num_procs_start: u16,
    /// Priority
    pub priority: u16,
    /// Auto start
    pub auto_start: bool,
    /// Start secs
    pub start_secs: u64,
    /// Start retries
    pub start_retries: u8,
    /// Auto restart condition
    pub auto_restart: AutoRestartCondition,
    /// Exit codes
    pub exit_codes: Vec<i32>,
    /// Stop signal
    pub stop_signal: StopSignal,
    /// Stop wait secs
    pub stop_wait_secs: u64,
    /// Stop as group
    pub stop_as_group: Option<String>,
    /// Kill as group
    pub kill_as_group: Option<String>,
    /// User
    pub user: Option<String>,
    /// Redirect stderr
    pub redirect_stderr: Option<PathBuf>,
    /// Stdout logfile
    pub stdout_logfile: OutputLog,
    /// Stdout max bytes
    pub stdout_logfile_maxbytes: usize,
    /// Stdout backups
    pub stdout_logfile_backups: u32,
    /// Stdout capture max bytes
    pub stdout_capture_maxbytes: usize,
    /// Stdout events enabled
    pub stdout_events_enabled: bool,
    /// Stderr logfile
    pub stderr_logfile: OutputLog,
    /// Stderr max bytes
    pub stderr_logfile_maxbytes: usize,
    /// Stderr backups
    pub stderr_logfile_backups: u32,
    /// Stderr capture maxbytes
    pub stderr_capture_maxbytes: usize,
    /// Stderr events enabled
    pub stderr_events_enabled: bool,
    /// set envs
    pub envs: Option<Vec<String>>,
    /// set working directory
    pub directory: Option<PathBuf>,
    /// set umask
    pub umask: Option<u16>,
}

impl Default for ProcessConfig {
    fn default() -> Self {
        ProcessConfig {
            name: String::new(),
            command: String::new(),
            proc_name: String::new(),
            num_procs: 1,
            num_procs_start: 0,
            priority: 999,
            auto_start: true,
            start_secs: 1,
            start_retries: 3,
            auto_restart: AutoRestartCondition::default(),
            exit_codes: vec![0, 2],
            stop_signal: StopSignal::default(),
            stop_wait_secs: 10,
            stop_as_group: None,
            kill_as_group: None,
            user: None,
            redirect_stderr: None,
            stdout_logfile: OutputLog::default(),
            stdout_logfile_maxbytes: 50000,
            stdout_logfile_backups: 10,
            stdout_capture_maxbytes: 0,
            stdout_events_enabled: false,
            stderr_logfile: OutputLog::default(),
            stderr_logfile_maxbytes: 50000,
            stderr_logfile_backups: 10,
            stderr_capture_maxbytes: 0,
            stderr_events_enabled: false,
            envs: None,
            directory: None,
            umask: None,
        }
    }
}

/// Global config
#[derive(Debug, Default)]
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
