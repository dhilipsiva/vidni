# Narration highlights

Timed emphasis uses measured geometry and a reviewed interval in the exact audio
take. Text is measured in the same pinned Chromium document that draws it. A
font change or a longer name therefore moves the highlight with the text.

Highlights normally stay visible for **at least three seconds from appearance**.
The spoken phrase supplies the start and the minimum meaningful end; reading
time extends the display without stretching speech or inserting a pause.
Several highlights can remain visible together.

```toml
[emphasis]
min_duration = 3.0
```

The default is three seconds. Set zero to use only the spoken interval. A
longer spoken interval is never shortened to the minimum. A scene change that
removes the target, a camera leaving it, or the end of the cut can end the extra
hold sooner. Status and review show the actual duration and the cutoff reason.

A new narration line does not itself end a hold. Identical captured pictures
can retain their measured target. Across changed pictures, explicitly mark the
same meaningful target with a shared `continuity` ID:

```toml
[scenes.next.targets.parcel-name]
element = "parcel-name"
continuity = "parcel-a-name"
```

Put that identity on the originating target too. Each identity must be unique
within a scene. The next scene supplies its own measured geometry; reused names
such as `focus` do not establish continuity. Stop the identity when the content
changes meaning. A missing identity ends continuation unless the captured
picture is identical. Targets never reappear after their hold has ended.

## Author a target and a cue

Give the visible SVG text an ID:

```xml
<text id="parcel-name" x="100" y="180"
      font-family="IBM Plex Mono" font-size="48">ParcelA</text>
```

Name that element, or an exact substring of it, in the scene:

```toml
[scenes.statement]
kind = "image"
url = "statement.svg"

[scenes.statement.targets.capital]
element = "parcel-name"
span = { text = "P" }

[[lines]]
id = "parcel-name"
section = "names"
show = "ParcelA begins with a capital P."
scene = "statement"
emphasis = [{ id = "capital", target = "capital", phrase = "capital P" }]
```

Without `span`, the target covers the element's measured bounds. `elements =
["first-name", "second-name"]` gives separately outlined elements. A web target
can use `css = "#parcel-name"` and the same `span` declaration. Wrapped text gets
one rectangle per visible line. Text matching is exact and case sensitive;
repeated text needs an explicit one-based `occurrence`. The cue's `phrase`
selects spoken words, while its `target` selects visible content. These are
separate declarations because spoken and displayed spellings can differ.

SVG text uses fonts from the pinned catalog. Unknown families or missing glyphs
fail capture. Missing, ambiguous, hidden or clipped targets fail instead of
choosing an approximate location. A measured target cannot be replaced by a
handwritten `rect`, and timed emphasis cannot mix with baked line highlights.
Legacy projects without `emphasis` keep their existing behavior.

The compositor draws the outline in the theme's accent color, above the scene
crossfade. It uses the same crop and interpolated camera transform as the scene.
Every active frame is checked for clipping and collisions with captions, labels,
overlays and platform insets. Expired emphasis does not survive a crossfade.
Identical outlines are drawn once even when several cues overlap. The original
spoken interval must be fully visible; only its added reading time can be cut
short when the camera leaves. Unexpected caption coverage remains an error.

## Propose, review, then build

```sh
vidni cues propose path/to/project --voice draft --preview
vidni cues preview path/to/project --voice draft
vidni cues review path/to/project --voice draft
vidni cues status path/to/project --voice draft
vidni draft path/to/project
```

Use `--cut long` or `--only parcel-name` to work on a subset; both flags can
repeat. For recorded narration use `--voice takes`, followed by `vidni final`.
Synthetic timing and recorded timing are separate. A verified draft cue never
verifies a recording.

`propose` prepares local audio and measured captures, then uses the pinned local
Whisper model to suggest intervals. Recognition can miss a name or word; those
cues have no proposed interval and need timing set in review. Proposals never
count as verification. `--preview` creates full-resolution clips and lossless
frames just before, during and after each display segment under `build/cues/`.
Clips include the complete reading hold and the following narration it overlaps.
The reviewer edits **spoken start/end**, sees the resulting display duration
separately, and plays the phrase plus its full hold before verification.
The clips carry a diagnostic label and cannot be exported as a production cut.

`cues preview` renders the entire selected cut continuously under
`build/cues/full/`, using the cut's original timeline, processed master audio
and proposed highlight intervals. A permanent review label identifies it as a
diagnostic copy. Its JSON receipt records the audio, layout, cue status and
video hash. This lets a creator watch the lesson in context before cue review;
it writes no cue verification or production build. An unset cue has no highlight
until its timing is proposed or set. Use `--cut` to choose full cuts; `--only`
belongs to individual-line review.

`review` opens the desktop reviewer. Play the line with audio, check the target
and timing, and select **Verify this cue**. Scrub, loop or step one frame to
inspect a boundary. Use **Start here**, **End here**, or the seconds fields to
adjust it, then **Save timing and rebuild preview** and play the new clip before
verifying. An unset interval must first be saved and previewed. Verification is
a person's decision; `VIDNI_ASSISTANT=1` refuses the interactive review and
self-verification. An owner's explicit confirmation of a continuous review
may instead be recorded as described below.

Normal draft and final builds require current verification for every cue in the
selected cut. There is no export bypass. Diagnostic previews remain available
while review is incomplete.

## Confirming a continuous review

An owner who watched the full review videos with audio and confirms that the
targets and timing are correct need not repeat the per-cue buttons. Save their
actual statement in a text file, then record it with the exact reviewed video
hash for each selected cut:

```sh
vidni cues record-confirmation path/to/project --voice takes \
  --statement-file owner-confirmation.txt \
  --review-sha256 long=EXACT_LONG_VIDEO_SHA256 \
  --review-sha256 short=EXACT_SHORT_VIDEO_SHA256
