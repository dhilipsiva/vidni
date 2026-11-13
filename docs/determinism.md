# Determinism

vidni's outputs are deterministic in three tiers (TODO.md, rule 2):

- **T1:** the resolved project, the timeline, the captions and metadata, the rgb24 frame stream before encoding, and the PCM before encoding are byte-identical on any x86-64 host with the same `flake.lock`.
- **T2:** an MP4's packets (`ffmpeg -map 0 -c copy -f framemd5`) and its file bytes are identical across runs on one host.
- **T3:** across hosts, MP4s are compared by SSIM and are not promised to match. The delivered files are archived.

Some inputs can't be made deterministic: a live web page, every Chromium render, sampled TTS across hosts (seeded, it repeats on one host), whisper, the creator's takes and the clock. They are frozen by hash when a cut is ratified, and whisper never feeds an output.

## Verified

| Date | Host | What | Tier | Evidence |
|---|---|---|---|---|
| 2026-11-13 | the maintainer's desktop: AMD Ryzen 9 9950X3D, NixOS, Linux 7.3 | the reference pipeline, both variants, both cuts | T2 | `just golden` re-derived 178 inputs and 14 outputs identically on independent runs, MP4 file bytes included |
| 2026-11-13 | same | the timeline and captions (Rust) | T1, equal to golden | `just parity timeline`: both variants' timeline JSON and SRT equal the reference's |
| 2026-11-13 | same | the narration track (Rust) | T1, equal to golden | `just parity narration`: both variants' PCM equal the reference's |
| 2026-11-13 | same | the encoder (Rust), ffmpeg 9.0.2 with libx264 | T2 | `just encode-t2` encoded the 9,188 golden frames of the 16:9 cut with the golden narration twice: the same framemd5 and the same 25,089,781 bytes |
| 2026-11-13 | same | the compositor (Rust), 20 sampled frames of each of the four cuts | equal to the reference's frames before encoding (DECISIONS.md, A9) | `just parity cuts`: 80 of 80 at SSIM ≥ 0.995. The lowest rounds to 1.0000, and no channel value differs by more than 2 of 255 |
| 2026-11-13 | same | the compositor, every picture of the four cuts | equal to the reference's frames before encoding | `just compose-stream`: in each cut, the 1,254 pictures change at the same frames as the reference's and differ from them by at most 2 of 255 (16:9) or 1 of 255 (9:16) |
| 2026-11-13 | same | the compositor's frame streams | T1 | `just compose-t1`: two renders of each cut gave the same rgb24 stream, hashed below |
| 2026-11-13 | a clean `nixos/nix` container on the same desktop: Nix 2.35.6, vidni built from source by `nix build` | the compositor's frame streams | T1, equal to the desktop's | `vidni-parity compose-t1 --fonts …` in the container: two identical renders of each cut, with the desktop's four hashes |
| 2026-11-13 | the maintainer's desktop | the draft voice: Piper 1.8.0 with vidni's seed patch, piper:en_US-joe-medium, on the nibli example's 57 lines | repeatable on one host | `vidni draft` chose each line's take from 3 (171 takes). With the cache cleared, a redraft from `voice.lock` gave the same PCM: the 57 clips' combined sha256 is `59359d31…ffca` both times. A redraft with the lock cleared as well chose the same seeds and gave the same PCM again |
| 2026-11-14 | same | capture (Rust): the nibli example's 53 stills and 8 cards, taken afresh from the live page with the host's fonts | equal to golden | `just parity stills`: 118 of 118 checks. Every still and card scores SSIM 1.0000. Each still differs from golden's in 548 to 812 of its 8.3 million pixels: anti-aliasing at a few edges, from rastering each layer as one tile. Overlay alpha is equal, and each still's frames are within 0.5 CSS px of the reference's |
| 2026-11-14 | same | capture: five captures of a local scene, each in a fresh Chromium | identical on one host | `just test-capture`: the stills and their frames are byte-identical across the five. One line of the scene, captured 30 times, gave one still |
| 2026-11-14 | same | capture: five captures of the nibli example's live page | identical on one host | `just capture-repeat`: the pixels and frames of all 46 stills agree across the five. One still, captured 40 times, gave one result |
| 2026-11-14 | same | capture: a local copy of the example's site header at scale 3 | identical on one host | 80 captures gave one still. Before the three fixes of T6, about 1 capture in 6 differed in a few pixels of an icon |
| 2026-11-14 | same | the listening check: whisper-cli 1.9.5 with `ggml-base.en`, on the nibli example's 57 takes and 57 draft clips | held to no tier; repeated on one host | two runs of `vidni hear`, and two of `vidni hear --draft`, printed the same transcripts and scores, and left the project folder's 223 files untouched |

