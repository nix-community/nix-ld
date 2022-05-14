{pkgs, ...}: {
  imports = [./modules/nix-ld.nix];
  environment.systemPackages = [
    (pkgs.runCommand "patched-hello" {} ''
      install -D -m755 ${pkgs.hello}/bin/hello $out/bin/hello
      patchelf $out/bin/hello --set-interpreter $(cat ${(pkgs.callPackage ./. {})}/nix-support/ldpath)
    '')
  ];
}
