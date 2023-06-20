use std::env::{self, remove_var, set_var};
use std::path::{Path, PathBuf};
use std::process::Command;

use lazy_static::lazy_static;
use rstest::*;
use tempfile::TempDir;

#[fixture]
#[once]
fn libtest() -> &'static str {
    eprintln!("Testing {} on {}", EXE, TARGET);

    eprintln!("Building libtest");
    compile_test_lib("test");
    TMPDIR.path().to_str().unwrap()
}

#[fixture]
#[once]
fn dt_needed_bin() -> PathBuf {
    compile_test_bin("dt-needed", &["test"])
}

/// Check that we can run a simple binary.
#[rstest]
fn test_hello() {
    let bin = compile_test_bin("hello", &[]);

    remove_var("LD_LIBRARY_PATH");
    remove_var("NIX_LD_LIBRARY_PATH");
    let (stdout, _) = must_succeed(&bin);
    assert!(stdout.contains("Hello, world!"));
}

/// Check that we can run a binary with DT_NEEDED dependencies.
#[rstest]
fn test_dt_needed(libtest: &str, dt_needed_bin: &Path) {
    // First make sure it doesn't run without the library
    {
        remove_var("LD_LIBRARY_PATH");
        remove_var("NIX_LD_LIBRARY_PATH");
        let (_, stderr) = must_fail(dt_needed_bin);
        assert!(stderr.contains("loading shared"));
    }

    // Now it should work
    {
        remove_var("LD_LIBRARY_PATH");
        set_var("NIX_LD_LIBRARY_PATH", libtest);
        let (stdout, _) = must_succeed(dt_needed_bin);
        assert!(stdout.contains("Hello from libtest"));
    }
}

/// Check that we can run a binary that does dlopen.
#[rstest]
fn test_dlopen(libtest: &str) {
    let bin = compile_test_bin("dlopen", &[]);
    eprintln!("test_dlopen: {}", libtest);

    // First make sure it doesn't run without the library
    {
        remove_var("LD_LIBRARY_PATH");
        remove_var("NIX_LD_LIBRARY_PATH");
        let (_, stderr) = must_fail(&bin);
        assert!(stderr.contains("Failed to dlopen libtest.so"));
    }

    // Now it should work
    {
        remove_var("LD_LIBRARY_PATH");
        set_var("NIX_LD_LIBRARY_PATH", libtest);
        let (stdout, _) = must_succeed(&bin);
        assert!(stdout.contains("Hello from libtest"));
    }
}

/// Check that LD_LIBRARY_PATH is restored.
#[cfg(all(
    feature = "entry_trampoline",
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
#[rstest]
fn test_ld_path_restore(libtest: &str, _dt_needed_bin: &Path) {
    let bin = compile_test_bin("ld-path-restore", &["test"]);
    set_var("NIX_LD_LIBRARY_PATH", format!("{}:POISON", libtest));

    // First try without LD_LIBRARY_PATH
    {
        remove_var("LD_LIBRARY_PATH");
        let (stdout, stderr) = must_succeed(&bin);
        assert!(stderr.contains("No LD_LIBRARY_PATH"));
        assert!(stdout.contains("Hello from libtest"));
    }

    // Now with LD_LIBRARY_PATH
    {
        set_var("LD_LIBRARY_PATH", "NEEDLE");
        let (stdout, stderr) = must_succeed(&bin);
        assert!(stderr.contains("LD_LIBRARY_PATH contains needle"));
        assert!(stdout.contains("Hello from libtest"));
        assert!(stderr.contains("Launching child process"));
        assert!(stderr.contains("loading shared")); // error from the child process
    }
}

// Utilities

const EXE: &str = env!("CARGO_BIN_EXE_nix-ld-rs");
const TARGET: &str = env!("NIX_LD_TEST_TARGET");

lazy_static! {
    static ref TMPDIR: TempDir = tempfile::tempdir().expect("Failed to create temporary directory");
}

fn find_cc() -> String {
    env::var(format!("CC_{}", TARGET))
        .or_else(|_| env::var("CC"))
        .unwrap_or_else(|_| "cc".to_string())
}

fn get_source_file(file: &str) -> PathBuf {
    // CARGO_MANIFEST_DIR doesn't necessarily point to the source, but
    // then there is no good way to get the source from here
    let base = PathBuf::from(&env::var("CARGO_MANIFEST_DIR").unwrap());
    base.join(file)
}

fn compile_test_lib(name: &str) {
    let cc = find_cc();
    let source_path = get_source_file(&format!("tests/lib{}.c", name));
    let out_path = TMPDIR.path().join(&format!("lib{}.so", name));

    let status = Command::new(cc)
        .arg("-shared")
        .arg("-o")
        .arg(&out_path)
        .arg(source_path)
        .status()
        .expect("Failed to spawn compiler");

    assert!(status.success(), "Failed to build test library {}", name);
}

fn compile_test_bin(name: &str, libs: &[&str]) -> PathBuf {
    let cc = find_cc();
    let source_path = get_source_file(&format!("tests/{}.c", name));
    let out_path = TMPDIR.path().join(name);

    let out_dir_arg = format!("-DOUT_DIR=\"{}\"", TMPDIR.path().to_str().unwrap());
    let dynamic_linker_arg = format!("-Wl,--dynamic-linker,{}", EXE);

    let status = Command::new(cc)
        .arg("-o")
        .arg(&out_path)
        .arg(out_dir_arg)
        .arg(dynamic_linker_arg)
        .arg("-L")
        .arg(TMPDIR.path())
        .args(libs.iter().map(|l| format!("-l{}", l)))
        .arg(source_path)
        .status()
        .expect("Failed to spawn compiler");

    assert!(status.success(), "Failed to build test binary {}", name);

    out_path
}

fn must_succeed(bin: impl AsRef<Path>) -> (String, String) {
    run(bin, true)
}

fn must_fail(bin: impl AsRef<Path>) -> (String, String) {
    run(bin, false)
}

fn run(bin: impl AsRef<Path>, want_success: bool) -> (String, String) {
    let bin = bin.as_ref();

    eprintln!("Running binary {:?}", bin);
    let output = Command::new(bin)
        .output()
        .expect("Failed to spawn test binary");

    let stdout = String::from_utf8(output.stdout).expect("stdout contains non-UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr contains non-UTF-8");

    print!("{}", stdout);
    eprint!("{}", stderr);

    if want_success {
        assert!(
            output.status.success(),
            "{:?} did not run successfully",
            bin.file_name().unwrap()
        );
    } else {
        assert!(
            !output.status.success(),
            "{:?} unexpectedly succeeded",
            bin.file_name().unwrap()
        );
    }

    (stdout, stderr)
}
