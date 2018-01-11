//! Process module

use command::Command;
use libc;
use std::fs::File;
use std::os::unix::io::IntoRawFd;
use std::path::PathBuf;
use taskmaster::config::*;
use taskmaster::process::*;
use taskmaster::ffi::Errno;

/// Get process state
#[derive(Clone, Debug, PartialEq)]
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
    command: Command,
    state: ProcessState,
    config: ProcessConfig,
    count_fail: u8,
}

impl Process {
    /// Create a new process
    pub fn new(config: ProcessConfig) -> Process {
        Process {
            command: Command::new(&config.command, config.envs.clone().unwrap_or(Vec::new())),
            state: ProcessState::Stopped,
            config: config,
            count_fail: 0,
        }
    }

    fn handle_fail(&mut self) {
        self.count_fail += 1;
        if self.count_fail < self.config.start_retries {
            self.state = ProcessState::Backoff;
        } else {
            self.state = ProcessState::Fatal;
        }
        ::std::process::exit(1);
    }

    pub fn update_state(&mut self) {
        match &self.state {
            &ProcessState::Running(pid) => {
                let mut status = 0;
                let res = unsafe { libc::waitpid(pid, &mut status as *mut _, libc::WNOHANG) };
                if res < 0 {
                    warn!("cannot get process state for pid {}", pid);
                    trace!("got errno {:#?}", Errno::last_error());
                } else if res == 0 {
                    self.state = ProcessState::Exited(status);
                    info!(
                        "process {} exit with status {}",
                        self.config.proc_name, status
                    );
                } else if res != pid {
                    trace!("unexpected value from waitpid: {}", res);
                }
            }
            _ => {}
        }
    }

    pub fn get_state(&mut self) -> &ProcessState {
        self.update_state();
        &self.state
    }

    fn setup_io(&self) {
        if let OutputLog::File(ref f) = self.config.stdout_logfile {
            let file = File::create(f);
            if let Ok(file) = file {
                unsafe {
                    libc::dup2(file.into_raw_fd(), libc::STDOUT_FILENO);
                }
            }
        }
        if let OutputLog::File(ref f) = self.config.stderr_logfile {
            let file = File::create(f);
            if let Ok(file) = file {
                unsafe {
                    libc::dup2(file.into_raw_fd(), libc::STDERR_FILENO);
                }
            }
        }
    }

    // use waitpid with WNOHANG to get status without waiting
    pub fn spawn(&mut self) {
        if self.state == ProcessState::Fatal {
            return;
        }
        trace!("spawning process {}", self.config.proc_name);
        self.state = ProcessState::Starting;
        match fork() {
            Ok(ForkResult::Child) => {
                self.setup_io();
                unsafe {
                    if let Some(umask) = self.config.umask {
                        libc::umask(umask);
                    }
                    if let Some(ref mut wd) = self.config.directory {
                        if libc::chdir(wd.to_str().unwrap().as_ptr() as *const i8) == -1 {
                            self.handle_fail();
                        }
                    }
                }
                self.command.exec();
                warn!("failed to execute {}", self.config.proc_name);
                trace!("got errno: {:#?}", Errno::last_error());
                self.handle_fail();
            }
            Ok(ForkResult::Parent(pid)) => {
                self.state = ProcessState::Running(pid);
                info!("process {} spawned on pid {}", self.config.proc_name, pid);
            }
            Err(_) => {
                // TODO: respawn
                self.handle_fail();
            }
        }
    }
}
