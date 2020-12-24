{ lib, system, stdenv, musl, meson, ninja, overrideCC, path, pkgs }:
let
  self = stdenv.mkDerivation rec {
    name = "nix-ld";
    src = ./.;

    doCheck = true;

    nativeBuildInputs = [ meson ninja ];

    mesonFlags = [
      "-Dmusl-lib=${lib.getLib musl}/lib"
      "-Dmusl-includes=${lib.getDev musl}/include"
    ];

    # our glibc is not compiled with support for static pie binaries,
    # also the musl binary is only 1/10 th of the size of the glibc binary

    passthru.tests = import ./nixos-test.nix {
      makeTest = import (path + "/nixos/tests/make-test-python.nix");
      inherit pkgs;
    };
    passthru.ldPath = let
      libDir = if system == "x86_64-linux" ||
                  system == "mips64-linux" ||
                  system == "powerpc64le-linux"
               then
                 "/lib64"
               else
                 "/lib";
      ldName = lib.fileContents "${self}/nix-support/ld-name";
    in "${libDir}/${ldName}";
  };
in self
