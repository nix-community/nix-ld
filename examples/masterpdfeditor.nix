with import <nixpkgs> {};
# run with
# $ nix-shell ./masterpdfeditor.nix
  mkShell {
    NIX_LD_LIBRARY_PATH = lib.makeLibraryPath [
      nss
      sane-backends
      nspr
      qt5.qtbase
      qt5.qtsvg
      stdenv.cc.cc
    ];

    NIX_LD = builtins.readFile "${stdenv.cc}/nix-support/dynamic-linker";

    QT_PLUGIN_PATH = "${qt5.qtbase}/${qt5.qtbase.qtPluginPrefix}";

    shellHook = ''
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
