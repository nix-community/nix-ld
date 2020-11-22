pub const STDOUT_FILENO: u32 = 1;

pub unsafe fn write(fd: u32, buf: *const u8, count: usize) {
    let syscall_number: u64 = 1;
    asm!(
        "syscall",
        inout("rax") syscall_number => _,
        in("rdi") fd,
        in("rsi") buf,
        in("rdx") count,
        lateout("rcx") _, lateout("r11") _,
        options(nostack)
    );
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
