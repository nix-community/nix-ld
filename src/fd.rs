use crate::syscall;
use core::slice::from_raw_parts as mkslice;
use libc::{c_int, c_void, off_t, size_t};

pub struct Fd {
    num: c_int,
}

impl Drop for Fd {
    fn drop(&mut self) {
        let _ = unsafe { syscall::close(self.num) };
    }
}

pub struct Mmap<'a> {
    pub data: &'a [u8],
}

impl<'a> Drop for Mmap<'a> {
    fn drop(&mut self) {
        let _ =
            unsafe { syscall::munmap(self.data.as_ptr() as *const libc::c_void, self.data.len()) };
    }
}

impl Fd {
    pub fn read(&self, buf: *mut c_void, count: size_t) -> libc::ssize_t {
        unsafe { syscall::read(self.num, buf, count) }
    }
    pub fn mmap(
        &self,
        addr: *const c_void,
        length: size_t,
        prot: c_int,
        flags: c_int,
        offset: off_t,
    ) -> Result<Mmap, c_int> {
        let res = unsafe { syscall::mmap(addr, length, prot, flags, self.num, offset) };
        if (res as c_int) < 0 {
            Err(-(res as c_int))
        } else {
            Ok(Mmap {
                data: unsafe { mkslice(addr as *const u8, length) },
            })
        }
    }
}

pub fn open(pathname: &[u8], flags: c_int) -> Result<Fd, c_int> {
    let res = unsafe { syscall::open(pathname.as_ptr() as *const i8, flags) };
    if res == -1 {
        Err(res)
    } else {
        Ok(Fd { num: res })
    }
}
