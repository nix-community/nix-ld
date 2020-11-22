use libc::{c_long, c_int, size_t, ssize_t, c_void};

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[path = "platform/linux-x86_64/mod.rs"]
pub mod platform;

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
#[path = "platform/linux-aarch64/mod.rs"]
pub mod platform;

pub use platform::*;

macro_rules! syscall {
    ($nr:expr) => {
        syscall0($nr)
    };

    ($nr:expr, $a1:expr) => {
        syscall1($nr, $a1 as c_long)
    };

    ($nr:expr, $a1:expr, $a2:expr) => {
        syscall2($nr, $a1 as c_long, $a2 as c_long)
    };

    ($nr:expr, $a1:expr, $a2:expr, $a3:expr) => {
        syscall3($nr, $a1 as c_long, $a2 as c_long, $a3 as c_long)
    };

    ($nr:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr) => {
        syscall4(
            $nr,
            $a1 as c_long,
            $a2 as c_long,
            $a3 as c_long,
            $a4 as c_long,
        )
    };

    ($nr:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr) => {
        syscall5(
            $nr,
            $a1 as c_long,
            $a2 as c_long,
            $a3 as c_long,
            $a4 as c_long,
            $a5 as c_long,
        )
    };

    ($nr:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr, $a6:expr) => {
        syscall6(
            $nr,
            $a1 as c_long,
            $a2 as c_long,
            $a3 as c_long,
            $a4 as c_long,
            $a5 as c_long,
            $a6 as c_long,
        )
    };

    ($nr:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr, $a6:expr, $a7:expr) => {
        syscall7(
            $nr,
            $a1 as c_long,
            $a2 as c_long,
            $a3 as c_long,
            $a4 as c_long,
            $a5 as c_long,
            $a6 as c_long,
            $a7 as c_long,
        )
    };
}

//pub unsafe fn writev(fd: usize, const struct iovec *iov, int iovcnt);

pub unsafe fn write(fd: c_int, buf: *const c_void, count: size_t) -> ssize_t {
    syscall!(libc::SYS_write, fd, buf, count) as isize
}

pub unsafe fn exit(code: c_int) {
    syscall!(libc::SYS_exit, code);
}
