{
  description = "nix-ld: run unpatched dynamic binaries on NixOS";

  inputs.utils.url = "github:numtide/flake-utils";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs";

  outputs = { self, nixpkgs, utils }: {
    nixosModules.nix-ld = import ./modules/nix-ld.nix;
  } // utils.lib.eachSystem ["x86_64-linux" "aarch64-linux"] (system: let
    pkgs = nixpkgs.legacyPackages.${system};
  in {
    packages.nix-ld = pkgs.callPackage ./default.nix {};
    defaultPackage = self.packages.${system}.nix-ld;
  });
}
