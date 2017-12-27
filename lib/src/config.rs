use log::LogLevel;
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;

#[derive(Debug)]
pub enum AutoRestartCondition {
    Unexpected,
    True,
    False,
}

impl Default for AutoRestartCondition {
    fn default() -> Self {
        AutoRestartCondition::Unexpected
    }
}

#[derive(Debug)]
pub enum StopSignal {
    Term,
    Hup,
    Int,
    Quit,
    Kill,
    Usr1,
    Usr2,
}

impl Default for StopSignal {
    fn default() -> Self {
        StopSignal::Term
    }
}

#[derive(Debug)]
pub enum OutputLog {
    None,
    File(PathBuf),
    Auto,
}

impl Default for OutputLog {
    fn default() -> Self {
        OutputLog::Auto
    }
}

#[derive(Debug)]
pub struct DaemonConfig {
    logfile: PathBuf,
    logfile_maxbytes: usize,
    logfile_backups: u16,
    loglevel: LogLevel,
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
            loglevel: LogLevel::Info,
            pidfile: cwd.join("taskmasterd.pid"),
            umask: 0o022,
            nodaemon: false,
            minfds: 1024,
            nocleanup: false,
            child_log_dir: cwd.join("tmp.5321"),
        }
    }
}

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

#[derive(Debug)]
pub struct ProcessConfig {
    name: String,
    command: String,
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
