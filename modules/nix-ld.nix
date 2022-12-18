{pkgs, config, lib, ...}:
let
  cfg = config.programs.nix-ld;

  # TODO make glibc here configureable?
  nix-ld-so = pkgs.runCommand "ld.so" {} ''
    ln -s "$(cat '${pkgs.stdenv.cc}/nix-support/dynamic-linker')" $out
  '';

  nix-ld-libraries = pkgs.buildEnv {
    name = "lb-library-path";
    pathsToLink = [ "/lib" ];
    paths = map lib.getLib cfg.libraries;
    extraPrefix = "/share/nix-ld";
    ignoreCollisions = true;
  };

  # We currently take all libraries from systemd and nix as the default.
  # Is there a better list?
  baseLibraries = with pkgs; [
    zlib
    zstd
    stdenv.cc.cc
    curl
    openssl
    attr
    libssh
    bzip2
    libxml2
    acl
    libsodium
    util-linux
    xz
    systemd
  ];
in
{
  options = {
    programs.nix-ld.libraries = lib.mkOption {
      type = lib.types.listOf lib.types.package;
      description = "Libraries that automatically become available to all programs. The default set includes common libraries.";
      default = baseLibraries;
    };
  };
  config = lib.mkIf cfg.enable {
    systemd.tmpfiles.packages = [
      (pkgs.callPackage ../nix-ld.nix {})
    ];

    environment.systemPackages = [ nix-ld-libraries ];

    environment.pathsToLink = [ "/share/nix-ld" ];

    environment.variables = {
      NIX_LD = toString nix-ld-so;
      NIX_LD_LIBRARY_PATH = "/run/current-system/sw/share/nix-ld/lib";
    };
  };
}
