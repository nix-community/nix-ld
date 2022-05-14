{pkgs, ...}: {
  systemd.tmpfiles.rules = let
    nix-ld = pkgs.callPackage ../nix-ld.nix {};
  in [
    "L+ ${nix-ld.ldPath} - - - - ${nix-ld}/libexec/nix-ld"
  ];
}
