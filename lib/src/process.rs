//! Module to create process

use ffi::errno::*;

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
    let res = unsafe { super::ffi::fork() };
    if res < 0 {
        match super::ffi::errno() {
            EAGAIN => return Err(ForkError::LimitExceeded),
            ENOMEM => return Err(ForkError::InsufficientSpace),
            _ => unreachable!(),
        }
    }
    if res == 0 {
        Ok(ForkResult::Child)
    } else {
        Ok(ForkResult::Parent(res))
    }
}
