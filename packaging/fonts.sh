#!/usr/bin/env bash
# packaging/fonts.sh [folder]: fetches the fonts vidni draws with into a
# folder, by default the one vidni looks in when VIDNI_FONTS isn't set
# (~/.local/share/vidni/fonts). For a system with no Nix and no portable
# bundle: macOS and Windows.
#
# vidni pins each font file by its SHA-256 (crates/vidni-core/src/fonts.toml)
# and refuses a file with other bytes, so this needs no check of its own:
# a wrong download is said by the first build.
set -euo pipefail
to=${1:-${XDG_DATA_HOME:-$HOME/.local/share}/vidni/fonts}
work=$(mktemp -d)
trap 'rm -r "$work"' EXIT
mkdir -p "$to"
fetch() { curl --fail --location --silent --show-error --retry 3 --output "$work/$1" "$2"; }

# IBM Plex Sans and Mono: the default theme's faces.
fetch sans.zip "https://github.com/IBM/plex/releases/download/@ibm/plex-sans@1.1.0/ibm-plex-sans.zip"
fetch mono.zip "https://github.com/IBM/plex/releases/download/%40ibm/plex-mono%402.5.0/ibm-plex-mono.zip"
# DejaVu Sans Mono: terminal and code scenes, and an overlay's signs.
fetch dejavu.tar.bz2 "https://github.com/dejavu-fonts/dejavu-fonts/releases/download/version_2_37/dejavu-fonts-ttf-2.37.tar.bz2"
# Space Grotesk: the display face of the QUINE theme.
fetch grotesk.zip "https://github.com/floriankarsten/space-grotesk/releases/download/2.0.0/SpaceGrotesk-2.0.0.zip"

(cd "$work" && unzip -q sans.zip -d sans && unzip -q mono.zip -d mono && unzip -q grotesk.zip -d grotesk \
  && mkdir dejavu && tar -xjf dejavu.tar.bz2 -C dejavu)
for name in IBMPlexSans-Bold.otf IBMPlexMono-Bold.otf DejaVuSansMono.ttf DejaVuSansMono-Bold.ttf SpaceGrotesk-Bold.otf; do
  found=$(find "$work" -name "$name" | grep -v -i -e unhinted -e split | head -n 1)
  if [ -z "$found" ]; then echo "$name wasn't in what was fetched" >&2; exit 1; fi
  cp "$found" "$to/$name"
done
echo "fonts are in $to"
