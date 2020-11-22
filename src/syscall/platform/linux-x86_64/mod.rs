//! This library was built for x86-64 Linux.

use libc::c_long;

#[inline(always)]
pub unsafe fn syscall0(n: c_long) -> c_long {
    let ret: c_long;
    asm!("syscall",
         inout("rax") n => ret,
         lateout("rcx") _, lateout("r11") _,
         options(nostack));
    ret
}

#[inline(always)]
pub unsafe fn syscall1(n: c_long, a1: c_long) -> c_long {
    let ret: c_long;
    asm!("syscall",
         inout("rax") n => ret,
         in("rdi") a1,
         lateout("rcx") _, lateout("r11") _,
         options(nostack));
    ret
}

#[inline(always)]
pub unsafe fn syscall2(n: c_long, a1: c_long, a2: c_long) -> c_long {
    let ret: c_long;
    asm!("syscall",
         inout("rax") n => ret,
         in("rdi") a1,
         in("rsi") a2,
         lateout("rcx") _, lateout("r11") _,
         options(nostack));
    ret
}

#[inline(always)]
pub unsafe fn syscall3(n: c_long, a1: c_long, a2: c_long, a3: c_long) -> c_long {
    let ret: c_long;
    asm!("syscall",
         inout("rax") n => ret,
         in("rdi") a1,
         in("rsi") a2,
         in("rdx") a3,
         lateout("rcx") _, lateout("r11") _,
         options(nostack));
    ret
}

#[inline(always)]
pub unsafe fn syscall4(n: c_long, a1: c_long, a2: c_long, a3: c_long, a4: c_long) -> c_long {
    let ret: c_long;
    asm!("syscall",
         inout("rax") n => ret,
         in("rdi") a1,
         in("rsi") a2,
         in("rdx") a3,
         in("r10") a4,
         lateout("rcx") _, lateout("r11") _,
         options(nostack));
    ret
}

#[inline(always)]
pub unsafe fn syscall5(
    n: c_long,
    a1: c_long,
    a2: c_long,
    a3: c_long,
    a4: c_long,
    a5: c_long,
) -> c_long {
    let ret: c_long;
    asm!("syscall",
         inout("rax") n => ret,
         in("rdi") a1,
         in("rsi") a2,
         in("rdx") a3,
         in("r10") a4,
         in("r8") a5,
         lateout("rcx") _, lateout("r11") _,
         options(nostack));
    ret
}

#[inline(always)]
pub unsafe fn syscall6(
    n: c_long,
    a1: c_long,
    a2: c_long,
    a3: c_long,
    a4: c_long,
    a5: c_long,
    a6: c_long,
) -> c_long {
    let ret: c_long;
    asm!("syscall",
         inout("rax") n => ret,
         in("rdi") a1,
         in("rsi") a2,
         in("rdx") a3,
         in("r10") a4,
         in("r8") a5,
         in("r9") a6,
         lateout("rcx") _, lateout("r11") _,
         options(nostack));
    ret
}
