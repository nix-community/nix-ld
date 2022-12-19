{pkgs, ...}:
{
  config = {
    programs.nix-ld.enable = true;
    programs.nix-ld.package = pkgs.callPackage ../nix-ld.nix {};
  };
}
