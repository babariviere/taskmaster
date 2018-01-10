//! Module to parse and get config

mod parser;

pub use self::parser::*;

use log::Level;
use signal::StopSignal;
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;

/// Condition to auto restart a program
#[derive(Debug)]
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
#[derive(Debug)]
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
#[derive(Debug)]
pub struct DaemonConfig {
    logfile: PathBuf,
    logfile_maxbytes: usize,
    logfile_backups: u16,
    loglevel: Level,
    pidfile: PathBuf,
    umask: u16,
    nodaemon: bool,
    minfds: i32,
    nocleanup: bool,
    child_log_dir: PathBuf,
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
#[derive(Debug)]
pub struct CtlConfig {
    server_ip: SocketAddr,
    prompt: String,
    history_file: Option<PathBuf>,
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
#[derive(Debug)]
pub struct ProcessConfig {
    name: String,
    command: String,
    proc_name: String,
    num_procs: u16,
    num_procs_start: u16,
    priority: u16,
    auto_start: bool,
    start_secs: u64,
    start_retries: u8,
    auto_restart: AutoRestartCondition,
    exit_codes: Vec<i32>,
    stop_signal: StopSignal,
    stop_wait_secs: u64,
    stop_as_group: Option<String>,
    kill_as_group: Option<String>,
    user: Option<String>,
    redirect_stderr: Option<PathBuf>,
    stdout_logfile: OutputLog,
    stdout_logfile_maxbytes: usize,
    stdout_logfile_backups: u32,
    stdout_capture_maxbytes: usize,
    stdout_events_enabled: bool,
    stderr_logfile: OutputLog,
    stderr_logfile_maxbytes: usize,
    stderr_logfile_backups: u32,
    stderr_capture_maxbytes: usize,
    stderr_events_enabled: bool,
    envs: Option<Vec<String>>,
    directory: Option<PathBuf>,
    umask: Option<u16>,
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
