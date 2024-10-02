#!/usr/bin/env bash

set -eu -o pipefail

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null && pwd )"
cd "$SCRIPT_DIR/.."

version=${1:-}
if [[ -z "$version" ]]; then
  echo "USAGE: $0 version" >&2
  exit 1
fi

if [[ "$(git symbolic-ref --short HEAD)" != "main" ]]; then
  echo "must be on main branch" >&2
  exit 1
fi

# ensure we are up-to-date
uncommited_changes=$(git diff --compact-summary)
if [[ -n "$uncommited_changes" ]]; then
  echo -e "There are uncommited changes, exiting:\n${uncommited_changes}" >&2
  exit 1
fi
git pull git@github.com:nix-community/nix-ld main
unpushed_commits=$(git log --format=oneline origin/main..main)
if [[ "$unpushed_commits" != "" ]]; then
  echo -e "\nThere are unpushed changes, exiting:\n$unpushed_commits" >&2
  exit 1
fi
sed -i -e "s!^version = \".*\"\$!version = \"${version}\"!" Cargo.toml
cargo build
git add Cargo.lock
nix flake check -vL
git commit -m "bump version harmonia-v${version}"
git tag "${version}"

echo "now run 'git push --tags origin main'"
