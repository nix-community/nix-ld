with import <nixpkgs> {};

let
  version = "5.4.57862.0110";
  src = fetchurl {
    url = "https://zoom.us/client/${version}/zoom_x86_64.tar.xz";
    sha256 = "11va3px42y81bwy10mxm7mk0kf2sni9gwb422pq9djck2dgchw5x";
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
    pulseaudio
    stdenv.cc.cc
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
          --strip-components 1 \
          -xf ${src}
    fi
    echo '$ ./zoom/zoom.sh'
    ./zoom/zoom.sh
  '';
}
