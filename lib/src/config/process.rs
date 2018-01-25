use super::*;

fn default_num_procs() -> u16 {
    1
}

fn default_priority() -> u16 {
    999
}

fn default_auto_start() -> bool {
    true
}

fn default_start_secs() -> u64 {
    1
}

fn default_start_retries() -> u8 {
    3
}

fn default_exit_codes() -> Vec<i32> {
    vec![0, 2]
}

fn default_stop_wait_secs() -> u64 {
    10
}

fn default_logfile_maxbytes() -> usize {
    50000
}

fn default_logfile_backups() -> u32 {
    10
}

/// Configuration for one process (program)
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProcessConfig {
    /// Name
    pub name: String,
    /// Command
    pub command: String,
    /// Num of procs
    #[serde(default = "default_num_procs")]
    pub num_procs: u16,
    /// Num procs to start
    #[serde(default)]
    pub num_procs_start: u16,
    /// Priority
    #[serde(default = "default_priority")]
    pub priority: u16,
    /// Auto start
    #[serde(default = "default_auto_start")]
    pub auto_start: bool,
    /// Start secs
    #[serde(default = "default_start_secs")]
    pub start_secs: u64,
    /// Start retries
    #[serde(default = "default_start_retries")]
    pub start_retries: u8,
    /// Auto restart condition
    #[serde(default)]
    pub auto_restart: AutoRestartCondition,
    /// Exit codes
    #[serde(default = "default_exit_codes")]
    pub exit_codes: Vec<i32>,
    /// Stop signal
    #[serde(default)]
    pub stop_signal: StopSignal,
    /// Stop wait secs
    #[serde(default = "default_stop_wait_secs")]
    pub stop_wait_secs: u64,
    /// Stop as group
    #[serde(default)]
    pub stop_as_group: Option<String>,
    /// Kill as group
    #[serde(default)]
    pub kill_as_group: Option<String>,
    /// User
    #[serde(default)]
    pub user: Option<String>,
    /// Redirect stderr
    #[serde(default)]
    pub redirect_stderr: Option<PathBuf>,
    /// Stdout logfile
    #[serde(default)]
    pub stdout_logfile: OutputLog,
    /// Stdout max bytes
    #[serde(default = "default_logfile_maxbytes")]
    pub stdout_logfile_maxbytes: usize,
    /// Stdout backups
    #[serde(default = "default_logfile_backups")]
    pub stdout_logfile_backups: u32,
    /// Stdout capture max bytes
    #[serde(default)]
    pub stdout_capture_maxbytes: usize,
    /// Stdout events enabled
    #[serde(default)]
    pub stdout_events_enabled: bool,
    /// Stderr logfile
    #[serde(default)]
    pub stderr_logfile: OutputLog,
    /// Stderr max bytes
    #[serde(default = "default_logfile_maxbytes")]
    pub stderr_logfile_maxbytes: usize,
    /// Stderr backups
    #[serde(default = "default_logfile_backups")]
    pub stderr_logfile_backups: u32,
    /// Stderr capture maxbytes
    #[serde(default)]
    pub stderr_capture_maxbytes: usize,
    /// Stderr events enabled
    #[serde(default)]
    pub stderr_events_enabled: bool,
    /// set envs
    #[serde(default)]
    pub envs: Option<Vec<String>>,
    /// set working directory
    #[serde(default)]
    pub directory: Option<PathBuf>,
    /// set umask
    #[serde(default)]
    pub umask: Option<u16>,
}

impl Default for ProcessConfig {
    fn default() -> Self {
        ProcessConfig {
            name: String::new(),
            command: String::new(),
            num_procs: default_num_procs(),
            num_procs_start: 0,
            priority: default_priority(),
            auto_start: default_auto_start(),
            start_secs: default_start_secs(),
            start_retries: default_start_retries(),
            auto_restart: AutoRestartCondition::default(),
            exit_codes: default_exit_codes(),
            stop_signal: StopSignal::default(),
            stop_wait_secs: default_stop_wait_secs(),
            stop_as_group: None,
            kill_as_group: None,
            user: None,
            redirect_stderr: None,
            stdout_logfile: OutputLog::default(),
            stdout_logfile_maxbytes: default_logfile_maxbytes(),
            stdout_logfile_backups: default_logfile_backups(),
            stdout_capture_maxbytes: 0,
            stdout_events_enabled: false,
            stderr_logfile: OutputLog::default(),
            stderr_logfile_maxbytes: default_logfile_maxbytes(),
            stderr_logfile_backups: default_logfile_backups(),
            stderr_capture_maxbytes: 0,
            stderr_events_enabled: false,
            envs: None,
            directory: None,
            umask: None,
        }
    }
}
