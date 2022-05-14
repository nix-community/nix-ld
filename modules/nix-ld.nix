{pkgs, ...}: {
  systemd.tmpfiles.rules = let
    nix-ld = pkgs.callPackage ./.. {};
  in [
    "L+ ${nix-ld.ldPath} - - - - ${nix-ld}/libexec/nix-ld"
  ];
}
