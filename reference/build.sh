#!/usr/bin/env bash
# Build the nibli syllogism explainer: narration, playground capture, both cuts.
# Usage: ./build.sh [voices|tts|capture|compose [16x9|9x16|--remux]|qa|all]   (default: all)
#        ./build.sh record [--test|--only N,M|--from N]   record your own narration (interactive)
#        ./build.sh own                                   build the cuts from your takes
# The example's takes live in vidni's take store, ../examples/nibli-syllogism/takes
# (out of git). `own` lays them out as numbered files with `vidni takes --link`
# (VIDNI_BIN, or vidni on PATH); OWN_AUDIO_DIR reads another folder instead.
set -euo pipefail
cd "$(dirname "$0")"

# This session exports an LD_LIBRARY_PATH (alsa-lib) that makes nix-store
# binaries fail with a GLIBC_2.43 error.
unset LD_LIBRARY_PATH

PLAYWRIGHT_MODULE="${PLAYWRIGHT_MODULE:-$HOME/projects/dhilipsiva/nibli-learn/node_modules/playwright/index.mjs}"
BROWSER_EXECUTABLE="${BROWSER_EXECUTABLE:-$(command -v google-chrome-stable || command -v chromium || true)}"
export PLAYWRIGHT_MODULE BROWSER_EXECUTABLE

VOICE=en_US-joe-medium
VOICE_URL=https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/joe/medium
WHISPER_URL=https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin

in_shell() { nix develop . --command bash -c "unset LD_LIBRARY_PATH; $*"; }

fetch() { # url dest sha256
  [ -s "$2" ] || curl -fL --retry 3 -o "$2" "$1"
  echo "$3  $2" | sha256sum -c --quiet -
}

voices() {
  mkdir -p build/voices build/models
  fetch "$VOICE_URL/$VOICE.onnx" "build/voices/$VOICE.onnx" abf4b998e34045cbd00eaf050de2de620f81ded6eb8b85a24b022a1c2b3f08d2
  fetch "$VOICE_URL/$VOICE.onnx.json" "build/voices/$VOICE.onnx.json" 7b5f0293c0bf5bda8e377be1696da84af798befeef31038e43beefc01fe858da
  fetch "$WHISPER_URL" build/models/ggml-base.en.bin a03779c86df3323075f5e796cb2ce5029f00ec8869eee3fdfb897afe36c6d002
}

tts() { in_shell python3 -I tts.py; }
capture() { node capture.mjs; }
compose() { in_shell python3 -I compose.py "$@"; }
qa() { in_shell python3 -I qa.py; }
record() { voices; in_shell python3 -I record.py "$@"; }

# Your recordings instead of Piper; outputs are named nibli-syllogism-voice-*.
own() {
  if [ -z "${OWN_AUDIO_DIR:-}" ]; then
    OWN_AUDIO_DIR="$(mktemp -d)/takes"
    "${VIDNI_BIN:-vidni}" takes ../examples/nibli-syllogism --link "$OWN_AUDIO_DIR"
  fi
  export VARIANT=voice OWN_AUDIO_DIR
  voices
  tts
  [ -s build/capture.json ] || capture
  compose
  qa
}

case "${1:-all}" in
  voices) voices ;;
  tts) tts ;;
  capture) capture ;;
  compose) shift; compose "$@" ;;
  qa) qa ;;
  record) shift; record "$@" ;;
  own) own ;;
  all) voices; tts; capture; compose; qa ;;
  *) echo "usage: $0 [voices|tts|capture|compose [16x9|9x16|--remux]|qa|record [--test|--only N,M|--from N]|own|all]" >&2; exit 2 ;;
esac
