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
      {
        nix-ld = pkgs.callPackage ./default.nix { };
        default = self.packages.${system}.nix-ld;
      });
    checks = nixpkgs.lib.genAttrs [ "x86_64-linux" "aarch64-linux" ] (system:
      self.packages.${system}.nix-ld.tests
    );
  };
}
