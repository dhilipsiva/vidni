# vidni

An open, deterministic studio for narrated tutorial and explainer videos.

*vidni* is Lojban for "video screen".

You write one `vidni.toml` per video. From it, vidni renders:
- a long 16:9 video and any number of 9:16 shorts
- captions, chapters and thumbnails
- an upload folder for each platform

You draft with a synthetic voice, then finish with your own recorded voice.

vidni has one Rust core and two front ends: the `vidni` command line and a native desktop app built with egui. Every step works without an LLM.

**Status:** being built, item by item, from [`TODO.md`](TODO.md). The decisions behind it are in [`DECISIONS.md`](DECISIONS.md). So far:

- `vidni validate` checks a project file against [schema 1](docs/spec.md), and `vidni fmt` puts it in the canonical layout.
- `vidni takes` tracks the recorded takes.
- `vidni voices` lists the draft voices with their licences, and `vidni draft` speaks every line in the draft voice, seeded so that a redraft repeats it.
- `vidni capture` captures web scenes and page scenes through Chromium, as the cuts need them.
- `vidni ratify` and `vidni status` freeze cuts.

Nothing renders yet.

## Layout

| Path | What it holds |
|---|---|
| `crates/` | The Rust workspace: `vidni-core`, `vidni-media`, `vidni-record`, `vidni-cli` and `vidni-app`, plus `vidni-parity`, the dev tool that compares a Rust stage with the golden outputs (`just parity`) |
| `reference/` | The Python and Node pipeline that made the first video. Each Rust stage must match its golden outputs before it replaces its counterpart. |
| `golden/` | The reference's golden outputs: hashes, timelines, captions and framemd5 logs |
| `examples/` | Example projects. `nibli-syllogism/` is the first video; its takes are kept out of git and listed in `assets.lock`. |
| `tools/` | Development scripts |

## Develop

```sh
nix develop     # the pinned toolchain and the media tools
just            # list the tasks
just lint test  # what CI runs
```

The reference checks (`just golden`, `just parity …` and `just test-reference`) also need the example's takes and the reference's build files, which are kept out of git. See [`examples/nibli-syllogism/README.md`](examples/nibli-syllogism/README.md).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
