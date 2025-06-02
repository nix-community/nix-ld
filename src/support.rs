//! Low-level support.

use core::fmt::Write;

use crate::arch::STACK_ALIGNMENT;
use crate::sys;

pub static LOGGER: StderrLogger = StderrLogger;

pub struct StderrLogger;

impl log::Log for StderrLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let mut stderr = sys::stderr();
            writeln!(stderr, "[nix-ld] {:>5}: {}", record.level(), record.args()).unwrap();
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

/// Aborts the program because something went terribly wrong.
///
/// Unlike panic!(), this doesn't trigger the panic-handling
/// or formatting machinery.
#[cold]
pub fn explode(s: &str) -> ! {
    let prefix = "[nix-ld] FATAL: ";

    unsafe {
        sys::write(2, prefix.as_ptr(), prefix.len());
        sys::write(2, s.as_ptr(), s.len());
        sys::write(2, "\n".as_ptr(), 1);
        sys::abort();
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    let mut stderr = sys::stderr();
    writeln!(stderr, "[nix-ld] FATAL: {}", info).unwrap();

    unsafe {
        sys::abort();
    }
}

#[unsafe(no_mangle)]
extern "C" fn __stack_chk_fail() -> ! {
    explode("stack smashing detected");
}
