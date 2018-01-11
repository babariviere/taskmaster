//! Process module

use command::Command;
use nix::sys::wait;
use nix::unistd::*;
use std::fs::File;
use std::os::unix::io::IntoRawFd;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use taskmaster::config::*;

/// Get process state
#[derive(Clone, Debug, PartialEq)]
pub enum ProcessState {
    /// Starting process
    Starting,
    /// In running state, param is pid
    Running(Pid),
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

    /// Get proc name
    pub fn proc_name(&self) -> &str {
        &self.config.proc_name
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

    fn track_state(&mut self) {
        match &self.state {
            &ProcessState::Running(pid) => {
                trace!("tracking state");
                match wait::waitpid(pid, None) {
                    Ok(wait::WaitStatus::Exited(_, status)) => {
                        info!(
                            "process {} exited with code {}",
                            self.config.proc_name, status
                        );
                        self.state = ProcessState::Exited(status);
                    }
                    Err(e) => {
                        warn!("unexpected error for pid {}", pid);
                        trace!("error: {}", e);
                        self.state = ProcessState::Stopped;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    pub fn get_state(&mut self) -> &ProcessState {
        &self.state
    }

    fn setup_io(&self) {
        // TODO: fix it
        //if let OutputLog::File(ref f) = self.config.stdout_logfile {
        //    let file = File::create(f);
        //    if let Ok(file) = file {
        //        unsafe {
        //            libc::dup2(file.into_raw_fd(), libc::STDOUT_FILENO);
        //        }
        //    }
        //}
        //if let OutputLog::File(ref f) = self.config.stderr_logfile {
        //    let file = File::create(f);
        //    if let Ok(file) = file {
        //        unsafe {
        //            libc::dup2(file.into_raw_fd(), libc::STDERR_FILENO);
        //        }
        //    }
        //}
    }

    pub fn spawn(&mut self) {
        if self.state == ProcessState::Fatal {
            return;
        }
        trace!("spawning process {}", self.config.proc_name);
        self.state = ProcessState::Starting;
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
                self.state = ProcessState::Running(child);
                info!("process {} spawned on pid {}", self.config.proc_name, child);
                self.track_state();
                ::std::process::exit(0);
            }
            Err(e) => {
                // TODO: respawn
                critical!("error: {:#?}", e);
                self.handle_fail();
            }
        }
    }
}
