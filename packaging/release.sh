#!/usr/bin/env bash
# packaging/release.sh [--rebuild]: builds what a release of vidni is
# made of into release/v<version>/ (P8-VIDNI-05):
#
#   vidni-<version>-x86_64-linux.tar.gz   the portable bundle
#   vidni-tools-x86_64-linux.tar.zst      the tools `vidni setup` fetches
#   SOURCES.txt                           what both were built from
#   SHA256SUMS                            the three files' SHA-256
#
# Every file is built by Nix from the committed tree and the pinned
# nixpkgs, so two runs give the same bytes. `--rebuild` proves it: each
# archive is built a second time, and the build fails if the second
# differs from the first.
#
# The signatures are GitHub's build-provenance attestations, which the
# release workflow makes of these files (DECISIONS.md, R5).
set -euo pipefail
here=$(cd "$(dirname "$0")/.." && pwd)
cd "$here"
rebuild=
case ${1:-} in
  --rebuild) rebuild=1 ;;
  "") ;;
  *) echo "usage: packaging/release.sh [--rebuild]" >&2; exit 2 ;;
esac

version=$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)
case $version in
  [0-9]*.[0-9]*.[0-9]*) ;;
  *) echo "Cargo.toml's version, \"$version\", isn't MAJOR.MINOR.PATCH" >&2; exit 1 ;;
esac
# A release is of a commit: what isn't committed isn't in it.
if [ -n "$(git status --porcelain)" ]; then
  echo "the tree has changes that aren't committed; commit them, or stash them" >&2
  exit 1
fi
# And it says what changed.
if ! grep -q "^## \[$version\]" CHANGELOG.md; then
  echo "CHANGELOG.md has no section for $version" >&2
  exit 1
fi

echo "== building vidni $version at $(git rev-parse --short HEAD)"
bundle=$(nix build .#bundle --no-link --print-out-paths)
if [ -n "$rebuild" ]; then
  echo "== building each part again, and holding it to the first"
  for part in vidni bundle-tools bundle; do
    nix build ".#$part" --rebuild --no-link
    echo "   $part: the same"
  done
fi

out=release/v$version
mkdir -p release
fresh=$(mktemp -d release/.v$version.XXXXXX)
cp -L "$bundle"/* "$fresh/"
chmod 644 "$fresh"/*
(cd "$fresh" && LC_ALL=C sha256sum -- * > SHA256SUMS)
# The folder from an earlier run is replaced whole.
if [ -e "$out" ]; then
  old=$(mktemp -d release/.old.XXXXXX)
  mv "$out" "$old/"
  rm -r "$old"
fi
mv "$fresh" "$out"
echo "== $out"
cat "$out/SHA256SUMS"
