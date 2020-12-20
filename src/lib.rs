#![no_std]

const PF_X: u32 = 1 << 0;
const PF_W: u32 = 1 << 1;
const PF_R: u32 = 1 << 2;

mod errno;
mod exit;
mod fd;
mod print;
mod string;
mod syscalls;
mod unwind_resume;
mod lossy;
mod breakpoint;

use core::mem::{self, size_of};
use core::ptr;
use core::slice::from_raw_parts as mkslice;
use core::usize;
use exit::exit;
use libc::{c_int, c_uint, c_void, off_t, PROT_EXEC, PROT_READ, PROT_WRITE};

use lossy::Utf8Lossy;

const ET_EXEC: u16 = 2;
const ET_DYN: u16 = 3;
const PT_LOAD: u32 = 1;


#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    print!("panicked with {}\n", info);
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

fn exe_name(args: &[*const u8]) -> &Utf8Lossy {
    Utf8Lossy::from_bytes(if args.len() > 0 {
        unsafe { slice_from_cstr(args[0]) }
    } else {
        b""
    })
}

#[cfg(target_pointer_width = "32")]
mod types {
    pub type ElfHeader = libc::Elf32_Ehdr;
    pub type ElfProgramHeader = libc::Elf32_Phdr;
    pub type IntPtr = u32;
    pub const INT_PTR_MAX: u32 = u32::MAX;
}

#[cfg(target_pointer_width = "64")]
mod types {
    pub type ElfHeader = libc::Elf64_Ehdr;
    pub type ElfProgramHeader = libc::Elf64_Phdr;
    pub type IntPtr = u64;
    pub const INT_PTR_MAX: u64 = u64::MAX;
}

use crate::types::*;

const ELF_MAGIC: &'static [u8] = b"\xb1ELF";


extern "C" {
    fn jmp_ld(stack_top: IntPtr, addr: IntPtr) -> !;
}

const PAGE_SIZE: IntPtr = 4096; // FIXME actual page size here

fn prot_flags(p_flags: c_uint) -> c_int {
    (if p_flags & PF_R != 0 { PROT_READ } else { 0 })
        | (if p_flags & PF_W != 0 { PROT_WRITE } else { 0 })
        | (if p_flags & PF_X != 0 { PROT_EXEC } else { 0 })
}

fn total_mapping_size(prog_headers: &[ElfProgramHeader]) -> IntPtr {
    let mut addr_min = INT_PTR_MAX;
    let mut addr_max = 0;
    for ph in prog_headers {
        if ph.p_type != PT_LOAD || ph.p_memsz == 0 {
            continue;
        }
        if ph.p_vaddr < addr_min {
            addr_min = ph.p_vaddr;
        }
        if ph.p_vaddr + ph.p_memsz > addr_max {
            addr_max = ph.p_vaddr + ph.p_memsz;
        }
    }
    addr_max - addr_min
}

fn elf_page_start(v: IntPtr) -> IntPtr {
    v & !(PAGE_SIZE - 1)
}

fn elf_page_offset(v: IntPtr) -> IntPtr {
    v & (PAGE_SIZE - 1)
}

fn elf_page_align(v: IntPtr) -> IntPtr {
    (v + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)
}

fn map_elf<'a>(
    exe_name: &Utf8Lossy,
    ld_exe: &Utf8Lossy,
    fd: &fd::Fd,
    prog_headers: &[ElfProgramHeader],
) -> Result<(IntPtr, fd::Mmap<'a>), ()> {
    let total_size = total_mapping_size(prog_headers);

    if total_size == 0 {
        eprint!(
            "cannot execute {}: no program headers found in {}",
            exe_name, ld_exe
        );
        return Err(());
    }

    let mut load_addr: IntPtr = 0;
    let mut total_mapping: Option<fd::Mmap> = None;
    for ph in prog_headers {
        // zero sized segments are valid but we won't mmap them
        if ph.p_type != PT_LOAD || ph.p_filesz == 0 {
            continue;
        }
        let prot = prot_flags(ph.p_flags);
        let addr = if load_addr == 0 {
            0
        } else {
            elf_page_start(load_addr + ph.p_vaddr)
        };
        let size = if load_addr == 0 {
            // mmap the whole library range to reserve the area,
            // later smaller parts will be mmaped over it.
            elf_page_align(total_size)
        } else {
            elf_page_align(ph.p_filesz - elf_page_offset(ph.p_vaddr))
        };
        let off_start = ph.p_offset - elf_page_offset(ph.p_vaddr);
        let flags = if load_addr == 0 {
            libc::MAP_PRIVATE
        } else {
            libc::MAP_PRIVATE | libc::MAP_FIXED
        };
        let res = fd.mmap(
            addr as *const c_void,
            size as usize,
            prot,
            flags,
            off_start as off_t,
        );
        let mapping = match res {
            Ok(mapping) => mapping,
            Err(num) => {
                eprint!(
                    "cannot execute {}: mmap segment of {} failed: {} ({})\n",
                    exe_name,
                    ld_exe,
                    errno::strerror(num),
                    num
                );
                return Err(());
            }
        };
        //eprint!("mmap {:x} ({:x}) at {:x} ({:x}) (vaddr: {:x}, load_addr: {:x}, prot: ",
        //        size,
        //        ph.p_filesz,
        //        mapping.data.as_ptr() as usize,
        //        addr,
        //        ph.p_vaddr,
        //        load_addr);
        //eprint!("{}{}{}",
        //        (if ph.p_flags & PF_R != 0 { "r" } else { "-" }),
        //        (if ph.p_flags & PF_W != 0 { "w" } else { "-" }),
        //        (if ph.p_flags & PF_X != 0 { "x" } else { "-" }));
        //eprint!(")\n");

        if load_addr == 0 {
            load_addr = mapping.data.as_ptr() as IntPtr - ph.p_vaddr;
            total_mapping = Some(mapping);
        } else {
            // We can leak smaller allocations because total_mapping covers it
            unsafe { mapping.into_raw() };
        }
    }

    Ok((load_addr, total_mapping.unwrap()))
}

