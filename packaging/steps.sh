#!/bin/sh
# The steps of the bundle's README.txt, as someone on a Linux with no Nix
# would type them. `packaging/test.sh` runs this in a clean container,
# as a user who isn't root. The release's two files are in /release.
set -e
cd "$HOME"
tar xzf /release/vidni-*-x86_64-linux.tar.gz
cd vidni-*-x86_64-linux
bundle=$PWD
export PATH="$bundle/bin:$PATH"
vidni --version

# `vidni setup` fetches the tools. There is no release to fetch them from
# yet, so the release's folder stands in for its address: the same
# download, check, unpack and move.
vidni setup --mirror file:///release
# A second run finds everything in place, and fetches nothing.
vidni setup --mirror file:///nowhere | tee "$HOME/second.log"
grep -q '^the tools are set up already' "$HOME/second.log"
if grep -q '^fetching' "$HOME/second.log"; then exit 1; fi

# In a container Chromium can't make its own sandbox.
export VIDNI_NO_SANDBOX=1
cd "$HOME"
vidni new my-video
vidni draft my-video
cp -r "$bundle/share/vidni/examples/hello" .
vidni draft hello
vidni hear hello | tail -n 3

# What is missing is fetched again, by the tools' own curl: the
# listening model here, and another voice.
rm "$HOME/.cache/vidni/models/ggml-base.en.bin"
vidni setup --mirror file:///nowhere | tee "$HOME/third.log"
grep -q '^fetching the listening model' "$HOME/third.log"
vidni voices fetch piper:en_US-ryan-medium

# The tools from a folder, with nothing downloaded: the other way in.
rm -r "$HOME/.cache/vidni/tools"
vidni setup --from /release
vidni draft my-video --again >/dev/null

# What was made, and what the bundle carries, for the one who runs the
# test to compare.
for file in my-video/build/draft/long.build.json my-video/build/draft/long.mp4 \
  my-video/build/draft/vertical.mp4 hello/build/draft/long.mp4; do
  sum=$(sha256sum "$file")
  echo "made $sum"
done
for file in LICENSE-MIT LICENSE-APACHE THIRD-PARTY-NOTICES.md; do
  sum=$(sha256sum < "$bundle/share/doc/vidni/$file")
  echo "carries $file $sum"
done
