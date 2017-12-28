#![allow(missing_docs)]

pub mod errno {
    pub const EAGAIN: i32 = 11;
    pub const ENOMEM: i32 = 12;
}

pub const SC_OPEN_MAX: i32 = 5;

extern "C" {
    pub fn chdir(dir: *const u8) -> i32;
    pub fn close(fd: i32) -> i32;
    pub fn fork() -> i32;
    pub fn sysconf(name: i32) -> i64;
    pub fn umask(mask: u16) -> u16;

    fn __error() -> *const i32;
}

/// Get errno
pub fn errno() -> i32 {
    unsafe { *__error() }
}

/// Close all file descriptors
pub fn close_all_fd() {
    let end = unsafe { sysconf(SC_OPEN_MAX) };
    for i in (0..end - 1).rev() {
        unsafe {
            close(i as i32);
        }
    }
}
