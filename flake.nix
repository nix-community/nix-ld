{
  description = "Run unpatched dynamic binaries on NixOS, but this time with more Rust";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs, ... }:
    let
      # System types to support.
      supportedSystems = [
        "i686-linux"
        "x86_64-linux"
        "aarch64-linux"
      ];
      lib = nixpkgs.lib;
      forAllSystems =
        f:
        nixpkgs.lib.genAttrs supportedSystems (
          system:
          f {
            pkgs = nixpkgs.legacyPackages.${system};
            inherit system;
          }
        );
    in
    {
      packages = forAllSystems (
        { pkgs, system, ... }:
        {
          nix-ld = pkgs.callPackage ./package.nix { };
          nolibc = pkgs.callPackage ./vendor/nolibc.nix { };
          default = self.packages.${system}.nix-ld;
        }
      );

      checks = forAllSystems (
        { pkgs, system, ... }:
        let
          nixosTests = pkgs.callPackage ./nixos-tests { };
          packages = lib.mapAttrs' (n: lib.nameValuePair "package-${n}") self.packages.${system};
          devShells = lib.mapAttrs' (n: lib.nameValuePair "devShell-${n}") self.devShells.${system};
        in
        packages
        // devShells
        // lib.optionalAttrs (system != "i686-linux") {
          # test driver is broken on i686-linux
          inherit (nixosTests) basic;
        }
        // {
          clippy = self.packages.${system}.nix-ld.override {
            enableClippy = true;
          };
        }
      );

      devShells = forAllSystems (
        { pkgs, ... }:
        {
          nix-ld = pkgs.mkShell {
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
          };
        }
      );
    }
    // {
      nixosModules.nix-ld = import ./modules/nix-ld.nix;
    };
}
