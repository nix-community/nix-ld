{
  lib,
  stdenv,
  meson,
  ninja,
  overrideCC,
  path,
  pkgs,
}: let
  self = stdenv.mkDerivation rec {
    name = "nix-ld";
    src = ./.;

    doCheck = true;

    nativeBuildInputs = [meson ninja];

    mesonFlags = [
      "-Dnix-system=${stdenv.system}"
    ];

    hardeningDisable = [
      "stackprotector"
    ];

    postInstall = ''
      mkdir -p $out/nix-support
      basename $(< ${stdenv.cc}/nix-support/dynamic-linker) > $out/nix-support/ld-name
    '';

    passthru.tests = import ./nixos-test.nix {
      makeTest = import (path + "/nixos/tests/make-test-python.nix");
      inherit pkgs;
    };
    passthru.ldPath = let
      libDir =
        if
          stdenv.system
          == "x86_64-linux"
          || stdenv.system == "mips64-linux"
          || stdenv.system == "powerpc64le-linux"
        then "/lib64"
        else "/lib";
      ldName = lib.fileContents "${self}/nix-support/ld-name";
    in "${libDir}/${ldName}";
  };
in
  self
