# Example: the nibli syllogism explainer

The first video made with vidni's pipeline. It's a five-minute walkthrough of the Syllogism example in the live nibli playground, in a 16:9 cut and a 9:16 cut, narrated in its creator's own voice.

Until vidni can render it from a `vidni.toml` (P8-VIDNI-02), the reference pipeline in [`../../reference/`](../../reference/) builds it:
- The script is `reference/segments.json`.
- The cuts land in `reference/out/`. The creator's voice is in `nibli-syllogism-voice-{16x9,9x16}.mp4`; the synthetic draft voice is in `nibli-syllogism-{16x9,9x16}.mp4`.

## The creator's takes

`takes/` holds the recorded narration, filed by line ID and spoken words (docs/spec.md, "Takes and spoken words"):
- `takes/<line-id>/<hash>.wav` is the kept take for each of the 57 lines, at 48 kHz, 16-bit mono.
- `takes/<line-id>/raw/` keeps every raw take, before trimming.
- `takes/_room/` holds the room tone from the mic check.

The takes were recorded as numbered files and moved here in P2-VIDNI-03, each checked by sha256. `vidni takes examples/nibli-syllogism` shows them all as current.

The takes are kept out of git (DECISIONS.md, D4), so a clone of vidni doesn't include them. `assets.lock` records the sha256 of each one, so anyone can check a copy restored from a backup:

```sh
just assets-check   # or: cd examples/nibli-syllogism && sha256sum -c assets.lock
```

Without the takes, only the synthetic variant can be rebuilt. `just golden` needs them too, because the creator's voice is one of the golden variants. It lays them out for the reference pipeline with `vidni takes --link`.

After a retake, refresh the lock:

```sh
cd examples/nibli-syllogism && find takes -type f -print0 | sort -z | xargs -0 sha256sum > assets.lock
```
