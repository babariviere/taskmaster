//! Process module

use command::Command;
use nix::fcntl;
use nix::sys::{stat, wait};
use nix::unistd::*;
use std::os::unix::io::*;
use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard};
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

// TODO: use signal to handle process
#[derive(Clone, Debug, PartialEq)]
pub struct ProcessHolder {
    stdin: Option<RawFd>,
    stdout: Option<RawFd>,
    stderr: Option<RawFd>,
    stdout_readed: Vec<u8>,
    stderr_readed: Vec<u8>,
}

impl ProcessHolder {
    /// create a new process holder
    pub fn new() -> ProcessHolder {
        ProcessHolder {
            stdin: None,
            stdout: None,
            stderr: None,
            stdout_readed: Vec::new(),
            stderr_readed: Vec::new(),
        }
    }

    /// set stdin
    pub fn stdin(mut self, stdin: RawFd) -> ProcessHolder {
        self.stdin = Some(stdin);
        self
    }

    /// Set stdout
    pub fn stdout(mut self, stdout: RawFd) -> ProcessHolder {
        match fcntl::fcntl(stdout, fcntl::FcntlArg::F_GETFL) {
            Ok(f) => match fcntl::fcntl(
                stdout,
                fcntl::FcntlArg::F_SETFL(fcntl::OFlag::from_bits_truncate(f) | fcntl::O_NONBLOCK),
            ) {
                Ok(_) => {}
                Err(e) => trace!("error with fcntl: {}", e),
            },
            Err(_) => {}
        };
        self.stdout = Some(stdout);
        self
    }

    /// Set stderr
    pub fn stderr(mut self, stderr: RawFd) -> ProcessHolder {
        match fcntl::fcntl(stderr, fcntl::FcntlArg::F_GETFL) {
            Ok(f) => match fcntl::fcntl(
                stderr,
                fcntl::FcntlArg::F_SETFL(fcntl::OFlag::from_bits_truncate(f) | fcntl::O_NONBLOCK),
            ) {
                Ok(_) => {}
                Err(e) => trace!("error with fcntl: {}", e),
            },
            Err(_) => {}
        };
        self.stderr = Some(stderr);
        self
    }

    /// Get stdout
    pub fn get_stdout(&self) -> &Vec<u8> {
        &self.stdout_readed
    }

    /// Get stderr
    pub fn get_stderr(&self) -> &Vec<u8> {
        &self.stderr_readed
    }

    /// Write to stdin
    pub fn write_stdin(&self, buf: &[u8]) -> ::nix::Result<usize> {
        if let Some(fd) = self.stdin {
            write(fd, buf)
        } else {
            Ok(0)
        }
    }

    /// Read to stdout
    pub fn read_stdout(&mut self) {
        blather!("started reading stdout");
        if let Some(fd) = self.stdout {
            let mut buf = [0; 1024];
            while let Ok(size) = read(fd, &mut buf) {
                blather!("stdout read: {}", size);
                if size == 0 {
                    break;
                }
                self.stdout_readed.extend(buf.iter());
                buf = [0; 1024];
            }
        }
        blather!("ended reading stdout");
    }

    /// Read to stderr
    pub fn read_stderr(&mut self) {
        blather!("started reading stderr");
        if let Some(fd) = self.stderr {
            let mut buf = [0; 1024];
            while let Ok(size) = read(fd, &mut buf) {
                blather!("stderr read: {}", size);
                if size == 0 {
                    break;
                }
                self.stderr_readed.extend(buf.iter());
                buf = [0; 1024];
            }
        }
        blather!("ended reading stderr");
    }
}

impl Drop for ProcessHolder {
    fn drop(&mut self) {
        if let Some(fd) = self.stdin {
            let _ = close(fd);
        }
        if let Some(fd) = self.stdout {
            let _ = close(fd);
        }
        if let Some(fd) = self.stderr {
            let _ = close(fd);
        }
    }
}

/// Process handler
#[derive(Debug)]
pub struct Process {
    command: Command,
    state: RwLock<ProcessState>,
    config: ProcessConfig,
    count_fail: u8,
    holder: Mutex<ProcessHolder>,
}

