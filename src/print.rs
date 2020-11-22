use crate::syscall::write;
use core::fmt;
use core::str;
use libc::{STDOUT_FILENO, STDERR_FILENO};

pub struct PrintBuffer<'a> {
    buf: &'a mut [u8],
    cursor: usize,
}

impl<'a> PrintBuffer<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        PrintBuffer { buf, cursor: 0 }
    }

    pub fn as_bytes(&self) -> &[u8] {
        return self.buf;
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    pub fn clear(&mut self) {
        self.cursor = 0;
    }
}

impl fmt::Write for PrintBuffer<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let cap = self.capacity();
        for (i, &b) in self.buf[self.cursor..cap]
            .iter_mut()
            .zip(s.as_bytes().iter())
        {
            *i = b;
        }
        self.cursor = usize::min(cap, self.cursor + s.as_bytes().len());
        Ok(())
    }
}

pub fn print(s: &[u8]) {
    unsafe {
        write(STDOUT_FILENO as i32, s.as_ptr() as *const libc::c_void, s.len());
    }
}

pub fn eprint(s: &[u8]) {
    unsafe {
        write(STDERR_FILENO as i32, s.as_ptr() as *const libc::c_void, s.len());
    }
}

#[macro_export]
macro_rules! print {
    ($buf:expr, $fmt:expr $(, $args:expr)*) => {
        write!($buf, $fmt, $( $args ),*).unwrap();
        print::print($buf.as_bytes());
        $buf.clear();
    }
}

#[macro_export]
macro_rules! eprint {
    ($buf:expr, $fmt:expr $(, $args:expr)*) => {
        write!($buf, $fmt, $( $args ),*).unwrap();
        print::eprint($buf.as_bytes());
        $buf.clear();
    }
}
