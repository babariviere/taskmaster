#![allow(missing_docs)]

use libc;

/// Close all file descriptors
pub fn close_all_fd() {
    let end = unsafe { libc::sysconf(libc::_SC_OPEN_MAX) };
    for i in (0..end - 1).rev() {
        unsafe {
            libc::close(i as i32);
        }
    }
}