impl Process {
    /// Create a new process
    pub fn new(config: ProcessConfig) -> Process {
        Process {
            command: Command::new(&config.command, config.envs.clone().unwrap_or(Vec::new())),
            state: RwLock::new(ProcessState::Stopped),
            config: config,
            count_fail: 0,
            holder: Mutex::new(ProcessHolder::new()),
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

    pub fn kill(&self) {
        let state = self.state.read().unwrap().clone();
        let pid = match state {
            ProcessState::Running(pid) => pid,
            _ => return,
        };
        drop(state);
        let mut state_lock = self.state.write().unwrap();
        *state_lock = ProcessState::Stopping;
        drop(state_lock);
        match self.config.stop_signal.kill(pid) {
            Ok(_) => {}
            Err(e) => {
                error!("killing pid {} failed", pid);
                trace!("error: {}", e);
            }
        }
    }

    pub fn track_state(&self) {
        let state = self.state.read().unwrap().clone();
        let pid = match state {
            ProcessState::Running(pid) => pid,
            _ => return,
        };
        drop(state);
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
                    drop(state_lock);
                    break;
                }
                Ok(wait::WaitStatus::Signaled(_, _, _)) => {
                    let mut state_lock = self.state.write().unwrap();
                    if *state_lock == ProcessState::Stopping {
                        *state_lock = ProcessState::Stopped;
                    }
                    drop(state_lock);
                }
                Ok(s) => {
                    blather!("pid {} received status {:#?}", pid, s);
                }
                Err(e) => {
                    warn!("unexpected error for pid {}", pid);
                    trace!("error: {}", e);
                    let mut state_lock = self.state.write().unwrap();
                    *state_lock = ProcessState::Stopped;
                    drop(state_lock);
                    break;
                }
            }
        }
    }

    pub fn holder(&self) -> MutexGuard<ProcessHolder> {
        self.holder.lock().unwrap()
    }

    pub fn get_state(&self) -> RwLockReadGuard<ProcessState> {
        self.state.read().unwrap()
    }

    pub fn spawn(&mut self) {
        let mut state_lock = self.state.write().unwrap();
        if *state_lock == ProcessState::Fatal {
            return;
        }
        trace!("spawning process {}", self.config.proc_name);
        *state_lock = ProcessState::Starting;
        drop(state_lock);
        let (c_stdin, p_stdin) = pipe().unwrap();
        let (p_stdout, c_stdout) = pipe().unwrap();
        let (p_stderr, c_stderr) = pipe().unwrap();
        match fork() {
            Ok(ForkResult::Child) => {
                if let Some(mask) = self.config.umask {
                    stat::umask(stat::Mode::from_bits_truncate(mask));
                }
                if let Some(ref mut wd) = self.config.directory {
                    match chdir(wd) {
                        Ok(_) => {}
                        Err(e) => {
                            trace!("error in process {}: {}", self.config.proc_name, e);
                            self.handle_fail();
                        }
                    }
                }

                close(p_stdin).unwrap();
                close(p_stdout).unwrap();
                close(p_stderr).unwrap();
                dup2(c_stdin, 0).unwrap();
                dup2(c_stdout, 1).unwrap();
                dup2(c_stderr, 2).unwrap();
                close(c_stdin).unwrap();
                close(c_stdout).unwrap();
                close(c_stderr).unwrap();
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
                let mut holder_lock = self.holder.lock().unwrap();
                *holder_lock = ProcessHolder::new()
                    .stdin(p_stdin)
                    .stdout(p_stdout)
                    .stderr(p_stderr);
                drop(holder_lock);
                close(c_stdin).unwrap();
                close(c_stdout).unwrap();
                close(c_stderr).unwrap();
                let mut state_lock = self.state.write().unwrap();
                *state_lock = ProcessState::Running(child);
                drop(state_lock);
                info!("process {} spawned on pid {}", self.config.proc_name, child);
            }
            Err(e) => {
                // TODO: respawn
                critical!("error: {:#?}", e);
                self.handle_fail();
            }
        }
    }
}
