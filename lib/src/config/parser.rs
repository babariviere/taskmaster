//! Module to parse configuration

use log::Level;
use parser::Parser;
use std::net::ToSocketAddrs;
use std::str::FromStr;
use super::*;

/// Ini value
#[derive(Clone, Debug, PartialEq)]
pub enum IniValue {
    /// Ini key
    Key(String, String),
    /// Ini section
    Section(String, Vec<IniValue>),
}

/// Ini parser
pub struct IniParser<'a> {
    parser: Parser<'a>,
}

impl<'a> IniParser<'a> {
    /// Create parser
    pub fn new(buf: &'a str) -> IniParser {
        IniParser {
            parser: Parser::new(buf),
        }
    }

    /// Parse everything
    pub fn parse(mut self) -> Vec<IniValue> {
        let mut values = Vec::new();
        while let Some(val) = self.parse_value() {
            values.push(val);
        }
        values
    }

    /// Parse next value
    pub fn parse_value(&mut self) -> Option<IniValue> {
        while let Some(c) = self.parser.next_char() {
            if !c.is_whitespace() && c != ';' {
                break;
            }
            self.parser.eat_line();
        }
        match self.parser.next_char() {
            Some('[') => Some(self.parse_section()),
            Some(_) => Some(self.parse_key()),
            _ => None,
        }
    }

    /// Parse key
    pub fn parse_key(&mut self) -> IniValue {
        let name = self.parser.eat_while(|c| c != '=').trim().to_string();
        self.parser.eat_char();
        let value = self.parser
            .eat_while_esc(|c| c != ';' && c != '\n', '\\')
            .trim()
            .to_string();
        IniValue::Key(name, value)
    }

    /// Parse section
    pub fn parse_section(&mut self) -> IniValue {
        assert_eq!(self.parser.eat_char(), Some('['));
        let sec = self.parser.eat_while(|c| c != ']');
        assert_eq!(self.parser.eat_char(), Some(']'));
        self.parser.eat_line();
        let mut keys = Vec::new();
        loop {
            while let Some(c) = self.parser.next_char() {
                if !c.is_whitespace() && c != ';' {
                    break;
                }
                self.parser.eat_line();
            }
            match self.parser.next_char() {
                Some('[') | None => break,
                _ => {}
            }
            keys.push(self.parse_key());
        }
        IniValue::Section(sec, keys)
    }
}

macro_rules! nbr {
    ($dst:expr, $dst_name:expr, $src:expr, $section:expr) => {
        $dst = $src.parse().unwrap_or_else(|e| {
            warn!("config: invalid field `{}` in section [{}]", $dst_name, $section);
            trace!("config error: {}", e);
            $dst
        });
    };
}

macro_rules! boolean {
    ($dst:expr, $dst_name:expr, $src:expr, $section:expr) => {
        match $src.as_str() {
            "true" => $dst = true,
            "false" => $dst = false,
            v => {
                warn!("config: invalid field `{}` in section [{}]", $dst_name, $section);
                trace!("user have put value `{}`", v);
            }
        }
    }
}

/// Config parser
pub struct ConfigParser {
    config: Config,
    values: Vec<IniValue>,
}

impl ConfigParser {
    /// Create config parser
    pub fn new<S: AsRef<str>>(buf: S) -> ConfigParser {
        ConfigParser {
            config: Config::default(),
            values: IniParser::new(buf.as_ref()).parse(),
        }
    }

    /// Parse config
    pub fn parse(mut self) -> Config {
        while let Some(mut ini) = self.values.pop() {
            match ini {
                IniValue::Section(ref s, ref mut v) if s == "taskmasterd" => {
                    self.config.daemon = Some(self.parse_daemon(v))
                }
                IniValue::Section(ref s, ref mut v) if s == "taskmasterctl" => {
                    self.config.ctl = Some(self.parse_ctl(v))
                }
                IniValue::Section(ref s, ref mut v)
                    if s.starts_with("program") || s.starts_with("process") =>
                {
                    if let Some(idx) = s.find(':') {
                        let process = self.parse_process(s[idx + 1..].to_string(), v);
                        self.config.processes.push(process);
                    } else {
                        warn!("missing program name in config, program will be ignored");
                    }
                }
                val => {
                    warn!("unexpected value in configuration {:?}", val);
                }
            }
        }
        self.config
    }

