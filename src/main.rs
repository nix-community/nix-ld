#![no_std]
#![no_main]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(lang_items)]
#![feature(link_args)]

#[allow(unused_attributes)]
#[link_args = "-nostartfiles -static"]
extern {}

use core::mem::size_of;
use core::str;

mod string;
mod print;
mod start;
mod syscall;
mod unwind_resume;
use core::fmt::Write;

use crate::print::{PrintBuffer, print};
pub use crate::start::_start;

#[lang = "eh_personality"]
fn eh_personality() {}

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

fn starts_with<T>(slice: &[T], prefix: &[T]) -> bool
where
    T: PartialEq,
{
    if slice.len() < prefix.len() {
        false
    } else {
        // this is not an idiomatic for loop - but the for..in
        // version using ranges *also* pulls in compiler builtins
        // we don't currently have.
        let mut i = 0;
        while i < prefix.len() {
            if slice[i] != prefix[i] {
                return false;
            }
            i += 1;
        }
        true
    }
}

const NIX_LD : &[u8; 7] = b"NIX_LD=";
const NIX_LD_LIB_PATH : &[u8; 20] = b"NIX_LD_LIBRARY_PATH=";

/// # Safety
///
/// This function performs unsafe pointer aritmethic
#[no_mangle]
pub unsafe fn main(stack_top: *const u8) {
    let argc = *(stack_top as *const isize);
    let argv = stack_top.add(size_of::<*const isize>()) as *const *const u8;
    let mut envp = argv.add(argc as usize + 1) as *const *const u8;

    use core::slice::from_raw_parts as mkslice;
    let args = mkslice(argv, argc as usize);

    let mut buf = [0u8; 4096];
    let mut buf = PrintBuffer::new(&mut buf[..]);

    for &arg in args {
        let arg = mkslice(arg, strlen(arg));
        print(arg);
    }

    let mut nix_ld: Option<&[u8]> = None;
    let mut nix_ld_lib_path: Option<&[u8]> = None;

    while !(*envp).is_null() {
        let var = *envp;
        let var = mkslice(var, strlen(var));
        if starts_with(var, NIX_LD) {
            nix_ld = Some(&var[NIX_LD.len()..]);
        };
        if starts_with(var, NIX_LD_LIB_PATH) {
            nix_ld_lib_path = Some(&var[NIX_LD_LIB_PATH.len()..]);
        };

        envp = envp.add(1);
    }

    if let Some(ld) = nix_ld {
        print!(buf, "ld_path {}\n", str::from_utf8_unchecked(ld));
    }

    if let Some(ld_lib_path) = nix_ld_lib_path {
        eprint!(buf, "ld_library_path: {}\n", str::from_utf8_unchecked(ld_lib_path));
    }

    syscall::exit(argc as i32 - 1);
}
