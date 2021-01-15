with import <nixpkgs> {};

let
  version = "5.4.57862.0110";
  src = fetchurl {
    url = "https://zoom.us/client/${version}/zoom_x86_64.tar.xz";
    sha256 = "11va3px42y81bwy10mxm7mk0kf2sni9gwb422pq9djck2dgchw5x";
  };
in mkShell {
  # Based on https://support.zoom.us/hc/en-us/articles/204206269-Installing-or-updating-Zoom-on-Linux#h_eabd7b65-e032-450b-b65d-83ec6f75e0b5
  # and : $ find . -type f -iname '*.so*' | LD_LIBRARY_PATH=$NIX_LD_LIBRARY_PATH:$PWD xargs -n1 ldd | grep 'not found' | sort -u
  NIX_LD_LIBRARY_PATH = lib.makeLibraryPath [
    xorg.libX11
    xorg.libXfixes
    glib
    libGL
    sqlite
    cairo
    atk
    xorg.libXrender
    xorg.libXcomposite
    libxslt
    wayland
    gst_all_1.gst-plugins-base
    xorg.libxcb
    xorg.xcbutilimage
    xorg.xcbutilkeysyms
    xorg.libXtst
    xorg.libXext
    alsaLib
    pulseaudio
    udev
    xorg.libXi
    xorg.libSM
    fontconfig
    freetype
    libxkbcommon
    gtk3

    dbus
    stdenv.cc.cc
    zlib
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
