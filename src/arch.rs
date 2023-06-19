//! Arch-specific stuff.

use core::ffi::c_void;
use core::ptr;

use constcat::concat;

use crate::args::EnvEdit;

#[cfg(not(target_os = "linux"))]
compiler_error!("Only Linux is supported");

cfg_match::cfg_match! {
    target_pointer_width = "64" => {
        pub use goblin::elf64 as elf_types;
    }
    target_pointer_width = "32" => {
        pub use goblin::elf32 as elf_types;
    }
}

// Typically 16 is required
pub const STACK_ALIGNMENT: usize = 32;

pub const R_RELATIVE: u32 = {
    use elf_types::reloc::*;
    cfg_match::cfg_match! {
        target_arch = "x86_64" => R_X86_64_RELATIVE,
        target_arch = "x86" => R_386_RELATIVE,
        target_arch = "aarch64" => R_AARCH64_RELATIVE,
    }
};

pub const NIX_SYSTEM: &str = match option_env!("NIX_SYSTEM") {
    Some(system) => system,
    None => cfg_match::cfg_match! {
        target_arch = "x86_64" => "x86_64-linux",
        target_arch = "x86" => "i686-linux",
        target_arch = "aarch64" => "aarch64-linux",
    },
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
        cfg_match::cfg_match! {
            target_arch = "x86_64" => {
                core::arch::asm!("mov rsp, {}; call {}", in(reg) $sp, sym $func, options(noreturn));
            }
            target_arch = "x86" => {
                core::arch::asm!("mov esp, {}; call {}", in(reg) $sp, sym $func, options(noreturn));
            }
            target_arch = "aarch64" => {
                core::arch::asm!("mov sp, {}; bl {}", in(reg) $sp, sym $func, options(noreturn));
            }
        }
    };
}
pub(crate) use main_relocate_stack;

macro_rules! elf_jmp {
    ($sp:ident, $target:expr) => {
        cfg_match::cfg_match! {
            target_arch = "x86_64" => {
                core::arch::asm!("mov rsp, {}; jmp {}", in(reg) $sp, in(reg) $target, options(noreturn));
            }
            target_arch = "x86" => {
                core::arch::asm!("mov esp, {}; jmp {}", in(reg) $sp, in(reg) $target, options(noreturn));
            }
            target_arch = "aarch64" => {
                core::arch::asm!("mov sp, {}; br {}", in(reg) $sp, in(reg) $target, options(noreturn));
            }
        }
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
    entry: *const c_void,
    env_rewrite: *const *const u8,
    env_to: *const u8,
}

impl TrampolineContext {
    pub fn entry(&mut self, entry: *const c_void) {
        self.entry = entry;
    }

    pub fn revert_env(&mut self, edit: &EnvEdit) {
        self.env_rewrite = edit.entry;
        self.env_to = edit.old_env;
    }

    pub fn revert_env_entry(&mut self, entry: *const *const u8) {
        self.env_rewrite = entry;
    }
}

pub static mut TRAMPOLINE_CONTEXT: TrampolineContext = TrampolineContext {
    entry: ptr::null(),
    env_rewrite: ptr::null(),
    env_to: ptr::null(),
};

cfg_match::cfg_match! {
    not(feature = "entry_trampoline") => {
        pub const ENTRY_TRAMPOLINE: Option<unsafe extern "C" fn() -> !> = None;
    }
    target_arch = "x86_64" => {
        pub const ENTRY_TRAMPOLINE: Option<unsafe extern "C" fn() -> !> = Some(entry_trampoline);

        #[naked]
        unsafe extern "C" fn entry_trampoline() -> ! {
            core::arch::asm!(
                "lea r10, [rip + {context}]",
                "mov r11, [r10 + {size} * 1]", // .env_rewrite
                "test r11, r11",
                "jz 1f",
                "mov r10, [r10 + {size} * 2]", // .env_to
                "mov [r11], r10",
                "1:",
                "jmp [rip + {context}]",
                context = sym TRAMPOLINE_CONTEXT,
                size = const core::mem::size_of::<*const u8>(),
                options(noreturn),
            )
        }
    }
    target_arch = "aarch64" => {
        pub const ENTRY_TRAMPOLINE: Option<unsafe extern "C" fn() -> !> = Some(entry_trampoline);

        #[naked]
        unsafe extern "C" fn entry_trampoline() -> ! {
            core::arch::asm!(
                "adrp x8, {context}",
                "ldr x9, [x8, {env_rewrite_off}]", // .env_rewrite
                "cbz x9, 1f",
                "ldr x10, [x8, {env_to_off}]", // .env_to
                "str x10, [x9]",
                "1:",
                "ldr x8, [x8]",
                "br x8",
                context = sym TRAMPOLINE_CONTEXT,
                env_rewrite_off = const core::mem::size_of::<*const u8>(),
                env_to_off = const core::mem::size_of::<*const u8>() * 2,
                options(noreturn),
            )
        }
    }
    // !!!!
    // After adding a trampoline, remember to enable test_ld_path_restore for
    // the target_arch in tests/tests.rs as well
    // !!!!
    _ => {
        pub const ENTRY_TRAMPOLINE: Option<unsafe extern "C" fn() -> !> = None;
    }
}
