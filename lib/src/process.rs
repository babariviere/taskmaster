//! Module to create process

use libc;
use ffi::Errno;

/// Result of the fork
#[derive(Debug)]
pub enum ForkResult {
    /// Currently in child process
    Child,
    /// Currently in parent process, i32 is the child pid
    Parent(i32),
}

/// Fork syscall
pub fn fork() -> Result<ForkResult, Errno> {
    let res = unsafe { libc::fork() };
    if res < 0 {
        return Err(Errno::last_error());
    }
    if res == 0 {
        Ok(ForkResult::Child)
    } else {
        Ok(ForkResult::Parent(res))
    }
}
