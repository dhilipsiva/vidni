# vidni TODO: an open studio for narrated tutorial videos

Maintainer: dhilipsiva. Written 2026-11-13. *vidni* is Lojban for "video screen".

vidni starts from the pipeline that made the nibli syllogism explainer: Python and Node, now in `reference/` with its history. It began at `talks/bengaluru-2026/video` in the maintainer's workspace, commits 63472ce … 6759f16. That pipeline stays the reference implementation until the Rust port matches it. Items are ticked as they are built, each with its evidence.

## What vidni is

- **For anyone who makes tutorials and explainers.** A creator writes one project file, `vidni.toml`. vidni renders from it:
  - a long 16:9 video and any number of 9:16 shorts
  - captions, chapters and thumbnails
  - an upload folder per platform
- **Two voices.** Draft with a synthetic voice, then finish with the creator's own recorded voice.
- **One Rust core, two front ends:** the scriptable `vidni` command line, and a native desktop app built with egui.
- **No LLM needed.** Every step works without one. An optional assistant layer may only edit the project file.
- **Linux first.** macOS and Windows come later.

## Rules every item keeps

1. **No LLM inside vidni.** No command or app action calls a model. An assistant, if a creator uses one, edits `vidni.toml` and runs the same commands a person would.
2. **Deterministic, in tiers.**
   - **T1:** the resolved config, the timeline, captions, metadata, the rgb24 frame stream before encoding and the PCM before encoding are byte-identical on any x86-64 host with the same `flake.lock`. This is the target; `docs/determinism.md` records which hosts have been verified.
   - **T2:** MP4 packet hashes (`ffmpeg -map 0 -c copy -f framemd5`) and the file bytes are identical on one host.
   - **T3:** across hosts, MP4s are compared by SSIM and not promised to match. The delivered files are archived.

   Some inputs are not deterministic: a live web page, every Chromium render, sampled TTS, whisper, the creator's takes, and the clock. They are frozen by hash at ratification, and whisper never feeds an output.
3. **Configure, don't script.** A new video needs no new code. A new capability is a versioned schema feature, documented and tested.
4. **One set of gates.** The CLI and the app call the same core functions, so no app action can get around a gate. A final build refuses any of these: content that changed since its cut was ratified, missing or stale takes, and failed QA.
5. **Honest content stays first-class:** wording lint, a source for each fact in strict projects, and checks of the text on screen.
6. **Local and private.** No telemetry. Nothing leaves the machine except pinned downloads during setup and the pages a project captures.
7. **For anyone.** Nothing the maintainer prefers is a default. The QUINE theme, the nibli example and the maintainer's platforms are examples only.
8. **Recurring dev and maintenance commands are `just` recipes.** The product's own commands are `vidni …`.

## Architecture (D3)

- **Rust workspace:**
  - `vidni-core`: the project model (TOML via `toml_edit` and serde), validation, line IDs, hashes and locks, per-cut state, timeline and caption cues, lint, platform profiles. Deterministic, file I/O only.
  - `vidni-media`: the stages that drive pinned external tools. Compose renders in Rust (tiny-skia, cosmic-text, resvg). Encode and QA use ffmpeg and ffprobe. TTS uses Piper. Capture uses Chromium over CDP. The intelligibility check uses whisper.cpp.
  - `vidni-record`: audio capture and playback through cpal, the level meter, and the take store.
  - `vidni-cli`: the `vidni` command.
  - `vidni-app`: the egui/eframe desktop app. It calls the core, media and record crates directly.
- **External tools,** pinned by version and sha256: ffmpeg, Chromium, the Piper voice models, the whisper.cpp model, and fonts. A Nix flake provides them on NixOS. `vidni setup` downloads them for everyone else.
- **Task runner:** a Justfile with recipes for build, test, lint, golden, parity, repro, app, example and release.
- **The reference pipeline** (Python and Node) produces the golden outputs. Each Rust stage replaces its counterpart only after matching them.

## The workflow, without an LLM

| Step | CLI | App |
|---|---|---|
| 1. Topic | `vidni new <slug> [--from outline.md]` | New project |
| 2. Sample | Write the lines and scenes in `vidni.toml`, then `vidni draft` | Script editor, Draft, preview |
| 3. Changes | Add requests by line ID to `notes.md`, edit, then `vidni draft` (incremental) | Notes per line on the review screen |
| 4. Ratify | `vidni ratify [--cut …]` | Ratify per cut, showing what is frozen |
| 5. Config | Written by ratify: `build/resolved.json`, the project plus defaults plus profiles as canonical JSON. It's the only file the engine reads and the file the lock hashes. | Nothing to do; the app shows it read-only |
| 6. Record | `vidni record` | Recording studio |
| 7. Final draft | `vidni final` | Final build, with QA |
| 8. Approve | `vidni approve` | Approve, showing hashes and tags |

## Decisions

All twelve are ruled; `DECISIONS.md` holds the full record (2026-11-13), including amendments A1–A8 to the *Done when* clauses below.

- **D1. Name and home.** *Ruled 2026-11-13:* `vidni`, a local repository at `~/projects/dhilipsiva/vidni`. Licence: MIT OR Apache-2.0. The repository is public from the first commit; P8-VIDNI-06 makes the release.
- **D2. Project file.** *Ruled 2026-11-13, replacing an earlier YAML ruling:* TOML, in `vidni.toml`, edited with `toml_edit` so comments and layout survive edits from both the app and the creator. The schema is versioned, and a JSON Schema is generated from it for editors.
- **D3. Architecture.** *Ruled 2026-11-13:* a Rust workspace (as described above), ported in stages behind golden-output parity. ffmpeg, Chromium, Piper models and whisper.cpp stay external pinned tools.
- **D4. Large assets.** *Ruled 2026-11-13:* frozen stills and takes stay in the project folder, out of git, listed with their sha256 in a committed `assets.lock`. Backups go to two destinations, one of them off the machine.
- **D5. Draft voice.** Default: Piper `en_US-joe-medium` (CC0 training data), seeded so it repeats. Voices come as packs with licence metadata, and a voice with a non-commercial licence is flagged. Kokoro is evaluated later (P9), seeded, with pinned weights and offline loading, because as packaged it calls `torch.rand` without a seed and downloads its weights at runtime.
- **D6. Platforms.** There are profiles for YouTube long-form and Shorts, Instagram Reels, TikTok, WhatsApp Status, LinkedIn and X, and the creator chooses which to enable. Regional notes live in the docs: TikTok is banned in India, per the latest official statements (August and September 2025). Enabled by default: YouTube long-form, YouTube Shorts, Instagram Reels and WhatsApp Status.
- **D7. Captions.** The long-form master has no burned-in captions, and its SRT/VTT is uploaded separately. A captioned variant is made for feeds that autoplay muted. Shorts are always captioned.
- **D8. Uploads.** Manual, from the generated upload folders. An API upload is opt-in and later, with a confirmation per upload.
- **D9. UI toolkit.** *Ruled 2026-11-13:* egui/eframe with AccessKit. The preview decodes frames through the pinned ffmpeg (no libmpv).
- **D10. Operating systems.** *Ruled 2026-11-13:* Linux first, through a Nix flake and a portable bundle. macOS and Windows come later.
- **D11. Task runner.** *Ruled 2026-11-13:* a Justfile wherever a command recurs.
- **D12. Default look.** *Ruled 2026-11-13:* a neutral vidni theme using IBM Plex Sans and IBM Plex Mono (OFL), plus Noto Sans Tamil for Tamil. QUINE ships in `examples/themes/`.

## Release lines

