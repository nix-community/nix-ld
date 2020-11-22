#[cfg(all(target_os="linux", target_arch="x86_64"))]
#[path="platform/linux-x86_64/mod.rs"]
pub mod platform;

#[cfg(all(target_os="linux", target_arch="aarch64"))]
#[path="platform/linux-aarch64/mod.rs"]
pub mod platform;

pub use platform::*;

macro_rules! syscall {
    ($nr:ident)
        => ( syscall0($nr) );

    ($nr:ident, $a1:expr)
        => ( syscall1($nr,
                $a1 as usize) );

    ($nr:ident, $a1:expr, $a2:expr)
        => ( syscall2($nr,
                $a1 as usize, $a2 as usize) );

    ($nr:ident, $a1:expr, $a2:expr, $a3:expr)
        => ( syscall3($nr,
                $a1 as usize, $a2 as usize, $a3 as usize) );

    ($nr:ident, $a1:expr, $a2:expr, $a3:expr, $a4:expr)
        => ( syscall4($nr,
                $a1 as usize, $a2 as usize, $a3 as usize,
                $a4 as usize) );

    ($nr:ident, $a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr)
        => ( syscall5($nr,
                $a1 as usize, $a2 as usize, $a3 as usize,
                $a4 as usize, $a5 as usize) );

    ($nr:ident, $a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr, $a6:expr)
        => ( syscall6($nr,
                $a1 as usize, $a2 as usize, $a3 as usize,
                $a4 as usize, $a5 as usize, $a6 as usize) );

    ($nr:ident, $a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr, $a6:expr, $a7:expr)
        => ( syscall7($nr,
                $a1 as usize, $a2 as usize, $a3 as usize,
                $a4 as usize, $a5 as usize, $a6 as usize,
                $a7 as usize) );
}


pub unsafe fn write(fd: u32, buf: *const u8, count: usize) {
    let syscall_number: usize = 1;
    syscall!(syscall_number, fd, buf, count);
}

pub unsafe fn exit(code: i32) {
    let syscall_number: usize = 60;
    syscall!(syscall_number, code);
}
