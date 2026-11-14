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
| 2026-11-14 | same | the encoder with x264's lookahead in the encoder's own thread (`sync-lookahead=0`): 451 frames of the 9:16 cut, around a crossfade | T2, on 1 to 32 cores | 40 encodes on 1, 2, 3, 4 and 32 cores gave one packet stream. With x264's default, a lookahead thread, 40 encodes gave 8 different ones (DECISIONS.md, T8). `just encode-t2`, run again with the fix, gave the same 25,089,781 bytes twice |
| 2026-11-14 | same | the nibli example's final cuts, built by `vidni final` and twice more by each of two runs of `vidni repro` | T1 and T2 | five builds of each cut agree in every hash: one in the maintainer's own environment on 32 cores, and four in clean environments at other paths, two on 2 cores and two on 3. They agree in the resolved project, the timeline, the captions, the narration before and after normalization, the 8,441 frames, the packets and the file's bytes. `examples/nibli-syllogism/release.json` holds the hashes |
| 2026-11-14 | same | the same 9:16 cut, with no network (`vidni repro --offline`) | T1 and T2 | two more builds, each in a namespace with no network, agree with the five above |
| 2026-11-14 | same | the example's 54 captures, taken from the live page twice, two hours apart, with the fonts the flake pins | identical where the cuts show them | the final cuts built from each set have the same frame streams: the two hashes listed below, both times |
| 2026-11-14 | same | the example's drafts (`vidni repro --draft`), with the draft voice speaking all 57 lines again in each build | T1 and T2 on one host, on 2, 3 and 32 cores | three builds of each cut agree in every hash, the narration's among them: Piper, seeded, gave the same PCM on 2 cores, on 3 and on 32 |
| 2026-11-14 | same | the nibli example's eight card renders, taken again by `vidni draft` over an hour after the first, in a fresh Chromium each | identical on one host | each PNG has the SHA-256 that `ratified.lock` froze and `assets.lock` lists, so both cuts stayed ratified and their drafts were kept |
| 2026-11-14 | same | the recorder's trim (Rust) on the nibli example's raw takes | equal to the reference's kept takes | `just parity takes`: 57 of 57. Each kept take is, sample for sample, the trim of one of its line's raw takes, with the two samples ffmpeg's `silenceremove` misplaces at each splice put where it put them |
| 2026-11-14 | same | the product against the reference: the example's final build, planned as `vidni final` plans it from fresh captures and the creator's takes | equal to golden, and to the reference's frames before encoding | `just parity product`: 46 of 46 checks. Each cut's timeline, captions and narration equal golden's, and its 20 sampled frames score SSIM 1.0000 against the reference's, with no channel more than 49 of 255 apart. The captures were taken with the maintainer's fonts and the frames drawn with the reference's faces, as golden's were |

The nibli example's final cuts, as approved (`long-r1` and `vertical-r1`), drawn with the fonts vidni pins and from captures taken with the flake's fonts:

```
long.mp4      8,441 frames  frames c27e3dbef168063755bc231a395444d85531d0695c94f6cb32c28488394b91c8
                            file   d9987cbad416c53729d8425c6e683e0e4c2396a8e3a639020fd5c9d03c18b950
vertical.mp4  8,441 frames  frames 7f1d9f0f086c0ab9025c04f533b283d749f20f3f5341815a9115eeddd8ffe125
                            file   c152b5ebcdba3de951566676b123768a51b3fd701829d6421c66c1f5877b0d0c
```

Since then the long cut has become a clean master with a captioned variant, and has chapters (`long-r3`, 2026-11-14). The captioned variant has the frames above, and `vidni repro examples/nibli-syllogism --cut long` built both pictures twice, on cores 0,1 and on cores 29,30,31, with every T1 and T2 hash equal and equal to the approved build's:

```
long.captioned.mp4  8,441 frames  frames c27e3dbef168063755bc231a395444d85531d0695c94f6cb32c28488394b91c8
                                  file   d3c52d04d4030febeba81ca47890916a283f3a848279448fbd58e14c295cbd32
long.mp4            8,441 frames  frames b2e1cdb1530e8a0b66178c07f56b8b487d1c4bc7a9e85813ce08892e8d7cf100
                                  file   3445455bba04104ddb398b84a83041cee1091ee311867958d50f7c9b92f66a64
```

The files differ from `long-r1`'s because chapters are written into them; the vertical cut's file is the one above still.

The voice chain (P4-VIDNI-03, 2026-11-14) changed every cut's sound and none of its frames: `long-r4` and `vertical-r3` have the frame hashes above and new files. (`long-r5` and `vertical-r4`, the same day, have captions cut into shorter cues, so the frames of the pictures with captions drawn in differ from the ones above: `long.captioned.mp4` is `ebfc5c3a3d2618d0…` and `vertical.mp4` is `3ef14319f57bac35…`. The clean master, `long.mp4`, still has the frames above.) The narration is now made by vidni's own arithmetic, with no ffmpeg filter before the resampling, so its samples are held to T1. `vidni repro examples/hello --draft` built both cuts twice with every hash equal, and a unit test pins the hash of a clip through the chain, which CI checks on another machine.

The compositor's frame streams (rgb24, every frame in order) on the reference's captures, drawn with IBM Plex Sans Bold (sha256 `19de5aec…c316`), IBM Plex Mono Bold (`44f6da8e…a6c7`) and Space Grotesk Bold (`be8709ab…368f`):

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
- libx264's lookahead runs in the encoder's own thread (`sync-lookahead=0`). In a thread of its own, which is x264's default with several threads, the packets depended on how the threads were scheduled (DECISIONS.md, T8).
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

Five captures of a local scene give identical bytes, stills and frames alike (`just test-capture`, which CI runs), and so did five captures of the nibli example's live page. Each capture is filed by what it shows and frozen by hash when its cut is ratified. It is taken again only when its scene changes, when `--refresh` asks for it, or, for a page in the project folder, when a file the page loaded changes (DECISIONS.md, T3 and T9).

**Builds** (`vidni_media::build`):
- A build's record (`<cut>.build.json`) names every input and output by its SHA-256. `vidni repro` builds twice more, from copies of the project at other paths, in clean environments and on other cores, and compares the records with each other and with the creator's own build (DECISIONS.md, T8).
- Nothing in a build reads the clock, the time zone, the locale or the home folder, and the timeline names each clip by its path inside the project.
- A build takes no captures. They are inputs, frozen with each still's frames when a cut is ratified.
- A build is kept, not made again, when its record has the `inputs` the cut has now and every file it names still has the hash it records (DECISIONS.md, T9). Keeping a build rests on the same claim that `vidni repro` checks: the same inputs give the same files. A record names vidni by its version, so between versions a change to what vidni draws must raise the compositor's count, as it must for a release to notice it.
- The day a release is approved is read from the clock. It goes into `release.json` and into no output.

**Takes** (`vidni_record::clean`): a kept take is trimmed from its raw take once, when it is recorded, and is an input from then on, frozen by its hash. The trim itself repeats: it is integer and single-precision arithmetic in a fixed order, and the filter's five coefficients are rounded to single precision from sines and cosines worked out in double (DECISIONS.md, T10).

**The listening check** (`vidni_media::hear`) is held to no tier, because no output depends on it (DECISIONS.md, T7):
- It writes no file, and no build step reads what it hears.
- Its model is pinned by SHA-256, and whisper-cli by the flake. What whisper hears may still differ from one CPU or build to the next; only one host has been measured.
- Its score is plain arithmetic on the two texts, so the same transcript always gets the same score.
