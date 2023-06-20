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

## Testing

```
cargo b
cp -L $(which coreutils) ./
chmod u+w ./coreutils
patchelf --set-interpreter $PWD/target/debug/nix-ld-rs ./coreutils
NIX_LD_LOG=trace ./coreutils
NIX_LD= NIX_LD_LOG=trace ./coreutils
```