vidni final path/to/project
```

Use `--cut` to select complete cuts. A hash is required for every selected cut;
`--only` is not supported. The command uses the existing continuous-review
receipts under `build/cues/full/`, checks the actual video bytes and all current
audio, content, layout and cue timings, and refuses missing or unset cues.
Every selected cut must pass before the lock is written atomically.

An assistant may relay this decision only after the owner explicitly confirms
the reviewed videos; it cannot substitute its own assessment. The lock retains
the owner's statement, video and receipt hashes, cue keys and whether the owner
or an assistant saved the decision. Existing per-cue review remains available
for corrections. Retakes, layout edits and changed cue intervals still require
review of the changed material. This confirms highlights, not release approval
or permission to upload.

## What a verification belongs to

`cues.lock` is written by the reviewer or the explicit-confirmation command,
not edited by hand. Each record is keyed
by cut, voice, line ID and cue ID. Its half-open interval is stored in audio
sample offsets, with the exact audio hash, rate and sample count. It also binds
the spoken text, cue declaration, measured capture, target geometry, fonts,
theme, camera, reading-time policy, continuation targets and neighbouring review
context. A retake, trim, wording or layout change invalidates
the affected verification. The reviewer checks the current binding again when
verification is saved.

`build/cues/proposals.json` holds advisory timings. The per-cut review JSON holds
clip paths, clip hashes and the bindings depicted in those clips. These build
files can be regenerated. Keep `cues.lock` with the project; it is protected by
the assistant protocol like the other person-written locks.

## Targets inside screenshots

A screenshot embedded in an SVG needs a measurement receipt from its source
document. Embed the exact PNG bytes in an SVG `<image id="screen" ...
preserveAspectRatio="none">` and declare:

```toml
[scenes.statement.targets.result]
element = "screen"
capture = { receipt = "screen.targets.json", target = "verdict" }
```

The receipt is JSON with `schema: 1`, project-relative `image`, PNG `sha256`,
pixel `width` and `height`, a nonempty `browser` provenance string, and `targets`
mapping names to arrays of `[x, y, width, height]` in the PNG's pixels. Produce
those rectangles from the source DOM when capturing the screenshot; do not
guess them later from the picture. Convert CSS coordinates by the capture's
device scale when writing the receipt.

Capture verifies the file and embedded bytes, dimensions and named geometry.
The SVG image's own transform and nested viewport crop are applied to those
rectangles. A target outside the captured or displayed crop fails. Unsupported
clip paths fail explicitly. This preserves measurement provenance through a
composed screenshot instead of baking its outline into the image.
