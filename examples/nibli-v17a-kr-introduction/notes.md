# Neuro-Symbolic Lesson 2 — Introducing Nibli KR: Its Purpose and Origins

Main lesson and independent Short · creator-voice final builds · 2026-11-19.

**Production status:** Both cuts are built with the creator's recorded voice:
[captioned main, 5:19.598](out/long.captioned.mp4), [clean main](out/long.mp4)
and [independent Short, 0:49.338](out/short.mp4). Media QA passes; all 80 current
lines pass the automatic speech check after one creator retake. QUINE styling
and corrected targeted highlights are preserved. The opening 39 seconds are
quieter than later narration and remain a final listen-through item. See the
[recorded production review](#recorded-production-review) and the complete
[publishing/sharing copy](#v17a-publishing-copy). The creator reported the main
lesson uploaded on 2026-11-19 and supplied its
[YouTube URL](https://www.youtube.com/watch?v=i0nlnWrlFCw&list=PLAQKTXMK9CyY&index=1).
Learner feedback and native release approval remain separate; no Short
publication has been reported.

The lesson has one goal: distinguish a real situation, the records we make about
it, the language used to represent those records, and the engine that evaluates
claims. It assumes no formal logic or software engineering background. The shop
is established before recalling the previous lesson.

The internal production ID is **V17a**; the viewer-facing label is
**Neuro-Symbolic Lesson 2**. The main lesson follows Lesson 1's parcel example.
Lesson 3 will teach writing statements. No engine transcript or verdict is
shown here.

- [Complete main narration](#complete-main-narration)
- [Complete Short narration](#complete-short-narration)
- [Main storyboard](#main-storyboard)
- [Short storyboard](#short-storyboard)
- [Public source receipts](#public-source-receipts)
- [Review and production handoff](#review-and-production-handoff)
- [Final recorded files and checks](#recorded-production-review)
- [Publishing and sharing copy](#v17a-publishing-copy)
- [Synthetic review files and checks](#synthetic-review-record)

## Complete main narration

Read this as connected speech. The paragraph anchors connect the narration to
the storyboard and sources; they are not spoken headings or recording-line IDs.
The displayed KR line is read through the surrounding English explanation, not
dictated character by character.

<!-- V17A_SCRIPT_START -->
<a id="v17a-shop"></a>

Welcome to Neuro-Symbolic Lesson 2. We're continuing with the symbolic side of
reasoning: writing down information so we can work out which claims it supports.

It's the next day at our small shop. Two parcels, A and B, are on the packing
table. Before either parcel goes to the collection area, it must be packed and
have its own address label attached. That's the shop's meaning of ready for
collection.

Yesterday, a note about A's packing was combined with a note about B's label.
The records described different parcels. Today, a colleague wants to make the
records clearer for the next shift, and eventually use a program to check them.

<a id="v17a-representation"></a>

For that task, we give each parcel a name, record which checks concern it, and
write down the requirement that both checks must concern the same parcel. We're
choosing the things and relationships that matter to the shop's decision, along
with the rule that connects them to readiness.

That is knowledge representation: choosing how to describe information so it
can be used for a purpose. Here, the purpose is to evaluate a claim about a
particular parcel's readiness using the records available.

<a id="v17a-parcel-and-note"></a>

Start with Parcel A itself. It's a physical object on the table. Beside it is
a note saying, “Parcel A is packed.” The note describes the parcel. It can be
copied or passed to a colleague while the parcel stays where it is.

That English note is already a representation. To give a reasoning program
something it can process in a defined way, we'll write the same information in
a formal language.

<a id="v17a-language-and-engine"></a>

The language we'll use is Nibli KR. The letters K R stand for knowledge
representation. Nibli KR gives us a way to write statements and rules.

Nibli is the engine that reads them and evaluates claims. For example, it can
evaluate the claim that Parcel A meets the shop's checklist against the records
and rule supplied to it. The language expresses the information. The engine
does the reasoning with it.

The parcel remains on the table throughout. Writing a statement and evaluating
a claim don't pack it or attach its label.

<a id="v17a-one-statement"></a>

Let's put just the packing note into that language. For this example, we use
a list for parcels with a packing confirmation. Saying “Parcel A is on the
packed-parcels list” is how we represent the note “Parcel A is packed.”

Nibli's vocabulary includes a relation called member. It connects an item to
a group it belongs to. Its two roles are the member and the set, or group.
Here is our statement in Nibli KR.

```nibli-kr
member(ParcelA, PackedParcels).
```

ParcelA names this parcel. PackedParcels names the group we're using for packing
confirmations. Those are names we've chosen for our example. The word member
and its two roles come from the vocabulary that ships with Nibli.

We check that this vocabulary entry expresses the relationship we intend:
membership in a group. The name PackedParcels gets its meaning in our model
from our decision to use that group for packing confirmations.

<a id="v17a-checking-and-limits"></a>

With the relation and its roles made explicit, a program can check the
statement's structure and apply the same reasoning procedure to the same
records. Someone inspecting the statement can also see exactly which parcel
and group it names. Relation words must come from the supported vocabulary;
an English word doesn't become a supported relation just because we type it.

Asserting this statement supplies a premise: something we take as given for
the reasoning. Evaluating a claim is a separate step. It checks the claim's
support against the supplied premises and rules.

There are still two places where our work can go wrong. We could copy A's note
using B's name. The statement could be well formed while describing the wrong
parcel. Or we could copy the note perfectly when the original packing
confirmation was mistaken.

A person checks that the encoding matches the note and the intended meaning
of the vocabulary. The packing confirmation itself needs evidence from the
shop. Formal structure helps us inspect and process the record; it doesn't
establish that the parcel really is packed.

<a id="v17a-origins"></a>

Nibli originally accepted Lojban, a deliberately designed human language; Later,
I retired that input and kept the language we now call Nibli KR,
whose vocabulary still records its Lojban origins.

The statement we've just read uses the current language.

<a id="v17a-library"></a>

The same distinction applies at a library. Someone places a particular copy
of a book, Copy 7, on the returns desk. A staff member records, “Copy 7 was
returned.” That record can then be written in a formal language and used by
an engine to evaluate claims against the library's records and rules.

The returned copy, the note about it, the formal statement and the evaluation
remain separate. An accurate-looking record still needs to match the actual
return of that particular copy.

<a id="v17a-next"></a>

Back at the shop, we have the same four parts: Parcel A, the note describing
it, the statement written in Nibli KR, and Nibli evaluating claims from the
information supplied.

In Lesson 3, we'll write the parcel notes ourselves, read each part of a
statement, and keep supplying a premise separate from evaluating a claim.
That will give us the first pieces of a small knowledge base to reason with.
<!-- V17A_SCRIPT_END -->

## Complete Short narration

Standalone title: **Nibli and Nibli KR: The Engine and the Language**.
Keep **Neuro-Symbolic Lesson 2** as the series label. The opening establishes
its own shop and note; it assumes neither the main lesson nor Lesson 1.

<!-- V17A_SHORT_SCRIPT_START -->
<a id="v17a-short-shop"></a>

A small shop has a parcel on the packing table. We'll call it Parcel A.
A note says, “Parcel A is packed.”

<a id="v17a-short-language"></a>

To use that record in a reasoning program, we can write it in Nibli KR.
K R stands for knowledge representation. For this example, a packed-parcels
list records the shop's packing confirmations. Our statement is “Parcel A is
on the packed-parcels list.” Here it is in Nibli KR.

```nibli-kr
member(ParcelA, PackedParcels).
```

<a id="v17a-short-engine"></a>

Nibli KR is the language used to write the record. Nibli is the engine that
evaluates claims against the records and rules we supply.

<a id="v17a-short-boundary"></a>

Neither checks the physical parcel for us. If the packing note is mistaken,
writing it formally doesn't repair that mistake. We still need evidence that
the parcel was packed.
<!-- V17A_SHORT_SCRIPT_END -->

## Storyboard conventions

The project uses the existing [QUINE theme](../themes/quine/theme.toml).
All custom illustrations, cards, captions and title
treatments must follow it too: near-black `#0B0B10`, ink `#F4F3EF`, muted text
`#908F9A`, ember `#F2542D`, and edge `#3A3A4A`; Space Grotesk for headings,
IBM Plex Sans for prose and captions, and IBM Plex Mono for labels and KR.
Use the theme's `//` label marker. Production-only IDs and source identifiers
do not appear in the video.

Main format: 1920×1080. Short format: 1080×1920, composed vertically rather than
cropped from the main. Reserve the title and caption bands; place essential
Short content inside x=90–900 and y=260–1460, subject to checking the actual
platform overlay in the rendered review. Wrap the single KR statement between
its arguments if necessary; it must remain one readable statement.

Use a restrained ember outline and light fill to highlight the exact word,
parcel letter, note phrase or card named by the narration. Pair color with
the outline and a plain label. Keep context at readable contrast. A highlight
begins on its quoted trigger phrase, moves as the explanation moves, and ends
with that point. Do not flash, pulse, color whole paragraphs, or dim context
until it becomes unreadable. Reveal explanatory labels as they are explained.

The real-world parcel stays a reference image beside the four-part diagram;
its first card represents that situation. Show the parcel as a neutral wrapped
box with a large **A** identification tag, explicitly distinct from an address
label. Do not add a tick, a shipping label or a “verified” badge that would
invent evidence. The two-check requirement is a rule card, not a checklist
whose checks have been marked complete.

The four-part diagram uses **Parcel**, **Note**, **Nibli KR · language** and
**Nibli · engine**. Before the expression is introduced, the KR card contains
only its role, “Write statements and rules.” The engine card says “Evaluate
claims from supplied records and rules.” It never displays a simulated
verdict, a proof trace, or a screenshot of an engine run. Diagrams are teaching
illustrations, not captures of the product UI.

No audience questions, prediction instructions or response holds. Keep normal
speech and scene-transition spacing only. The storyboard ranges below are
planning estimates, not recorded timings, chapter timestamps or instructions
to stretch a scene. The rendered cuts use the measured synthetic narration;
creator recordings will determine the timing of their later final cuts.

## Main storyboard

| Scene / narration | Visible content and reveal order | Highlight trigger and target | Planning time |
|---|---|---|---|
| M01 · [Opening](#v17a-shop), first paragraph | Show “Neuro-Symbolic Lesson 2” and “Introducing Nibli KR” over the shop illustration. Subtitle: “Its Purpose and Origins.” Move into the packing table as the shop is introduced. | On “symbolic side of reasoning,” emphasize “Symbolic” in the series label. | 10–13 s |
| M02 · [Shop setup](#v17a-shop), remaining paragraphs | Reveal parcels A and B, then the rule “Ready for collection = packed + own address label attached.” Reveal the two written notes, “A: packed” and “B: label attached,” followed by “Clear records for the next shift.” Neither parcel receives an address label. | On “its own address label,” outline “own.” On “A's packing,” outline A on its note and parcel; move to B on “B's label.” On “next shift,” move to the handoff label. | 30–36 s |
| M03 · [Choosing the representation](#v17a-representation) | Keep the same shop objects. Add three compact labels in order: “Named parcels,” “Recorded checks,” “Same-parcel rule.” Reveal “Knowledge representation” only when spoken, with “Represent information for a purpose.” | Follow “give each parcel a name,” “record which checks concern it,” and “both checks must concern the same parcel.” End on “evaluate a claim about a particular parcel.” | 31–38 s |
| M04 · [Parcel and note](#v17a-parcel-and-note) | Keep A on the table. Reveal the **Parcel** card, then the **Note** card containing “Parcel A is packed.” A second copy of the note can move toward a colleague's tray while the parcel stays still. | On “physical object,” outline A's box. On “note saying,” outline the sentence. On “already a representation,” keep the note emphasized. | 28–34 s |
| M05 · [Language and engine](#v17a-language-and-engine), first two paragraphs | Reveal **Nibli KR · language**, then **Nibli · engine**. Expand “KR = knowledge representation” once. Keep these cards free of code. Place “Claim: A meets the shop's checklist” beside the engine, with inputs labelled “Supplied records + rule.” | On “write statements and rules,” highlight the KR card's verb “Write.” On “reads them and evaluates claims,” highlight the engine's verb “Evaluate.” Follow the example claim without displaying any result. | 28–34 s |
| M06 · [Physical boundary](#v17a-language-and-engine), last paragraph | Keep the four cards visible. The parcel remains on the table; no arrow points from the engine to a physical action. | On “parcel remains on the table,” move emphasis back to the parcel, leaving the two software-related cards readable. | 8–10 s |
| M07 · [Meaning before notation](#v17a-one-statement), first two paragraphs before the expression | Add a card headed “Packed-parcels list” and sublabel “Our record of packing confirmations,” containing A. Show “Parcel A is on the packed-parcels list.” Then reveal a two-row reading guide: “member · item” and “set · group,” headed “Supported relation: member.” This is an explanatory crop of the vocabulary, not the full source file. | On “packing confirmation,” emphasize that phrase. On the English membership sentence, move from A to the list heading. On “member” and “set,” outline the corresponding role label once each. | 28–34 s |
| M08 · [One KR statement](#v17a-one-statement), expression and next paragraph | Replace the KR card's placeholder with `member(ParcelA, PackedParcels).` Reveal the complete line together. Keep the English reading and the same parcel/list context visible. Add “Names chosen for this example” under the two names. | On “ParcelA names this parcel,” connect the name to A's tag. On “PackedParcels names the group,” connect that name to the list. On “word member and its two roles,” emphasize `member` and its reading guide. Do not animate individual punctuation marks. | 18–22 s |
| M09 · [Vocabulary fit](#v17a-one-statement), last paragraph | Keep the same statement; show “member: item belongs to group” and the existing list sublabel. This is the only distinct KR expression in the main lesson. | On “relationship we intend,” emphasize the membership reading; on “packing confirmations,” move to the list's sublabel. | 14–18 s |
| M10 · [Structure and premise](#v17a-checking-and-limits), first two paragraphs | Retain the expression as a small reference. Show “Supported relation + explicit roles,” then two distinct flow labels: “Supply a premise” and “Evaluate a claim against premises + rules.” Do not introduce UI selectors, command syntax or verdict labels. | On “which parcel and group,” outline the two names in sequence. On “supplies a premise,” highlight that input label; move to “checks the claim's support” on evaluation. | 32–40 s |
| M11 · [Meaning and evidence checks](#v17a-checking-and-limits), remaining paragraphs | Use two English-only comparisons, one at a time: “Note names A / Encoding names B” under “Check the match,” then “Note says packed / Packing still needs evidence” under “Check the observation.” Keep the actual KR line out of this comparison so no second expression is introduced. | On “A's note using B's name,” outline A then B. On “original packing confirmation was mistaken,” emphasize the note's “packed.” On “evidence from the shop,” move back to the physical parcel. | 34–42 s |
| M12 · [Origins](#v17a-origins) | Replace the comparisons with one history card: “Original input: Lojban” → “Current input: Nibli KR”; below, “Vocabulary retains its origins.” Return to today's membership statement after the card. Exact dates stay in the source receipts, following the creator's edit. No retired syntax or former working name appears. | Follow “Lojban,” then “the language we now call Nibli KR.” On “current language,” return emphasis to today's statement. | 17–22 s |
| M13 · [Library variation](#v17a-library) | Establish a returns desk and a book labelled “Copy 7.” Add the note “Copy 7 was returned,” then cards “Formal statement” and “Engine evaluation.” Keep the same four-part layout and show no library KR syntax. | On “particular copy,” emphasize 7. On “staff member records,” emphasize the note. On “match the actual return,” connect the note's 7 back to the copy's 7. | 32–40 s |
| M14 · [Handoff](#v17a-next) | Return to the shop's four cards. Move through them in narration order, then show “Next · Neuro-Symbolic Lesson 3” and “Writing Your First Nibli KR Statements.” Keep “Premise” and “Claim for evaluation” as separate labels. | Follow “Parcel A,” “note,” “statement written in Nibli KR,” and “Nibli evaluating claims.” On “we'll write the parcel notes ourselves,” emphasize “Writing” in the next title; on “supplying a premise” and “evaluating a claim,” follow their separate labels. | 27–34 s |

The composition grows from one parcel to the four-part diagram; it does not
begin with a full slide of unfamiliar labels. Paragraph transitions carry
the story. The dictionary explanation stays brief; writing conventions,
named arguments, rules, variables, connectives, quantifiers and derivation
reading belong to subsequent lessons.

## Short storyboard

| Scene / narration | Vertical composition and visible content | Highlight trigger and target | Planning time |
|---|---|---|---|
| S01 · [Shop and note](#v17a-short-shop) | A small series label at the top; a parcel A above its note, “Parcel A is packed.” Establish the table before revealing the note. Keep the caption area clear below. | On “Parcel A,” outline A's tag; on “note says,” move to the sentence. | 9–10 s |
| S02 · [Language and statement](#v17a-short-language) | Move the note upward and reveal “Nibli KR · language” and “KR = knowledge representation.” Add “Packed-parcels list · packing confirmations,” then the English membership sentence. Reveal the single `member(ParcelA, PackedParcels).` statement only after that sentence is spoken, on “Here it is in Nibli KR.” If wrapping, put `member(ParcelA,` on the first line and `PackedParcels).` on the second. No horizontal four-card strip. | On the first “Nibli KR,” outline the language label. On the list's first mention, outline its plain-English label. Follow “Parcel A” and “packed-parcels list” within the English sentence; on “Here it is in Nibli KR,” emphasize the complete formal statement. | 20–23 s |
| S03 · [Engine](#v17a-short-engine) | Stack two cards: “Nibli KR · write the record” and “Nibli · evaluate claims.” A small input label reads “Supplied records + rules.” Keep the parcel visible as a thumbnail. No mock verdict. | On “language used to write,” outline the first card's verb; on “engine that evaluates claims,” move to the second card's verb. | 10–12 s |
| S04 · [Evidence boundary](#v17a-short-boundary) | Return the parcel and original note to the center. Show “The packing still needs evidence.” End on that lesson point, without a silent CTA or a response hold. | On “physical parcel,” emphasize the box; on “note is mistaken,” emphasize the record; on “evidence,” connect attention back to the box. | 12–14 s |

## Public source receipts

These sources support the engine, language and history claims. The shop and
library situations are original teaching examples, not reports of real events.
The script does not depend on a private manuscript or an unimplemented language
profile. Links below pin the inspected repository revision rather than a moving
branch.

| Receipt | Public source and inspected evidence | Use in this script |
|---|---|---|
| R01 | [README at `1eab5d1f1d52d1c106145aa0cbc18205ff9b0902`](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/README.md): introduction, scoped guarantee, KR language and dictionary-data sections. | Nibli is the engine; KR is its input language; conclusions depend on supplied input; supported vocabulary ships with the engine. |
| R02 | [Current language contract at the same revision](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/NIBLI_KR.md): status header, classification, current statement syntax. Its v2 target is explicitly unimplemented. | Current language, statements and rules, historical Lojban connection. Only the implemented input is taught. |
| R03 | [Committed `member` entry](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/nibli-lexicon/src/corpus/predicates.rs#L1502): name `member`; places `member`, `set`; template “{x1} is a member of {x2}”; provenance field `source_gismu: "cmima"`. | The membership reading, two roles, and documented vocabulary provenance. The provenance field is a source receipt, not syntax to display or teach. |
| R04 | [KR cookbook](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/mdbook/src/user/kr-cookbook.md) and [validator](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/nibli/src/bin/validate.rs). | Current statements, supplied premises versus evaluated claims, and the narrow expression-validation check. No CLI spelling is needed in the narration. |
| H01 | [Frontend retirement commit `975b19f8f857557120ff80a69236c2f483f2bc2b`](https://github.com/dhilipsiva/nibli/commit/975b19f8f857557120ff80a69236c2f483f2bc2b), authored and committed 2026-07-13: message and deletion of the old parser and language selectors. | The July 2026 transition to a single KR input. Do not claim all underlying Lojban-derived semantics disappeared. |
| H02 | [Display-name change `4bfe18c2ca6dbfe4ab4041d8ee00465666dbf45d`](https://github.com/dhilipsiva/nibli/commit/4bfe18c2ca6dbfe4ab4041d8ee00465666dbf45d), 2026-07-13: earlier working name changed to nibli KR in public copy. The last dual-frontend tag `v0.1-lojban-final` resolves locally to `3877ea50836f837b8026e040eef1d2b8b339072b`. | Further reading only; no extra terminology, old syntax or historical release installation in the lesson. |

### Narration-to-source bindings

| Narration anchors | Basis |
|---|---|
| `v17a-shop`, `v17a-representation`, `v17a-parcel-and-note` | Original staged shop example; R01–R02 support the purpose and distinction between representation and evaluation. |
| `v17a-language-and-engine` | R01–R02, with the shop's claim used as an illustration, not a captured result. |
| `v17a-one-statement` | R01–R04; the choice to represent packing confirmations through membership is explicit and belongs to the teaching example. |
| `v17a-checking-and-limits` | R01–R04; the wrong-name and mistaken-observation cases are original controls explaining the input boundary. |
| `v17a-origins` | R01–R03 and H01; H02 corroborates the current name but stays out of the narration. |
| `v17a-library` | Original Copy 7 variation; R01–R02 support the engine/representation boundary. |
| `v17a-next` | Recap of this example and the planned next lesson; no new engine capability. |
| `v17a-short-shop`, `v17a-short-language`, `v17a-short-engine`, `v17a-short-boundary` | Same original shop model and R01–R04; no history claim or additional expression. |

## Review and production handoff

The main narration is expected to fit roughly 5–7 minutes at a comfortable
explanatory pace; its hard ceiling is 13 minutes and it has no minimum. The
Short is planned for 45–60 seconds. The edited main has **870 spoken words** and the
Short **123**, excluding the displayed code and nonspoken markup, and counting
hyphenated words as one. At 140–160 words per minute, the text alone takes about
**5:26–6:13** and **0:46–0:53**, respectively. These are reading-rate estimates.
The synthetic cuts measure **5:30.868** and **0:50.532**; their generated chapter
times are linked below as earlier review material. The final creator-voice cuts
measure **5:19.598** and **0:49.338**. Their measured chapters and captions are
linked in the recorded production review.

- [x] Draft both continuous scripts and their separate QUINE storyboards.
- [x] Tie highlights to narration phrases and introduce the single KR statement
  only after its everyday reading.
- [x] Complete source, expression, editorial and document checks for this draft;
  see the writing check record below. Human review remains separate.
- [x] Incorporate the creator's wording edits and request to render both cuts.
  Preserve the complete scripts while binding stable narration lines to them.
- [x] Creator reviews the synthetic cuts, flags the misplaced highlights and
  records both scripts. The replacement take resolves the one incomplete line.
- [ ] Watch and listen through both final cuts, including the quieter opening.
- [ ] Obtain beginner feedback on distinguishing the four parts and applying
  the distinction to Copy 7; keep this outside the streaming narration.
- [x] Configure Vidni with stable V17a line IDs, source
  bindings and the existing QUINE theme; implement the original illustrations
  and render synthetic previews of both cuts.
- [x] Inspect rendered scene sheets, final video frames, type, contrast, caption
  clearance and narration-linked highlights, including the vertical safe area.
- [ ] Check the complete videos on a physical phone during the human review.
- [x] Record the ratified narration and produce final cuts through the normal
  Vidni workflow. Measure runtime and chapter timing from those cuts.
- [x] With final creator-voice videos, automatically prepare the full publishing
  package: main YouTube title/description with actual chapters; separate Short
  title/description; LinkedIn, Facebook, X.com, Threads and Instagram link posts;
  WhatsApp Status and Instagram Reels captions. Saved here with the Lesson 2
  URL supplied by the creator on 2026-11-19.

### Writing check record

Checked on 2026-11-18 at the writing stage, before the creator's origin edit and
the synthetic production pass below:

- Public source files match the pinned revision. The two history commits and
  the last dual-frontend tag resolve as recorded above.
- Both narration drafts contain exactly one displayed KR statement, introduced
  after its English meaning. Neither contains an audience question, prediction
  task or response hold. The two meaning/evidence controls use English only.
- The existing local `nibli-validate` accepts
  `member(ParcelA, PackedParcels).` with `valid: true`. Its SHA-256 is
  `77edb4f6a99b47cea827ad802f5e7a3d2b3d5d92572b9d2bb9b23f138bf3f385`.
  Its recorded loader was absent, so it ran through the installed glibc 2.42
  loader with the installed libc/libgcc libraries. The binary was unchanged
  and was not rebuilt. This checks the single expression, not a lesson's
  claims about physical parcels or a full engine test suite.
- The local editorial originality screen found no shared 12-word or 6-word
  windows in either the narration-only package or these production notes.
  A clean text screen does not establish independent teaching structure or
  learner comprehension.
- Course structure, local links, coverage consistency and whitespace checks
  pass. The actual layout, caption fit, highlight timing and spoken pacing
  await production and human review.

This writing-stage record did not include rendering or Vidni/media checks.
The production passes below add them. Creator recording and ratification are
now complete; release approval and upload have not taken place for this lesson.

## Synthetic review record

Created on 2026-11-18 following the creator's edits and request for synthetic
narration. The manifest preserves the edited continuous scripts word for word,
including the first-person origin sentence and its removal of the spoken date.
Historical dates remain in the public source receipts. The scripts contain
870 main words and 123 Short words under the counting convention above.

| Cut | Review video | Measured runtime | Picture | Narration / captions |
|---|---|---|---|---|
| Main | [long.mp4](build/draft/long.mp4) | 330.868 s · 5:30.868 | 1920×1080, 30 fps | 67 clips, 95 caption cues |
| Independent Short | [short.mp4](build/draft/short.mp4) | 50.532 s · 0:50.532 | 1080×1920, 30 fps | 13 clips, 13 caption cues |

Both files contain burned captions and AAC stereo audio at 48 kHz. Additional
files: main [SRT](build/draft/long.en.srt), [VTT](build/draft/long.en.vtt) and
[measured chapters](build/draft/long.chapters.txt); Short
[SRT](build/draft/short.en.srt) and [VTT](build/draft/short.en.vtt).
The voice is Piper `en_US-joe-medium`, with length scale 1.04. This is synthetic
review narration, not a recording or imitation of the creator's voice.

The 80 original QUINE SVG scenes follow the narration. The main diagram grows
from the parcel to the note, language and engine; the Short has its own stacked
compositions. The one current KR expression follows its English reading. The
origin card has no date or retired syntax. Captions use the creator's wording;
voice-only pronunciation hints separate the letter A and read KR as letters.
There is no music, audience task or response hold. Natural clip spacing is
0.28 seconds, section spacing 0.75, with 0.6 seconds before the first line and
1.2 seconds after the final line.

**Production verification:** Native validation and media QA pass for both cuts.
The [media receipt](build/review/media-checks.json) independently checks the
delivered hashes, dimensions, codecs, runtime, ordered caption cues, exact
narration text, natural gaps and nine main chapters. Loudness is −14.1 LUFS for
both cuts; true peaks are −1.9 dBTP (main) and −1.8 dBTP (Short). Final frame and
scene-sheet review checked the four-part diagram, membership roles, history
card, library variation and separate vertical composition. A parcel-label
overlap found in the first pass was corrected before these delivered files.
The red occlusion mask in native Short review sheets is a review aid only; it
does not appear in the MP4.

The [source check](build/review/source-checks.json) confirms exact preservation
of the creator's scripts and agreement with every manifest line. Course document
checks pass (73 briefs, 276 coverage families, 818 obligations); local production
links and whitespace checks also pass. A fresh screen of narration, displayed
scene text and the preview description against the 32 canonical manuscripts
finds no shared 12-word or 6-word windows. This remains a mechanical text check,
not a judgment of learner comprehension. All 838 protected file hashes and
1,528 Lesson 1 file metadata entries match the pre-production snapshot.

**Speech review:** The [automatic speech check](build/review/speech-check-delivery.txt)
scores 76 of 80 clips at or above 0.90. Four remain below that threshold. These
are transcription comparisons, not a completed human listen-through:

| Cut / time | Line | Phrase to listen to | Score |
|---|---|---|---|
| Main 1:15.84–1:19.29 | `v17a-parcel-and-note-03` | “Parcel A is packed” within the note sentence | 0.79 |
| Main 2:41.55–2:43.81 | `v17a-one-statement-08` | “ParcelA names this parcel” | 0.77 |
| Short 0:06.19–0:09.04 | `v17a-short-shop-03` | “A note says, ‘Parcel A is packed’” | 0.79 |
| Short 0:26.58–0:28.57 | `v17a-short-language-05` | “Here it is in Nibli KR” | 0.88 |

Creator listen-through, physical-phone viewing and beginner feedback remain
separate from these checks. No recording, ratification, release approval or
upload was performed in this synthetic pass. The subsequent recorded production
review below supersedes its recording handoff and supplies the publishing copy.

### Highlight correction — 2026-11-18

The creator reported misplaced highlights around 2:40–2:56 and 3:20. Two
rectangles in the shared statement layout used guessed footer offsets: one
began midway through a phrase and extended into empty space, and the other
included words from the next phrase. The initial frame review missed these.

The corrected layout highlights `ParcelA` and `PackedParcels` directly in the
statement and their matching card labels. When the narration discusses the
vocabulary, it highlights `member` and the already-introduced member/set roles.
The offset footer rectangles are removed. Card labels use the same monospace
advance as their highlights so each box follows its own text. This correction
applies to all 11 uses of the statement layout, including the repeat at 3:20.

The [main video](build/draft/long.mp4) has been rerendered. Native media QA and
the refreshed [media receipt](build/review/media-checks.json) pass. Frames from
the delivered MP4 were inspected at 2:42.5, 2:46, 2:49.5, 2:53, 3:20 and 3:26.
The [correction receipt](build/review/highlight-correction/checks.json) records
the affected scenes and unchanged hashes: narration, script, voice choices,
captions, timeline, chapter times and the Short. The main remains 5:30.868.
Recording words and line IDs are unchanged; this visual fix requires no retakes.

<a id="recorded-production-review"></a>
## Recorded production review — 2026-11-19

The creator ratified both cuts and recorded all 80 line IDs. One incomplete
take was replaced by the creator during this pass. The final files use only
current creator recordings, the ratified QUINE scenes and the corrected
statement highlights. No recording, trim choice or ratification lock was edited
by the assistant. All 494 recording and lock file hashes match the snapshot
taken immediately after the creator's replacement take.

| Cut | Delivered files | Measured runtime | Narration / captions |
|---|---|---|---|
| Main, 1920×1080 | [Clean master](out/long.mp4) · [Captioned video](out/long.captioned.mp4) | 319.598 s · 5:19.598 | 67 clips, 95 cues |
| Independent Short, 1080×1920 | [Captioned video](out/short.mp4) | 49.338 s · 0:49.338 | 13 clips, 13 cues |

All three videos are H.264 at 30 fps with AAC stereo sound at 48 kHz. Main
captions: [SRT](out/long.en.srt), [VTT](out/long.en.vtt); Short captions:
[SRT](out/short.en.srt), [VTT](out/short.en.vtt). The main's
[nine chapter timestamps](out/long.chapters.txt) come from this final render.
Platform upload folders contain [YouTube metadata](out/youtube/long/metadata.md)
and [Short metadata](out/youtube-shorts/short/metadata.md). The full tailored
copy package for all requested platforms follows below.

**Checks:** Native validation and media QA pass. The independent
[media receipt](build/recorded-review/media-checks.json) verifies delivered
hashes, runtimes, dimensions, codecs, captions, recorded audio paths, natural
spacing and the chapter text copied into the publishing package. The main
measures −14.2 LUFS with −1.9 dBTP true peak; the Short measures −14.1 LUFS and
−2.2 dBTP. Frames from the final files confirm readable captions and highlights
on the names, relation and roles. The formerly reported statement moments now
occur near 2:36, 2:42, 2:46 and 3:12 in the creator-voice main; the highlight
follows the same narration line despite the different recording duration.

**Retake and caption repair:** The initial
[speech check](build/recorded-review/speech-check.txt) passed 79 lines and found
that `v17a-representation-01` contained only “For that,” lasting 0.584 seconds.
Both stored raw takes were incomplete. The creator replaced the take with the
complete 9.289-second sentence; its [targeted recheck](build/recorded-review/retake-check.txt)
scores 0.99. The other 79 recordings are unchanged, so all 80 current lines
meet the 0.90 speech-check threshold. This automated comparison is separate
from the final human listen-through.

The initial main was withheld by native caption QA. In addition to the
incomplete sentence, four complete lines needed slightly more reading time.
Normal spacing between lines was increased from 0.28 to 0.44 seconds, retaining
0.75-second section spacing, the 0.6-second lead and 1.2-second tail. The new
renders pass the unchanged 20-character/second gate. No words, voice speed or
ratified scene content were changed. Captions follow the measured recordings
and new natural spacing. There are no response holds.

**Remaining audio review:** Vidni reports that the opening nine takes are
7.9 dB quieter than the main recording session. A measurement of the processed
narration finds a 4.31 dB difference between the median speech RMS of those
opening lines and the later lines. Their last line ends at 0:38.657. The
[level receipt](build/recorded-review/level-review.json) records the method and
per-line values; this is a level comparison, not a perceptual judgment. Listen
across the first-minute transition during the final watch-through. The takes
were preserved and the standard Vidni audio chain was used.

Course document checks, local links and whitespace checks pass. The publishing
copy screen finds no shared 12-word window with the local comparison corpus;
its one six-word match is the generic phrase “it does not establish that the”.
Index, prerequisites and coverage assignments are unchanged. Human learner
feedback, a physical-phone review and native release approval remain separate
from these production checks. The assistant did not upload or approve a
release. The creator subsequently reported the main lesson uploaded and
supplied the URL recorded in the publishing package below.


<a id="v17a-publishing-copy"></a>
## Publishing and sharing copy — 2026-11-19

Copy for the creator-narrated main lesson (5:19.598) and independent Short
(0:49.338). Main chapter times below are verified against the delivered final
chapter file after the replacement take. The creator reported the main lesson
uploaded on 2026-11-19 and supplied this
[published Lesson 2 URL](https://www.youtube.com/watch?v=i0nlnWrlFCw&list=PLAQKTXMK9CyY&index=1).
All full-lesson links below use that exact URL, including its playlist context.
This update prepares sharing copy; the assistant has not posted it or recorded
native release approval or publication. No Short publication has been reported.

### Main YouTube title

```text
Introducing Nibli KR: From Records to Reasoning | Neuro-Symbolic Lesson 2
```

### Main YouTube description

```text
Knowledge representation begins with choosing how to describe a situation so we can reason about it.

In Neuro-Symbolic Lesson 2, we return to a small shop preparing parcels. We separate four things: the physical parcel, the note about it, the statement written in Nibli KR, and the Nibli engine that evaluates claims.

We read one current KR statement after explaining its meaning in everyday language. Along the way, we distinguish supplying a premise from evaluating a claim, introduce Nibli's connection to Lojban, and apply the same distinction to a library return.

A formal statement can make a record easier to inspect and process. It does not establish that the record matches what we intended or that the original observation was correct.

This course builds from symbolic reasoning toward combining it with neural AI. No formal logic or programming background is required. The shop and library are fictional teaching examples.

Chapters
0:00 Clear records for the next shift
0:39 What knowledge representation means
1:06 The parcel and its record
1:30 Nibli KR and the Nibli engine
2:05 Reading one current KR statement
3:01 What formal structure can check
4:04 The Lojban connection
4:21 A returned library copy
4:52 Writing statements in Lesson 3

Nibli: https://github.com/dhilipsiva/nibli

#SymbolicReasoning #NeuroSymbolicAI #KnowledgeRepresentation
```

### YouTube Short title

```text
Nibli and Nibli KR: Two Different Jobs | Neuro-Symbolic Lesson 2
```

### YouTube Short description

```text
A note says Parcel A is packed. Writing that note formally still leaves the packing to be checked.

In this 49-second example, Nibli KR is the language used to write the record. Nibli is the engine that evaluates claims against supplied records and rules.

From Neuro-Symbolic Lesson 2. No logic or programming background is needed.
Full lesson: https://www.youtube.com/watch?v=i0nlnWrlFCw&list=PLAQKTXMK9CyY&index=1

#Shorts #SymbolicReasoning #NeuroSymbolicAI
```

### LinkedIn link post

```text
The next lesson in my Neuro-Symbolic Reasoning from First Principles course starts with a parcel and a note about it.

Before connecting neural AI with symbolic reasoning, we need to be clear about what our records represent. Lesson 2 separates the real situation, the record, the formal language and the engine that evaluates claims.

I introduce Nibli KR through one small example, explain its role alongside the Nibli engine, and show why a well-formed statement still needs the right meaning and evidence behind it.

The lesson assumes no formal logic or programming background.

Watch Lesson 2: https://www.youtube.com/watch?v=i0nlnWrlFCw&list=PLAQKTXMK9CyY&index=1

#NeuroSymbolicAI #SymbolicReasoning #KnowledgeRepresentation
```

### Facebook link post

```text
A parcel is on the table. A note says it is packed. The note and the parcel are two different things.

Lesson 2 of my Neuro-Symbolic Reasoning from First Principles course builds from that simple distinction. I introduce the language we use to write records and the engine that evaluates claims from them.

We start in an everyday shop and ease into one formal statement. No logic or coding background needed.

Watch: https://www.youtube.com/watch?v=i0nlnWrlFCw&list=PLAQKTXMK9CyY&index=1
```

### X.com link post

```text
A formal statement can be precise while the note behind it is wrong.

Neuro-Symbolic Lesson 2 introduces Nibli KR through a parcel, its record and the engine that evaluates claims.

Watch: https://www.youtube.com/watch?v=i0nlnWrlFCw&list=PLAQKTXMK9CyY&index=1
```

### Threads link post

```text
A parcel, a note about it, a formal statement and a reasoning engine have different roles.

In Neuro-Symbolic Lesson 2, I build that distinction from a small shop example. We read one Nibli KR statement and explain why formal structure still needs an accurate record behind it.

No formal logic or programming background needed.

Full lesson: https://www.youtube.com/watch?v=i0nlnWrlFCw&list=PLAQKTXMK9CyY&index=1
```

### Instagram link post

```text
The parcel. The note. The language. The reasoning engine.

Four parts of one everyday example, explained step by step in Neuro-Symbolic Lesson 2.

We introduce Nibli KR and see how a formal statement helps us reason from records—while the original observation still needs evidence.

Full lesson on YouTube: https://www.youtube.com/watch?v=i0nlnWrlFCw&list=PLAQKTXMK9CyY&index=1

#NeuroSymbolicAI #SymbolicReasoning #KnowledgeRepresentation
```

### WhatsApp Status caption

```text
A parcel, its record and the claims we can evaluate.
Neuro-Symbolic Lesson 2: https://www.youtube.com/watch?v=i0nlnWrlFCw&list=PLAQKTXMK9CyY&index=1
```

### Instagram Reels caption

```text
A note says the parcel is packed. Writing it formally doesn't check the parcel.

Nibli KR writes the record; Nibli evaluates claims. A 49-second introduction from Neuro-Symbolic Lesson 2.

Full lesson: https://www.youtube.com/watch?v=i0nlnWrlFCw&list=PLAQKTXMK9CyY&index=1

#NeuroSymbolicAI #SymbolicReasoning #KnowledgeRepresentation
```

## QUINE production recipe

The current source is the manifest plus this recipe. It generates original SVG
scenes under build/quine-assets/, with exact-word or label highlights keyed to
stable narration-line IDs. Extract and run the Python block with the project
folder as its only argument before drafting. Native Vidni framing and captions
use ../themes/quine. The Short uses its own 900×620 compositions inside the
platform-safe vertical view, rather than cropping the main lesson.

The creator edited the origin sentence to first person and removed the spoken
date before requesting these synthetic previews. That wording is preserved;
the exact dates stay in the source receipts. No audience holds are introduced.

<!-- V17A_SCENES_START -->
```json
{
  "v17a-shop-01": [
    "shop",
    "title",
    ""
  ],
  "v17a-shop-02": [
    "shop",
    "title",
    "symbolic"
  ],
  "v17a-shop-03": [
    "shop",
    "plain",
    ""
  ],
  "v17a-shop-04": [
    "shop",
    "plain",
    "A"
  ],
  "v17a-shop-05": [
    "shop",
    "rule",
    "own"
  ],
  "v17a-shop-06": [
    "shop",
    "rule",
    "ready"
  ],
  "v17a-shop-07": [
    "shop",
    "notes",
    "A"
  ],
  "v17a-shop-08": [
    "shop",
    "notes",
    "both"
  ],
  "v17a-shop-09": [
    "shop",
    "next",
    "next"
  ],
  "v17a-representation-01": [
    "representation",
    "",
    "all"
  ],
  "v17a-representation-02": [
    "representation",
    "",
    "2"
  ],
  "v17a-representation-03": [
    "representation",
    "",
    "term"
  ],
  "v17a-representation-04": [
    "representation",
    "",
    "purpose"
  ],
  "v17a-parcel-and-note-01": [
    "four",
    "1",
    "0"
  ],
  "v17a-parcel-and-note-02": [
    "four",
    "1",
    "0"
  ],
  "v17a-parcel-and-note-03": [
    "four",
    "2",
    "1"
  ],
  "v17a-parcel-and-note-04": [
    "four",
    "2",
    "1"
  ],
  "v17a-parcel-and-note-05": [
    "four",
    "2",
    "1"
  ],
  "v17a-parcel-and-note-06": [
    "four",
    "2",
    "1"
  ],
  "v17a-parcel-and-note-07": [
    "four",
    "2",
    "1"
  ],
  "v17a-language-and-engine-01": [
    "four",
    "3",
    "2"
  ],
  "v17a-language-and-engine-02": [
    "four",
    "3",
    "2"
  ],
  "v17a-language-and-engine-03": [
    "four",
    "3",
    "2"
  ],
  "v17a-language-and-engine-04": [
    "four",
    "4",
    "3"
  ],
  "v17a-language-and-engine-05": [
    "four",
    "4",
    "claim"
  ],
  "v17a-language-and-engine-06": [
    "four",
    "4",
    "2"
  ],
  "v17a-language-and-engine-07": [
    "four",
    "4",
    "3"
  ],
  "v17a-language-and-engine-08": [
    "four",
    "4",
    "physical"
  ],
  "v17a-language-and-engine-09": [
    "four",
    "4",
    "physical"
  ],
  "v17a-one-statement-01": [
    "four",
    "3",
    "1"
  ],
  "v17a-one-statement-02": [
    "list",
    "1",
    "list"
  ],
  "v17a-one-statement-03": [
    "list",
    "1",
    "member"
  ],
  "v17a-one-statement-04": [
    "list",
    "2",
    "relation"
  ],
  "v17a-one-statement-05": [
    "list",
    "2",
    "member"
  ],
  "v17a-one-statement-06": [
    "list",
    "2",
    "roles"
  ],
  "v17a-one-statement-07": [
    "statement",
    "",
    ""
  ],
  "v17a-one-statement-08": [
    "statement",
    "",
    "ParcelA"
  ],
  "v17a-one-statement-09": [
    "statement",
    "",
    "PackedParcels"
  ],
  "v17a-one-statement-10": [
    "statement",
    "",
    "names"
  ],
  "v17a-one-statement-11": [
    "statement",
    "",
    "vocabulary"
  ],
  "v17a-one-statement-12": [
    "statement",
    "",
    "relation"
  ],
  "v17a-one-statement-13": [
    "statement",
    "",
    "PackedParcels"
  ],
  "v17a-checking-and-limits-01": [
    "statement",
    "",
    "member"
  ],
  "v17a-checking-and-limits-02": [
    "statement",
    "",
    "names"
  ],
  "v17a-checking-and-limits-03": [
    "statement",
    "",
    "vocabulary"
  ],
  "v17a-checking-and-limits-04": [
    "premise",
    "",
    "premise"
  ],
  "v17a-checking-and-limits-05": [
    "premise",
    "",
    "claim"
  ],
  "v17a-checking-and-limits-06": [
    "premise",
    "",
    "claim"
  ],
  "v17a-checking-and-limits-07": [
    "checks",
    "both",
    ""
  ],
  "v17a-checking-and-limits-08": [
    "checks",
    "name",
    "mismatch"
  ],
  "v17a-checking-and-limits-09": [
    "checks",
    "name",
    "encoding"
  ],
  "v17a-checking-and-limits-10": [
    "checks",
    "observation",
    "note"
  ],
  "v17a-checking-and-limits-11": [
    "checks",
    "both",
    "encoding"
  ],
  "v17a-checking-and-limits-12": [
    "checks",
    "observation",
    "evidence"
  ],
  "v17a-checking-and-limits-13": [
    "checks",
    "observation",
    "parcel"
  ],
  "v17a-origins-01": [
    "history",
    "",
    "lojban"
  ],
  "v17a-origins-02": [
    "history",
    "",
    "kr"
  ],
  "v17a-origins-03": [
    "history",
    "",
    "current"
  ],
  "v17a-library-01": [
    "library",
    "1",
    "0"
  ],
  "v17a-library-02": [
    "library",
    "1",
    "0"
  ],
  "v17a-library-03": [
    "library",
    "2",
    "1"
  ],
  "v17a-library-04": [
    "library",
    "4",
    "3"
  ],
  "v17a-library-05": [
    "library",
    "4",
    "all"
  ],
  "v17a-library-06": [
    "library",
    "4",
    "1"
  ],
  "v17a-next-01": [
    "four",
    "4",
    "all"
  ],
  "v17a-next-02": [
    "next",
    "",
    ""
  ],
  "v17a-next-03": [
    "next",
    "",
    ""
  ],
  "v17a-short-shop-01": [
    "short",
    "shop",
    ""
  ],
  "v17a-short-shop-02": [
    "short",
    "shop",
    "parcel"
  ],
  "v17a-short-shop-03": [
    "short",
    "shop",
    "note"
  ],
  "v17a-short-language-01": [
    "short",
    "language",
    "kr"
  ],
  "v17a-short-language-02": [
    "short",
    "language",
    "meaning"
  ],
  "v17a-short-language-03": [
    "short",
    "language",
    "list"
  ],
  "v17a-short-language-04": [
    "short",
    "language",
    "reading"
  ],
  "v17a-short-language-05": [
    "short",
    "code",
    ""
  ],
  "v17a-short-engine-01": [
    "short",
    "engine",
    "language"
  ],
  "v17a-short-engine-02": [
    "short",
    "engine",
    "engine"
  ],
  "v17a-short-boundary-01": [
    "short",
    "boundary",
    "parcel"
  ],
  "v17a-short-boundary-02": [
    "short",
    "boundary",
    "note"
  ],
  "v17a-short-boundary-03": [
    "short",
    "boundary",
    "evidence"
  ]
}
```
<!-- V17A_SCENES_END -->

<!-- V17A_VISUAL_RECIPE_START -->
```python
from pathlib import Path
from html import escape
import json, sys

BG='#0B0B10'; CARD='#181822'; INK='#F4F3EF'; BODY='#C9C8C2'
MUTED='#908F9A'; EDGE='#3A3A4A'; EMBER='#F2542D'; VIOLET='#8B7CF6'
project=Path(sys.argv[1]).resolve()
out=project/'build/quine-assets'
out.mkdir(parents=True,exist_ok=True)

def text(x,y,s,size=38,fill=INK,family='IBM Plex Sans',anchor='start'):
    return f'<text x="{x}" y="{y}" font-family="{family}" font-size="{size}" font-weight="700" fill="{fill}" text-anchor="{anchor}">{escape(s)}</text>'
def rect(x,y,w,h,fill=CARD,stroke=EDGE,radius=8):
    return f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{radius}" fill="{fill}" stroke="{stroke}" stroke-width="2"/>'
def hi(x,y,w,h):
    return f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="6" fill="{EMBER}" fill-opacity=".09" stroke="{EMBER}" stroke-width="4"/>'
def line(x1,y1,x2,y2,c=EDGE):
    return f'<path d="M{x1} {y1} L{x2} {y2}" stroke="{c}" stroke-width="3" fill="none"/>'
def arrow(x1,y1,x2,y2):
    return line(x1,y1,x2,y2,MUTED)+f'<path d="M{x2-13} {y2-8} L{x2} {y2} L{x2-13} {y2+8}" fill="none" stroke="{MUTED}" stroke-width="3"/>'
def heading(kicker,title,small=False):
    x=48 if small else 76
    return text(x,46 if small else 58,'// '+kicker.upper(),20 if small else 22,EMBER,'IBM Plex Mono')+text(x,106 if small else 135,title,42 if small else 62,INK,'Space Grotesk')
def foot(s,small=False):
    return text(48 if small else 76,590 if small else 706,s,26 if small else 29,BODY)
def parcel(x,y,label='A',scale=1,focus=False):
    b=f'<g transform="translate({x} {y}) scale({scale})">'
    b+=f'<path d="M0 70 L175 15 L310 78 L130 140 Z" fill="#908F9A" stroke="#C9C8C2" stroke-width="2"/>'
    b+=f'<path d="M0 70 L130 140 L130 310 L0 240 Z" fill="#2A2A38" stroke="#908F9A" stroke-width="2"/>'
    b+=f'<path d="M130 140 L310 78 L310 245 L130 310 Z" fill="#3A3A4A" stroke="#908F9A" stroke-width="2"/>'
    b+=f'<path d="M84 44 L118 33 L251 102 L217 114 Z" fill="#FF9A6F"/>'
    b+=f'<path d="M217 114 L251 102 L251 160 L217 171 Z" fill="{EMBER}"/>'
    b+=text(155,365,'Parcel '+label,42,INK,'IBM Plex Mono','middle')
    if focus:b+=hi(224,324,44,57)
    return b+'</g>'
def note(x,y,w=620,label='A',message='packed',focus=''):
    b=rect(x,y,w,165)+text(x+28,y+54,'Parcel '+label,34,INK,'IBM Plex Mono')+text(x+28,y+121,message,38,BODY)
    if focus=='name':b+=hi(x+22,y+18,190,49)
    if focus=='message':b+=hi(x+22,y+79,min(w-44,len(message)*22+15),58)
    return b
def box(x,y,w,h,title,lines=(),focus=False,size=34):
    b=rect(x,y,w,h,stroke=EMBER if focus else EDGE)+text(x+28,y+52,title,size,EMBER if focus else INK,'Space Grotesk')
    for i,s in enumerate(lines):b+=text(x+28,y+100+i*44,s,30,BODY)
    if focus:b+=hi(x+14,y+max(4,44-size*.8),w-28,size+8)
    return b
def code(x=150,y=340,size=62,focus='',wrap=False):
    parts=[('member',EMBER),('(',MUTED),('ParcelA',INK),(', ',MUTED),('PackedParcels',INK),(').',MUTED)]
    b='';cur=x
    for token,color in parts:
        if wrap and token=='PackedParcels':cur=x+size*.6*2;y+=size*1.4
        width=len(token)*size*.6
        if token==focus or (focus=='names' and token in ('ParcelA','PackedParcels')):b+=hi(cur-6,y-size-4,width+12,size+18)
        b+=text(cur,y,token,size,color,'IBM Plex Mono');cur+=width
    return b
def shop(mode,focus):
    b=heading('A small shop','Clear records for the next shift')
    if mode=='title':
        b=heading('Neuro-Symbolic Lesson 2','Introducing Nibli KR')+text(80,212,'Its Purpose and Origins',42,BODY,'Space Grotesk')
        b+=parcel(170,290,'A',.8)+note(650,340,760)+foot('Write down information. Evaluate the claims it supports.')
        if focus=='symbolic':b+=hi(190,26,124,42)
        return b
    b+=parcel(180,270,'A',.85,focus=='A')+parcel(585,270,'B',.85,focus=='B')
    b+=line(90,623,1050,623)+text(110,658,'PACKING TABLE',22,MUTED,'IBM Plex Mono')
    if mode in ('rule','notes','next'):
        b+=box(1100,355,390,180,'Collection area',['Own parcel,','both checks.'],focus=='ready')
        b+=rect(80,177,1440,96)+text(108,214,'READY FOR COLLECTION',22,EMBER,'IBM Plex Mono')
        b+=text(108,251,'Packed',33)+text(285,251,'+',33,MUTED)+text(340,251,'own address label attached',33)
        if focus=='rule':b+=hi(96,221,930,43)
        if focus=='own':b+=hi(328,219,86,47)
    if mode=='notes':
        b=heading('Read the records','The notes concern different parcels')
        b+=parcel(100,250,'A',.75,focus in ('A','both'))+parcel(1100,250,'B',.75,focus in ('B','both'))
        b+=note(455,225,570,'A','packed','name' if focus in ('A','both') else '')
        b+=note(455,435,570,'B','label attached','name' if focus in ('B','both') else '')
        b+=foot('A packing note about A. A label note about B.')
    if mode=='next':
        b+=foot('Clear records can be passed to the next shift.')
        if focus=='next':b+=hi(70,666,940,55)
    return b
def representation(focus):
    b=heading('Choose what matters','Represent information for a purpose')
    data=[('Named parcels',['Parcel A','Parcel B']),('Recorded checks',['Packing confirmation','Label confirmation']),('Same-parcel rule',['Both checks concern','the parcel being moved.'])]
    for i,(title,ls) in enumerate(data):b+=box(80+i*490,220,460,240,title,ls,focus in (str(i),'all'),34)
    if focus in ('term','purpose'):
        b+=text(80,568,'Knowledge representation',54,INK,'Space Grotesk')
        if focus=='term':b+=hi(69,510,820,79)
    b+=foot('Purpose: evaluate the claim that this parcel is ready.')
    if focus=='purpose':b+=hi(67,665,1190,57)
    return b
def four(stage,focus,library=False):
    b=heading('One situation, different parts','From a record to an evaluation' if not library else 'The same idea at a library')
    xs=[80,840,80,840];ys=[185,185,425,425]
    titles=['Parcel' if not library else 'Book copy','Note','Nibli KR · language' if not library else 'Formal representation','Nibli · engine' if not library else 'Engine evaluation']
    for i in range(min(int(stage),4)):
        x,y=xs[i],ys[i];b+=rect(x,y,680,210)+text(x+28,y+47,titles[i],34,EMBER if str(i)==focus else INK,'Space Grotesk')
        if i==0:
            if library:b+=rect(x+34,y+77,82,110,'#3A3A4A')+line(x+49,y+80,x+49,y+183,VIOLET)+text(x+157,y+124,'Copy 7',42,INK,'IBM Plex Mono')+text(x+157,y+172,'on the returns desk',29,BODY)
            else:b+=parcel(x+34,y+53,'A',.35)+text(x+208,y+126,'Physical object',38)+text(x+208,y+176,'on the packing table',29,BODY)
        elif i==1:b+=text(x+32,y+123,'“Copy 7 was returned.”' if library else '“Parcel A is packed.”',38)+text(x+32,y+173,'A record about this copy' if library else 'A record about this parcel',28,BODY)
        elif i==2:b+=text(x+32,y+120,'Write the record' if library else 'Write statements and rules',36)+text(x+32,y+172,'KR = knowledge representation' if not library else 'The record in a formal language',26,BODY)
        else:b+=text(x+32,y+120,'Evaluate claims',38)+text(x+32,y+172,'Using supplied records + rules',28,BODY)
        if str(i)==focus:b+=hi(x+16,y+15,648,51)
    if focus=='claim':b+=foot('Claim: Parcel A meets the shop’s checklist.')+hi(68,665,1080,55)
    elif focus=='physical':b+=hi(95,199,650,184)+foot('The physical parcel still needs its own evidence.')
    elif focus=='all':b+=foot('Object → record → formal representation → evaluation')
    elif library:b+=foot('The record needs to match the return of Copy 7.')
    return b
def listing(stage,focus):
    b=heading('Represent the packing note','Parcel A is on the packed-parcels list')
    b+=parcel(180,220,'A',.75,focus=='parcel')+box(720,210,760,260,'Packed-parcels list',['Our record of','packing confirmations.'],focus=='list',39)
    b+=rect(751,391,660,56,BG,EDGE)+text(774,432,'Parcel A',33,INK,'IBM Plex Mono')
    if focus=='member':b+=hi(741,385,680,68)
    if int(stage)>1:
        b+=rect(80,545,1400,105)+text(110,585,'SUPPORTED RELATION',22,EMBER,'IBM Plex Mono')+text(110,628,'member',32,INK,'IBM Plex Mono')
        b+=text(430,626,'member · item',32)+text(950,626,'set · group',32)
        if focus=='relation':b+=hi(99,591,197,49)
        if focus=='roles':b+=hi(420,585,350,58)+hi(935,585,300,58)
    return b
def statement(focus):
    b=heading('One current KR statement','The same packing record, written formally')
    code_focus='member' if focus in ('vocabulary','relation') else focus
    b+=rect(80,205,1440,210)+code(140,332,62,code_focus)
    # Keep each emphasis attached to the text it names. Mono labels use the
    # same 0.6-em advance as the statement; no footer offsets.
    def label(x,y,value,size,active=False):
        mark=hi(x-6,y-size-4,len(value)*size*.6+12,size+18) if active else ''
        return mark+text(x,y,value,size,EMBER if active else INK,'IBM Plex Mono')
    cards=[(80,430,'member','item belongs to group'),
           (555,430,'ParcelA','This parcel’s name'),
           (1030,490,'PackedParcels','Packing confirmations')]
    for x,w,title,meaning in cards:
        active=(title==focus or (focus=='names' and title!='member') or
                (title=='member' and focus in ('vocabulary','relation')))
        b+=rect(x,470,w,170)+label(x+28,522,title,38,active)
        b+=text(x+28,570,meaning,30,BODY)
    # These are the same two roles introduced just before this statement.
    b+=label(108,614,'member · item',22,focus=='vocabulary')
    b+=label(302,614,'set · group',22,focus=='vocabulary')
    return b+foot('Supported relation. Names chosen for this example.')
def premise(focus):
    b=heading('Two distinct steps','Supply a premise. Evaluate a claim.')
    b+=box(80,230,665,300,'Supply a premise',['Information taken as given','for this reasoning.'],focus=='premise',42)
    b+=box(855,230,665,300,'Evaluate a claim',['Check its support against','the supplied premises + rules.'],focus=='claim',42)
    return b+foot('A claim being evaluated is not a new premise.')
def checks(mode,focus):
    b=heading('Check what the record means','The record and the parcel still matter')
    if mode=='name':
        b+=box(80,225,650,260,'Original note',['Names Parcel A'],focus=='note',42)+box(870,225,650,260,'Encoding',['Names Parcel B'],focus=='encoding',42)+arrow(756,358,839,358)
        b+=text(340,434,'A',68,INK,'IBM Plex Mono')+text(1128,434,'B',68,INK,'IBM Plex Mono')
        if focus=='mismatch':b+=hi(323,365,85,87)+hi(1111,365,85,87)
        b+=foot('Well-formed text can describe the wrong parcel.')
    elif mode=='observation':
        b+=note(90,270,680,'A','packed','message' if focus=='note' else '')+parcel(1030,200,'A',.95,focus=='parcel')
        b+=foot('The packing confirmation still needs evidence.')
        if focus=='evidence':b+=hi(75,665,1130,57)
    else:
        b+=box(80,230,650,270,'Check the encoding',['Match the note’s','intended meaning.'],focus=='encoding',42)
        b+=box(870,230,650,270,'Check the observation',['Evidence from the shop','supports the packing record.'],focus=='evidence',42)
        b+=foot('Structure helps checking. Evidence supports the observation.')
    return b
def history(focus):
    b=heading('Purpose and origins','From Lojban to current Nibli KR')
    b+=box(80,235,620,240,'Lojban',['Original input','A designed human language'],focus=='lojban',54)+arrow(725,352,840,352)
    b+=box(870,235,650,240,'Nibli KR',['Current input','Vocabulary retains its origins'],focus=='kr',54)+foot('The lesson teaches the current language.')
    return statement('member') if focus=='current' else b
def next_lesson():
    b=heading('Next · Neuro-Symbolic Lesson 3','Writing Your First Nibli KR Statements')+parcel(160,240,'A',.8)
    b+=box(650,230,830,170,'Write the parcel notes',['Read each part of a statement.'],True,42)
    return b+box(650,435,400,175,'Premise',['Supply a record.'],False,38)+box(1080,435,400,175,'Claim',['Evaluate support.'],False,38)
def short_scene(mode,focus):
    b=heading('Neuro-Symbolic Lesson 2','A parcel and its record',True)
    if mode in ('shop','boundary'):
        b+=parcel(85,170,'A',.65,focus=='parcel')
        if mode=='boundary' or focus=='note':
            b+=rect(390,207,448,220)+text(418,266,'THE NOTE',24,EMBER,'IBM Plex Mono')+text(418,325,'Parcel A',39)+text(418,377,'is packed.',39)
            if focus=='note':b+=hi(405,282,401,111)
        if mode=='boundary':
            b+=foot('The packing still needs evidence.',True)
            if focus=='evidence':b+=hi(36,549,816,57)
        return b
    if mode=='language':
        b=heading('Write the record','Nibli KR · language',True)
        if focus=='kr':b+=hi(35,59,800,62)
        b+=box(48,160,792,160,'Knowledge representation',['KR names the language’s purpose.'],focus=='meaning',36)
        if focus in ('list','reading'):
            b+=box(48,356,792,165,'Packed-parcels list',['Our record of packing confirmations.'],focus=='list',35)
            if focus=='reading':b+=rect(48,356,792,165)+text(75,415,'Parcel A is on the',39)+text(75,468,'packed-parcels list.',39)+hi(62,373,760,118)
        return b
    if mode=='code':
        b=heading('The same record','Written in Nibli KR',True)+rect(48,184,792,240)+code(88,278,38,'',True)+hi(70,215,744,150)
        return b+text(48,493,'Parcel A is on the',33,BODY)+text(48,538,'packed-parcels list.',33,BODY)
    b=heading('Language and engine','Two different jobs',True)
    return b+box(48,170,792,155,'Nibli KR · language',['Write the record.'],focus=='language',37)+box(48,360,792,155,'Nibli · engine',['Evaluate claims from records + rules.'],focus=='engine',37)
def draw(spec):
    family,stage,focus=spec
    if family=='shop':return shop(stage,focus)
    if family=='representation':return representation(focus)
    if family=='four':return four(stage,focus)
    if family=='list':return listing(stage,focus)
    if family=='statement':return statement(focus)
    if family=='premise':return premise(focus)
    if family=='checks':return checks(stage,focus)
    if family=='history':return history(focus)
    if family=='library':return four(stage,focus,True)
    if family=='next':return next_lesson()
    if family=='short':return short_scene(stage,focus)
    raise ValueError(spec)

notes=(project/'notes.md').read_text()
fence=chr(96)*3
specs=json.loads(notes.split('<!-- V17A_SCENES_START -->')[1].split('<!-- V17A_SCENES_END -->')[0].split(fence+'json')[1].split(fence)[0])
for name,spec in specs.items():
    w,h=(900,620) if spec[0]=='short' else (1600,760)
    svg=f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">'+rect(0,0,w,h,BG,BG,0)+draw(spec)+'</svg>'
    (out/(name+'.svg')).write_text(svg)
print(f'Generated {len(specs)} QUINE scenes in {out}')
```
<!-- V17A_VISUAL_RECIPE_END -->
