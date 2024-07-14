{
  stdenv,
  pkgs,
  rustPlatform,
  nix-gitignore,
  enableClippy ? false,
}:

let
  libDir =
    if
      builtins.elem stdenv.system [
        "x86_64-linux"
        "mips64-linux"
        "powerpc64le-linux"
      ]
    then
      "/lib64"
    else
      "/lib";

  nix-ld-rs = rustPlatform.buildRustPackage {
    name = "nix-ld-rs";

    cargoLock.lockFile = ./Cargo.lock;

    src = nix-gitignore.gitignoreSource [ ] ./.;

    hardeningDisable = [ "stackprotector" ];

    NIX_SYSTEM = stdenv.system;
    RUSTC_BOOTSTRAP = "1";

    preCheck = ''
      export NIX_LD=${stdenv.cc.bintools.dynamicLinker}
    '';

    postInstall = ''
      mkdir -p $out/libexec
      ln -s $out/bin/nix-ld-rs $out/libexec/nix-ld-rs
      ln -s $out/bin/nix-ld-rs $out/libexec/nix-ld

      mkdir -p $out/nix-support

      ldpath=${libDir}/$(basename ${stdenv.cc.bintools.dynamicLinker})
      echo "$ldpath" > $out/nix-support/ldpath
      mkdir -p $out/lib/tmpfiles.d/
      cat > $out/lib/tmpfiles.d/nix-ld.conf <<EOF
        L+ $ldpath - - - - $out/libexec/nix-ld-rs
      EOF
    '';

    passthru.tests = import ./nixos-tests { inherit pkgs nix-ld-rs; };
  };
in
if enableClippy then
  nix-ld-rs.overrideAttrs (oldAttrs: {
    nativeBuildInputs = oldAttrs.nativeBuildInputs ++ [ pkgs.clippy ];
    phases = [
      "unpackPhase"
      "patchPhase"
      "installPhase"
    ];
    installPhase = ''
      cargo clippy -- -D warnings
      touch $out
    '';
  })
else
  nix-ld-rs
