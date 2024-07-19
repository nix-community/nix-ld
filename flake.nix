{
  description = "Run unpatched dynamic binaries on NixOS, but this time with more Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, flake-utils, ... }: let
    # System types to support.
    supportedSystems = [ "i686-linux" "x86_64-linux" "aarch64-linux" ];
  in flake-utils.lib.eachSystem supportedSystems (system: let
    pkgs = nixpkgs.legacyPackages.${system};
    lib = pkgs.lib;
  in {
    packages = rec {
      nix-ld = pkgs.callPackage ./package.nix {};
      default = nix-ld;
    };

    checks = let
      nixosTests = import ./nixos-tests {
        inherit pkgs;
        nix-ld = self.packages.${system}.nix-ld;
      };
      packages = lib.mapAttrs' (n: lib.nameValuePair "package-${n}") self.packages.${system};
      devShells = lib.mapAttrs' (n: lib.nameValuePair "devShell-${n}") self.devShells.${system};
    in packages //
      devShells //
      # test driver is broken on i686-linux
      lib.optionalAttrs (system != "i686-linux") nixosTests // {
      clippy = self.packages.${system}.nix-ld.override {
        enableClippy = true;
      };
    };

    devShells.default = pkgs.mkShell ({
      nativeBuildInputs = [
        pkgs.rustc
        pkgs.cargo
        pkgs.cargo-watch
        pkgs.cargo-bloat
        pkgs.cargo-nextest
        pkgs.just
      ];

      hardeningDisable = [ "stackprotector" ];

      # For convenience in devShell
      DEFAULT_NIX_LD = pkgs.stdenv.cc.bintools.dynamicLinker;
      NIX_LD = pkgs.stdenv.cc.bintools.dynamicLinker;

      RUSTC_BOOTSTRAP = "1";
    });
  }) // {
    overlays.default = final: prev: {
      nix-ld = final.callPackage ./package.nix { };
    };

    nixosModules.nix-ld = import ./modules/nix-ld.nix;
  };
}
