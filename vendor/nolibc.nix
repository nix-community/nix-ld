{
  linuxPackages_latest,
  runCommand,
  git,
}:
let
  linux = linuxPackages_latest.kernel;
in
runCommand "linux-nolibc-${linux.version}"
  {
    inherit (linux) src;
    nativeBuildInputs = [ git ];
  }
  ''
    tar xvf $src --wildcards '*/tools/include/nolibc/*.h' --strip-components=3
    cp -r nolibc{,.orig}

    # Stores into environ and _auxv in _start break PIE :(
    sed -i -E '/".*(_auxv|environ)/s/^\/*/\/\//' nolibc/arch-*.h

    mkdir -p $out
    cp -r nolibc $out

    echo '# Vendored from Linux ${linux.version} (${linux.src.url})' >$out/nolibc/vendor.patch
    git diff --no-index nolibc.orig nolibc >>$out/nolibc/vendor.patch || true
  ''
