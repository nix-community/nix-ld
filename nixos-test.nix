{
  makeTest ? import <nixpkgs/nixos/tests/make-test-python.nix>,
  pkgs ? import <nixpkgs>,
}: let
  nix-ld = pkgs.callPackage ./nix-ld.nix {};
in {
  smoketest =
    makeTest {
      name = "smoketest";
      nodes.machine = import ./nixos-example.nix;
      testScript = ''
        start_all()
        machine.succeed("hello")
        machine.succeed("ls -la /run/current-system/sw/share/nix-ld/lib/ld.so >&2")
        machine.succeed("$(< ${nix-ld}/nix-support/ldpath) --version")

        # test fallback if NIX_LD is not set
        #machine.succeed("unset NIX_LD; unset NIX_LD_LIBRARY_PATH; $(< ${nix-ld}/nix-support/ldpath) $(which hello)")
      '';
    } {
      inherit pkgs;
      inherit (pkgs) system;
    };
}
