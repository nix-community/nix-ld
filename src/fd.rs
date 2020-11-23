use crate::syscall;
use libc::{c_int, c_void, size_t};

pub struct Fd {
    num: c_int,
}
impl Fd {
    pub fn new(fd: c_int) -> Fd {
        Fd { num: fd }
    }
    pub fn read(&self, buf: *mut c_void, count: size_t) -> libc::ssize_t {
        unsafe { syscall::read(self.num, buf, count) }
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

impl Drop for Fd {
    fn drop(&mut self) {
        let _ = unsafe { syscall::close(self.num) };
    }
}
