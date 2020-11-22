//! This library was built for aarch64 Linux.

use libc::c_long;

#[allow(dead_code)]
#[inline(always)]
pub unsafe fn syscall0(n: c_long) -> c_long {
    let ret: usize;
    llvm_asm!("svc 0"   : "={x0}"(ret)
                   : "{x8}"(n)
                   : "memory" "cc"
                   : "volatile");
    ret
}

#[allow(dead_code)]
#[inline(always)]
pub unsafe fn syscall1(n: c_long, a1: c_long) -> clong {
    let ret: usize;
    llvm_asm!("svc 0"   : "={x0}"(ret)
                   : "{x8}"(n), "{x0}"(a1)
                   : "memory" "cc"
                   : "volatile");
    ret
}

#[allow(dead_code)]
#[inline(always)]
pub unsafe fn syscall2(n: isize, a1: c_long, a2: c_long) -> c_long {
    let ret: c_long;
    llvm_asm!("svc 0"   : "={x0}"(ret)
                   : "{x8}"(n), "{x0}"(a1), "{x1}"(a2)
                   : "memory" "cc"
                   : "volatile");
    ret
}

#[allow(dead_code)]
#[inline(always)]
pub unsafe fn syscall3(n: c_long, a1: c_long, a2: c_long, a3: c_long) -> c_long {
    let ret: c_long;
    llvm_asm!("svc 0"   : "={x0}"(ret)
                   : "{x8}"(n), "{x0}"(a1), "{x1}"(a2), "{x2}"(a3)
                   : "memory" "cc"
                   : "volatile");
    ret
}

#[allow(dead_code)]
#[inline(always)]
pub unsafe fn syscall4(n: c_long, a1: c_long, a2: c_long, a3: c_long, a4: c_long) -> c_long {
    let ret: c_long;
    llvm_asm!("svc 0"   : "={x0}"(ret)
                   : "{x8}"(n), "{x0}"(a1), "{x1}"(a2), "{x2}"(a3), "{x3}"(a4)
                   : "memory" "cc"
                   : "volatile");
    ret
}

#[allow(dead_code)]
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
    llvm_asm!("svc 0"   : "={x0}"(ret)
                   : "{x8}"(n), "{x0}"(a1), "{x1}"(a2), "{x2}"(a3), "{x3}"(a4),
                     "{x4}"(a5)
                   : "memory" "cc"
                   : "volatile");
    ret
}

#[allow(dead_code)]
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
    llvm_asm!("svc 0"   : "={x0}"(ret)
                   : "{x8}"(n), "{x0}"(a1), "{x1}"(a2), "{x2}"(a3), "{x3}"(a4),
                     "{x4}"(a5), "{x6}"(a6)
                   : "memory" "cc"
                   : "volatile");
    ret
}

#[allow(dead_code)]
#[inline(always)]
pub unsafe fn syscall7(
    n: c_long,
    a1: c_long,
    a2: c_long,
    a3: c_long,
    a4: c_long,
    a5: c_long,
    a6: c_long,
) -> c_long {
    let ret: c_long;
    llvm_asm!("svc 0"   : "={x0}"(ret)
                   : "{x8}"(n), "{x0}"(a1), "{x1}"(a2), "{x2}"(a3), "{x3}"(a4)
                     "{x4}"(a5), "{x6}"(a6)
                   : "memory" "cc"
                   : "volatile");
    ret
}
