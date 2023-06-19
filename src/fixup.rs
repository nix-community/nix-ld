//! Relocation fixups.
//!
//! panic!()/unwrap() cannot be used here.

use core::mem;
use core::ptr;
use core::slice;

use crate::arch::R_RELATIVE;
use crate::auxv::AuxVec;
use crate::elf::{
    elf_types,
    elf_types::dynamic::{DT_NULL, DT_REL, DT_RELA, DT_RELAENT, DT_RELASZ, DT_RELENT, DT_RELSZ},
    elf_types::header::Header,
    elf_types::program_header::PT_DYNAMIC,
    ProgramHeaders,
};
use crate::support::explode;

extern "C" {
    fn __executable_start();
}

struct Dynamic {
    ptr: *const elf_types::dynamic::Dyn,
    load_offset: usize,
}

impl Dynamic {
    fn from_program_headers(phs: &ProgramHeaders, load_offset: usize) -> Option<Self> {
        phs.iter().find(|ph| ph.p_type == PT_DYNAMIC).map(|ph| {
            let ptr =
                load_offset.wrapping_add(ph.p_vaddr as usize) as *const elf_types::dynamic::Dyn;
            Self { ptr, load_offset }
        })
    }

    fn fixup(&self) {
        let mut cur = self.ptr;

        let mut rela_data = None;
        let mut rela_len = None;

        let mut rel_data = None;
        let mut rel_len = None;

        loop {
            let entry = unsafe { &*cur };

            #[allow(clippy::unnecessary_cast)] // it's necessary with ELF32
            match entry.d_tag as u64 {
                DT_NULL => break,

                // DT_RELA
                DT_RELA => {
                    rela_data = Some(entry.d_val.wrapping_add(self.load_offset as _)
                        as *const elf_types::reloc::Rela);
                }
                DT_RELASZ => {
                    rela_len =
                        Some(entry.d_val as usize / mem::size_of::<elf_types::reloc::Rela>());
                }
                DT_RELAENT => {
                    let actual_size = entry.d_val as usize;
                    if actual_size != mem::size_of::<elf_types::reloc::Rela>() {
                        explode("DT_RELAENT has unsupported size");
                    }
                }

                // DT_REL
                DT_REL => {
                    rel_data = Some(entry.d_val.wrapping_add(self.load_offset as _)
                        as *const elf_types::reloc::Rel);
                }
                DT_RELSZ => {
                    rel_len = Some(entry.d_val as usize / mem::size_of::<elf_types::reloc::Rel>());
                }
                DT_RELENT => {
                    let actual_size = entry.d_val as usize;
                    if actual_size != mem::size_of::<elf_types::reloc::Rel>() {
                        explode("DT_RELENT has unsupported size");
                    }
                }

                _ => {}
            }

            cur = unsafe { cur.add(1) };
        }

        if let (Some(rela_data), Some(rela_len)) = (rela_data, rela_len) {
            let rela = unsafe { slice::from_raw_parts(rela_data, rela_len) };
            for reloc in rela {
                let r_type = elf_types::reloc::r_type(reloc.r_info);
                if r_type != R_RELATIVE {
                    explode("Unsupported relocation type");
                }

                let ptr = self.load_offset.wrapping_add(reloc.r_offset as usize) as *mut usize;
                unsafe {
                    *ptr = self.load_offset.wrapping_add(reloc.r_addend as usize);
                }
            }
        }

        if let (Some(rel_data), Some(rel_len)) = (rel_data, rel_len) {
            let rel = unsafe { slice::from_raw_parts(rel_data, rel_len) };
            for reloc in rel {
                let r_type = elf_types::reloc::r_type(reloc.r_info);
                if r_type != R_RELATIVE {
                    explode("Unsupported relocation type");
                }

                let ptr = self.load_offset.wrapping_add(reloc.r_offset as usize) as *mut usize;
                unsafe {
                    let addend = *ptr;
                    let relocated = self.load_offset.wrapping_add(addend);
                    *ptr = relocated;
                }
            }
        }
    }
}

pub unsafe fn fixup_relocs(envp: *const *const u8) {
    // Reference: <https://gist.github.com/Amanieu/588e3f9d330019c5d39f3ce60e8e0aae>
    let auxv = find_auxv(envp);
    let auxv = AuxVec::from_raw(auxv);

    let load_offset = {
        let at_base = auxv.at_base.as_ref().map_or_else(ptr::null, |v| v.value());
        let at_phdr = auxv.at_phdr.as_ref().map_or_else(ptr::null, |v| v.value());
        let exec_start = __executable_start as *const usize;

        if !at_base.is_null() {
            at_base as usize
        } else if !at_phdr.is_null() {
            at_phdr as usize - mem::size_of::<Header>()
        } else {
            exec_start as usize
        }
    };

    let phs = if let Some(phs) = ProgramHeaders::from_auxv(&auxv) {
        phs
    } else {
        explode("Couldn't load our own headers - ld-dispatch should be used as the interpreter");
    };

    let dynamic = if let Some(dynamic) = Dynamic::from_program_headers(&phs, load_offset) {
        dynamic
    } else {
        explode("No dynamic section in own executable");
    };

    dynamic.fixup();
}

unsafe fn find_auxv(envp: *const *const u8) -> *const usize {
    let mut cur = envp;
    while !(*cur).is_null() {
        cur = cur.add(1);
    }
    cur.add(1) as *const usize
}
