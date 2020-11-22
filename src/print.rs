pub const STDOUT_FILENO: u32 = 1;

use crate::syscall;

pub unsafe fn write(fd: u32, buf: *const u8, count: usize) {
    let syscall_number: usize = 1;
    syscall!(syscall_number, fd, buf, count);
}

pub fn print(s: &[u8]) {
    unsafe {
        write(STDOUT_FILENO, s.as_ptr(), s.len());
    }
}


pub fn println(s: &[u8]) {
    print(s);
    print(b"\n");
}
