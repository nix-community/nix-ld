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
        machine.succeed("$(< ${nix-ld}/nix-support/ldpath) --version")
        machine.succeed("$(< ${nix-ld}/nix-support/ldpath) $(which hello)")
      '';
    } {
      inherit pkgs;
      inherit (pkgs) system;
    };
}
