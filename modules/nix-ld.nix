{pkgs, ...}: {
  systemd.tmpfiles.packages = [
    (pkgs.callPackage ../nix-ld.nix {})
  ];
}
