#!/usr/bin/env bash
# packaging/package.sh <name>: packs the vidni command that cargo built
# (target/release) for macOS or Windows into dist/<name>/ and an archive
# of it: the program, the licences and notices, how to set it up, and the
# hello example (P9-VIDNI-02). `name` is as vidni-0.1.0-aarch64-macos.
#
# These builds aren't signed: README.txt says what the system will ask.
set -euo pipefail
name=$1
here=$(cd "$(dirname "$0")/.." && pwd)
cd "$here"
program=target/release/vidni
[ -f "$program.exe" ] && program=$program.exe
out=dist/$name
mkdir -p "$out/bin" "$out/examples/hello" "$out/setup"
cp "$program" "$out/bin/"
cp LICENSE-MIT LICENSE-APACHE THIRD-PARTY-NOTICES.md "$out/"
cp packaging/README-other-systems.txt "$out/README.txt"
cp packaging/fonts.sh packaging/seed-piper.py "$out/setup/"
cp examples/hello/README.md examples/hello/flow.svg examples/hello/vidni.toml examples/hello/voice.lock "$out/examples/hello/"
case $program in
  *.exe) (cd dist && 7z a -bso0 "$name.zip" "$name") ;;
  *) tar -czf "dist/$name.tar.gz" -C dist "$name" ;;
esac
ls -l dist
