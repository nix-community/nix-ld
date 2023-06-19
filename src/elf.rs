//! ELF wrangling.

use core::ffi::{c_void, CStr};
use core::fmt;
use core::mem;
use core::ptr;

use crate::arch::elf_jmp;
pub use crate::arch::elf_types;
use crate::arch::elf_types::{
    header::{Header, ET_DYN},
    program_header::{ProgramHeader, PF_R, PF_W, PF_X, PT_LOAD},
};
use crate::auxv::AuxVec;
use crate::nolibc::{
    self, ENOENT, MAP_ANONYMOUS, MAP_FAILED, MAP_FIXED, MAP_PRIVATE, O_RDONLY, PROT_EXEC,
    PROT_NONE, PROT_READ, PROT_WRITE,
};

pub struct ElfHandle {
    fd: i32,
    phs: ProgramHeaders,
    page_size: usize,
    entry_point_v: usize,
    eh_map: *mut c_void,
    eh_map_len: usize,
}

pub struct ElfMapping {
    load_bias: usize,
    entry_point: *const c_void,
}

pub struct ProgramHeaders {
    phdr: *const ProgramHeader,
    phent: usize,
    phnum: usize,
}

pub struct ProgramHeadersIter<'ph> {
    headers: &'ph ProgramHeaders,
    index: usize,
}

struct DisplayPFlags<'ph>(&'ph ProgramHeader);

struct LoadableSummary {
    total_mapping_size: usize,
    first_vaddr: usize,
}

trait ProgramHeaderExt {
    fn prot_flags(&self) -> i32;
}

impl ElfHandle {
    pub fn open(path: &CStr, page_size: usize) -> Result<Self, ()> {
        let mut fd = unsafe { nolibc::open(path.as_ptr(), O_RDONLY, 0) };

        if fd < 0 && nolibc::errno() == ENOENT {
            let path_bytes = path.to_bytes();
            if path_bytes.ends_with(&[b'\n']) {
                // ${stdenv.cc}/nix-support/dynamic-linker contains trailing newline
                fd = nolibc::alloc_temp(path_bytes.len(), |path_trunc| {
                    path_trunc.copy_from_slice(path_bytes);
                    path_trunc[path_bytes.len() - 1] = 0;
                    let path_trunc = CStr::from_bytes_with_nul(path_trunc).unwrap();
                    unsafe { nolibc::open(path_trunc.as_ptr(), O_RDONLY, 0) }
                });
            }
        }

        if fd < 0 {
            log::error!("Failed to open {:?} ({})", path, nolibc::errno());
            return Err(());
        }

        let mut header = Header::default();
        let read = unsafe {
            nolibc::read(
                fd,
                &mut header as *mut _ as *mut c_void,
                mem::size_of::<Header>(),
            )
        };
        if read < 0 {
            log::error!(
                "Failed to read ELF header of {:?} ({})",
                path,
                nolibc::errno()
            );
            return Err(());
        }

        if &header.e_ident[..4] != b"\x7fELF".as_slice() {
            log::error!("{:?} is not an ELF", path);
            return Err(());
        }

        if header.e_type != ET_DYN {
            log::error!("{:?} is not a dynamic library", path);
            return Err(());
        }

        let phsize = header.e_phentsize as usize * header.e_phnum as usize;
        if phsize == 0 || phsize > 65536 {
            log::error!("{:?} has incorrect program header size {}", path, phsize);
            return Err(());
        }

        let eh_map_len = mem::size_of::<Header>() + phsize;
        let eh_map =
            unsafe { nolibc::mmap(ptr::null_mut(), eh_map_len, PROT_READ, MAP_PRIVATE, fd, 0) };

        if eh_map == MAP_FAILED {
            log::error!("Couldn't map headers of {:?} ({})", path, nolibc::errno());
            return Err(());
        }

        let phdr = unsafe { eh_map.add(mem::size_of::<Header>()) };

        let phs = ProgramHeaders {
            phdr: phdr.cast(),
            phent: header.e_phentsize as usize,
            phnum: header.e_phnum as usize,
        };

        Ok(Self {
            fd,
            phs,
            page_size,
            entry_point_v: header.e_entry as usize,
            eh_map,
            eh_map_len,
        })
    }

