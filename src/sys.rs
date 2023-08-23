//! System interface.
//!
//! We currently use `nolibc` for the following things:
//!
//! - `_start` code
//! - Thin syscall wrappers like `open()`, `read()`, `write()` (can be
//!   replaced by `syscalls`)
//!
//! This dependency may be reduced further. For memory operations,
//! compiler-builtins provides faster implementations.

use core::ffi::{c_char, c_int, c_void, CStr};
use core::fmt;
use core::ptr;
use core::slice;

use embedded_io as eio;
pub use embedded_io::{Read, Write};
#[rustfmt::skip]
pub use linux_raw_sys::general::{
    O_RDONLY,
    PROT_NONE, PROT_READ, PROT_WRITE, PROT_EXEC,
    MAP_PRIVATE, MAP_FIXED, MAP_ANONYMOUS,
};
use heapless::Vec as ArrayVec;
pub use linux_raw_sys::errno;

#[link(name = "c_kinda", kind = "static")]
extern "C" {
    pub fn write(fd: i32, buf: *const u8, count: usize) -> isize;
    #[must_use]
    pub fn mmap(
        addr: *mut c_void,
        len: usize,
        prot: u32,
        flags: u32,
        fd: i32,
        offset: isize,
    ) -> *mut c_void;
    pub fn munmap(addr: *mut c_void, len: usize) -> c_int;
    pub fn open(path: *const c_char, oflag: u32, _: ...) -> c_int;
    pub fn read(fd: i32, buf: *mut c_void, count: usize) -> isize;
    pub fn close(fd: i32) -> c_int;
    pub fn abort() -> !;
    pub fn memset(dst: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn execve(prog: *const c_char, argv: *const *const u8, envp: *const *const u8) -> c_int;

    #[link_name = "errno"]
    static c_errno: u32;
}

pub const MAP_FAILED: *mut c_void = !0 as *mut c_void;

macro_rules! if_ok {
    ($ret:ident, $expr:expr) => {
        if $ret < 0 {
            Err(Error::Posix(errno()))
        } else {
            Ok($expr)
        }
    };
    ($ret:ident $($rest:tt)+) => {
        if_ok!($ret, $ret $($rest)*)
    };
}

/// A file.
#[derive(Debug)]
pub struct File(c_int);

/// An error.
///
/// TODO: Convert to human-readable form.
#[derive(Debug)]
pub enum Error {
    Posix(u32),
    PathTooLong,
    Unknown,
}

impl File {
    /// Opens a file.
    ///
    /// This copies the path into a temporary buffer to convert it
    /// into a NUL-terminated string.
    pub fn open(path: &[u8]) -> Result<Self, Error> {
        let mut temp = ArrayVec::<_, 100>::from_slice(path).map_err(|_| Error::PathTooLong)?;
        temp.push(0).map_err(|_| Error::PathTooLong)?;
        let ret = unsafe { open(temp.as_ptr().cast(), O_RDONLY, 0) };
        if_ok!(ret, Self(ret))
    }

    /// Opens a file.
    pub fn open_cstr(path: &CStr) -> Result<Self, Error> {
        let ret = unsafe { open(path.as_ptr(), O_RDONLY, 0) };
        if_ok!(ret, Self(ret))
    }

    /// Returns the underlying file descriptor number.
    pub fn as_raw_fd(&self) -> c_int {
        self.0
    }
}

impl Drop for File {
    fn drop(&mut self) {
        if self.0 > 2 {
            unsafe { close(self.0) };
        }
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let ret = unsafe { read(self.0, buf.as_mut_ptr().cast(), buf.len()) };
        if_ok!(ret as usize)
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let ret = unsafe { write(self.0, buf.as_ptr(), buf.len()) };
        if_ok!(ret as usize)
    }
    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl eio::ErrorType for File {
    type Error = Error;
}

impl fmt::Write for File {
    fn write_str(&mut self, buf: &str) -> fmt::Result {
        eio::Write::write(self, buf.as_bytes()).map_err(|_| fmt::Error)?;

        Ok(())
    }
}

impl eio::Error for Error {
    fn kind(&self) -> embedded_io::ErrorKind {
        eio::ErrorKind::Other
    }
}

impl PartialEq<u32> for Error {
    fn eq(&self, other: &u32) -> bool {
        matches!(self, Self::Posix(num) if num == other)
    }
}

pub const fn stderr() -> impl fmt::Write {
    File(2)
}

#[inline(always)]
pub fn errno() -> u32 {
    unsafe { c_errno }
}

pub fn new_slice_leak(size: usize) -> Option<&'static mut [u8]> {
    let ptr = unsafe {
        mmap(
            ptr::null_mut(),
            size,
            PROT_READ | PROT_WRITE,
            MAP_PRIVATE | MAP_ANONYMOUS,
            -1,
            0,
        )
    };

    if ptr == MAP_FAILED {
        None
    } else {
        Some(unsafe { slice::from_raw_parts_mut(ptr as *mut u8, size) })
    }
}
