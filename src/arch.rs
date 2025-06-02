//! Arch-specific stuff.

use core::ffi::c_void;
use core::mem;
use core::ptr;

use crate::args::EnvEdit;
use crate::const_concat::concat;

#[cfg(not(target_os = "linux"))]
compiler_error!("Only Linux is supported");

#[cfg(target_pointer_width = "32")]
pub use goblin::elf32 as elf_types;
#[cfg(target_pointer_width = "64")]
pub use goblin::elf64 as elf_types;

// Typically 16 is required
pub const STACK_ALIGNMENT: usize = 32;

pub const EM_SELF: u16 = {
    use elf_types::header::*;
    #[cfg(target_arch = "x86_64")]
    const VALUE: u16 = EM_X86_64;
    #[cfg(target_arch = "x86")]
    const VALUE: u16 = EM_386;
    #[cfg(target_arch = "aarch64")]
    const VALUE: u16 = EM_AARCH64;
    VALUE
};

pub const R_RELATIVE: u32 = {
    use elf_types::reloc::*;
    #[cfg(target_arch = "x86_64")]
    const VALUE: u32 = R_X86_64_RELATIVE;
    #[cfg(target_arch = "x86")]
    const VALUE: u32 = R_386_RELATIVE;
    #[cfg(target_arch = "aarch64")]
    const VALUE: u32 = R_AARCH64_RELATIVE;
    VALUE
};

pub const NIX_SYSTEM: &str = match option_env!("NIX_SYSTEM") {
    Some(system) => system,
    None => {
        #[cfg(target_arch = "x86_64")]
        const VALUE: &str = "x86_64_linux";
        #[cfg(target_arch = "x86")]
        const VALUE: &str = "i686_linux";
        #[cfg(target_arch = "aarch64")]
        const VALUE: &str = "aarch64_linux";
        VALUE
    }
};

pub const NIX_LD_SYSTEM_ENV: &str = concat!("NIX_LD_", NIX_SYSTEM);
pub const NIX_LD_LIBRARY_PATH_SYSTEM_ENV: &str = concat!("NIX_LD_LIBRARY_PATH_", NIX_SYSTEM);
pub const NIX_LD_SYSTEM_ENV_BYTES: &[u8] = NIX_LD_SYSTEM_ENV.as_bytes();
pub const NIX_LD_LIBRARY_PATH_SYSTEM_ENV_BYTES: &[u8] = NIX_LD_LIBRARY_PATH_SYSTEM_ENV.as_bytes();

// Note: We separate main_relocate_stack and elf_jmp to make stack alignment
// easier. For elf_jmp, we expect the loader to take care of aligning the
// stack pointer in _start.

macro_rules! main_relocate_stack {
    ($sp:ident, $func:ident) => {
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("mov rsp, {}; call {}", in(reg) $sp, sym $func, options(noreturn));
        #[cfg(target_arch = "x86")]
        core::arch::asm!("mov esp, {}; call {}", in(reg) $sp, sym $func, options(noreturn));
        #[cfg(target_arch = "aarch64")]
        core::arch::asm!("mov sp, {}; bl {}", in(reg) $sp, sym $func, options(noreturn));
    };
}
pub(crate) use main_relocate_stack;

macro_rules! elf_jmp {
    ($sp:ident, $target:expr) => {
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("mov rsp, {}; jmp {}", in(reg) $sp, in(reg) $target, options(noreturn));
        #[cfg(target_arch = "x86")]
        core::arch::asm!("mov esp, {}; jmp {}", in(reg) $sp, in(reg) $target, options(noreturn));
        #[cfg(target_arch = "aarch64")]
        core::arch::asm!("mov sp, {}; br {}", in(reg) $sp, in(reg) $target, options(noreturn));
    };
}
pub(crate) use elf_jmp;

/// Context for the entry point trampoline.
///
/// The goal is to revert our LD_LIBRARY_PATH changes once
/// ld.so has done its job.
#[repr(C, align(4096))]
#[derive(Debug)]
pub struct TrampolineContext {
    elf_entry: *const c_void,
    env_entry: *const *const u8,
    env_string: *const u8,
}

impl TrampolineContext {
    #[allow(unused)]
    const ENV_ENTRY_OFFSET: usize = mem::size_of::<*const u8>();

    #[allow(unused)]
    const ENV_STRING_OFFSET: usize = mem::size_of::<*const u8>() * 2;

    pub fn set_elf_entry(&mut self, entry: *const c_void) {
        self.elf_entry = entry;
    }

    pub fn revert_env(&mut self, edit: &EnvEdit) {
        self.env_entry = edit.entry;
        self.env_string = edit.old_string;
    }

    pub fn revert_env_entry(&mut self, entry: *const *const u8) {
        self.env_entry = entry;
    }
}

pub static mut TRAMPOLINE_CONTEXT: TrampolineContext = TrampolineContext {
    elf_entry: ptr::null(),
    env_entry: ptr::null(),
    env_string: ptr::null(),
};

#[cfg(not(feature = "entry_trampoline"))]
pub const ENTRY_TRAMPOLINE: Option<unsafe extern "C" fn() -> !> = None;

#[cfg(target_arch = "x86_64")]
pub const ENTRY_TRAMPOLINE: Option<unsafe extern "C" fn() -> !> = Some(entry_trampoline);

#[cfg(target_arch = "x86_64")]
#[naked]
unsafe extern "C" fn entry_trampoline() -> ! {
    unsafe {
        core::arch::naked_asm!(
            "lea r10, [rip + {context}]",
            "mov r11, [r10 + {size} * 1]", // .env_entry
            "test r11, r11",
            "jz 2f",
            "mov r10, [r10 + {size} * 2]", // .env_string
            "mov [r11], r10",
            "2:",
            "jmp [rip + {context}]",
            context = sym TRAMPOLINE_CONTEXT,
            size = const core::mem::size_of::<*const u8>(),
        )
    }
}

#[cfg(target_arch = "aarch64")]
pub const ENTRY_TRAMPOLINE: Option<unsafe extern "C" fn() -> !> = Some(entry_trampoline);

#[cfg(target_arch = "aarch64")]
#[naked]
unsafe extern "C" fn entry_trampoline() -> ! {
    unsafe {
        core::arch::naked_asm!(
            "adrp x8, {context}",
            "ldr x9, [x8, {env_entry_off}]", // .env_entry
            "cbz x9, 2f",
            "ldr x10, [x8, {env_string_off}]", // .env_string
            "str x10, [x9]",
            "2:",
            "ldr x8, [x8]",
            "br x8",
            context = sym TRAMPOLINE_CONTEXT,
            env_entry_off = const TrampolineContext::ENV_ENTRY_OFFSET,
            env_string_off = const TrampolineContext::ENV_STRING_OFFSET,
        )
    }
}

// !!!!
// After adding a trampoline, remember to enable test_ld_path_restore for
// the target_arch in tests/tests.rs as well
// !!!!
#[cfg(all(
    feature = "entry_trampoline",
    not(target_arch = "x86_64"),
    not(target_arch = "aarch64")
))]
pub const ENTRY_TRAMPOLINE: Option<unsafe extern "C" fn() -> !> = None;
