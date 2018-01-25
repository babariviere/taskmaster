use super::*;

fn default_addr() -> SocketAddr {
    (IpAddr::from(Ipv4Addr::new(127, 0, 0, 1)), ::DEFAULT_PORT).into()
}

fn default_prompt() -> String {
    "taskmaster> ".to_owned()
}

/// Configuration for taskmasterctl
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CtlConfig {
    /// Server ip
    #[serde(default = "default_addr")]
    pub server_ip: SocketAddr,
    /// Prompt
    #[serde(default = "default_prompt")]
    pub prompt: String,
    /// Path to file history
    #[serde(default)]
    pub history_file: Option<PathBuf>,
}

impl Default for CtlConfig {
    fn default() -> Self {
        CtlConfig {
            server_ip: default_addr(),
            prompt: default_prompt(),
            history_file: None,
        }
    }
}
