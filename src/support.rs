//! Low-level support.

use core::fmt::{self, Write};
use core::panic::PanicInfo;

use crate::arch::STACK_ALIGNMENT;
use crate::nolibc;

pub static LOGGER: StderrLogger = StderrLogger;

pub struct Fd(i32);
impl fmt::Write for Fd {
    fn write_str(&mut self, buf: &str) -> fmt::Result {
        unsafe {
            nolibc::write(self.0, buf.as_ptr(), buf.len());
        }

        Ok(())
    }
}

pub struct StderrLogger;

impl log::Log for StderrLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let mut fd = Fd(2);
            write!(fd, "[nix-ld] {}: {}\n", record.level(), record.args()).unwrap();
        }
    }

    fn flush(&self) {}
}

#[repr(transparent)]
pub struct StackSpace([u8; 1024 * 1024 * 5]);

impl StackSpace {
    pub fn bottom(&self) -> *const u8 {
        let end = self.0.as_ptr() as usize + self.0.len();
        let aligned = end & !(STACK_ALIGNMENT - 1);
        aligned as *const u8
    }
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    let mut stderr = Fd(2);
    write!(stderr, "[nix-ld] FATAL: {}\n", info).unwrap();

    unsafe {
        nolibc::abort();
    }
}

#[no_mangle]
extern "C" fn __stack_chk_fail() -> ! {
    explode("stack smashing detected");
}

pub fn explode(s: &str) -> ! {
    let prefix = "[nix-ld] FATAL: ";

    unsafe {
        nolibc::write(2, prefix.as_ptr(), prefix.len());
        nolibc::write(2, s.as_ptr(), s.len());
        nolibc::write(2, "\n".as_ptr(), 1);
        nolibc::abort();
    }
}
