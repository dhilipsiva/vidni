# vidni

An open, deterministic studio for narrated tutorial and explainer videos.

*vidni* is Lojban for "video screen".

You write one `vidni.toml` per video. From it, vidni renders:
- a long 16:9 video and any number of 9:16 shorts
- captions, chapters and thumbnails
- an upload folder for each platform

You draft with a synthetic voice, then finish with your own recorded voice.

vidni has one Rust core and two front ends: the `vidni` command line and a native desktop app built with egui. Every step works without an LLM.

**To try it:** [docs/install.md](docs/install.md) installs vidni on Linux, with Nix or without, and [docs/quickstart.md](docs/quickstart.md) goes from there to a draft video.

| Doc | What it is |
|---|---|
| [install.md](docs/install.md) | Installing vidni on Linux: `nix run`, or the portable bundle and `vidni setup` |
| [releasing.md](docs/releasing.md) | Versions, the changelog, and how a release is built and checked |
| [quickstart.md](docs/quickstart.md) | From a clone to your first draft, and the steps from there to a finished video |
| [spec.md](docs/spec.md) | How a project file works, with every command |
| [keys.md](docs/keys.md) | Every key of `vidni.toml`, generated from the schema |
| [recording.md](docs/recording.md) | Recording your narration, and keeping takes consistent |
| [platforms.md](docs/platforms.md) | What each platform takes, and what to check before uploading |
| [determinism.md](docs/determinism.md) | What repeats, where, and how it was measured |
| [app.md](docs/app.md) | The desktop app, as far as it is built |
| [assistant.md](docs/assistant.md) | What an LLM may do in a project, and how vidni holds it to that |
| [faq.md](docs/faq.md) | Short answers |

**To help:** [CONTRIBUTING.md](CONTRIBUTING.md) says how to set up and what a change needs, and everyone taking part follows the [code of conduct](CODE_OF_CONDUCT.md). Issues start from a template: something that is wrong, or an idea. [CHANGELOG.md](CHANGELOG.md) lists what each release changed.

**Status:** being built, item by item, from [`TODO.md`](TODO.md). The decisions behind it are in [`DECISIONS.md`](DECISIONS.md). So far:

- `vidni new` starts a project that renders as it is, from an outline when you have one.
- `vidni validate` checks a project file against [schema 1](docs/spec.md), and `vidni fmt` puts it in the canonical layout.
- `vidni takes` tracks the recorded takes, and `vidni plan` prints the recording sheet.
- `vidni record` records your takes in the terminal, one line at a time, with single keys. It checks and trims each take, and picks up where the last session stopped.
- `vidni voices` lists the draft voices with their licences, and `vidni speak` speaks every line in the draft voice, seeded so that it repeats.
- `vidni capture` captures web scenes and page scenes through Chromium, as the cuts need them.
- Image scenes put a PNG or an SVG under the virtual camera, with no browser (`examples/images`).
- Card scenes are drawn from five templates (title, section, bullets, quote, end) in the project's theme. The default theme is built in; `examples/themes/quine` is the nibli example's.
- `vidni hear` listens to each line's audio with whisper and flags a line that doesn't sound like its script. It only advises.
- `vidni draft` renders each cut in the draft voice, to watch before recording. Drafted again with nothing changed, it renders nothing.
- One script makes several cuts: a long cut and shorts, each picking its lines, with lines that are only for some (`only`).
- A 9:16 cut is laid out clear of what its platforms' apps cover, QA lists what an app would hide, and every build writes review frames with the covered edges shaded.
- A 16:9 cut for YouTube is a clean master with caption files, a captioned variant, and a chapter list from its sections.
- `vidni ratify` and `vidni status` freeze cuts.
- `vidni final` builds each ratified cut from your takes, `vidni approve` records it as a release, and `vidni repro` checks that a build repeats.
- `vidni backup` copies the takes and frozen captures to two places and lists them in `assets.lock`; `vidni verify` checks them, `vidni restore` brings them back, and `vidni gc` deletes what can be made again.

The nibli example goes from draft to approved through these commands ([docs/spec.md](docs/spec.md#from-draft-to-release)). The desktop app has its shell: projects, each cut's standing, and drafting as a job ([docs/app.md](docs/app.md)).

## Layout

| Path | What it holds |
|---|---|
| `crates/` | The Rust workspace: `vidni-core`, `vidni-media`, `vidni-record`, `vidni-cli` and `vidni-app`, plus `vidni-parity`, the dev tool that compares a Rust stage with the golden outputs (`just parity`) |
| `reference/` | The Python and Node pipeline that made the first video. Each Rust stage must match its golden outputs before it replaces its counterpart. |
| `golden/` | The reference's golden outputs: hashes, timelines, captions and framemd5 logs |
| `examples/` | Example projects. `hello/` is a one-minute video of cards and a picture that drafts with no network (`just example hello`). `nibli-syllogism/` is the first video; its takes are kept out of git and listed in `assets.lock`. |
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

vidni runs other programs and draws with other people's fonts and voices. [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md) lists each with its licence, and says what the GPL programs among them (ffmpeg with x264, Piper, eSpeak NG) ask of anyone who passes them on. `just notices` writes it again.
