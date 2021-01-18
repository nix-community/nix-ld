with import <nixpkgs> {};

let
  version = "5.4.57862.0110";
  src = fetchurl {
    url = "https://zoom.us/client/${version}/zoom_x86_64.pkg.tar.xz";
    sha256 = "sha256-ZAwXhbZ3nT6PGkSC1vnX2y/XUOZfped0r3OuedI62gY=";
  };
in mkShell {
  NIX_LD_LIBRARY_PATH = lib.makeLibraryPath [
    # found by
    # $ LD_LIBRARY_PATH=$NIX_LD_LIBRARY_PATH:$PWD ldd zoom | grep 'not found'
    alsaLib
    atk
    cairo
    dbus
    libGL
    fontconfig
    freetype
    gtk3
    gdk-pixbuf
    glib
    pango
    stdenv.cc.cc
    pulseaudio
    wayland
    xorg.libX11
    xorg.libxcb
    xorg.libXcomposite
    xorg.libXext
    libxkbcommon
    xorg.libXrender
    zlib
    xorg.xcbutilimage
    xorg.xcbutilkeysyms
    xorg.libXfixes
    xorg.libXtst
  ];
  NIX_LD = builtins.readFile "${stdenv.cc}/nix-support/dynamic-linker";
  shellHook = ''
    if [ ! -d zoom ]; then
      echo "unpack zoom..."
      mkdir zoom
      tar -C zoom \
          -xf ${src}
    fi
    export LD_LIBRARY_PATH=$PWD/zoom/opt/zoom/
    echo '$ ./zoom/opt/zoom/ZoomLauncher'
    ./zoom/opt/zoom/ZoomLauncher
  '';
}
