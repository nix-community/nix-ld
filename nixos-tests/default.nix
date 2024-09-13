{
  testers,
}:
{
  basic = testers.runNixOSTest {
    name = "basic";
    nodes.machine =
      { pkgs, config, ... }:
      {
        imports = [
          ../modules/nix-ld.nix
        ];
        programs.nix-ld.dev.enable = true;
        environment.systemPackages = [
          (pkgs.runCommand "patched-hello" { } ''
            install -D -m755 ${pkgs.hello}/bin/hello $out/bin/hello
            patchelf $out/bin/hello --set-interpreter $(cat ${config.programs.nix-ld.dev.package}/nix-support/ldpath)
          '')
        ];
      };
    testScript = { nodes, ... }: let
      nix-ld = nodes.machine.programs.nix-ld.dev.package;
    in ''
      start_all()
      machine.succeed("hello")
      machine.succeed("ls -la /run/current-system/sw/share/nix-ld/lib/ld.so >&2")
      machine.succeed("$(< ${nix-ld}/nix-support/ldpath) --version")

      # test fallback if NIX_LD is not set
      machine.succeed("unset NIX_LD; unset NIX_LD_LIBRARY_PATH; $(< ${nix-ld}/nix-support/ldpath) $(which hello)")
    '';
  };
}
