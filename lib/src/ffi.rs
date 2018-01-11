#![allow(missing_docs)]

use nix::unistd::*;

/// Close all file descriptors
pub fn close_all_fd() {
    let end = sysconf(SysconfVar::OPEN_MAX).unwrap().unwrap();
    for i in (0..end - 1).rev() {
        let _ = close(i as i32);
    }
}
