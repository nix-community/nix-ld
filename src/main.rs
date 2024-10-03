#![feature(naked_functions)]
#![feature(asm_const)]
#![feature(lang_items)]
#![no_std]
#![no_main]
#![allow(internal_features)]

mod arch;
mod args;
mod auxv;
mod elf;
mod fixup;
mod support;
mod sys;

use core::ffi::{c_void, CStr};
use core::mem::MaybeUninit;
use core::ptr;

use constcat::concat_slices;

use arch::{
    NIX_LD_LIBRARY_PATH_SYSTEM_ENV, NIX_LD_LIBRARY_PATH_SYSTEM_ENV_BYTES, NIX_LD_SYSTEM_ENV,
    NIX_LD_SYSTEM_ENV_BYTES,
};
use args::{Args, EnvEdit, VarHandle};
use default_env::default_env;
use support::StackSpace;

static mut ARGS: MaybeUninit<Args> = MaybeUninit::uninit();
static mut STACK: MaybeUninit<StackSpace> = MaybeUninit::uninit();

const DEFAULT_NIX_LD: &CStr = unsafe {
    CStr::from_bytes_with_nul_unchecked(concat_slices!([u8]:
        default_env!("DEFAULT_NIX_LD", "/run/current-system/sw/share/nix-ld/lib/ld.so").as_bytes(),
        b"\0"
    ))
};

const DEFAULT_NIX_LD_LIBRARY_PATH: &[u8] = b"/run/current-system/sw/share/nix-ld/lib";
const EMPTY_LD_LIBRARY_PATH_ENV: &CStr =
    unsafe { CStr::from_bytes_with_nul_unchecked(b"LD_LIBRARY_PATH=\0") };

#[derive(Default)]
struct Context {
    nix_ld: Option<VarHandle>,
    nix_ld_library_path: Option<VarHandle>,
    ld_library_path: Option<VarHandle>,
}

#[no_mangle]
unsafe extern "C" fn main(argc: usize, argv: *const *const u8, envp: *const *const u8) -> ! {
    fixup::fixup_relocs(envp);

    ARGS.write(Args::new(argc, argv, envp));
    let stack = STACK.assume_init_mut().bottom();
    arch::main_relocate_stack!(stack, real_main);
}

