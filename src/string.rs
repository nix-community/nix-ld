use core::mem;
use libc::{c_int, c_void, size_t};

#[no_mangle]
pub unsafe extern "C" fn memcpy(s1: *mut c_void, s2: *const c_void, n: size_t) -> *mut c_void {
    let mut i = 0;
    while i + 7 < n {
        *(s1.add(i) as *mut u64) = *(s2.add(i) as *const u64);
        i += 8;
    }
    while i < n {
        *(s1 as *mut u8).add(i) = *(s2 as *const u8).add(i);
        i += 1;
    }
    s1
}

#[no_mangle]
pub unsafe extern "C" fn memset(s: *mut c_void, c: c_int, n: size_t) -> *mut c_void {
    for i in 0..n {
        *(s as *mut u8).add(i) = c as u8;
    }
    s
}

#[no_mangle]
pub unsafe extern "C" fn memcmp(s1: *const c_void, s2: *const c_void, n: size_t) -> c_int {
    let (div, rem) = (n / mem::size_of::<usize>(), n % mem::size_of::<usize>());
    let mut a = s1 as *const usize;
    let mut b = s2 as *const usize;
    for _ in 0..div {
        if *a != *b {
            for i in 0..mem::size_of::<usize>() {
                let c = *(a as *const u8).add(i);
                let d = *(b as *const u8).add(i);
                if c != d {
                    return c as c_int - d as c_int;
                }
            }
            unreachable!()
        }
        a = a.offset(1);
        b = b.offset(1);
    }

    let mut a = a as *const u8;
    let mut b = b as *const u8;
    for _ in 0..rem {
        if *a != *b {
            return *a as c_int - *b as c_int;
        }
        a = a.offset(1);
        b = b.offset(1);
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn bcmp(first: *const c_void, second: *const c_void, n: size_t) -> c_int {
    memcmp(first, second, n)
}
