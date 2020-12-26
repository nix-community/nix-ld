with import <nixpkgs> {};

mkShell {
  NIX_LD_LIBRARY_PATH = lib.makeLibraryPath [
    nss
    sane-backends
    nspr
    qt5.qtbase
    qt5.qtsvg
    stdenv.cc.cc
  ];
  QT_PLUGIN_PATH = "${qt5.qtbase}/${qt5.qtbase.qtPluginPrefix}";
  shellHook = ''
    export NIX_LD="$(cat $NIX_CC/nix-support/dynamic-linker)"

    if [ ! -d master-pdf-editor ]; then
      echo "unpack master-pdf-editor..."
      mkdir master-pdf-editor
      tar -C master-pdf-editor \
          --strip-components 1 \
          -xf ${masterpdfeditor.src}
    fi
    echo '$ ./master-pdf-editor/masterpdfeditor5'
    ./master-pdf-editor/masterpdfeditor5
  '';
}
