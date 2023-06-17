//! Args wrangling.

use core::ffi::{c_void, CStr};
use core::mem;
use core::ptr;
use core::slice;

use heapless::Vec as ArrayVec;

use crate::arch::STACK_ALIGNMENT;
use crate::auxv::AuxVec;
use crate::nolibc::new_slice_leak;

trait CStrExt {
    fn parse_env(&self) -> Option<(&[u8], &[u8])>;
}

pub struct Args {
    /// Whether we have already iterated the envp.
    env_iterated: bool,

    argc: isize,
    argv: *const *const u8,
    envp: *const *const u8,
    auxv: AuxVec,

    // The number of original environment variables.
    envc: usize,

    new_environment: ArrayVec<&'static [u8], 10>,
}

pub struct EnvIter<'args> {
    args: &'args mut Args,
    index: usize,
    ended: bool,
}

/// A handle to manipulate an environment variable.
///
/// Invariant: If `Args::iter_env()` is called only once, at most one
/// `VarHandle` is present for each slot.
///
/// Any methods that mutate the view will consume the `VarHandle`.
#[derive(Debug)]
pub struct VarHandle {
    ptr: *const *const u8,
    name: &'static [u8],
    value_c: &'static [u8],

    /// Length of the buffer, including the final NUL.
    buf_len: usize,
}

pub struct WiggledStack {
    pub sp: *const c_void,
    pub argv: *const *const u8,
    pub envp: *const *const u8,
}

struct Wiggler<'a> {
    arg_slice: &'a mut [usize],
    orig_idx: usize,
    idx: usize,
    idx_argv: Option<usize>,
    idx_envp: Option<usize>,
}

impl CStrExt for CStr {
    fn parse_env(&self) -> Option<(&[u8], &[u8])> {
        let bytes = self.to_bytes_with_nul();
        let equal = bytes.iter().position(|b| '=' == *b as char)?;

        let name = &bytes[..equal];
        let value = &bytes[equal + 1..];

        Some((name, value))
    }
}

impl Args {
    pub unsafe fn new(argc: isize, argv: *const *const u8, envp: *const *const u8) -> Self {
        let (envc, auxv) = count_env(envp);

        Self {
            env_iterated: false,
            argc,
            argv,
            envp,
            envc,
            auxv: AuxVec::from_raw(auxv),
            new_environment: ArrayVec::new(),
        }
    }

    pub fn auxv(&self) -> &AuxVec {
        &self.auxv
    }
    pub fn auxv_mut(&mut self) -> &mut AuxVec {
        &mut self.auxv
    }

