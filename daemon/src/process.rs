//! Process module

use libc;
use std::fs::{File, OpenOptions};
use std::os::unix::io::IntoRawFd;
use std::path::PathBuf;
use taskmaster::config::ProcessConfig;
use taskmaster::process::*;

/// Get process state
pub enum ProcessState {
    /// Starting process
    Starting,
    /// In running state, param is pid
    Running(i32),
    /// Fail start
    Backoff,
    /// Stopping process
    Stopping,
    /// Process manually stopped
    Stopped,
    /// Exited, param is exit code
    Exited(i32),
    /// Fail start a lot of time
    Fatal,
}

/// Process handler
pub struct Process {
    pid: i32,
    state: ProcessState,
    opt: ProcessOpt,
    config: ProcessConfig,
}

impl Process {
    /// Create a new process
    pub fn new(config: ProcessConfig) -> Process {
        Process {
            pid: -1,
            state: ProcessState::Stopped,
            opt: ProcessOpt::default(),
            config: config,
        }
    }

    fn handle_fail(&self) {
        self.state = ProcessState::Backoff;
    }

    fn setup_io(&self) {
        if let Some(ref stdin) = self.opt.stdin {
            let file = match stdin {
                &ChildStdio::Null => File::open("/dev/null"),
                &ChildStdio::File(ref f) => File::open(f),
            };
            if let Ok(file) = file {
                unsafe {
                    libc::dup2(file.into_raw_fd(), libc::STDIN_FILENO);
                }
            }
        }
        if let Some(ref stdout) = self.opt.stdout {
            let file = match stdout {
                &ChildStdio::Null => OpenOptions::new().write(true).open("/dev/null"),
                &ChildStdio::File(ref f) => File::create(f),
            };
            if let Ok(file) = file {
                unsafe {
                    libc::dup2(file.into_raw_fd(), libc::STDOUT_FILENO);
                }
            }
        }
        if let Some(ref stderr) = self.opt.stderr {
            let file = match stderr {
                &ChildStdio::Null => OpenOptions::new().write(true).open("/dev/null"),
                &ChildStdio::File(ref f) => File::create(f),
            };
            if let Ok(file) = file {
                unsafe {
                    libc::dup2(file.into_raw_fd(), libc::STDERR_FILENO);
                }
            }
        }
    }

    // use waitpid with WNOHANG to get status without waiting
    pub fn spawn(&mut self) {
        self.state = ProcessState::Starting;
        match fork() {
            Ok(ForkResult::Child) => {
                self.setup_io();
                unsafe {
                    if let Some(umask) = self.opt.umask {
                        libc::umask(umask);
                    }
                    if let Some(ref wd) = self.opt.working_dir {
                        libc::chdir(wd.to_str().unwrap().as_ptr() as *const i8);
                    }
                }
            }
            Ok(ForkResult::Parent(pid)) => {
                self.pid = pid;
            }
            Err(_) => {
                // TODO: respawn
            }
        }
    }
}

#[derive(Debug)]
pub enum ChildStdio {
    Null,
    File(PathBuf),
}

/// Process opts
#[derive(Default, Debug)]
pub struct ProcessOpt {
    args: Vec<String>,
    bin: String,
    envs: Vec<String>,
    working_dir: Option<PathBuf>,
    umask: Option<u16>,
    stdin: Option<ChildStdio>,
    stdout: Option<ChildStdio>,
    stderr: Option<ChildStdio>,
}

impl ProcessOpt {
    /// Create process opts
    pub fn new() -> ProcessOpt {
        Self::default()
    }

    /// Set args
    pub fn args(&mut self, args: Vec<String>) -> &mut ProcessOpt {
        self.args = args;
        self
    }

    /// Set bin
    pub fn bin(&mut self, bin: String) -> &mut ProcessOpt {
        self.bin = bin;
        self
    }

    /// Set envs
    pub fn envs(&mut self, envs: Vec<String>) -> &mut ProcessOpt {
        self.envs = envs;
        self
    }

    /// Set working dir
    pub fn working_dir(&mut self, wd: Option<PathBuf>) -> &mut ProcessOpt {
        self.working_dir = wd;
        self
    }

    /// Set umask
    pub fn umask(&mut self, umask: Option<u16>) -> &mut ProcessOpt {
        self.umask = umask;
        self
    }

    /// Set stdin
    pub fn stdin(&mut self, stdin: Option<ChildStdio>) -> &mut ProcessOpt {
        self.stdin = stdin;
        self
    }

    /// Set stdout
    pub fn stdout(&mut self, stdout: Option<ChildStdio>) -> &mut ProcessOpt {
        self.stdout = stdout;
        self
    }

    /// Set stderr
    pub fn stderr(&mut self, stderr: Option<ChildStdio>) -> &mut ProcessOpt {
        self.stderr = stderr;
        self
    }
}
