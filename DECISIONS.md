# vidni decisions

Every decision has a date and a ruling, or "default" where the maintainer accepted the TODO's proposal. Rulings come from the maintainer, dhilipsiva. When a ruling changes an item's *Done when* clause in `TODO.md`, the change is recorded under Amendments with its reason. Technical choices made while building go at the end, numbered T1, T2 and so on.

## Product decisions

### D1. Name, home and licence
- **2026-11-13, ruled.** The name is `vidni`, Lojban for "video screen". The repository is <https://github.com/dhilipsiva/vidni> and is public from the first commit (R1).
- The licence is **MIT OR Apache-2.0** (`LICENSE-MIT`, `LICENSE-APACHE`) (R2).
- The maintainer's assistant makes the releases; the maintainer announces them (R4).

### D2. Project file
- **2026-11-13, ruled.** Each video's project file is `vidni.toml`, in TOML.
- It is edited with `toml_edit`, so comments and layout survive edits from both the app and the creator. This replaces a YAML ruling from earlier the same day, made before the UI was known to be native Rust.
- The schema is versioned, and a JSON Schema is generated from it for editors.

### D3. Architecture
- **2026-11-13, ruled.** vidni is a Rust workspace of five crates: `vidni-core`, `vidni-media`, `vidni-record`, `vidni-cli` and `vidni-app`.
- It is ported from the reference pipeline one stage at a time, and each stage must reproduce the reference's golden outputs.
- ffmpeg, Chromium (driven over CDP), Piper's voice models and whisper.cpp stay external tools, pinned by version and sha256.

### D4. Large assets
- **2026-11-13, ruled.** Frozen stills and takes stay in the project folder, out of git.
- A committed `assets.lock` records the sha256 of each.
- Backups go to two destinations, one of them off the machine.

### D5. Draft voice
- **2026-11-13, default.** The draft voice is Piper `en_US-joe-medium`, whose training data is CC0, seeded so it repeats.
- Voices come as packs that carry licence metadata. A voice under a non-commercial licence is flagged.
- Kokoro is evaluated later (P9-VIDNI-04).

### D6. Platforms
- **2026-11-13, ruled.** There are profiles for YouTube long-form, YouTube Shorts, Instagram Reels, TikTok, WhatsApp Status, LinkedIn and X.
- Enabled by default: **YouTube long-form, YouTube Shorts, Instagram Reels and WhatsApp Status** (R14). TikTok, LinkedIn and X are off by default.
- Regional notes live in the docs. For example, TikTok is banned in India, per the latest official statements (August and September 2025).

### D7. Captions
- **2026-11-13, default.** The long-form master has no burned-in captions; its SRT/VTT is uploaded separately.
- A captioned variant is made for feeds that autoplay muted.
- Shorts are always captioned.

### D8. Uploads
- **2026-11-13, default.** Uploads are manual, from the generated upload folders.
- The YouTube Data API upload is opt-in (P9-VIDNI-01) and tested only against a mock (R9).

### D9. UI toolkit and preview
- **2026-11-13, ruled.** The app uses egui/eframe with AccessKit.
- The preview decodes frames through the same pinned ffmpeg the build uses, not libmpv (R15).

### D10. Operating systems
- **2026-11-13, ruled.** Linux comes first, through a Nix flake and a portable bundle.
- macOS and Windows get unsigned builds from CI runners (R10, P9-VIDNI-02).

### D11. Task runner
- **2026-11-13, ruled.** Any command that recurs gets a Justfile recipe.

### D12. Default look
- **2026-11-13, ruled.** The default theme is a neutral vidni theme using IBM Plex Sans and IBM Plex Mono (OFL), with Noto Sans Tamil for Tamil text (R16).
- The QUINE theme ships as an example in `examples/themes/`.

## Process rulings, 2026-11-13

