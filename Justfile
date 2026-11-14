# vidni development tasks. Run `just` to list them.
# Recurring commands live here (DECISIONS D11); the product's own commands are `vidni ...`.

set shell := ["bash", "-euo", "pipefail", "-c"]

# List the recipes.
default:
    @just --list

# Build every crate and target.
build:
    cargo build --workspace --all-targets

# Run every test.
test:
    cargo test --workspace --all-targets

# Regenerate docs/vidni.schema.json, the JSON Schema of vidni.toml, from the project model.
schema:
    cargo run -q -p vidni-cli -- schema > docs/vidni.schema.json

# Check formatting, and run clippy with warnings denied.
lint:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings

# Rewrite the code in rustfmt's layout.
fmt:
    cargo fmt --all

# Run the desktop app: `just app`, or `just app examples/hello` to open a project. Its window
# and its drawing find their libraries when it starts.
app *args:
    LD_LIBRARY_PATH="$VIDNI_WINDOW_LIBS${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" VIDNI_VOICES="${VIDNI_VOICES:-$VIDNI_PINNED_VOICES}" cargo run -p vidni-app -- {{args}}

# Open the app's window where nobody sees it, in a headless Wayland compositor drawn in
# software, and save a picture of it after five seconds: `just app-shot window.png`, or
# `just app-shot window.png examples/hello --script` for a project's script. It is how the
# window is looked at on a machine with no screen to spare.
app-shot out="app.png" *args:
    #!/usr/bin/env bash
    set -u
    cargo build -q -p vidni-app || exit 1
    run="$(mktemp -d /tmp/vidni-shot.XXXX)"; chmod 700 "$run"; trap 'rm -rf "$run"' EXIT
    export XDG_RUNTIME_DIR="$run"
    nix shell --inputs-from . nixpkgs#weston -c bash -c '
      weston --backend=headless --renderer=pixman --debug --socket=wl-vidni --width=1000 --height=760 >"$XDG_RUNTIME_DIR/weston.log" 2>&1 &
      compositor=$!; sleep 2
      (WAYLAND_DISPLAY=wl-vidni DISPLAY= VIDNI_CONFIG="${VIDNI_CONFIG:-$XDG_RUNTIME_DIR/settings}" LD_LIBRARY_PATH="$VIDNI_WINDOW_LIBS" timeout 12 target/debug/vidni-app {{args}} >"$XDG_RUNTIME_DIR/app.log" 2>&1 &)
      sleep 5; (cd "$XDG_RUNTIME_DIR" && WAYLAND_DISPLAY=wl-vidni weston-screenshooter); kill $compositor'
    shot="$(ls "$run"/wayland-screenshot-*.png 2>/dev/null | head -1)"
    if [ -n "$shot" ]; then cp "$shot" "{{out}}" && echo "{{out}}: the app's window"; else cat "$run/app.log" >&2; exit 1; fi

# The app's tests that draft a project (P7-VIDNI-01): they need Piper and its voice, Chromium
# and ffmpeg, as the dev shell has them. The rest of the app's tests run in `just test`.
test-app:
    VIDNI_PREVIEW_MUTE=1 VIDNI_VOICES="${VIDNI_VOICES:-$VIDNI_PINNED_VOICES}" cargo test -p vidni-app --test app -- --ignored
    VIDNI_PREVIEW_MUTE=1 cargo test -p vidni-app --test preview -- --ignored
    VIDNI_VOICES="${VIDNI_VOICES:-$VIDNI_PINNED_VOICES}" cargo test -p vidni-app --test notes -- --ignored
    cargo build -p vidni-cli
    VIDNI_BIN="$PWD/target/debug/vidni" VIDNI_VOICES="${VIDNI_VOICES:-$VIDNI_PINNED_VOICES}" cargo test -p vidni-app --test gates -- --ignored
    VIDNI_BIN="$PWD/target/debug/vidni" cargo test -p vidni-app --test interchange -- --ignored
    VIDNI_VOICES="${VIDNI_VOICES:-$VIDNI_PINNED_VOICES}" cargo test -p vidni-app --test export -- --ignored

