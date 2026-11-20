# V01 — A Confident Claim Still Needs Evidence

Main lesson and Short · continuous explanations with targeted QUINE highlights.

This lesson introduces symbolic and neuro-symbolic reasoning through an everyday
parcel situation. No formal logic background is assumed. Length follows the
explanation; there is no nine-minute minimum. Both cuts are streaming videos:
no audience questions, prediction tasks or deliberate response gaps.

Read the [complete narration](#complete-narration) as connected speech. The
[scene plan](#scene-plan) describes how highlights follow that explanation.
Paragraph anchors are review points, not new recording-line IDs.

**Production status:** Both cuts are now rendered with the creator's recorded
narration. All 59 takes are current and both cuts remain ratified.

- [Main lesson with captions](out/long.captioned.mp4): **5:24.189**, 1920×1080.
- [Main clean master](out/long.mp4), with separate [SRT](out/long.en.srt),
  [VTT](out/long.en.vtt) and [chapters](out/long.chapters.txt).
- [Short with captions](out/short.mp4): **0:48.137**, 1080×1920,
  with [SRT](out/short.en.srt) and [VTT](out/short.en.vtt).

QUINE, targeted highlights and continuous narration are preserved. There are no
audience questions, thinking tasks or deliberate response holds. The recorded
takes determine the new scene, caption and chapter timing. The earlier synthetic
review cuts remain under `build/review-v01-highlighted/`; previous QUINE versions
remain archived. The owner supplied the [main lesson’s YouTube URL](https://www.youtube.com/watch?v=v9OfHXMdB7E&list=PLAQKTXMK9CyY)
on 2026-11-18. Human viewer feedback remains separate from media verification.

The [publishing and sharing copy](#v01-publishing-copy) is ready for both cuts.
The sharing copy now uses the [YouTube link](https://www.youtube.com/watch?v=v9OfHXMdB7E&list=PLAQKTXMK9CyY)
supplied by the owner on 2026-11-18.

## Complete narration

<!-- V01_SCRIPT_START -->
<a id="v01-welcome"></a>

Welcome to Neuro-Symbolic Reasoning from First Principles. In this course, you'll learn how to represent information, write rules, and evaluate which claims follow from them. This is the foundation of symbolic reasoning.

We'll also explore how that reasoning can work alongside neural AI, including language models that generate text. That combination is called neuro-symbolic reasoning.

You don't need a background in formal logic. We'll build these ideas through everyday situations before introducing formal notation. Today, we'll begin with the symbolic side: two parcels and a claim that one is ready for collection.

<a id="v01-shop"></a>

Imagine you're helping a small shop prepare today's orders for collection.
There are two parcels on the packing table. We'll call them A and B so we can
keep track of them.

Before a parcel moves to the collection area, the shop checks two things: it
has been packed, and its address label is attached. That's what “ready” means
for this example. Both checks need to be complete for the parcel being moved.

A colleague points to Parcel A and says, “This one's ready. The packing is
done, and there's a label confirmation.”

You haven't inspected the parcel yourself. You're working from the packing
team's notes, and your colleague hands you two of them. For this example,
let's take those notes as accurate records of what the team checked.

<a id="v01-records"></a>

The first says, “Parcel A: packed.” The second says, “Parcel B: label attached.”

There is a packing confirmation and a label confirmation, just as your
colleague said.

The packing note concerns A. The label note concerns B. Each tells us something
useful, but they describe different parcels. The shop needs both checks for
the same parcel.

You can see how the mix-up happened. At a glance, there seems to be one note
for each check. Reading the parcel letters changes the picture. We know that
A is packed and that B has a label. The label confirmation for B doesn't tell
us whether A has its own label.

So the colleague's explanation has a gap. The claim about A goes further than
these two notes support.

<a id="v01-missing-support"></a>

There's a distinction here that will matter throughout the course. We haven't
shown that A has no label. It might already have one, with its confirmation
still waiting to be written. Or the label might not have been attached yet.
Either situation would fit the notes in front of us.

For now, the careful conclusion is simply that these records don't establish
A's readiness. That leaves room to find out more. It doesn't turn a missing
confirmation into proof that the parcel is unlabeled.

A's label status needs its own evidence. Reading A's packing note again won't settle the label check. Neither will another look at B's label confirmation. We need evidence about A's own label.

<a id="v01-repair"></a>

Now imagine someone checks Parcel A and confirms that its address label is
attached. They write a new note: “Parcel A: label attached.” Put that beside
the original packing confirmation.

Both notes now concern A. One confirms the packing; the other confirms the
label. Together, they meet the shop's two-check requirement.

 That's why we can now say that A meets this checklist. The new
observation supplied the part that was missing.

<a id="v01-terms"></a>

You've just followed a small piece of reasoning. We can give its parts names
now that we've used them.

The statements we start from are called premises. Here, we're starting from
the statements that A is packed and that its label is attached. The shop's
requirement supplies a rule: a parcel that meets both conditions meets this
checklist. Applying that rule to these premises supports our claim about A.

As our examples grow, these names will help us separate the information we start with from the conclusions we reach. Here, two notes about the same parcel are connected by the shop's requirement.

<a id="v01-variation"></a>

The same idea appears in a classroom display. Each drawing needs its title and the artist's name beside it.

You're helping organize the display from notes sent by the classroom helpers.
One note confirms that Drawing 1 has its title. Another confirms that Drawing
2 has its artist's name. Someone says, “Drawing 1 is ready to go on the wall.”

We still need confirmation that Drawing 1 has its own artist's name. The note
about Drawing 2 can't complete that check. Once we have the artist-name
confirmation for Drawing 1, its two requirements are supported. Until then,
the notes leave its readiness unsettled, just as they did for the parcel.

<a id="v01-bridge"></a>

This is the habit we'll build on: follow each piece of evidence back to what
it actually describes, then see whether it supports the claim we're making.
An explanation can sound convincing while quietly joining information that
belongs to different things.

That is our starting point for symbolic reasoning: explicit statements, rules, and the claims they support. We'll build this foundation before exploring how neural models can work alongside it.

In the next lesson, we'll meet Nibli, the reasoning engine we'll use in this
course, and Nibli KR, the language for writing its premises and rules. We'll
carry these same parcels with us, so the new part will be how we express
reasoning we've already worked through together.

We'll still need to review the records and whether our written rule means
what we intend. Nibli will help us evaluate what follows from the statements
and rules we give it.
<!-- V01_SCRIPT_END -->

## Independent Short

<a id="v01-short-case"></a>
This Short establishes the shop's two-check requirement, the claim, the accurate
records, their different subjects, the uncertainty and the new observation.
It uses the same fictional case and support distinction as the main lesson.
Its purpose is one compact explanation, not an interactive exercise. The title
band reads “Neuro-Symbolic Lesson 1”; the final line connects the example to
symbolic reasoning. No knowledge of the long lesson is required.

<!-- V01_SHORT_SCRIPT_START -->
You're helping a shop prepare parcels for collection.

Here, each parcel must be packed and have its own address label attached.

A colleague claims Parcel A is ready.

In this example, the team's notes are accurate.

One confirms that Parcel A is packed.

Another confirms that Parcel B has its label attached.

Those notes describe different parcels. B's label confirmation doesn't establish that A has its label.

That doesn't mean A is unlabeled. We need evidence about A.

Now someone checks A and confirms its label is attached.

Together with A's packing note, that supports the claim that A meets this checklist.

That is a first step in symbolic reasoning: connecting the evidence to the claim.
<!-- V01_SHORT_SCRIPT_END -->

## Scene plan

The narration leads; targeted ember outlines and a faint fill guide attention
to the exact text being explained. Context remains visible, and each line's
emphasis changes with the explanation. There are no response holds. Show no KR
syntax, terminal output or verdict badges. Names and words carry meaning
independently of color.

| Narration passage | Visual and attention | Pacing |
|---|---|---|
| [Welcome](#v01-welcome) | QUINE course title and “Neuro-Symbolic Lesson 1”; information/rules, neural language models and symbolic statements/rules. | Name the course destination before entering the parcel case. |
| [Enter the shop](#v01-shop) | Packing table, two parcels and collection area; then the two checks, the same-parcel condition, the colleague's claim and the assumption that the notes are accurate. | Give objects a visible referent before using their labels; address-label faces remain outside the view. |
| [Read the notes](#v01-records) | Reveal each note; highlight Packed and Label attached when introduced, then specifically A and B during the mismatch explanation. | Explain the comparison directly after the records, without a prediction task. |
| [Missing support](#v01-missing-support) | Focus “not established by these notes”, the two possible situations in turn and the needed evidence about A's label. | Explain why missing confirmation does not establish an absent label. |
| [New evidence](#v01-repair) | Focus the new A label note, then both A letters and the completed support. | Narrate why the new observation changes the support immediately. |
| [Name the parts](#v01-terms) | Retain the same notes and requirement; focus the premises, then the rule. | Introduce names only after the reasoning is familiar. |
| [Drawing variation](#v01-variation) | Establish the display and requirements; show both notes, focus their different drawing numbers, then the additional Drawing 1 confirmation. | A narrated worked variation, with no audience task or response gap. |
| [Next lesson](#v01-bridge) | Return to statements/rule/claim; distinguish Nibli the engine from Nibli KR the language. | Close fluidly, retaining the need to review inputs. |
| [Short](#v01-short-case) | Dedicated compact scenes within the vertical safe area; highlight checks, claim, note values, parcel letters and new A label confirmation in narration order. | An independent continuous explanation in approximately 45–60 seconds. |

## Timing and review

The main script contains 895 words; the independent Short contains
116 words (whitespace count). The draft voice reads the first note's
colon as “is”. Natural timing uses a 0.6-second lead-in, 0.44-second line gaps,
0.85-second section transitions and a 1.2-second tail. No added end hold or
post-production silence is used. Retired response invitations and one repeated
answer are removed; rewritten lines retain their IDs, and new Short lines have
their own IDs.

A later human comprehension review can check whether the explanation made
the subject mismatch, missing evidence and repair clear. That review is outside
the streaming video. Technical checks and creator approval of the previous
visual direction do not establish a new viewer's understanding.

## Sources for the lesson

- The shop, colleague, parcels, notes and class display are original fictional
  teaching situations. The two-check requirements are stipulated for these
  examples. The initial notes are explicitly assumed accurate; the missing
  confirmation is not a negative observation.
- The [public Nibli README](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/README.md)
  and [language contract](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/NIBLI_KR.md)
  support the closing distinction between the engine and its KR language.
  The [engine's stated limits](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/GUARANTEES.md#soundness)
  support keeping the meaning and accuracy of inputs subject to review.
- This lesson performs no engine evaluation. Its ordinary-language conclusion
  about missing evidence is not a newly captured FALSE verdict. The terminal
  transcript below belongs exclusively to the older notes-app prototype.

<a id="v01-publishing-copy"></a>
## Publishing and sharing copy — updated 2026-11-18

Prepared for the creator-narrated main lesson (5:24.189) and Short (0:48.137).
The YouTube chapters below come from the final recorded export. The owner
supplied the [full lesson URL](https://www.youtube.com/watch?v=v9OfHXMdB7E&list=PLAQKTXMK9CyY) on 2026-11-18.
Every link below preserves that URL, including its playlist parameter.
This is the copy/paste package for this release, including a separate Short
description and the social posts. It does not change the video or publish it.
The workspace and course instructions now require this package automatically
with every final video using the creator's recorded voice.

### Main YouTube title

```text
A Confident Claim Still Needs Evidence | Neuro-Symbolic Lesson 1
```

### Main YouTube description

```text
Symbolic reasoning starts with making clear claims and connecting them to the right evidence.

In Lesson 1 of Neuro-Symbolic Reasoning from First Principles, we use a fictional shop preparing two parcels. One note confirms that Parcel A is packed. Another confirms that Parcel B has its label attached. Both notes are accurate, but they do not yet establish that Parcel A is ready.

The example introduces claims, premises and rules, explains why missing support is not proof of the opposite, and shows how a new observation changes what follows. A classroom display provides a second everyday example.

This course builds toward combining symbolic reasoning with neural AI, including language models. No formal logic or programming background is required.

Chapters
0:00 Learning symbolic and neuro-symbolic reasoning
0:35 Preparing the parcels
1:16 Read the notes
2:01 What is still unsettled
2:43 A new observation
3:11 Names for the reasoning
3:47 A class display
4:29 The next lesson

#SymbolicReasoning #NeuroSymbolicAI #Logic
```

### YouTube Short title

```text
Two Accurate Notes, One Unsupported Claim | Neuro-Symbolic Lesson 1
```

### YouTube Short description

```text
Two accurate notes can still leave a claim unsupported when they concern different parcels.

This 48-second example introduces a foundation of symbolic reasoning: connecting the evidence to the claim.

From Neuro-Symbolic Lesson 1.
Full lesson: https://www.youtube.com/watch?v=v9OfHXMdB7E&list=PLAQKTXMK9CyY

#Shorts #SymbolicReasoning #NeuroSymbolicAI
```

### LinkedIn link post

```text
I'm building Neuro-Symbolic Reasoning from First Principles for people with or without a background in formal logic.

The first lesson starts with two parcels and two accurate notes. The mix-up is simple: the evidence describes different parcels, so it doesn't yet support the claim.

In just over five minutes, I explain how to connect evidence to a claim, why missing support isn't proof of the opposite, and how a new observation changes the reasoning.

This is our starting point for symbolic reasoning, before exploring how it can work with neural AI.

Watch Lesson 1: https://www.youtube.com/watch?v=v9OfHXMdB7E&list=PLAQKTXMK9CyY

#SymbolicReasoning #NeuroSymbolicAI
```

### Facebook link post

```text
The first lesson of my Neuro-Symbolic Reasoning from First Principles course is ready.

We start with two parcels, two notes and one confident claim. I walk through what the notes support, what's still missing and how a new observation changes the conclusion.

No background in logic or programming is needed. The lesson runs just over five minutes.

Watch: https://www.youtube.com/watch?v=v9OfHXMdB7E&list=PLAQKTXMK9CyY
```

### X.com link post

```text
Two accurate notes can still fail to support a claim.

Neuro-Symbolic Lesson 1 starts with parcels, evidence and symbolic reasoning. No formal logic background needed.

Watch: https://www.youtube.com/watch?v=v9OfHXMdB7E&list=PLAQKTXMK9CyY
```

### Threads link post

```text
I'm starting a course on symbolic and neuro-symbolic reasoning, using everyday situations before formal notation.

Lesson 1 begins with two parcels and shows why evidence has to concern the thing in our claim. Missing support leaves room for more evidence.

The full lesson is just over five minutes: https://www.youtube.com/watch?v=v9OfHXMdB7E&list=PLAQKTXMK9CyY
```

### Instagram link post

```text
Two parcels. Two accurate notes. One claim the notes don't yet support.

Neuro-Symbolic Lesson 1 starts with an everyday example of claims and evidence. No logic or programming background needed.

Full lesson on YouTube: https://www.youtube.com/watch?v=v9OfHXMdB7E&list=PLAQKTXMK9CyY

#SymbolicReasoning #NeuroSymbolicAI #LearnAI
```

### WhatsApp Status caption

```text
Two parcels. One claim. A quick lesson in evidence and symbolic reasoning.

Neuro-Symbolic Lesson 1: https://www.youtube.com/watch?v=v9OfHXMdB7E&list=PLAQKTXMK9CyY
```

### Instagram Reels caption

```text
Parcel A is packed. Parcel B has a label. That still doesn't establish that A is ready.

A 48-second example of connecting evidence to a claim—the starting point for symbolic reasoning.

Full lesson: https://www.youtube.com/watch?v=v9OfHXMdB7E&list=PLAQKTXMK9CyY

#SymbolicReasoning #NeuroSymbolicAI #LearnAI
```

<a id="recorded-video-review"></a>
## Recorded-narration final builds — 2026-11-17

The owner completed recording after ratifying both cuts. `vidni takes` reports
59 current takes, zero stale and zero missing. The automatic speech comparison
matches all 59 at 0.95–1.00, with no below-threshold flags or missing audio.
The recorded first-note take matches at 1.00; the earlier “Parcel A” flag belonged
to the retired synthetic clip and does not apply to this recording.

The following command builds the existing ratified content from the creator's
takes:

```bash
VIDNI_ASSISTANT=1 nix develop --command target/release/vidni final examples/nibli-v01-claims
```

It produces a clean long master, its captioned variant and the captioned
vertical Short. No recording, selection, trim, scene or script was changed in
this production pass. The publish description now identifies recorded narration;
both ratified content hashes remain current. The draft Piper configuration stays
available solely for synthetic previews.

| Cut | Duration | Captions | Audio |
|---|---|---|---|
| Main | 5:24.189 | 97 cues; eight chapters | -14.15 LUFS; -1.77 dBTP |
| Short | 0:48.137 | 14 cues | -14.24 LUFS; -1.51 dBTP |

All outputs are 30 fps H.264 with 48 kHz stereo AAC. Native final-build QA and
independent checks pass for codec/dimensions, delivered hashes, stream durations,
caption ordering, chapters and loudness. Every timeline audio path points to a
recorded take. The 315 snapshotted files, including raw/selected
recordings and ratification/voice locks, remain unchanged. Natural gaps remain
0.44 seconds between clips and 0.85 seconds at long section transitions, with
a 0.6-second lead-in and 1.2-second tail. No response hold is inserted.
Final-frame review checks the course title, key evidence highlights, captions
and vertical safe layout at their new recorded timings.

The first build was withheld by native QA because the two captions of
`v01-parcel-shop-03` needed slightly more reading time (20.6 characters/second).
Increasing natural clip spacing from 0.28 to 0.44 seconds lets those captions
remain visible long enough without changing the audio or caption words. The
rerender passes the same 20-character/second check. The initial failed build's
receipt is retained in `build/recorded-review/initial-caption-qa.json`.

The successful native final build took 183.388 seconds, including the
two long variants and the Short. Speech comparison, export review and document
updates are separate from this measurement; the initial attempt took
175.303 seconds. Release approval and upload have
not been performed.

[Recorded speech check](build/recorded-speech-check.txt) ·
[Media checks and output hashes](build/recorded-review/media-checks.json) ·
[Render timing](build/recorded-render-timing.json).

<a id="highlighted-video-review"></a>
## Earlier synthetic long and Short review record — 2026-11-17

The owner requested targeted highlights and both formats, then clarified that
streaming lessons should explain continuously, with no questions to the audience,
thinking tasks or response gaps. The 6 / 4 / 8 second holds and their invitations
are retired. One repeated answer is removed, three transitions are rephrased,
and the rest of the approved main explanation is retained. Existing rewritten
lines keep their IDs; four dropped IDs are retired. Eleven new Short IDs provide
an independent 116-word parcel explanation. The main narration is 895 words.

There are 48 main narration clips and 11 Short clips, using 58 referenced QUINE
scene compositions. Thirty-nine main clips have targeted focus variants; the
Short has dedicated compact layouts. Highlighted values and individual A/B or
1/2 labels move with the narration. Surrounding records remain readable. The
course title and the distinction between symbolic and neuro-symbolic reasoning
remain in the main welcome; both cuts say “Neuro-Symbolic Lesson 1”. The Short's
kicker is shortened to “Reasoning from First Principles” to fit its title band.

| Export | Duration | Picture | Caption cues | Audio |
|---|---|---|---|---|
| Main | 333.371 s | 1920×1080, H.264, 30 fps | 97, eight chapters | -14.12 LUFS, -1.97 dBTP |
| Short | 49.705 s | 1080×1920, H.264, 30 fps | 14 | -14.18 LUFS, -2.10 dBTP |

Both audio streams are 48 kHz stereo AAC. Native media QA and independent checks
pass for dimensions, codec, stream lengths, ordered captions and export hashes.
The timeline contains only the planned natural gaps: 0.28 seconds between clips
and 0.85 seconds at long-lesson section boundaries, plus a 0.6-second lead-in and
1.2-second tail. No post-processing inserts a hold or retimes a cue. The connected
scripts match their respective cut manifests and contain no audience prompts.
Final-frame review covers the long explanation and every Short scene, including
safe caption placement, exact letter highlights and the corrected title band.

All 14 new/rephrased clips pass the local automatic speech check at 0.91–1.00;
45 retained clips use the cached voice. The older first-note “Parcel A” automatic
transcription flag remains a creator listen-through item. This is synthetic
review narration, not the creator's recording or a comprehension result.

Vidni validation and whitespace checks pass. The local originality screen of
current narration, publish copy and displayed scene text finds no shared
12-word or six-word windows. Historical source notes remain preserved verbatim.

The initial two-cut draft took 127.539 seconds locally;
the title-band-only Short rerender took 11.705 seconds.
Writing, visual review and document checks are additional work.

[Render receipt](build/review-v01-highlighted/render-receipt.json) ·
[Media checks](build/review-v01-highlighted/media-checks.json) ·
[Render timing](build/highlight-render-timing.json) ·
[Speech check](build/highlight-speech-check.txt).

The earlier records below describe their dated versions. Their former pauses
and deferred-Short statuses do not apply to the current exports.

<a id="quine-video-review"></a>
## Earlier QUINE and course-introduction review record

This historical record describes the version before targeted highlights and
the continuous-streaming revision. Its thinking gaps are retired.

This revision names the course and its purpose in the welcome. Symbolic
reasoning is introduced through explicit information, rules and supported
claims; the neural/symbolic combination is named as the course's broader
destination. The welcome says this lesson starts with the symbolic side.
The closing returns to that connection before introducing Nibli KR.
The parcel and drawing explanations and their thinking holds are unchanged.

All 28 scene illustrations use QUINE's resolved color tokens. Display text
uses Space Grotesk, labels use IBM Plex Mono, and prose uses IBM Plex Sans.
Void surfaces, ember accents, tight cards and restrained blueprint grids
replace the earlier custom palette. The title reads “Neuro-Symbolic Lesson 1”.
The existing QUINE Vidni theme supplies the caption styling.

The measured export is 384.493 seconds, 1920×1080 at 30 fps, with
H.264 video and 48 kHz stereo AAC. Its 106 caption cues and eight
chapters are retimed around the three thinking gaps: 6.013 / 4.013 / 8.013
seconds. Independent media checks passed for stream lengths, captions,
chapters, source hashes and the quiet stationary holds. Audio measures
-14.13 LUFS integrated and -1.58 dBTP.
The native build reports no QA problems. Line sheets and full-size final
frames were inspected for typography, legibility, caption placement and
answer timing. The viewer-copy originality screen found no shared 12-word
or six-word windows; wording lint passes.

Six new or revised narration clips matched their scripts at 1.00 in the local
speech check. The other 46 clips were reused. The earlier automatic “Parcel A”
transcription flag remains a listen-through item; it was not reclassified by
this pass. Synthetic narration and media checks do not establish human
comprehension or approval for publication.

The measured base draft took 108.9 seconds, including
the six new voice clips and scene captures. The timed review export took
26.8 seconds: 135.7 seconds of
rendering/export work in total. Writing, design and review are separate from
these measurements. Both stages use local installed tools.

[Render receipt](build/review-v01-quine/render-receipt.json) ·
[Media checks](build/review-v01-quine/media-checks.json) ·
[Render timing](build/quine-render-timing.json) ·
[Speech check](build/quine-speech-check.txt)

Export SHA-256: `7c128371af625a0c47170bc56fb4d66135df89ff575cec9226365ecb487a62e2`.

<a id="main-video-review-record--2026-11-17"></a>
## Earlier parcel review record — 2026-11-17

This record describes the parcel video before the course-introduction and QUINE revision.

The approved parcel lesson is now a local review video. It is still a draft,
with synthetic `piper:en_US-joe-medium` narration at length scale 1.04. The
creator's recorded voice, a viewer comprehension check and the separate Short
remain pending. This record does not approve or publish the video.

The measured file is 358.798 seconds (about 5:59), 1920×1080 at 30 fps, H.264
with AAC stereo at 48 kHz. It has 97 caption cues and eight chapters. The three
speech gaps are 6.013, 4.013 and 8.013 seconds, within one frame of their planned
6 / 4 / 8 seconds. The retained invitation scenes contain no answer reveal.
The final audio measures −14.12 LUFS integrated and −1.51 dBTP.

The base Vidni build reports no QA problems. Independent export checks passed
for codecs, stream lengths, the ordered captions, chapters, exact source hashes,
the three stationary thinking holds and the absence of speech within them.
The line sheets and selected full-size export frames were visually reviewed.
The speech checker matched 49 of 50 clips above its 0.90 threshold. It still
transcribes “Parcel A” as “Parsalate” in the first-note clip (0.89); the complete
spoken sentence is “The first says, ‘Parcel A is packed.’” Retain this flag for
the creator's listen-through rather than treating an automatic score as a
human judgment of pronunciation or understanding.

Viewer-copy lint and local originality screening passed: no shared 12-word or
six-word windows with the comparison corpus. The approved continuous text and
historical source notes are preserved. No engine output is represented or
recaptured in this conceptual lesson.

[Render receipt](build/review-v01-parcels/render-receipt.json) ·
[Media checks](build/review-v01-parcels/media-checks.json) ·
[Review frames](build/review-v01-parcels/review-frames/)

The export SHA-256 is
`b7aa37f19c299cf0c75a874c7a41ce11cfd0f4325c90d4f1d63c7d517690aba4`.

## QUINE production recipe

The current source creates both aspect ratios with the QUINE palette, Space
Grotesk display text, IBM Plex Mono labels and IBM Plex Sans prose. The native
`../themes/quine` theme supplies captions and framing. Targeted focus rectangles
are keyed to narration lines; letter highlights distinguish A/B and 1/2 without
hiding the surrounding evidence. Short scenes are composed for the available
vertical view, with captions and lesson branding in their separate safe bands.

The generated assets live in `build/highlight-assets/`. This recipe edits no
theme, historical HTML, takes or fixture. Together with `vidni.toml`, this note
is the editable production source. Extract the block to a temporary Python file
and run `python3 <file> <project-folder>` before drafting. Run Vidni commands in
its pinned Nix environment with `VIDNI_ASSISTANT=1`.

<!-- V01_VISUAL_RECIPE_START -->
```python
from pathlib import Path
from html import escape
import sys

# QUINE colors resolved from the design handoff's semantic tokens.
# Color source SHA-256: be4e71c0cb6877240238d46b82d21c2c83033689d89ed5b1355c30279287fce8
# Display: Space Grotesk; labels: IBM Plex Mono; prose: IBM Plex Sans.
# Native SVG rendering uses the same pinned font families as Vidni's theme.
QUINE = {
  "void-1000": "#07070A",
  "void-900": "#0B0B10",
  "void-800": "#111119",
  "void-700": "#181822",
  "void-600": "#20202C",
  "void-500": "#2A2A38",
  "void-400": "#3A3A4A",
  "paper-100": "#F4F3EF",
  "paper-200": "#C9C8C2",
  "fg-muted": "#908F9A",
  "fg-faint": "#5E5E6C",
  "fg-ghost": "#3C3C49",
  "ember-50": "#FFE9E0",
  "ember-300": "#FF9A6F",
  "ember-500": "#F2542D",
  "ember-600": "#D8431B",
  "ember-700": "#B23413",
  "phosphor-300": "#8BF0CE",
  "phosphor-500": "#38E3A6",
  "phosphor-700": "#16A877",
  "quanta-300": "#B6A8FF",
  "quanta-500": "#8B7CF6",
  "quanta-700": "#5B4ACb",
  "amber-500": "#F5B544",
  "sky-500": "#4DB8FF",
  "crimson-500": "#FF4D5E",
  "text-on-ember": "#2A0A00",
  "bg-base": "#0B0B10",
  "bg-sunken": "#07070A",
  "bg-raised": "#111119",
  "surface-card": "#181822",
  "surface-inset": "#20202C",
  "surface-hover": "#20202C",
  "text-strong": "#F4F3EF",
  "text-body": "#C9C8C2",
  "text-muted": "#908F9A",
  "text-faint": "#5E5E6C",
  "border-subtle": "#2A2A38",
  "border-strong": "#3A3A4A",
  "border-focus": "#F2542D",
  "accent": "#F2542D",
  "accent-hover": "#FF9A6F",
  "accent-press": "#D8431B",
  "alive": "#38E3A6",
  "symbol": "#8B7CF6",
  "success": "#38E3A6",
  "warning": "#F5B544",
  "info": "#4DB8FF",
  "danger": "#FF4D5E"
}
project=Path(sys.argv[1]).resolve()
out=project/'build/highlight-assets'
out.mkdir(parents=True,exist_ok=True)
BG=QUINE['bg-base']
INK=QUINE['text-strong']
BODY=QUINE['text-body']
MUTED=QUINE['text-muted']
ACCENT=QUINE['accent']
AMBER=QUINE['warning']
CARD=QUINE['surface-card']
EDGE=QUINE['border-subtle']

def text(x,y,s,size=38,fill=INK,weight=400,anchor='start',family=None):
    family=family or ('Space Grotesk' if size>=55 else 'IBM Plex Mono' if s.isupper() or s.startswith('//') else 'IBM Plex Sans')
    spacing=' letter-spacing="0.12em"' if family=='IBM Plex Mono' and size<32 else ''
    return f'<text x="{x}" y="{y}" font-family="{family}" font-size="{size}" font-weight="{weight}" fill="{fill}" text-anchor="{anchor}"{spacing}>{escape(s)}</text>'

def rect(x,y,w,h,fill,rx=10,stroke=None):
    return f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{min(rx,10)}" fill="{fill}"'+(f' stroke="{stroke}" stroke-width="2"' if stroke else '')+'/>'

def line(x1,y1,x2,y2,c=MUTED,width=2):
    return f'<path d="M{x1} {y1} L{x2} {y2}" fill="none" stroke="{c}" stroke-width="{width}"/>'

def heading(kicker,title):
    return text(80,64,'// '+kicker.upper(),24,ACCENT,600)+text(80,144,title,64,INK,600)

def footer(s):
    return text(80,716,s,28,MUTED)

def save(name,body):
    grid=''
    if name in {'welcome','course-title','symbolic-intro','neuro-symbolic-intro','next'}:
        grid=f'<defs><pattern id="grid" width="24" height="24" patternUnits="userSpaceOnUse"><path d="M24 0 H0 V24" fill="none" stroke="{QUINE["fg-ghost"]}" stroke-width="1"/></pattern><radialGradient id="fade"><stop offset="0" stop-color="{INK}"/><stop offset="1" stop-color="{BG}"/></radialGradient><mask id="grid-mask"><rect width="1600" height="760" fill="url(#fade)"/></mask></defs><rect width="1600" height="760" fill="url(#grid)" opacity="0.23" mask="url(#grid-mask)"/>'
    (out/f'{name}.svg').write_text(f'<svg xmlns="http://www.w3.org/2000/svg" width="1600" height="760" viewBox="0 0 1600 760">{rect(0,0,1600,760,BG,0)}{grid}{body}</svg>')

def parcel(x,y,label,scale=1):
    # Rear/side view: address-label faces remain outside the illustration.
    return f'<g transform="translate({x} {y}) scale({scale})">'+ \
        f'<path d="M0 70 L175 15 L310 78 L130 140 Z" fill="{QUINE["fg-muted"]}" stroke="{QUINE["paper-200"]}" stroke-width="2"/>'+ \
        f'<path d="M0 70 L130 140 L130 310 L0 240 Z" fill="{QUINE["void-500"]}" stroke="{QUINE["fg-muted"]}" stroke-width="2"/>'+ \
        f'<path d="M130 140 L310 78 L310 245 L130 310 Z" fill="{QUINE["void-400"]}" stroke="{QUINE["fg-muted"]}" stroke-width="2"/>'+ \
        f'<path d="M84 44 L118 33 L251 102 L217 114 Z" fill="{QUINE["ember-300"]}"/>'+ \
        f'<path d="M217 114 L251 102 L251 160 L217 171 Z" fill="{ACCENT}"/>'+ \
        text(155,370,'Parcel '+label,40,INK,600,'middle','IBM Plex Mono')+'</g>'

def note(x,y,who,body,highlight=False,width=620):
    return rect(x,y,width,195,CARD,10,QUINE['border-strong'])+ \
        text(x+48,y+70,who,40,INK,600,family='IBM Plex Mono')+ \
        text(x+48,y+145,body,42,BODY,500)

def requirement(y=175,terms=False):
    return rect(80,y,1440,95,CARD,10,EDGE)+ \
        line(82,y+2,1518,y+2,ACCENT,3)+ \
        text(112,y+39,'// RULE' if terms else '// THE SHOP’S TWO CHECKS',22,ACCENT,600)+ \
        text(112,y+77,'Packed',36,INK,500)+text(245,y+77,'·',36,MUTED)+ \
        text(275,y+77,'Own address label attached',36,INK,500)+text(790,y+77,'·',36,MUTED)+ \
        text(820,y+77,'For the same parcel',36,INK,500)

def pair_notes(second='B',highlight=False,terms=False):
    s=requirement(175,terms)
    s+=note(80,340,'Parcel A','Packed',highlight)
    s+=note(820,340,'Parcel '+second,'Label attached',highlight)
    if terms: s+=text(80,315,'PREMISES — the statements we start from',28,ACCENT,600)
    return s

def shop(checks=False,claim=False):
    s=heading('At the shop','Preparing orders for collection')
    s+=rect(90,552,940,28,QUINE['border-strong'],6)+rect(130,580,20,70,QUINE['border-strong'],3)+rect(950,580,20,70,QUINE['border-strong'],3)
    s+=parcel(180,285,'A',.68)+parcel(620,285,'B',.68)
    s+=rect(1120,300,390,260,QUINE['surface-card'],20,QUINE['border-strong'])
    s+=text(1315,405,'Collection',36,INK,500,'middle')+text(1315,452,'area',36,INK,500,'middle')
    s+=text(560,650,'Packing table',29,MUTED,400,'middle')
    if checks: s+=requirement(175)
    if claim:
        s+=rect(102,185,850,80,QUINE['surface-inset'],16)+text(130,237,'“This one’s ready.” — colleague, about A',35,INK,500)
    s+=footer('Address-label faces are out of view. We will work from the team’s notes.')
    return s

save('course-title',text(80,104,'// NEURO-SYMBOLIC LESSON 1',28,ACCENT,600)+
     line(80,152,1520,152,EDGE)+text(80,292,'Neuro-Symbolic Reasoning',80,INK,600)+
     text(80,388,'from First Principles',64,INK,600)+
     text(80,544,'An everyday starting point.',40,BODY)+footer('A Confident Claim Still Needs Evidence'))
save('symbolic-intro',heading('The foundation','Symbolic reasoning')+
     rect(80,252,1440,292,CARD,10,EDGE)+line(82,254,1518,254,ACCENT,3)+
     text(128,344,'Information and rules',48,INK,600)+
     text(128,428,'Evaluate which claims follow from them.',42,BODY)+
     footer('We will learn to make the reasoning explicit.'))
save('neuro-symbolic-intro',heading('The course ahead','Neuro-symbolic reasoning')+
     rect(80,252,640,310,CARD,10,EDGE)+rect(880,252,640,310,CARD,10,EDGE)+
     line(82,254,718,254,ACCENT,3)+line(882,254,1518,254,QUINE['symbol'],3)+
     text(128,336,'Neural AI',44,INK,600)+text(128,424,'Language models',36,BODY)+
     text(128,482,'that generate text',36,BODY)+
     text(928,336,'Symbolic reasoning',44,INK,600)+text(928,424,'Explicit statements',36,BODY)+
     text(928,482,'and rules',36,BODY)+text(800,424,'+',64,ACCENT,600,'middle')+
     footer('We will explore how these can work together.'))
save('welcome',text(80,112,'// NEURO-SYMBOLIC LESSON 1',28,ACCENT,600)+line(80,160,1520,160,EDGE)+
     text(80,296,'A confident claim',88,INK,600)+text(80,408,'still needs evidence',88,INK,600)+
     text(80,548,'Start with an everyday situation.',38,BODY)+
     parcel(1170,280,'A',.68)+footer('No formal logic background needed.'))
save('shop',shop())
save('shop-checks',shop(True))
save('colleague',shop(False,True))
save('records-context',heading('Working from records','The packing team’s notes')+
     note(80,265,'Packing confirmation','A team observation')+
     note(820,265,'Label confirmation','A team observation')+
     text(80,590,'For this example, we take the notes as accurate.',38,ACCENT)+
     footer('The viewer has not inspected the parcels.'))
save('first-note',heading('Read the notes','One packing confirmation')+
     note(80,300,'Parcel A','Packed')+footer('The first record from the packing team.'))
save('two-notes',heading('Read the notes','Each note names its parcel')+pair_notes()+
     footer('Claim: Parcel A is ready.'))
save('mismatch',heading('Follow each record','They describe different parcels')+pair_notes(highlight=True)+
     footer('Both checks need to concern the parcel in the claim.'))
save('missing',heading('What the notes support','A’s label is not established')+
     text(80,240,'PARCEL A',29,ACCENT,600)+text(820,240,'PARCEL B',29,ACCENT,600)+
     note(80,275,'Parcel A','Packed')+note(820,275,'Parcel B','Label attached')+
     text(80,555,'A’s label check:',35,INK,500)+text(80,605,'not established by these notes',35,AMBER)+
     footer('A missing confirmation does not show that the label is absent.'))
save('possibilities',heading('The physical situation is unsettled','Either could fit these notes')+
     rect(80,245,700,300,QUINE['surface-card'])+rect(820,245,700,300,QUINE['surface-card'])+
     text(120,305,'POSSIBILITY',25,ACCENT,600)+text(860,305,'POSSIBILITY',25,ACCENT,600)+
     text(120,385,'A already has its label.',38,INK,500)+text(120,450,'The note is still unwritten.',34,MUTED)+
     text(860,385,'A’s label is not attached yet.',35,INK,500)+text(860,450,'No label note is available.',34,MUTED)+
     footer('Neither possibility is an observation of A.'))
save('need-a',heading('Find the relevant evidence','We need A’s own label confirmation')+
     note(80,290,'Parcel A','Packed')+
     rect(820,290,700,195,QUINE['surface-card'],15,QUINE['accent'])+
     text(860,365,'Evidence still needed:',35,ACCENT,600)+text(860,435,'A’s own label is attached',37,INK,500)+
     footer('Re-reading either existing note does not settle this check.'))
save('new-note',heading('A new observation','Someone checks A’s own label')+
     note(80,320,'Parcel A','Packed')+note(820,320,'Parcel A','Label attached',True)+
     text(820,285,'NEW CONFIRMATION',26,ACCENT,600)+
     footer('The new note records the check that was missing.'))
save('repaired',heading('Connect the evidence','Both notes now concern A')+pair_notes(second='A')+
     footer('The two notes meet the shop’s requirement for A.'))
save('supported',heading('Connect the evidence','A meets this checklist')+pair_notes(second='A',highlight=True)+
     footer('The new observation completes the support.'))
save('premises',heading('Name the parts we have used','Premises: our starting statements')+
     pair_notes(second='A')+text(80,310,'PREMISES',28,ACCENT,600)+
     footer('Two notes about the same parcel.'))
save('rule',heading('Name the parts we have used','A rule connects the two checks')+
     pair_notes(second='A',terms=True)+footer('Applying this rule supports the claim that A meets the checklist.'))

def drawing(x,y,num):
    # Artwork centres only; title and name areas remain outside the view.
    s=rect(x,y,600,250,QUINE['border-strong'],10)+rect(x+16,y+16,568,218,QUINE['paper-100'],4)
    if num==1:
        s+=f'<circle cx="{x+440}" cy="{y+75}" r="32" fill="{ACCENT}"/>'
        s+=f'<path d="M{x+17} {y+232} L{x+150} {y+105} L{x+280} {y+232} M{x+180} {y+232} L{x+360} {y+84} L{x+565} {y+232}" fill="{QUINE["quanta-700"]}"/>'
    else:
        s+=f'<path d="M{x+285} {y+230} L{x+285} {y+110}" stroke="{QUINE["void-400"]}" stroke-width="16"/>'
        s+=f'<circle cx="{x+285}" cy="{y+100}" r="73" fill="{QUINE["phosphor-700"]}"/>'
    return s+text(x+300,y+300,f'Drawing {num}',36,INK,600,'middle','IBM Plex Mono')

save('display',heading('A small variation','Getting a class display ready')+
     drawing(100,245,1)+drawing(900,245,2)+
     text(80,650,'Each drawing needs its title and the artist’s name beside it.',37,ACCENT)+
     footer('Artwork centres only; the title/name areas are outside the view.'))
save('drawing-first',heading('The classroom helpers’ notes','One title confirmation')+
     note(80,320,'Drawing 1','Title in place')+
     text(80,230,'Each drawing needs: its title · its artist’s name',37,ACCENT)+
     footer('These are the helpers’ records.'))
save('drawing-notes',heading('The classroom helpers’ notes','Each note names its drawing')+
     note(80,320,'Drawing 1','Title in place')+note(820,320,'Drawing 2','Artist’s name in place')+
     text(80,230,'Each drawing needs: its title · its artist’s name',37,ACCENT)+
     footer('Claim: Drawing 1 is ready to go on the wall.'))
save('drawing-missing',heading('Follow each record','Drawing 1 needs its own confirmation')+
     note(80,280,'Drawing 1','Title in place',True)+note(820,280,'Drawing 2','Artist’s name in place',True)+
     text(80,565,'Still needed: Drawing 1’s own artist-name confirmation.',37,AMBER)+
     footer('The current notes leave its readiness unsettled.'))
save('drawing-repaired',heading('Once that confirmation arrives','Both requirements have support')+
     note(80,300,'Drawing 1','Title in place')+note(820,300,'Drawing 1','Artist’s name in place',True)+
     text(820,265,'THE ADDITIONAL CONFIRMATION',24,ACCENT,600)+
     footer('This is the change that would complete the support.'))
save('habit',heading('The habit we will build on','Follow evidence back to its subject')+pair_notes(second='A')+
     footer('Then see whether it supports the claim we are making.'))
save('symbolic-bridge',heading('Our starting point','The foundation of symbolic reasoning')+pair_notes(second='A',terms=True)+footer('Explicit statements · A rule · Support for a claim'))
save('next',text(80,130,'// NEXT LESSON',28,ACCENT,600)+line(80,170,255,170,ACCENT,5)+
     text(80,290,'Introducing Nibli KR',83,INK,600)+
     text(80,400,'Nibli',45,ACCENT,600)+text(410,400,'The reasoning engine',40,INK)+
     text(80,485,'Nibli KR',45,ACCENT,600)+text(410,485,'The language for premises and rules',40,INK)+
     text(80,630,'The same parcels. A new way to express the reasoning.',34,MUTED))
save('closing',heading('The same example continues','We still review what goes in')+
     rect(80,240,1440,285,QUINE['surface-card'],20)+
     text(120,325,'Our part',38,ACCENT,600)+text(440,325,'Review the records and the rule’s meaning.',38,INK)+
     text(120,435,'Nibli’s part',38,ACCENT,600)+text(440,435,'Evaluate what follows from those inputs.',38,INK)+
     footer('Next: Introducing Nibli KR'))
print(f'Generated {len(list(out.glob("*.svg")))} scene illustrations in {out}')

# Appended to the QUINE scene recipe. Each focus rectangle names the exact
# displayed text; the surrounding scene stays visible during the explanation.
def focus_mark(x,y,w,h):
    return f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="4" fill="{ACCENT}" fill-opacity="0.09" stroke="{ACCENT}" stroke-width="3"/>'

def letter(x,y,label):
    # IBM Plex Mono: 0.6 em advance at the 40px note-label size.
    return [x+48+(len(label)-1)*24-6,y+28,36,54]

def value(x,y,width):
    return [x+40,y+96,width,58]

FOCUS = {
 'welcome-02': ('symbolic-intro','Information and rules',[[116,296,580,64]]),
 'welcome-03': ('neuro-symbolic-intro','Language models and explicit statements/rules',[[116,378,530,126],[916,378,540,126]]),
 'shop-01': ('shop','The two named parcels',[[208,504,156,44],[648,504,156,44]]),
 'shop-02': ('shop-checks','The two required checks',[[104,216,140,48],[264,216,520,48]]),
 'shop-03': ('shop-checks','The same parcel',[[808,216,392,48]]),
 'shop-04': ('colleague','The colleague’s claim about A',[[122,196,794,56],[208,504,156,44]]),
 'shop-05': ('records-context','Working from the notes',[[72,544,1140,62]]),
 'shop-06': ('records-context','The stated accuracy assumption',[[72,544,1140,62]]),
 'records-01': ('first-note','The packing confirmation', [value(80,300,176)]),
 'records-02': ('two-notes','The label confirmation', [value(820,340,340)]),
 'records-05': ('mismatch','The different parcel letters',[letter(80,340,'Parcel A'),letter(820,340,'Parcel B')]),
 'records-06': ('mismatch','The same-parcel requirement',[[808,216,392,48]]),
 'records-08': ('mismatch','Which parcel each note describes',[letter(80,340,'Parcel A'),letter(820,340,'Parcel B')]),
 'records-09': ('missing','B’s label is evidence about B',[letter(820,275,'Parcel B')]),
 'records-10': ('missing','What is not established',[[72,568,618,52]]),
 'missing-support-01': ('missing','Missing evidence is not an absent label',[[72,674,1330,58]]),
 'missing-support-02': ('possibilities','One possible situation',[[112,344,620,130]]),
 'missing-support-03': ('possibilities','The other possible situation',[[852,344,620,130]]),
 'missing-support-05': ('missing','Unestablished by these notes',[[72,568,618,52]]),
 'missing-support-06': ('missing','The label check still lacks evidence',[[72,512,618,110]]),
 'missing-support-07': ('need-a','Evidence about A’s own label',[[852,390,600,62]]),
 'repair-01': ('new-note','The new label confirmation',[letter(820,320,'Parcel A'),value(820,320,340)]),
 'repair-02': ('new-note','The new note is about A',[letter(820,320,'Parcel A'),value(820,320,340)]),
 'repair-03': ('repaired','Both notes now name A',[letter(80,340,'Parcel A'),letter(820,340,'Parcel A')]),
 'repair-06': ('supported','The observation that completes the support',[value(820,340,340)]),
 'terms-02': ('premises','The starting statements',[value(80,340,176),value(820,340,340)]),
 'terms-03': ('rule','The rule connecting the two checks',[[104,216,1098,48]]),
 'terms-04': ('rule','The same parcel in both starting statements',[letter(80,340,'Parcel A'),letter(820,340,'Parcel A')]),
 'variation-01': ('display','The title and artist-name requirement',[[72,606,1410,60]]),
 'variation-03': ('drawing-first','Drawing 1’s title',[value(80,320,310)]),
 'variation-04': ('drawing-notes','Drawing 2’s artist name',[value(820,320,530)]),
 'variation-05': ('drawing-notes','The claim is about Drawing 1',[[72,674,850,58]]),
 'variation-07': ('drawing-missing','Different drawings',[letter(80,280,'Drawing 1'),letter(820,280,'Drawing 2')]),
 'variation-08': ('drawing-repaired','The additional confirmation for Drawing 1',[letter(820,300,'Drawing 1'),value(820,300,530)]),
 'variation-09': ('drawing-missing','The current evidence is incomplete',[[72,520,1410,62]]),
 'bridge-01': ('habit','The subject of each record',[letter(80,340,'Parcel A'),letter(820,340,'Parcel A')]),
 'bridge-05': ('symbolic-bridge','Statements and their connecting rule',[[104,216,1098,48],[72,274,810,54]]),
 'bridge-02': ('next','Engine and language are distinct',[[72,352,270,62],[72,438,270,62]]),
 'bridge-04': ('closing','The two roles',[[112,276,278,62],[112,388,278,62]]),
}
for suffix,(base,description,rectangles) in FOCUS.items():
    svg=(out/f'{base}.svg').read_text()
    marks='<g id="attention" aria-label="'+escape(description)+'">'+''.join(focus_mark(*r) for r in rectangles)+'</g>'
    (out/f'focus-{suffix}.svg').write_text(svg.replace('</svg>',marks+'</svg>'))

# Compact compositions for Vidni's vertical safe view. Native captions and the
# QUINE lesson label occupy their own bands, outside these 900×620 illustrations.
def stitle(title):
    return text(48,78,title,54,INK,600)

def sfooter(message):
    return text(48,572,message,28,BODY)

def snote(y,who,body):
    return rect(48,y,804,144,CARD,10,QUINE['border-strong'])+text(76,y+52,who,44,INK,600,family='IBM Plex Mono')+text(76,y+112,body,42,BODY,500)

def sletter(y,label):
    return [76+(len(label)-1)*26.4-6,y+10,39,54]

def svalue(y,width):
    return [68,y+68,width,56]

def ssave(name,body,marks=()):
    highlights=''.join(focus_mark(*r) for r in marks)
    svg=f'<svg xmlns="http://www.w3.org/2000/svg" width="900" height="620" viewBox="0 0 900 620">{rect(0,0,900,620,BG,0)}{body}{highlights}</svg>'
    (out/f'{name}.svg').write_text(svg)

ssave('short-setup',stitle('Orders for collection')+parcel(132,134,'A',.66)+parcel(554,134,'B',.66)+
      rect(76,408,750,22,QUINE['border-strong'],4)+sfooter('The shop is preparing two parcels.'))
ssave('short-checks',stitle('Before collection')+text(48,140,'For each parcel',32,MUTED)+
      snote(180,'Check 1','Packed')+snote(348,'Check 2','Own address label attached'),
      [svalue(180,190),svalue(348,720)])
ssave('short-claim',stitle('The colleague’s claim')+
      text(76,254,'“Parcel A is ready.”',64,INK,600)+parcel(302,304,'A',.44),[[65,190,736,84]])
ssave('short-accurate',stitle('The team’s notes')+
      rect(48,192,804,280,CARD,10,EDGE)+text(88,270,'For this example,',44,INK,600)+
      text(88,346,'the notes are accurate.',44,INK,600)+sfooter('The claim is evaluated from these records.'),[[76,292,726,72]])
ssave('short-packed',stitle('The first note')+snote(180,'Parcel A','Packed'),[svalue(180,190)])
ssave('short-label',stitle('The second note')+snote(180,'Parcel A','Packed')+snote(348,'Parcel B','Label attached'),[svalue(348,370)])
ssave('short-mismatch',stitle('Different parcels')+snote(180,'Parcel A','Packed')+snote(348,'Parcel B','Label attached'),[sletter(180,'Parcel A'),sletter(348,'Parcel B')])
ssave('short-uncertainty',stitle('A’s label is unestablished')+
      text(48,202,'A’s label might be attached.',40,BODY)+text(48,272,'It might not be attached yet.',40,BODY)+
      rect(48,352,804,140,CARD,10,EDGE)+text(76,410,'These notes do not settle',40,INK,500)+
      text(76,466,'A’s actual label status.',40,INK,500),[[65,414,680,62]])
ssave('short-observation',stitle('A new observation')+snote(180,'Parcel A','Packed')+snote(348,'Parcel A','Label attached')+
      text(48,552,'NEW: A’s label is confirmed',26,ACCENT,600),[sletter(348,'Parcel A'),svalue(348,370)])
ssave('short-repair',stitle('Both checks concern A')+snote(180,'Parcel A','Packed')+snote(348,'Parcel A','Label attached'),[sletter(180,'Parcel A'),sletter(348,'Parcel A')])
ssave('short-closing',stitle('Symbolic reasoning')+
      rect(48,188,804,288,CARD,10,EDGE)+text(80,270,'Explicit information',46,INK,600)+
      text(80,344,'and rules',46,INK,600)+text(80,426,'support a claim.',46,BODY)+
      sfooter('Neuro-Symbolic Lesson 1'),[[68,372,650,68]])
print(f'Generated {len(FOCUS)} focused long-form scenes and 11 Short scenes.')

```
<!-- V01_VISUAL_RECIPE_END -->

### Continuous review exports

After `VIDNI_ASSISTANT=1 vidni draft <project-folder>`, run the following recipe
as `python3 <file> <project-folder>`. It copies both native review cuts and their
captions/timelines into a versioned review folder and records hashes. It does
not insert silence, freeze frames, retime captions or alter narration. The
earlier hold-insertion recipe is retained only in the archived previous inputs.

<!-- V01_EXPORT_RECIPE_START -->
```python
from pathlib import Path
import hashlib, json, shutil, sys

project=Path(sys.argv[1]).resolve()
draft=project/'build/draft'
out=project/'build/review-v01-highlighted'
out.mkdir(parents=True,exist_ok=True)
def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()
receipt={'method':'Unmodified native review exports; no added response gaps.',
         'synthetic_voice':'piper:en_US-joe-medium',
         'config_sha256':sha(project/'vidni.toml'),'cuts':{}}
for cut in ['long','short']:
    files={}
    for suffix in ['.mp4','.en.srt','.en.vtt','.timeline.json','.chapters.txt','.chapters.ffmeta','.build.json']:
        source=draft/(cut+suffix)
        if source.exists():
            target=out/source.name
            shutil.copy2(source,target)
            assert sha(source)==sha(target)
            files[target.name]={'sha256':sha(target),'bytes':target.stat().st_size}
    receipt['cuts'][cut]=files
(out/'render-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(out)

```
<!-- V01_EXPORT_RECIPE_END -->

## Historical notes-app prototype — preserved

Everything below this heading is the previous prototype's source notes and
production record, retained verbatim. It describes the archived notes-app `vidni.toml`
and videos, not the current parcel lesson above. Its links and source anchors
remain available as historical evidence.

# V01 — Making and Checking a Claim

Revised production draft: a notes-app release, explained before any formal
notation. Voice: synthetic Piper en_US-joe-medium. Both cuts remain drafts.

### Fictional policy

Imagine a team maintaining a small app that saves and reopens notes. VersionOne
is an earlier package; VersionTwo is the new package the team wants to offer
on its download page. Each name refers to one fixed package for this example.
No package changes during the session, and no statement says the earlier one
has already been released. "Ready to release" means meeting the team's stated
two-condition checklist, making that version eligible for the download page.

Automated tests run checks on the app, such as saving and reopening a note.
A passing test report records a successful outcome, rather than just a test
having run. A code review is a teammate's examination of the changes, followed
by approval or requested fixes. Both kinds of report name a particular version.
Here they are invented records, with accuracy assumed for the worked example.

The checklist in ordinary language:

```text
For this same version:
  automated tests passed AND code review approved
  -> ready to release on the download page
```

Initially VersionTwo has a passing test report and VersionOne has an approved
code review. VersionTwo's own approval is absent. Repeating its test premise
adds no approval; adding its own approved-review premise completes the rule.
VersionOne still lacks a supplied passing-test report. A missing record does
not establish a failed test, a rejected review, or a defective app.

The encoded names are VersionOne, VersionTwo, TestsPassed, ReviewApproved and
ReadyForRelease. `member(version, list)` records membership in a named list.
The lists model the reports' stated outcomes, not the authenticating evidence:
inspector identity, timestamps, test logs, review contents and file hashes are
not checked by this toy encoding. No ready premise is asserted directly.
The fictional fluent summary deliberately mixes reports from different versions;
it is teaching copy, not output attributed to a real model or person.

Meeting a checklist is not a claim of zero defects and does not itself publish
or authorize a download. The transfer case is an invented newsletter: today's
draft has checked links, yesterday's has editorial approval, and today's needs
its own approval before it meets the stated conditions for emailing subscribers.

### Text generation and checking

Brown et al., [Language Models are Few-Shot Learners](https://arxiv.org/abs/2005.14165),
especially §2.1 and §3.9, is the public reference for autoregressive text
generation. A token is a unit of text. Successive next-token distributions and
decoding produce a continuation; sampling need not choose the most likely token.
The lesson distinguishes this operation from running tests, checking reports,
or checking a derivation. It does not claim models cannot reason. Generated
solutions and tool use can be useful; inspect the evidence and verify the steps.

For a citation, locating the document and checking its relevant passage are
separate operations. A generator repeating its own citation is not independent
confirmation. The illustrative release guide describes a requirement; it does
not supply a review approval for VersionTwo. No fabricated source is presented
as a real document.

### Engine contract

- [Soundness and implementation limits](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/GUARANTEES.md#soundness).
- [Host, claim evaluation and verdicts](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/README.md).
- [Engine implementation](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/nibli-engine/src/lib.rs).

TRUE derives from the encoded premises and rules, given a correct implementation.
This fixture's FALSE is closed-world: not derivable from the supplied knowledge
base, rather than a proof of negation. UNKNOWN and RESOURCE_EXCEEDED are separate
outcomes, not low-confidence TRUE or FALSE. The host's literal `?` selects claim
evaluation and its `[Query]` label is retained verbatim. Plain statements assert
premises. The examples distinguish wrong source data, mistranslated rules,
invalid inference, and inappropriate actions as separate review boundaries.

### Engine receipt

Receipt recorded at 2026-11-17T07:37:06.661245+00:00 for revision `1eab5d1f1d52d1c106145aa0cbc18205ff9b0902`.
The existing native host and WASI component were used; their hashes are below.
The only existing engine checkout change was `nibli-db/tests/db1.rs`, preserved.
The revised fixture and expected results were written before the new runs.
Both fresh sessions use import off, depth 10, quiet mode, and no persistent
database. Materialization off and on both give FALSE, FALSE, TRUE, FALSE;
the TRUE trace cites VersionTwo's test and approval premises and the rule.
Both captures are byte-identical. Their complete on-profile capture is embedded
below without repairing the FALSE explanation's internal label. Video scenes
select the verdict and the closed-world note instead of that explanation.

| Artifact | SHA-256 |
|---|---|
| host | `65e695932696f94952714e4247e4c021a1cd376b1c09d3d67158cfc7bdea1703` |
| component | `5a3e52768410e773f7e864e863bc2b05782d59b3e7a90b0a30b8c52679d03bfa` |
| fixture | `6089b935f9294566d4326fac93084a650a451b1e6058336d2b28e50c96a46a04` |
| expectation | `16c7c7dc357fbcede2f2b1534367bf997b3cad06a2a5a7cacfefb2dc21deed44` |
| capture_on | `7c14aedcce46aab7552706cbd23df142a737a8011314ebce059d447d9dada5c1` |
| capture_off | `7c14aedcce46aab7552706cbd23df142a737a8011314ebce059d447d9dada5c1` |

### Reproduction

Save the session below as `ready.nibli` in an engine checkout with the host
and WASI component built. Clear other NIBLI_* settings, then run a fresh process:

```sh
NIBLI_QUIET=1 NIBLI_EXISTENTIAL_IMPORT=0 NIBLI_MAX_CHAIN_DEPTH=10 \
NIBLI_MATERIALIZE=1 \
NIBLI_WASM_PATH=target/wasm32-wasip2/debug/nibli.wasm \
target/debug/nibli-host < ready.nibli
```

Repeat with `NIBLI_MATERIALIZE=0`. Both must match the prewritten verdict sequence
FALSE, FALSE, TRUE, FALSE and cite both VersionTwo reports in the TRUE proof.

### Session input

```text
# An invented notes app has two fixed release candidates: VersionOne and VersionTwo.
# ReadyForRelease means eligible for the download page under this teaching checklist.
# The same version needs a report of passing tests AND an approved code review.
member(VersionTwo, TestsPassed).
member(VersionOne, ReviewApproved).
all $version: member($version, TestsPassed) & member($version, ReviewApproved) -> member($version, ReadyForRelease).
? member(VersionTwo, ReadyForRelease).
# Repeating VersionTwo's passing test report supplies no review approval.
member(VersionTwo, TestsPassed).
? member(VersionTwo, ReadyForRelease).
# Approval of VersionTwo's own code review supplies the missing premise.
member(VersionTwo, ReviewApproved).
? member(VersionTwo, ReadyForRelease).
? member(VersionOne, ReadyForRelease).
```

### Verbatim terminal capture

```text
nibli> member(VersionTwo, TestsPassed).
nibli> member(VersionOne, ReviewApproved).
nibli> all $version: member($version, TestsPassed) & member($version, ReviewApproved) -> member($version, ReadyForRelease).
nibli> ? member(VersionTwo, ReadyForRelease).
[Query] FALSE
[Why] Nothing known establishes that #4(versiontwo) is a member.
[Note: FALSE is closed-world — not derivable from the KB, not a proof of the negation]
  ∃ No witness found -> FALSE
    ∧ Conjunction -> FALSE
      ∧ Conjunction -> FALSE
        ? Not found: member(sk_4(versiontwo)) -> FALSE
          ✗ Rule failed (member → member): condition member(?ev___ev0) not satisfied -> FALSE
    ∧ Conjunction -> FALSE
      ∧ Conjunction -> TRUE
        ▣ Asserted: member(sk_0) [assertions: #0 member(VersionTwo, TestsPassed).] -> TRUE
        ▣ Asserted: member_x1(sk_0, versiontwo) [assertions: #0 member(VersionTwo, TestsPassed).] -> TRUE
      ? Not found: member_x2(sk_0, readyforrelease) -> FALSE
nibli> member(VersionTwo, TestsPassed).
nibli> ? member(VersionTwo, ReadyForRelease).
[Query] FALSE
[Why] Nothing known establishes that #4(versiontwo) is a member.
[Note: FALSE is closed-world — not derivable from the KB, not a proof of the negation]
  ∃ No witness found -> FALSE
    ∧ Conjunction -> FALSE
      ∧ Conjunction -> FALSE
        ? Not found: member(sk_4(versiontwo)) -> FALSE
          ✗ Rule failed (member → member): condition member(?ev___ev0) not satisfied -> FALSE
    ∧ Conjunction -> FALSE
      ∧ Conjunction -> TRUE
        ▣ Asserted: member(sk_0) [assertions: #0 member(VersionTwo, TestsPassed).] -> TRUE
        ▣ Asserted: member_x1(sk_0, versiontwo) [assertions: #0 member(VersionTwo, TestsPassed).] -> TRUE
      ? Not found: member_x2(sk_0, readyforrelease) -> FALSE
    ∧ Conjunction -> FALSE
      ∧ Conjunction -> TRUE
        ▣ Asserted: member(sk_5) [assertions: #3 member(VersionTwo, TestsPassed).] -> TRUE
        ▣ Asserted: member_x1(sk_5, versiontwo) [assertions: #3 member(VersionTwo, TestsPassed).] -> TRUE
      ? Not found: member_x2(sk_5, readyforrelease) -> FALSE
nibli> member(VersionTwo, ReviewApproved).
nibli> ? member(VersionTwo, ReadyForRelease).
[Query] TRUE
[Why] Because VersionTwo is a member of TestsPassed and VersionTwo is a member of ReviewApproved, VersionTwo is a member of ReadyForRelease.
  ⊢ VersionTwo is a member of ReadyForRelease  [by recorded source #2/0: all $version: member($version, TestsPassed) & member($version, ReviewApproved) -> member($version, ReadyForRelease).] -> TRUE
    ▣ VersionTwo is a member of TestsPassed  [given] -> TRUE
    ▣ VersionTwo is a member of ReviewApproved  [given] -> TRUE
nibli> ? member(VersionOne, ReadyForRelease).
[Query] FALSE
[Why] Nothing known establishes that #4(versionone) is a member.
[Note: FALSE is closed-world — not derivable from the KB, not a proof of the negation]
  ∃ No witness found -> FALSE
    ∧ Conjunction -> FALSE
      ∧ Conjunction -> FALSE
        ? Not found: member(sk_4(versionone)) -> FALSE
          ✗ Rule failed (member → member): condition member(?ev___ev0) not satisfied -> FALSE
    ∧ Conjunction -> FALSE
      ∧ Conjunction -> TRUE
        ▣ Asserted: member(sk_1) [assertions: #1 member(VersionOne, ReviewApproved).] -> TRUE
        ▣ Asserted: member_x1(sk_1, versionone) [assertions: #1 member(VersionOne, ReviewApproved).] -> TRUE
      ? Not found: member_x2(sk_1, readyforrelease) -> FALSE
```

### Production review

The app, the two versions, the two successful outcomes, and the meaning of ready
are established in the opening. The reports and rule come next. The learner
predicts the missing approval, then sees the same case encoded and evaluated.
The language-model mechanism follows the worked case, instead of interrupting
the introduction. The Short states its own app and release context.

The initial hardware analogy was rejected by the creator as insufficiently
explained. The revision replaces that context throughout; line IDs are retained
for surviving narration, while obsolete lines are dropped. Scene files and
theme assets are unchanged: cards and overlays are data in vidni.toml.

The AND graphic groups both reports before the conclusion. Labels identify
versions, recorded facts, the assumed checklist and the supported conclusion.
Terminal sources are unedited ranges of the capture above. The checklist scene
is clearly labeled as the invented team's rule, not engine output.

Targets remain 9–13 minutes for the main lesson and 45–60 seconds for the Short.
Private coverage and review receipts remain in the editorial repository. No
undeployed practice link is advertised. Creator narration, recording-day capture
freshness, beginner comprehension review, ratification and release remain open.
