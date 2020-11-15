with import <nixpkgs> {};
stdenv.mkDerivation rec {
  name = "nix-ld";
  src = ./.;

  # our glibc is not compiled with support for static pie binaries,
  # also the musl binary is only 1/10 th of the size of the glibc binary
  postConfigure = lib.optionalString (stdenv.hostPlatform.libc != "musl") ''
    makeFlagsArray+=("LD_CC=${stdenv.cc.targetPrefix}cc -isystem ${musl.dev}/include -B${musl}/lib -L${musl}/lib")
  '';
  doCheck = true;

  installFlags = [ "PREFIX=$(out)" ];
}
