//! Manage signal

use nix::libc;
use nix::sys::signal::kill;
use nix::sys::signal::Signal;
use nix::unistd::Pid;

/// Signal to stop a program
#[derive(Clone, Copy, Debug)]
#[repr(i32)]
pub enum StopSignal {
    /// Term signal
    Term = libc::SIGTERM,
    /// Hup signal
    Hup = libc::SIGHUP,
    /// Int signal
    Int = libc::SIGINT,
    /// Quit signal
    Quit = libc::SIGQUIT,
    /// Kill signal
    Kill = libc::SIGKILL,
    /// User 1 signal
    Usr1 = libc::SIGUSR1,
    /// User 2 signal
    Usr2 = libc::SIGUSR2,
}

impl StopSignal {
    /// Kill a process
    pub fn kill(&self, pid: Pid) -> ::nix::Result<()> {
        kill(pid, Some(Signal::from_c_int(*self as i32).unwrap()))
    }
}

impl Default for StopSignal {
    fn default() -> Self {
        StopSignal::Term
    }
}
