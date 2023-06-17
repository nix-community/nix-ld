# nix-ld-rs

RIIR of [nix-ld](https://github.com/Mic92/nix-ld).
To be upstreamed at some point?

## Testing

```
cargo b
cp --no-preserve=mode -L $(which coreutils) ./
chmod u+w ./coreutils
patchelf --set-interpreter $PWD/target/debug/nix-ld-rs ./coreutils
NIX_LD_LOG=trace ./coreutils
NIX_LD= NIX_LD_LOG=trace ./coreutils
```
