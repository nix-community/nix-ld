{ pkgs ? import <nixpkgs> { } }:
let
  inherit (pkgs) lib stdenv xorg;
  src = pkgs.zoom-us.src;
in
pkgs.mkShell {
  NIX_LD_LIBRARY_PATH = lib.makeLibraryPath [
    # found by
    # $ LD_LIBRARY_PATH=$NIX_LD_LIBRARY_PATH:$PWD ldd zoom | grep 'not found'
    pkgs.alsa-lib
    pkgs.atk
    pkgs.cairo
    pkgs.dbus
    pkgs.libGL
    pkgs.fontconfig
    pkgs.freetype
    pkgs.gtk3
    pkgs.gdk-pixbuf
    pkgs.glib
    pkgs.pango
    stdenv.cc.cc
    pkgs.pulseaudio
    pkgs.wayland
    xorg.libX11
    xorg.libxcb
    xorg.libXcomposite
    xorg.libXext
    pkgs.libxkbcommon
    xorg.libXrender
    pkgs.zlib
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
