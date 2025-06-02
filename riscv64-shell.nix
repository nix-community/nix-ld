{ mkShell
, rustc
, cargo
, cargo-watch
, cargo-bloat
, cargo-nextest
, just
, stdenv
, qemu
}:

mkShell {
  nativeBuildInputs = [
    rustc
    cargo
    cargo-watch
    cargo-bloat
    cargo-nextest
    just
    qemu
  ];

  hardeningDisable = [ "stackprotector" ];

  RUSTC_BOOTSTRAP = "1";

  # Cross compilation environment
  CARGO_TARGET_RISCV64GC_UNKNOWN_LINUX_GNU_LINKER = "${stdenv.cc}/bin/${stdenv.cc.targetPrefix}cc";
  CARGO_TARGET_RISCV64GC_UNKNOWN_LINUX_GNU_RUNNER = "qemu-riscv64 -L /";
  CC_riscv64gc_unknown_linux_gnu = "${stdenv.cc}/bin/${stdenv.cc.targetPrefix}cc";
  NIX_LD_TEST_TARGET = "riscv64gc-unknown-linux-gnu";
  NIX_LD = "${stdenv.cc.libc}/lib/ld-linux-riscv64-lp64d.so.1";
  TARGET_CC = "${stdenv.cc}/bin/${stdenv.cc.targetPrefix}cc";

  shellHook = ''
    echo "RISC-V 64-bit cross-compilation environment"
    echo "Target: riscv64gc-unknown-linux-gnu"
    echo "Cross compiler: ${stdenv.cc}/bin/${stdenv.cc.targetPrefix}cc"
    echo "Usage:"
    echo "  cargo build --target riscv64gc-unknown-linux-gnu"
    echo "  cargo test --target riscv64gc-unknown-linux-gnu"
  '';
}
