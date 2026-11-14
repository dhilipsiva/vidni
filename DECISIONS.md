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
- **Clauses that can't be met.** When a *Done when* clause turns out to be impossible as written, it is amended in the table below with the measured reason, never weakened quietly. This is step 4 of the ratified plan for each item.

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
| A9 (clauses that can't be met) | P3-VIDNI-02 | The 20 frames per cut are compared with the reference's own frames before encoding, instead of with frames decoded from golden's MP4s. `tools/reference_frames.py` regenerates them with `reference/compose.py`. The reference's MP4s are still checked against golden first, and `just golden` shows that the reference re-derives them byte for byte, so these are the frames golden's MP4s were encoded from. | Golden's MP4s are x264 at CRF 18. While the camera moves, compression alone keeps the reference's own frames below SSIM 0.995 against them: 0.9811 at 206.74 s of the 16:9 cut, and 0.9860 at 49.24 s of the voice 16:9 cut. No port could pass there. Against the frames before encoding, the check measures the compositor alone, and vidni's frames at those two times differ from the reference's by at most one level of 255. |

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
- **2026-11-13, P2-VIDNI-05.** A web line's capture is keyed by:
  - its scene without the cameras;
  - every action run on the page so far, in script order;
  - every expectation waited for on the page so far (added in P3-VIDNI-04: capture waits for an expectation before it takes a still, so adding one to fix a stale still must retake it);
  - its highlights.

  A page render is keyed by its scene, its size, and the cut's aspect, which `{aspect}` in its address becomes. (The aspect was added in P3-VIDNI-04: an overlay is wider than tall in both aspects, so the size alone can't say which to render.)
- **Why:**
  - A still depends on the page and on how the page got there, not on how the still is framed later. So changing a camera never re-captures.
  - The page carries state from line to line (a clicked tab, a scroll position), so a line's key has to include the actions before it.
  - Lines that show the same state with the same highlights share one capture. The reference captured each line separately.

### T4. How the compositor draws
- **2026-11-13, P3-VIDNI-02.** The compositor reproduces Pillow's arithmetic instead of drawing with tiny-skia. Its resampling (BICUBIC over a float box), blending, pasting, lines, rounded rectangles and mask fills are ports of Pillow's C. Fixtures written by Pillow itself (`tools/pillow_fixtures.py`) test each one byte for byte.
- Text is shaped by harfrust, one run per line, as libraqm shapes it for Pillow. Positions are rounded to 26.6 as FreeType and HarfBuzz round them, and swash rasterizes the glyphs through cosmic-text's cache. Widths match Pillow's `getlength` exactly, and drawn text differs from Pillow's by at most 2 levels of 255, at glyph edges.
- resvg isn't used yet: cards and overlays arrive as PNGs that the browser rendered (P3-VIDNI-04).
- **Why:** the frames must reach SSIM 0.995 against the reference's at every sampled time, and one-pixel shifts were what kept most of them below it.
  - cosmic-text's own layout shapes word by word and sums float advances. A caption measured 1281 px where Pillow measured 1280.9375, so its box came out a pixel wider and the box and text shifted by a pixel.
  - With that layout, 15 of the 80 sampled frames fell between SSIM 0.987 and 0.995 against golden's MP4s.
  - Matching Pillow's arithmetic removes those differences at their source, and the ports are small. It left two sampled frames below 0.995, and those were golden's own compression in the middle of a camera move (A9).

### T5. How the draft voice repeats
- **2026-11-13, P3-VIDNI-03.** The draft voice is the pinned Piper CLI, patched in vidni's flake to take `--seed` (`nix/piper-seed.patch`), not `piper-rs` over `ort`.
  - **Why:** Piper 1.8 draws its sampling noise inside the voice's ONNX graph, from generators that onnxruntime seeds when a session starts.
  - onnxruntime's Python API can set that seed, so the patch is a few lines, and the same seed then gives byte-identical PCM. Without it, two runs of the same line differ.
  - `ort` exposes no such seed. Seeding there would mean rewriting each voice's graph to take its noise as an input.
  - Piper ignores flags it doesn't know, so vidni checks that its Piper lists `--seed` and refuses one that doesn't, rather than draft unrepeatable takes.
