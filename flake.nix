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

    inherit (pkgs) lib;

    otherSystems = lib.filter (s: system != s) supportedSystems;

    crossPlatforms = map (crossSystem: let
      pkgsCross = import nixpkgs {
        inherit crossSystem;
        localSystem = system;
        overlays = [];
      };

      rustTargetSpec = pkgs.rust.toRustTargetSpec pkgsCross.stdenv.hostPlatform;
      rustTargetSpecUnderscored = builtins.replaceStrings [ "-" ] [ "_" ] rustTargetSpec;
      systemUnderscored = builtins.replaceStrings [ "-" ] [ "_" ] crossSystem;
      ccbin = "${pkgsCross.stdenv.cc}/bin/${pkgsCross.stdenv.cc.targetPrefix}cc";
    in {
      inherit rustTargetSpec;
      env = {
        "CARGO_TARGET_${lib.toUpper rustTargetSpecUnderscored}_LINKER" = ccbin;
        "CC_${rustTargetSpecUnderscored}" = ccbin;
        "DEFAULT_NIX_LD_${rustTargetSpecUnderscored}" = pkgsCross.stdenv.cc.bintools.dynamicLinker;
        "NIX_LD_${systemUnderscored}" = pkgsCross.stdenv.cc.bintools.dynamicLinker;
      };
    }) otherSystems;

    crossEnvs = lib.foldl (acc: p: acc // p.env) {} crossPlatforms;

    rustDev = pkgs.rust-bin.stable."${pkgs.rustc.version}".default.override {
      extensions = [ "rust-src" ];
      targets = map (p: p.rustTargetSpec) crossPlatforms;
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
    devShell = pkgs.mkShell (crossEnvs // {
      nativeBuildInputs = with pkgs; [
        rustDev
        cargo-bloat
        cargo-nextest
        just
      ];

      hardeningDisable = [ "stackprotector" ];

      # For convenience in devShell
      DEFAULT_NIX_LD = pkgs.stdenv.cc.bintools.dynamicLinker;
      NIX_LD = pkgs.stdenv.cc.bintools.dynamicLinker;

      RUSTC_BOOTSTRAP = "1";
    });
  }) // {
    overlays.default = final: prev: {
      nix-ld-rs = final.callPackage ./package.nix { };
    };
  };
}
