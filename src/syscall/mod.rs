use libc::{c_char, c_int, c_long, c_void, off_t, size_t, ssize_t};

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[path = "platform/linux-x86_64/mod.rs"]
mod platform;

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
#[path = "platform/linux-aarch64/mod.rs"]
mod platform;

use platform::*;

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

pub unsafe fn mmap(
    addr: *const c_void,
    length: size_t,
    prot: c_int,
    flags: c_int,
    fd: c_int,
    offset: off_t,
) -> *mut c_void {
    syscall!(libc::SYS_mmap, addr, length, prot, flags, fd, offset) as *mut c_void
}

pub unsafe fn munmap(addr: *const c_void, length: size_t) -> *mut c_void {
    syscall!(libc::SYS_munmap, addr, length) as *mut c_void
}

pub unsafe fn open(pathname: *const c_char, flags: c_int) -> c_int {
    syscall!(libc::SYS_open, pathname, flags) as c_int
}

pub unsafe fn read(fd: c_int, buf: *mut c_void, count: size_t) -> ssize_t {
    syscall!(libc::SYS_read, fd, buf, count) as ssize_t
}

pub unsafe fn close(fd: c_int) -> c_int {
    syscall!(libc::SYS_open, fd) as c_int
}

pub unsafe fn write(fd: c_int, buf: *const c_void, count: size_t) -> ssize_t {
    syscall!(libc::SYS_write, fd, buf, count) as ssize_t
}

pub unsafe fn exit(code: c_int) {
    syscall!(libc::SYS_exit, code);
}
