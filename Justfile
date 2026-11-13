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
    ref="$(realpath {{reference}})"; env -u LD_LIBRARY_PATH nix develop "$ref" --command python3 -I tools/golden.py verify "$ref"

# Re-derive the golden outputs and overwrite golden/.
golden-update:
    ref="$(realpath {{reference}})"; env -u LD_LIBRARY_PATH nix develop "$ref" --command python3 -I tools/golden.py update "$ref"

# Run the reference's tests (reference/tests) in its nix shell, in a network namespace with
# no network and with no PipeWire or Pulse socket reachable, so no microphone (P1-VIDNI-05).
test-reference:
    ref="$(realpath {{reference}})"; env -u LD_LIBRARY_PATH nix develop "$ref" --command bash -c 'unset LD_LIBRARY_PATH; cd "$1"; run="$(mktemp -d)"; trap "rm -rf $run" EXIT; env -u PIPEWIRE_REMOTE -u PULSE_SERVER XDG_RUNTIME_DIR="$run" unshare -rn python3 -I -B -m unittest discover -s tests -v' _ "$ref"

# Check every example's out-of-git assets (takes) against its assets.lock.
assets-check:
    for lock in examples/*/assets.lock; do (cd "$(dirname "$lock")" && sha256sum -c --quiet --strict assets.lock) && echo "$lock: every file matches"; done

# Compare a Rust stage's outputs with the golden outputs (timeline, narration, encode,
# cuts, stills or all): `just parity cuts --candidate DIR`, or `--reference-self`.
parity stage *args:
    cargo run -q -p vidni-parity -- {{stage}} {{args}}

# Show that the parity harness passes the reference itself and catches a perturbed output per stage.
parity-selftest:
    cargo run -q -p vidni-parity -- selftest

# Build twice under a hermetic environment and compare (P3-VIDNI-06).
repro:
    @echo "not built yet: P3-VIDNI-06 adds vidni repro" >&2; exit 2

# Render an example project (P8-VIDNI-02).
example name:
    @echo "not built yet: P8-VIDNI-02 adds the examples ({{name}})" >&2; exit 2