# Build the portable bundle for a Linux with no Nix (P8-VIDNI-03): vidni's own archive and
# the tools' archive, which `vidni setup` fetches. Prints the folder that holds the two.
bundle:
    nix build .#bundle --no-link --print-out-paths

# Follow the bundle's README in a clean container with no Nix, as a user who isn't root, and
# hold what it drafts to what the flake's own tools draft. Needs docker and the network.
test-bundle image="debian:stable-slim":
    packaging/test.sh {{image}}

# Build the flake's checks (the workspace build and its tests).
check:
    nix flake check

# Where the reference pipeline lives. Its flake is entered by path, so inside this
# checkout Nix sees only tracked files and never copies build/ or out/ to the store.
reference := env_var_or_default("VIDNI_REFERENCE", "reference")

# Re-derive the reference's golden outputs and compare them with golden/ (about 5 minutes).
golden:
    cargo build -q -p vidni-cli
    ref="$(realpath {{reference}})"; env -u LD_LIBRARY_PATH VIDNI_BIN="$PWD/target/debug/vidni" nix develop "$ref" --command python3 -I tools/golden.py verify "$ref"

# Re-derive the golden outputs and overwrite golden/.
golden-update:
    cargo build -q -p vidni-cli
    ref="$(realpath {{reference}})"; env -u LD_LIBRARY_PATH VIDNI_BIN="$PWD/target/debug/vidni" nix develop "$ref" --command python3 -I tools/golden.py update "$ref"

# Run the reference's tests (reference/tests) in its nix shell, in a network namespace with
# no network and with no PipeWire or Pulse socket reachable, so no microphone (P1-VIDNI-05).
test-reference:
    cargo build -q -p vidni-cli
    ref="$(realpath {{reference}})"; tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT; target/debug/vidni takes examples/nibli-syllogism --cut long --link "$tmp/takes" >/dev/null; env -u LD_LIBRARY_PATH RECORDINGS_DIR="$tmp/takes" nix develop "$ref" --command bash -c 'unset LD_LIBRARY_PATH; cd "$1"; run="$(mktemp -d)"; trap "rm -rf $run" EXIT; env -u PIPEWIRE_REMOTE -u PULSE_SERVER XDG_RUNTIME_DIR="$run" unshare -rn python3 -I -B -m unittest discover -s tests -v' _ "$ref"

