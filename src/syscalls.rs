use core::ffi::c_void;

const SYS_open: isize = 2;
const SYS_close: isize = 3;
const SYS_exit:  isize = 60;
const SYS_mmap:  isize = 9;
const SYS_munmap: isize = 11;
const SYS_read:  isize = 0;
const SYS_write: isize = 1;

extern "C" {
    fn __syscall0(n: isize) -> isize;
    fn __syscall1(n: isize, a1: isize) -> isize;
    fn __syscall2(n: isize, a1: isize, a2: isize) -> isize;
    fn __syscall3(n: isize, a1: isize, a2: isize, a3: isize) -> isize;
    fn __syscall4(n: isize, a1: isize, a2: isize, a3: isize, a4: isize) -> isize;
    fn __syscall5(n: isize, a1: isize, a2: isize, a3: isize, a4: isize, a5: isize) -> isize;
    fn __syscall6(n: isize, a1: isize, a2: isize, a3: isize, a4: isize, a5: isize, a6: isize) -> isize;
}

macro_rules! syscall {
    ($nr:expr) => {
        __syscall0($nr)
    };

    ($nr:expr, $a1:expr) => {
        __syscall1($nr, $a1 as isize)
    };

    ($nr:expr, $a1:expr, $a2:expr) => {
        __syscall2($nr, $a1 as isize, $a2 as isize)
    };

    ($nr:expr, $a1:expr, $a2:expr, $a3:expr) => {
        __syscall3($nr, $a1 as isize, $a2 as isize, $a3 as isize)
    };

    ($nr:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr) => {
        __syscall4(
            $nr,
            $a1 as isize,
            $a2 as isize,
            $a3 as isize,
            $a4 as isize,
        )
    };

    ($nr:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr) => {
        __syscall5(
            $nr,
            $a1 as isize,
            $a2 as isize,
            $a3 as isize,
            $a4 as isize,
            $a5 as isize,
        )
    };

    ($nr:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr, $a6:expr) => {
        __syscall6(
            $nr,
            $a1 as isize,
            $a2 as isize,
            $a3 as isize,
            $a4 as isize,
            $a5 as isize,
            $a6 as isize,
        )
    };

    ($nr:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr, $a6:expr, $a7:expr) => {
        __syscall7(
            $nr,
            $a1 as isize,
            $a2 as isize,
            $a3 as isize,
            $a4 as isize,
            $a5 as isize,
            $a6 as isize,
            $a7 as isize,
        )
    };
}

pub unsafe fn mmap(
    addr: *const c_void,
    length: usize,
    prot: i32,
    flags: i32,
    fd: i32,
    offset: i64,
) -> *mut c_void {
    syscall!(SYS_mmap, addr, length, prot, flags, fd) as *mut c_void
}

pub unsafe fn munmap(addr: *const c_void, length: usize) -> *mut c_void {
    syscall!(SYS_munmap, addr, length) as *mut c_void
}

pub unsafe fn open(pathname: *const u8, flags: i32) -> i32 {
    syscall!(SYS_open, pathname, flags) as i32
}

pub unsafe fn read(fd: i32, buf: *mut c_void, count: usize) -> isize {
    syscall!(SYS_read, fd, buf, count) as isize
}

pub unsafe fn close(fd: i32) -> i32 {
    syscall!(SYS_close, fd) as i32
}

pub unsafe fn write(fd: i32, buf: *const c_void, count: usize) -> isize {
    syscall!(SYS_write, fd, buf, count) as isize
}

pub unsafe fn exit(code: i32) {
    syscall!(SYS_exit, code);
}
