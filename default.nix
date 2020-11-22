{ pkgs ? import <nixpkgs> {} }:
let
  inherit (pkgs) lib stdenv rustup;
  self = stdenv.mkDerivation rec {
    name = "nix-ld";
    src = ./.;

    doCheck = true;

    nativeBuildInputs = [ rustup ];

    passthru.tests = import ./nixos-test.nix {
      makeTest = import (pkgs.path + "/nixos/tests/make-test-python.nix");
      inherit pkgs;
    };
    passthru.ldPath = let
      libDir = if pkgs.system == "x86_64-linux" ||
                  pkgs.system == "mips64-linux" ||
                  pkgs.system == "powerpc64le-linux"
               then
                 "/lib64"
               else
                 "/lib";
      ldName = lib.fileContents "${self}/nix-support/ld-name";
    in "${libDir}/${ldName}";

    dontStrip = true;

    installFlags = [ "PREFIX=$(out)" ];
  };
in self
