{pkgs, lib, ...}:
{
  config = {
    programs.nix-ld.enable = true;
  } // lib.optionalAttrs (lib.versionAtLeast (lib.versions.majorMinor lib.version) "23.05") {
    # 22.11 users won't actually use nix-ld from this repo but at least it does not break their configuration.
    programs.nix-ld.package = pkgs.callPackage ../nix-ld.nix {};
  };
}
