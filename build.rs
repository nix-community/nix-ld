fn main() {
    cc::Build::new()
        .file("src/nolibc.c")
        .include("vendor/nolibc")
        .flag("-ffreestanding")
        .flag("-fvisibility=hidden")
        .compile("c_kinda");

    println!("cargo:rustc-link-arg=-nostartfiles");
    println!("cargo:rustc-link-arg=-nodefaultlibs");
    println!("cargo:rustc-link-arg=-static-pie");
    println!("cargo:rustc-link-arg=-fstack-protector");
    println!("cargo:rustc-link-arg=-Wl,--no-dynamic-linker");

    //let out_dir = std::env::var("OUT_DIR").unwrap();
    //panic!("OUT_DIR: {}", out_dir);
}