- **v0.1, CLI on Linux:** someone other than the maintainer can install vidni and make a video from the docs alone, with no new code. It contains:
  - P0-VIDNI-01, P0-VIDNI-02
  - P1-VIDNI-01 to P1-VIDNI-05
  - P2-VIDNI-01 to P2-VIDNI-09
  - P3-VIDNI-01 to P3-VIDNI-06
  - P4-VIDNI-01, P4-VIDNI-02, P4-VIDNI-05
  - P5-VIDNI-01 (one theme), P5-VIDNI-04 (images only)
  - P6-VIDNI-01, P6-VIDNI-02, P6-VIDNI-03
  - P8-VIDNI-01 (quickstart), P8-VIDNI-02, P8-VIDNI-04
- **v0.2, the desktop app:** P7-VIDNI-01 to P7-VIDNI-07, plus P3-VIDNI-07, P4-VIDNI-03, P4-VIDNI-04, P6-VIDNI-04 to P6-VIDNI-07, P8-VIDNI-03 and P8-VIDNI-05.
- **Public release:** P8-VIDNI-06, when the maintainer decides.
- **Out of scope:** analytics, and channel or monetization strategy (see the notes at the end).

## Items

Grammar, as in the workspace TODO, one line per item: `- [ ] **P<m>-VIDNI-<nn>** (<LOW|HIGH>) <imperative>. *Done when:* <evidence>. [S|M|L]`.
- **HIGH** means irreversible or public: tags, pushes, publishing, uploads, deleting takes. The maintainer runs or approves these.
- **P\<m\>** is the milestone.

### P0. Decisions and bootstrap
- [x] **P0-VIDNI-01** (HIGH) Rule on the open decisions (the D1 licence and publish timing, D5, D6's default set, D7, D8, D9's preview decoder, D12) and record every decision, D1–D12, in `DECISIONS.md`, giving the ruled ones their date. *Done when:* `DECISIONS.md` lists D1–D12, each with a date and a ruling or "default". [S]
  - Evidence (2026-11-13): the maintainer ruled every decision in plan mode. `grep -c '^### D[0-9]' DECISIONS.md` prints 12; every section is dated; amendments A1–A8 are recorded.
- [x] **P0-VIDNI-02** (LOW) Bootstrap the Rust workspace with the five crates (`vidni-core`, `vidni-media`, `vidni-record`, `vidni-cli`, `vidni-app`), a declared MSRV, rustfmt and clippy settings, a Nix flake pinning rustc, ffmpeg, Chromium, Piper, whisper-cpp and the fonts, and a Justfile with `build`, `test`, `lint`, `golden`, `parity`, `repro`, `app` and `example` recipes. *Done when:* inside `nix develop`, `just lint` and `just test` pass on the empty workspace, and so does `nix flake check`. [S]
  - Evidence (2026-11-13): in `nix develop`, `just lint` exits 0 (fmt check plus clippy -D warnings) and `just test` runs 5 test binaries, all ok. `nix flake check` prints "all checks passed!" (it builds the workspace and runs its tests). nixpkgs is pinned to 27eb89b (rustc 1.99.0), and the MSRV is 1.99, edition 2024.

### P1. Reference and parity harness
- [x] **P1-VIDNI-01** (LOW) Record golden outputs from the reference pipeline before anything moves: every line's audio, every still and card, both SRTs, the `framemd5` of the four nibli MP4s, and a `timeline.json` normalized to hold no absolute paths. The kept takes are the frozen audio; a clean run would draw new random Piper takes. *Done when:* `golden/` holds every hash, with the command that made it, and `just golden` re-derives them. [S]
  - Evidence (2026-11-13): `just golden-update` derived 178 inputs and 14 outputs in 320 s. An independent re-derivation, `just golden`, printed "golden verified: 178 inputs and 14 outputs match (311 s)". The match covers the byte-identical MP4 files and their framemd5 hashes for all four cuts, so the reference is already T2-deterministic on this host. `golden/README.md` lists each file and the command that makes it.
- [x] **P1-VIDNI-02** (LOW) Fix the variant leak in the reference, where running `compose` after `own` puts the creator's voice into the synthetic-named cut: make `timeline.json`, `narration.wav` and `narration-norm.wav` per variant, and make the paths relative. *Done when:* a test runs `own` and then a plain `compose`, and the synthetic cut keeps its own timeline. [S]
  - Evidence (2026-11-13): each variant now writes its own `build/timeline{,-voice}.json`, `build/narration{,-voice}.wav` and `narration{,-voice}-norm.wav`, with clip paths relative to the reference (workspace commit 6759f16, which P1-VIDNI-03 moves in). `tests/test_variants.py` builds the voice variant after the synthetic one, then checks with `compose.py --plan` that a plain compose reads the synthetic timeline and narration: 4 tests OK. End to end, `own`'s voice build followed by a plain `compose.py 16x9` re-rendered the synthetic cut with framemd5 `3d6323ed…` (the golden value), and `just golden` printed "golden verified: 178 inputs and 14 outputs match (318 s)".
- [x] **P1-VIDNI-03** (LOW) Move the reference into vidni with its history (`git subtree split` of `talks/bengaluru-2026/video`, merged with `--allow-unrelated-histories`) as `reference/`, and the nibli video as `examples/nibli-syllogism/`, with its takes moved rather than copied and a pointer left in the old folder. *Done when:* `just golden` passes from the new location. [M]
  - Evidence (2026-11-13): merge commit 1e57618 brings in the reference's four commits from `git subtree split`, rewritten under `reference/` so that `git log reference/tts.py` lists all four, and merged with `--allow-unrelated-histories`. 403 files were copied: `build/` and `out/` into `reference/`, and the takes (57 kept, 104 raw) into `examples/nibli-syllogism/recordings/`. Every sha256 matched at the new location before the originals were removed. `assets.lock` lists the 161 takes, and `just assets-check` passes. The old folder holds only a pointer README (workspace commit 106f220, local). From the new location, `just golden` printed "golden verified: 178 inputs and 14 outputs match (322 s)"; `golden/` changed only in `commands.json`'s takes path.
- [x] **P1-VIDNI-04** (LOW) Build the parity harness: `just parity <stage>` compares a Rust stage with the golden outputs, by exact equality for logic outputs and SSIM ≥ 0.995 for rendered frames while the port is in progress. *Done when:* the harness fails on a deliberately perturbed output and passes on the reference itself. [M]
  - Evidence (2026-11-13): `crates/vidni-parity` is the harness. `just parity <stage> --candidate DIR` (or `--reference-self`) compares the timelines, captions and narration PCM exactly; the MP4 packets by a fingerprint that doesn't depend on the ffmpeg reading them (DECISIONS.md, T1: the golden logs come from ffmpeg 8.1, vidni pins 9.0); and 20 frames per cut, plus every still and card, by SSIM ≥ 0.995. `just parity-selftest` printed "the reference itself: all 151 checks pass" and then caught all six deliberate perturbations. Each failed only its own stage and named only the changed file: a cue ending 1 ms later and a misspelled caption (timeline), one PCM sample off by one (narration), re-encoded audio with the pictures untouched (encode), a box over every frame (cuts, 20 of 80 checks) and a box on one still (stills). `just parity timeline --reference-self` exits 0, and an empty candidate exits 1. The SSIM unit test now always runs, in CI's shell and in `nix flake check`.