    /// Parse daemon configuration
    pub fn parse_daemon(&mut self, values: &mut Vec<IniValue>) -> DaemonConfig {
        let mut config = DaemonConfig::default();
        while let Some(value) = values.pop() {
            match value {
                IniValue::Key(k, v) => match k.as_str() {
                    "logfile" => config.logfile = PathBuf::from(v),
                    "logfile_maxbytes" => nbr!(config.logfile_maxbytes, k, v, "taskmasterd"),
                    "logfile_backups" => nbr!(config.logfile_backups, k, v, "taskmasterd"),
                    "loglevel" => {
                        config.loglevel = Level::from_str(&v).unwrap_or_else(|_| {
                            warn!("config: invalid field `loglevel` in section [taskmasterd]");
                            config.loglevel
                        })
                    }
                    "pidfile" => config.pidfile = PathBuf::from(v),
                    // TODO: parse octal
                    "umask" => nbr!(config.umask, k, v, "taskmasterd"),
                    "nodaemon" => boolean!(config.nodaemon, k, v, "taskmasterd"),
                    "minfds" => nbr!(config.minfds, k, v, "taskmasterd"),
                    "nocleanup" => boolean!(config.nocleanup, k, v, "taskmasterd"),
                    "child_log_dir" => config.child_log_dir = PathBuf::from(v),
                    k => warn!("config: unknown field `{}` in section [taskmasterd]", k),
                },
                IniValue::Section(_, _) => unreachable!(),
            }
        }
        config
    }

    /// Parse ctl configuration
    pub fn parse_ctl(&mut self, values: &mut Vec<IniValue>) -> CtlConfig {
        let mut config = CtlConfig::default();
        while let Some(value) = values.pop() {
            match value {
                IniValue::Key(k, v) => match k.as_str() {
                    "server_ip" => match v.to_socket_addrs() {
                        Ok(mut ip) => config.server_ip = ip.next().unwrap(),
                        Err(_) => {
                            warn!("config: invalid field `server_ip` in section [taskmasterctl]");
                            trace!("user have put value `{}`", v);
                        }
                    },
                    "prompt" => config.prompt = v,
                    "history_file" => match v.as_str() {
                        "none" => config.history_file = None,
                        s => config.history_file = Some(PathBuf::from(s)),
                    },
                    k => warn!("config: unknown field `{}` in section [taskmasterctl]", k),
                },
                IniValue::Section(_, _) => unreachable!(),
            }
        }
        config
    }

