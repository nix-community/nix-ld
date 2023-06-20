# nix-ld-rs

Run unpatched dynamic binaries on NixOS.
This is a rewrite of [nix-ld](https://github.com/Mic92/nix-ld) in Rust, with extra functionalities.
It's intended to be upstreamed at some point.

## Usage

`nix-ld-rs` is a drop-in replacement for `nix-ld`.

It honors the following environment variables:

- `NIX_LD`
- `NIX_LD_{system}`
- `NIX_LD_LIBRARY_PATH`
- `NIX_LD_LIBRARY_PATH_{system}`
- `NIX_LD_LOG` (error, warn, info, debug, trace)

Here `{system}` refers to the Nix `system`, like `x86_64-linux`.
You can also run `nix-ld-rs` directly for a list.

## Extra functionalities

- `NIX_LD_LIBRARY_PATH` doesn't affect child processes (on `x86_64-linux` and `aarch64-linux`)
    - For example, shell environments spawned by the binary VSCode Server no longer get polluted

## Development

The included `devShell` provides all dependencies required to build the project.
It's recommended to set up transparent emulation using binfmt-misc so you can run tests on all supported platforms:

```nix
{
  # x86_64-linux, i686-linux, aarch64-linux
  boot.binfmt.emulatedSystems = [ "aarch64-linux" ];
}
```

Run `cargo test` or `cargo nextest run` to run the integration tests, and `just test` to run them on all supported platforms (binfmt required).