# Check every example's out-of-git assets (takes) against its assets.lock.
assets-check:
    for lock in examples/*/assets.lock; do (cd "$(dirname "$lock")" && sha256sum -c --quiet --strict assets.lock) && echo "$lock: every file matches"; done

# Compare a Rust stage's outputs with the golden outputs (timeline, narration, encode,
# cuts, stills or all). With no candidate, the stage's Rust port builds one, as in
# `just parity timeline`; `--candidate DIR` and `--reference-self` pick another.
# `just parity takes` trims the example's raw takes as the recorder does, and compares
# each with the take the reference's recorder kept.
# A release build: the cuts stage renders four cuts of some 9,000 frames each.
# Golden was made with the host's fonts, so the dev shell's pinned fontconfig is set aside.
parity stage *args:
    env -u FONTCONFIG_FILE cargo run -q --release -p vidni-parity -- {{stage}} {{args}}

# Every parity stage in turn, the encoder's own check, and the harness's test: about half an
# hour (P3-VIDNI-07). The encoder is held to T2 and not to the reference's packets, since its
# flags differ on purpose, so `encode-t2` stands for the encode stage.
parity-all:
    for stage in timeline narration cuts stills takes product; do just parity "$stage" || exit 1; done
    just encode-t2
    just parity-selftest

# The reference is retired and stays as it is: fail if a file under reference/ differs from
# the ones listed in golden/reference.sha256 (P3-VIDNI-07). `just reference-frozen update`
# lists them again, for a change that is meant.
reference-frozen mode="check":
    #!/usr/bin/env bash
    now="$(git ls-files -s reference | sha256sum | cut -d' ' -f1)"
    if [ "{{mode}}" = update ]; then echo "$now" > golden/reference.sha256; echo "golden/reference.sha256: $now"; exit 0; fi
    if [ "$now" = "$(cat golden/reference.sha256)" ]; then echo "reference/: unchanged since it was retired"; else echo "reference/ has changed since it was retired; it is read-only (just reference-frozen update, if the change is meant)" >&2; exit 1; fi

# Show that the parity harness passes the reference itself and catches a perturbed output per stage.
parity-selftest:
    env -u FONTCONFIG_FILE cargo run -q --release -p vidni-parity -- selftest

# Render every cut twice with the Rust compositor and compare the frame streams (T1,
# P3-VIDNI-02). The fonts are the reference's, found by fc-match, unless `--fonts` names them.
compose-t1 *args:
    cargo run -q --release -p vidni-parity -- compose-t1 {{args}}

# Compare every picture of every cut with the reference's own, from compose.py in the
# reference's nix shell: the frames where the picture changes, and every pixel (P3-VIDNI-02).
compose-stream *args:
    cargo run -q --release -p vidni-parity -- compose-stream {{args}}

# The capture runtime's tests on a local page; they need Chromium, as the dev shell has (P3-VIDNI-04).
test-capture:
    cargo test -p vidni-media --test capture -- --ignored

# The listening check's tests with whisper-cli and its model, as the dev shell has them
# (P3-VIDNI-05): the library on its own, and `vidni hear` on a small project.
test-hear:
    cargo test -p vidni-media -p vidni-cli --test hear -- --ignored
    cargo test -p vidni-cli --test record the_review_says -- --ignored

# The microphone's path through cpal, on ALSA's null device, which gives zeros (P4-VIDNI-01),
# and `vidni record` in a pseudo-terminal, recording a made-up input (P4-VIDNI-02). Nothing
# is heard and nobody is recorded.
test-record:
    cargo test -p vidni-record --test mic -- --ignored
    cargo test -p vidni-cli --test record -- --ignored --skip the_review_says

# Capture the example's web scene five times, each from scratch, and compare every still's
# pixels. It reads the live page over the network, so it measures that page as much as vidni
# (docs/determinism.md); `just test-capture` checks five captures of a local scene.
# VIDNI_KEEP_RUNS=DIR keeps the five captures in DIR, to compare them by hand.
capture-repeat *args:
    cargo run -q --release -p vidni-parity -- capture-repeat {{args}}

# Encode the golden 16:9 frames and narration twice with the Rust encoder, and compare the
# packets and bytes (T2, P3-VIDNI-01).
encode-t2:
    dir="$(mktemp -d)"; trap 'rm -rf "$dir"' EXIT; cargo run -q --release -p vidni-media --example t2_golden -- {{reference}}/out/nibli-syllogism-16x9.mp4 {{reference}}/build/narration.wav "$dir"

# A small project from draft to approved through the `vidni` command, with `vidni repro` and
# `--offline` (P3-VIDNI-06). It drafts with Piper and the pinned voice, captures with
# Chromium, and needs unshare and taskset, as the dev shell has them.
test-build:
    VIDNI_VOICES="${VIDNI_VOICES:-$VIDNI_PINNED_VOICES}" cargo test -p vidni-cli --test build -- --ignored

# Build an example's cuts twice from copies of it, in clean environments, at two paths and on
# two sets of cores, and compare every hash (P3-VIDNI-06). `just repro nibli-syllogism --draft`
# builds the drafts instead of the final cuts, which need the creator's takes.
repro example="nibli-syllogism" *args:
    cargo run -q --release -p vidni-cli -- repro examples/{{example}} {{args}}

# Write docs/keys.md, the reference of every key in vidni.toml, again from the schema
# (P8-VIDNI-01). `just keys check` fails when the page is out of date.
keys mode="write":
    python3 -I tools/keys.py {{mode}}

# Write THIRD-PARTY-NOTICES.md again from what vidni pins: its crates, fonts, voices, tools and
# models (P8-VIDNI-04). `just notices check` fails when the file is out of date.
notices mode="write":
    python3 -I tools/notices.py {{mode}}

# Draft an example project with no network: `just example hello` (P8-VIDNI-02). The nibli
# example shows a page on the web, so it drafts offline only once its captures are taken.
example name *args:
    VIDNI_VOICES="${VIDNI_VOICES:-$VIDNI_PINNED_VOICES}" cargo run -q --release -p vidni-cli -- draft examples/{{name}} --offline {{args}}
