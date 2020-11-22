use crate::syscall::write;
use libc::{STDOUT_FILENO, STDERR_FILENO};

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

pub fn println(s: &[u8]) {
    print(s);
    print(b"\n");
}

pub fn eprintln(s: &[u8]) {
    eprint(s);
    eprint(b"\n");
}
