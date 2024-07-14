use std::env;

fn main() {
    println!("cargo:rerun-if-changed=src/nolibc.c");
    println!("cargo:rerun-if-changed=vendor");

    cc::Build::new()
        .file("src/nolibc.c")
        .include("vendor/nolibc")
        .flag("-fPIE")
        .flag("-ffreestanding")
        .flag("-fvisibility=hidden")
        .flag("-fno-common")
        // We don't want nolibc to be linked in the integration tests
        // TODO: Send an issue to cc-rs to restrict rustc-link-arg to specific targets
        // By disabling cargo_metadata, we are also losing a few "cargo:rerun-if-env-changed="
        // which are useful :(
        .cargo_metadata(false)
        .compile("c_kinda");

    println!("cargo:rustc-link-arg-bins=-lc_kinda");
    println!(
        "cargo:rustc-link-search=native={}",
        env::var("OUT_DIR").unwrap()
    );

    println!("cargo:rustc-link-arg-bins=-nostartfiles");
    println!("cargo:rustc-link-arg-bins=-nodefaultlibs");
    println!("cargo:rustc-link-arg-bins=-static-pie");
    println!("cargo:rustc-link-arg-bins=-fstack-protector");
    println!("cargo:rustc-link-arg-bins=-Wl,--no-dynamic-linker");

    let target = env::var("TARGET").unwrap();

    // For Cargo integration tests
    println!("cargo:rustc-env=NIX_LD_TEST_TARGET={}", target);

    // For cross-compiling in the devShell *only*
    let target_suffix = target.replace('-', "_");
    if let Ok(target_default_nix_ld) = env::var(format!("DEFAULT_NIX_LD_{}", target_suffix)) {
        println!("cargo:rustc-env=DEFAULT_NIX_LD={}", target_default_nix_ld);
    }

    if let Ok(nix_system) = env::var("NIX_SYSTEM") {
        let underscored = nix_system.replace('-', "_");
        println!("cargo:rustc-env=NIX_SYSTEM={}", underscored);
    }

    //let out_dir = std::env::var("OUT_DIR").unwrap();
    //panic!("OUT_DIR: {}", out_dir);
}
