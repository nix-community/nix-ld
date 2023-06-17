//! Arch-specific stuff.

// Typically 16 is required
pub const STACK_ALIGNMENT: usize = 32;

pub const R_RELATIVE: u32 = {
    use crate::elf::elf_types::reloc::*;
    cfg_match::cfg_match! {
        target_arch = "x86_64" => R_X86_64_RELATIVE,
        target_arch = "x86" => R_386_RELATIVE,
        target_arch = "aarch64" => R_AARCH64_RELATIVE,
    }
};

// TODO: Make this configurable via env!()
pub const NIX_LD_SYSTEM_ENV: &[u8] = cfg_match::cfg_match! {
    target_arch = "x86_64" => b"NIX_LD_x86_64-linux",
    target_arch = "x86" => b"NIX_LD_i686-linux",
    target_arch = "aarch64" => b"NIX_LD_aarch64-linux",
};

// Note: We separate main_relocate_stack and elf_jmp to make stack alignment
// easier. For elf_jmp, we expect the loader to take care of aligning the
// stack pointer in _start.

macro_rules! main_relocate_stack {
    ($sp:ident, $func:ident) => {
        cfg_match::cfg_match! {
            target_arch = "x86_64" => {
                asm!("mov rsp, {}; call {}", in(reg) $sp, sym $func, options(noreturn));
            }
            target_arch = "x86" => {
                asm!("mov esp, {}; call {}", in(reg) $sp, sym $func, options(noreturn));
            }
            target_arch = "aarch64" => {
                asm!("mov sp, {}; bl {}", in(reg) $sp, sym $func, options(noreturn));
            }
        }
    };
}
pub(crate) use main_relocate_stack;

macro_rules! elf_jmp {
    ($sp:ident, $target:expr) => {
        cfg_match::cfg_match! {
            target_arch = "x86_64" => {
                asm!("mov rsp, {}; jmp {}", in(reg) $sp, in(reg) $target, options(noreturn));
            }
            target_arch = "x86" => {
                asm!("mov esp, {}; jmp {}", in(reg) $sp, in(reg) $target, options(noreturn));
            }
            target_arch = "aarch64" => {
                asm!("mov sp, {}; br {}", in(reg) $sp, in(reg) $target, options(noreturn));
            }
        }
    };
}
pub(crate) use elf_jmp;
