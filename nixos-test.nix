{ makeTest ? import <nixpkgs/nixos/tests/make-test-python.nix>, pkgs ? import <nixpkgs> }:
{
 smoketest = makeTest {
   name = "smoketest";
   nodes.machine = import ./nixos-example.nix;
   testScript = ''
     start_all()
     machine.succeed("hello")
   '';
 } {
   inherit pkgs;
 };
}
