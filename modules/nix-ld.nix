{ pkgs, ... }:
{
  systemd.tmpfiles.rules = let
    nix-ld = pkgs.callPackage ./.. { inherit pkgs; };
  in [
    "L+ ${nix-ld.ldPath} - - - - ${nix-ld}/libexec/nix-ld"
  ];
}
