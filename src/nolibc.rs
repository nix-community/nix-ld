//! nolibc
//!
//! For memory operations, compiler-builtins provides faster
//! implementations.

use core::ffi::{c_char, c_int, c_void};
use core::ptr;
use core::slice;

#[link(name = "c_kinda", kind = "static")]
extern "C" {
    pub fn write(fd: i32, buf: *const u8, count: usize) -> isize;
    #[must_use]
    pub fn mmap(
        addr: *mut c_void,
        len: usize,
        prot: i32,
        flags: i32,
        fd: i32,
        offset: isize,
    ) -> *mut c_void;
    pub fn munmap(addr: *mut c_void, len: usize) -> c_int;
    pub fn open(path: *const c_char, oflag: i32, _: ...) -> c_int;
    pub fn read(fd: i32, buf: *mut c_void, count: usize) -> isize;
    pub fn abort() -> !;
    pub fn memset(dst: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn execve(prog: *const c_char, argv: *const *const u8, envp: *const *const u8) -> c_int;

    fn alloc_temp_c(
        size: usize,
        f: unsafe extern "C" fn(*mut u8, usize, *mut c_void) -> i32,
        closure_ptr: *mut c_void,
    ) -> i32;

    #[link_name = "errno"]
    static c_errno: i32;
}

pub const O_RDONLY: i32 = 0;

pub const PROT_NONE: i32 = 0x0;
pub const PROT_READ: i32 = 0x1;
pub const PROT_WRITE: i32 = 0x2;
pub const PROT_EXEC: i32 = 0x4;

pub const ENOENT: i32 = 0x2;

pub const MAP_PRIVATE: i32 = 0x02;
pub const MAP_FIXED: i32 = 0x10;
pub const MAP_ANONYMOUS: i32 = 0x20;
pub const MAP_FAILED: *mut c_void = !0 as *mut c_void;

type AllocaClosureObj<'a> = &'a mut dyn FnMut(&mut [u8]) -> i32;

pub fn errno() -> i32 {
    unsafe { c_errno }
}

pub fn alloc_temp<F>(size: usize, mut f: F) -> i32
where
    F: FnMut(&mut [u8]) -> i32,
{
    // Well, we have to do this dance
    // <https://stackoverflow.com/a/32270215>
    unsafe extern "C" fn wrapper(buf: *mut u8, size: usize, closure_ptr: *mut c_void) -> i32 {
        if buf.is_null() {
            panic!("Failed to allocate on stack");
        }

        let slice = slice::from_raw_parts_mut(buf, size);
        let closure_ptr: *mut AllocaClosureObj = closure_ptr.cast();
        let closure_obj: AllocaClosureObj = unsafe { *closure_ptr };
        closure_obj(slice)
    }

    let closure_ptr = {
        let mut obj = &mut f as AllocaClosureObj;
        &mut obj as *mut _ as *mut AllocaClosureObj
    };

    unsafe { alloc_temp_c(size, wrapper, closure_ptr as *mut c_void) }
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