    /// Perform a handoff to the actual ld.so.
    ///
    /// The function must not return.
    pub fn handoff<F>(&self, f: F) -> !
    where
        F: FnOnce(WiggledStack),
    {
        if self.new_environment.is_empty() {
            // No need to mess with the stack
            log::info!("No new environment variables added - Not wiggling the stack");
            let sp = unsafe { self.argv.sub(1) };
            f(WiggledStack {
                sp: sp.cast(),
                argv: self.argv,
                envp: self.envp,
            });
        } else {
            // Wiggle the stack to add the new environment
            //
            // <https://fasterthanli.me/content/series/making-our-own-executable-packer/part-12/assets/elf-stack.43b0caae88ae7ef5.svg>
            //
            // Since we are operating on an entirely separate stack, we can
            // simply treat the old stack as a slice of usizes and shift
            // stuff around in it with `copy_within`.
            log::info!("Wiggling the stack to accomodate new environment variables");

            let auxvc = self.auxv.count().unwrap();
            let apparent_len = 1 + self.argc as usize + 1 + self.envc + 1 + auxvc * 2 + 2;
            let additional_len = self.new_environment.len();
            let desired_len = apparent_len + additional_len;

            // Does the stack actually look like what we expect?
            let end_of_storage: *const usize =
                unsafe { self.auxv.as_ptr().unwrap().add(2 * auxvc + 2) };
            let start_of_storage: *const usize = unsafe { self.argv.sub(1).cast() };
            let actual_len = unsafe { end_of_storage.offset_from(start_of_storage) };
            if actual_len < 0 || actual_len as usize != apparent_len {
                panic!(
                    "Stack layout different from expected ({} vs {})",
                    actual_len, apparent_len
                );
            }

            // Now apply stack alignment
            let new_start_of_storage_unpad =
                unsafe { start_of_storage.sub(additional_len).cast_mut() };
            let new_start_of_storage =
                (new_start_of_storage_unpad as usize & !(STACK_ALIGNMENT - 1)) as *mut usize;
            let padding_len = {
                let pad_end = new_start_of_storage_unpad as *const u8;
                let pad_start = new_start_of_storage as *const u8;
                let bytes = unsafe { pad_end.offset_from(pad_start) };
                if bytes < 0 || bytes as usize % mem::size_of::<*const usize>() != 0 {
                    panic!("Padding not a multiple of pointers: {}", bytes);
                }

                bytes as usize / mem::size_of::<*const usize>()
            };

            // We want it to look like this:
            //
            // [argc][argv][0][envp][newenv][0][auxv][last entry][padding]
            let mut wiggler = Wiggler {
                arg_slice: unsafe {
                    slice::from_raw_parts_mut(new_start_of_storage, desired_len + padding_len)
                },
                orig_idx: additional_len + padding_len,
                idx: 0,
                idx_argv: None,
                idx_envp: None,
            };

            wiggler.copy(1); // [argc]
            wiggler.mark_argv();
            wiggler.copy(self.argc as usize + 1); // [argv][0]
            wiggler.mark_envp();
            wiggler.copy(self.envc); // [envp]
            for env in self.new_environment.iter() {
                wiggler.push(env.as_ptr() as usize);
            }
            wiggler.copy(1 + auxvc * 2 + 2); // [0][auxv][last entry]

            f(wiggler.finalize());
        }

        // Nothing might work at this point
        loop {}
    }

    /// Creates a new environment variable.
    pub fn add_env<F>(&mut self, name: &str, value_len: usize, f: F)
    where
        F: FnOnce(&mut [u8]),
    {
        let name_len = name.len();
        let whole_len = name_len + 1 + value_len;
        let new_buf = new_slice_leak(whole_len + 1).unwrap();
        new_buf[..name_len].copy_from_slice(name.as_bytes());
        new_buf[name_len] = '=' as u8;
        new_buf[whole_len] = 0;

        f(&mut new_buf[name_len + 1..whole_len]);

        self.new_environment.push(new_buf).unwrap();
    }

    pub fn iter_env(&mut self) -> Option<EnvIter> {
        if self.env_iterated {
            return None;
        }

        self.env_iterated = true;
        Some(EnvIter {
            args: self,
            ended: false,
            index: 0,
        })
    }
}

impl<'args> Iterator for EnvIter<'args> {
    type Item = VarHandle;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ended {
            return None;
        }

        let ptr = unsafe { self.args.envp.add(self.index) };
        let pptr = unsafe { *ptr };
        if pptr == ptr::null() {
            self.ended = true;
            return None;
        }

        let env = unsafe { core::ffi::CStr::from_ptr(pptr.cast()) };
        let buf_len = env.to_bytes_with_nul().len();
        if let Some((name, value_c)) = env.parse_env() {
            self.index += 1;
            Some(VarHandle {
                ptr,
                name,
                value_c,
                buf_len,
            })
        } else {
            // Bad environment
            self.ended = true;
            None
        }
    }
}

impl VarHandle {
    /// Returns the name as bytes.
    pub fn name(&self) -> &[u8] {
        self.name
    }

    /// Returns the value as bytes, without the trailing NUL.
    pub fn value(&self) -> &[u8] {
        &self.value_c[..self.value_c.len() - 1]
    }

