{
  description = "nix-ld: run unpatched dynamic binaries on NixOS";

  inputs.utils.url = "github:numtide/flake-utils";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs";

  outputs = { self, nixpkgs, utils }:
    utils.lib.eachSystem nixpkgs.lib.platforms.linux (system: let
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      packages.nix-ld = pkgs.callPackages ./default.nix {};
      defaultPackage = self.packages.${system}.nix-ld;
    });
}
