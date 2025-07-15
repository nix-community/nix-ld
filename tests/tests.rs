use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use rstest::*;
use std::sync::OnceLock;
use tempfile::TempDir;

#[fixture]
#[once]
fn libtest() -> &'static str {
    eprintln!("Testing {EXE} on {TARGET}");

    eprintln!("Building libtest");
    compile_test_lib("test");
    get_tmpdir().path().to_str().unwrap()
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

    let (stdout, _) = Command::new(&bin)
        .env_remove("LD_LIBRARY_PATH")
        .env_remove("NIX_LD_LIBRARY_PATH")
        .must_succeed();
    assert!(stdout.contains("Hello, world!"));
}

/// Check that we can run a binary with DT_NEEDED dependencies.
#[rstest]
fn test_dt_needed(libtest: &str, dt_needed_bin: &Path) {
    // First make sure it doesn't run without the library
    {
        let (_, stderr) = Command::new(dt_needed_bin)
            .env_remove("LD_LIBRARY_PATH")
            .env_remove("NIX_LD_LIBRARY_PATH")
            .must_fail();
        assert!(stderr.contains("loading shared"));
    }

    // Now it should work
    {
        let (stdout, _) = Command::new(dt_needed_bin)
            .env_remove("LD_LIBRARY_PATH")
            .env("NIX_LD_LIBRARY_PATH", libtest)
            .must_succeed();
        assert!(stdout.contains("Hello from libtest"));
    }
}

/// Check that we can run a binary that does dlopen.
#[rstest]
fn test_dlopen(libtest: &str) {
    let bin = compile_test_bin("dlopen", &[]);
    eprintln!("test_dlopen: {libtest}");

    // First make sure it doesn't run without the library
    {
        let (_, stderr) = Command::new(&bin)
            .env_remove("LD_LIBRARY_PATH")
            .env_remove("NIX_LD_LIBRARY_PATH")
            .must_fail();
        assert!(stderr.contains("Failed to dlopen libtest.so"));
    }

    // Now it should work
    {
        let (stdout, _) = Command::new(&bin)
            .env_remove("LD_LIBRARY_PATH")
            .env("NIX_LD_LIBRARY_PATH", libtest)
            .must_succeed();
        assert!(stdout.contains("Hello from libtest"));
    }
}

/// Check that LD_LIBRARY_PATH is restored.
#[cfg(all(
    feature = "entry_trampoline",
    any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        target_arch = "riscv64"
    )
))]
#[rstest]
fn test_ld_path_restore(libtest: &str, _dt_needed_bin: &Path) {
    let bin = compile_test_bin("ld-path-restore", &["test"]);

    let nix_ld_path = format!("{libtest}:POISON");

    // First try without LD_LIBRARY_PATH
    {
        let (stdout, stderr) = Command::new(&bin)
            .env_remove("LD_LIBRARY_PATH")
            .env("NIX_LD_LIBRARY_PATH", &nix_ld_path)
            .must_succeed();
        assert!(stderr.contains("No LD_LIBRARY_PATH"));
        assert!(stdout.contains("Hello from libtest"));
    }

    // Now with LD_LIBRARY_PATH
    {
        let (stdout, stderr) = Command::new(&bin)
            .env("LD_LIBRARY_PATH", "NEEDLE")
            .env("NIX_LD_LIBRARY_PATH", &nix_ld_path)
            .must_succeed();
        assert!(stderr.contains("LD_LIBRARY_PATH contains needle"));
        assert!(stdout.contains("Hello from libtest"));
        assert!(stderr.contains("Launching child process"));
        assert!(stderr.contains("loading shared")); // error from the child process
    }
}

// Utilities

const EXE: &str = env!("CARGO_BIN_EXE_nix-ld");
const TARGET: &str = env!("NIX_LD_TEST_TARGET");

static TMPDIR: OnceLock<TempDir> = OnceLock::new();

fn get_tmpdir() -> &'static TempDir {
    TMPDIR.get_or_init(|| tempfile::tempdir().expect("Failed to create temporary directory"))
}

fn find_cc() -> String {
    let target_suffix = TARGET.replace('-', "_");
    env::var(format!("CC_{target_suffix}"))
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
    let source_path = get_source_file(&format!("tests/lib{name}.c"));
    let out_path = get_tmpdir().path().join(format!("lib{name}.so"));

    let status = Command::new(cc)
        .arg("-fPIC")
        .arg("-shared")
        .arg("-o")
        .arg(&out_path)
        .arg(source_path)
        .status()
        .expect("Failed to spawn compiler");

    assert!(status.success(), "Failed to build test library {name}");
}

fn compile_test_bin(name: &str, libs: &[&str]) -> PathBuf {
    let cc = find_cc();
    let source_path = get_source_file(&format!("tests/{name}.c"));
    let out_path = get_tmpdir().path().join(name);

    let out_dir_arg = format!("-DOUT_DIR=\"{}\"", get_tmpdir().path().to_str().unwrap());
    let dynamic_linker_arg = format!("-Wl,--dynamic-linker,{EXE}");

    let status = Command::new(cc)
        .arg("-o")
        .arg(&out_path)
        .arg(out_dir_arg)
        .arg(dynamic_linker_arg)
        .arg("-L")
        .arg(get_tmpdir().path())
        .args(libs.iter().map(|l| format!("-l{l}")))
        .arg(source_path)
        .status()
        .expect("Failed to spawn compiler");

    assert!(status.success(), "Failed to build test binary {name}");

    out_path
}

trait CommandExt {
    fn output_checked(&mut self, want_success: bool) -> (String, String);
    fn must_succeed(&mut self) -> (String, String);
    fn must_fail(&mut self) -> (String, String);
}

impl CommandExt for Command {
    fn output_checked(&mut self, want_success: bool) -> (String, String) {
        eprintln!("Running binary {:?}", self.get_program());
        let output = self.output().expect("Failed to spawn test binary");

        let stdout = String::from_utf8(output.stdout).expect("stdout contains non-UTF-8");
        let stderr = String::from_utf8(output.stderr).expect("stderr contains non-UTF-8");

        print!("{stdout}");
        eprint!("{stderr}");

        if want_success {
            assert!(
                output.status.success(),
                "{:?} did not run successfully",
                self.get_program()
            );
        } else {
            assert!(
                !output.status.success(),
                "{:?} unexpectedly succeeded",
                self.get_program()
            );
        }

        (stdout, stderr)
    }

    fn must_succeed(&mut self) -> (String, String) {
        self.output_checked(true)
    }

    fn must_fail(&mut self) -> (String, String) {
        self.output_checked(false)
    }
}
