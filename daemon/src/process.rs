//! Process module

use command::Command;
use nix::fcntl::*;
use nix::sys::{stat, wait};
use nix::unistd::*;
use std::os::unix::io::*;
use std::sync::{RwLock, RwLockReadGuard};
use taskmaster::config::*;

/// Get process state
#[derive(Clone, Debug, PartialEq)]
pub enum ProcessState {
    /// Starting process
    Starting,
    /// In running state, param is pid
    Running(ProcessHolder),
    /// Fail start
    Backoff,
    /// Stopping process
    Stopping,
    /// Process manually stopped
    Stopped,
    /// Exited, param is exit code
    Exited(i8),
    /// Fail start a lot of time
    Fatal,
}

// TODO: process holder with pid, stdin, stdout, stderr
// TODO: use signal to handle process
#[derive(Clone, Debug, PartialEq)]
pub struct ProcessHolder {
    pid: Pid,
    stdin: Option<RawFd>,
    stdout: Option<RawFd>,
    stderr: Option<RawFd>,
}

impl ProcessHolder {
    /// create a new process holder
    pub fn new(pid: Pid) -> ProcessHolder {
        ProcessHolder {
            pid: pid,
            stdin: None,
            stdout: None,
            stderr: None,
        }
    }

    /// set stdin
    pub fn stdin(mut self, stdin: RawFd) -> ProcessHolder {
        self.stdin = Some(stdin);
        self
    }

    /// Set stdout
    pub fn stdout(mut self, stdout: RawFd) -> ProcessHolder {
        self.stdout = Some(stdout);
        self
    }

    /// Set stderr
    pub fn stderr(mut self, stderr: RawFd) -> ProcessHolder {
        self.stderr = Some(stderr);
        self
    }

    /// Get stdin
    pub fn get_stdin(&self) -> Option<RawFd> {
        self.stdin
    }

    /// Get stdout
    pub fn get_stdout(&self) -> Option<RawFd> {
        self.stdout
    }

    pub fn get_stderr(&self) -> Option<RawFd> {
        self.stderr
    }
}

/// Process handler
pub struct Process {
    command: Command,
    state: RwLock<ProcessState>,
    config: ProcessConfig,
    count_fail: u8,
}

impl Process {
    /// Create a new process
    pub fn new(config: ProcessConfig) -> Process {
        Process {
            command: Command::new(&config.command, config.envs.clone().unwrap_or(Vec::new())),
            state: RwLock::new(ProcessState::Stopped),
            config: config,
            count_fail: 0,
        }
    }

    /// Get proc name
    pub fn proc_name(&self) -> &str {
        &self.config.proc_name
    }

    fn handle_fail(&mut self) {
        self.count_fail += 1;
        let mut state_lock = self.state.write().unwrap();
        if self.count_fail < self.config.start_retries {
            *state_lock = ProcessState::Backoff;
        } else {
            *state_lock = ProcessState::Fatal;
        }
        ::std::process::exit(1);
    }

    pub fn track_state(&self) {
        let state = self.state.read().unwrap().clone();
        match state {
            ProcessState::Running(ProcessHolder { pid, .. }) => {
                trace!("tracking state");
                loop {
                    match wait::waitpid(pid, None) {
                        Ok(wait::WaitStatus::Exited(_, status)) => {
                            info!(
                                "process {} exited with code {}",
                                self.config.proc_name, status
                            );
                            let mut state_lock = self.state.write().unwrap();
                            *state_lock = ProcessState::Exited(status);
                            break;
                        }
                        Err(e) => {
                            warn!("unexpected error for pid {}", pid);
                            trace!("error: {}", e);
                            let mut state_lock = self.state.write().unwrap();
                            *state_lock = ProcessState::Stopped;
                            break;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    pub fn get_state(&self) -> RwLockReadGuard<ProcessState> {
        self.state.read().unwrap()
    }

    pub fn spawn(&mut self) {
        {
            let mut state_lock = self.state.write().unwrap();
            if *state_lock == ProcessState::Fatal {
                return;
            }
            trace!("spawning process {}", self.config.proc_name);
            *state_lock = ProcessState::Starting;
        }
        let (c_stdin, p_stdin) = pipe().unwrap();
        let (p_stdout, c_stdout) = pipe().unwrap();
        let (p_stderr, c_stderr) = pipe().unwrap();
        match fork() {
            Ok(ForkResult::Child) => {
                //self.setup_io();
                //unsafe {
                //    if let Some(umask) = self.config.umask {
                //        libc::umask(umask);
                //    }
                //    if let Some(ref mut wd) = self.config.directory {
                //        if libc::chdir(wd.to_str().unwrap().as_ptr() as *const i8) == -1 {
                //            self.handle_fail();
                //        }
                //    }
                //}
                //open("/dev/stdin", O_RDONLY, stat::Mode::empty()).unwrap();
                //open("/dev/stdout", O_WRONLY, stat::Mode::empty()).unwrap();
                //open("/dev/stderr", O_WRONLY, stat::Mode::empty()).unwrap();
                close(p_stdin).unwrap();
                close(p_stdout).unwrap();
                close(p_stderr).unwrap();
                dup2(c_stdin, 0).unwrap();
                dup2(c_stdout, 1).unwrap();
                dup2(c_stderr, 2).unwrap();
                trace!("executing command for process {}", self.config.proc_name);
                match self.command.exec() {
                    Ok(_) => {
                        trace!("command executed {}", self.config.proc_name);
                    }
                    Err(e) => {
                        warn!("error when executing {}", self.config.proc_name);
                        trace!("error: {}", e);
                    }
                }
                ::std::process::exit(1);
            }
            Ok(ForkResult::Parent { child }) => {
                let holder = ProcessHolder::new(child)
                    .stdin(p_stdin)
                    .stdout(p_stdout)
                    .stderr(p_stderr);
                close(c_stdin).unwrap();
                close(c_stdout).unwrap();
                close(c_stderr).unwrap();
                let mut state_lock = self.state.write().unwrap();
                *state_lock = ProcessState::Running(holder);
                info!("process {} spawned on pid {}", self.config.proc_name, child);
                //::std::process::exit(0);
            }
            Err(e) => {
                // TODO: respawn
                critical!("error: {:#?}", e);
                self.handle_fail();
            }
        }
    }
}
