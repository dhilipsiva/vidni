# Determinism

vidni's outputs are deterministic in three tiers (TODO.md, rule 2):

- **T1:** the resolved project, the timeline, the captions and metadata, the rgb24 frame stream before encoding, and the PCM before encoding are byte-identical on any x86-64 host with the same `flake.lock`.
- **T2:** an MP4's packets (`ffmpeg -map 0 -c copy -f framemd5`) and its file bytes are identical across runs on one host.
- **T3:** across hosts, MP4s are compared by SSIM and are not promised to match. The delivered files are archived.

Some inputs can't be made deterministic: a live web page, every Chromium render, sampled TTS, whisper, the creator's takes and the clock. They are frozen by hash when a cut is ratified, and whisper never feeds an output.

## Verified

| Date | Host | What | Tier | Evidence |
|---|---|---|---|---|
| 2026-11-13 | the maintainer's desktop: AMD Ryzen 9 9950X3D, NixOS, Linux 7.3 | the reference pipeline, both variants, both cuts | T2 | `just golden` re-derived 178 inputs and 14 outputs identically on independent runs, MP4 file bytes included |
| 2026-11-13 | same | the timeline and captions (Rust) | T1, equal to golden | `just parity timeline`: both variants' timeline JSON and SRT equal the reference's |
| 2026-11-13 | same | the narration track (Rust) | T1, equal to golden | `just parity narration`: both variants' PCM equal the reference's |
| 2026-11-13 | same | the encoder (Rust), ffmpeg 9.0.2 with libx264 | T2 | `just encode-t2` encoded the 9,188 golden frames of the 16:9 cut with the golden narration twice: the same framemd5 and the same 25,089,781 bytes |

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

**Captures** (`vidni_core::capture`) aren't deterministic, because they come from a live page. Each is filed by what it shows, frozen by hash when its cut is ratified, and never retaken unless its scene changes or `--refresh` asks for it (DECISIONS.md, T3).
