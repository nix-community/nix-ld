//! Args wrangling.

use core::ffi::{c_void, CStr};
use core::mem;
use core::slice;

use crate::arch::STACK_ALIGNMENT;
use crate::auxv::AuxVec;
use crate::support::explode;
use crate::sys::new_slice_leak;

trait CStrExt {
    fn parse_env(&self) -> Option<(&[u8], &[u8])>;
}

pub struct Args {
    /// Whether we have already iterated the envp.
    env_iterated: bool,

    argc: usize,
    argv: *const *const u8,
    envp: *const *const u8,
    auxv: AuxVec,

    // The number of original environment variables.
    envc: usize,

    extra_env: Option<*const u8>,
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
}

/// The context to call _start with.
#[derive(Debug)]
pub struct StartContext {
    pub sp: *const c_void,
    pub argv: *const *const u8,
    pub envp: *const *const u8,
    pub extra_env: Option<*const *const u8>,
}

pub struct EnvEdit {
    pub entry: *const *const u8,
    pub old_string: *const u8,
}

struct StackShifter<'a> {
    arg_slice: &'a mut [usize],
    orig_idx: usize,
    idx: usize,
    idx_argv: Option<usize>,
    idx_envp: Option<usize>,
    idx_extra_env: Option<usize>,
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
    pub unsafe fn new(argc: usize, argv: *const *const u8, envp: *const *const u8) -> Self {
        let (envc, auxv) = count_env(envp);

        Self {
            env_iterated: false,
            argc,
            argv,
            envp,
            envc,
            auxv: AuxVec::from_raw(auxv),
            extra_env: None,
        }
    }

    pub fn auxv(&self) -> &AuxVec {
        &self.auxv
    }
    pub fn auxv_mut(&mut self) -> &mut AuxVec {
        &mut self.auxv
    }

    pub fn argc(&self) -> usize {
        self.argc
    }

    /// Perform a handoff to the actual ld.so.
    ///
    /// The function must not return.
    pub fn handoff<F>(&self, f: F) -> !
    where
        F: FnOnce(StartContext),
    {
        if let Some(extra_env) = self.extra_env {
            // Wiggle the stack to add the new environment
            //
            // <https://fasterthanli.me/content/series/making-our-own-executable-packer/part-12/assets/elf-stack.43b0caae88ae7ef5.svg>
            //
            // Since we are operating on an entirely separate stack, we can
            // simply treat the old stack as a slice of usizes and shift
            // stuff around in it with `copy_within`.
            //
            // TODO: Too overengineered - Simplify
            log::info!("Shifting the stack to accomodate new environment variable");

            let auxvc = self.auxv.count().unwrap();
            let apparent_len = 1 + self.argc + 1 + self.envc + 1 + auxvc * 2 + 2;
            let additional_len = 1;
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

            let arg_slice = unsafe {
                slice::from_raw_parts_mut(new_start_of_storage, desired_len + padding_len)
            };
            let mut shifter = StackShifter::new(arg_slice, additional_len + padding_len);

            // We want it to look like this:
            //
            // [argc][argv][0][envp][newenv][0][auxv][last element][padding]
            shifter.copy(1); // [argc]
            shifter.mark_argv();
            shifter.copy(self.argc + 1); // [argv][0]
            shifter.mark_envp();
            shifter.copy(self.envc); // [envp]
            shifter.mark_extra_env();
            shifter.push(extra_env as usize);
            shifter.copy(1 + auxvc * 2 + 2); // [0][auxv][last element]

            f(shifter.finalize());
        } else {
            // No need to mess with the stack
            log::info!("No new environment variable added - Not shifting the stack");
            let sp = unsafe { self.argv.sub(1) };
            f(StartContext {
                sp: sp.cast(),
                argv: self.argv,
                envp: self.envp,
                extra_env: None,
            });
        }

        // Nothing might work at this point
        explode("The handoff function returned");
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
        if pptr.is_null() {
            self.ended = true;
            return None;
        }

        let env = unsafe { core::ffi::CStr::from_ptr(pptr.cast()) };
        if let Some((name, value_c)) = env.parse_env() {
            self.index += 1;
            Some(VarHandle { ptr, name, value_c })
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
    pub fn rename(self, new_name: &str) -> EnvEdit {
        let value_len = self.value().len();
        self.edit(Some(new_name), value_len, |old, new| {
            new.copy_from_slice(old);
        })
    }

    /// Edits the value.
    ///
    /// The function will take the original value and must fill the
    /// entire new buffer.
    pub fn edit<F>(mut self, name: Option<&str>, value_len: usize, f: F) -> EnvEdit
    where
        F: FnOnce(&[u8], &mut [u8]),
    {
        let name = name.map(|s| s.as_bytes()).unwrap_or(self.name);
        let name_len = name.len();
        let whole_len = name_len + 1 + value_len;
        let (old_buf, new_buf) = self.replace_buf(whole_len);
        new_buf[..name_len].copy_from_slice(name);
        new_buf[name_len] = b'=';
        new_buf[whole_len] = 0;
        f(self.value(), &mut new_buf[name_len + 1..whole_len]);

        log::debug!(
            "Edited env element {:?} ({:?} -> {:?})",
            self.ptr,
            old_buf,
            new_buf.as_ptr()
        );

        EnvEdit {
            entry: self.ptr,
            old_string: old_buf,
        }
    }

    fn replace_buf(&mut self, new_len: usize) -> (*const u8, &'static mut [u8]) {
        // Safety: Existing view of the environent is not affected
        let new_buf = new_slice_leak(new_len + 1).unwrap();
        let mptr = self.ptr.cast_mut();
        unsafe {
            let old_buf = *mptr;
            *mptr = new_buf.as_ptr();
            (old_buf, new_buf)
        }
    }
}

impl<'a> StackShifter<'a> {
    fn new(arg_slice: &'a mut [usize], orig_idx: usize) -> Self {
        Self {
            arg_slice,
            orig_idx,
            idx: 0,
            idx_argv: None,
            idx_envp: None,
            idx_extra_env: None,
        }
    }

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
    fn mark_extra_env(&mut self) {
        self.idx_extra_env = Some(self.idx);
    }

    #[inline(always)]
    fn push(&mut self, value: usize) {
        self.arg_slice[self.idx] = value;
        log::trace!(" Added [{}]: {:x}", self.idx, value);
        self.idx += 1;
    }

    #[inline(always)]
    fn finalize(self) -> StartContext {
        let idx_argv = self.idx_argv.expect("Must have argv");
        let idx_envp = self.idx_envp.expect("Must have envp");
        let extra_env = self
            .idx_extra_env
            .map(|idx| (&self.arg_slice[idx] as *const usize).cast());

        StartContext {
            sp: self.arg_slice.as_ptr().cast(),
            argv: (&self.arg_slice[idx_argv] as *const usize).cast(),
            envp: (&self.arg_slice[idx_envp] as *const usize).cast(),
            extra_env,
        }
    }
}

unsafe fn count_env(envp: *const *const u8) -> (usize, *const usize) {
    let mut envc = 0;
    let mut cur = envp;
    while !(*cur).is_null() {
        cur = cur.add(1);
        envc += 1;
    }
    (envc, cur.add(1) as *const usize)
}
