# Example: the nibli syllogism explainer

The first video made with vidni. It's a five-minute walkthrough of the Syllogism example in the live nibli playground, in a 16:9 cut and a 9:16 cut, narrated in its creator's own voice.

vidni builds it from `vidni.toml` ([From draft to release](../../docs/spec.md#from-draft-to-release)):

```sh
vidni draft examples/nibli-syllogism   # both cuts in the draft voice, in build/draft/
vidni final examples/nibli-syllogism   # both cuts in the creator's voice, in out/
```

Both cuts are ratified and approved:
- `ratified.lock` freezes what they show and say.
- `release.json` records the releases `long-r1` and `vertical-r1`, each with the hashes of its build.
- `vidni repro` built both cuts again, on other cores and at other paths, and every hash agreed (docs/determinism.md).

The final cuts need the creator's takes and the captures the cuts froze, and both are kept out of git. Without them, `vidni status` reports the cuts as changed and `vidni final` refuses them. `vidni draft` still works: it captures the playground as it is now, and builds both cuts in the draft voice.

The reference pipeline in [`../../reference/`](../../reference/) made the first version of this video, from `reference/segments.json`. Its cuts are in `reference/out/`, and `just parity product` checks vidni's final build against them.

## The creator's takes

`takes/` holds the recorded narration, filed by line ID and spoken words (docs/spec.md, "Takes and spoken words"):
- `takes/<line-id>/<hash>.wav` is the kept take for each of the 57 lines, at 48 kHz, 16-bit mono.
- `takes/<line-id>/raw/` keeps every raw take, before trimming.
- `takes/_room/` holds the room tone from the mic check.

The takes were recorded as numbered files and moved here in P2-VIDNI-03, each checked by sha256. `vidni takes examples/nibli-syllogism` shows them all as current.

The takes are kept out of git (DECISIONS.md, D4), so a clone of vidni doesn't include them. `assets.lock` records the sha256 of each one, and of each frozen capture, so anyone can check a copy restored from a backup:

```sh
just assets-check   # or: cd examples/nibli-syllogism && sha256sum -c assets.lock
```

Without the takes, only the synthetic variant can be rebuilt. `just golden` needs them too, because the creator's voice is one of the golden variants. It lays them out for the reference pipeline with `vidni takes --link`.

After a retake or a new capture, back them up, which also writes the list afresh:

```sh
vidni backup examples/nibli-syllogism --to /mnt/other-disk/vidni --to /mnt/nas/vidni
vidni verify examples/nibli-syllogism   # the same check as assets-check, by vidni
```

## Captures

`vidni capture examples/nibli-syllogism` takes what the two cuts show into `build/captures/` (out of git): 46 stills of the live playground, one for each state a line shows, each with the boxes it is framed by, and the 8 renders of the cards and overlays, each with a record of the file it was made from (`cards/card.html`). `vidni draft` does the same on its way.

The captures there now were taken on 2026-11-14, with the fonts vidni's flake pins, and both cuts froze them when they were ratified. So they are kept with the takes, and listed in `assets.lock`. Captured with the maintainer's own fonts instead, they match the reference's captures (`just parity stills`), and five captures in a row are pixel-identical (`just capture-repeat`).

## The draft voice

`vidni speak examples/nibli-syllogism` speaks every line in the draft voice, piper:en_US-joe-medium, after `vidni voices fetch piper:en_US-joe-medium` has fetched it; `vidni draft` begins with the same step. `voice.lock` records the seed each line's take was kept at, so a redraft gives the same audio: clearing `build/` and drafting again gives the same PCM (docs/determinism.md). These are vidni's own takes, not the reference's: the reference ran Piper unseeded, so its synthetic clips can't be repeated.

## The look

`[style] theme = "../themes/quine"` names the example's theme: near-black, an ember accent, Space Grotesk for titles and `//` before each label, as the reference pipeline drew it. With it the final cuts have the frames and files they had before themes existed, and they were approved again as `long-r2` and `vertical-r2` because the theme is now one of a build's inputs.
