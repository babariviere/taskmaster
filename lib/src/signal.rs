//! Manage signal

use libc;
use ffi::Errno;

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
    pub fn kill(&self, pid: libc::c_int) -> Result<(), Errno> {
        unsafe {
            match libc::kill(pid, *self as libc::c_int) {
                0 => Ok(()),
                _ => Err(Errno::last_error()),
            }
        }
    }
}

impl Default for StopSignal {
    fn default() -> Self {
        StopSignal::Term
    }
}
