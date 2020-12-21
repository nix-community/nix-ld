use crate::syscalls;
use core::fmt;
use core::str;
use core::ffi::c_void;

pub struct UnbufferedPrint {
    fd: i32,
}

impl UnbufferedPrint {
    pub fn new(fd: i32) -> Self {
        UnbufferedPrint { fd }
    }
}

impl fmt::Write for UnbufferedPrint {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // not fast but little code and we only print on the error path anyway
        unsafe {
            syscalls::write(self.fd, s.as_ptr() as *const c_void, s.len());
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($fmt:expr $(, $args:expr)*) => {
        {
            use core::fmt::Write;
            let mut buf = crate::print::UnbufferedPrint::new(1);
            // Should not fail because PrintBuffer does not fail
            write!(buf, $fmt, $( $args ),*).unwrap();
        }
    }
}

#[macro_export]
macro_rules! eprint {
    ($fmt:expr $(, $args:expr)*) => {
        {
            use core::fmt::Write;
            let mut buf = crate::print::UnbufferedPrint::new(2);
            // Should not fail because PrintBuffer does not fail
            write!(buf, $fmt, $( $args ),*).unwrap();
        }
    }
}
