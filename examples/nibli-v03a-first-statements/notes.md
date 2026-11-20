# Neuro-Symbolic Lesson 3 — Writing Your First Nibli KR Statements

Main lesson and independent Short · recorded-voice final exports · 2026-11-19.

**Status:** The creator has ratified both cuts and recorded all 111 current
lines. The local speech check reports 111 heard as scripted, with no missing,
silent or flagged takes; the saved recordings have no clipped samples.
The recorded timelines measure **7:13.604** for the main and **0:48.202** for
the Short. The [full master](out/long.mp4), [captioned main](out/long.captioned.mp4)
and [captioned vertical Short](out/short.mp4) are complete and pass final media
checks; see the [final production record](#recorded-final).
Both cuts use QUINE, the full-playground camera views and measured highlights
with three-second holds while the target remains visible. The owner explicitly
confirmed the reviewed videos’ cues; all 93 confirmations are now saved against
those exact files. Beginner feedback remains pending. The earlier
[recording review](#recorded-voice-review) is retained as provenance.

The current main stays with the parcel records and ends with the shop’s
dispatch checklist and Lesson 4’s rule, claim and supporting records in plain
English. The [parcel transition revision](#parcel-transition), earlier synthetic
reviews and all recordings are retained. The [publishing copy](#publishing-copy)
is confirmed against the final exported timeline and chapter file.
The creator reported the [main lesson published](https://www.youtube.com/watch?v=ceEup-8bjyk&list=PLAQKTXMK9CyY)
on 2026-11-20. Sharing copy now uses that exact URL and playlist context.

The internal episode ID is **V03a**; the viewer-facing label is
**Neuro-Symbolic Lesson 3**. The lesson follows the
[published Lesson 2](https://www.youtube.com/watch?v=i0nlnWrlFCw&list=PLAQKTXMK9CyY&index=1).
Its one goal is to write down what a packing note says, then evaluate a claim
against that record. It assumes no logic or programming background.

- [Complete main narration](#complete-main-narration)
- [Complete Short narration](#complete-short-narration)
- [Main storyboard](#main-storyboard)
- [Short storyboard](#short-storyboard)
- [Statement reference](#statement-reference)
- [Public source receipts](#public-source-receipts)
- [Verification and production handoff](#verification-and-production-handoff)
- [Recorded-voice review and export handoff](#recorded-voice-review)
- [Owner confirmation and final production](#recorded-final)
- [Publishing and sharing copy](#publishing-copy)
- [Parcel-only ending and Lesson 4 handoff](#parcel-transition)
- [Readable highlights and predicate terminology](#readable-highlights)
- [Full-playground camera revision](#full-playground-zoom)
- [Measured highlights and cue review](#measured-highlights)
- [Continuous synthetic review](#continuous-synthetic-review)
- [First synthetic production record](#synthetic-production-record)
- [Reproducible capture and scene recipes](#production-recipes)

## Complete main narration

Read the paragraphs as connected speech. Anchors identify passages for scene
planning, not individual recording lines. Code fences are displayed text;
the surrounding narration explains their meaning without dictating punctuation
character by character. No audience task or response pause is inserted.

<!-- V03A_SCRIPT_START -->
<a id="v03a-shop"></a>

At our small shop, two parcels are waiting on the packing table. One note says
Parcel A is packed. Another says Parcel B has a label. The next person needs
to use those records without confusing one parcel with the other.

Welcome to Neuro-Symbolic Lesson 3. We're learning symbolic reasoning by making
the information we rely on explicit. Last time, we separated the parcel, its
record, Nibli KR as the language, and Nibli as the engine. Now we'll write the
records ourselves and evaluate claims against them.

For this example, a parcel with a packing note goes on our packed-parcels list.
So the first note becomes: Parcel A is on the packed-parcels list. That gives
us two things to name and a relationship between them.

<a id="v03a-build-statement"></a>

We name the parcel Parcel A, written as one word with a capital P. We name the
list Packed Parcels, also written as one word. These are names we've chosen.
The list's name doesn't tell the engine that anything has been packed; we
still need to state which parcel belongs to it.

The predicate member expresses membership. Its vocabulary entry gives it two
places: the member, then the set it belongs to. A set is the collection we're
calling a list here. The dictionary word is fixed, while the names of our
parcel and list are ours to choose.

Here is the whole statement.

```nibli-kr
member(ParcelA, PackedParcels).
```

The lowercase word member is the predicate name. The parentheses hold its
arguments: the things taking part in that relation. Parcel A occupies the
first place; Packed Parcels occupies the second. The comma separates them,
and the period finishes the statement. Read together, it says that Parcel A
is a member of our packed-parcels list.

<a id="v03a-named-places"></a>

Those positions matter. If we put the list first and the parcel second, we
have written a different statement: that the list belongs to the parcel.
The spelling can be accepted even though it doesn't express our note.

```nibli-kr
member(PackedParcels, ParcelA).
```

Labels make the intended roles visible. Here is our original statement with
each place named.

```nibli-kr
member(member: ParcelA, set: PackedParcels).
```

The word member appears twice. Before the parentheses, member is the predicate name.
Inside them, member followed by a colon labels the first place. The label set
identifies the other place. These are called named arguments.

Now we can write the labelled pair in the other order without changing the
claim. Each name still has its own label.

```nibli-kr
member(set: PackedParcels, member: ParcelA).
```

The distinction is between moving a labelled pair and moving an unlabelled
name. Labels carry the roles with them; without labels, the positions assign
the roles. We'll keep the shorter positional form for our two shop records.

<a id="v03a-supply-premises"></a>

The second note follows the same pattern: Parcel B is on our labelled-parcels
list. Here are both records together.

```nibli-kr
member(ParcelA, PackedParcels).
member(ParcelB, LabeledParcels).
```

In the playground, we choose Custom and enter these lines in the Nibli KR
pane. This pane holds the knowledge base: the information we're supplying
for reasoning. We are asserting these statements as premises, meaning the
starting information the engine may rely on.

Before using them, we open the back-translation and compare its reading with
our notes. A must be on the packed list; B must be on the labelled list.
Then we select Use reviewed encoding. An edit requires another review,
because approval of one version doesn't establish the meaning of a changed
version. We're using the playground's default reasoning settings.

This review checks how we've written the records. The packing and labelling
observations still need their own evidence from the shop.
Client Name 1:

Date of Birth (DOB):

Client Name 2:

Date of Birth (DOB):

Unit Number:

Wedding Anniversary Date:
<a id="v03a-evaluate-claims"></a>

Now we use the separate claim field. We enter the statement that Parcel A
belongs to the packed-parcels list and run the evaluation.

```nibli-kr
member(ParcelA, PackedParcels).
```

The verdict is TRUE. Its support is the packing premise we supplied. No rule
is needed for this result: that statement is already among our records.

Running the same evaluation again gives the same support. The two lines in
the knowledge-base pane stay exactly as they were. Evaluating a claim does
not add it to the premises.

We change just the list name in the claim field, so the claim now concerns
Parcel A's label record.

```nibli-kr
member(ParcelA, LabeledParcels).
```

The verdict is FALSE. Here that means the claim cannot be derived from the
records we've supplied. Our label note concerns B, not A. This result does
not establish that the physical parcel has no label. An answer may print aClient Name 1:

Date of Birth (DOB):

Client Name 2:

Date of Birth (DOB):

Unit Number:

Wedding Anniversary Date:
name in lowercase; it still refers to the same named thing.

<a id="v03a-supplied-conclusion"></a>

There is an easy way to get a misleading success: put the desired claim into
the premises. In a separate copy of our example, we add a line saying that
Parcel A belongs to the labelled-parcels list, review that change, and
evaluate it again. Now it is TRUE because we supplied that very statement.

That hasn't produced a missing observation. It has changed the starting
information. We return to the original two records. The premise pane andClient Name 1:

Date of Birth (DOB):

Client Name 2:

Date of Birth (DOB):

Unit Number:

Wedding Anniversary Date:
claim field have different jobs even when we put identical words in them.

<a id="v03a-refusals"></a>

Some mistakes stop the input from being used at all. Leaving off the final
period leaves the statement unfinished. Restoring it repairs the syntax;
it doesn't change the packing note's meaning.

Another tempting spelling is packed followed by Parcel A. Packed is ordinary English, but it isn't a predicate in the supported vocabulary for this build.
The engine refuses that word. We use the supported member predicate to express our chosen list representation instead.

And member has two places. Adding a third argument is refused too. The message identifies the predicate and its two-place limit. We remove the extra
argument to recover our original statement.

These are input refusals, not FALSE verdicts. In the playground, a bad line
prevents the whole candidate knowledge base from being used. We keep each
mistake in a separate demonstration and restore the valid records afterward.

<a id="v03a-host-and-next"></a>

There is one useful interface distinction on this reference card. In the host
program, the leading evaluation marker shown here sends a statement for
evaluation. Without that prefix, the line supplies a premise. In the
playground, the separate input areas make that choice; we don't type the
host prefix into the claim field. The period remains in both forms.

An evaluation can also stop without deciding; we'll explain those outcomes
later. Today, we've written records with clear names and roles, supplied
premises, and evaluated claims against them.

The shop's dispatch checklist needs two confirmations for the same parcel: one for packing and one for its label.

In Neuro-Symbolic Lesson 4, we'll write that requirement as a rule and evaluate the claim that Parcel A meets the checklist.

Then we'll follow the rule and its supporting records to see how that conclusion is derived, without supplying the conclusion as a premise.

<!-- V03A_SCRIPT_END -->

## Complete Short narration

The Short stands alone. Its display fences are not additional spoken words.
It introduces one parcel and one note before showing an input area or verdict.

<!-- V03A_SHORT_SCRIPT_START -->
<a id="v03a-short-note"></a>

A note at our shop says Parcel A is packed. We represent that by putting
Parcel A on a packed-parcels list.

<a id="v03a-short-premise"></a>

Here is the statement in Nibli KR. Member connects the parcel to the list.
Parcel A fills the first place, and the packed-parcels list fills the second.

```nibli-kr
member(ParcelA, PackedParcels).
```

Placed in the knowledge-base pane and reviewed, it becomes a premise: a
record we're supplying.

<a id="v03a-short-claim"></a>

The same statement in the claim field is evaluated against our records.
Nibli returns TRUE because we supplied that premise. Evaluating it adds
nothing to the record.

<a id="v03a-short-boundary"></a>

The words are identical; the input area determines their job. And neither
input checks the physical parcel. This is Neuro-Symbolic Lesson 3: writing
statements and evaluating claims.

<!-- V03A_SHORT_SCRIPT_END -->

## Main storyboard

Use [QUINE](../themes/quine/theme.toml): near-black `#0b0b10`, light ink
`#f4f3ef`, ember emphasis `#f2542d`, Space Grotesk titles, IBM Plex Sans text
and IBM Plex Mono notation. Use labelled outlines or underlines as well as
color. Never put a highlight in a footer when the spoken target is in the
statement above it. Remove the previous emphasis as the narration moves.

The landscape layout reserves the lower caption area. Keep the parcel/list
context beside enlarged notation. Playground demonstrations use complete, fixed-layout
captures with cameras that move into the narrated input, control or result.
Establish the full workspace at entry and return wider for comparisons. Fit long named
forms across two lines instead of shrinking them. No unexplained diagram,
code editor chrome or coverage table opens the lesson. The times below are
planning estimates, not measured chapters.

| Scene / passage | Composition and visible content | Narration trigger → exact highlight | Approximate allocation |
|---|---|---|---|
| M01 · [Shop](#v03a-shop) | Packing table, parcels A and B, then the two notes. Small series label: “Neuro-Symbolic Lesson 3”. Reveal lesson title after the situation is established. | “Parcel A is packed” → A and its packing note; “Parcel B has a label” → B and its label note. | 20–25 s |
| M02 · [Purpose](#v03a-shop) | Keep the scene; add “Nibli KR · language” and “Nibli · engine” only on recall. Introduce “Packed-parcels list · parcels with packing notes” and the English membership sentence. | “language” / “engine” → their labels; “Parcel A” / “packed-parcels list” → the two English participants. | 28–34 s |
| M03 · [Names and relation](#v03a-build-statement) | Parcel and list remain visible. Reveal `ParcelA`, then `PackedParcels`; show a compact, accurately attributed vocabulary card headed “Signature · predicate and places”: `member(member, set)`, with plain-English captions “thing belonging” and “collection”. | “capital P” → the P of `ParcelA`; “names we've chosen” → the two names; “member, then the set” → the two dictionary places in order. | 36–43 s |
| M04 · [Complete statement](#v03a-build-statement) | Reveal the complete positional statement after “Here is the whole statement”. Animate highlights, not a simulated typing delay. | “lowercase word member” → predicate token; “parentheses” → both brackets; “first place” → `ParcelA`; “second” → `PackedParcels`; “comma” → `,`; “period” → `.`. | 28–34 s |
| M05 · [Wrong positional order](#v03a-named-places) | Two rows labelled “Our note” and “Different statement”, with the two names reversed in the second row. Arrow labels show how the roles changed. This is a meaning comparison, not a syntax refusal. | “list first” → `PackedParcels` in the first slot; “parcel second” → `ParcelA` in the second. | 16–21 s |
| M06 · [Named roles](#v03a-named-places) | Show the correct labelled statement. Keep the predicate token visually separate from `member:` inside the brackets. Reorder the two complete label/name pairs on the next sentence. | “Before the parentheses” → outer `member`; “inside them” → `member:`; “set” → `set:`; “each name still has its own label” → label/name pairs together. | 42–50 s |
| M07 · [Two premises](#v03a-supply-premises) | Return to the original two notes, then the two statements. Full playground → measured zoom into the Nibli KR pane, with native tabs and controls visible. Zoom into the actual default-profile area once: “Saturation on · existential import off · depth 10”. | “second note” → B's note and second line; “knowledge base” → the pane; “premises” → its label. | 26–32 s |
| M08 · [Review](#v03a-supply-premises) | Camera follows the actual Back-translation area → readings → Use reviewed encoding button. Show the edit/review requirement without a separate technical detour. Keep the playground’s panel positions stable. | “A must be” → A's reading and packing note; “B must be” → B's reading and label note; “Use reviewed encoding” → actual button. | 28–35 s |
| M09 · [TRUE and repeated evaluation](#v03a-evaluate-claims) | Establish the complete workspace, then zoom to its actual lower-left claim field, lower-right verdict and upper-right given support. Show repeated evaluation, then move back to the unchanged premise pane. No invented engine transcript. | “separate claim field” → field; “TRUE” → verdict; “premise we supplied” → support then original line; “stay exactly as they were” → both premise lines. | 32–40 s |
| M10 · [Missing support](#v03a-evaluate-claims) | Zoom into the claim’s change from `PackedParcels` to `LabeledParcels`, then move to a close view of the actual FALSE verdict. Return to the two input areas for the B/A comparison. Keep attention on the inputs and verdict; do not zoom into the complex FALSE proof or introduce internal witness names. | “just the list name” → changed token; “FALSE” → verdict; “B, not A” → B's supplied label record and A's claim. Brief case note: “A printed lowercase name still names the same thing.” | 27–34 s |
| M11 · [Supplying the desired claim](#v03a-supplied-conclusion) | Use the separate comparison state: zoom to the added A label premise, then to TRUE and its given support. Pull back to the complete workspace when restoring the original two-line pane and its reviewed state. | “add a line” → added premise; “supplied that very statement” → given support and added line; “original two records” → restored two lines. | 35–42 s |
| M12 · [Refusals and repairs](#v03a-refusals) | Separate fresh inputs: missing period; `packed(ParcelA).`; `member(ParcelA, PackedParcels, ParcelB).`. Retain each actual refusal message in full at readable size, using a camera that includes the complete message and admission status. Repair to the valid positional statement. Browser captures preserve the actual bundled-playground wording; see the production record. | “period” → missing terminator then restored `.`; “isn't a predicate” → `packed` and its refusal; “third argument” → final `ParcelB` and the arity reason; “whole candidate” → admission failure, without a FALSE badge. | 52–64 s |
| M14 · [Host reference and parcel handoff](#v03a-host-and-next) | Host reference card: `member(ParcelA, PackedParcels).` under “Supply a premise”; `? member(ParcelA, PackedParcels).` under “Evaluate a claim”. Mark this as an alternative interface, not a browser instruction. Footer reference-strip terms: “commands · query interface · keywords · predication · tags (later)”. Return to Parcel A and the shop’s dispatch checklist: packing and label confirmations must concern the same parcel. Introduce the next lesson’s rule, claim and supporting-record cards in that order. Label the cards as a Lesson 4 preview, without KR rule notation or a simulated verdict. | “leading evaluation marker” → only the host prefix; “without that prefix” → unprefixed line; “separate input areas” → browser pane/field labels; “period” → both final periods; “same parcel” → the shared-case requirement; “packing” / “one for its label” → the two confirmations; “rule” / “claim” → their titles; “supporting records” / “derived” → their labels. | 55–65 s |

The storyboards specify semantic targets; exact bounds must come from the final
text layout, not copied coordinates. Explanatory highlighting starts as the
matching phrase is spoken. Inspect actual rendered frames at every transition
between predicate, argument, label and punctuation before creator recording.

## Short storyboard

Keep the portrait parcel and notation illustrations at the opening. For the
demonstration, frame the real playground’s two left-hand inputs together to
establish their relationship, then move closer to the claim text, the given
support/TRUE badge and the unchanged premise. Pull back for the input-area
comparison. Preserve native panel positions; do not rearrange UI fragments into
cards. Use dedicated portrait cameras and reserve captions/platform-control space.

| Scene / passage | Vertical composition | Highlight trigger and target | Planning time |
|---|---|---|---|
| S01 · [Note](#v03a-short-note) | Establish parcel A and the packing note, then a list headed “Packing notes”. Series label stays small. | “Parcel A” → parcel tag; “note” → note; “list” → list entry. | 9–11 s |
| S02 · [Premise](#v03a-short-premise) | Reveal `member(ParcelA, PackedParcels).`, wrapping after the comma if needed. Then show the real reviewed one-premise playground, framed to orient both input areas. | “Member” → predicate token; “parcel” → `ParcelA`; “list” → `PackedParcels`; “premise” → the actual premise text. | 15–18 s |
| S03 · [Claim](#v03a-short-claim) | Move from the two-input view into the actual claim text, then to TRUE and given support, and back to the unchanged premise. | “claim field” → claim text; “TRUE” → proof verdict; “supplied that premise” → [given]; “adds nothing” → unchanged premise text. | 13–16 s |
| S04 · [Boundary](#v03a-short-boundary) | Widen to the real input areas, return to the physical-parcel illustration, and finish with the existing Lesson 3 closing scene; no response hold or audience task. | “input area” → the actual premise and claim text; “physical parcel” → actual parcel drawing. | 10–13 s |

<a id="statement-reference"></a>
## Statement reference

This is the draft public companion reference. Publication remains a later step.
It is not additional narration and should not be shown as a dense teaching slide.

| Item | Meaning in this lesson |
|---|---|
| `ParcelA`, `PackedParcels` | Names chosen for one parcel and one collection. A name alone does not establish membership, packing or any other property. Names begin with an uppercase ASCII letter; answers can display lowercase forms. |
| `member(member, set)` | Vocabulary signature: predicate name and its two places in order. `member` and `set` here describe the places; executable examples supply names in them. No optional sort declaration is introduced. |
| `member(ParcelA, PackedParcels).` | Positional statement; first argument fills `member`, second fills `set`. A statement of a relation is also called a predication. |
| `member(member: ParcelA, set: PackedParcels).` | Named arguments fill the declared places explicitly. Reordering complete labelled pairs preserves their roles. |
| Mixed forms | Positional arguments must come before named ones; do not fill a place twice. `parcel:` is not a declared label for `member`; use `member:`. |
| `(` `)` `,` `:` `.` | Argument enclosure, separator, label separator and statement terminator. Keep the final period in both premise and claim inputs. |
| Playground / query interface | Custom KR pane supplies premises; compare the reading and select Use reviewed encoding. The claim field receives KR without the host's leading marker. Formalize and English-to-KR translation are outside this lesson. |
| Host commands | A plain KR line supplies a premise. A leading `? ` selects claim evaluation. These are interface choices; `?` is not part of the displayed KR statement. Installation, other commands and witness finding are left to their own guides. |
| TRUE / FALSE here | TRUE cites the supplied premise. FALSE means this claim was not derivable in this example; it does not prove the physical parcel lacks a label. Other outcomes are taught later. |
| Refusal | Unsupported predicate, unfinished statement or invalid argument layout: no claim verdict. A playground admission failure discards the whole candidate KB. |
| Keywords | Reserved syntax words cannot be used as convenient predicate names. See the language contract's lexical reference; no keyword catalog is needed to write these two statements. |
| Tags (`via`) | Add a separate relation connected to the main predication's first participant. They do not name its existing argument places. Their worked demonstration belongs to the later events/tags lesson. |

No variable, rule, connective, quantifier, omitted place or find result appears
in either narration. The example introduces only one supported predicate.

## Public source receipts

Technical receipts use the locally inspected public engine revision
`1eab5d1f1d52d1c106145aa0cbc18205ff9b0902`. The shop, its records and the library
variation are fictional teaching examples; their names are chosen here.

| Receipt | Public source | Supports |
|---|---|---|
| R01 | [Current language contract](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/NIBLI_KR.md), lexical structure, names and predication | Capitalization, named/positional arguments, punctuation, supported vocabulary and the reference-only distinction from tags. Teach the implemented profile, not proposed syntax or retired examples. |
| R02 | [Executable grammar](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/nibli-kr/src/nibli_kr.pest) and [resolver](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/nibli-kr/src/resolve.rs) | Statement terminator, name form, argument labels, positional ordering and vocabulary/arity refusal. |
| R03 | [Committed predicate entries](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/nibli-lexicon/src/corpus/predicates.rs) | `member` is curated, with `member` then `set`; `packed` is not a supported predicate in this build. |
| R04 | [Playground guide](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/mdbook/src/user/playground.md), [UI implementation](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/nibli-ui/src/main.rs) and [review state](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/nibli-ui/src/review.rs) | Separate premise/claim inputs, reviewed encoding, edit invalidation, fresh candidate admission and all-or-nothing failure. The bundled browser execution is verified separately below; public deployment was not checked. |
| R05 | [Public README](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/README.md) and [host implementation](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/nibli-host/src/main.rs) | Relative support of verdicts, FALSE qualification and host assertion/evaluation routing. |
| R06 | [KR cookbook](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/mdbook/src/user/kr-cookbook.md) | Public follow-up reference for current statements and places. |

| Narration passages | Basis |
|---|---|
| `v03a-shop`; `v03a-short-note` | Original situations and records; R01–R03 for their formal representation. |
| `v03a-build-statement`; `v03a-named-places` | R01–R03 and the positional, named and reversed-role engine checks. |
| `v03a-supply-premises` | R04; native review/admission tests and the bundled-browser receipt below. |
| `v03a-evaluate-claims`; `v03a-supplied-conclusion` | R04–R05 and repeated-evaluation/direct-given fixture receipts. |
| `v03a-refusals` | R01–R04 and refusal/repair fixtures; actual bundled-browser messages captured below. |
| `v03a-host-and-next` | R04–R06; verified host routing and established course sequence. |
| `v03a-short-premise`; `v03a-short-claim`; `v03a-short-boundary` | The same original packing record, R01–R05 and the main lesson's engine checks. |

## Verification and production handoff

**Timing estimate:** The revised main has 1,106 spoken words; the Short has 117,
excluding displayed code and markup and counting hyphenated words as one.
At 140–160 words per minute, speech alone is approximately 6:55–7:54 and
0:44–0:50. With ordinary speech/scene spacing, budget roughly 7–9 minutes for
the main and 48–57 seconds for the Short. The main's hard ceiling is 13 minutes,
with no minimum. The Short's target is 45–60 seconds. These are writing-stage estimates; measured render durations are recorded below.

**Retained engine evidence:** Expectations were saved before execution. The library fixture is retained as historical regression evidence; it is no longer shown in the lesson. New sessions cover
the two premises, repeated evaluations with identical before/after fact listings,
named-place equivalence, positional reversal, supplying a desired conclusion,
the library variation, unsupported vocabulary, extra/duplicate/misordered
arguments, wrong labels and period repair. They run with materialization off
and on, existential import off and depth 10. The existing host and WASM
component were used unchanged through an installed compatible glibc loader;
this was not a fresh host build. Historical fixture captures are preserved.
All eight sessions pass on both profiles: 28 expectations per profile, 56
checks in total. No expectation was changed to match the output.

The missing-period refusal lists possible continuation tokens rather than
literally saying “missing period”. Show that real message; the narration
explains the successful repair. Do not replace it with a friendlier invented
engine message. TRUE support is marked `[given]` in the host receipt. The
missing-label FALSE includes a complex internal tree; only the verdict and
qualification belong in this first lesson.

**Writing-stage playground limit (superseded by the local capture below):**
All seven existing native UI tests passed, including
exact-version review, edit/revert invalidation, independent KB/claim reviews,
whole-candidate admission failure, empty KB behavior, valid loading and shipped
examples. Browser discovery returned no browsers and “The browser access host
is unavailable”. A web fetch exposed only the static page shell. Neither result
establishes a successful interactive walkthrough or the deployed build's identity.

The writing package required the following walkthrough. It is now complete on
the real bundled [playground](https://dhilipsiva.dev/nibli-playground/) served
locally; this does not establish the public deployment’s current identity:

1. Select Custom, replace the KR pane's contents with the original two period-terminated records,
   inspect Back-translation and activate Use reviewed encoding. Keep the
   optional English Source input empty; no translation service is used.
2. Evaluate the packing claim twice; inspect the direct given support and
   confirm the pane is unchanged. Evaluate A's label claim and retain its FALSE
   qualification without expanding the internal failure tree.
3. Change an input, including an edit followed by undo, and confirm fresh review
   is required. Test the labelled spelling in a separate reviewed pane.
4. In a separate comparison, supply the missing label statement, re-review and
   inspect its given support. Restore and review the original two records.
5. Test each refusal in a fresh pane, preserving its actual message. Also try a
   valid line followed by an invalid line: no partial KB verdict is acceptable.
6. Restore the original two shop records for the closing reference. Record the
   actual build/profile and capture date; reconcile differences before filming.
   The captured Copy 7 case remains historical evidence and is omitted from
   the revised cut.

**Writing checks:** Spoken word counts exclude fences/markup. Both narrations
pass the audience-task and claim/evaluation framing screen. The full package's
originality screen has no shared 12-word window; its four six-word matches are
ordinary wording about what a result establishes and the profile settings,
reviewed as generic technical language. Local links, anchors and pinned public
source paths resolve. The course structure/coverage checker passes. The prior
840 protected files and 2,581 existing lesson project/media file entries are
unchanged. These checks establish consistency, not independent authorship,
beginner comprehension or media readability.

- [x] Draft continuous main and independent Short narration with separate QUINE storyboards.
- [x] Tie highlights to exact spoken targets, including both uses of `member` and the final period.
- [x] Write new example expectations before execution and preserve prior cases/media.
- [x] Inspect the actual playground source and run its existing native tests.
- [x] Complete the real bundled-playground walkthrough and capture its actual results.
- [x] Obtain the creator’s writing approval and request for synthetic rendering.
- [ ] Review the explanation with a beginner; production checks do not establish comprehension.
- [x] After writing review, configure a Vidni manifest and generate synthetic long/Short drafts.
- [x] Record the approved narration and measure the recorded timelines and chapter times.
- [x] Record the owner’s explicit review confirmation, build both final cuts and inspect the exports.
- [x] Prepare the full YouTube/Short publishing and social-sharing package against the recorded timeline; update its full-lesson links to the creator-supplied published URL on 2026-11-20.
- [x] Confirm the publishing copy and chapter times against the final exports and include them in the delivery.

The main title is **Writing Your First Nibli KR Statements |
Neuro-Symbolic Lesson 3**. The Short's focus is **Same Statement, Different Job:
Premise or Claim**. Thumbnail concept: the same statement between “Premise” and
“Claim” cards. The copy below uses the measured final chapter times.


<a id="recorded-voice-review"></a>
## Recorded-voice review and export handoff — 2026-11-19

This records the initial preparation and handoff. The later
[owner confirmation](#recorded-final) supersedes its pending-verification status.

The creator reported recording complete. Vidni reports both cuts ratified and
**111 current takes, zero stale and zero missing**: 99 main lines and 12 Short
lines. The local speech check reports 111 heard as scripted, zero below its
0.90 advisory threshold and zero silent clips. This automated comparison is
not a human pronunciation or pacing review. All saved takes are 48 kHz mono
16-bit PCM; none contains a full-scale clipped sample. No retake was requested.

The native recorded timelines are **433.604 seconds** (main) and **48.202
seconds** (Short), including ordinary line/scene spacing. The main has 127
caption cues and eight measured chapters; the Short has 13 caption cues.
The native audit writes matching SRT/VTT sidecars for the continuous reviews.
The ratification and all 572 recording/metadata files are unchanged. A
before/after hash inventory covers those 573 files and the manifest; the only
manifest edit removes the obsolete “Synthetic narration for review” label from
the publishing description. Its narration, scenes, cameras and timing are
unchanged, and both cuts remain ratified.

Recorded audio requires its own cue bindings. All **81 main and 12 Short
highlights** have advisory intervals; none is owner-verified yet. Eight exact
phrase matches needed explicit advisory substitutions from local alignment
receipts, such as `labeled pair` for `labelled pair`. The initial `TRUE`
proposal in `v03a-supplied-conclusion-03` began at 0.29 seconds into its take,
while the camera was still arriving. The visibility guard rejected it.
A separate DTW alignment places the `true` token centre at 0.64 seconds;
neighbouring token midpoints provide a **0.52–0.91 second proposal**. This is
documented alignment evidence, not verification by the assistant.

All **6,397 main and 818 Short active highlight frames** pass native visibility
and caption-overlap checks with these proposals. Fifty-seven main and three
Short highlights receive the full three-second minimum; the remaining 24 main
and nine Short highlights end when their scene or camera removes the target.
Those cutoffs are explicitly recorded in the audit. No speech gaps were added
to extend a highlight. No narration, scene, camera or recorded take was changed.

Both continuous diagnostic previews pass complete decoding, receipt/hash,
duration, dimensions, 30 fps H.264 and 48 kHz stereo AAC checks. Selected encoded
frames were inspected for ParcelA, simultaneous name highlights, TRUE with its
given support, the Lesson 4 handoff and the Short’s statement and verdict.
All 71 main and 10 Short synchronized review clips match current native cue
bindings, overlapping context and media hashes. They retain the visible review
label; these are not final exports.

- [Full lesson review · 7:14](build/cues/full/long-takes.review.mp4)
- [Short review · 0:48](build/cues/full/short-takes.review.mp4)
- [Main captions · SRT](build/cues/full/long-takes.review.en.srt)
- [Short captions · SRT](build/cues/full/short-takes.review.en.srt)

Audit evidence, alignment receipts and recording checks live in
[build/review/recorded-voice](build/review/recorded-voice/).

The owner opens the synchronized reviewer in the Vidni Nix shell:

```sh
./target/release/vidni cues review examples/nibli-v03a-first-statements --voice takes
```

Play each clip with audio, check its target and timing, and select **Verify this
cue**. Adjust and rebuild a proposal first if needed. Both cuts are included.
[Vidni’s assistant protocol](../../docs/assistant.md) reserves verification for
the person: “an assistant cannot verify its own suggestions.” The assistant
does not write `cues.lock` or bypass the final-build gate. After this review,
build the ratified recordings with `vidni final`, check the exports and confirm
the prepared publishing copy against the delivered runtime and chapters.
The final-export preflight was run and refused the unverified cues as expected;
it produced no final export. The release desktop reviewer was rebuilt against
the current three-second-hold implementation before this handoff. Vidni
validation and the course document checker pass; the latter reports zero
problems with all 73 briefs, 276 coverage families and 818 obligations retained.

<a id="recorded-final"></a>
## Owner confirmation and final production — 2026-11-19

The creator confirmed that they watched the review videos and that the cues
are correct, while clarifying that they had not used the per-cue buttons.
That explicit decision is now recorded through Vidni’s
[`cues record-confirmation` workflow](../../docs/cues.md#confirming-a-continuous-review).
The assistant relayed the owner’s decision; it did not substitute its own
assessment or use the interactive verifier on the owner’s behalf.

The command checked the exact reviewed MP4 hashes and their receipts against
every current audio, visual and timing binding, then atomically saved **81 main
and 12 Short cue records**. The lock preserves the owner’s actual statement,
each video and receipt hash and the fact that an assistant relayed it. The
[reviewed-artifact inventory](build/review/recorded-final/reviewed-artifacts.json)
links this decision to the 7:13.604 main and 0:48.202 Short. No recorded take,
narration, scene, camera, cue interval or reviewed video was changed.

The normal final-build gate passed, and all exports are complete:

| File | Runtime | Format |
| --- | --- | --- |
| [Full master](out/long.mp4) | 7:13.604 | 1920×1080, separate captions |
| [Captioned main](out/long.captioned.mp4) | 7:13.604 | 1920×1080, burned captions |
| [Vertical Short](out/short.mp4) | 0:48.202 | 1080×1920, burned captions |

The main has [SRT](out/long.en.srt), [VTT](out/long.en.vtt) and
[chapter text](out/long.chapters.txt); the Short also has
[SRT](out/short.en.srt) and [VTT](out/short.en.vtt) sidecars. The complete
[publishing/sharing copy](#publishing-copy) uses those final chapter starts.

All three files pass native QA, complete decoding, checksum and format checks:
30 fps H.264 with 48 kHz stereo AAC. Loudness is −14.1 LUFS; true peak is
−2.0 dBTP for both main variants and −2.1 dBTP for the Short. Final normalized
audio, timelines and caption text match the owner-reviewed versions exactly.
Selected final frames show the intended highlights without the diagnostic
review label; the master also has no burned captions. All 575 original source,
ratification, voice and recording files in the final-stage inventory are
unchanged, as are the 93 saved cue records and both confirmation receipts.
The final review does not establish beginner comprehension. No release approval
or upload was recorded by the assistant.

The new confirmation path passed its focused core/media/CLI tests and a real
continuous-review render test. Tests cover changed recordings and artifacts,
wrong cuts/voices/hashes, missing or altered cues, absent owner text and repeated
confirmation. A separate multi-cut check supplied a mismatched Short hash:
the whole operation was refused and no partial main confirmation was written.
Interactive self-verification remains unavailable to assistants; changes still
invalidate the relevant cue bindings. Clippy and formatting checks pass.
Evidence is retained under [build/review/recorded-final](build/review/recorded-final/).

<a id="publishing-copy"></a>
## Publishing and sharing copy — published link updated, 2026-11-20

Confirmed against the final main lesson (**7:13.604**) and independent Short
(**0:48.202**). All eight chapter starts match the delivered chapter file;
the final audio, timeline and captions match the owner-reviewed videos.
The creator reported the main lesson uploaded on 2026-11-20 and supplied this
[published Lesson 3 URL](https://www.youtube.com/watch?v=ceEup-8bjyk&list=PLAQKTXMK9CyY).
All full-lesson links below use that exact URL, including its playlist context.
No Short publication has been reported. This updates sharing copy; the assistant
has not posted it or written to Vidni’s native publication register.

### Main YouTube title

```text
Writing Your First Nibli KR Statements | Neuro-Symbolic Lesson 3
```

### Main YouTube description

```text
Two parcels. Two notes. A clear way to write down what each note says and evaluate claims against those records.

In Neuro-Symbolic Lesson 3, we turn an everyday shop example into our first Nibli KR statements. We introduce names, the member predicate, argument positions and named arguments, then use the Nibli playground to supply premises and evaluate claims.

We inspect a TRUE result supported by a supplied premise, explain what FALSE means in this example, and show why adding a desired conclusion to the premises doesn't create a missing observation. We also repair an unfinished statement, unsupported vocabulary and an extra argument.

A result supported by our records does not establish that the physical parcel matches them. We keep that distinction visible throughout.

This course builds from symbolic reasoning toward combining it with neural AI. No formal logic or programming background is required. The shop is a fictional teaching example.

Chapters
0:00 Two parcels and their records
0:47 Writing the first statement
1:49 Positions and named arguments
2:47 Supplying and reviewing premises
3:44 Evaluating claims and their support
4:41 Supplying a desired conclusion
5:16 Refusals and repairs
6:13 Interfaces and the next lesson

Previous lesson — Introducing Nibli KR:
https://www.youtube.com/watch?v=i0nlnWrlFCw&list=PLAQKTXMK9CyY&index=1

Try the playground: https://dhilipsiva.dev/nibli-playground/
Nibli: https://github.com/dhilipsiva/nibli

Next, Lesson 4 continues with the same parcels: we write a dispatch-checklist rule, evaluate a claim and follow its derivation.

#SymbolicReasoning #NeuroSymbolicAI #KnowledgeRepresentation
```

### YouTube Short title

```text
Same Statement, Different Job: Premise or Claim | Neuro-Symbolic Lesson 3
```

### YouTube Short description

```text
A note says Parcel A is packed. We write that record as a Nibli KR statement, then place the same words in two different input areas.

In the knowledge base, the statement supplies a premise. In the claim field, it is evaluated against the supplied records. Evaluating the claim adds nothing to the record, and neither input checks the physical parcel.

A 48-second example from Neuro-Symbolic Lesson 3. No logic or programming background needed.

Full lesson: https://www.youtube.com/watch?v=ceEup-8bjyk&list=PLAQKTXMK9CyY

#Shorts #SymbolicReasoning #NeuroSymbolicAI
```

### LinkedIn link post

```text
A claim can return TRUE simply because we supplied that same statement as a premise. That distinction matters when we inspect the support behind a result.

In Neuro-Symbolic Lesson 3, I start with two parcels and their packing records. We write our first Nibli KR statements, explain predicates and argument roles, and evaluate claims in the playground.

We also see why supplying a desired conclusion changes the starting information without creating a missing observation.

This is part of my Neuro-Symbolic Reasoning from First Principles course. No formal logic or programming background is required.

Watch Lesson 3: https://www.youtube.com/watch?v=ceEup-8bjyk&list=PLAQKTXMK9CyY

#NeuroSymbolicAI #SymbolicReasoning #KnowledgeRepresentation
```

### Facebook link post

```text
One note says Parcel A is packed. Another says Parcel B has a label. Keeping those records straight is the starting point for Lesson 3 of my Neuro-Symbolic Reasoning course.

We turn the notes into simple statements, then evaluate claims against them. Along the way, we see why a TRUE result still needs its supporting record to be checked.

The lesson starts in plain English and introduces the notation step by step. No logic or coding background needed.

Watch: https://www.youtube.com/watch?v=ceEup-8bjyk&list=PLAQKTXMK9CyY
```

### X.com link post

```text
The same statement can supply a premise or make a claim for evaluation.

Neuro-Symbolic Lesson 3 turns two parcel notes into Nibli KR statements, then inspects what supports the results.

Watch: https://www.youtube.com/watch?v=ceEup-8bjyk&list=PLAQKTXMK9CyY
```

### Threads link post

```text
Adding a desired conclusion to our records can make its evaluation return TRUE. It doesn't create the missing observation.

In Neuro-Symbolic Lesson 3, we write our first Nibli KR statements using two parcels and their notes. We explain names, predicates and argument roles, then evaluate claims and inspect their support.

Everyday example first; notation follows. No formal logic background needed.

Full lesson: https://www.youtube.com/watch?v=ceEup-8bjyk&list=PLAQKTXMK9CyY
```

### Instagram link post

```text
Two parcels. Two notes. Our first statements in Nibli KR.

Neuro-Symbolic Lesson 3 builds from an everyday shop example into names, predicates and argument roles. Then we use those records to evaluate claims and inspect what supports the results.

Supplying a premise and evaluating a claim do different jobs—even when their words are identical.

Full lesson on YouTube: https://www.youtube.com/watch?v=ceEup-8bjyk&list=PLAQKTXMK9CyY

#NeuroSymbolicAI #SymbolicReasoning #KnowledgeRepresentation
```

### WhatsApp Status caption

```text
From parcel notes to our first Nibli KR statements—and the claims those records support.
Neuro-Symbolic Lesson 3: https://www.youtube.com/watch?v=ceEup-8bjyk&list=PLAQKTXMK9CyY
```

### Instagram Reels caption

```text
Same statement, different job.

In the knowledge base, we supply a premise. In the claim field, we evaluate it against our records. Neither input checks the physical parcel.

A 48-second example from Neuro-Symbolic Lesson 3.
Full lesson: https://www.youtube.com/watch?v=ceEup-8bjyk&list=PLAQKTXMK9CyY

#NeuroSymbolicAI #SymbolicReasoning #KnowledgeRepresentation
```

<a id="parcel-transition"></a>
## Parcel-only ending and Lesson 4 handoff — 2026-11-19

The creator approved removing the library detour. Seven library lines and their
active scenes are removed; M13 is retired. All surviving line IDs remain stable.
The existing final line is rewritten and two new lines, `v03a-host-and-next-09`
and `v03a-host-and-next-10`, extend the closing. The main now has 99 lines; the
independent Short retains its 12 lines unchanged.

The closing stays with the shop’s dispatch checklist: both packing and label
confirmations must concern the same parcel. It then names Lesson 4’s work:
write that requirement as a rule, evaluate A’s checklist claim and trace the
supporting records to a derived conclusion. It does not present an executed
rule or introduce variables early. The retained parcel variations already
teach the names, predicate roles and premise/claim distinction; TODO’s coverage
mapping now points to those passages instead of Copy 7.

The three closing scenes share the same QUINE checklist layout so targets can
remain visible across narration lines. Measured text targets identify the
shared parcel, confirmations, rule, claim, supporting records and derivation.
The existing three-second hold policy and natural speech spacing remain.

The refreshed [main · 7:15.016](build/cues/full/long-draft.review.mp4) is
18.416 seconds shorter than the preceding cut. The closing handoff begins at
6:51.755. Three closing narration clips were generated; 108 current clips were
reused. The current main has 1,106 spoken words and 127 caption cues. Its
[SRT](build/cues/full/long-draft.review.en.srt) and
[VTT](build/cues/full/long-draft.review.en.vtt) are rebuilt from the current
timeline. The independent Short is not rerendered. The earlier 7:33.432 main
is retained in [the preceding revision](build/review/before-parcel-transition-2026-11-19/build/cues/full/long-draft.review.mp4),
with its [script and production notes](build/review/before-parcel-transition-2026-11-19/notes.md).
The historical library fixtures, captures and audio clips remain available;
they are not part of the revised main. Fresh highlight timings remain advisory
until owner verification.

The [native frame audit](build/review/parcel-transition/native-frame-checks.json)
checks all 6,445 active main-cue frames. All seven closing highlights last three
seconds, including carries into the following narration line. The
[timing evidence](build/review/parcel-transition/advisory-timing-evidence.json)
documents the advisory label-phrase interval whose recognition end exceeded the
real clip length; it is clamped to that clip and remains unverified.

The [media checks](build/review/parcel-transition/media-checks.json) confirm a
complete error-free decode, 1920 × 1080 at 30 fps, AAC stereo at 48 kHz and the
current captions. The three changed closing scenes were inspected in encoded
frames. The [line-preview audit](build/review/parcel-transition/line-preview-checks.json)
passes for all 71 rebuilt main clips and the 10 retained Short clips, including
658 segment-boundary checks. The [narration check](build/review/parcel-transition/narration-checks.json)
confirms that both written scripts match their current recording lines. [Source checks](build/review/parcel-transition/source-checks.json)
confirm the retired lines, stable surviving IDs, unchanged Short and preserved
old voice clips. An [isolated recipe check](build/review/parcel-transition/recipe-checks.json)
reproduces the manifest and all 152 active generated SVGs exactly. The course
structure/coverage check and whitespace checks pass; the text screen has no
shared 12-word window. These checks do not replace a human listen-through or
establish beginner comprehension.

<a id="readable-highlights"></a>
## Readable highlights and predicate terminology — 2026-11-19

Historical production record before the parcel-only transition revision.

The creator selected **three seconds from appearance** as Vidni’s default
minimum, allowed simultaneous highlights, and chose to clear a highlight early
when a scene or camera change removes its target. The audio phrase and display
hold are now separate. Narration and scene pacing are not stretched. Measured
continuity follows unchanged content across narration lines; expired emphasis
does not reappear. Duplicate outlines are drawn once.

The missing whole-`ParcelA` cue now starts during its name. Its capital `P`
retains a separate cue. The two-things explanation marks both the parcel and
list, and both chosen names remain emphasized across the following narration
line. In the latest synthetic timeline, the reported cues display at:

| Target | Visible interval | Hold |
|---|---|---|
| Parcel A near 0:43 | 0:42.600–0:45.600 | 3 seconds |
| The two things | 0:46.333–0:49.333 | 3 seconds |
| `ParcelA` as a name | 0:51.200–0:54.200 | 3 seconds |
| Capital `P` | 0:53.467–0:56.467 | 3 seconds |
| Names we have chosen | 0:59.133–1:02.133 | 3 seconds |

**Terminology:** `member` is a predicate name; membership is the relation it
expresses. The lesson introduces “The predicate member expresses membership”
and uses predicate consistently for code tokens, vocabulary entries and arity
messages. Plain-English relationships remain relationships. This distinction
and the highlight preference are saved in the workspace’s agent instructions.
Seven main-lesson narration lines were regenerated; 109 clips were reused.
All 116 line IDs, recordings and prior production exports are preserved.

The cuts at that revision were [full lesson · 7:33.432](build/review/before-parcel-transition-2026-11-19/build/cues/full/long-draft.review.mp4)
and [Short · 0:50.343](build/review/before-parcel-transition-2026-11-19/build/cues/full/short-draft.review.mp4). The 1.149-second
change in main runtime comes from the seven wording corrections. There are
**93 cues: 81 main and 12 Short**; all are advisory and none owner-verified.
The [native audit](build/review/readable-holds/native-frame-checks.json) checks
7,186 active cue frames. 58 cues reach at least three seconds; 35 end earlier
because their target leaves the scene or camera, following the creator’s choice.
The exact cutoffs are listed in that audit rather than hidden by a duration claim.

The [timing evidence](build/review/readable-holds/advisory-timing-evidence.json)
records reused intervals only for identical audio, words and phrases. Three
new advisory intervals use documented local Whisper token boundaries, including
its recognition substitutions; they still need owner timing review. Review
clips include following narration through the full hold and check overlapping
cue context before verification.

The [source/document audit](build/review/readable-holds/source-checks.json),
[rendered-media audit](build/review/readable-holds/media-checks.json) and
[line-preview audit](build/review/readable-holds/line-preview-checks.json) record
validation, source preservation, measured geometry and encoded output checks.
All 84 line review clips include the complete visible hold and overlapping
cue context; 648 segment-boundary frame checks pass. Both complete MP4s decode
without errors. Their SRT and VTT captions are regenerated from the current
timeline, and the Short’s decoded audio matches its previous draft exactly.
The workspace suite passed 319 tests; the final overlap regression and focused
hold tests also pass, along with all five measured-browser tests, the continuous
review render test, clippy, project validation and document checks.
An [isolated recipe check](build/review/readable-holds/recipe-checks.json)
reproduces the same manifest and all 159 generated SVGs byte for byte; running
the continuity recipe again leaves the manifest unchanged.
The previous review copies and voice lock are archived under
[the prior revision](build/review/before-readable-holds-2026-11-19/notes.md).
Current renders remain visibly marked review copies until owner cue verification.

<a id="full-playground-zoom"></a>
## Full-playground camera revision — 2026-11-19

Historical production record before the readability and terminology revision above.

The creator requested the full-playground camera approach used by the earlier
intro, applying it to the playground demonstrations in both cuts while retaining
the opening illustrations and syntax explanations. **43 demonstration scenes**
now embed complete screenshots of the actual bundled playground at a fixed
1280 × 720 CSS viewport and 3× density. The application layout is unmodified.
The real-case walkthrough passes all 12 checks and records 20 states, including
reviewed pre-evaluation states for claim entry in both cuts.
See the [browser receipt](build/playground-zoom/receipt.json).

The landscape camera establishes the complete workspace, follows the premise
and review areas, widens before introducing the separate claim field, then
moves through the verdict and supporting derivation. It returns to the premise
pane for unchanged records and to broader views for input comparisons and
restoring the original example. Refusals retain their complete actual messages
and admission status. The Short frames the two left-hand inputs together, uses
tighter portrait views for claim text and proof support, and returns to the
unchanged premise. Existing 0.7-second easing supplies motion between lines;
no response gaps or simulated typing delays were added.

**Measured targets:** screenshot receipts hold DOM element/text-range bounds in
image pixels, tied to the exact PNG hash and capture browser. Each SVG embeds
that complete PNG; camera regions and emphasis use the same measured receipt.
Capture waits for both application stylesheets and fonts before measuring.
Remote font requests are blocked; the pinned Nix font configuration is hashed
in the receipt. The FALSE proof is collapsed through its native disclosure
control before capture, keeping advanced internals out of this introductory view.
The pointer is moved away from controls, focus is cleared and native transitions
finish before measuring; incidental hover outlines do not compete with emphasis.
Statement-wide emphasis in the Short joins syntax-token DOM fragments sharing
the same measured text row, so each statement has one continuous outline. The
[derived-target record](build/playground-zoom/line-target-provenance.json) retains
the raw measurement receipt and its hash. No character-count estimates, baked
highlight outlines or rearranged UI crops are used for these demonstrations. The native compositor checks every active
highlight frame through the camera transform and caption/platform clearances.

That revision had **92 cues: 80 main and 12 Short**, all advisory and none
owner-verified. Redundant highlights inherited from the earlier separate crops
were consolidated to their intended on-screen occurrence; the Short’s input
comparison now marks both actual inputs. Existing timing suggestions can be
reused only where the audio hash, spoken words and cue phrase match exactly;
the new visual bindings still require owner review. Its native
[frame audit](build/review/before-readable-holds-2026-11-19/build/review/playground-zoom/native-frame-checks.json) records
2,189 active highlight frames and 212 inspection frames at camera starts,
mid-move, settled positions and narration holds, including exits to illustrations.

The [source audit](build/review/before-readable-holds-2026-11-19/build/review/playground-zoom/source-checks.json) confirms that
the narration, all 116 line IDs, voice settings and timing remain unchanged.
The synthetic runs stay 7:32.283 and 0:50.343. The previous review copies and
receipts are archived for comparison:
[previous main](build/review/before-playground-zoom-2026-11-19/build/cues/full/long-draft.review.mp4)
and [previous Short](build/review/before-playground-zoom-2026-11-19/build/cues/full/short-draft.review.mp4).
Those review copies are archived as
[main](build/review/before-readable-holds-2026-11-19/build/cues/full/long-draft.review.mp4) and
[Short](build/review/before-readable-holds-2026-11-19/build/cues/full/short-draft.review.mp4).

**Rendered-media checks:** the [media audit](build/review/before-readable-holds-2026-11-19/build/review/playground-zoom/media-checks.json)
checks both complete video/audio streams, their receipts and hashes, H.264 at
30 fps, landscape 1920 × 1080 / portrait 1080 × 1920, and AAC stereo at 48 kHz.
Decoded narration matches the original synthetic videos exactly. Encoded frames
were inspected for workspace context, close inputs, verdicts, proof support,
refusal messages and statement outlines. The refreshed
[line-preview audit](build/review/before-readable-holds-2026-11-19/build/review/playground-zoom/line-preview-checks.json)
covers 84 clips and 460 cue-boundary frame references. All 24 focused geometry,
cue-gate and camera/compositor regressions pass; all 124 protected original
files remain unchanged. These checks do not mark any cue as owner-verified.

The [scene plan](build/playground-zoom-shot-plan.json) records every state,
landscape/portrait camera and cue target. The full-playground recipes at the end
of this document follow the base illustration recipe. Existing production
exports, original audio, owner locks and earlier lessons are preserved.
The videos remain visibly labelled diagnostic review copies until cue timing
has been checked by the owner.

<a id="measured-highlights"></a>
## Measured highlights and cue review — initial migration, 2026-11-19

Historical record before the full-playground revision above. That migration declared **98 emphasis cues across 84 narrated lines**, with
semantic targets in all 116 scene SVGs. It preserves every approved `show`,
`say` and line ID, the synthetic voice choices and all existing audio. The main
and independent Short retain their QUINE compositions and continuous narration.
Earlier published lessons are unchanged.

**Geometry:** Chromium draws and measures each SVG with pinned fonts. The
capital `P` is an exact text span; `PackedParcels` uses its full measured text
bounds. Adjacent code tokens are SVG text spans laid out by the browser rather
than character-width arithmetic. Captured playground content uses the original
DOM measurements and exact PNG hashes, carried through its SVG transform and
crop. No emphasis outline is baked into the scene. The compositor applies the
same camera movement to both content and outline, checks visible bounds and
caption/overlay collisions, and removes an expired outline before crossfading.

**Timing:** Local alignment proposes sample offsets for the exact audio clip.
Those are suggestions, not approved timings. The desktop reviewer plays short
synchronized clips with scrubbing, looping and single-frame steps. Changed
boundaries require a rebuilt preview. `cues.lock` records the owner's
verification for a particular voice, cut, line, cue, audio and layout. A retake,
trim, wording or relevant layout change makes that record stale. Draft and final
exports require all cues in their cut to be current; diagnostic clips cannot
be exported as the lesson. See [the Vidni cue workflow](../../docs/cues.md).

From the Vidni Nix shell, the owner's review commands are (the voice path
selects the shell's pinned Piper model):

```sh
VIDNI_VOICES="$VIDNI_PINNED_VOICES" ./target/release/vidni cues review examples/nibli-v03a-first-statements --voice draft
./target/release/vidni cues status examples/nibli-v03a-first-statements --voice draft
```

To begin with the reported passage, add `--cut long --only
v03a-build-statement-01 --only v03a-build-statement-02`. Both cuts still need
review before full production export. After recording, run review again with
`--voice takes`; synthetic verification does not transfer to recorded narration.

**Review artifacts:** 74 main clips and 10 Short clips cover all 98 cues.
All 98 now have advisory intervals; none is owner-verified. The initial pass
proposed 69 intervals; the remaining 29 were supplied for the continuous review
below using saved recognition segments. Each clip uses the cut's full resolution
and 30 fps, with diagnostic
labeling and the production master's 48 kHz stereo audio. Both processed master
WAVs are byte-identical to the first drafts' audio. The long-name proposal uses
the recognizer's measured 0.65–2.00 s segment, where it merged “list” with
“Packed Parcels”; [its advisory receipt](build/cues/packed-parcels-proposal.json)
records that limitation. It still requires the owner's timing review.

| Passage | Diagnostic clip | Full-resolution evidence |
|---|---|---|
| Capital P, reported around 0:51 | [Play line](build/cues/long-v03a-build-statement-01-c9a93540db66.review.mp4) | [Before](build/cues/long-v03a-build-statement-01-frame-1603.png) · [During](build/cues/long-v03a-build-statement-01-frame-1604.png) · [After](build/cues/long-v03a-build-statement-01-frame-1633.png) |
| Full `PackedParcels` name | [Play line](build/cues/long-v03a-build-statement-02-40c88cc5d1a6.review.mp4) | [Before](build/cues/long-v03a-build-statement-02-frame-1661.png) · [During](build/cues/long-v03a-build-statement-02-frame-1682.png) · [After](build/cues/long-v03a-build-statement-02-frame-1702.png) |
| Short: TRUE and given support | [Play line](build/cues/short-v03a-short-claim-02-84ff13dfd3e7.review.mp4) | [Given support](build/cues/short-v03a-short-claim-02-frame-968.png) |

The [main review index](build/cues/long-draft.review.json) and
[Short review index](build/cues/short-draft.review.json) bind each clip to its
current audio, geometry and proposed interval. Rebuilds after timing changes
update those indexes; the desktop reviewer follows the current entries.

**Engineering checks:** [The initial audit](build/review/measured-highlight-checks.json)
checked all 84 clip hashes and formats and 345 before/during/after boundary-frame
references. The [refreshed audit](build/review/continuous/line-preview-checks.json)
checks all 84 current clips and 490 boundary-frame references for the 98 proposed
intervals. The 332 workspace tests passed, followed by focused geometry,
recorder, reviewer and build-gate regressions. Four Chromium tests cover the
capital and long name, font and text changes, wrapped text, ambiguous matches,
hidden/clipped targets and screenshot receipts. Twelve legacy Chromium capture
tests pass. Recorder scenarios cover Space start/stop/retake, debounce, discard,
Enter keep, saved-take preservation and microphone-check retry. Reviewer tests
cover accessible controls and refusal to verify a changed or unplayed preview.
Clippy with warnings denied, schema/key consistency, document links/fences,
whitespace and project validation pass.

A [real draft attempt](build/review/cue-build-gate.txt) is refused for unverified
cues before replacing either existing MP4. All 124 protected audio, lock and
original media hashes match their pre-migration inventory. These checks verify
the implementation and artifacts; the owner still verifies narration timing.

**Pending production:** owner cue verification; production synthetic long/Short
exports and their final media checks; creator recording; verification of cues
against those takes; final creator-voice exports and automatic sharing copy.
The old video files below have deliberately not been overwritten or relabeled
as corrected outputs. No verification or release approval was performed by the
assistant.

<a id="continuous-synthetic-review"></a>
## Continuous synthetic review — before playground zooms, 2026-11-19

Historical record of the first measured review copies. The full-playground
revision above supersedes their compositions and cue counts.

At the creator's request, both cuts were rendered with the existing pinned
Piper `en_US-joe-medium` voice, QUINE styling, captions and measured highlights.
These use the complete original timeline and continuous narration. A persistent
top label reads “REVIEW COPY — HIGHLIGHT TIMING UNVERIFIED”; the files are
diagnostic review artifacts, and do not mark any cue as approved.

| Cut | Current video | Measured runtime | Picture | Captions |
|---|---|---|---|---|
| Main | [Synthetic review](build/review/before-playground-zoom-2026-11-19/build/cues/full/long-draft.review.mp4) | 7:32.283 | 1920 × 1080, 30 fps | [SRT](build/review/before-playground-zoom-2026-11-19/build/cues/full/long-draft.review.en.srt) · [VTT](build/review/before-playground-zoom-2026-11-19/build/cues/full/long-draft.review.en.vtt) |
| Short | [Synthetic review](build/review/before-playground-zoom-2026-11-19/build/cues/full/short-draft.review.mp4) | 0:50.343 | 1080 × 1920, 30 fps | [SRT](build/review/before-playground-zoom-2026-11-19/build/cues/full/short-draft.review.en.srt) · [VTT](build/review/before-playground-zoom-2026-11-19/build/cues/full/short-draft.review.en.vtt) |

**Timing proposals:** The 29 previously unset cues now use advisory intervals
selected from cached recognition segments for their exact audio. The
[proposal evidence](build/review/continuous-review-proposal-evidence.json)
records the audio hashes, recognized text, segment indexes and limitations.
Transcription variants and merged words required conservative boundaries in
some cases, including the short FALSE line. All 98 proposed intervals fit their
audio and contain a video frame. The owner must still check their timing with
the synchronized reviewer; `cues.lock` has not been created.

**Checks:** The [continuous-video audit](build/review/continuous/checks.json)
checks file hashes, H.264 picture, AAC stereo 48 kHz audio, dimensions, frame
counts and runtime. Decoded narration is byte-identical to the original videos
for both cuts. As in those originals, the encoder trims one final partial
picture frame to the audio duration; the audit records rendered and encoded
frame counts separately. All 124 protected original files retain their hashes.
The [line-preview audit](build/review/continuous/line-preview-checks.json)
also passes for all 84 refreshed clips and 490 boundary-frame references.

Encoded movie frames were visually inspected for the exact
[capital P](build/review/continuous/long-v03a-build-statement-01-focus-1-during.png),
the complete [PackedParcels name](build/review/continuous/long-v03a-build-statement-02-focus-1-during.png),
their unhighlighted boundary frames, the named-argument comparison,
the [FALSE verdict](build/review/continuous/long-v03a-evaluate-claims-10-focus-1-during.png)
and the Short's [given support](build/review/continuous/short-v03a-short-claim-02-focus-2-during.png).
These checks establish placement and rendering; they do not replace owner
verification of synchronization against speech.

Vidni now supports a repeatable continuous diagnostic render from the Nix shell:

```sh
VIDNI_ASSISTANT=1 VIDNI_VOICES="$VIDNI_PINNED_VOICES" \
  ./target/release/vidni cues preview examples/nibli-v03a-first-statements --voice draft
```

It writes into `build/cues/full/`, with a hash-bound JSON receipt alongside
each video, and preserves the production export gate. Four focused media cue
tests pass, including the continuous timeline/stereo-audio regression and
refusal to treat diagnostics as production builds. Workspace Clippy with
warnings denied also passes. Creator recording and final publishing copy remain
later production steps.

<a id="synthetic-production-record"></a>
## Synthetic production record — 2026-11-19

Historical record of the first draft, before the highlight issue was reported.
Its media checks do not verify the replacement cue system above.

The creator approved the main and Short and requested synthetic voice. Both
scripts remain word-for-word unchanged. The manifest divides them into 104 main
lines and 12 independent Short lines; the two punctuation splits preserve the
connected narration. Piper `en_US-joe-medium`, length scale 1.04, supplies the
review voice. Voice-only pronunciation hints separate the letter A and read KR
as letters. This is synthetic narration, not an imitation or recording of the
creator’s voice. There is no music, audience task or response hold. Spacing is
0.32 seconds between lines, 0.65 between sections, 0.6 before the first line and
1.2 after the final line.

**Real playground evidence:** [The browser receipt](build/playground/receipt.json)
contains 12 passing checks and 18 captured states, including the separate
one-premise Short. It executes the actual Dioxus/WebAssembly playground through
its UI in Chromium 156.0.8078.79, served from the website repository’s bundled
public files on loopback. No result, proof, error or review state is mocked.
The optional English Source input stays empty. No translation provider is used.
The direct shared browser was unavailable; the public deployment was not reached.

The bundle’s shared build manifest names engine revision
`36739cca7060d0758af36e7d56b0623a930943a2`. That manifest’s `wasm_sha256` refers
to the separate guided-page module, as the website build script specifies; it
is verified against that module. The actual playground module has SHA-256
`ef4f340bbd0ecbe4c17c69b5ec814753336319f37e30fcbd41a0ec27764f2925`.
The technical source receipts and host checks above use current engine revision
`1eab5d1`; they are not mislabeled as this browser build. Both implement the
current KR forms used here. The older bundle back-translates membership as
“Parcela members Packedparcels” and prints lowercase names in its proof.
Screenshots retain that real gloss; the narration explains membership in plain
English without silently replacing the engine’s output.

The walkthrough verifies review before evaluation, packing TRUE with `[given]`
support, repeated evaluation without changing premises, the missing-label FALSE,
edit/revert review invalidation, equivalent named forms and reversed roles,
supplying the desired conclusion, each input refusal, whole-candidate rejection,
the fresh library example and the independent Short. Original records are
restored in a fresh reviewed pane. Refusal scenes retain complete real messages;
the FALSE comparison shows the verdict with its qualification rather than the
advanced internal failure tree.

**Visual production:** 116 QUINE compositions pair original parcel/list/book
illustrations with enlarged actual UI crops. Highlights follow the narrated
names, role labels, punctuation, verdicts and given support. Screenshot targets
come from DOM text ranges recorded during capture; positions are transformed
with each crop. The Short has separate portrait-safe compositions. The extra
argument is displayed before its refusal. Technical build identifiers stay in
these notes, outside the teaching visuals.

The full resvg CLI flattens the source SVGs to generated PNGs before native
composition because Vidni’s linked SVG renderer omits embedded raster images.
An initial in-progress render exposed that limitation and was stopped; the
corrected render uses the verified flattened images. No engine, UI or Vidni
implementation was changed for this production.

<!-- V03A_MEDIA_RECORD -->
| Cut | Review video | Measured runtime | Picture | Captions |
|---|---|---|---|---|
| Main | [long.mp4](build/draft/long.mp4) | 452.283 s · 7:32.283 | 1920×1080, 30 fps | 132 cues, burned |
| Independent Short | [short.mp4](build/draft/short.mp4) | 50.343 s · 0:50.343 | 1080×1920, 30 fps | 13 cues, burned |

Additional files: main [SRT](build/draft/long.en.srt),
[VTT](build/draft/long.en.vtt) and [nine measured chapters](build/draft/long.chapters.txt);
Short [SRT](build/draft/short.en.srt) and [VTT](build/draft/short.en.vtt).
These are captioned draft exports. The final creator-voice pass will produce the
main master, captioned variant, Short and complete publishing/sharing package.

**Verification:** [Media checks](build/review/media-checks.json) confirm both
output hashes against native build receipts, H.264 at 30 fps, AAC stereo at
48 kHz, timeline/caption text, natural gaps and nine main chapters. Both native
QA reports have no problems. The main measures −14.1 LUFS and −1.9 dBTP; the
Short measures −14.1 LUFS and −2.2 dBTP. Caption rates stay below the unchanged
20-character/second gate (19.99 main; 16.87 Short). Main and Short meet the
13-minute ceiling and 45–60-second target respectively.

All 116 native line-review frames were inspected, with six decoded MP4 frames
checking named labels, TRUE/given support, the FALSE qualification, complete
refusal text and portrait layout. The Short’s TRUE badge crop was widened to
include its complete leading symbol. The red platform-occlusion mask appears
only in review sheets, not in the MP4. Physical-phone viewing and beginner
feedback remain separate from this frame review.

[Source checks](build/review/source-checks.json) confirm exact preservation of
the approved scripts, all manifest lines, QUINE styling, 12 browser checks,
840 protected file hashes and 2,581 earlier-lesson file metadata entries.
The course document checker and whitespace checks pass. The viewer-text
[originality screen](build/review/originality.txt) finds no shared 12-word window
against the 32 canonical manuscripts; its two six-word matches are ordinary
wording about what a result does not establish. This is a mechanical text
screen, not a judgment of authorship or learner comprehension.

**Speech review:** [The native speech comparison](build/review/speech-check.txt)
scores 113 of 116 clips at or above 0.90, with audio present for every line.
Three remain below that threshold; these are automatic transcription flags,
not confirmed speech errors or a completed human listen-through:

| Main time | Line | Phrase to listen to | Score |
|---|---|---|---|
| 0:50.031–0:54.408 | `v03a-build-statement-01` | “We name the parcel Parcel A … capital P.” | 0.86 |
| 1:15.995–1:18.793 | `v03a-build-statement-07` | “A set is the collection we're calling a list here.” | 0.88 |
| 4:27.621–4:30.372 | `v03a-evaluate-claims-12` | “Our label note concerns B, not A.” | 0.86 |

No creator recording, ratification, release approval, upload or social posting
was performed. Lesson 1 and Lesson 2 projects and media are unchanged.
<!-- V03A_MEDIA_RECORD_END -->

<a id="production-recipes"></a>
## Reproducible capture and scene recipes

The manifest and these recipes are the production source. All screenshots,
receipts, measured SVGs, diagnostic previews and media are generated under `build/`. They do
not alter earlier lessons or owner recordings. Run from the workspace’s Vidni
Nix shell with the pinned tools and `VIDNI_ASSISTANT=1` for every Vidni command.
The browser recipe uses the installed Playwright package and website bundle;
its local paths identify the actual dependencies used for this capture.

Save the following JavaScript to a temporary `.mjs` file and run it with Node.
It starts a temporary loopback server, verifies the real cases, writes captures
and their receipt, then closes the browser/server. It does not contact or modify
the public deployment.

```javascript
// Capture the real bundled Dioxus/WASM playground. No engine or result mocks.
import assert from 'node:assert/strict';
import {chromium} from '/home/dhilipsiva/projects/dhilipsiva/nibli-learn/node_modules/playwright/index.mjs';
import {createServer} from 'node:http';
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {extname,resolve} from 'node:path';
import {createHash} from 'node:crypto';
const project='/home/dhilipsiva/projects/dhilipsiva/vidni/examples/nibli-v03a-first-statements';
const site='/home/dhilipsiva/projects/dhilipsiva/dhilipsiva.dev/public';
const out=project+'/build/playground';await mkdir(out,{recursive:true});
const manifest=JSON.parse(await readFile(site+'/nibli-playground/build-manifest.json','utf8'));
const wasm=await readFile(site+'/nibli-playground/assets/nibli-ui_bg-dxha398699a91accdef.wasm');
const playgroundWasmSha256=createHash('sha256').update(wasm).digest('hex');
// The site build script's shared manifest hashes the separate guided-page module.
const guidedWasm=await readFile(site+'/nibli/wasm/nibli_wasm_bg.wasm');
assert.equal(createHash('sha256').update(guidedWasm).digest('hex'),manifest.wasm_sha256);
const mime={'.html':'text/html','.js':'text/javascript','.wasm':'application/wasm','.css':'text/css','.json':'application/json'};
const server=createServer(async(req,res)=>{try{let name=decodeURIComponent(new URL(req.url,'http://localhost').pathname);if(name.endsWith('/'))name+='index.html';const file=resolve(site,'.'+name);if(!file.startsWith(site+'/'))throw Error('path');const bytes=await readFile(file);res.writeHead(200,{'content-type':mime[extname(file)]||'application/octet-stream'}).end(bytes);}catch{res.writeHead(404).end();}});
await new Promise(r=>server.listen(0,'127.0.0.1',r));
const browser=await chromium.launch({executablePath:'/nix/store/fqriy9vihl23l4md56qgiks8z0d2xdlw-chromium-156.0.8078.79/bin/chromium',headless:true});
const url=`http://127.0.0.1:${server.address().port}/nibli-playground/`;
const baseline='member(ParcelA, PackedParcels).\nmember(ParcelB, LabeledParcels).';
const packing='member(ParcelA, PackedParcels).';const label='member(ParcelA, LabeledParcels).';
const checks=[];const captures=[];const pageErrors=[];let page;
const kr=async()=>{await page.locator('button.tab').filter({hasText:/nibli kr/i}).first().click();};
const back=async()=>{await page.locator('button.tab').filter({hasText:/back-translation/i}).first().click();};
async function fresh(kb=baseline){
 if(page)await page.close();
 page=await browser.newPage({viewport:{width:1440,height:1000},deviceScaleFactor:2});page.setDefaultTimeout(10000);page.on('pageerror',e=>pageErrors.push(String(e)));
 await page.goto(url,{waitUntil:'domcontentloaded'});await page.locator('#query-input').waitFor();await page.locator('.source-input').fill('');await kr();await page.locator('#kb-kr-editor').fill(kb);await page.locator('#query-input').fill(packing);
}
async function approve(){await back();await page.getByRole('button',{name:'Use reviewed encoding',exact:true}).click();}
async function query(q,want){let previous=await page.locator('#output-log .output-entry').count();if(await page.locator('#query-input').inputValue()!==q){await page.locator('#query-input').fill(q);await page.waitForFunction(()=>document.querySelectorAll('#output-log .output-entry').length===0);previous=0;}await page.locator('#query-input').press('Enter');await page.waitForFunction(n=>document.querySelectorAll('#output-log .output-entry').length===n,previous+1);const row=page.locator('#output-log .output-entry').last();await row.waitFor();assert.equal((await row.locator('.output-result').innerText()).trim(),want);assert.equal((await row.locator('.output-input').innerText()).trim(),q);return await row.innerText();}
async function shot(name){
 await page.locator('#query-input').blur();await page.screenshot({path:out+'/'+name+'.png'});
 const text=await page.locator('body').innerText();await writeFile(out+'/'+name+'.txt',text);
 const pieces={};
 for(const [label,selector,maxHeight] of [['custom','.app-header__examples',55],['tabs','.tabs',55],['history','#output-log',180],['profile','.output-profile',55],['kb','#kb-kr-editor',115],['review','.back-translation',160],['review-action','button.toolbar-btn',150],['claim','.query-bar',90],['result','#output-log .output-entry:last-child',180],['proof','.proof-tree',210],['error','.translate-error',190],['status','.kb-status-bar',110]]){
  let e=page.locator(selector).first();if(label==='kb')e=e.locator('..');if(label==='review-action')e=page.getByRole('button',{name:'Use reviewed encoding',exact:true});
  if(await e.count()&&await e.isVisible()){
   const b=await e.boundingBox();if(!b)continue;let crop={x:Math.max(0,b.x-3),y:Math.max(0,b.y-3),width:Math.min(b.width+6,1440-Math.max(0,b.x-3)),height:Math.min(b.height+6,maxHeight,1000-Math.max(0,b.y-3))};
   const file=name+'-'+label+'.png';await page.screenshot({path:out+'/'+file,clip:crop});const targets=await e.evaluate(root=>{
     const terms=['ParcelA','ParcelB','PackedParcels','LabeledParcels','Copy7','ReturnedCopies','member','set','TRUE','FALSE','[given]','Use reviewed encoding','2 asserted','3 asserted','0 asserted','unknown predicate','too many arguments','arity 2','expected'];
     const result={};for(const term of terms){const boxes=[];const walk=document.createTreeWalker(root,NodeFilter.SHOW_TEXT);let n;while(n=walk.nextNode()){const value=n.textContent||'';let start=value.toLowerCase().indexOf(term.toLowerCase());while(start>=0){const range=document.createRange();range.setStart(n,start);range.setEnd(n,start+term.length);for(const r of range.getClientRects()){if(r.width>0&&r.height>0)boxes.push({x:r.x,y:r.y,width:r.width,height:r.height});}start=value.toLowerCase().indexOf(term.toLowerCase(),start+term.length);}}result[term]=boxes;}return result;
    });pieces[label]={file,css:crop,targets};
  }
 }
 captures.push({name,text,pieces});console.log('captured',name);
}
try{
 await fresh();await shot('01-premises');
 await page.getByRole('button',{name:'Query',exact:true}).click();assert.match(await page.locator('body').innerText(),/Use reviewed encoding before querying/);assert.equal(await page.locator('#output-log .output-entry').count(),0);checks.push('unreviewed KB blocks evaluation');
 await back();await shot('02-review');await approve();await shot('03-reviewed');await kr();
 await query(packing,'TRUE');assert.match(await page.locator('.proof-tree').innerText(),/\[given\]/);await shot('04-true');
 const before=await page.locator('#kb-kr-editor').inputValue();await query(packing,'TRUE');assert.equal(await page.locator('#kb-kr-editor').inputValue(),before);assert.equal(await page.locator('#output-log .output-entry').count(),2);await shot('05-repeat');checks.push('packing TRUE twice with given support; KB unchanged');
 await query(label,'FALSE');assert.match(await page.locator('.proof-tree').innerText(),/not derivable|nothing known|no.*found/i);await shot('06-false');checks.push('missing label FALSE');
 const count=await page.locator('#output-log .output-entry').count();await page.locator('#kb-kr-editor').fill(baseline+'\n# edit');await page.locator('#kb-kr-editor').fill(baseline);await page.getByRole('button',{name:'Query',exact:true}).click();assert.equal(await page.locator('#output-log .output-entry').count(),0);assert.match(await page.locator('body').innerText(),/Use reviewed encoding before querying/);await shot('07-edit-review');checks.push('edit then revert requires fresh review');
 await approve();await kr();await query(packing,'TRUE');await shot('08-restored');
 await fresh('member(set: PackedParcels, member: ParcelA).');await approve();await kr();await query(packing,'TRUE');await query('member(member: ParcelA, set: PackedParcels).','TRUE');await query('member(set: PackedParcels, member: ParcelA).','TRUE');await shot('09-named');await query('member(PackedParcels, ParcelA).','FALSE');checks.push('named orders and positional form equivalent; reversed roles FALSE');
 await fresh();await approve();await kr();await query(label,'FALSE');await shot('10-before-supply');await page.locator('#kb-kr-editor').fill(baseline+'\n'+label);await approve();await kr();await query(label,'TRUE');assert.match(await page.locator('.proof-tree').innerText(),/\[given\]/);await shot('11-supplied');checks.push('supplying desired claim gives TRUE with given support');
 for(const [name,input,reason] of [['12-period','member(ParcelA, PackedParcels)','expected'],['13-word','packed(ParcelA).','unknown predicate'],['14-arity','member(ParcelA, PackedParcels, ParcelB).','too many arguments'],['15-whole-candidate',packing+'\npacked(ParcelB).','unknown predicate']]){
  await fresh(input);await approve();assert.match(await page.locator('body').innerText(),new RegExp(reason));assert.match(await page.locator('body').innerText(),/KB not loaded|candidate discarded/);assert.equal(await page.locator('#output-log .output-entry').count(),0);await shot(name);checks.push(name+': admission refused, zero verdicts');
 }
 await fresh('member(Copy7, ReturnedCopies).');await approve();await kr();await query('member(Copy7, ReturnedCopies).','TRUE');await shot('16-library');await query('member(member: Copy7, set: ReturnedCopies).','TRUE');checks.push('library positional/named TRUE');
 await fresh();await approve();await kr();await query(packing,'TRUE');await shot('17-final-starter');
 await fresh(packing);await approve();await kr();await query(packing,'TRUE');await shot('18-short');checks.push('independent Short: one supplied premise, TRUE with given support');
 assert.deepEqual(pageErrors,[]);
 const receipt={date:'2026-11-19',surface:'Real bundled Dioxus playground served on loopback; public deployment not contacted.',manifest,playgroundWasmSha256,browser:await browser.version(),checks,pageErrors,captures,allPassed:true};
 await writeFile(out+'/receipt.json',JSON.stringify(receipt,null,2)+'\n');console.log('PASS',checks.length,'browser checks;',captures.length,'captures');
}catch(e){if(page){await page.screenshot({path:out+'/failure.png'});await writeFile(out+'/failure.txt',await page.locator('body').innerText());}console.error('PAGE ERRORS',pageErrors);console.error(e);throw e;}
finally{await browser.close();await new Promise(r=>server.close(r));}
```

Save the following Python as `build/render_measured_scenes.py` and run it with
this project's path after capture. It generates clean SVGs and screenshot
measurement receipts. The manifest uses these SVGs directly; Vidni's pinned
Chromium draws them and resolves their semantic targets. This replaces the
retired flattening recipe used by the first draft. Focus targets outside a crop
fail generation, and missing or ambiguous measured elements fail capture.

```python
from pathlib import Path
from html import escape
import base64,json,re,struct,sys,textwrap,tomllib,hashlib
p=Path(sys.argv[1]).resolve();out=p/'build/quine-measured-assets';out.mkdir(parents=True,exist_ok=True)
lines=tomllib.loads((p/'vidni.toml').read_text())['lines']
receipt=json.loads((p/'build/playground/receipt.json').read_text());assert receipt['allPassed']
captures={x['name']:x for x in receipt['captures']}
BG='#0b0b10';CARD='#181822';INK='#f4f3ef';BODY='#c9c8c2';MUTED='#908f9a';EDGE='#3a3a4a';EMBER='#f2542d'
serial=0;focus_elements=[];targets={}
def identifier(prefix='node'):
 global serial
 serial+=1
 return f'{prefix}-{serial:03}'
def mark(element,text='',span=None):
 if span:targets[identifier('focus')]={'element':element,'span':{'text':span,'occurrence':1}}
 else:focus_elements.append(element)
def tx(x,y,s,size=36,color=INK,font='IBM Plex Sans',anchor='start',focus=False,span=None):
 ident=identifier('text')
 if focus:mark(ident,s,span)
 return f'<text id="{ident}" x="{x}" y="{y}" font-family="{font}" font-weight="700" font-size="{size}" fill="{color}" text-anchor="{anchor}">{escape(s)}</text>'
def rect(x,y,w,h,fill=CARD,stroke=EDGE,r=8):return f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{r}" fill="{fill}" stroke="{stroke}" stroke-width="2"/>'
def wrap(x,y,s,width=65,size=32,color=BODY):return ''.join(tx(x,y+i*size*1.3,l,size,color) for i,l in enumerate(textwrap.wrap(s,width)))
def heading(title,small=False,tag='Neuro-Symbolic Lesson 3'):
 x=44 if small else 76
 return tx(x,45 if small else 53,'// '+tag.upper(),19 if small else 22,EMBER,'IBM Plex Mono')+tx(x,102 if small else 126,title,42 if small else 56,INK,'Space Grotesk')
def foot(s,small=False):return tx(44 if small else 76,588 if small else 710,s,25 if small else 28,BODY)
def box(x,y,w,h,title,detail='',focus=False,small=False):
 b=rect(x,y,w,h,stroke=EDGE)+tx(x+24,y+49,title,30 if small else 36,INK,'Space Grotesk',focus=focus)
 if detail:b+=wrap(x+24,y+100,detail,int((w-48)/(17 if small else 18)),28 if small else 30)
 return b
def parcel(x,y,scale=.7,label='A',focus=False):
 b=f'<g transform="translate({x} {y}) scale({scale})">'
 b+=f'<path d="M0 70 L175 15 L310 78 L130 140 Z" fill="#908f9a" stroke="#c9c8c2" stroke-width="2"/>'
 b+=f'<path d="M0 70 L130 140 L130 310 L0 240 Z" fill="#2a2a38" stroke="#908f9a" stroke-width="2"/>'
 b+=f'<path d="M130 140 L310 78 L310 245 L130 310 Z" fill="#3a3a4a" stroke="#908f9a" stroke-width="2"/>'
 b+=f'<path d="M84 44 L118 33 L251 102 L217 114 Z" fill="#ff9a6f"/><path d="M217 114 L251 102 L251 160 L217 171 Z" fill="{EMBER}"/>'
 b+=tx(155,365,'Parcel '+label,42,INK,'IBM Plex Mono','middle',focus=focus)
 return b+'</g>'
def book(x,y,scale=1,focus=False):
 b=f'<g transform="translate({x} {y}) scale({scale})">'+rect(0,0,210,265,'#3a3a4a')+rect(16,14,190,235,'#2a2a38')+tx(110,134,'COPY',33,INK,'Space Grotesk','middle')+tx(110,195,'7',60,EMBER,'IBM Plex Mono','middle',focus=focus)
 return b+'</g>'
def note(x,y,w=580,label='A',message='packed',focus=False):
 b=rect(x,y,w,140)+tx(x+26,y+48,'Packing note' if message=='packed' else 'Label note',29,MUTED)+tx(x+26,y+104,'Parcel '+label+' is '+message+'.',36,focus=focus)
 return b

def code(x,y,mode='normal',focus='',size=54,library=False,prefix=False):
 a,b=('Copy7','ReturnedCopies') if library else ('ParcelA','PackedParcels')
 if mode=='label':b='LabeledParcels'
 if mode=='b-label':a,b='ParcelB','LabeledParcels'
 if mode=='swap':a,b=b,a
 toks=[]
 if prefix:toks.append(('? ','prefix',EMBER))
 if mode=='word':toks=[('packed','rel',EMBER),('(','open',MUTED),(a,'arg1',INK),(')','close',MUTED),('.','period',INK)]
 else:
  toks += [('member','rel',EMBER),('(','open',MUTED)]
  if mode=='named':toks += [('member: ','member-label',MUTED),(a,'arg1',INK),(', ','comma',MUTED),('set: ','set-label',MUTED),(b,'arg2',INK)]
  elif mode=='named-reorder':toks += [('set: ','set-label',MUTED),(b,'arg2',INK),(', ','comma',MUTED),('member: ','member-label',MUTED),(a,'arg1',INK)]
  else:toks += [(a,'arg1',INK),(', ','comma',MUTED),(b,'arg2',INK)]
  if mode=='extra':toks += [(', ','comma2',MUTED),('ParcelB','extra',INK)]
  toks += [(')','close',MUTED)]
  if mode!='missing-period':toks += [('.','period',INK)]
 bsvg=f'<text x="{x}" y="{y}" font-family="IBM Plex Mono" font-size="{size}" font-weight="700" xml:space="preserve">'
 keys=set(focus.split('+'))
 for value,key,color in toks:
  ident=identifier('token')
  active=key in keys or ('names' in keys and key in ['arg1','arg2']) or ('labels' in keys and key in ['member-label','set-label']) or ('brackets' in keys and key in ['open','close'])
  if active:mark(ident,value)
  if focus=='capital' and key=='arg1':mark(ident,value,'P')
  bsvg+=f'<tspan id="{ident}" fill="{color}">{escape(value)}</tspan>'
 bsvg+='</text>'
 if mode=='missing-period':bsvg+=tx(x,y+80,'Missing final period',30,EMBER,focus=True)
 return bsvg

def statement(mode='normal',focus='',title='Write the packing note',library=False):
 b=heading(title)+rect(76,205,1448,270)
 size=46 if mode.startswith('named') or mode=='extra' else 59
 b+=code(112,355,mode,focus,size,library)
 meaning='Copy 7 is on the returned-copies list.' if library else 'Parcel A is on the packed-parcels list.'
 if mode=='swap':meaning='Different roles: the list is placed where the parcel belongs.'
 if mode=='word':meaning='Attempted input: this predicate name is unsupported.'
 if mode=='extra':meaning='The member predicate has two places.'
 if mode=='missing-period':meaning='A period finishes the statement.'
 b+=box(76,520,1448,120,'Read it in plain English',small=True)+tx(101,618,meaning,34)
 return b+foot('One predicate · two places · names chosen for this example')

def shop(stage=0,focus=''):
 b=heading('Two parcels. Two records.' if stage!=5 else 'Writing your first statements')
 b+=parcel(250,180,.73,'A',focus=='a')+parcel(1030,180,.73,'B',focus=='b')
 b+=note(100,498,620,'A','packed',focus=='packed')+note(875,498,620,'B','labelled',focus=='labelled')
 return b+foot('Keep each note attached to the parcel it describes.')
def relation(focus='',stage=0):
 b=heading('Names and a relationship')+parcel(140,200,.65,'A',focus=='parcel')
 b+=box(650,205,800,220,'Packed-parcels list','Parcels with a packing note',focus=='list')+tx(685,375,'A',52,EMBER,'IBM Plex Mono')
 if stage>=1:b+=tx(100,535,'ParcelA',52,INK,'IBM Plex Mono',focus=focus in ['name','capital','names'],span='P' if focus=='capital' else None)+tx(650,535,'PackedParcels',52,INK,'IBM Plex Mono',focus=focus in ['list-name','names'])
 b+=tx(100,634,'Parcel A belongs to the packed-parcels list.',40,BODY)
 return b+foot('The names identify things; membership is stated separately.')
def signature(focus=''):
 b=heading('The supported predicate and its places')+rect(76,200,1448,115)
 b+=tx(110,281,'member(member, set)',52,INK,'IBM Plex Mono',focus=focus=='rel',span='member' if focus=='rel' else None)
 b+=box(76,370,680,205,'1 · member','The thing that belongs',focus in ['member','both'])
 b+=box(844,370,680,205,'2 · set','The collection it belongs to',focus in ['set','both'])
 return b+foot('Use the vocabulary’s declared places. Choose your own names.')
def two_records(focus=''):
 b=heading('The two supplied records')+rect(76,215,1448,145)+rect(76,405,1448,145)
 b+=code(112,309,'normal','names' if focus=='a' else '',50)+code(112,499,'b-label','names' if focus=='b' else '',50)
 return b+foot('A has a packing note. B has a label note.')

def capture(name,key,x,y,width,focus='',max_height=None,selection=None):
 c=captures[name]['pieces'].get(key)
 if not c:return ''
 data=(p/'build/playground'/c['file']).read_bytes();b64=base64.b64encode(data).decode();css=c['css'];cw,ch=css['width'],css['height']
 pw,ph=struct.unpack('>II',data[16:24])
 vx,vy,vw,vh=0,0,cw,ch
 if selection:
  selected=[c.get('targets',{}).get(term,[])[-1] for term in selection if c.get('targets',{}).get(term,[])]
  if selected:
   vx=min(t['x'] for t in selected)-css['x']-10;vy=min(t['y'] for t in selected)-css['y']-7
   vw=max(t['x']+t['width'] for t in selected)-css['x']-vx+10;vh=max(t['y']+t['height'] for t in selected)-css['y']-vy+7
 if selection==['TRUE']:vx-=22;vw+=22
 if max_height:vh=min(vh,max_height)
 height=width*vh/vw;ident=identifier('capture')
 all_targets={}
 for n,(term,boxes) in enumerate(c.get('targets',{}).items()):
  for j,target in enumerate(boxes):
   all_targets[f'term-{n}-{j}']=[[(target['x']-css['x'])*pw/cw,(target['y']-css['y'])*ph/ch,target['width']*pw/cw,target['height']*ph/ch]]
 receipt_name=f'{name}-{key}.targets.json'
 record={'schema':1,'image':'build/playground/'+c['file'],'sha256':hashlib.sha256(data).hexdigest(),'width':pw,'height':ph,'browser':receipt['browser'],'targets':all_targets}
 (out/receipt_name).write_text(json.dumps(record,indent=2)+'\n')
 for term in focus.split('+'):
  if not term:continue
  last=term.endswith(':last');term=term.removesuffix(':last')
  boxes=c.get('targets',{}).get(term,[])
  for j,target in enumerate(boxes):
   if last and j!=len(boxes)-1:continue
   bx=target['x']-css['x'];by=target['y']-css['y']
   if not (bx>=vx and by>=vy and bx+target['width']<=vx+vw and by+target['height']<=vy+vh):
    raise ValueError(f'Focused target {name}/{key}/{term}/{j} is outside its crop')
   n=list(c['targets']).index(term)
   targets[identifier('focus')]={'element':ident,'capture':{'receipt':'build/quine-measured-assets/'+receipt_name,'target':f'term-{n}-{j}'}}
 b=f'<svg x="{x}" y="{y}" width="{width}" height="{height}" viewBox="{vx} {vy} {vw} {vh}"><image id="{ident}" x="0" y="0" width="{cw}" height="{ch}" preserveAspectRatio="none" href="data:image/png;base64,{b64}"/></svg>'
 return b

def browser_scene(name,mode='both',focus='',title='Evaluate a claim',annotation=''):
 b=heading(title)
 if mode=='review':
  b+=tx(90,208,'Read the records back',30,MUTED)+capture(name,'review',90,232,1400,focus,max_height=138)
  b+=capture(name,'review-action',90,548,1400,focus)+capture(name,'status',90,625,1050)
 elif mode=='setup':
  b+=tx(90,205,'Custom input',30,MUTED)+capture(name,'custom',320,166,440)
  b+=tx(90,290,'Nibli KR · supplied records',34,INK)+capture(name,'kb',90,315,1400,focus)
 elif mode=='kb':
  b+=tx(90,219,'Premises · supplied records',34)+capture(name,'kb',90,246,1400,focus)
  b+=box(90,525,1400,100,annotation or 'The records remain available for evaluation.',small=True)
 elif mode=='claim':
  b+=tx(90,220,'Claim field · query interface',34)+capture(name,'claim',90,254,1400,focus)
  b+=capture(name,'result',90,430,1400,focus) if name not in ['01-premises'] else ''
  if annotation:b+=wrap(90,610,annotation,75,34)
 elif mode=='proof':
  b+=capture(name,'result',90,204,1400,'TRUE')+tx(90,365,'Supporting derivation · given premise',32,MUTED)
  b+=capture(name,'proof',90,385,1300,focus,max_height=146)
 elif mode=='repeat':
  b+=tx(90,207,'The same claim, evaluated again',34)+capture(name,'history',90,245,1400,'TRUE',max_height=125)
  b+=tx(90,559,'The two supplied records are unchanged.',34,BODY)
 elif mode=='error':
  b+=tx(90,205,'Input refused · no claim verdict',32,EMBER)+capture(name,'error',90,235,1400,focus,max_height=190)
  b+=capture(name,'status',90,565,1400)
 elif mode=='both':
  b+=tx(90,201,'Premises',32,MUTED)+capture(name,'kb',90,223,1400,focus,max_height=85)
  b+=tx(90,450,'Claim for evaluation',32,MUTED)+capture(name,'claim',90,475,1400,focus)
  b+=capture(name,'result',90,583,1400,focus)
 else:raise ValueError(mode)
 return b+foot('Premises supply records. Claims are evaluated against those records.')

def recall():
 b=heading('From the parcel to an evaluated claim')
 for x,title,body in [(76,'Parcel','The situation'),(450,'Record','What we wrote down'),(824,'Nibli KR','The language'),(1198,'Nibli','The engine')]:b+=box(x,260,325,285,title,body,False,True)
 return b+foot('Write the records, then evaluate claims against them.')
def physical(library=False):
 b=heading('The observation still needs evidence')
 b+=book(215,225,1,True) if library else parcel(190,197,1,'A',True)
 b+=box(740,225,755,295,'Record and observation','A supported statement does not inspect the physical object.')
 return b+foot('Review the representation. Check the observation in the world.')
def library_scene(stage=0):
 b=heading('The same structure at a library')+book(180,213,1,stage==1)+box(650,223,840,220,'Return note','Copy 7 was returned.',stage==2)
 return b+foot('ReturnedCopies names the collection of recorded returns.')
def host(focus=''):
 b=heading('An alternative interface · the host program')+tx(90,218,'Supply a premise',34,MUTED)+rect(76,243,1448,110)
 b+=code(110,318,'normal','period' if focus=='period' else ('rel+arg1+arg2' if focus=='premise' else ''),46)
 b+=tx(90,429,'Evaluate a claim',34,MUTED)+rect(76,455,1448,110)+code(110,531,'normal',focus,46,prefix=True)
 return b+tx(76,640,'Reference: commands · query interface · keywords · predication · tags (later)',25,MUTED)+foot('The playground uses separate inputs. Keep the KR period in both routes.')
def next_scene(stage=8):
 def marked(key,x,y,label,size=36,span=None):
  rendered=tx(x,y,label,size)
  element=re.search(r'id="([^"]+)"',rendered).group(1)
  targets[key]={'element':element}
  if span:targets[key]['span']={'text':span,'occurrence':1}
  return rendered
 b=heading('From parcel records to a conclusion',tag='Next · Neuro-Symbolic Lesson 4')
 b+=rect(76,220,650,390)+rect(844,220,650,180)+rect(844,410,650,200)
 b+=marked('records',106,280,'Supporting records',38)
 b+=marked('same-parcel',106,340,'Both must concern the same parcel',28,'same parcel')
 b+=marked('packing',106,420,'Packing confirmation',36)
 b+=marked('label',106,500,'Label confirmation',36)
 b+=marked('rule',874,280,'Rule',38)+wrap(874,332,'With both confirmations, the parcel meets the checklist.',38,28)
 b+=marked('claim',874,470,'Claim',38)+wrap(874,535,'Parcel A meets the dispatch checklist.',29,32)
 if stage==10:
  b+=marked('derived',106,662,'Derived from the rule and supporting records',32)
  return b+foot('The conclusion is not supplied as a premise.')
 b+=tx(106,662,'Next: write the rule, evaluate the claim, inspect its support.',32,BODY)
 return b+foot('The requirement and both records concern the same parcel.')
def short_scene(stage,focus=''):
 b=heading('Premise or claim',True)
 if stage=='shop':
  b+=parcel(120,165,.7,'A',focus=='parcel')+box(430,220,405,205,'Packing note','Parcel A is packed.',True,True)
 elif stage=='list':
  b+=parcel(100,175,.65,'A',False)+box(410,200,440,235,'Packed-parcels list','Parcel A',True,True)
 elif stage=='statement':
  b+=box(44,170,812,130,'Parcel A → packed-parcels list',small=True)
  b+=code(65,400,'normal',focus,40)+tx(44,501,'One predicate. Two argument places.',31,BODY)
 elif stage in ['premise','claim','boundary']:
  b+=rect(44,160,812,148)+tx(66,206,'Premise · supplied record',31,INK,focus=focus=='premise')+code(66,265,'normal','',35)
  b+=rect(44,347,812,148)+tx(66,394,'Claim · evaluate',31,INK,focus=focus=='claim')+code(66,454,'normal','',35)
 elif stage=='true':
  b+=tx(44,181,'Real playground result',28,MUTED)+capture('18-short','result',55,205,250,'TRUE',selection=['TRUE'])
  b+=tx(44,370,'Supporting premise',32)+capture('18-short','proof',44,400,812,'[given]',selection=['ParcelA','PackedParcels','[given]'])
 elif stage=='physical':
  b+=parcel(320,155,.8,'A',True)+tx(44,533,'The physical parcel still needs checking.',30,BODY)
 else:raise ValueError(stage)
 return b+foot('Neuro-Symbolic Lesson 3',True)

specs={};intent={}
for l in lines:
 serial=0;focus_elements=[];targets={}
 id=l['id'];sec=l['section'];n=int(id.rsplit('-',1)[1]);cut=l['only'][0];svg=''
 if cut=='short':
  anchor=id.rsplit('-',1)[0]
  if anchor=='v03a-short-note':spec=('short','shop','parcel') if n==1 else ('short','list','')
  elif anchor=='v03a-short-premise':spec=('short','statement',['','rel','names',''][n-1]) if n<4 else ('short','premise','premise')
  elif anchor=='v03a-short-claim':spec=('short','true','') if n==2 else ('short','claim','claim' if n==1 else 'premise')
  else:spec=('short','physical','') if n==2 else ('short','boundary','premise' if n==1 else '')
  svg=short_scene(spec[1],spec[2])
 elif sec=='shop':
  if n==7:spec=('recall',);svg=recall()
  elif n>=9:spec=('relation','list' if n==9 else 'parcel',0);svg=relation(spec[1],0)
  else:spec=('shop',n,{1:'',2:'packed',3:'labelled',4:'',5:'',6:'',8:''}.get(n,''));svg=shop(n,spec[2])
 elif sec=='build-statement':
  if n<=4:spec=('relation',{1:'capital',2:'list-name',3:'names',4:'list-name'}[n],1);svg=relation(spec[1],1)
  elif n<=8:spec=('signature',{5:'rel',6:'both',7:'set',8:'rel'}[n]);svg=signature(spec[1])
  else:focus={9:'',10:'rel',11:'brackets',12:'names',13:'names',14:'comma',15:'period',16:'names'}[n];spec=('statement','normal',focus);svg=statement('normal',focus)
 elif sec=='named-places':
  if n in [1,2,3]:mode='swap' if n>1 else 'normal';focus='names'
  elif n>=11 and n<=14:mode='named-reorder';focus='labels' if n!=13 else 'names'
  elif n==15:mode='normal';focus=''
  else:mode='named';focus={4:'labels',5:'labels',6:'rel+member-label',7:'rel',8:'member-label',9:'set-label',10:'labels'}[n]
  spec=('statement',mode,focus);svg=statement(mode,focus,'Keep the roles clear')
 elif sec=='supply-premises':
  if n<3:spec=('records','b' if n==1 else '');svg=two_records(spec[1])
  elif n<=5:spec=('browser','01-premises','setup' if n==3 else 'kb','','Supply the premises');svg=browser_scene(*spec[1:])
  elif n in [6,7,8,11]:name='03-reviewed' if n==11 else '02-review';focus='ParcelA+ParcelB' if n==7 else ('Use reviewed encoding' if n==8 else '');spec=('browser',name,'review',focus,'Review the written records');svg=browser_scene(*spec[1:])
  elif n==9:spec=('browser','07-edit-review','error','','An edit needs a new review');svg=browser_scene(*spec[1:])
  elif n==10:spec=('profile',);svg=heading('Use the same reasoning settings')+box(76,230,1448,270,'Playground defaults','Keep the settings consistent across these examples.')+capture('04-true','profile',100,555,640)+foot('The same settings are used throughout this lesson.')
  else:spec=('physical',);svg=physical()
 elif sec=='evaluate-claims':
  mapping={1:('01-premises','claim','','The separate claim field'),2:('01-premises','claim','ParcelA+PackedParcels','Evaluate the packing claim'),3:('04-true','claim','TRUE','The packing claim is supported'),4:('04-true','proof','[given]','Inspect the supporting premise'),5:('04-true','proof','[given]','This statement was supplied'),6:('05-repeat','repeat','','Evaluate the same claim again'),7:('05-repeat','kb','','The records are unchanged'),8:('05-repeat','kb','','Evaluation adds no premise'),9:('06-false','claim','LabeledParcels','Change only the list name'),10:('06-false','claim','FALSE','The label claim is unsupported'),11:('06-false','claim','FALSE','Not derived from these records'),12:('06-false','both','ParcelB+ParcelA','The label note concerns B'),14:('04-true','proof','ParcelA','A lowercase printed name')}
  if n==13:spec=('physical',);svg=physical()
  else:spec=('browser',*mapping[n]);svg=browser_scene(*mapping[n],annotation='This does not establish that the physical parcel has no label.' if n==11 else '')
 elif sec=='supplied-conclusion':
  if n==4:spec=('physical',);svg=physical()
  else:
   mapping={1:('10-before-supply','claim','FALSE','Separate comparison · changed premises'),2:('11-supplied','kb','LabeledParcels:last','Add the desired claim as a premise'),3:('11-supplied','proof','[given]','TRUE because it was supplied'),5:('11-supplied','kb','LabeledParcels:last','The starting information changed'),6:('08-restored','kb','','Restore the original two records'),7:('08-restored','both','','Two inputs with different jobs')}
   spec=('browser',*mapping[n]);svg=browser_scene(*mapping[n])
 elif sec=='refusals':
  if n in [2,3,4,7,9,11]:mode={2:'missing-period',3:'normal',4:'word',7:'normal',9:'extra',11:'normal'}[n];focus={2:'',3:'period',4:'rel',7:'rel',9:'extra',11:'names'}[n];spec=('statement',mode,focus);svg=statement(mode,focus,'Input and repair')
  elif n==8:spec=('signature','both');svg=signature('both')
  else:
   mapping={1:('12-period','error','expected','A refusal happens before evaluation'),5:('13-word','error','unknown predicate','Use a supported predicate'),6:('13-word','error','unknown predicate','The unsupported word is refused'),9:('14-arity','error','too many arguments','An extra argument is refused'),10:('14-arity','error','arity 2','The predicate has two places'),12:('14-arity','error','','A refusal is not a FALSE verdict'),13:('15-whole-candidate','error','','The whole candidate is refused'),14:('17-final-starter','kb','','Restore the valid records')}
   spec=('browser',*mapping[n]);svg=browser_scene(*mapping[n])
 elif sec=='library':
  if n<=2:spec=('library',n);svg=library_scene(n)
  elif n in [3,4,7]:mode='named' if n==4 else 'normal';spec=('library-statement',mode);svg=statement(mode,'labels' if n==4 else 'names','A library return',True)
  else:spec=('browser','16-library','kb' if n==5 else 'proof','' if n==5 else '[given]','A fresh library example');svg=browser_scene(*spec[1:])
 elif sec=='host-and-next':
  if n in [1,2,3,5]:focus={1:'',2:'prefix',3:'premise',5:'period'}[n];spec=('host',focus);svg=host(focus)
  elif n==4:spec=('browser','17-final-starter','both','','The playground uses separate inputs');svg=browser_scene(*spec[1:])
  elif n==6:spec=('undecided',);svg=heading('Some evaluations can remain undecided')+box(76,260,1448,240,'Later in the course','We will explain outcomes where evaluation stops without deciding.')+foot('Today’s examples: supplied premises and evaluated claims.')
  elif n==7:spec=('recall',);svg=recall()
  else:spec=('next',n);svg=next_scene(n)
 else:raise ValueError(sec)
 if focus_elements:targets['focus']={'elements':list(dict.fromkeys(focus_elements))}
 intent[id]=targets.copy();targets['canvas']={'element':'scene-canvas'}
 specs[id]={'recipe':spec,'targets':targets}
 w,h=(900,620) if cut=='short' else (1600,760)
 (out/(id+'.svg')).write_text(f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">'+rect(0,0,w,h,BG,BG,0).replace('<rect ','<rect id="scene-canvas" ',1)+svg+'</svg>')
(p/'build/measured-scene-plan.json').write_text(json.dumps(specs,indent=2)+'\n')
print('Generated',len(specs),'QUINE scenes with real captured UI crops.')

(p/'build/emphasis-intents.json').write_text(json.dumps(intent,indent=2)+'\n')
```

Then run:

```sh
VIDNI_ASSISTANT=1 vidni validate examples/nibli-v03a-first-statements
VIDNI_ASSISTANT=1 vidni cues propose examples/nibli-v03a-first-statements --voice draft --preview
VIDNI_ASSISTANT=1 vidni cues status examples/nibli-v03a-first-statements --voice draft
# After the owner has verified the cues:
VIDNI_ASSISTANT=1 vidni draft examples/nibli-v03a-first-statements --offline
VIDNI_ASSISTANT=1 vidni hear examples/nibli-v03a-first-statements --draft
```


### Full-playground capture and camera recipes

Run these after generating the unchanged base illustrations above. Save the
first block as `build/capture_playground_zoom.mjs` and execute it with Node in
the pinned Nix shell. It waits for loaded styles/fonts, exercises the real
bundled engine and writes full screenshots with matching DOM measurements.

```javascript
// Capture the real bundled Dioxus/WASM playground. No engine or result mocks.
import assert from 'node:assert/strict';
import {chromium} from '/home/dhilipsiva/projects/dhilipsiva/nibli-learn/node_modules/playwright/index.mjs';
import {createServer} from 'node:http';
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {extname,resolve} from 'node:path';
import {createHash} from 'node:crypto';
const project='/home/dhilipsiva/projects/dhilipsiva/vidni/examples/nibli-v03a-first-statements';
const site='/home/dhilipsiva/projects/dhilipsiva/dhilipsiva.dev/public';
const out=project+'/build/playground-zoom';await mkdir(out,{recursive:true});
const manifest=JSON.parse(await readFile(site+'/nibli-playground/build-manifest.json','utf8'));
const wasm=await readFile(site+'/nibli-playground/assets/nibli-ui_bg-dxha398699a91accdef.wasm');
const playgroundWasmSha256=createHash('sha256').update(wasm).digest('hex');
// The site build script's shared manifest hashes the separate guided-page module.
const guidedWasm=await readFile(site+'/nibli/wasm/nibli_wasm_bg.wasm');
assert.equal(createHash('sha256').update(guidedWasm).digest('hex'),manifest.wasm_sha256);
const mime={'.html':'text/html','.js':'text/javascript','.wasm':'application/wasm','.css':'text/css','.json':'application/json'};
const server=createServer(async(req,res)=>{try{let name=decodeURIComponent(new URL(req.url,'http://localhost').pathname);if(name.endsWith('/'))name+='index.html';const file=resolve(site,'.'+name);if(!file.startsWith(site+'/'))throw Error('path');const bytes=await readFile(file);res.writeHead(200,{'content-type':mime[extname(file)]||'application/octet-stream'}).end(bytes);}catch{res.writeHead(404).end();}});
await new Promise(r=>server.listen(0,'127.0.0.1',r));
const browser=await chromium.launch({executablePath:'/nix/store/fqriy9vihl23l4md56qgiks8z0d2xdlw-chromium-156.0.8078.79/bin/chromium',headless:true});
const url=`http://127.0.0.1:${server.address().port}/nibli-playground/`;
const baseline='member(ParcelA, PackedParcels).\nmember(ParcelB, LabeledParcels).';
const packing='member(ParcelA, PackedParcels).';const label='member(ParcelA, LabeledParcels).';
assert.ok(process.env.FONTCONFIG_FILE,'Run in the pinned Vidni Nix shell');
const fontconfigSha256=createHash('sha256').update(await readFile(process.env.FONTCONFIG_FILE)).digest('hex');
const blockedOrigins=new Set();
const checks=[];const captures=[];const pageErrors=[];let page;
const kr=async()=>{await page.locator('button.tab').filter({hasText:/nibli kr/i}).first().click();};
const back=async()=>{await page.locator('button.tab').filter({hasText:/back-translation/i}).first().click();};
async function fresh(kb=baseline){
 if(page)await page.close();
 page=await browser.newPage({viewport:{width:1280,height:720},deviceScaleFactor:3,reducedMotion:'reduce'});page.setDefaultTimeout(10000);page.on('pageerror',e=>pageErrors.push(String(e)));
 await page.route('**/*',async route=>{const origin=new URL(route.request().url()).origin;if(origin===new URL(url).origin)await route.continue();else{blockedOrigins.add(origin);await route.abort();}});
 await page.goto(url,{waitUntil:'domcontentloaded'});await page.locator('#query-input').waitFor();await page.waitForFunction(()=>{const e=document.querySelector('.main-row .col-tabs');return e&&getComputedStyle(e).borderTopLeftRadius!=='0px'&&[...document.querySelectorAll('link[rel=stylesheet]')].length>=2&&[...document.querySelectorAll('link[rel=stylesheet]')].every(l=>l.sheet);});await page.evaluate(()=>document.fonts.ready);await page.locator('.source-input').fill('');await kr();await page.locator('#kb-kr-editor').fill(kb);await page.locator('#query-input').fill(packing);
}
async function approve(){await back();await page.getByRole('button',{name:'Use reviewed encoding',exact:true}).click();}
async function query(q,want){let previous=await page.locator('#output-log .output-entry').count();if(await page.locator('#query-input').inputValue()!==q){await page.locator('#query-input').fill(q);await page.waitForFunction(()=>document.querySelectorAll('#output-log .output-entry').length===0);previous=0;}await page.locator('#query-input').press('Enter');await page.waitForFunction(n=>document.querySelectorAll('#output-log .output-entry').length===n,previous+1);const row=page.locator('#output-log .output-entry').last();await row.waitFor();assert.equal((await row.locator('.output-result').innerText()).trim(),want);assert.equal((await row.locator('.output-input').innerText()).trim(),q);return await row.innerText();}
async function shot(name){
 if(['06-false','10-before-supply'].includes(name)){const row=page.locator('.proof-tree > .proof-node > .proof-row[aria-expanded="true"]').first();if(await row.count())await row.click();assert.equal(await page.locator('.proof-tree > .proof-node > .proof-row').first().getAttribute('aria-expanded'),'false');}
 await page.mouse.move(0,0);
 await page.evaluate(()=>document.activeElement?.blur());
 await page.evaluate(async()=>{await document.fonts.ready;await Promise.all(document.getAnimations().map(a=>a.finished.catch(()=>{}))); await new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r)));});
 const regions={workspace:'.app-shell',header:'.app-header',custom:'.app-header__examples',kb:'.main-row .col-tabs', 'kb-tabs':'.main-row .tab-bar','kb-code':'#kb-kr-highlight',claim:'.query-row .col-tabs','claim-tabs':'.query-row .tab-bar','claim-field':'.query-bar','claim-code':'#query-input-hl',proof:'.main-row .col-proof','proof-content':'.proof-tree-container',log:'.output-log-container',history:'#output-log',result:'#output-log .output-entry:last-child',profile:'.output-profile',review:'.back-translation',status:'.kb-status-bar'};
 const scopes={'kb-text':'#kb-kr-highlight','claim-text':'#query-input-hl',result:'#output-log .output-entry:last-child',history:'#output-log','proof-line':'.proof-tree > .proof-node > .proof-row > .proof-label','proof-verdict':'.proof-tree > .proof-node > .proof-row','review-text':'.back-translation',status:'.kb-status-bar',error:'.translate-error'};
 const terms=['ParcelA','ParcelB','PackedParcels','LabeledParcels','Copy7','ReturnedCopies','member','set','TRUE','FALSE','[given]','Use reviewed encoding','2 asserted','3 asserted','0 asserted','unknown predicate','too many arguments','arity 2','expected'];
 const measured=await page.evaluate(({regions,scopes,terms})=>{
  const targets={},contents={};
  const rect=r=>[r.x*3,r.y*3,r.width*3,r.height*3];
  const visible=e=>!!e&&getComputedStyle(e).visibility!=='hidden'&&e.getBoundingClientRect().width>0&&e.getBoundingClientRect().height>0;
  const first=selector=>[...document.querySelectorAll(selector)].find(visible);
  for(const [key,selector] of Object.entries(regions)){const e=first(selector);if(e)targets[key]=[rect(e.getBoundingClientRect())];}
  const action=[...document.querySelectorAll('button')].find(e=>visible(e)&&e.textContent.trim()==='Use reviewed encoding');
  if(action)targets['review-action']=[rect(action.getBoundingClientRect())];
  const error=first('.main-row .translate-error')||first('.query-row .translate-error');
  if(error)targets.error=[rect(error.getBoundingClientRect())];
  for(const [scope,selector] of Object.entries(scopes)){
   const root=scope==='error'?error:first(selector);if(!root)continue;
   const nodes=[];let text='';const walker=document.createTreeWalker(root,NodeFilter.SHOW_TEXT);
   while(walker.nextNode()){nodes.push([walker.currentNode,text.length]);text+=walker.currentNode.textContent;}
   contents[scope]=text;
   const ink=[];for(const [node] of nodes){if(!node.textContent.trim())continue;const r=document.createRange();r.selectNodeContents(node);ink.push(...[...r.getClientRects()].filter(b=>b.width>0&&b.height>0).map(rect));}if(ink.length)targets[scope+'-ink']=ink;
   for(const term of terms){let count=0;for(let at=0;(at=text.toLowerCase().indexOf(term.toLowerCase(),at))>=0;at+=term.length){
    const boxes=[];for(const [node,start] of nodes){const lo=Math.max(0,at-start),hi=Math.min(node.length,at+term.length-start);if(lo>=hi)continue;const range=document.createRange();range.setStart(node,lo);range.setEnd(node,hi);boxes.push(...[...range.getClientRects()].filter(r=>r.width>0&&r.height>0).map(rect));}
    if(boxes.length)targets[scope+'/'+term+'/'+(++count)]=boxes;
   }}
  }
  return {targets,contents};
 },{regions,scopes,terms});
 const file=name+'.png';await page.screenshot({path:out+'/'+file});
 const bytes=await readFile(out+'/'+file);const text=await page.locator('body').innerText();
 await writeFile(out+'/'+name+'.txt',text);
 const receipt={schema:1,image:'build/playground-zoom/'+file,sha256:createHash('sha256').update(bytes).digest('hex'),width:3840,height:2160,browser:await browser.version(),targets:measured.targets};
 await writeFile(out+'/'+name+'.targets.json',JSON.stringify(receipt,null,2)+'\n');
 captures.push({name,text,file,contents:measured.contents,receipt:name+'.targets.json'});console.log('captured',name,Object.keys(measured.targets).length,'measured targets');
}
try{
 await fresh();await shot('01-premises');
 await page.getByRole('button',{name:'Query',exact:true}).click();assert.match(await page.locator('body').innerText(),/Use reviewed encoding before querying/);assert.equal(await page.locator('#output-log .output-entry').count(),0);checks.push('unreviewed KB blocks evaluation');
 await fresh();await back();await shot('02-review');await approve();await shot('03-reviewed');await kr();await shot('03-ready');
 await query(packing,'TRUE');assert.match(await page.locator('.proof-tree').innerText(),/\[given\]/);await shot('04-true');
 const before=await page.locator('#kb-kr-editor').inputValue();await query(packing,'TRUE');assert.equal(await page.locator('#kb-kr-editor').inputValue(),before);assert.equal(await page.locator('#output-log .output-entry').count(),2);await shot('05-repeat');checks.push('packing TRUE twice with given support; KB unchanged');
 await query(label,'FALSE');assert.match(await page.locator('.proof-tree').innerText(),/not derivable|nothing known|no.*found/i);await shot('06-false');checks.push('missing label FALSE');
 const count=await page.locator('#output-log .output-entry').count();await page.locator('#kb-kr-editor').fill(baseline+'\n# edit');await page.locator('#kb-kr-editor').fill(baseline);await page.getByRole('button',{name:'Query',exact:true}).click();assert.equal(await page.locator('#output-log .output-entry').count(),0);assert.match(await page.locator('body').innerText(),/Use reviewed encoding before querying/);await shot('07-edit-review');checks.push('edit then revert requires fresh review');
 await approve();await kr();await query(packing,'TRUE');await shot('08-restored');
 await fresh('member(set: PackedParcels, member: ParcelA).');await approve();await kr();await query(packing,'TRUE');await query('member(member: ParcelA, set: PackedParcels).','TRUE');await query('member(set: PackedParcels, member: ParcelA).','TRUE');await shot('09-named');await query('member(PackedParcels, ParcelA).','FALSE');checks.push('named orders and positional form equivalent; reversed roles FALSE');
 await fresh();await approve();await kr();await query(label,'FALSE');await shot('10-before-supply');await page.locator('#kb-kr-editor').fill(baseline+'\n'+label);await approve();await kr();await query(label,'TRUE');assert.match(await page.locator('.proof-tree').innerText(),/\[given\]/);await shot('11-supplied');checks.push('supplying desired claim gives TRUE with given support');
 for(const [name,input,reason] of [['12-period','member(ParcelA, PackedParcels)','expected'],['13-word','packed(ParcelA).','unknown predicate'],['14-arity','member(ParcelA, PackedParcels, ParcelB).','too many arguments'],['15-whole-candidate',packing+'\npacked(ParcelB).','unknown predicate']]){
  await fresh(input);await approve();assert.match(await page.locator('body').innerText(),new RegExp(reason));assert.match(await page.locator('body').innerText(),/KB not loaded|candidate discarded/);assert.equal(await page.locator('#output-log .output-entry').count(),0);await shot(name);checks.push(name+': admission refused, zero verdicts');
 }
 await fresh('member(Copy7, ReturnedCopies).');await approve();await kr();await query('member(Copy7, ReturnedCopies).','TRUE');await shot('16-library');await query('member(member: Copy7, set: ReturnedCopies).','TRUE');checks.push('library positional/named TRUE');
 await fresh();await approve();await kr();await query(packing,'TRUE');await shot('17-final-starter');
 await fresh(packing);await approve();await kr();await shot('18-short-ready');await query(packing,'TRUE');await shot('18-short');checks.push('independent Short: one supplied premise, TRUE with given support');
 assert.deepEqual(pageErrors,[]);
 const receipt={date:'2026-11-19',surface:'Real bundled Dioxus playground served on loopback; public deployment not contacted.',manifest,playgroundWasmSha256,fontconfigSha256,blockedRemoteOrigins:[...blockedOrigins],viewport:[1280,720],scale:3,browser:await browser.version(),checks,pageErrors,captures,allPassed:true};
 await writeFile(out+'/receipt.json',JSON.stringify(receipt,null,2)+'\n');console.log('PASS',checks.length,'browser checks;',captures.length,'captures');
}catch(e){if(page){await page.screenshot({path:out+'/failure.png'});await writeFile(out+'/failure.txt',await page.locator('body').innerText());}console.error('PAGE ERRORS',pageErrors);console.error(e);throw e;}
finally{await browser.close();await new Promise(r=>server.close(r));}
```

Save the following as `build/render_playground_zoom.py` and run it with Python.
It changes only the affected scene/camera/emphasis declarations in `vidni.toml`
and generates the full-screenshot SVGs under `build/`. All narrated wording,
line IDs, existing illustration scenes and voice settings stay intact.

```python
from pathlib import Path
import base64,hashlib,json,re,tomllib
p=Path('/home/dhilipsiva/projects/dhilipsiva/vidni/examples/nibli-v03a-first-statements')
source=(p/'vidni.toml').read_text().split('# Readable highlight continuation targets.')[0];manifest=tomllib.loads(source)
out=p/'build/playground-zoom-scenes';out.mkdir(exist_ok=True)
receipt=json.loads((p/'build/playground-zoom/receipt.json').read_text());assert receipt['allPassed']
lines={l['id']:l for l in manifest['lines']}
plans={}
def put(id,state,view,focus=None,portrait=None):
 plans[id]={'state':state,'camera':view,'portrait':portrait or view,'focus':focus}
def seq(section,n,state,view,focus=None):put(f'v03a-{section}-{n:02}',state,view,focus)
# Each camera frames actual parts of the unmodified browser layout.
views={
 'workspace':['workspace'],
 'premises':['kb-tabs','kb-text-ink'],
 'premise-detail':['kb-text-ink'],
 'review':['kb-tabs','review-action','review-text-ink'],
 'review-action':['review-action','review-text-ink'],
 'reviewed':['kb-tabs','review-action','status','review-text-ink'],
 'claim':['claim-tabs','claim-field'],
 'claim-detail':['claim-text-ink'],
 'claim-result':['claim-field','result'],
 'result':['result'],
 'proof':['proof-content'],
 'proof-detail':['proof-verdict-ink'],
 'proof-support':['proof-line/PackedParcels/1','proof-line/[given]/1','proof-verdict/TRUE/1'],
 'repeat':['history/TRUE/1','history/TRUE/2','result'],
 'profile':['profile'],
 'inputs':['kb-tabs','kb-text-ink','claim-tabs','claim-field'],
 'error':['error','status'],
 'edit-error':['error','claim-field'],
}
seq('supply-premises',3,'01-premises','workspace',[])
for n in [4,5]:seq('supply-premises',n,'01-premises','premises',[])
seq('supply-premises',6,'02-review','review',[])
seq('supply-premises',7,'02-review','review',['review-text/ParcelA/1','review-text/ParcelB/1'])
seq('supply-premises',8,'02-review','review-action',['review-action'])
seq('supply-premises',9,'07-edit-review','edit-error',[])
seq('supply-premises',10,'04-true','profile',[])
seq('supply-premises',11,'03-reviewed','reviewed',[])
seq('evaluate-claims',1,'03-ready','workspace',[])
seq('evaluate-claims',2,'03-ready','claim',['claim-text/ParcelA/1','claim-text/PackedParcels/1'])
seq('evaluate-claims',3,'04-true','result',['result/TRUE/1'])
for n in [4,5]:seq('evaluate-claims',n,'04-true','proof',['proof-line/[given]/1'])
seq('evaluate-claims',6,'05-repeat','repeat',['history/TRUE/1','history/TRUE/2'])
for n in [7,8]:seq('evaluate-claims',n,'05-repeat','premises',[])
seq('evaluate-claims',9,'06-false','claim',['claim-text/LabeledParcels/1'])
for n in [10,11]:seq('evaluate-claims',n,'06-false','result',['result/FALSE/1'])
seq('evaluate-claims',12,'06-false','inputs',['kb-text/ParcelB/1','claim-text/ParcelA/1'])
seq('evaluate-claims',14,'04-true','proof',['proof-line/ParcelA/1'])
seq('supplied-conclusion',1,'10-before-supply','claim-result',['result/FALSE/1'])
seq('supplied-conclusion',2,'11-supplied','premises',['kb-text/LabeledParcels/2'])
seq('supplied-conclusion',3,'11-supplied','proof',['proof-verdict/TRUE/1','proof-line/[given]/1'])
seq('supplied-conclusion',5,'11-supplied','premises',['kb-text/LabeledParcels/2'])
seq('supplied-conclusion',6,'08-restored','workspace',[])
seq('supplied-conclusion',7,'08-restored','inputs',[])
for n,state,focus in [(1,'12-period','expected'),(5,'13-word','unknown predicate'),(6,'13-word','unknown predicate'),(10,'14-arity','arity 2')]:
 seq('refusals',n,state,'error',['error/'+focus+'/1'])
for n,state in [(12,'14-arity'),(13,'15-whole-candidate')]:seq('refusals',n,state,'error',[])
seq('refusals',14,'17-final-starter','workspace',[])
if 'v03a-library-05' in lines:seq('library',5,'16-library','premises',[])
if 'v03a-library-06' in lines:seq('library',6,'16-library','proof',['proof-verdict/TRUE/1','proof-line/[given]/1'])
seq('host-and-next',4,'17-final-starter','inputs',[])
# Portrait uses the same full desktop, with readable local framing.
put('v03a-short-premise-04','18-short-ready','inputs',['kb-text-lines'])
put('v03a-short-claim-01','18-short-ready','claim',['claim-text-lines'],portrait='claim-detail')
put('v03a-short-claim-02','18-short','proof',['proof-verdict/TRUE/1','proof-line/[given]/1'],portrait='proof-detail')
put('v03a-short-claim-03','18-short','premises',['kb-text-lines'],portrait='premise-detail')
put('v03a-short-boundary-01','18-short','inputs',['kb-text-lines','claim-field'])
# Keep the last line's existing lesson title illustration.
# Join the source DOM's syntax-token fragments on each measured text row.
# Preserve the raw receipts; no text-width estimate or pixel-image inference.
joined={};provenance={}
for state in ['18-short-ready','18-short']:
 path=p/f'build/playground-zoom/{state}.targets.json';raw=json.loads(path.read_text())
 targets={}
 for scope in ['kb-text','claim-text']:
  rows={}
  for x,y,w,h in raw['targets'][scope+'-ink']:rows.setdefault((y,h),[]).append((x,w))
  targets[scope+'-lines']=[[min(x for x,w in xs),y,max(x+w for x,w in xs)-min(x for x,w in xs),h] for (y,h),xs in sorted(rows.items())]
 joined[state]=targets
 relative=f'build/playground-zoom/{state}.lines.targets.json'
 (p/relative).write_text(json.dumps({**raw,'targets':targets},indent=2)+'\n')
 provenance[state]={'source':str(path.relative_to(p)),'source_sha256':hashlib.sha256(path.read_bytes()).hexdigest(),'derived':relative,'method':'Union exact DOM Range fragments sharing measured y and height; one box per text row.'}
(p/'build/playground-zoom/line-target-provenance.json').write_text(json.dumps(provenance,indent=2)+'\n')
new_tables=[]
for id,plan in plans.items():
 state=plan['state'];raw=json.loads((p/f'build/playground-zoom/{state}.targets.json').read_text())
 targets={**raw['targets'],**joined.get(state,{})};needed=views[plan['camera']];portrait=views[plan['portrait']]
 targetmap={};cameras={}
 for camera,names in [('shot',needed),('portrait',portrait)]:
  keys=[]
  for name in names:
   assert name in targets,(id,name)
   key='region-'+re.sub('-+','-',re.sub('[^a-z0-9-]','-',name.lower())).strip('-')
   keys.append(key);targetmap[key]={'element':'screen','capture':{'receipt':f'build/playground-zoom/{state}.targets.json','target':name}}
  cameras[camera]={'union':keys}
 old=lines[id].get('emphasis',[]);focus=plan['focus'];cues=[]
 for i,name in enumerate(focus):
  assert name in targets,(id,name)
  # Keep existing cue IDs and narration phrases wherever possible.
  cue=dict(old[min(i,len(old)-1)]) if old else None
  assert cue,(id,'no narration cue for focus')
  if i>=len(old):cue['id']=f'focus-{i+1}'
  cue['target']='cue-'+cue['id'];cues.append(cue)
  receipt_name=state+('.lines' if name in joined.get(state,{}) else '')
  targetmap[cue['target']]={'element':'screen','capture':{'receipt':f'build/playground-zoom/{receipt_name}.targets.json','target':name}}
 plan['emphasis']=cues
 encoded=base64.b64encode((p/f'build/playground-zoom/{state}.png').read_bytes()).decode()
 svg=f'<svg xmlns="http://www.w3.org/2000/svg" width="1280" height="720" viewBox="0 0 1280 720"><image id="screen" x="0" y="0" width="1280" height="720" preserveAspectRatio="none" href="data:image/png;base64,{encoded}"/></svg>\n'
 (out/f'{id}.svg').write_text(svg)
 # Replace only the existing scene definition and its target/camera tables.
 source=re.sub(r'(?m)^\[scenes\.'+re.escape(id)+r'(?:\.[^\]]+)?\]\n.*?(?=^\[|\Z)','',source,flags=re.S)
 tables=f'[scenes.{id}]\nkind = "image"\nurl = "build/playground-zoom-scenes/{id}.svg"\nscale = 3\ndefault_camera = "shot"\n'
 for name,target in targetmap.items():
  tables+=f'\n[scenes.{id}.targets.{name}]\nelement = "screen"\ncapture = {{ receipt = {json.dumps(target["capture"]["receipt"])}, target = {json.dumps(target["capture"]["target"])} }}\n'
 for name,camera in cameras.items():tables+=f'\n[scenes.{id}.cameras.{name}]\nunion = '+json.dumps(camera['union'])+'\n'
 new_tables.append(tables)
 def change_line(match):
  block=match.group(0)
  if re.search(r'^id = '+re.escape(json.dumps(id))+'$',block,re.M):
   block=re.sub(r'^emphasis = .*\n','',block,flags=re.M)
   block=re.sub(r'^camera(?:_vertical)? = .*\n','',block,flags=re.M)
   def tomlcue(c):return '{ '+', '.join(k+' = '+json.dumps(v) for k,v in c.items())+' }'
   extra=('emphasis = ['+', '.join(map(tomlcue,cues))+']\n' if cues else '')+'camera = "shot"\ncamera_vertical = "portrait"\n'
   block=block.replace('id = '+json.dumps(id)+'\n','id = '+json.dumps(id)+'\n'+extra,1)
  return block
 source=re.sub(r'(?m)^\[\[lines\]\]\n.*?(?=^\[|\Z)',change_line,source,flags=re.S)
source=source.replace('# Full-playground screenshots and measured camera targets.\n','').rstrip()+'\n\n# Full-playground screenshots and measured camera targets.\n\n'+'\n'.join(new_tables)
(p/'vidni.toml').write_text(source)
(p/'build/playground-zoom-shot-plan.json').write_text(json.dumps(plans,indent=2)+'\n')
print('Updated',len(plans),'playground scenes; preserved narration and line IDs.')
```

```sh
VIDNI_ASSISTANT=1 ./target/release/vidni validate examples/nibli-v03a-first-statements
VIDNI_ASSISTANT=1 VIDNI_VOICES="$VIDNI_PINNED_VOICES" ./target/release/vidni cues propose examples/nibli-v03a-first-statements --voice draft --preview
VIDNI_ASSISTANT=1 VIDNI_VOICES="$VIDNI_PINNED_VOICES" ./target/release/vidni cues preview examples/nibli-v03a-first-statements --voice draft
```

Unmatched recognition spans remain unset unless an advisory interval is supplied
from the exact audio or reviewed by the owner. Saved timing proposals never
constitute verification. Use the current native review indexes when checking
cue timing, rather than any historical clips linked above.


### Readable-highlight continuity recipe

After both scene recipes above, save this block as
`build/render_readable_holds.py` and run it with Python. It writes explicit
continuity targets from exact authored SVG elements or measured screenshot
scopes, retaining the original receipts. The renderer resolves their holds
against actual narration and camera visibility. Run `vidni validate` afterward.

```python
"""Author explicit continuity from unchanged source elements, never image guesses.
Run after the base illustration and full-playground recipes. The three-second
policy is resolved by Vidni against the actual audio and camera in each cut.
"""
from pathlib import Path
import copy,hashlib,json,re,tomllib,xml.etree.ElementTree as ET
p=Path('/home/dhilipsiva/projects/dhilipsiva/vidni/examples/nibli-v03a-first-statements')
s=(p/'vidni.toml').read_text().split('# Readable highlight continuation targets.')[0].rstrip()+'\n'
s=re.sub(r'^continuity = .*\n','',s,flags=re.M)
m=tomllib.loads(s);scenes=m['scenes'];lines={l['id']:l for l in m['lines']}
shots=json.loads((p/'build/playground-zoom-shot-plan.json').read_text())
contents={c['name']:c['contents'] for c in json.loads((p/'build/playground-zoom/receipt.json').read_text())['captures']}
roots={};parents={};lookup={}
def node_key(e,parent):
 # Ignore serial IDs only: exact text, roles, transforms and layout stay bound.
 def node(n):return [n.tag,{k:v for k,v in n.attrib.items() if k!='id'},n.text or '',[node(c) for c in n]]
 chain=[];a=parent.get(e)
 while a is not None:
  chain.append([a.tag,{k:v for k,v in a.attrib.items() if k!='id'}]);a=parent.get(a)
 return json.dumps([node(e),chain],sort_keys=True)
for id,scene in scenes.items():
 root=ET.fromstring((p/scene['url']).read_text());parent={c:n for n in root.iter() for c in n};nodes={n.attrib['id']:n for n in root.iter() if 'id' in n.attrib}
 roots[id]=nodes;parents[id]=parent;lookup[id]={}
 for ident,node in nodes.items():lookup[id].setdefault(node_key(node,parent),[]).append(ident)
def describe(id,target):
 if 'capture' in target:
  name=target['capture']['target'];state=shots[id]['state']
  scope=next((k for k in contents[state] if name.startswith(k+'/') or name.startswith(k+'-')),None)
  if scope is None:scope={'claim-field':'claim-text','review-action':'review-text','status':'status'}.get(name)
  if not scope:return None
  return ['capture',name,scope,contents[state].get(scope)]
 ids=target.get('elements') or [target.get('element')]
 if not all(e in roots[id] for e in ids):return None
 return ['svg',[node_key(roots[id][e],parents[id]) for e in ids],target.get('span')]
def remap(description,id):
 kind,items,extra,*tail=description
 if kind=='capture':
  if id not in shots:return None
  state=shots[id]['state']
  if contents[state].get(extra)!=tail[0]:return None
  for suffix in ['','.lines']:
   rel=f'build/playground-zoom/{state}{suffix}.targets.json';file=p/rel
   if file.exists() and items in json.loads(file.read_text())['targets']:
    return {'element':'screen','capture':{'receipt':rel,'target':items}}
  return None
 ids=[]
 for key in items:
  matches=lookup[id].get(key,[])
  if len(matches)!=1:return None
  ids.append(matches[0])
 result={'element':ids[0]} if len(ids)==1 else {'elements':ids}
 if extra:result['span']=extra
 return result
canonical={};added={};identities={};changed={};evidence=[]
def attach(id,description,spec,preferred=None):
 signature=json.dumps(description,sort_keys=True);key=(id,signature)
 if key in canonical:return canonical[key]
 identity='hold-'+hashlib.sha256(signature.encode()).hexdigest()[:16]
 name=preferred
 if name is None:
  name=next((n for n,t in scenes[id].get('targets',{}).items() if describe(id,t)==description),None)
 if name is None:
  name=identity;added[(id,name)]=spec
 canonical[key]=name;identities[(id,name)]=identity
 return name
for cut in m['cuts']:
 seq=[l for l in m['lines'] if cut['id'] in l.get('only',[cut['id']])]
 for i,line in enumerate(seq):
  id=line['scene'];cues=copy.deepcopy(line.get('emphasis',[]))
  for cue in cues:
   spec=scenes[id]['targets'][cue['target']];description=describe(id,spec)
   if description is None:continue
   cue['target']=attach(id,description,spec,cue['target']);continued=[]
   for nxt in seq[i+1:]:
    mapped=remap(description,nxt['scene'])
    if mapped is None:break
    attach(nxt['scene'],description,mapped);continued.append(nxt['id'])
   evidence.append({'line':line['id'],'cue':cue['id'],'continuity':identities[(id,cue['target'])],'available_in_following_lines':continued})
  if cues:changed[line['id']]=cues
for (id,name),identity in identities.items():
 if (id,name) in added:continue
 header=f'[scenes.{id}.targets.{name}]\n';assert header in s,(id,name)
 s=s.replace(header,header+'continuity = '+json.dumps(identity)+'\n',1)
def update_line(match):
 block=match.group();id=re.search(r'^id = "([^"]+)"',block,re.M).group(1)
 if id not in changed:return block
 value='['+', '.join('{ '+', '.join(k+' = '+json.dumps(v) for k,v in c.items())+' }' for c in changed[id])+']'
 return re.sub(r'^emphasis = .*$',lambda _: 'emphasis = '+value,block,flags=re.M)
s=re.sub(r'(?ms)^\[\[lines\]\]\n.*?(?=^\[|\Z)',update_line,s)
s+='\n# Readable highlight continuation targets.\n'
for (id,name),spec in added.items():
 s+=f'\n[scenes.{id}.targets.{name}]\ncontinuity = '+json.dumps(identities[(id,name)])+'\n'
 for k,v in spec.items():
  value='{ '+', '.join(k+' = '+json.dumps(v) for k,v in v.items())+' }' if isinstance(v,dict) else json.dumps(v)
  s+=k+' = '+value+'\n'
(p/'vidni.toml').write_text(s)
(p/'build/readable-hold-continuity.json').write_text(json.dumps(evidence,indent=2)+'\n')
print('Authored',len(identities),'measured identities for',len(evidence),'cues;',len(added),'continuation targets.')
```
