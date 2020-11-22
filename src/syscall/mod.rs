#[cfg(all(target_os="linux", target_arch="x86_64"))]
#[path="platform/linux-x86_64/mod.rs"]
pub mod platform;

#[cfg(all(target_os="linux", target_arch="aarch64"))]
#[path="platform/linux-aarch64/mod.rs"]
pub mod platform;

pub use platform::*;

#[macro_export]
macro_rules! syscall {
    ($nr:ident)
        => ( syscall::syscall0($nr) );

    ($nr:ident, $a1:expr)
        => ( syscall::syscall1($nr,
                $a1 as usize) );

    ($nr:ident, $a1:expr, $a2:expr)
        => ( syscall::syscall2($nr,
                $a1 as usize, $a2 as usize) );

    ($nr:ident, $a1:expr, $a2:expr, $a3:expr)
        => ( syscall::syscall3($nr,
                $a1 as usize, $a2 as usize, $a3 as usize) );

    ($nr:ident, $a1:expr, $a2:expr, $a3:expr, $a4:expr)
        => ( syscall::syscall4($nr,
                $a1 as usize, $a2 as usize, $a3 as usize,
                $a4 as usize) );

    ($nr:ident, $a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr)
        => ( syscall::syscall5($nr,
                $a1 as usize, $a2 as usize, $a3 as usize,
                $a4 as usize, $a5 as usize) );

    ($nr:ident, $a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr, $a6:expr)
        => ( syscall::syscall6($nr,
                $a1 as usize, $a2 as usize, $a3 as usize,
                $a4 as usize, $a5 as usize, $a6 as usize) );

    ($nr:ident, $a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr, $a6:expr, $a7:expr)
        => ( syscall::syscall7($nr,
                $a1 as usize, $a2 as usize, $a3 as usize,
                $a4 as usize, $a5 as usize, $a6 as usize,
                $a7 as usize) );
}
