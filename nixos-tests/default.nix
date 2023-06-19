{
  pkgs ? import ./nixpkgs.nix,
  nix-ld-rs ? null,
}: let
  inherit (pkgs) lib;

  nixosLib = import (pkgs.path + "/nixos/lib") {
    inherit pkgs lib;
  };
  nix-ld-rs' = if nix-ld-rs != null then nix-ld-rs else pkgs.nix-ld-rs;
in {
  basic = nixosLib.runTest {
    name = "basic";
    hostPkgs = pkgs;
    nodes.machine = { pkgs, ... }: {
      programs.nix-ld = {
        enable = true;
        package = nix-ld-rs';
      };
      environment.systemPackages = [
        (pkgs.runCommand "patched-hello" {} ''
          install -D -m755 ${pkgs.hello}/bin/hello $out/bin/hello
          patchelf $out/bin/hello --set-interpreter $(cat ${nix-ld-rs'}/nix-support/ldpath)
        '')
      ];
    };
    testScript = ''
      start_all()
      machine.succeed("hello")
      machine.succeed("ls -la /run/current-system/sw/share/nix-ld/lib/ld.so >&2")
      machine.succeed("$(< ${nix-ld-rs'}/nix-support/ldpath) --version")

      # test fallback if NIX_LD is not set
      machine.succeed("unset NIX_LD; unset NIX_LD_LIBRARY_PATH; $(< ${nix-ld-rs'}/nix-support/ldpath) $(which hello)")
    '';
  };
}
