#![no_std]
#![no_main]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(lang_items)]
#![feature(default_alloc_error_handler)]
#![feature(link_args)]

#[allow(unused_attributes)]
#[link_args = "-nostartfiles -static"]
extern "C" {}

use static_alloc::Bump;

use core::mem::size_of;

mod memcpy;
mod print;
mod start;
mod syscall;
mod unwind_resume;

use crate::print::println;
pub use crate::start::_start;

extern crate alloc;

#[lang = "eh_personality"]
fn eh_personality() {}

#[global_allocator]
static A: Bump<[u8; 1 << 16]> = Bump::uninit();

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

/// # Safety
///
/// This function performs unsafe pointer aritmethic
pub unsafe fn strlen(mut s: *const u8) -> usize {
    let mut count = 0;
    while *s != b'\0' {
        count += 1;
        s = s.add(1);
    }
    count
}

/// # Safety
///
/// This function performs unsafe pointer aritmethic
#[no_mangle]
pub unsafe fn main(stack_top: *const u8) {
    let argc = *(stack_top as *const isize);
    let argv = stack_top.add(size_of::<*const isize>()) as *const *const u8;
    use core::slice::from_raw_parts as mkslice;
    let args = mkslice(argv, argc as usize);

    for &arg in args {
        let arg = mkslice(arg, strlen(arg));
        println(arg);
    }

    syscall::exit(argc as i32 - 1);
}
