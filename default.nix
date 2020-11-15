{ pkgs ? import <nixpkgs> {} }:
let
  inherit (pkgs) lib stdenv musl;
  self = stdenv.mkDerivation rec {
    name = "nix-ld";
    src = ./.;

    # our glibc is not compiled with support for static pie binaries,
    # also the musl binary is only 1/10 th of the size of the glibc binary
    postConfigure = lib.optionalString (stdenv.hostPlatform.libc != "musl") ''
      makeFlagsArray+=("LD_CC=${stdenv.cc.targetPrefix}cc -isystem ${musl.dev}/include -B${musl}/lib -L${musl}/lib")
    '';
    doCheck = true;

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
