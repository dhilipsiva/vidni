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

# Recompute the golden outputs from the reference pipeline (P1-VIDNI-01).
golden:
    @echo "not built yet: P1-VIDNI-01 adds the golden outputs" >&2; exit 2

# Compare a Rust stage with the golden outputs (P1-VIDNI-04).
parity stage:
    @echo "not built yet: P1-VIDNI-04 adds the parity harness (stage: {{stage}})" >&2; exit 2

# Build twice under a hermetic environment and compare (P3-VIDNI-06).
repro:
    @echo "not built yet: P3-VIDNI-06 adds vidni repro" >&2; exit 2

# Render an example project (P8-VIDNI-02).
example name:
    @echo "not built yet: P8-VIDNI-02 adds the examples ({{name}})" >&2; exit 2
