#![no_std]
#![no_main]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(lang_items)]
#![feature(default_alloc_error_handler)]

use static_alloc::Bump;
mod print;
mod memcpy;
mod unwind_resume;
mod exit;

use crate::exit::exit;
use crate::print::println;

extern crate alloc;

#[lang = "eh_personality"]
fn eh_personality() {}

#[global_allocator]
static A: Bump<[u8; 1 << 16]> = Bump::uninit();

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

pub unsafe fn strlen(mut s: *const u8) -> usize {
    let mut count = 0;
    while *s != b'\0' {
        count += 1;
        s = s.add(1);
    }
    count
}

#[no_mangle]
pub unsafe fn main(stack_top: *const u8) {
    let argc = *(stack_top as *const u64);
    let argv = stack_top.add(8) as *const *const u8;
    use core::slice::from_raw_parts as mkslice;
    let args = mkslice(argv, argc as usize);

    for &arg in args {
        let arg = mkslice(arg, strlen(arg));
        println(arg);
    }

    exit(argc as i32 - 1);
}