    pub fn map(self) -> Result<ElfMapping, ()> {
        let summary = if let Some(summary) = self.phs.summarize_loadable() {
            summary
        } else {
            log::error!("No program headers found");
            return Err(());
        };

        // For now, we assume the loader is relocatable and let
        // the kernel decide the load addr.
        let load_addr = unsafe {
            nolibc::mmap(
                ptr::null_mut(), // TODO: Maybe the ELF isn't relocatable
                self.page_align(summary.total_mapping_size),
                PROT_NONE,
                MAP_PRIVATE | MAP_ANONYMOUS,
                -1,
                0,
            )
        };
        if load_addr == MAP_FAILED {
            log::error!("Failed to reserve");
            return Err(());
        }

        // The first section's code starts at
        //
        //     load_addr + page_offset(ph.p_vaddr)
        let load_bias = (load_addr as usize).wrapping_sub(self.page_start(summary.first_vaddr));
        let entry_point = (load_bias + self.entry_point_v) as *const c_void;

        log::debug!("   Total Size: 0x{:x}", summary.total_mapping_size);
        log::debug!("    Load Addr: {:x?}", load_addr);
        log::debug!("  First Vaddr: 0x{:x?}", summary.first_vaddr);
        log::debug!("    Load Bias: 0x{:x?}", load_bias);
        log::debug!("  Entry Point: 0x{:x?}", entry_point);
        log::debug!("    Page Size: {}", self.page_size);

        log::debug!(
            "GDB: add-symbol-file /path/to/ld.so.symbols 0x{:x}",
            load_bias
        );

        for ph in self.phs.iter() {
            if ph.p_type != PT_LOAD || ph.p_memsz == 0 {
                continue;
            }

            let memsz = ph.p_memsz as usize;
            let filesz = ph.p_filesz as usize;
            let vaddr = ph.p_vaddr as usize;
            let vend = vaddr + memsz;
            let fend = vaddr + filesz;
            let offset = ph.p_offset as usize;

            let prot = ph.prot_flags();

            let total_map_size = self.page_align(vend) - self.page_start(vaddr);
            let file_map_size =
                self.page_align(core::cmp::min(fend, vend)) - self.page_start(vaddr);

            // Assumption:
            //
            //     page_offset(ph.p_vaddr) == page_offset(ph.p_offset)
            //
            // We do the following mmap for the file-backed portion:
            let mapping = unsafe {
                let addr = self.page_start(load_bias + vaddr);
                let offset = self.page_start(offset);
                let size = file_map_size;

                log::trace!(
                    "mmap [{ph}] [0x{addr:x}-0x{mend:x}] (vaddr=0x{vaddr:x}, offset=0x{offset:x})",
                    mend = addr + size,
                    ph = DisplayPFlags(ph),
                );

                nolibc::mmap(
                    addr as *mut c_void,
                    size,
                    prot,
                    MAP_PRIVATE | MAP_FIXED,
                    self.fd,
                    offset.try_into().unwrap(),
                )
            };

            if mapping == MAP_FAILED {
                log::error!("Failed to map segment 0x{:x} ({})", vaddr, nolibc::errno());
                return Err(());
            }

            // Memory beyond memsz is zero-initialized
            if memsz > filesz && (ph.p_flags & PF_W != 0) {
                if file_map_size < total_map_size {
                    let mapping = unsafe {
                        let addr = load_addr.add(file_map_size);
                        let size = total_map_size - file_map_size;
                        nolibc::mmap(
                            addr,
                            size,
                            prot,
                            MAP_PRIVATE | MAP_FIXED | MAP_ANONYMOUS,
                            -1,
                            0,
                        )
                    };

                    if mapping == MAP_FAILED {
                        log::error!("Failed to map anonymous portion for segment 0x{:x}", vaddr);
                        return Err(());
                    }
                }

                // Zeroing to (load_bias + vaddr + memsz) isn't enough - Need to zero the
                // entire page
                unsafe {
                    let zero_addr = (load_bias + vaddr + filesz) as *mut c_void;
                    let zero_end = self.page_align(load_bias + vaddr + memsz);
                    let zero_size = zero_end - zero_addr as usize;
                    nolibc::memset(zero_addr, 0, zero_size);
                }
            }
        }

        Ok(ElfMapping {
            load_bias,
            entry_point,
        })
    }

