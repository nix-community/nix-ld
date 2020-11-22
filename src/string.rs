#[no_mangle]
pub unsafe extern "C" fn memcpy(s1: *mut u8, s2: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;
    while i + 7 < n {
        *(s1.add(i) as *mut u64) = *(s2.add(i) as *const u64);
        i += 8;
    }
    while i < n {
        *s1.add(i) = *s2.add(i);
        i += 1;
    }
    s1
}

#[no_mangle]
pub unsafe extern "C" fn memset(s: *mut u8, c: isize, n: usize) -> *mut u8 {
    for i in 0..n {
        *(s as *mut u8).add(i) = c as u8;
    }
    s
}