fn load_elf<'a>(exe_name: &Utf8Lossy, ld_exe: &[u8]) -> Result<(IntPtr, fd::Mmap<'a>), ()> {
    let fd = match fd::open(ld_exe, libc::O_RDONLY) {
        Ok(fd) => fd,
        Err(num) => {
            eprint!(
                "cannot execute {}: cannot open link loader {}: {} ({})",
                exe_name,
                Utf8Lossy::from_bytes(ld_exe),
                errno::strerror(num),
                num
            );
            return Err(());
        }
    };

    let mut header: ElfHeader = unsafe { mem::zeroed() };
    fd.read(
        (&mut header as *mut ElfHeader) as *mut c_void,
        size_of::<ElfHeader>(),
    );
    if header.e_ident[..ELF_MAGIC.len()] == *ELF_MAGIC {
        eprint!(
            "cannot execute {}: link loader has invalid elf magic\n",
            exe_name
        );
        return Err(());
    }
    // TODO also support dynamic excutable
    //if header.e_type != ET_EXEC && header.e_type != ET_DYN {
    if header.e_type != ET_DYN {
        eprint!(
            "cannot execute {}: link loader is not an dynamic library\n",
            exe_name
        );
        return Err(());
    }

    // XXX check if e_machine of elf interpreter matches the one in our binary
    // XXX binfmt_elf also check if elf is an fdpic

    let ph_size = size_of::<ElfProgramHeader>() * (header.e_phnum as usize);
    // XXX binfmt_elf also checks ELF_MIN_ALIGN here
    if ph_size == 0 || ph_size > 65536 {
        eprint!(
            "cannot execute {}: link loader has program header size: {}\n",
            exe_name, ph_size
        );
        return Err(());
    }

    let res = fd.mmap(
        ptr::null(),
        size_of::<ElfHeader>() + ph_size,
        libc::PROT_READ,
        libc::MAP_PRIVATE,
        0,
    );
    let headers_mapping = match res {
        Err(num) => {
            eprint!(
                "cannot execute {}: cannot mmap link loader headers: {} ({})\n",
                exe_name,
                errno::strerror(num),
                num
            );
            return Err(());
        }
        Ok(mapping) => mapping
    };
    // FIXME careful! prog_headers does borrow ownership from headers_start
    // meaning that if headers_start goes out of scope than memory is unmmaped
    let headers_start = &headers_mapping.data[size_of::<ElfHeader>()..];
    let headers_p = headers_start.as_ptr() as *const ElfProgramHeader;
    let prog_headers = unsafe { mkslice(headers_p, header.e_phnum as usize) };

    let elf = map_elf(exe_name, Utf8Lossy::from_bytes(ld_exe), &fd, prog_headers);
    elf.map(|(load_addr, mapping)| {
        let entry = load_addr + header.e_entry as IntPtr;
        (entry, mapping)
    })
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

    let ld_exe = match ld_config.exe {
        None => {
            eprint!(
                "Cannot execute binary {}: No NIX_LD environment variable set\n",
                exe_name(args)
            );
            exit(1);
        }
        Some(s) => s,
    };

    if let Some(lib_path) = ld_config.lib_path {
        eprint!("ld_library_path: {}\n", Utf8Lossy::from_bytes(lib_path));
    }
    match load_elf(exe_name(args), ld_exe) {
        Err(()) => { exit(1); }
        Ok((entry_point, _mapping)) => {
            unsafe { jmp_ld(stack_top as IntPtr, entry_point) };
        }
    };
}