The compositor's frame streams (rgb24, every frame in order), drawn with IBM Plex Sans Bold (sha256 `19de5aec…c316`), IBM Plex Mono Bold (`44f6da8e…a6c7`) and Space Grotesk Bold (`be8709ab…368f`):

```
nibli-syllogism-16x9        9,189 frames  39d61b473952ca905d463fdb9aac2878067e3099f783a8e156b058d2e2801674
nibli-syllogism-9x16        9,189 frames  e53e6c267b4d116e4860790865af1122af4c54bdb9e5130281eb5fa94baaeed4
nibli-syllogism-voice-16x9  8,441 frames  cbf1414e513c5574596b3d9907dcfc38af6f56badfb890a1f232ab5f2d04cd5c
nibli-syllogism-voice-9x16  8,441 frames  97cc6585c827b80c014d03e7eb9f888cc56835b7a9f7b45f343c0bd71eb74b96
```

## How each stage stays deterministic

**Timeline and captions** (`vidni_core::timeline`):
- Times are rounded to milliseconds as Python's `round` rounds them: correctly rounded, with ties to even.
- Caption lengths are counted in characters, not bytes.
- Cue splitting breaks ties between equally good cuts in a fixed order: score, then position.

**Narration** (`vidni_media::narration`): each clip is placed at sample `floor(start × rate)` on the timeline's rounded start, in a track of `floor(total × rate)` samples, as the reference places them.

**Encoding** (`vidni_media::encode`):
- libx264 runs with a fixed thread count, preset and CRF. Its packets depend on the thread count, so the count never follows the host's cores.
- `+bitexact` applies to the container and both codecs, and `-map_metadata -1` drops metadata. No date or version number enters the file; the encoder tag names the codec without its version.
- The conversion from RGB to YUV is BT.709, with bit-exact, accurate rounding, and the colour tags say so.
- `-cpuflags 0` stops ffmpeg's filters from choosing SIMD paths by CPU.
- Loudness is normalized in two passes before encoding, with the measurement fed back into the second pass.

**Compositing** (`vidni_media::compose`):
- The compositor draws only with the font files it is given and never asks fontconfig or the system for a font. The parity tools pin the reference's three faces by sha256 (`REFERENCE_FONTS` in `vidni-parity`), because fc-match answers with some other font when it has none of the family. `compose-t1` prints each file's path and hash.
- Drawing ports Pillow's arithmetic (DECISIONS.md, T4). Resampling is fixed-point, text positions are whole 26.6 units, and the floating-point steps are plain IEEE operations: no transcendental function is called (the camera's ease is a polynomial).
- A frame whose line and caption match the frame before, with nothing moving, is the same picture, so it is reused instead of drawn again (`FrameKey`).

**Draft voice** (`vidni_media::tts`):
- Piper runs with a seed (DECISIONS.md, T5), so a take depends only on what the voice is given, the voice pack's files, the pace and the seed.
- The pack's files are checked against their SHA-256 before every draft.
- Each line's kept seed is recorded in `voice.lock`, so a draft never depends on what the cache holds.
- The same seed gives the same PCM on one host. Across hosts it isn't promised, since onnxruntime may take different paths on different CPUs.

**Captures** (`vidni_media::capture`) come from a page, which can change, so they aren't promised to repeat. What vidni controls is fixed:
- the pinned Chromium, its switches, the viewport and scale, the emulated media and the font families;
- the fonts fontconfig offers, which vidni's flake pins;
- when a still is taken: after the page has settled, and after the line's expectations hold;
- three things that varied from one launch to the next until they were pinned (DECISIONS.md, T6): a page above scale 1 is loaded twice, each layer is rastered as one tile, and vidni's style and highlight layer are in a document from its start.

Five captures of a local scene give identical bytes, stills and frames alike (`just test-capture`, which CI runs), and so did five captures of the nibli example's live page. Each capture is filed by what it shows, frozen by hash when its cut is ratified, and never retaken unless its scene changes or `--refresh` asks for it (DECISIONS.md, T3).

**The listening check** (`vidni_media::hear`) is held to no tier, because no output depends on it (DECISIONS.md, T7):
- It writes no file, and no build step reads what it hears.
- Its model is pinned by SHA-256, and whisper-cli by the flake. What whisper hears may still differ from one CPU or build to the next; only one host has been measured.
- Its score is plain arithmetic on the two texts, so the same transcript always gets the same score.
