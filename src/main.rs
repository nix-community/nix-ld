#![no_std]
#![no_main]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(lang_items)]
#![feature(link_args)]
#![feature(str_internals)]

#[allow(unused_attributes)]
#[link_args = "-nostartfiles -static"]
extern "C" {}

mod errno;
mod exit;
mod fd;
mod print;
mod start;
mod string;
mod syscall;
mod unwind_resume;

use core::fmt::Write;
use core::mem::{self, size_of};
use core::slice::from_raw_parts as mkslice;
use core::str;
use exit::exit;
use libc::{c_int, c_void};

use crate::print::{PrintBuffer, PrintableBytes};
pub use crate::start::_start;

#[lang = "eh_personality"]
fn eh_personality() {}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let mut buf = [0u8; 4096];
    let mut buf = PrintBuffer::new(&mut buf[..]);
    print!(buf, "panicked with {}", info);
    exit(1);
}

const NIX_LD: &'static [u8] = b"NIX_LD=";
const NIX_LD_LIB_PATH: &'static [u8] = b"NIX_LD_LIBRARY_PATH=";

struct LdConfig {
    exe: Option<&'static [u8]>,
    lib_path: Option<&'static [u8]>,
}

unsafe fn slice_from_cstr(s: *const u8) -> &'static [u8] {
    let mut count = 0;
    let mut sp = s;
    while *sp != b'\0' {
        count += 1;
        sp = sp.add(1);
    }
    mkslice(s, count)
}

fn process_env(env: &[*const u8]) -> LdConfig {
    let mut config = LdConfig {
        exe: None,
        lib_path: None,
    };
    for varp in env.iter() {
        let var = unsafe { slice_from_cstr(*varp) };
        if var.starts_with(NIX_LD) {
            config.exe = Some(&var[NIX_LD.len()..]);
        };
        if var.starts_with(NIX_LD_LIB_PATH) {
            config.lib_path = Some(&var[NIX_LD_LIB_PATH.len()..]);
        };
    }
    config
}

fn exe_name(args: &[*const u8]) -> PrintableBytes {
    PrintableBytes {
        data: if args.len() > 0 {
            unsafe { slice_from_cstr(args[0]) }
        } else {
            b""
        },
    }
}

#[cfg(target_pointer_width = "32")]
type ElfHeader = libc::Elf32_Phdr;

#[cfg(target_pointer_width = "64")]
type ElfHeader = libc::Elf64_Phdr;

fn load_elf(buf: &mut PrintBuffer, args: &[*const u8], ld_exe: &[u8]) -> Result<(), ()> {
    let fd = match fd::open(ld_exe, libc::O_RDONLY) {
        Ok(fd) => fd,
        Err(num) => {
            eprint!(
                buf,
                "cannot execute {}: cannot open link loader {}: {} ({})",
                exe_name(args),
                PrintableBytes { data: ld_exe },
                errno::strerror(num),
                num
            );
            return Err(());
        }
    };

    let mut header = unsafe { mem::zeroed() };
    fd.read(
        (&mut header as *mut ElfHeader) as *mut c_void,
        size_of::<ElfHeader>(),
    );
    Ok(())
}

unsafe fn get_args_and_env(stack_top: *const u8) -> (&'static [*const u8], &'static [*const u8]) {
    let argc = *(stack_top as *const c_int) as usize;
    let argv = stack_top.add(size_of::<*const c_int>()) as *const *const u8;
    let env_start = argv.add(argc + 1) as *const *const u8;
    let mut envp = env_start;
    let mut envc: usize = 0;
    while !(*envp).is_null() {
        envp = envp.add(1);
        envc += 1;
    }
    let args = mkslice(argv, argc);
    let env = mkslice(env_start, envc);
    (args, env)
}

#[no_mangle]
pub fn main(stack_top: *const u8) {
    let (args, env) = unsafe { get_args_and_env(stack_top) };
    let ld_config = process_env(env);

    let mut buf = [0u8; 4096];
    let mut buf = PrintBuffer::new(&mut buf[..]);

    let ld_exe = match ld_config.exe {
        None => {
            eprint!(
                buf,
                "Cannot execute binary {}: No NIX_LD environment variable set",
                exe_name(args)
            );
            exit(1);
        }
        Some(s) => s,
    };

    if let Some(lib_path) = ld_config.lib_path {
        eprint!(
            buf,
            "ld_library_path: {}\n",
            PrintableBytes { data: lib_path }
        );
    }

    if load_elf(&mut buf, args, ld_exe).is_err() {
        exit(1);
    }

    exit(0);
}
