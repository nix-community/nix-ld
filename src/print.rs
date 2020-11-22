use crate::syscall::write;

pub const STDOUT_FILENO: u32 = 1;

pub fn print(s: &[u8]) {
    unsafe {
        write(STDOUT_FILENO as i32, s.as_ptr() as *const libc::c_void, s.len());
    }
}

pub fn println(s: &[u8]) {
    print(s);
    print(b"\n");
}
