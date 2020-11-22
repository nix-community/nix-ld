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
use core::slice::from_raw_parts as mkslice;

use crate::print::PrintBuffer;
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

const NIX_LD : &'static str = "NIX_LD=";
const NIX_LD_LIB_PATH : &'static str = "NIX_LD_LIBRARY_PATH=";

struct LdConfig {
    exe: Option<&'static str>,
    lib_path: Option<&'static str>,
}

unsafe fn str_slice_from_ptr(ptr: *const u8) -> &'static str {
    str::from_utf8_unchecked(mkslice(ptr, strlen(ptr)))
}

unsafe fn process_env(mut envp: *const *const u8) -> LdConfig {
    let mut config = LdConfig { exe: None, lib_path: None };
    while !(*envp).is_null() {
        let var = str_slice_from_ptr(*envp);
        if var.starts_with(NIX_LD) {
            config.exe = Some(&var[NIX_LD.len()..]);
        };
        if var.starts_with(NIX_LD_LIB_PATH) {
            config.lib_path = Some(&var[NIX_LD_LIB_PATH.len()..]);
        };

        envp = envp.add(1);
    }
    config
}

unsafe fn exe_name(args: &[*const u8]) -> &str {
    if args.len() > 0 {
        str_slice_from_ptr(args[0])
    } else {
        ""
    }
}

/// # Safety
///
/// This function performs unsafe pointer aritmethic
#[no_mangle]
pub unsafe fn main(stack_top: *const u8) {
    let argc = *(stack_top as *const isize);
    let argv = stack_top.add(size_of::<*const isize>()) as *const *const u8;
    let envp = argv.add(argc as usize + 1) as *const *const u8;

    let args = mkslice(argv, argc as usize);

    let ld_config = process_env(envp);

    let mut buf = [0u8; 4096];
    let mut buf = PrintBuffer::new(&mut buf[..]);

    if ld_config.exe.is_none() {
        eprint!(buf, "Cannot execute binary {}: No NIX_LD environment variable set", exe_name(args));
    }

    if let Some(lib_path) = ld_config.lib_path {
        eprint!(buf, "ld_library_path: {}\n", lib_path);
    }

    syscall::exit(0);
}