- A line's first draft speaks three takes and keeps the one of median length.
  - **Why:** whisper's score would pick better, but whisper is advisory only and never decides an output (P3-VIDNI-05).
  - Length is deterministic and catches Piper's usual failures, a stretched or dropped word.
  - The kept seed goes into `voice.lock`, a committed file, so a creator can also choose a take by writing a seed.
- The voice packs vidni ships pin each file by size and SHA-256 at one revision of rhasspy/piper-voices, and carry the licence their model card gives.
  - joe (CC0) is the default, and ryan (CC BY-NC-SA 4.0) shows the warning for a non-commercial voice.
  - libritts_r is left out. Its data is CC BY 4.0, but it was fine-tuned from lessac, whose recordings are licensed for research only (the Blizzard 2013 research licence), so its status is unclear.

### T6. How capture drives Chromium
- **2026-11-14, P3-VIDNI-04.** Capture speaks the DevTools protocol through a small client of vidni's own (`vidni_media::cdp`), not chromiumoxide.
  - **Why:** golden's stills were taken by Playwright, so capture repeats Playwright's calls: its switches, window size, device metrics, emulated media, font families, screenshot clip and mouse events. A small synchronous client sends exactly those, without an async runtime.
  - It talks over Chromium's pipes (`--remote-debugging-pipe`), not a WebSocket on a local port. A port would let any local user attach to the browser, and with pipes Chromium exits when vidni does. The `command-fds` crate hands Chromium the two pipes, so the workspace still has no unsafe code.
