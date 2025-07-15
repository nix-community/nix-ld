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

waitForPr() {
  local pr=$1
  while true; do
    if gh pr view "$pr" | grep -q 'MERGED'; then
      break
    fi
    echo "Waiting for PR to be merged..."
    sleep 5
  done
}

# ensure we are up-to-date
uncommitted_changes=$(git diff --compact-summary)
if [[ -n "$uncommitted_changes" ]]; then
  echo -e "There are uncommitted changes, exiting:\n${uncommitted_changes}" >&2
  exit 1
fi
git pull git@github.com:nix-community/nix-ld main
unpushed_commits=$(git log --format=oneline origin/main..main)
if [[ "$unpushed_commits" != "" ]]; then
  echo -e "\nThere are unpushed changes, exiting:\n$unpushed_commits" >&2
  exit 1
fi
# make sure tag does not exist
if git tag -l | grep -q "^${version}\$"; then
  echo "Tag ${version} already exists, exiting" >&2
  exit 1
fi
sed -i -e "s!^version = \".*\"\$!version = \"${version}\"!" Cargo.toml
cargo build
git add Cargo.lock Cargo.toml
git branch -D "release-${version}" || true
git checkout -b "release-${version}"
nix flake check -vL
git commit -m "bump version nix-ld ${version}"
git push origin "release-${version}"
pr_url=$(gh pr create \
  --base main \
  --head "release-${version}" \
  --title "Release ${version}" \
  --body "Release ${version} of nix-ld")

# Extract PR number from URL
pr_number=$(echo "$pr_url" | grep -oE '[0-9]+$')

# Enable auto-merge with specific merge method and delete branch
gh pr merge "$pr_number" --auto --merge
git checkout main

waitForPr "release-${version}"
git pull git@github.com:nix-community/nix-ld main
git tag "${version}"
git push origin "${version}"
gh release create "${version}" --draft --title "${version}" --notes ""
