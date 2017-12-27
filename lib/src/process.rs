use ffi::errno::*;

#[derive(Debug)]
pub enum ForkError {
    LimitExceeded,
    InsufficientSpace,
}

#[derive(Debug)]
pub enum ForkResult {
    Child,
    Parent(i32),
}

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
