# nix-ld

Run unpatched dynamic binaries on NixOS. Precompiled binaries not built for
NixOS usually have a so called link-loader hard coded.
For example, on Linux/x86_64 this is `/lib64/ld-linux-x86-64.so.2` for glibc.
NixOS on the other hand normally has its dynamic linker in the glibc
package in the nix store and therefore cannot run those binaries.
Nix-ld provides a shim layer for these kind of binaries. It
is installed to the same location where other Linux distributions 
install their link loader i.e. `/lib64/ld-linux-x86-64.so.2` and
it will then chainload the actual link loader as specified in the environment
variable `NIX_LD`. Furthermore, it also accepts a comma seperated
path of library lookup paths in `NIX_LD_LIBRARY_PATH`. This environment
variable will be rewritten to `LD_LIBRARY_PATH` before passing execution
to the actual ld. This allows you to specify additional libraries that the
executable needs for execution.

## Installation

```sh
$ sudo nix-channel --add https://github.com/Mic92/nix-ld/archive/main.tar.gz nix-ld
$ sudo nix-channel --update
```

`/etc/nixos/configuration.nix`

```nix
{
  imports = [
    <nix-ld/modules/nix-ld.nix>
  ];
}
```

### With nix flake 

Add the following lines to `/etc/nixos/flake.nix`. Replace `myhostname` with the
actual hostname of your system. 

```nix
# flake.nix
{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/master";
  inputs.nix-ld.url = "github:Mic92/nix-ld";
  # this line assume that you also have nixpkgs as an input
  inputs.nix-ld.inputs.nixpkgs.follows = "nixpkgs";
  
  outputs = { nix-ld, nixpkgs, ... }: {
    # replace `myhostname` with your actual hostname
    nixosConfigurations.myhostname = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        # ... add this line to the rest of your configuration modules
        nix-ld.nixosModules.nix-ld
      ];
    };
  };
}
```


## Usage

After setting up the nix-ld symlink as described above, one needs to  set
`NIX_LD` and `NIX_LD_LIBRARY_PATH` to run executables. For example, this can
be done with a `shell.nix` in a nix-shell like this:

```nix
{
  NIX_LD_LIBRARY_PATH = lib.makeLibraryPath [
    stdenv.cc.cc
    openssl
    # ...
  ];
  NIX_LD = lib.fileContents "${stdenv.cc}/nix-support/dynamic-linker";
}
```

A full example is shown in `./examples/masterpdfeditor.nix`.
In [nix-autobahn](https://github.com/Lassulus/nix-autobahn) there is also a
script called `nix-autobahn-ld` that automates generating shell expressions.

## Known Issues

### LD_LIBRARY_PATH is inherited by child processes

nix-ld currently rewrites NIX_LD_LIBRARY_PATH to LD_LIBRARY_PATH. This can
lead to problems if program loaded with this loader executes a normal binary
that should not receive those libraries. It might be possible in future
redirect execution back to nix-ld after the real library loader has performed
its work by changing the entry point in memory to fix this.

## FAQ

### How to find libraries for my executables?

You can use tools like [nix-autobahn](https://github.com/Lassulus/nix-autobahn) or use [nix-index](https://github.com/bennofs/nix-index)

### Why not set LD_LIBRARY_PATH directly instead of NIX_LD_LIBRARY_PATH?

LD_LIBRARY_PATH affects all programs, which can inject the wrong libraries in correct build nix application 
that have an RPATH set in their executable.

### Does this work on non-NixOS system?

No. Normal Linux distributions will have their own link-loader. Replacing those with nix-ld will break the system.
