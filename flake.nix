{
  description = "nix-ld: run unpatched dynamic binaries on NixOS";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs";

  nixConfig.extra-substituters = [ "https://cache.garnix.io" ];
  nixConfig.extra-trusted-public-keys = [ "cache.garnix.io:CTFPyKSLcx5RMJKfLo5EEPUObbA78b0YQ2DTCJXqr9g=" ];

  outputs = { self, nixpkgs }: {
    nixosModules.nix-ld = import ./modules/nix-ld.nix;
    packages = nixpkgs.lib.genAttrs [ "x86_64-linux" "aarch64-linux" ] (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      ({
        nix-ld = pkgs.callPackage ./default.nix { };
        default = self.packages.${system}.nix-ld;
      } // nixpkgs.lib.optionalAttrs (system == "x86_64-linux") {
        nix-ld_32bit = pkgs.pkgsi686Linux.callPackage ./default.nix { };
      }));
    checks = nixpkgs.lib.genAttrs [ "x86_64-linux" "aarch64-linux" ] (system:
      let
        inherit (nixpkgs) lib;
        packages = lib.mapAttrs' (n: lib.nameValuePair "package-${n}") self.packages.${system};
        devShells = lib.mapAttrs' (n: lib.nameValuePair "devShell-${n}") self.devShells.${system};
      in
      packages // devShells // self.packages.${system}.nix-ld.tests
    );
  };
}
