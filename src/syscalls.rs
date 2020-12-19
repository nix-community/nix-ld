use libc::{c_char, c_int, c_long, c_void, off_t, size_t, ssize_t};

extern "C" {
    fn __syscall0(n: c_long) -> c_long;
    fn __syscall1(n: c_long, a1: c_long) -> c_long;
    fn __syscall2(n: c_long, a1: c_long, a2: c_long) -> c_long;
    fn __syscall3(n: c_long, a1: c_long, a2: c_long, a3: c_long) -> c_long;
    fn __syscall4(n: c_long, a1: c_long, a2: c_long, a3: c_long, a4: c_long) -> c_long;
    fn __syscall5(n: c_long, a1: c_long, a2: c_long, a3: c_long, a4: c_long, a5: c_long) -> c_long;
    fn __syscall6(n: c_long, a1: c_long, a2: c_long, a3: c_long, a4: c_long, a5: c_long, a6: c_long) -> c_long;
}

macro_rules! syscall {
    ($nr:expr) => {
        __syscall0($nr)
    };

    ($nr:expr, $a1:expr) => {
        __syscall1($nr, $a1 as c_long)
    };

    ($nr:expr, $a1:expr, $a2:expr) => {
        __syscall2($nr, $a1 as c_long, $a2 as c_long)
    };

    ($nr:expr, $a1:expr, $a2:expr, $a3:expr) => {
        __syscall3($nr, $a1 as c_long, $a2 as c_long, $a3 as c_long)
    };

    ($nr:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr) => {
        __syscall4(
            $nr,
            $a1 as c_long,
            $a2 as c_long,
            $a3 as c_long,
            $a4 as c_long,
        )
    };

    ($nr:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr) => {
        __syscall5(
            $nr,
            $a1 as c_long,
            $a2 as c_long,
            $a3 as c_long,
            $a4 as c_long,
            $a5 as c_long,
        )
    };

    ($nr:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr, $a6:expr) => {
        __syscall6(
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
        __syscall7(
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
    syscall!(libc::SYS_close, fd) as c_int
}

pub unsafe fn write(fd: c_int, buf: *const c_void, count: size_t) -> ssize_t {
    syscall!(libc::SYS_write, fd, buf, count) as ssize_t
}

pub unsafe fn exit(code: c_int) {
    syscall!(libc::SYS_exit, code);
}
