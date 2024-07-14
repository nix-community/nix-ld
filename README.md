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

Here `{system}` is the value of the Nix `system` with dashes replaced with underscores, like `x86_64_linux`.
You can also run `nix-ld-rs` directly for a list.

## Use in NixOS

```
{ pkgs, ... }: {
  programs.nix-ld.enable = true;
  programs.nix-ld.package = pkgs.nix-ld-rs;
}
```

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

### Current behavior

<table>
<thead>
  <tr>
    <th rowspan="2"></th>
    <th colspan="2">Launch</th>
    <th colspan="2">Seen by ld.so</th>
    <th colspan="2">Seen by getenv() and children <sup>(a)</sup></th>
  </tr>
  <tr>
    <th>NIX_LD_LIBRARY_PATH</th>
    <th>LD_LIBRARY_PATH</th>
    <th>NIX_LD_LIBRARY_PATH</th>
    <th>LD_LIBRARY_PATH</th>
    <th>NIX_LD_LIBRARY_PATH</th>
    <th>LD_LIBRARY_PATH</th>
  </tr>
</thead>
<tbody>
  <tr>
    <td>1</td>
    <td>(unset)</td>
    <td>(unset)</td>
    <td>(unset)</td>
    <td>"/run/current-system/sw/share/nix-ld/lib"</td>
    <td>(unset)</td>
    <td>"" <sup>(b)</sup></td>
  </tr>
  <tr>
    <td>2</td>
    <td>(unset)</td>
    <td>"/some/lib"</td>
    <td>(unset)</td>
    <td>"/some/lib:/run/current-system/sw/share/nix-ld/lib"</td>
    <td>(unset)</td>
    <td>"/some/lib"</td>
  </tr>
  <tr>
    <td>3</td>
    <td>"/some/nix/ld/lib"</td>
    <td>(unset)</td>
    <td>(unset)</td>
    <td>"/some/nix/ld/lib"</td>
    <td>"/some/nix/ld/lib"</td>
    <td>(unset)</td>
  </tr>
  <tr>
    <td>4</td>
    <td>"/some/nix/ld/lib"</td>
    <td>"/some/lib"</td>
    <td>"/some/nix/ld/lib"</td>
    <td>"/some/lib:/some/nix/ld/lib"</td>
    <td>"/some/nix/ld/lib"</td>
    <td>"/some/lib"</td>
  </tr>
</tbody>
</table>

<sup>(a)</sup> On X86-64 and AArch64 only (see `src/arch.rs`). On other platforms, the "Seen by ld.so" state will persist.<br/>
<sup>(b)</sup> The variable will be present but set to an empty string.<br/>
