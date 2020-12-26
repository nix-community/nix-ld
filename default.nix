{ pkgs ? import <nixpkgs> {} }:
pkgs.callPackage ./nix-ld.nix {}