#[no_mangle]
extern "C" fn real_main() -> ! {
    let args = unsafe { ARGS.assume_init_mut() };
    let mut ctx = Context::default();

    log::set_logger(&support::LOGGER)
        .map(|_| log::set_max_level(log::LevelFilter::Warn))
        .unwrap();

    for env in args.iter_env().unwrap() {
        match env.name() {
            b"NIX_LD_LOG" => {
                if let Ok(log_level) = env.value_cstr().to_str() {
                    if let Ok(level) = log_level.parse::<log::LevelFilter>() {
                        log::set_max_level(level);
                    } else {
                        log::warn!("Unknown log level {}", log_level);
                    }
                }
            }

            // The system-specific variants (e.g., NIX_LD_x86_64_linux) always
            // take precedence. Currently, NIX_LD_LIBRARY_PATH_{system} clobbers
            // the generic one, and we should revisit this decision (maybe
            // concatenate?).
            NIX_LD_SYSTEM_ENV_BYTES => {
                ctx.nix_ld = Some(env);
            }
            b"NIX_LD" => {
                ctx.nix_ld.get_or_insert(env);
            }
            NIX_LD_LIBRARY_PATH_SYSTEM_ENV_BYTES => {
                ctx.nix_ld_library_path = Some(env);
            }
            b"NIX_LD_LIBRARY_PATH" => {
                ctx.nix_ld_library_path.get_or_insert(env);
            }
            b"LD_LIBRARY_PATH" => {
                ctx.ld_library_path = Some(env);
            }
            _ => {}
        }
    }

    // Deal with NIX_LD
    let nix_ld = match &mut ctx.nix_ld {
        None => {
            log::info!("NIX_LD is not set - Falling back to default");
            DEFAULT_NIX_LD
        }
        Some(nix_ld) if nix_ld.value().is_empty() => {
            log::info!("NIX_LD is empty - Falling back to default");
            DEFAULT_NIX_LD
        }
        Some(nix_ld) => {
            let cstr = nix_ld.value_cstr();
            log::info!("NIX_LD is set to {:?}", cstr);
            cstr
        }
    };

    // Deal with {NIX_,}LD_LIBRARY_PATH
    let env_edit = if let Some(ld_library_path) = ctx.ld_library_path {
        // Concatenate:
        //
        // Basically LD_LIBRARY_PATH=$LD_LIBRARY_PATH:$NIX_LD_LIBRARY_PATH
        let head = ld_library_path.value();
        let tail = if let Some(nix_ld_library_path) = &ctx.nix_ld_library_path {
            log::info!("Appending NIX_LD_LIBRARY_PATH to LD_LIBRARY_PATH");
            nix_ld_library_path.value()
        } else {
            log::info!("Appending default NIX_LD_LIBRARY_PATH to LD_LIBRARY_PATH");
            DEFAULT_NIX_LD_LIBRARY_PATH
        };

        let sep: &[u8] = if head.is_empty() || *head.last().unwrap() == b':' {
            &[]
        } else {
            &[b':']
        };
        let new_len = head.len() + tail.len() + sep.len();

        ld_library_path.edit(None, new_len, |head, new| {
            new[..head.len()].copy_from_slice(head);
            new[head.len()..head.len() + sep.len()].copy_from_slice(sep);
            new[head.len() + sep.len()..].copy_from_slice(tail);
        })
    } else if let Some(nix_ld_library_path) = ctx.nix_ld_library_path.take() {
        log::info!("Renaming NIX_LD_LIBRARY_PATH to LD_LIBRARY_PATH");

        // NIX_LD_LIBRARY_PATH must always exist for impure child processes to work
        nix_ld_library_path.rename("LD_LIBRARY_PATH")
    } else {
        log::info!("Neither LD_LIBRARY_PATH or NIX_LD_LIBRARY_PATH exist - Setting default");

        args
            .add_env(
                "LD_LIBRARY_PATH",
                DEFAULT_NIX_LD_LIBRARY_PATH.len(),
                |buf| {
                    buf.copy_from_slice(DEFAULT_NIX_LD_LIBRARY_PATH);
                },
            )
            .unwrap();

        // If the entry trampoline is available on the platform, LD_LIBRARY_PATH
        // will be replaced with an empty LD_LIBRARY_PATH when ld.so launches
        // the actual program.
        //
        // We cannot replace it with NIX_LD_LIBRARY_PATH as it would take
        // precedence over config files.
        EnvEdit {
            entry: ptr::null(),
            old_string: EMPTY_LD_LIBRARY_PATH_ENV.as_ptr().cast(),
        }
    };

    let pagesz = args
        .auxv()
        .at_pagesz
        .as_ref()
        .expect("AT_PAGESZ must exist")
        .value();

    log::info!("Loading {:?}", nix_ld);
    let loader = elf::ElfHandle::open(nix_ld, pagesz).unwrap();
    let loader_map = loader.map().unwrap();

    let mut at_base = args.auxv_mut().at_base.as_mut().and_then(|base| {
        if base.value().is_null() {
            None
        } else {
            Some(base)
        }
    });

    match at_base {
        None => {
            // We were executed directly - execve the actual loader
            if args.argc() <= 1 {
                log::warn!("Environment honored by nix-ld:");
                log::warn!("- NIX_LD, {}", NIX_LD_SYSTEM_ENV);
                log::warn!("- NIX_LD_LIBRARY_PATH, {}", NIX_LD_LIBRARY_PATH_SYSTEM_ENV);
                log::warn!("- NIX_LD_LOG (error, warn, info, debug, trace)");
                log::warn!("Default ld.so: {:?}", DEFAULT_NIX_LD);
            }

            args.handoff(|start| unsafe {
                log::debug!("Start context: {:#?}", start);
                sys::execve(nix_ld.as_ptr(), start.argv, start.envp);
                sys::abort();
            });
        }
        Some(ref mut at_base) => {
            // We are the loader - Set the AT_BASE to the actual loader
            at_base.set(loader_map.load_bias() as *const c_void);
        }
    }

    // We want our LD_LIBRARY_PATH to only affect the loaded binary
    // and not propagate to child processes. To achieve this, we
    // replace the entry point with a trampoline that reverts our
    // LD_LIBRARY_PATH edit and jumps to the real entry point.
    if let Some(trampoline) = arch::ENTRY_TRAMPOLINE {
        log::info!("Using entry trampoline");
        if let Some(ref mut at_entry) = args.auxv_mut().at_entry {
            unsafe {
                arch::TRAMPOLINE_CONTEXT.set_elf_entry(at_entry.value());
                arch::TRAMPOLINE_CONTEXT.revert_env(&env_edit);
            }
            at_entry.set(trampoline as *const _);
        } else {
            log::warn!("No AT_ENTRY found");
        }
    }

    args.handoff(|start| unsafe {
        log::debug!("Start context: {:#?}", start);

        if arch::ENTRY_TRAMPOLINE.is_some() {
            if let Some(extra_env) = start.extra_env {
                arch::TRAMPOLINE_CONTEXT.revert_env_entry(extra_env);
            }
            log::debug!("Trampoline context: {:#?}", arch::TRAMPOLINE_CONTEXT);
        }

        log::info!("Transferring control to ld.so");
        loader_map.jump_with_sp(start.sp);
    });
}