- **R1. Public from day one.** Every item is committed to `main` and pushed. The reference pipeline's git history goes public when it moves into this repository (P1-VIDNI-03). Takes, captures and models stay out of git (D4).
- **R3. CI.** GitHub Actions runs lint, test and the parity checks on Linux, with every action pinned by commit SHA. macOS and Windows runners come with P9-VIDNI-02.
- **R5. Release signing.** Each release ships `SHA256SUMS` plus GitHub build-provenance attestations, which `gh attestation verify` checks.
- **R17. Moving the reference's takes.** The maintainer's takes are moved, not copied, into `examples/nibli-syllogism/`, where they stay out of git and are hash-locked. Every hash is checked before the originals are removed, and a README in the old folder points to the new location.
- **Branching.** Work happens on `main`, which is pushed after every item. This is the maintainer's standing rule to work on the current branch.

## Amendments to *Done when* clauses

| Ruling | Item | Amendment | Reason |
|---|---|---|---|
| A1 (R6) | P3-VIDNI-02 | Determinism is recorded on this desktop and in a clean docker container, instead of on three NixOS hosts. The other hosts become an optional later check. | The build machine can't reach the other hosts. |
| A2 (R7) | P6-VIDNI-02 | The safe zones are confirmed from grid-overlay test clips. vidni generates the clips, the maintainer uploads them privately to YouTube Shorts, Instagram Reels and WhatsApp Status, and the zones are measured from the screenshots they share. | Only the maintainer can upload to their accounts. |
| A3 (R8) | P8-VIDNI-01 | A fresh agent session that has never seen the code follows only the docs in a clean Linux container, instead of an outside person. A real person may follow later. | No outside tester is available now. |
| A4 (R9) | P9-VIDNI-01 | Tested against a mocked API only; no live upload is made. | The maintainer ruled out a live test in this goal. |
| A5 (R10) | P9-VIDNI-02 | macOS and Windows builds are unsigned, and the OS warnings are documented. | There are no signing certificates. |
| A6 (R11) | P9-VIDNI-03 | The second language is Tamil. The assistant prepares the translations, and the maintainer corrects them and records the takes. | The maintainer chose the language and records the voice. |
| A7 (R12) | P9-VIDNI-04 | The voices are judged on measurements only, and naturalness is recorded as not judged. | The maintainer chose not to do listening tests. |
| A8 (R4) | P8-VIDNI-06 | The maintainer announces the release, and the announcement is not part of *done*. | Announcing belongs to the maintainer. |

## Technical decisions

Added as the items are built.

### T1. Comparing MP4 packets across ffmpeg versions
- **2026-11-13, P1-VIDNI-04.** The parity harness compares a cut's packets by a fingerprint of its framemd5 log, not by the log's hash.
- The fingerprint drops the `#software` line and the side data listed after a packet's hash. It keeps the extradata, the stream parameters, and every packet's timing, size and payload MD5.
- **Why:** the golden logs were written by the reference's ffmpeg 8.1.3, and vidni pins ffmpeg 9.0.2. For the same file, ffmpeg 9 writes a different `#software` line and lists the last AAC packet's skip-samples side data, which ffmpeg 8 doesn't. Every packet payload is identical, so the fingerprint changes only when the streams do.
- `golden/hashes.json` keeps the hash of each whole log, and the harness checks each committed log against it before reading it.

### T2. How schema 1 reports problems
- **2026-11-13, P2-VIDNI-01.** The project model is plain structs that deny unknown keys, not serde's tagged enums. Rules that reach across tables are checked after parsing, and each finding is placed in the file by its key path, through toml_edit's spans.
- **Why:** a tagged enum reads its whole table before choosing a variant, so a problem inside a scene could only be reported at the scene's header. Flat structs keep each type error and unknown key at its own line and column, and the key path does the same for the cross-table rules. A key that belongs to the other kind of scene is still caught, by a rule.

### T3. What a capture's key covers
- **2026-11-13, P2-VIDNI-05.** A web line's capture is keyed by three things:
  - its scene without the cameras;
  - every action run on the page so far, in script order;
  - its highlights.

  A page render is keyed by its scene and its size.
- **Why:**
  - A still depends on the page and on how the page got there, not on how the still is framed later. So changing a camera never re-captures.
  - The page carries state from line to line (a clicked tab, a scroll position), so a line's key has to include the actions before it.
  - Lines that show the same state with the same highlights share one capture. The reference captured each line separately.
