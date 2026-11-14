#!/usr/bin/env bash
# packaging/test.sh [image]: builds the portable bundle, and follows its
# README in a clean container of a Linux with no Nix, as a user who isn't
# root (P8-VIDNI-03). Then it drafts the same new project with the flake's
# own tools, and holds the container's files to those.
#
# It needs docker and the network (the voice and the listening model are
# fetched inside the container), and about 5 GB of disk.
set -euo pipefail
image=${1:-debian:stable-slim}
here=$(cd "$(dirname "$0")/.." && pwd)
work=$(mktemp -d "${TMPDIR:-/tmp}/vidni-bundle.XXXXXX")
volume=vidni-bundle-test-$$
trap 'docker volume rm -f "$volume" >/dev/null 2>&1; chmod -R u+w "$work"; rm -r "$work"' EXIT

echo "== building the bundle"
bundle=$(nix build "$here#bundle" --no-link --print-out-paths)
mkdir "$work/release"
cp -L "$bundle"/* "$work/release/"
ls -l "$work/release"

echo "== the README's steps, in $image"
docker volume create "$volume" >/dev/null
docker run --rm -v "$volume:/home/someone" "$image" chown 1000:1000 /home/someone
docker run --rm --user 1000:1000 -e HOME=/home/someone \
  -v "$volume:/home/someone" -v "$work/release:/release:ro" \
  -v "$here/packaging/steps.sh:/steps.sh:ro" "$image" \
  sh -c 'if [ -e /nix ]; then echo "this image has Nix"; exit 1; fi; sh /steps.sh' \
  | tee "$work/container.log" | grep -v ' of [0-9]* frames$'
# The bundle carries the two licences and the notices, as they are here.
for file in LICENSE-MIT LICENSE-APACHE THIRD-PARTY-NOTICES.md; do
  sum=$(sha256sum < "$here/$file")
  grep -qxF "carries $file $sum" "$work/container.log" \
    || { echo "the bundle's $file isn't the repository's"; exit 1; }
done
[ "$(grep -c '^made ' "$work/container.log")" = 4 ] || { echo "a draft is missing"; exit 1; }

echo "== the same new project, with the flake's own tools"
(
  cd "$work"
  vidni() { nix run "$here" -- "$@"; }
  # Nothing of this machine's own vidni: its voices, its cache, its data.
  unset VIDNI_VOICES XDG_DATA_HOME XDG_CACHE_HOME XDG_CONFIG_HOME
  export HOME="$work/home" && mkdir "$HOME"
  vidni voices fetch piper:en_US-joe-medium >/dev/null
  vidni new my-video >/dev/null
  vidni draft my-video >/dev/null
  cp -r "$here/examples/hello" . && rm -rf hello/build
  vidni draft hello >/dev/null
  for file in my-video/build/draft/long.build.json my-video/build/draft/long.mp4 \
    my-video/build/draft/vertical.mp4 hello/build/draft/long.mp4; do
    sum=$(sha256sum "$file")
    echo "made $sum"
  done
) > "$work/flake.log"
if diff <(grep '^made ' "$work/container.log") <(grep '^made ' "$work/flake.log"); then
  echo "== the container's drafts are the flake's, byte for byte"
else
  echo "== the container's drafts differ from the flake's"
  exit 1
fi
