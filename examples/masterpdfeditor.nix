{
  pkgs ? import <nixpkgs> { },
}:
let
  inherit (pkgs) lib stdenv qt5;
in
# run with
# $ nix-shell ./masterpdfeditor.nix
pkgs.mkShell {
  NIX_LD_LIBRARY_PATH = lib.makeLibraryPath [
    pkgs.nss
    pkgs.sane-backends
    pkgs.nspr
    pkgs.zlib
    pkgs.libglvnd
    qt5.qtbase
    qt5.qtsvg
    qt5.qtdeclarative
    qt5.qtwayland
    pkgs.pkcs11helper
    stdenv.cc.cc
  ];

  NIX_LD = builtins.readFile "${stdenv.cc}/nix-support/dynamic-linker";

  QT_PLUGIN_PATH = "${qt5.qtbase}/${qt5.qtbase.qtPluginPrefix}:${qt5.qtwayland.bin}/${qt5.qtbase.qtPluginPrefix}";
  QML2_IMPORT_PATH = "${qt5.qtdeclarative.bin}/${qt5.qtbase.qtQmlPrefix}:${qt5.qtwayland.bin}/${qt5.qtbase.qtQmlPrefix}";

  shellHook = ''
    if [ ! -d master-pdf-editor ]; then
      echo "unpack master-pdf-editor..."
      mkdir master-pdf-editor
      tar -C master-pdf-editor \
          --strip-components 1 \
          -xf ${pkgs.masterpdfeditor.src}
    fi
    echo '$ ./master-pdf-editor/masterpdfeditor5'
    ./master-pdf-editor/masterpdfeditor5
  '';
}