- [x] **P1-VIDNI-05** (LOW) Specify the reference's behaviour as tests before porting it: cue splitting, camera framing, timeline assembly, and the recorder's pty scenarios (silent take, double press, Esc, keep) using the `NIBLI_RECORD_SOURCE` hook. *Done when:* `just test-reference` passes in under 2 minutes, with no microphone and no network. [M]
  - Evidence (2026-11-13): `reference/tests/test_behaviour.py` specifies cue splitting (every golden sentence splits as its captions do), timeline assembly (lead-in, lines as long as their clips, gaps, tail, cues in proportion), camera framing (aspect, on screen, minimum width, a crop clear of the edges centres its target, a crop pinned to an edge still holds it, the bottom-edge trade-off, the overlay reserve, four known framings) and the recorder's pty scenarios (a silent take, a double press, Esc, a kept take) through `NIBLI_RECORD_SOURCE`. `just test-reference` runs it with `test_variants.py`: "Ran 23 tests in 45.272s OK", 46 s in all. It runs under `unshare -rn` with an empty `XDG_RUNTIME_DIR`. In that wrapper, a TCP connect fails with "Network is unreachable" and `pw-record` fails with "pw_context_connect() failed", while PipeWire answers outside it.

### P2. The core in Rust (deterministic logic)
- [x] **P2-VIDNI-01** (LOW) Define schema v1 for `vidni.toml` as serde types (`meta`, `voice`, `lint`, `lines` with id, show, say and source, `scenes`, `cuts`, `music` defaulting to `none`, `expectations`), generate its JSON Schema with schemars, write `docs/spec.md`, and convert the nibli example. *Done when:* `vidni validate` accepts the example and rejects 10 broken fixtures with line-and-column messages. [M]
  - Evidence (2026-11-13): `vidni-core` defines schema 1 (`project.rs`) and checks it (`validate.rs`). `vidni schema` generates `docs/vidni.schema.json`, which a test keeps current, and `docs/spec.md` describes every key. `examples/nibli-syllogism/vidni.toml` is converted from the reference's script, and a test checks it line for line against `reference/segments.json`. `vidni validate examples/nibli-syllogism` prints "ok: 57 lines in 10 sections, 5 scenes, 2 cuts" and exits 0. Each of 20 broken fixtures exits 1 with exactly its pinned `file:line:column: message`, checked in `crates/vidni-core/tests/fixtures.rs` and `crates/vidni-cli/tests/cli.rs`. They cover a syntax error, an unknown key, a wrong type, the schema version, a missing key, duplicate and malformed IDs, unknown sections, scenes, targets, cameras, lines and platforms, two verbs in one action, a highlight on a page scene, music, a split section, `text` with `texts`, a bad regular expression, and a missing page file.
- [x] **P2-VIDNI-02** (LOW) Add format-preserving edits with `toml_edit`: `vidni fmt` produces one canonical layout and keeps comments, and an editing API lets the app change one line or scene without touching the rest of the file. *Done when:* `fmt` round-trips the commented template unchanged, and a property test shows that random edit sequences keep every untouched comment. [M]
  - Evidence (2026-11-13): `vidni_core::fmt` writes one canonical layout (tables and keys in schema order, fixed spacing and blank lines, arrays on one line within 100 columns or else one element per line) and keeps every comment and every value's spelling; `vidni fmt [--check]` runs it. It leaves the commented template (`crates/vidni-core/src/template.toml`, for `vidni new`) unchanged, formats a scrambled copy of it (shuffled tables and keys, odd spacing, a wrapped array, stray blank lines) to exactly the template, keeps every comment, and is idempotent. The nibli example is now in the canonical layout. `vidni_core::edit::ProjectFile` sets and removes a line's keys and highlights, inserts, removes and moves lines, and sets scene values and the title. Exact-output tests show each edit changes only what it names; an edit to the nibli example changes one line of the file. A proptest runs 512 random sequences of up to 30 edits. Every comment of every remaining line stays attached to it, and every other comment stays, once. With `set_value` deliberately broken to drop a value's comment, the proptest fails with a shrunk counterexample.
- [x] **P2-VIDNI-03** (LOW) Key audio, captions and takes by stable line ID plus a hash of the spoken form, so a punctuation-only edit does not make a take stale, and migrate the nibli example's kept and raw takes. *Done when:* inserting a line reuses every other take, editing one line's spoken text marks only that take stale, and the migrated example builds. [M]
  - Evidence (2026-11-13): `vidni_core::spoken` gives a line's spoken form (`say`, or `show` with `voice.read_as`), its words (lowercase, punctuation removed) and their 16-hex-digit hash. `vidni_core::takes` files takes as `takes/<line-id>/<hash>.wav`, with raw takes in `raw/<hash>-<n>.wav`, and reports each line as current, stale or missing (`vidni takes`; `--link DIR` lays the takes out as numbered files). Tests on a project with a take per line show:
    - inserting lines leaves every other take current;
    - changing one line's words makes only its take stale;
    - a punctuation or capital edit keeps the take current, and so does undoing an edit;
    - moving and removing lines keep the others current.

    The nibli example's takes (57 kept, 102 raw and 2 room tone) moved from `recordings/NNN.wav` into the take store. Each was copied and checked by sha256 before the originals were removed. `assets.lock` lists the 161 files, and `vidni takes examples/nibli-syllogism` shows 57 current. The migrated example builds: `just golden`, which links the takes with `vidni takes --link`, printed "golden verified: 178 inputs and 14 outputs match (312 s)", and `just test-reference` passes 23 tests.
- [x] **P2-VIDNI-04** (LOW) Port the timeline and caption cues to Rust: gaps, cue splitting, cue timing, and the SRT and VTT writers. *Done when:* `just parity timeline` shows the timeline JSON and the SRT identical to the golden outputs. [S]
  - Evidence (2026-11-13): `vidni_core::timeline` ports the reference's timeline:
    - the lead-in, the line and section gaps, and the tail;
    - cue splitting, counting characters as Python does;
    - cue timing in proportion to each cue's length;
    - millisecond rounding as Python's `round` does it (correctly rounded, ties to even);
    - SRT and VTT writers.

    With no candidate given, `just parity timeline` now builds one with the Rust port, from the reference's synthetic clips and the example's takes. It prints "parity timeline: 4 of 4 checks pass": both variants' timeline JSON and SRT equal golden exactly. Unit and integration tests cover the gaps, a new section, mismatched clips, Python's rounding, and the SRT and VTT text. An unported stage, such as `just parity cuts`, exits 2 and names the item that will port it.
- [x] **P2-VIDNI-05** (LOW) Ratify per cut, with each cut's status derived from hashes and never stored: `ratified.lock` hashes each cut's lines, scenes and captures through `build/resolved.json`; gaps, caption style and platform toggles are hashed only by the release; and `vidni unratify` keeps captures, retaking one only when its scene changes or on `--refresh`. *Done when:* tests show that editing a short leaves the approved long cut approved, editing a shared line regresses every cut using it, a gap change needs no unratify, and a redraft re-captures nothing. [M]
  - Evidence (2026-11-13): `vidni_core::ratify` writes `build/resolved.json` (every default filled in) and `ratified.lock`. The lock holds each ratified cut's content hash, its lines' and scenes' hashes, and its capture files' SHA-256. A cut's status (draft, ratified, or changed with the reasons) is worked out from them each time; the commands are `vidni status`, `vidni ratify` and `vidni unratify [--refresh]`. `vidni_core::capture` keys web captures by scene without cameras, actions so far and highlights, and page renders by scene and size (DECISIONS.md, T3). `crates/vidni-core/tests/ratify.rs` shows:
    - editing the short's own line, or its line list, leaves the ratified long cut ratified;
    - editing a line both cuts use regresses both, and names the line;
    - changing gaps, caption style, platforms and the theme needs no unratify;
    - after a draft, a redraft plans no captures, and neither does one after `unratify`;
    - a changed camera re-captures nothing, while a moved page re-captures its three lines but not the card;
    - `--refresh` deletes only the capture no ratified cut shares;
    - a capture changed on disk regresses the cuts that froze it.

    A CLI test runs status, ratify, an edit and unratify end to end.
