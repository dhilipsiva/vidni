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

# Run the desktop app.
app:
    cargo run -p vidni-app

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
    ref="$(realpath {{reference}})"; tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT; target/debug/vidni takes examples/nibli-syllogism --link "$tmp/takes" >/dev/null; env -u LD_LIBRARY_PATH RECORDINGS_DIR="$tmp/takes" nix develop "$ref" --command bash -c 'unset LD_LIBRARY_PATH; cd "$1"; run="$(mktemp -d)"; trap "rm -rf $run" EXIT; env -u PIPEWIRE_REMOTE -u PULSE_SERVER XDG_RUNTIME_DIR="$run" unshare -rn python3 -I -B -m unittest discover -s tests -v' _ "$ref"

# Check every example's out-of-git assets (takes) against its assets.lock.
assets-check:
    for lock in examples/*/assets.lock; do (cd "$(dirname "$lock")" && sha256sum -c --quiet --strict assets.lock) && echo "$lock: every file matches"; done

# Compare a Rust stage's outputs with the golden outputs (timeline, narration, encode,
# cuts, stills or all). With no candidate, the stage's Rust port builds one, as in
# `just parity timeline`; `--candidate DIR` and `--reference-self` pick another.
# A release build: the cuts stage renders four cuts of some 9,000 frames each.
# Golden was made with the host's fonts, so the dev shell's pinned fontconfig is set aside.
parity stage *args:
    env -u FONTCONFIG_FILE cargo run -q --release -p vidni-parity -- {{stage}} {{args}}

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

# Render an example project (P8-VIDNI-02).
example name:
    @echo "not built yet: P8-VIDNI-02 adds the examples ({{name}})" >&2; exit 2
