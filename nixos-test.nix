{
  makeTest ? import <nixpkgs/nixos/tests/make-test-python.nix>,
  pkgs ? import <nixpkgs>,
}: {
  smoketest =
    makeTest {
      name = "smoketest";
      nodes.machine = import ./nixos-example.nix;
      testScript = ''
        start_all()
        path = "${pkgs.stdenv.cc}/nix-support/dynamic-linker"
        with open(path) as f:
            real_ld = f.read().strip()
        machine.succeed(f"NIX_LD={real_ld} hello")
      '';
    } {
      inherit pkgs;
      inherit (pkgs) system;
    };
}