    #[inline(always)]
    fn page_align(&self, v: usize) -> usize {
        (v + self.page_size - 1) & !(self.page_size - 1)
    }

    #[inline(always)]
    fn page_start(&self, v: usize) -> usize {
        v & !(self.page_size - 1)
    }
}

impl Drop for ElfHandle {
    fn drop(&mut self) {
        unsafe {
            nolibc::munmap(self.eh_map, self.eh_map_len);
        }
    }
}

impl ElfMapping {
    pub fn load_bias(&self) -> usize {
        self.load_bias
    }

    /// Jumps to the entry point with a stack.
    pub unsafe fn jump_with_sp(self, sp: *const c_void) -> ! {
        elf_jmp!(sp, self.entry_point);
    }
}

impl ProgramHeaders {
    /// Loads the interpreter's own program headers.
    pub fn from_auxv(auxv: &AuxVec) -> Option<Self> {
        let at_base = auxv.at_base.as_ref()?.value();

        if at_base.is_null() {
            // We are executed directly
            if let (Some(phdr), Some(phent), Some(phnum)) =
                (&auxv.at_phdr, &auxv.at_phent, &auxv.at_phnum)
            {
                if phdr.value().is_null() {
                    return None;
                }
                Some(Self {
                    phdr: phdr.value(),
                    phent: phent.value(),
                    phnum: phnum.value(),
                })
            } else {
                None
            }
        } else {
            // We are the loader
            let ehdr: *const Header = at_base.cast();
            let header: &Header = unsafe { &*ehdr };

            if &header.e_ident[..4] != b"\x7fELF".as_slice() {
                return None;
            }

            if header.e_type != ET_DYN {
                return None;
            }

            let phsize = header.e_phentsize as usize * header.e_phnum as usize;
            if phsize == 0 || phsize > 65536 {
                return None;
            }

            let phdr = unsafe { ehdr.add(1).cast() };

            Some(Self {
                phdr,
                phent: header.e_phentsize as usize,
                phnum: header.e_phnum as usize,
            })
        }
    }

    pub fn iter(&self) -> ProgramHeadersIter {
        ProgramHeadersIter {
            headers: self,
            index: 0,
        }
    }

    fn summarize_loadable(&self) -> Option<LoadableSummary> {
        let mut first_vaddr = None;
        let mut addr_min = usize::MAX;
        let mut addr_max = usize::MIN;

        for ph in self.iter() {
            if ph.p_type != PT_LOAD || ph.p_memsz == 0 {
                continue;
            }

            if first_vaddr.is_none() {
                first_vaddr = Some(ph.p_vaddr as usize);
            }

            if addr_min > ph.p_vaddr as usize {
                addr_min = ph.p_vaddr as usize;
            }

            let vend = ph.p_vaddr as usize + ph.p_memsz as usize;
            if addr_max < vend {
                addr_max = vend;
            }
        }

        first_vaddr.map(|first_vaddr| LoadableSummary {
            first_vaddr,
            total_mapping_size: addr_max - addr_min,
        })
    }
}

// TODO: Just make a slice out of them, no need for impl Iterator
impl<'ph> Iterator for ProgramHeadersIter<'ph> {
    type Item = &'ph ProgramHeader;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.headers.phnum {
            return None;
        }

        let base: *const u8 = self.headers.phdr.cast();
        let entry_p = unsafe { base.add(self.index * self.headers.phent).cast() };
        let entry = unsafe { &*entry_p };

        self.index += 1;

        Some(entry)
    }
}

impl ProgramHeaderExt for ProgramHeader {
    #[inline(always)]
    fn prot_flags(&self) -> i32 {
        let p_flags = &self.p_flags;
        (if p_flags & PF_R != 0 { PROT_READ } else { 0 })
            | (if p_flags & PF_W != 0 { PROT_WRITE } else { 0 })
            | (if p_flags & PF_X != 0 { PROT_EXEC } else { 0 })
    }
}

impl<'ph> fmt::Display for DisplayPFlags<'ph> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let p_flags = &self.0.p_flags;
        let mut write_prot = |mask, s| {
            if p_flags & mask != 0 {
                write!(f, "{}", s)
            } else {
                write!(f, " ")
            }
        };
        write_prot(PF_R, "R")?;
        write_prot(PF_W, "W")?;
        write_prot(PF_X, "X")?;
        Ok(())
    }
}