- **Three things keep a still the same from one launch to the next.** Each was found by capturing one page many times.
  1. **A web scene above scale 1 is loaded twice.** At an emulated scale, the first load after launch came out in one of two renderings: the layout was the same, but text in a TrueType font was painted one CSS pixel lower. A second load in the same browser gave one rendering. The first load is inert (no scripts, and its cookies cleared), so the captured load is still a first visit.
  2. **Each layer is rastered as one tile.** Chromium placed its raster tiles differently from launch to launch, and the edge of a curve came out up to 4 levels of 255 different (an icon in the example's site header). With the tile switches, 40 captures of 40 agreed where 5 of 40 had differed.
  3. **vidni's style and highlight layer are mounted at each document's start,** not after it has loaded. Mounted late, 9 captures of 70 differed in that icon; mounted at the start, 1 of 70, before the tile switches.

  With all three, a local copy of the example's site header at scale 3 gave one still in 80 captures of 80, the test scene one in 30 of 30, and five captures of the live example agreed in every pixel.
- **What was tried and dropped:**
  - Starting Chromium at the scale itself (`--force-device-scale-factor`) was steady, but it lays pages out differently. Against golden, 75 of 118 checks failed, with SSIM down to 0.93.
  - Rendering a blank page first, in place of the first load, steadied text but not the icon.
  - `--font-render-hinting=none` and `--disable-partial-raster` made no difference to any of this.
- **Against golden:** every still and card passes at SSIM 1.0000, and each still's frames are within 0.5 CSS px of the reference's. No still is byte-identical any more: one tile per layer moves anti-aliasing at a few edges, 548 to 812 pixels of 8.3 million in each still, at most 28 of them by more than 16 levels.
- **The page runtime** (`capture/runtime.js`) generalizes the reference's nibli-only helpers to the scene schema, and keeps their geometry:
  - boxes clipped to their scroll containers;
  - a text line's box padded by 6 and 3 pixels;
  - highlight boxes drawn 4 pixels outside their target;
  - merged groups drawn as one box.

  It runs in an isolated world, as Playwright's does, so a page can't change it.
- **Settling** replaces the reference's fixed waits: running animations finished, fonts loaded, network quiet, and two screenshots a quarter second apart identical. Expectations are waited for, so a late result can't leave a stale still.
- **A click** scrolls its target into view and refuses to land on anything covering it, as Playwright's does. The first port clicked a row hidden by its scroll panel, and the still that followed showed the wrong state.
- **The sandbox stays on.** Playwright turns it off; stills are the same either way.
- **Fonts:** the flake's fontconfig file lists only pinned fonts. `makeFontsConf` had let the host's fonts in, against D12. Golden was made with the host's fonts, so the parity recipes set the flake's file aside, and their result holds on the maintainer's desktop only.

### T7. How the listening check listens
- **2026-11-14, P3-VIDNI-05.** The check runs the pinned `whisper-cli` (whisper.cpp 1.9.5, from the flake) as a process, with the `ggml-base.en` model, not `whisper-rs`.
  - **Why:** the check only advises, so nothing is gained by linking whisper in. `whisper-rs` would compile whisper.cpp, a C++ build through cmake, into every build of vidni, for a tool no output depends on.
  - The command is the one the reference ran, and a whisper that fails stays outside vidni's process.
  - The model is pinned by size and SHA-256 at one revision of `ggerganov/whisper.cpp` on Hugging Face. The flake fetches it, and vidni refuses another file.
- **Nothing it hears reaches an output.** `vidni hear` writes no file, and no build step reads a score.
  - The reference chose between Piper's takes by this score. vidni's draft keeps the take of median length instead (T5).
- **Empty and silent clips are never given to whisper.** Measured with the pinned whisper-cli and model:
  - A clip with no samples makes it exit 11.
  - It hears "you" in 2 and in 30 seconds of digital silence, and in noise at −66 dBFS.
  - It answers a tone, or louder noise, with a note in parentheses, such as `(eerie music)`.

  So a clip shorter than 0.3 seconds, or never above −60 dBFS, is reported as having nothing to hear without asking. The notes are left out of what was heard, and a transcript that holds only notes is nothing to hear as well.
- **The score is the reference's,** ported exactly: Python's `difflib.SequenceMatcher(autojunk=False).ratio()` over the letters of the two texts.
  - The port gives the reference's score, to four decimals, for each of its 57 lines.
- **`voice.heard_as` replaces the table of homophones the reference had built in.** Those were the nibli video's words (`nibley` for `nibli`, `atom` for `adam`), so they now live in that project's file.
  - The key sits in `[voice]`, which no cut's ratified content covers. Whatever later hashes a release must leave it out as well.
- **The check knows English only:** the model is English, and the score compares the letters a to z. `vidni hear` refuses a script in another language instead of scoring it. P9-VIDNI-03 will need a multilingual model and a score over Unicode letters.

### T8. How a cut is built, released and checked to repeat
- **2026-11-14, P3-VIDNI-06.** A build names everything it is made from by hash, in two layers.
  - **Sources** are the cut's own: what it shows and says (the hash that ratifying freezes), each line's clip by its place and its bytes, each capture's files, and the look (the gaps, the title and the cut's settings). None needs a tool to work out, so `vidni status` can tell how a cut stands against its release without rendering anything.
  - **Inputs** add the tools and the timeline: vidni's and ffmpeg's versions, each font file, the encoder's settings, and the timeline the build starts from. Two builds with the same inputs must give the same T1 and T2 hashes.
  - **What is left out, on purpose:**
    - `voice.heard_as` (T7).
    - The draft voice's settings, in a final build. Its timeline writes what the creator reads, not what the draft voice is given, so a respelling changes no release.
    - The resolved project. It is the whole project's, so an edit to another cut would change it; `vidni repro` compares it once, for the project.

    Tests hold each of these.
- **A still's frames hold its targets, and cameras are framed when a cut is built.**
  - A capture's key leaves the cameras out, so that a camera can change without a new capture (T3). The first wiring read each camera's box from the frames file, as it was measured at capture. So a camera edited afterwards never reached the picture, and ratifying froze the stale framing. The review found it.
  - Now `<key>.json` holds the boxes of every target of the scene, and a camera is the union of its targets' boxes, worked out at build time as the page runtime worked it out. `just parity stills` still passes 118 of 118, with the cameras framed this way.
  - Ratifying hashes the frames file with the still, and a still without usable frames counts as not taken. A still whose picture a ratified cut froze is never taken again for want of its frames: a page may not give the same picture twice.
- **A 9:16 cut freezes the project's subtitle,** because its title band shows it. A 16:9 cut draws none.
- **Approving writes a record, not a git tag.** A release is the final build's record in `release.json`, under a tag such as `long-r1`.
  - **Why:** a project folder needn't be a git repository, and tagging and pushing are the creator's to do (TODO.md marks them HIGH).
  - Releases are only ever added, so `release.json` is each cut's history. It names its layout (`"schema": 1`), since it outlives the vidni that wrote it.
  - The release's day is read from the clock, the one place vidni reads it. `SOURCE_DATE_EPOCH` isn't used for it: Nix shells set it to 1980.