    /// Parse process configuration
    pub fn parse_process(&mut self, name: String, values: &mut Vec<IniValue>) -> ProcessConfig {
        let mut config = ProcessConfig::default();
        let section_name = format!("program:{}", name);
        config.name = name;
        while let Some(value) = values.pop() {
            match value {
                IniValue::Key(k, v) => match k.as_str() {
                    "command" => config.command = v,
                    "num_procs" => nbr!(config.num_procs, k, v, section_name),
                    "num_procs_start" => nbr!(config.num_procs_start, k, v, section_name),
                    "priority" => nbr!(config.priority, k, v, section_name),
                    "auto_start" => boolean!(config.auto_start, k, v, section_name),
                    "start_secs" => nbr!(config.start_secs, k, v, section_name),
                    "start_retries" => nbr!(config.start_retries, k, v, section_name),
                    "auto_restart" => match v.as_str() {
                        "unexpected" => config.auto_restart = AutoRestartCondition::Unexpected,
                        "true" => config.auto_restart = AutoRestartCondition::True,
                        "false" => config.auto_restart = AutoRestartCondition::False,
                        v => {
                            warn!(
                                "config: invalid field `{}` in section [{}]",
                                k, section_name
                            );
                            trace!("user have put value `{}`", v);
                        }
                    },
                    "exit_codes" => {
                        let codes = v.split(' ')
                            .into_iter()
                            .filter_map(|n| n.parse().ok())
                            .collect::<Vec<i32>>();
                        if codes.is_empty() {
                            warn!(
                                "config: invalid field `exit_codes` in section [{}]",
                                section_name
                            );
                        } else {
                            config.exit_codes = codes;
                        }
                    }
                    "stop_signal" => match v.to_lowercase().as_str() {
                        "sigterm" | "term" => config.stop_signal = StopSignal::Term,
                        "sighup" | "hup" => config.stop_signal = StopSignal::Hup,
                        "sigint" | "int" => config.stop_signal = StopSignal::Int,
                        "sigquit" | "quit" => config.stop_signal = StopSignal::Quit,
                        "sigkill" | "kill" => config.stop_signal = StopSignal::Kill,
                        "sigusr1" | "usr1" => config.stop_signal = StopSignal::Usr1,
                        "sigusr2" | "usr2" => config.stop_signal = StopSignal::Usr2,
                        v => {
                            warn!(
                                "config: invalid field `{}` in section [{}]",
                                k, section_name
                            );
                            trace!("user have put value `{}`", v);
                        }
                    },
                    "stop_wait_secs" => nbr!(config.stop_wait_secs, k, v, section_name),
                    "stop_as_group" => match v.as_str() {
                        "none" => config.stop_as_group = None,
                        _ => config.stop_as_group = Some(v),
                    },
                    "kill_as_group" => match v.as_str() {
                        "none" => config.kill_as_group = None,
                        _ => config.kill_as_group = Some(v),
                    },
                    "user" => match v.as_str() {
                        "none" => config.user = None,
                        _ => config.user = Some(v),
                    },
                    "redirect_stderr" => match v.as_str() {
                        "none" => config.redirect_stderr = None,
                        _ => config.redirect_stderr = Some(PathBuf::from(v)),
                    },
                    "stdout_logfile" => match v.as_str() {
                        "none" => config.stdout_logfile = OutputLog::None,
                        "auto" => config.stdout_logfile = OutputLog::Auto,
                        _ => config.stdout_logfile = OutputLog::File(PathBuf::from(v)),
                    },
                    "stdout_logfile_maxbytes" => {
                        nbr!(config.stdout_logfile_maxbytes, k, v, section_name)
                    }
                    "stdout_logfile_backups" => {
                        nbr!(config.stdout_logfile_backups, k, v, section_name)
                    }
                    "stdout_capture_maxbytes" => {
                        nbr!(config.stdout_capture_maxbytes, k, v, section_name)
                    }
                    "stdout_events_enabled" => {
                        boolean!(config.stdout_events_enabled, k, v, section_name)
                    }
                    "stderr_logfile" => match v.as_str() {
                        "none" => config.stderr_logfile = OutputLog::None,
                        "auto" => config.stderr_logfile = OutputLog::Auto,
                        _ => config.stdout_logfile = OutputLog::File(PathBuf::from(v)),
                    },
                    "stderr_logfile_maxbytes" => {
                        nbr!(config.stderr_logfile_maxbytes, k, v, section_name)
                    }
                    "stderr_logfile_backups" => {
                        nbr!(config.stderr_logfile_backups, k, v, section_name)
                    }
                    "stderr_capture_maxbytes" => {
                        nbr!(config.stderr_capture_maxbytes, k, v, section_name)
                    }
                    "stderr_events_enabled" => {
                        boolean!(config.stderr_events_enabled, k, v, section_name)
                    }
                    "envs" => match v.as_str() {
                        "none" => config.envs = None,
                        _ => {
                            config.envs = Some(
                                v.split(',')
                                    .filter(|s| !s.is_empty())
                                    .map(|s| s.to_string())
                                    .collect(),
                            )
                        }
                    },
                    "directory" => match v.as_str() {
                        "none" => config.directory = None,
                        _ => config.directory = Some(PathBuf::from(v)),
                    },
                    "umask" => match v.as_str() {
                        "none" => config.umask = None,
                        _ => match v.parse() {
                            Ok(n) => config.umask = Some(n),
                            Err(e) => {
                                warn!(
                                    "config: invalid field `{}` in section [{}]",
                                    k, section_name
                                );
                                trace!("error: {}", e);
                            }
                        },
                    },
                    k => warn!(
                        "config: unknown field `{}` in section [{}]",
                        k, section_name
                    ),
                },
                IniValue::Section(_, _) => unreachable!(),
            }
        }
        config
    }
}
