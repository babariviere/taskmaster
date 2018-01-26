use super::*;

fn default_logfile() -> PathBuf {
    env::current_dir()
        .expect("unable to get current dir")
        .join("taskmasterd.log")
}

fn default_logfile_maxbytes() -> usize {
    50000
}

fn default_logfile_backups() -> u16 {
    10
}

fn default_loglevel() -> Level {
    Level::Info
}

fn default_pidfile() -> PathBuf {
    env::current_dir()
        .expect("unable to get current dir")
        .join("taskmasterd.pid")
}

fn default_umask() -> u16 {
    0o022
}

fn default_minfds() -> i32 {
    1024
}

/// Configuration for taskmasterd
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DaemonConfig {
    /// Daemon log output
    #[serde(default = "default_logfile")]
    pub logfile: PathBuf,
    /// Max log output
    #[serde(default = "default_logfile_maxbytes")]
    #[serde(serialize_with = "serialize_human")]
    #[serde(deserialize_with = "deserialize_human")]
    pub logfile_maxbytes: usize,
    /// Log backups
    #[serde(default = "default_logfile_backups")]
    pub logfile_backups: u16,
    /// Log level
    #[serde(default = "default_loglevel")]
    pub loglevel: Level,
    /// Pid file path
    #[serde(default = "default_pidfile")]
    pub pidfile: PathBuf,
    /// Umask
    #[serde(default = "default_umask")]
    #[serde(serialize_with = "serialize_octal")]
    #[serde(deserialize_with = "deserialize_octal")]
    pub umask: u16,
    /// Disable daemon
    #[serde(default)]
    pub nodaemon: bool,
    /// Set min file descriptors
    #[serde(default = "default_minfds")]
    pub minfds: i32,
    /// Disable cleanup
    #[serde(default)]
    pub nocleanup: bool,
    /// Log dir for children
    #[serde(default)]
    // TODO: gen child log dir
    pub child_log_dir: PathBuf,
}

impl Default for DaemonConfig {
    fn default() -> DaemonConfig {
        let cwd = env::current_dir().expect("unable to get current dir");
        DaemonConfig {
            logfile: default_logfile(),
            logfile_maxbytes: default_logfile_maxbytes(),
            logfile_backups: default_logfile_backups(),
            loglevel: Level::Info,
            pidfile: default_pidfile(),
            umask: default_umask(),
            nodaemon: false,
            minfds: default_minfds(),
            nocleanup: false,
            child_log_dir: cwd.join("tmp.5321"),
        }
    }
}
