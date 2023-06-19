use std::env;

fn main() {
    println!("cargo:rerun-if-changed=src/nolibc.c");
    println!("cargo:rerun-if-changed=vendor");

    cc::Build::new()
        .file("src/nolibc.c")
        .include("vendor/nolibc")
        .flag("-ffreestanding")
        .flag("-fvisibility=hidden")
        .flag("-fno-common")
        // We don't want it to be linked in the integration tests
        // TODO: Send an issue to cc-rs to restrict rustc-link-arg to specific targets
        // This way we are losing a few "cargo:rerun-if-env-changed=" which
        // are useful.
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

    // For Cargo integration tests
    println!(
        "cargo:rustc-env=NIX_LD_TEST_TARGET={}",
        env::var("TARGET").unwrap()
    );

    //let out_dir = std::env::var("OUT_DIR").unwrap();
    //panic!("OUT_DIR: {}", out_dir);
}
