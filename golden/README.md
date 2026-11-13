# Golden outputs of the reference pipeline

These files record what the reference pipeline produces for the nibli syllogism example (TODO.md, P1-VIDNI-01). The reference is the Python and Node pipeline that made the first vidni video. Every Rust stage is checked against these files before it replaces its counterpart (P1-VIDNI-04): `just parity <stage>` runs the stage's Rust port and compares its output with them. The timeline and captions are ported so far (P2-VIDNI-04).

| File | What it holds |
|---|---|
| `hashes.json` | sha256 of every frozen input and every re-derived output |
| `commands.json` | the command that produces each output |
| `timeline-{synthetic,voice}.json` | the normalized timelines: clip paths relative (the creator's takes as `recordings/NNN.wav`, wherever they are stored), whisper's `heard` and `score` dropped |
| `nibli-syllogism{,-voice}.en.srt` | the captions |
| `framemd5/*.framemd5.gz` | per-packet MD5s of the four cuts (`ffmpeg -map 0 -c copy -f framemd5`), gzipped with no timestamp |

**Frozen inputs** are hashed, never regenerated, because they cannot be re-made deterministically. Each is hashed twice: the file bytes, and the decoded PCM or RGBA pixels. They are:
- the script (`segments.json`)
- the synthetic Piper takes (`build/audio`)
- the creator's takes (`examples/nibli-syllogism/takes`, laid out as numbered files by `vidni takes --link` and listed as `recordings/`)
- the captures and cards Chromium rendered (`build/stills`, `build/cards`)
- `capture.json`, and `onscreen.json` without its clock field

**Outputs** are re-derived from those inputs, for the synthetic variant and the creator's voice:
- `tts.py` makes the timeline, the narration and the captions
- `compose.py` renders both cuts

```sh
just golden          # re-derive everything and compare with these files (about 5 minutes)
just golden-update   # re-derive everything and overwrite these files
```

Both recipes run inside the reference's own nix shell. The reference lives in `reference/`; `VIDNI_REFERENCE` points them at another copy.

The golden media are not committed: the MP4s, stills and cards are listed here only by hash. They live in the reference's `out/` and `build/` folders, and the parity harness checks their hashes before comparing against them.

The harness compares MP4 packets by a fingerprint of these framemd5 logs, not by the logs' hashes (DECISIONS.md, T1). The logs were written by the reference's ffmpeg 8.1, and vidni's ffmpeg 9.0 writes the same packets with a different version line and extra side data.
