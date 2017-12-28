//! Module to create process

use libc;

/// Error when forking
#[derive(Debug)]
pub enum ForkError {
    /// Too much processes are currently running
    LimitExceeded,
    /// Not enough swap space
    InsufficientSpace,
}

/// Result of the fork
#[derive(Debug)]
pub enum ForkResult {
    /// Currently in child process
    Child,
    /// Currently in parent process, i32 is the child pid
    Parent(i32),
}

/// Fork syscall
pub fn fork() -> Result<ForkResult, ForkError> {
    let res = unsafe { libc::fork() };
    if res < 0 {
        unsafe {
            match *libc::__error() {
                libc::EAGAIN => return Err(ForkError::LimitExceeded),
                libc::ENOMEM => return Err(ForkError::InsufficientSpace),
                _ => unreachable!(),
            }
        }
    }
    if res == 0 {
        Ok(ForkResult::Child)
    } else {
        Ok(ForkResult::Parent(res))
    }
}
