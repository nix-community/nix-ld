//! This library was built for x86-64 Linux.

#[inline(always)]
pub unsafe fn syscall0(n: usize) -> usize {
    let ret : usize;
    asm!("syscall",
         inout("rax") n => ret,
         lateout("rcx") _, lateout("r11") _,
         options(nostack));
    ret
}

#[inline(always)]
pub unsafe fn syscall1(n: usize, a1: usize) -> usize {
    let ret : usize;
    asm!("syscall",
         inout("rax") n => ret,
         in("rdi") a1,
         lateout("rcx") _, lateout("r11") _,
         options(nostack));
    ret
}

#[inline(always)]
pub unsafe fn syscall2(n: usize, a1: usize, a2: usize) -> usize {
    let ret : usize;
    asm!("syscall",
         inout("rax") n => ret,
         in("rdi") a1,
         in("rsi") a2,
         lateout("rcx") _, lateout("r11") _,
         options(nostack));
    ret
}

#[inline(always)]
pub unsafe fn syscall3(n: usize, a1: usize, a2: usize, a3: usize) -> usize {
    let ret : usize;
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
pub unsafe fn syscall4(n: usize, a1: usize, a2: usize, a3: usize,
                                a4: usize) -> usize {
    let ret : usize;
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
pub unsafe fn syscall5(n: usize, a1: usize, a2: usize, a3: usize,
                                a4: usize, a5: usize) -> usize {
    let ret : usize;
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
pub unsafe fn syscall6(n: usize, a1: usize, a2: usize, a3: usize,
                                a4: usize, a5: usize, a6: usize) -> usize {
    let ret : usize;
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
