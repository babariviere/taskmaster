#![allow(missing_docs)]

use libc;

macro_rules! errno {
    ($($errno:ident)*) => {
        pub enum Errno {
            UNKNOWN,
            $($errno,)*
        }

        impl Errno {
            /// Get last error
            #[cfg(target_os = "macos")]
            pub fn last_error() -> Errno {
                unsafe {
                    Errno::from(*libc::__error())
                }
            }

            /// Get last error
            #[cfg(target_os = "linux")]
            pub fn last_error() -> Errno {
                unsafe {
                    Errno::from(*libc::__errno_location())
                }
            }
        }

        impl From<i32> for Errno {
            fn from(errno: i32) -> Errno {
                match errno {
                    $(
                        ::libc::$errno => Errno::$errno,
                    )*
                    _ => Errno::UNKNOWN,
                }
            }
        }
    }
}

/// Represent an error from libc
errno!(
    EPERM
    ENOENT
    ESRCH
    EINTR
    EIO
    ENXIO
    E2BIG
    ENOEXEC
    EBADF
    ECHILD
    EDEADLK
    ENOMEM
    EACCES
    EFAULT
    ENOTBLK
    EBUSY
    EEXIST
    EXDEV
    ENODEV
    ENOTDIR
    EISDIR
    EINVAL
    ENFILE
    EMFILE
    ENOTTY
    ETXTBSY
    EFBIG
    ENOSPC
    ESPIPE
    EROFS
    EMLINK
    EPIPE
    EDOM
    ERANGE
    EAGAIN
    EINPROGRESS
    EALREADY
    ENOTSOCK
    EDESTADDRREQ
    EMSGSIZE
    EPROTOTYPE
    ENOPROTOOPT
    EPROTONOSUPPORT
    ESOCKTNOSUPPORT
    ENOTSUP
    EPFNOSUPPORT
    EAFNOSUPPORT
    EADDRINUSE
    EADDRNOTAVAIL
    ENETDOWN
    ENETUNREACH
    ENETRESET
    ECONNABORTED
    ECONNRESET
    ENOBUFS
    EISCONN
    ENOTCONN
    ESHUTDOWN
    ETOOMANYREFS
    ETIMEDOUT
    ECONNREFUSED
    ELOOP
    ENAMETOOLONG
    EHOSTDOWN
    EHOSTUNREACH
    ENOTEMPTY
    EPROCLIM
    EUSERS
    EDQUOT
    ESTALE
    EREMOTE
    EBADRPC
    ERPCMISMATCH
    EPROGUNAVAIL
    EPROGMISMATCH
    EPROCUNAVAIL
    ENOLCK
    ENOSYS
    EFTYPE
    EAUTH
    ENEEDAUTH
    EPWROFF
    EDEVERR
    EOVERFLOW
    EBADEXEC
    EBADARCH
    ESHLIBVERS
    EBADMACHO
    ECANCELED
    EIDRM
    ENOMSG
    EILSEQ
    ENOATTR
    EBADMSG
    EMULTIHOP
    ENODATA
    ENOLINK
    ENOSR
    ENOSTR
    EPROTO
    ETIME
    EOPNOTSUPP
    ENOPOLICY
    ENOTRECOVERABLE
    EOWNERDEAD
    EQFULL
);

/// Close all file descriptors
pub fn close_all_fd() {
    let end = unsafe { libc::sysconf(libc::_SC_OPEN_MAX) };
    for i in (0..end - 1).rev() {
        unsafe {
            libc::close(i as i32);
        }
    }
}
