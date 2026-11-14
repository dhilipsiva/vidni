# Example: the nibli syllogism explainer

The first video made with vidni. It's a five-minute walkthrough of the Syllogism example in the live nibli playground, in a 16:9 cut and a 9:16 cut, narrated in its creator's own voice.

vidni builds it from `vidni.toml` ([From draft to release](../../docs/spec.md#from-draft-to-release)):

```sh
vidni draft examples/nibli-syllogism                              # every cut in the draft voice, in build/draft/
vidni final examples/nibli-syllogism --cut long --cut vertical    # the two full cuts in the creator's voice, in out/
```

The two full cuts are ratified and approved:
- `ratified.lock` freezes what they show and say.
- `release.json` records their releases, each with the hashes of its build. The newest are `long-r5` and `vertical-r4`. Since the first releases, the narration goes through vidni's voice chain ([The narration's sound](../../docs/spec.md#the-narrations-sound)), the long cut is as loud as YouTube plays, and the captions are cut into cues of two lines of 42 characters at most ([`[captions]`](../../docs/spec.md#captions)). So the vertical cut's captions are no longer the reference's; `just parity` still holds vidni to the reference, by cutting captions the reference's way. Their pauses hold vidni's stand-in noise, because the takes came with no recording of the room; `vidni record --test` keeps one for the next build.
- `vidni repro` built both cuts again, on other cores and at other paths, and every hash agreed (docs/determinism.md).

The long cut is delivered as YouTube takes it ([The final build](../../docs/spec.md#the-final-build-vidni-final)):

| File in `out/` | What it is |
|---|---|
| `long.mp4` | The clean master |
| `long.en.srt`, `long.en.vtt` | Its captions, to upload with it |
| `long.captioned.mp4` | The same picture with the captions drawn in, as the reference's cut had them |
| `long.chapters.txt` | Its ten chapters, to paste into the description. The same chapters are in both pictures. |
| `youtube/long/` | The upload for YouTube: the master, its captions, two thumbnails, and `metadata.md` with the title, the description, the tags and the steps ([`[publish]`](../../docs/spec.md#publish)) |

The final cuts need the creator's takes and the captures the cuts froze, and both are kept out of git. Without them, `vidni status` reports the cuts as changed and `vidni final` refuses them. `vidni draft` still works: it captures the playground as it is now, and builds the cuts in the draft voice.

## The short

A third cut, `short`, is a 54-second 9:16 video for YouTube Shorts, Instagram Reels and WhatsApp Status ([Several cuts](../../docs/spec.md#several-cuts)).
- **It lists 12 lines.** Ten are lines the full cuts say too: the syllogism, and the start and end of the backward search.
- **Two are its own:** `short-hook` opens it and `short-end` closes it. Both set `only = ["short"]`, so the full cuts leave them out and stay ratified.
- **It is laid out clear of the apps.** Shorts and Reels cover the top, the bottom and the right of a video with their own buttons and labels, so the short keeps its title, its view and its captions inside the part they leave free ([What the apps cover](../../docs/spec.md#what-the-apps-cover)). Its two own lines are on cards drawn from the theme's templates, `hook` and `more`, which are laid out there too. `build/draft/review/short/` has a frame of each section for each app, with what the app covers shaded.
- **It is a draft.** Its two own lines have no take yet, so `vidni draft --cut short` builds it in the draft voice and `vidni final` refuses it. `vidni plan --cut short` prints what is left to record.

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
