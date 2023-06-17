//! Relocation fixups.

use core::mem;
use core::slice;

use crate::arch::R_RELATIVE;
use crate::auxv::AuxVec;
use crate::elf::{
    elf_types,
    elf_types::dynamic::{DT_NULL, DT_REL, DT_RELA, DT_RELAENT, DT_RELASZ},
    elf_types::program_header::PT_DYNAMIC,
    ProgramHeaders,
};
use crate::support::explode;

extern "C" {
    fn __executable_start();
}

fn debug_trap<T>(val: T) -> T {
    //unsafe { core::arch::asm!("int3") };
    core::intrinsics::black_box(val)
}

struct Dyn(*const elf_types::dynamic::Dyn);

impl Dyn {
    fn from_program_headers(phs: &ProgramHeaders, load_offset: usize) -> Option<Self> {
        phs.iter()
            .find(|ph| debug_trap(ph).p_type == PT_DYNAMIC)
            .map(|ph| {
                let dynamic_p =
                    load_offset.wrapping_add(ph.p_vaddr as usize) as *const elf_types::dynamic::Dyn;
                Dyn(dynamic_p)
            })
    }

    fn find_rela(&self, load_offset: usize) -> Option<&[elf_types::reloc::Rela]> {
        let mut cur = self.0;

        let mut rela_data = None;
        let mut rela_len = None;

        loop {
            let entry = unsafe { &*cur };
            match entry.d_tag as u64 {
                DT_NULL => break,
                DT_RELA => {
                    rela_data =
                        Some(entry.d_val.wrapping_add(load_offset as _)
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
                DT_REL => {
                    explode("DT_REL is not supported");
                }
                _ => {}
            }

            cur = unsafe { cur.add(1) };
        }

        if let (Some(rela_data), Some(rela_len)) = (rela_data, rela_len) {
            let slice = unsafe { slice::from_raw_parts(rela_data, rela_len) };
            Some(slice)
        } else {
            None
        }
    }
}

pub unsafe fn fixup_relocs(auxv: &AuxVec) {
    // Reference: <https://gist.github.com/Amanieu/588e3f9d330019c5d39f3ce60e8e0aae>
    let load_offset = __executable_start as *const u8 as usize;

    let phs = if let Some(phs) = ProgramHeaders::from_auxv(&auxv) {
        phs
    } else {
        explode("Couldn't load our own headers - ld-dispatch should be used an the interpreter");
    };

    let dynamic = if let Some(dynamic) = Dyn::from_program_headers(&phs, load_offset) {
        dynamic
    } else {
        explode("No dynamic???");
        //return;
    };

    let rela = if let Some(rela) = dynamic.find_rela(load_offset) {
        rela
    } else {
        explode("DT_RELA not found");
    };

    for reloc in rela {
        let r_type = elf_types::reloc::r_type(reloc.r_info);
        if r_type != R_RELATIVE {
            explode("Unsupported relocation type");
        }

        let ptr = load_offset.wrapping_add(reloc.r_offset as usize) as *mut usize;
        *ptr = load_offset.wrapping_add(reloc.r_addend as usize);
    }
}
