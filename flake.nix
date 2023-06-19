{
  description = "Run unpatched dynamic binaries on NixOS, but this time with more Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }: let
    # System types to support.
    supportedSystems = [ "i686-linux" "x86_64-linux" "aarch64-linux" ];
  in flake-utils.lib.eachSystem supportedSystems (system: let
    pkgs = import nixpkgs {
      inherit system;
      overlays = [
        rust-overlay.overlays.default
      ];
    };

    rustDev = pkgs.rust-bin.stable."${pkgs.rustc.version}".default.override {
      extensions = [ "rust-src" ];
      targets = [ "x86_64-unknown-linux-gnu" "i686-unknown-linux-gnu" "aarch64-unknown-linux-gnu" ];
    };
  in {
    packages = rec {
      nix-ld-rs = pkgs.callPackage ./package.nix {};
      default = nix-ld-rs;
    };
    checks = import ./nixos-tests {
      inherit pkgs;
      nix-ld-rs = self.packages.${system}.nix-ld-rs;
    };
    devShell = pkgs.mkShell {
      nativeBuildInputs = with pkgs; [
        rustDev
        cargo-bloat
        cargo-nextest
      ];

      hardeningDisable = [ "stackprotector" ];

      # For convenience in devShell
      DEFAULT_NIX_LD = pkgs.stdenv.cc.bintools.dynamicLinker;

      RUSTC_BOOTSTRAP = "1";
    };
  }) // {
    overlays.default = final: prev: {
      nix-ld-rs = final.callPackage ./package.nix { };
    };
  };
}