    /// Returns the value as a NUL-terminated CStr.
    pub fn value_cstr(&self) -> &CStr {
        core::ffi::CStr::from_bytes_with_nul(self.value_c).unwrap()
    }

    /// Rename the variable without changing its value.
    pub fn rename(mut self, new_name: &str) {
        if new_name.len() <= self.name.len() {
            // use existing buffer
            let old_name_len = self.name.len();
            let new_name_len = new_name.len();
            let buf = self.get_buf_mut();
            buf[..new_name_len].copy_from_slice(new_name.as_bytes());
            buf.copy_within(old_name_len.., new_name_len);
        } else {
            // use new buffer
            let value_len = self.value().len();
            let new_name_len = new_name.len();
            let whole_len = new_name_len + 1 + value_len;
            let new_buf = self.replace_buf(whole_len);
            new_buf[..new_name_len].copy_from_slice(new_name.as_bytes());
            new_buf[new_name_len] = '=' as u8;
            new_buf[new_name_len + 1..whole_len].copy_from_slice(self.value());
            new_buf[whole_len] = 0;
        }
    }

    /// Edits the value.
    ///
    /// The function will take the original value and must fill the
    /// entire new buffer.
    pub fn edit<F>(mut self, new_len: usize, f: F)
    where
        F: FnOnce(&[u8], &mut [u8]),
    {
        let name_len = self.name.len();
        let whole_len = name_len + 1 + new_len;
        let new_buf = self.replace_buf(whole_len);
        new_buf[..name_len].copy_from_slice(self.name);
        new_buf[name_len] = '=' as u8;
        new_buf[whole_len] = 0;
        f(self.value(), &mut new_buf[self.name.len() + 1..whole_len]);
    }

    fn get_buf_mut(self) -> &'static mut [u8] {
        // Safety: Existing view of the environent is consumed
        let pptr = unsafe { (*self.ptr).cast_mut() };
        unsafe { slice::from_raw_parts_mut(pptr, self.buf_len) }
    }

    fn replace_buf(&mut self, new_len: usize) -> &'static mut [u8] {
        // Safety: Existing view of the environent is not affected
        let new_buf = new_slice_leak(new_len + 1).unwrap();
        let mptr = self.ptr.cast_mut();
        unsafe {
            *mptr = new_buf.as_ptr();
        }

        new_buf
    }
}

impl<'a> Wiggler<'a> {
    #[inline(always)]
    fn copy(&mut self, count: usize) {
        self.arg_slice
            .copy_within(self.orig_idx..self.orig_idx + count, self.idx);
        for i in 0..count {
            log::trace!(
                "Copied [{}]: {:x}",
                self.idx + i,
                self.arg_slice[self.idx + i]
            );
        }
        self.orig_idx += count;
        self.idx += count;
    }

    #[inline(always)]
    fn mark_argv(&mut self) {
        self.idx_argv = Some(self.idx);
    }

    #[inline(always)]
    fn mark_envp(&mut self) {
        self.idx_envp = Some(self.idx);
    }

    #[inline(always)]
    fn push(&mut self, value: usize) {
        self.arg_slice[self.idx] = value;
        log::trace!(" Added [{}]: {:x}", self.idx, value);
        self.idx += 1;
    }

    #[inline(always)]
    fn finalize(self) -> WiggledStack {
        let idx_argv = self.idx_argv.expect("Must have argv");
        let idx_envp = self.idx_envp.expect("Must have envp");

        WiggledStack {
            sp: self.arg_slice.as_ptr().cast(),
            argv: (&self.arg_slice[idx_argv] as *const usize).cast(),
            envp: (&self.arg_slice[idx_envp] as *const usize).cast(),
        }
    }
}

unsafe fn count_env(envp: *const *const u8) -> (usize, *const usize) {
    let mut envc = 0;
    let mut cur = envp;
    while *cur != ptr::null() {
        cur = cur.add(1);
        envc += 1;
    }
    (envc, cur.add(1) as *const usize)
}