- **`vidni repro` found that the encoder didn't repeat.** On the nibli example, two clean builds of the 9:16 cut gave the same frames and different packets, in a stretch of 222 frames after one crossfade.
  - **The cause** was x264's threaded lookahead, which x264 turns on when it has more than one thread. Encoding 451 frames around that crossfade 40 times gave 8 different packet streams, on the same cores as well as on different ones.
  - **The fix** is `sync-lookahead=0`, which runs the lookahead in the encoder's own thread. With it, 40 encodes on 1, 2, 3, 4 and 32 cores gave one packet stream, in the same time.
  - P3-VIDNI-01's check had encoded twice and seen the same packets. Twice wasn't enough.
  - **Ruled out on the way:** the conversion from RGB to YUV gives the same bytes on 1 to 32 cores for those frames, and `lookahead-threads=1`, `deterministic=1` and `direct=spatial` each left the packets unsteady.
- **What repro varies, and what it compares.**
  - The two builds run at once, on two cores and on three others, from copies of the project at paths of different depth. A copy holds only what a build reads: the project file, the locks, the files its scenes name, the current takes and the captures.
  - Their environment holds only `PATH`, a fixed time zone and locale, `SOURCE_DATE_EPOCH`, a home and a temporary folder of their own, the font folders and, for a draft, the voice store.
  - They are compared with each other, and with the creator's own build of the same inputs. That build was made in the creator's environment, so a build that depended on the time zone, the locale or the home folder would differ from them.
  - As a check of the check: with the encoder's thread count made to follow the cores, repro reports the packets and files as different and the frames as equal.
- **`--offline` runs the command under `unshare --user --map-user --map-group --net`.**
  - Mapped to root instead, as `unshare -r` maps it, Chromium refuses to start. vidni replaces itself with the command inside the namespace, so signals and the exit code are the command's own.
  - The command inside checks that it has no route to the network, so that the mark it carries can't be set by hand to skip the namespace. It looks at routes, not at interfaces: where tunnel modules are loaded, every new namespace lists their idle devices.
  - With no network, anything a page asks the network for fails the capture. A page that is a file of the project's is held to that with a network too, so a still taken while a part was missing is never kept as if it were whole.
  - Names can still be looked up where the host's resolver answers through a socket in the file system, which a network namespace doesn't cover. Nothing can be connected to.
