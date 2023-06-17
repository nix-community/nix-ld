#![feature(core_intrinsics)]
#![no_std]
#![no_main]

extern crate compiler_builtins;

mod arch;
mod args;
mod auxv;
mod elf;
mod fixup;
mod nolibc;
mod support;

use core::arch::asm;
use core::ffi::{c_void, CStr};
use core::mem::MaybeUninit;

use arch::NIX_LD_SYSTEM_ENV;
use args::{Args, VarHandle};
use support::StackSpace;

static mut ARGS: MaybeUninit<Args> = MaybeUninit::uninit();
static mut STACK: MaybeUninit<StackSpace> = MaybeUninit::uninit();

const DEFAULT_NIX_LD: &CStr = unsafe {
    CStr::from_bytes_with_nul_unchecked(b"/run/current-system/sw/share/nix-ld/lib/ld.so\0")
};
const DEFAULT_NIX_LD_LIBRARY_PATH: &[u8] = b"/run/current-system/sw/share/nix-ld/lib";

#[derive(Default)]
struct Context {
    nix_ld: Option<VarHandle>,
    nix_ld_library_path: Option<VarHandle>,
    ld_library_path: Option<VarHandle>,
}

#[no_mangle]
extern "C" fn main(argc: isize, argv: *const *const u8, envp: *const *const u8) -> ! {
    unsafe {
        let args = ARGS.write(Args::new(argc, argv, envp));
        fixup::fixup_relocs(args.auxv());

        let stack = STACK.assume_init_mut().bottom();
        arch::main_relocate_stack!(stack, real_main);
    }
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
            b"NIX_LD" | NIX_LD_SYSTEM_ENV => {
                ctx.nix_ld = Some(env);
            }
            b"NIX_LD_LIBRARY_PATH" => {
                ctx.nix_ld_library_path = Some(env);
            }
            b"LD_LIBRARY_PATH" => {
                ctx.ld_library_path = Some(env);
            }
            _ => {}
        }
    }

    let pagesz = args
        .auxv()
        .at_pagesz
        .as_ref()
        .expect("AT_PAGESZ must exist")
        .value();

    // Deal with {NIX_,}LD_LIBRARY_PATH
    if let Some(ld_library_path) = ctx.ld_library_path {
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

        let sep: &[u8] = if head.is_empty() || *head.last().unwrap() == ':' as u8 {
            &[]
        } else {
            &[':' as u8]
        };
        let new_len = head.len() + tail.len() + sep.len();

        ld_library_path.edit(new_len, |head, new| {
            new[..head.len()].copy_from_slice(head);
            new[head.len()..head.len() + sep.len()].copy_from_slice(sep);
            new[head.len() + sep.len()..].copy_from_slice(tail);
        });
    } else if let Some(nix_ld_library_path) = ctx.nix_ld_library_path.take() {
        log::info!("Renaming NIX_LD_LIBRARY_PATH to LD_LIBRARY_PATH");
        nix_ld_library_path.rename("LD_LIBRARY_PATH");
    } else {
        log::info!("Neither LD_LIBRARY_PATH or NIX_LD_LIBRARY_PATH exist - Setting default");
        args.add_env(
            "LD_LIBRARY_PATH",
            DEFAULT_NIX_LD_LIBRARY_PATH.len(),
            |buf| {
                buf.copy_from_slice(DEFAULT_NIX_LD_LIBRARY_PATH);
            },
        );
    }

    // Deal with NIX_LD
    let default_bytes = DEFAULT_NIX_LD.to_bytes();
    let nix_ld = match &mut ctx.nix_ld {
        None => {
            // Not set at all, let's add it
            log::info!("NIX_LD is not set - Falling back to default");
            args.add_env("NIX_LD", default_bytes.len(), |buf| {
                buf.copy_from_slice(default_bytes);
            });
            DEFAULT_NIX_LD
        }
        Some(nix_ld) if nix_ld.value().is_empty() => {
            log::info!("NIX_LD is empty - Falling back to default");
            ctx.nix_ld
                .take()
                .unwrap()
                .edit(default_bytes.len(), |_, buf| {
                    buf.copy_from_slice(default_bytes);
                });
            DEFAULT_NIX_LD
        }
        Some(nix_ld) => {
            let cstr = nix_ld.value_cstr();
            log::info!("NIX_LD is set to {:?}", cstr);
            cstr
        }
    };

    log::debug!("Going to open loader {:?}", nix_ld);
    let loader = elf::ElfHandle::open(nix_ld, pagesz).unwrap();
    let loader_map = loader.map().unwrap();

    if let Some(ref mut at_base) = args.auxv_mut().at_base {
        if at_base.value().is_null() {
            // We were executed directly - execve the actual loader
            log::warn!("We were executed directly - execve-ing");
            log::warn!("Hi there, thanks for testing! However, executing nix-ld directly isn't how it's normally used and much of the core functionality isn't exercised this way.");
            log::warn!("Instead, try patchelf-ing any executable to use nix-ld-rs as the interperter.");

            args.handoff(|stack| unsafe {
                nolibc::execve(
                    nix_ld.as_ptr(),
                    stack.argv,
                    stack.envp,
                );
                nolibc::abort();
            });
        } else {
            // We are the loader - Set the AT_BASE to the actual loader
            at_base.set(loader_map.load_bias() as *const c_void);
        }
    }

    args.handoff(|stack| unsafe {
        loader_map.jmp(stack.sp);
    });
}