- [x] **P2-VIDNI-06** (LOW) Move the honesty checks into the core: banned-phrase lint, a required `source` on every factual line in strict projects, and checks of the text on screen. *Done when:* each check has a failing fixture with a clear message. [S]
  - Evidence (2026-11-13): `vidni validate` now runs the honesty checks in the core: `lint.banned` against every line's `show` and `say`, and, in strict projects, a `source` on every line that states a fact (a new `claim = false` marks a line that states none). `vidni_core::onscreen` compares a capture's readings (`build/onscreen.json`) with `[[expectations]]`, whitespace collapsed. Each check has a failing fixture:
    - `invalid/21-banned-wording.toml` fails at its `show` with "line \"main-1\" shows banned wording: \"guarantee\" matches the lint pattern \"guarantee\"".
    - `invalid/22-claim-without-source.toml` fails with "line \"main-3\" states a fact with no source, and the project is strict; add source = \"…\", or claim = false if the line states no fact". Its `claim = false` line and its sourced line pass.
    - `fixtures/onscreen/changed.json` fails with messages such as "at line \"main-1\", target \"result\" shows \"UNKNOWN\", not \"FALSE\"", while `matching.json` passes.

    The nibli example passes its own wording lint.
- [x] **P2-VIDNI-07** (LOW) Write the platform profiles as data. Each covers aspect, resolution, `max_upload` and `max_recommended` durations, maximum size, loudness target, caption style and safe zones, with a "verified on" date and a source. The profiles are YouTube long, YouTube Shorts, Instagram Reels, TikTok, WhatsApp Status, LinkedIn and X. *Done when:* `vidni platforms` lists them with their dates and sources. [S]
  - Evidence (2026-11-13): `crates/vidni-core/src/platforms.toml` holds the seven profiles. Each has an aspect and resolution, `max_upload`, `max_recommended` where the platform states one, and the maximum size where it is published. Each also has a loudness target with its basis (official, measured, or vidni's default), caption delivery, safe zones with their basis, notes, a `verified_on` date, and its sources, official pages first. `vidni platforms` lists them with their dates and sources, and cuts take their platform IDs and defaults from the profiles. The research found:
    - no platform publishes a loudness target;
    - the only published safe zones are ads guidance;
    - WhatsApp Status now takes 90 s, where an earlier note in this file said 60 s.

    Tests check that every profile is complete and sourced, that the defaults match D6, and that the CLI lists each profile with its date and https sources.
- [x] **P2-VIDNI-08** (LOW) Add `vidni new <slug> [--from outline.md]`, where outline headings become sections and paragraphs become lines, and `vidni plan`, which prints the recording sheet. *Done when:* `vidni new demo && vidni draft demo` renders with no edits. [S]
  - Evidence (2026-11-14): `vidni new demo && vidni draft demo` renders both cuts with no edits, in 11 s: `demo/build/draft/long.mp4` (1920x1080) and `demo/build/draft/vertical.mp4` (1080x1920), 218 frames each, 7.25 s, -17 LUFS. `just test-build` runs exactly that (`a_new_project_drafts_as_it_is`).
    - **`vidni new <name>`** writes `vidni.toml`, `page.html` and a `.gitignore`. The scene is that page, a file in the project folder, so a first draft needs no network: the same test drafts a second new project with `--offline`. The project is valid and in the canonical layout. A folder that exists is left alone, and the name must be an ID.
    - **`--from outline.md`** makes the first `#` heading the title, every other heading a section, and each paragraph and list item a line. IDs come from the headings (`## Draft, then record!` gives `draft-then-record-1`). An outline with no line makes no project.
    - **`vidni plan`** prints each line's spoken form under its section's label, marked `[x]` recorded, `[~]` stale or `[ ]` to record, with how long the draft voice took over it, and totals. `--cut` lists one cut's lines. It writes nothing.
    - **Found on the way:** `--offline` refused every web scene, even one whose page is a file in the project folder. Now it refuses only a page on the web that isn't captured yet.

    Tests: `starter` (5) and `sheet` (1) unit tests, three CLI tests of `new` and `plan`, and the end-to-end test. `vidni repro demo --draft --offline` on a new project: T1 and T2 equal. docs/spec.md describes both commands.
- [x] **P2-VIDNI-09** (LOW) Add a content-addressed cache for every stage: a TTS clip by hash(spoken text, voice, seed, params), a still by hash(scene spec, source revision), and a render by hash(inputs). *Done when:* a second `vidni draft` with nothing changed renders nothing and finishes in under 10 s. [M]
  - Evidence (2026-11-14): a second `vidni draft` of the nibli example (57 lines, two cuts of five minutes each) with nothing changed takes 0.6 s, where the first takes minutes. It speaks nothing (`57 from the cache`), says `every capture is taken`, and says of each cut `nothing has changed since it was built; nothing to render`. `just test-build` holds the same on two small projects: under 10 s, nothing rendered, and the files' times unchanged.
    - **A draft clip** was already filed by what the voice is given, the pack and the pace, with its seed (P3-VIDNI-03).
    - **A capture** of a page in the project folder now has a record beside it, `<key>.files.json`: the project's files the page loaded, each with its SHA-256, read from the requests Chromium reports. The capture is out of date once one of them changes. The next draft takes it again and says which file changed. `just test-capture` holds this on a page with an imported stylesheet, pictures, a script, and a picture that a click loads. A page on the web has no record. A page of the project's that loads a file from outside the project folder fails its capture.
    - **A build** is kept when its record has the cut's `inputs` and each file it made still has its hash. `--again` renders anyway. A final build that is kept is still delivered.
    - **A ratified cut** with a capture that is out of date reads `changed` and names the file. A draft takes that capture again though the cut froze it. With the file put back, the cut is the ratified one again; the end-to-end test does this with a card.
    - **The source revision is a record, not a part of the key** (DECISIONS.md, T9): which files a page loads is known only once it has loaded. The keys are unchanged, so no capture of the example was renamed.
    - **The nibli example:** its eight card renders had no record. `vidni draft` took them again, each with the hash `ratified.lock` had frozen, so both cuts stayed ratified and approved. `assets.lock` now lists the eight records.

    Checked on the example after the change: `vidni repro`, for the final cuts and with `--draft`, agrees with the builds that were kept; `just parity product` passes 46 of 46 and `just parity stills` 118 of 118. A fresh project was walked through an edit, ratification, a file put back, deleted records, a removed draft file, `--again` and an interrupted draft, and did the right thing each time.

### P3. Media stages in Rust, each behind parity
- [x] **P3-VIDNI-01** (LOW) Write encode and QA as an ffmpeg wrapper with fixed flags: fixed libx264 `-threads`, `+bitexact`, `-map_metadata -1`, an explicit BT.709 conversion with colour tags, two-pass loudnorm, and `-cpuflags 0` for audio filters. QA checks streams, duration, loudness, caption count and chapters. *Done when:* fed the golden frames and PCM, it produces the same `framemd5` twice on one host (T2). [M]
  - Evidence (2026-11-13): `vidni_media::encode` drives ffmpeg with fixed flags:
    - libx264 at 4 threads, preset slow, CRF 18;
    - `+bitexact` for the container and both codecs, and `-map_metadata -1`;
    - a BT.709 conversion with bit-exact rounding and the colour tags to match;
    - `-cpuflags 0`;
    - two-pass loudnorm before encoding.

    `vidni_media::qa` checks streams, duration, loudness, caption count and chapters, and names each problem. `just encode-t2` encoded the golden 16:9 cut's 9,188 frames with the golden narration twice, giving the same framemd5 (23,545 packet lines) and the same 25,089,781 bytes. A second invocation gave the same bytes again. Tests show the same packets and bytes for the same synthetic input, no date or version number in the container tags, loudnorm on target, and QA naming five deliberate faults. The narration track is ported too: `just parity narration` shows both variants' PCM equal to golden. `docs/determinism.md` records the results.
- [x] **P3-VIDNI-02** (LOW) Port the compositor to Rust (tiny-skia, cosmic-text, resvg): virtual camera framing with top and bottom reserves, crossfades, captions, labels, overlays and vertical bands, with fonts named by store path and sha256. *Done when:* frames reach SSIM ≥ 0.995 against golden at 20 timestamps per cut, two runs are T1-equal, and `docs/determinism.md` records the result on this desktop and in a clean docker container (amended A1 and A9). [L]
  - Evidence (2026-11-13): `vidni_media::compose` is the reference's `Cut`, ported:
    - camera framing with the top and bottom reserves, crossfades, captions, section labels, overlays, and the 9:16 bands;
    - Pillow's drawing ported exactly (`pillow.rs`), and tested byte for byte against fixtures Pillow wrote;
    - text shaped by harfrust and placed in 26.6 as Pillow places it (`text.rs`). Every caption width matches Pillow's `getlength`, and drawn text is within 2 levels of 255.

    How it was checked:
    - `just parity cuts`: 80 of 80 sampled frames reach SSIM ≥ 0.995, against the frames golden's MP4s were encoded from (A9). The lowest rounds to 1.0000, and the largest channel difference is 2.
    - `just compose-stream`: all 1,254 pictures of each cut change at the reference's frames and stay within 2 levels. It fails on a 0.02 s change to the caption lead.
    - `just parity-selftest`: catches a box drawn on one frame.
    - The fonts are pinned by sha256, and `compose-t1` prints each one's store path and hash.
    - `just compose-t1` rendered each cut twice with identical rgb24 streams. A clean `nixos/nix` container that built vidni from source got the same four sha256. `docs/determinism.md` records both.
    - An independent review found no defect in the compositor. Its findings on the evidence led to the font pins, the stream comparison, the schedule and crossfade tests, and refusing a card of the wrong size. DECISIONS.md T4 records the design.
- [x] **P3-VIDNI-03** (LOW) Port TTS: Piper models run through `piper-rs` (ONNX via `ort` plus espeak-ng) or the pinned Piper CLI, seeded per line; best-of-N locks the winning seed; and voice packs carry licence metadata. *Done when:* clearing the cache and redrafting gives identical PCM, and a non-commercial voice is flagged in `vidni validate`. [M]
  - Evidence (2026-11-13): `vidni draft` speaks every line with the pinned Piper, patched in the flake to take `--seed` (`nix/piper-seed.patch`; DECISIONS.md T5). The same seed gives the same PCM, and vidni refuses a Piper that ignores the flag.
    - A line's first draft tries three seeds and keeps the take of median length. Its seed goes into the committed `voice.lock`, and clips are cached in `build/voice/`.
    - Voice packs (`vidni voices`, `vidni voices fetch`) pin their files by SHA-256 and carry their model card's licence. A creator's own pack carries a `pack.toml` of the same fields.

    On the nibli example, a first draft chose 57 takes from 171 in 1m52s. A redraft with the cache cleared spoke the 57 locked seeds in 33 s and gave the same PCM (combined sha256 `59359d31…ffca`). So did a redraft with `voice.lock` cleared too, which chose the same seeds. Tests cover the same flow with a stand-in engine, plus median choice, edited lines and seeds given by hand. `vidni validate` on a project whose draft voice is piper:en_US-ryan-medium exits 0 and warns at `voice.draft`: "the draft voice piper:en_US-ryan-medium is licensed CC-BY-NC-SA-4.0, which doesn't allow commercial use".
- [x] **P3-VIDNI-04** (LOW) Port capture to Chromium over CDP (chromiumoxide) as a generic web-scene runtime: actions, highlights including line ranges in text, cameras and expectations. A capture retries until two screenshots in a row match, and Chromium is pinned and run with `--font-render-hinting=none --force-color-profile=srgb`. *Done when:* the nibli example's stills reach SSIM ≥ 0.995 against golden, and five captures of one scene are pixel-identical. [L]
  - Evidence (2026-11-14): `vidni capture` takes each web line's still and each page scene's render through the pinned Chromium. vidni's own small client drives it over its DevTools pipes (`vidni_media::cdp`; DECISIONS.md T6 says why not chromiumoxide).
    - The page runtime (`capture/runtime.js`) covers every target key, highlights with line ranges, cameras and the seven actions. It runs in an isolated world.
    - A still is taken once the page has settled and the line's expectations hold. A scene's stills are kept only when it passes: a failed expectation, a script error, a dialog or a crashed renderer fails it.
    - Beside each still, `<key>.json` records the boxes of its cameras and highlights.

    How it was checked:
    - `just parity stills`: 118 of 118. All 53 stills and 8 cards score SSIM 1.0000 against golden, overlay alpha is equal, and each still's frames are within 0.5 CSS px of the reference's.
    - Five captures are pixel-identical, for the local test scene (`just test-capture`, in CI) and for the example's live page (`just capture-repeat`: the pixels and frames of all 46 stills). One live still captured 40 times gave one result.
    - That took three fixes, each found by capturing one page many times (T6): a page above scale 1 is loaded twice, each layer is rastered as one tile, and vidni's style and highlight layer are mounted at document start.
    - `just test-capture` runs 8 Chromium tests on local pages, among them a late result, a script error, a dialog, a covered click, a navigation and a busy network. `just parity-selftest` catches a moved camera box and an overlay made opaque.
    - An independent review ran twice. The first found defects that are now fixed: stale stills, ignored page errors, leaked browsers and an open debugging port among them. The second confirmed the fixes and traced the last run-to-run difference to capture itself, which led to the tile and mount fixes.
    - Fixed on the way: the flake's fontconfig file had let the host's fonts in and now pins them, and a line's capture key covers the expectations waited for.
- [x] **P3-VIDNI-05** (LOW) Port the intelligibility check (whisper.cpp through `whisper-rs` or the pinned CLI). It is advisory only, never feeds an output, and returns no transcript for empty audio instead of crashing. *Done when:* tests cover empty, silent and spoken clips. [S]
  - Evidence (2026-11-14): `vidni hear` listens to each line's take, or else its draft clip, with the pinned `whisper-cli` and its `ggml-base.en` model, and scores what it hears against the script (`vidni_media::hear`; DECISIONS.md T7 says why not `whisper-rs`).
    - It only advises: it writes no file, and no build step reads a score. A line that scores below 0.90 is listed with what was heard.
    - A clip that is empty, shorter than 0.3 s or silent has nothing to hear, and whisper is never given it. Measured with the pinned tools: whisper-cli exits 11 on an empty clip and hears "you" in silence.
    - The score is the reference's, ported exactly: it gives the reference's score for each of its 57 lines. The homophones the reference had built in became a key, `voice.heard_as`, and the example carries them.
    - The check knows English only, and refuses a script in another language.

    How it was checked:
    - `just test-hear`, which CI runs, uses the real whisper-cli. A spoken clip scores at least 0.9, and an empty and a silent one have nothing to hear, through the library and through `vidni hear` on a five-line project. That project also has a take of other words, a missing take and an unreadable one, and its folder is left untouched.
    - Unit tests need no whisper. They cover empty, short, silent and too-quiet clips, a whisper-cli that isn't there, whisper's notes on sounds, and which clip a line is heard from.
    - With the guard removed, so that whisper is given every clip, the unit test and `just test-hear` both fail.
    - On the nibli example, the 57 takes score 0.974 on average and the 57 draft clips 0.986. Two of each score below 0.90: the takes of `syllogism-1` and `end-1`, and the draft clips of `nibli-1` and `end-1`. Two runs of each printed the same.
- [x] **P3-VIDNI-06** (LOW) Wire up `vidni draft`, `ratify`, `final`, `approve`, `repro` and `--offline`. `repro` builds twice under `env -i` with a fixed TZ and locale, `SOURCE_DATE_EPOCH` and different paths and core sets; `--offline` runs under `unshare -n`. *Done when:* the nibli example goes from draft to approved through the Rust CLI, with T1 and T2 equal under `vidni repro`. [M]
  - Evidence (2026-11-14): the nibli example went from draft to approved through the `vidni` command, and `vidni repro` found its builds equal in T1 and T2.
    - `vidni draft` spoke the 57 lines, took the 54 captures and rendered both cuts in the draft voice (9,197 frames each, in 4m47s).
    - `vidni ratify` froze both cuts. `vidni final` built them from the creator's 57 takes into `out/` (8,441 frames each, −16.7 LUFS, in 3m58s). `vidni approve` recorded them in `release.json` as `long-r1` and `vertical-r1`.
    - `vidni_media::build` builds a cut and records every input and output by hash (`<cut>.build.json`). The gates are in `vidni_core::release`: a final build needs a ratified, unchanged cut and a current take for each line, and it is delivered only when QA passes. Approving needs that build, still current, and its delivered files untouched.
    - `vidni repro` builds twice from copies of the project at other paths, in clean environments, on two cores and on three others (`taskset`), and compares every hash. `--offline` runs a command in a namespace with no network (`unshare`).
    - Also new: `vidni speak` (the draft voice alone), fonts pinned by SHA-256 and found through `VIDNI_FONTS`, and each cut's standing against its release in `vidni status`. DECISIONS.md T8 has the reasons.

    How it was checked:
    - `vidni repro examples/nibli-syllogism`, run twice: five builds of each final cut agree in every hash, with each other and with the build in `build/final/`. So do the drafts under `--draft`, with all 57 lines spoken again in each build, and the 9:16 cut with no network (docs/determinism.md).
    - **repro found a fault first.** Two builds of the 9:16 cut had the same frames and different packets. The cause was x264's threaded lookahead: 40 encodes of 451 frames gave 8 packet streams with it, and one with `sync-lookahead=0`, on 1 to 32 cores. The encoder now sets it.
    - `just parity product`: 46 of 46. The example's final build, planned from fresh captures and the takes, has golden's timeline, captions and narration, and its 40 sampled frames score SSIM 1.0000 against the reference's. With a line's 9:16 camera ignored, it fails.
    - `just test-build`, which CI runs, takes a small project from `vidni draft` to `vidni approve` with the real tools, then runs repro on its final cuts and its drafts, and builds with `--offline`.
    - Tests that need neither Chromium nor Piper cover each gate and refusal, the build's record, the three caption settings, a build that fails QA, and each way an approval is refused. They also hold `voice.heard_as` and the draft voice's settings out of every final build's hashes.
    - With the encoder's threads made to follow the cores, repro reports the packets as different and the frames as equal. Interrupted once or twice, by SIGINT or SIGTERM, repro ends both builds and removes its copies.
    - **An independent review** found no way past a gate, and 21 other things, all dealt with (DECISIONS.md, T8). The worst: a camera edited after a capture never reached the picture, because each camera's box was read as the capture had measured it. A still's frames now hold its targets' boxes, and cameras are framed when a cut is built; `just parity stills` still passes 118 of 118.
    - Fixed on the way: a still's frames weren't frozen with it at ratification, and a Chromium that stopped at once could be reported without what it had said.
- [ ] **P3-VIDNI-07** (LOW) Retire the reference once every stage passes parity: it stays read-only in `reference/`, and `just parity` keeps running against it. *Done when:* no Python or Node remains on the build path, and `just parity` is green. [S]

### P4. Recording, in Rust
- [x] **P4-VIDNI-01** (LOW) Build the recorder core on cpal: capture, a level meter, raw takes that are never modified, the guard against silent or too-short takes, trimming of keypresses and silence, and room tone recorded during the mic check. *Done when:* tests driven by a synthetic input device cover a silent take, a short take, a normal take and room tone. [M]
  - Evidence (2026-11-14): `crates/vidni-record/tests/recorder.rs` records from a made-up input (`input::Synthetic`) through the same path a microphone's samples take, and covers:
    - **a normal take:** a click, room noise, two seconds of a voice-like tone, room noise and a click. The raw take is what the input gave, sample for sample. The trimmed take is the words with 0.15 s before them and 0.25 s after, and neither click. Kept, it is the line's take; a second take replaces it only when it is kept, and the raw take's hash never changes;
    - **a silent take:** three seconds of a room's hum is refused as having no speech (peak −54 dB), and so is digital silence. Nothing is kept;
    - **a short take:** half a second is refused, and a cancelled take leaves no file;
    - **room tone:** three seconds of a room are recorded, its middle is kept as `takes/_room/tone.wav`, and its level is measured (−60 dB, quiet; −43 dB, noisy).

    It also covers the level meter, and an input that stops on its own.
    - **The microphone** is `mic::Mic`, through cpal on ALSA. `just test-record`, which CI runs, records from ALSA's `null` device, which gives zeros and needs no sound card: the maintainer's microphone was not recorded from. `cargo run -p vidni-record --example inputs` lists the inputs and says which one vidni would use, without opening a stream; here that is the default device at 48 kHz.
    - **The trim is the reference's, sample for sample** (DECISIONS.md, T10). `just parity takes` passes 57 of 57: each kept take of the nibli example is vidni's trim of one of its line's raw takes, with the two samples ffmpeg's `silenceremove` misplaces at each splice put where it put them. With the filter's result rounded, or with another margin, none passes.
    - **Found on the way:** the reference's chain meant to drop 0.30 s for the stop key and dropped 0.10 s, because its second `atrim` read timestamps that started at 0.20 s. The 57 takes were made that way, so vidni drops 0.10 s.

    `vidni record`, the terminal front end, is P4-VIDNI-02.
- [x] **P4-VIDNI-02** (LOW) Port the terminal recorder `vidni record` with crossterm: the single-key flow (space, Enter, Esc, p, r, arrow keys), resuming, reviewing a section, recording a line shared by several cuts once, and showing stale takes. *Done when:* pty tests cover a silent take, a double press, Esc, keep and resume on a two-cut project. [M]
  - Evidence (2026-11-14): `crates/vidni-cli/tests/record.rs` runs `vidni record` in a pseudo-terminal on a project with two cuts that share a line. `just test-record` runs it, and so does CI. It covers:
    - **a silent take:** discarded with "There was no speech in that take", and the line stays; so does a take of half a second;
    - **a double press:** the second space comes while the screen settles, and the take it started goes on;
    - **Esc:** in the review it discards the new take, and while recording it cancels the take and leaves no raw take;
    - **keep:** enter keeps the take and moves to the next line still to record; `q` keeps it and ends the session;
    - **resume:** a new session starts at the first line with no take of its present words. With `--cut short` only the short cut's own line is left, since the line it shares with the long cut was recorded once;
    - **stale takes:** a line reworded after it was recorded says its take is of other words, and the session starts there;
    - **a section:** `--section` goes through one section's lines, and `s` plays a section's takes;
    - **the mic check** (`--test`): the room is recorded and measured, a line is read, and only the room tone is kept;
    - **an interrupt:** Ctrl-C or a termination signal keeps nothing that wasn't kept, puts the terminal back, and exits with 130.

    The takes are recorded from a made-up input and played through ALSA's null device: nobody is recorded and no sound is made. `just test-hear` records a spoken clip the same way, and the review says "matches the script 1.00" for its own line and shows what was heard for another. On the nibli example the recorder shows each of the 57 lines with its saved take and the draft voice's pace. The tests passed nine times in a row, three of them with every core busy.
- [ ] **P4-VIDNI-03** (LOW) Add a deterministic voice chain before loudness: high-pass, de-click, compression and de-esser. Gaps are filled with room tone instead of digital silence. *Done when:* QA reports every cut within ±1 LU of its target with true peak at or below −1 dBTP, and no digital-silence stretch longer than 50 ms. [M]
- [ ] **P4-VIDNI-04** (LOW) Make takes non-destructive: trims are stored as data, `vidni takes <line>` re-picks or re-trims a take, each take records its session metadata, and QA flags level jumps between sessions. *Done when:* re-picking an older take changes only that line. [M]
- [x] **P4-VIDNI-05** (LOW) Add `vidni backup`, `restore`, `verify` and `gc`, with two destinations, one of them off the machine. `gc` deletes only files that can be rebuilt. *Done when:* wiping the assets, then running `vidni restore` and `vidni final`, reproduces the T2 hashes in `release.json`. [S]
  - Evidence (2026-11-14): on a copy of the nibli example, `vidni backup --to disk --to elsewhere` listed 269 assets (the takes, and the captures both cuts froze) and copied them, with the 6 files the two releases delivered, to each place. The takes, `build/` and `out/` were then wiped: `vidni verify` found 0 of 269, and `vidni final` was refused. `vidni restore --from elsewhere` brought back 275 files, and `vidni final` built both cuts again. `vidni approve` answered `already approved as long-r1` and `vertical-r1`: the files have the T2 hashes in `release.json` (`d9987cba…b950` and `c152b5eb…0d0c`), and `release.json` is unchanged. The maintainer's own files weren't touched.
    - **The same in the tests:** `just test-build` backs a small project up to two places, deletes what can be made again, wipes the assets and what was delivered, restores from the places (one of which has lost its takes), builds again, and gets the approved releases. A quick test (`cargo test`) covers the commands without any media tool, and `crates/vidni-core/tests/assets.rs` covers what counts as an asset, what a backup holds, and what `gc` may delete.
    - **`gc`** deleted 85 files, 413.5 MB, on the example's copy: 57 draft clips, the drafts and the final builds. Every asset still verified afterwards.
    - **A place is a folder** (DECISIONS.md, T11). The one off the machine is a folder mounted from it or a disk carried away; vidni refuses fewer than two places and a place that isn't there, and warns when every place is on the project's disk. No machine other than this one was reachable to test with, so both places were folders here.
    - **The example's `assets.lock`** is now in the order vidni writes, by the bytes of the path. It lists the same 269 files with the same hashes.

### P5. Scenes
- [x] **P5-VIDNI-01** (LOW) Build card and slide scenes from theme templates (title, section, bullets, quote, end card), with themes as token files: a neutral default theme, and QUINE in `examples/themes/`. *Done when:* both themes render every template, and each template has a snapshot test. [M]
  - Evidence (2026-11-14): `crates/vidni-media/tests/cards.rs` draws each of the five templates (title, section, bullets, quote, end) in both themes and both shapes, and compares each with its PNG snapshot in `tests/fixtures/cards/<theme>/`: 20 snapshots. It runs in `cargo test`, with no browser.
    - **Themes are token files:** eight colours, three fonts and the mark before a label (`theme.toml`). The default theme is built in; `examples/themes/quine` holds the look the reference drew. `[style] theme` names a theme folder.
    - **A card scene** is `kind = "card"` with a `template` and its words. The compositor draws it at each cut's size, so it has no capture. A cut made only of cards builds with nothing in `build/captures/`.
    - **The nibli example names the QUINE theme,** and its final cuts have the frames and files they had: `c27e3dbe…91c8` and `d9987cba…b950` for the long cut, `7f1d9f0f…e125` and `c152b5eb…0d0c` for the vertical one. A theme is now one of a build's inputs, so they were approved again, as `long-r2` and `vertical-r2`.
    - **Both halves were done together** (DECISIONS.md, T12): once the default look became plain, the example needed its own theme at once.
    - **Left as it was:** highlight boxes on a still are ember in every theme, since they are part of the capture.

    Checked after the change: `just parity cuts` 80 of 80, `just parity product` 46 of 46, `just compose-t1`, `vidni repro` on the example, and the tool-dependent suites.
- [ ] **P5-VIDNI-02** (LOW) Add a terminal scene: a command transcript (recorded text, or a pinned command run in a sandbox) drawn as a styled terminal, with line highlights. *Done when:* an example renders a nibli REPL session from a transcript file, with T1 equal across runs. [M]
- [ ] **P5-VIDNI-03** (LOW) Add a code scene: a file excerpt at a pinned git revision, with syntax highlighting and line highlights. *Done when:* an example renders a Rust excerpt at a commit, with T1 equal across runs. [M]
- [ ] **P5-VIDNI-04** (LOW) Add image and diagram scenes: PNG and SVG through resvg, and Mermaid through a pinned CLI, all under the virtual camera. *Done when:* the examples render with T1 equal across runs. [S]
- [ ] **P5-VIDNI-05** (LOW) Make overlays templates (chip rows, flow diagrams, callouts) instead of the two hard-coded ones. *Done when:* the nibli verdict and firewall overlays are expressed as data. [S]

### P6. Cuts, shorts and exports
- [ ] **P6-VIDNI-01** (LOW) Allow several cuts per project: a long cut plus any number of shorts, each picking scenes and lines and adding its own hook lines. *Done when:* the nibli example defines a short of 60 s or less built from shared and short-only lines. [M]
- [ ] **P6-VIDNI-02** (LOW) Make the vertical layout respect the combined safe zones of every enabled app (about 14 % top and 35 % bottom per Meta's guidance for Reels ads, and about 130 px on the right from third-party measurements, confirmed by the maintainer's private uploads of generated grid-overlay test clips (amended A2)), and have QA draw the zones on review frames. The reference's caption band and 330 px title band both violate them. *Done when:* review frames for every vertical profile show nothing important inside a zone. [M]
- [ ] **P6-VIDNI-03** (LOW) Produce the long-form outputs for D7, a clean master with SRT/VTT and a captioned variant, plus YouTube chapters from the section labels (the first at 0:00, at least 3, each at least 10 s). *Done when:* both variants exist, and QA checks the captions and the chapter rules against the master. [S]
- [ ] **P6-VIDNI-04** (LOW) Generate metadata and images: per-platform titles and description templates (with an AI-disclosure note when a synthetic voice remains), tags, a 1280×720 thumbnail under 2 MB with up to 3 variants for YouTube's Test & Compare, a Reels cover with its title inside the 3:4 grid crop, a 5–20 s end-screen hold on the long cut, and an upload step to set each Short's related video. *Done when:* each `out/<platform>/` holds the video, captions, `metadata.md` and its images. [M]
- [ ] **P6-VIDNI-05** (LOW) Add per-platform QA (duration against `max_upload` and `max_recommended`, size, resolution, codec profile, loudness, caption sync) and caption QA (contrast of at least 4.5:1, at most 2 lines of 42 characters, at most 20 characters per second, cues of at least 0.83 s). *Done when:* a deliberately broken export fails each check, and the reference's 9:16 cut fails the caption checks. [M]
- [ ] **P6-VIDNI-06** (LOW) Make music `none` by default, allowing a track only from library entries that carry the licence, the attribution, the platforms covered and the sha256; any Content ID claim blocks a Short longer than 60 s worldwide. *Done when:* `vidni validate` rejects a track not licensed for every enabled platform. [S]
- [ ] **P6-VIDNI-07** (LOW) Generate an `UPLOAD.md` per platform and handle corrections: immutable per-cut releases, a `published.json` of URLs, `errata` entries that feed a Corrections block in the descriptions, and `vidni where <line>`. YouTube can't replace a published video file. *Done when:* an erratum on a shared line updates every affected description with no re-render. [M]

### P7. The desktop app (egui/eframe)
- [ ] **P7-VIDNI-01** (LOW) Build the app shell: a project browser, new and open, each cut's derived status, and a job runner with progress and logs that calls the core and media crates directly, with AccessKit on. *Done when:* a project can be created, drafted and opened from the app, and the help lists the CLI equivalent of every action. [M]
- [ ] **P7-VIDNI-02** (LOW) Build the script and scene editor on `toml_edit`: structured forms with a source view, inline validation and visible line IDs. *Done when:* a test interleaves app edits and hand edits and every comment survives. [L]
- [ ] **P7-VIDNI-03** (LOW) Build the preview player: draft frames decoded per D9 into egui textures, line markers on the timeline, click a line to seek, a caption toggle, and a contact-sheet view. *Done when:* scrubbing a 5-minute draft stays responsive, and clicking a line lands within one frame of its cue. [L]
- [ ] **P7-VIDNI-04** (LOW) Add review notes per line, written to `notes.md`, and a diff of what changed since the last draft or ratification. *Done when:* notes survive an app restart, and the diff names the changed lines. [M]
- [ ] **P7-VIDNI-05** (LOW) Add the gates to the app: ratify per cut (showing the frozen assets and hashes), final, and approve, with the same preconditions and messages as the CLI. *Done when:* the CLI's gate tests also pass when driven through the app's command layer. [M]
- [ ] **P7-VIDNI-06** (LOW) Build the recording studio: the prompter, live meter and waveform, a takes list with A/B playback, re-trimming, and room tone, sharing the CLI's take store. *Done when:* takes recorded in the app and in `vidni record` are interchangeable. [L]
- [ ] **P7-VIDNI-07** (LOW) Build the export panel: platform toggles, QA results, and buttons that open each upload folder. *Done when:* toggling a platform changes the next `vidni final` and only its exports. [S]

### P8. For anyone: docs, packaging, release, and the optional assistant layer
- [ ] **P8-VIDNI-01** (LOW) Write the docs: a Linux quickstart, a spec reference generated from the schema, a recording guide, a platform guide with regional notes, a determinism guide and an FAQ. *Done when:* a fresh agent session that has never seen the code makes a short video from the docs alone in a clean Linux container and reports where it got stuck (amended A3). [M]
- [ ] **P8-VIDNI-02** (LOW) Ship two examples: the nibli syllogism, and a neutral "hello vidni" project that uses only cards and images and needs no network. *Done when:* `just example hello` renders offline. [S]
- [ ] **P8-VIDNI-03** (LOW) Package for Linux: `nix run` from the flake, and a portable bundle whose `vidni setup` downloads the pinned ffmpeg, Chromium, voices and whisper model by sha256 into a user cache, with no root. *Done when:* a fresh non-Nix Linux VM goes from download to a rendered example with only the documented steps. [L]
- [ ] **P8-VIDNI-04** (LOW) Write the licences and notices: the LICENSE files for D1, and third-party notices for fonts, voices (with their dataset licences), models, Chromium and ffmpeg, including the source-offer duty a GPL ffmpeg build (x264) brings to the bundle. *Done when:* `just notices` regenerates the notices and the bundle includes them. [S]
- [ ] **P8-VIDNI-05** (LOW) Set up release engineering: semver, a Keep a Changelog `CHANGELOG.md`, `just release` producing checksums and signatures, and a reproducible Nix build of vidni itself. *Done when:* two `just release` runs give identical artifacts. [M]
- [ ] **P8-VIDNI-06** (HIGH) Publish vidni: publish the GitHub release and the flake; the maintainer announces it (amended A8). *Done when:* the release page shows the artifacts, their checksums and attestations, and the quickstart works from the published release. [S]
- [ ] **P8-VIDNI-07** (LOW) Write the contribution guide, issue templates and code of conduct. *Done when:* the files exist and the README links them. [S]
- [ ] **P8-VIDNI-08** (LOW) Write the assistant protocol, `docs/assistant.md`, for any LLM: it may edit only `vidni.toml` and `notes.md`, must run `vidni` commands, must respect every gate, and must never upload. *Done when:* the document exists and `vidni validate` blocks anything the protocol forbids. [S]
- [ ] **P8-VIDNI-09** (LOW) Write a Claude Code skill as a reference implementation of the assistant protocol: it drafts lines and scenes from a topic and a facts file, and applies the requests in `notes.md`. *Done when:* a fresh session takes a topic to a draft and through one change round using only what the protocol allows. [M]

### P9. Later
- [ ] **P9-VIDNI-01** (HIGH) Add an opt-in YouTube Data API upload, with a confirmation per upload and the OAuth token stored outside the project. *Done when:* tests against a mocked API cover the confirmation, the upload and the error paths, and the docs state that no live upload has been made (amended A4). [M]
- [ ] **P9-VIDNI-02** (LOW) Build for macOS, then Windows: the builds are unsigned, and the docs explain the OS warnings (amended A5). *Done when:* the hello example renders on each OS from its packaged build on CI runners. [L]
- [ ] **P9-VIDNI-03** (LOW) Support other languages: per-line translations and dubbed cuts, with the creator's voice per language. The second language is Tamil: the assistant prepares the translations, and the maintainer corrects them and records the takes (amended A6). *Done when:* the nibli example has a Tamil cut built from its own takes. [L]
- [ ] **P9-VIDNI-04** (LOW) Evaluate more draft voices, starting with Kokoro, seeded, with pinned weights and offline loading, measuring repeatability, intelligibility and licence; naturalness is not judged (amended A7). *Done when:* `docs/voices.md` records the results, and an accepted voice is pinned by sha256. [M]

## Where the reference pieces go (`reference/`)

| Reference file | What it holds | Destination |
|---|---|---|
| `tts.py` | lint, `cues_for`, best-of-N, `heard`/`closeness`, `spoken`, narration assembly, SRT, `script.md` | `vidni-core` (cues, lint, timeline) and `vidni-media` (TTS, whisper) |
| `capture.mjs` | highlight boxes clipped to scroll containers, KR line ranges, camera rects, the pinned-text check, card rendering | `vidni-media` capture (CDP) |
| `compose.py` | `crop_rect` (top and bottom reserves), crossfades, caption pieces, two-pass loudnorm, remux | `vidni-media` compose and encode |
| `qa.py` | stream, duration and loudness checks; review frames | `vidni-media` QA |
| `record.py` | single-key prompter, meter, silent-take guard, mic check, the test hook | `vidni-record` and `vidni record` |
| `flake.nix` | pins | the vidni flake (pins bumped on vidni's own schedule) |

## Platform and monetization notes

Checked 2026-11-13; confirm each in the app before relying on it.

- **YouTube Shorts:** up to 3 minutes ([YouTube Help](https://support.google.com/youtube/answer/15424877)). Any Content ID claim blocks a Short over 60 s worldwide.
- **Instagram Reels:** uploads up to 20 minutes (15 through the API), but Reels over 3 minutes aren't recommended to non-followers.
- **WhatsApp Status:** up to 90 s; only the first 90 s of a longer video are shared ([WhatsApp Help](https://faq.whatsapp.com/643144237275579), checked for P2-VIDNI-07). An earlier note here said 60 s.
- **TikTok:** banned in India, per the latest official statements (August and September 2025). It's available elsewhere.
- **YouTube Partner Program** ([YouTube Help](https://support.google.com/youtube/answer/13429240)):
  - *Full monetization:* 1,000 subscribers, plus 4,000 public watch hours in 12 months or 10M Shorts views in 90 days.
  - *Fan-funding tier:* 500 subscribers and 3 uploads in 90 days, plus 3,000 watch hours or 3M Shorts views.
  - *From 1 February 2027:* coverage reports that new applicants will need 8,000 hours or 20M Shorts views. Verify on YouTube's page.
