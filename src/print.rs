use crate::syscall;
use core::fmt;
use core::str;
use libc::c_int;

pub struct UnbufferedPrint {
    fd: c_int,
}

impl UnbufferedPrint {
    pub fn new(fd: c_int) -> Self {
        UnbufferedPrint { fd }
    }
}

impl fmt::Write for UnbufferedPrint {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // not fast but little code and we only print on the error path anyway
        unsafe {
            syscall::write(self.fd, s.as_ptr() as *const libc::c_void, s.len());
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($fmt:expr $(, $args:expr)*) => {
        {
            use core::fmt::Write;
            let mut buf = crate::print::UnbufferedPrint::new(::libc::STDOUT_FILENO);
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
            let mut buf = crate::print::UnbufferedPrint::new(::libc::STDERR_FILENO);
            // Should not fail because PrintBuffer does not fail
            write!(buf, $fmt, $( $args ),*).unwrap();
        }
    }
}