- **Fonts are pinned as voices are:** by size and SHA-256 in `fonts.toml`, and looked for only in the folders that `VIDNI_FONTS` names.
  - The flake's IBM Plex Mono Bold (2.5.0) isn't the file the reference drew with (1.1.0, from the maintainer's system). So vidni's own builds differ from golden wherever mono text is drawn, and so do captures taken with the flake's fonts.
  - `just parity product` therefore checks the wiring with the reference's fonts. It plans the example's final build as `vidni final` does, from fresh captures and the creator's takes, and compares it with the reference's cuts in the creator's voice.
- **A draft is one picture for each cut.** The final build makes every picture the cut's `captions` asks for.
- **An independent review** of the change found no way past a gate, and 21 other things. The ones that changed the design are above: the cameras, what a record leaves out, the subtitle, the frozen stills and the page that loses a part. The rest were fixed where they stood: repro built `vidni.toml` whatever file it was given, copied the whole folder, and could leave its copies behind on an interrupt; a language tag could put a slash into a file name; and an encoder that stopped was reported as a broken pipe.
- **Limits, for now:**
  - The compositor draws 1920×1080 and 1080×1920 only, and a build refuses another size.
  - Loudness is normalized by ffmpeg's loudnorm in two passes, as in the reference, and QA allows 1.5 LU around the target. On a very short cut the result falls short: a cut of 2.9 seconds measured −19.9 LUFS, which QA refuses. P4-VIDNI-03 revisits the voice chain.
  - A record names vidni by its version. Between versions, a change to what vidni draws must raise the compositor's own count (`COMPOSITOR`), or an earlier build would still pass as current.
  - When a rebuilt cut fails QA, `out/` keeps the files of the build before it. `vidni approve` refuses the cut until a build of it passes.

### T9. What each stage keeps, and when it makes it again
- **2026-11-14, P2-VIDNI-09.** Three stages keep what they make: the draft voice its clips, capture its stills and renders, and a build its files. Each is kept by what it was made from, so a second `vidni draft` with nothing changed does nothing. On the nibli example it takes 0.6 seconds, where the first takes minutes.
- **A draft clip** is filed under a key of what the voice is given, the pack's files and the pace, with its seed in its name (T5). That was in place.
- **A capture's key can't cover what a page holds,** only what the project file says about it (T3). Before this, an edit to a card's HTML left its render as it was. Now a capture of a page in the project folder has a record beside it, `<key>.files.json`: the project's files the page loaded, each with its SHA-256, as Chromium reports its requests. The capture is out of date once one of them isn't what it was.
  - **Why a record, and not a part of the key:** which files a page loads is only known once it has loaded. A stylesheet's import, a picture a script asks for after a click: none of it is in `vidni.toml`. A key has to be worked out before the capture, so it could only cover the page's own file, and an edit to a stylesheet would be missed.
  - **What that costs:** a capture's file is replaced when it is taken again, so the earlier one isn't kept beside it. A cut that froze it then reads `changed`, as it should: it no longer shows the project. With the file put back, the capture comes out the same on the same machine, and the cut is the ratified one again. The end-to-end test holds that.
  - **The key stays as it was,** so no capture of the nibli example was renamed, and its releases stand. Its eight card renders had no record. `vidni draft` took them again, and each came out with the hash `ratified.lock` had frozen.
  - **A page on the web has no record.** Nothing can tell whether it has changed without loading it. Its capture is kept until it is deleted (`vidni unratify --refresh`).
  - **A page of the project's may load only the project's files.** A file from elsewhere on the machine fails the capture: no record could tell when it changes, and a copy of the project wouldn't have it. `vidni repro` copies the files in each record along with the captures.
  - **A file that changes while its page is captured** fails the capture. Each file is hashed when the page first asks for it, the page's own file before Chromium reads it, and again when the still is taken.
  - **The rule about frozen stills is narrower now** (T8). A still whose picture a ratified cut froze is still never taken again for want of its frames. One that is out of date is, since the files that changed are the creator's own and can be put back.
- **A build is kept by its record.** The record already named the build's `inputs` and each file it made (T8). `vidni draft` and `vidni final` now work out a cut's `inputs` first, and render nothing when the record on disk has the same ones and each file still has its hash. `--again` renders anyway.
  - **Why this is safe:** it rests on what `vidni repro` checks, that the same inputs give the same files.
  - **Where it isn't:** a record names vidni and ffmpeg by their versions (T8, limits). While vidni is being changed, a build from before the change is kept unless the compositor's count is raised, and so is one made by another ffmpeg that prints the same version. `--again` is for that, and the tests build in fresh folders.
  - A final build that is kept is still delivered: a file missing from `out/` is copied again, and one that is there byte for byte is left alone.
- **Reading files once.** A status or a draft hashes each of the project's files once, however many captures were made from it.

### T10. How the recorder keeps a take
- **2026-11-14, P4-VIDNI-01.** The recorder knows its sound only as an input: blocks of samples with one channel. The microphone is one input, through cpal; a made-up signal is another, and the tests record from that one through the same path. cpal's own custom devices need `unsafe`, which the workspace forbids.
- **The trim is the reference's, sample for sample.** The reference's recorder trimmed a take with ffmpeg: `atrim`, `highpass=f=70` and `silenceremove` at each end. vidni does the same in Rust, in ffmpeg's arithmetic:
  - the high-pass is ffmpeg's biquad for 16-bit samples: single precision, its order of operations, a sharpness of 0.707, and a result cut off towards zero;
  - the level is `silenceremove`'s: the RMS of the window so far, from a running sum of squares in single precision.

  `just parity takes` holds this against the nibli example. Each of its 57 kept takes is, sample for sample, vidni's trim of one of the line's raw takes with ffmpeg's splice applied. With the filter's result rounded instead of cut off, or with 0.16 s kept instead of 0.15 s, none of the 57 passes.
- **The one difference is ffmpeg's splice.** Where `silenceremove` joins the silence it keeps to the sound, it writes the oldest sample in its buffer, from 0.15 s before, and then repeats the sample the sound begins at. vidni keeps the stretch whole. The two samples sit in the silence around the words, so nothing can be heard of them, and a take is as long either way. When the words begin less than 0.15 s into a take, ffmpeg's take is one sample longer.
- **The stop key's trim is 0.10 s, not the 0.30 s the reference asked for.** Its chain reads `atrim=start=0.20,areverse,atrim=start=0.30,areverse`. After the first trim the sound's timestamps start at 0.20 s, and reversing keeps them, so the second trim drops what is before 0.30 s: a tenth of a second. The 57 takes were made that way and sound right, so vidni drops 0.10 s.
- **A raw take is written once.** Its file is created new and never opened for writing again. The trimmed take waits as `<hash>.wav.new`, a name no take has, until it is kept or discarded.
- **The microphone is asked for 48 kHz** when it offers that, with as few channels as it has, and is mixed down to one. The takes so far are at 48 kHz, and it is the rate the cuts are encoded at.
- **The microphone's path is tested on ALSA's `null` device,** an input that gives zeros and needs no sound card, so the test hears nobody (`just test-record`). The maintainer's microphone was not recorded from in this work.
- **2026-11-14, P4-VIDNI-02: the terminal recorder.** `vidni record` is the reference's recorder, key for key, on crossterm, with the project's own lines in the place of numbered files.
  - **The review settles once.** Keys are ignored for 0.4 s when the review appears, so that the key that stopped the take, pressed twice, doesn't keep or discard it. The reference settled again after every playback; vidni doesn't, so a take can be kept as soon as it has played.
  - **Esc while recording leaves no raw take.** The reference left the file. A take that was stopped is a raw take; one that was cancelled was never a take.
  - **Reviewing a section** is `s`, which plays the section's kept takes one after another, and `--section`, which goes through one section's lines.
  - **The tests run it in a pseudo-terminal** with a made-up input (`VIDNI_RECORD_INPUT`) and ALSA's null output, so they hear nobody and make no sound. The listening check's part is tested with a recording as the input, in `just test-hear`.
  - **Playback** stretches a take to the output's rate by straight lines between samples when the output doesn't offer the take's rate. It is for listening, and no output is made from it.

### T11. What a backup holds, and where it goes
- **2026-11-14, P4-VIDNI-05.** D4 rules that assets are listed in `assets.lock` and backed up to two places, one of them off the machine. `vidni backup`, `verify`, `restore` and `gc` do that.
- **A place is a folder.** vidni copies files and checks hashes; it doesn't speak to another machine itself. The place off the machine is a folder mounted from it, or a disk that is carried away. So vidni needs no network tool and no credentials, and the same code is tested on two folders.
  - It can't tell where a folder really is. It refuses fewer than two places, refuses a place that isn't there (so that an unmounted folder isn't quietly made on this disk), and warns when every place is on the project's disk.
  - The places come from `--to` or `$VIDNI_BACKUP`, not from `vidni.toml`: they are one machine's paths, and the project file is shared.
- **The assets are the takes and what ratified cuts froze.** A capture of a cut that isn't ratified can be taken again, so it isn't one. `assets.lock` stays in `sha256sum`'s format, sorted by the bytes of the path, so that the list doesn't depend on a locale.
- **A backup also archives what each cut's newest release delivered,** under `releases/<tag>/`. A cut's file is promised to repeat on one machine only (T3), so the delivered file itself is kept. It is archived only when `out/` holds the very file the release records.
- **A backup only gains files,** and a restore only fills in what is missing. A file that is there and differs from the list is left alone: it may be a take recorded since.
- **`gc` deletes by kind, and the kinds are narrow:** draft clips, drafts, final builds, captures that no cut shows and none froze, and files written aside and never moved in. A capture that an unratified cut shows is kept, though it could be taken again: the page may have changed.
- **Checked on a copy of the nibli example,** not on the maintainer's own files: backed up to two folders (269 assets and 6 delivered files each), with `gc` deleting 413.5 MB of draft clips, drafts and final builds; then the takes, `build/` and `out/` wiped. `vidni restore` brought back 275 files, and `vidni final` built both cuts again in four minutes, with the frames, packets and file hashes that `release.json` holds for `long-r1` and `vertical-r1`.
