//! auxv wrangling.

use core::ffi::c_void;
use core::marker::PhantomData;
use core::ops::Deref;

use crate::elf::elf_types::program_header::ProgramHeader;

pub const AT_PHDR: usize = 3;
pub const AT_PHENT: usize = 4;
pub const AT_PHNUM: usize = 5;
pub const AT_PAGESZ: usize = 6;
pub const AT_BASE: usize = 7;
pub const AT_ENTRY: usize = 9;

#[derive(Debug, Default)]
pub struct AuxVec {
    ptr: Option<*const usize>,
    auxvc: Option<usize>,
    pub at_base: Option<Entry<*const c_void>>,
    pub at_entry: Option<Entry<*const c_void>>,
    pub at_phdr: Option<Entry<*const ProgramHeader>>,
    pub at_phent: Option<Entry>,
    pub at_phnum: Option<Entry>,
    pub at_pagesz: Option<Entry>,
}

#[derive(Debug)]
pub struct Entry<T = usize> {
    ptr: *const usize,
    _phantom: PhantomData<T>,
}

pub struct BorrowedEntry<'a> {
    entry: Entry,
    _phantom: PhantomData<&'a ()>,
}

pub struct AuxVecIter<'a> {
    auxv: &'a AuxVec,
    index: usize,
}

impl AuxVec {
    pub unsafe fn from_raw(ptr: *const usize) -> Self {
        let mut auxv = Self {
            ptr: Some(ptr),
            ..Self::default()
        };

        let mut at_base = None;
        let mut at_entry = None;
        let mut at_phdr = None;
        let mut at_phent = None;
        let mut at_phnum = None;
        let mut at_pagesz = None;
        let mut auxvc = 0;

        for entry in auxv.iter() {
            match entry.key() {
                AT_BASE => at_base = Some(entry.steal()),
                AT_ENTRY => at_entry = Some(entry.steal()),
                AT_PHDR => at_phdr = Some(entry.steal()),
                AT_PHENT => at_phent = Some(entry.steal()),
                AT_PHNUM => at_phnum = Some(entry.steal()),
                AT_PAGESZ => at_pagesz = Some(entry.steal()),
                _ => {}
            }
            auxvc += 1;
        }

        auxv.at_base = at_base;
        auxv.at_entry = at_entry;
        auxv.at_phdr = at_phdr;
        auxv.at_phent = at_phent;
        auxv.at_phnum = at_phnum;
        auxv.at_pagesz = at_pagesz;
        auxv.auxvc = Some(auxvc);
        auxv
    }

    pub fn as_ptr(&self) -> Option<*const usize> {
        self.ptr
    }

    pub fn count(&self) -> Option<usize> {
        self.auxvc
    }

    pub fn iter(&self) -> AuxVecIter {
        AuxVecIter {
            auxv: self,
            index: 0,
        }
    }
}

impl<T> Entry<T> {
    pub fn key(&self) -> usize {
        unsafe { *self.ptr }
    }

    pub fn set(&mut self, value: T) {
        unsafe {
            let valp = self.value_ptr().cast_mut();
            *valp = value;
        }
    }

    fn value_ptr(&self) -> *const T {
        unsafe { self.ptr.add(1).cast() }
    }

    fn steal<U>(&self) -> Entry<U> {
        // AuxVec/AuxVecIter only gives out references so lifetime is enforced
        Entry {
            ptr: self.ptr.cast(),
            _phantom: PhantomData,
        }
    }
}

impl<T: Copy> Entry<T> {
    pub fn value(&self) -> T {
        unsafe { *self.value_ptr() }
    }
}

impl<'a> BorrowedEntry<'a> {
    fn new(entry: Entry) -> Self {
        Self {
            entry,
            _phantom: PhantomData,
        }
    }
}

impl<'a> Deref for BorrowedEntry<'a> {
    type Target = Entry;

    fn deref(&self) -> &Self::Target {
        &self.entry
    }
}

impl<'a> Iterator for AuxVecIter<'a> {
    type Item = BorrowedEntry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let auxvp = self.auxv.ptr?;
        let entryp = unsafe { auxvp.add(self.index * 2) };
        if unsafe { *entryp } == 0 {
            return None;
        }

        let entry = BorrowedEntry::new(Entry {
            ptr: entryp,
            _phantom: PhantomData,
        });

        self.index += 1;

        Some(entry)
    }
}
