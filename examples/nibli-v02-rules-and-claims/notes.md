# Writing Rules and Evaluating Claims

**Neuro-Symbolic Lesson 4 · stable curriculum ID V02 · 2026-11-20**

This package follows the published Lesson 3 parcel handoff. Its one objective is
to write a reusable rule, evaluate a specific claim and follow its derivation.
The owner selected synthetic main and Short previews and a parcel-only narration.
The drawing transfer remains optional written practice. The shop and records are
original fictional examples, not observations of real parcels.

## Production status

Both synthetic diagnostic review videos are rendered and checked. The main has
1,357 scripted words and measures **8:47.937**; the standalone Short has 128 words
and measures **0:48.796**. They meet the 8–10 minute and 45–60 second targets;
the course's main-lesson ceiling remains 13 minutes, with no minimum.
The [delivery record](#synthetic-review) links the videos, captions and evidence.
Both were rendered again on 2026-11-20 after an independent review; the
[review revision](#review-revision) lists what changed.
The owner has since ratified both cuts and recorded all 77 lines, watched the
[recorded-voice review](#recorded-voice-review) copies and confirmed their
highlights and timing. The [final exports](#recorded-final) measure **9:02.906**
and **0:51.411**, and the [publishing and sharing copy](#publishing-copy) uses
their chapter file. The creator reported the [main lesson published](https://www.youtube.com/watch?v=GgVYSi_rROQ&list=PLAQKTXMK9CyY&index=1)
on 2026-11-20; the sharing copy now uses that exact URL and playlist context.

Use the existing Piper en_US-joe-medium voice at length_scale 1.04. Every Vidni
command uses VIDNI_ASSISTANT=1. Native measured emphasis has a 3.0-second minimum;
several highlights may overlap. Natural speech continues while a highlight holds.
No audience questions, prediction tasks or response gaps belong in either cut.
All drafts use QUINE and Neuro-Symbolic Lesson 4 branding.

The existing page.html and .gitignore were created by `vidni new` and are unused
but preserved. Generated graphics, source DOM measurement receipts, scripts and
diagnostic media live in build/. No ownership, ratification or cue-verification
lock is written by hand. Synthetic review videos are diagnostic previews; they
are not owner-approved exports or a substitute for review of recorded narration.

## Narration and storyboard

Read these paragraphs as connected prose before recording individual stable
lines. The source for the fictional situation is this authored scenario; the
technical claims are grounded in R01–R05 below and the fixture/browser receipts.
Illustrations explain the model; playground pictures and verdicts are real.

<a id="v02-shop"></a>
### The checklist for the same parcel

Back at our small shop, Parcel A has a packing record, and Parcel B has a label record.

The shop's dispatch checklist needs both confirmations for the same parcel.

A packing note about A and a label note about B do not satisfy that requirement for either parcel.

Welcome to Neuro-Symbolic Lesson 4. We're learning to make reasoning explicit, one step at a time.

Last time we wrote individual records and evaluated claims against them.

This time we'll write the shop's requirement once, as a rule, and follow a conclusion back to the records that support it.

That conclusion means the checklist is met according to our records. It does not send a parcel anywhere.

| Line | Picture / framing | Spoken emphasis → measured target |
|---|---|---|
| `v02-shop-01` | `shop` · full illustration | “Parcel A” → `a`; “Parcel B” → `b` |
| `v02-shop-02` | `shop` · full illustration | “same parcel” → `same` |
| `v02-shop-03` | `shop` · full illustration | Keep context readable. |
| `v02-shop-04` | `purpose` · full illustration | Keep context readable. |
| `v02-shop-05` | `purpose` · full illustration | “individual records” → `records` |
| `v02-shop-06` | `purpose` · full illustration | “as a rule” → `rule`; “a conclusion” → `conclusion` |
| `v02-shop-07` | `purpose` · full illustration | “according to our records” → `limit` |

<a id="v02-plain-rule"></a>
### Say the rule in ordinary language

The requirement starts with any parcel, without choosing A or B in advance.

If that parcel has a packing confirmation, and that parcel has a label confirmation, then that parcel meets the dispatch checklist.

Each time we say that parcel, we mean the same thing. We cannot take one confirmation from A and the other from B.

This is a reusable pattern. A particular record concerns one named parcel; the rule can apply whenever its conditions are met.

Our records already use two lists: packed parcels and labelled parcels. We'll represent the conclusion with a third list, Dispatch Checklist Met.

That third name is our label for parcels whose records satisfy this checklist. The name itself does not make anything qualify.

| Line | Picture / framing | Spoken emphasis → measured target |
|---|---|---|
| `v02-plain-rule-01` | `english-rule` · full illustration | “any parcel” → `every` |
| `v02-plain-rule-02` | `english-rule` · full illustration | “packing confirmation” → `condition-a`; “label confirmation” → `condition-b`; “meets the dispatch checklist” → `conclusion` |
| `v02-plain-rule-03` | `english-rule` · full illustration | “the same thing” → `same` |
| `v02-plain-rule-04` | `english-rule` · full illustration | Keep context readable. |
| `v02-plain-rule-05` | `lists` · full illustration | “third list” → `checklist` |
| `v02-plain-rule-06` | `lists` · full illustration | Keep context readable. |

<a id="v02-write-rule"></a>
### Write one rule with a shared variable

In Nibli KR, we give the placeholder a name: dollar parcel. The dollar sign marks a variable, and parcel reminds us what we're tracking.

The beginning reads all dollar parcel, followed by a colon. It says to consider each possible value for that variable.

This does not say that every parcel meets the checklist. The conditions that follow still have to hold.

The first condition uses our familiar member predicate. Its first argument is now the variable, instead of a particular parcel name.

The second argument is Packed Parcels. Together, those arguments say that this parcel is on the packed-parcels list.

Next comes the ampersand, read as and. Both conditions must hold for the rule to apply.

The second condition puts the same variable on the labelled-parcels list. We have changed the list, while keeping the parcel together.

The arrow is written with a hyphen and a greater-than sign. Here we read it as then: it leads to the conclusion.

The conclusion puts that parcel on Dispatch Checklist Met. A period finishes the complete rule.

Notice the same variable in all three argument positions: packing, label, and conclusion. The engine must use one value throughout.

We've written the shop's requirement, rather than a separate checklist result for each parcel. Now we can evaluate a particular claim.

This is one complete statement, wrapped here for reading. In the playground input, we keep that statement together.

| Line | Picture / framing | Spoken emphasis → measured target |
|---|---|---|
| `v02-write-rule-01` | `rule-1` · full illustration | “dollar parcel” → `var-binding` |
| `v02-write-rule-02` | `rule-1` · full illustration | “all dollar parcel” → `binder` |
| `v02-write-rule-03` | `rule-1` · full illustration | “every parcel” → `all` |
| `v02-write-rule-04` | `rule-2` · full illustration | “the variable” → `var-packed` |
| `v02-write-rule-05` | `rule-2` · full illustration | “Packed Parcels” → `packed-set` |
| `v02-write-rule-06` | `rule-3` · full illustration | “the ampersand” → `and` |
| `v02-write-rule-07` | `rule-3` · full illustration | “same variable” → `var-labeled`; “labelled-parcels list” → `labeled-set` |
| `v02-write-rule-08` | `rule-4` · full illustration | “The arrow” → `arrow` |
| `v02-write-rule-09` | `rule-4` · full illustration | “Dispatch Checklist Met” → `dispatch-set`; “A period” → `period` |
| `v02-write-rule-10` | `rule-4` · full illustration | “same variable” → `shared` |
| `v02-write-rule-11` | `rule-4` · full illustration | Keep context readable. |
| `v02-write-rule-12` | `rule-4` · full illustration | Keep context readable. |

<a id="v02-first-evaluation"></a>
### Supply the rule and evaluate the claim

Here is the playground with our two original records and the completed rule in the knowledge-base pane.

The rule goes with the premises. Rules are starting assumptions too: this is the checklist we have chosen for our example.

Before using it, we compare the back-translation with our intended requirement. Its repeated X stands for the same shared variable.

The reading must retain both conditions and the same parcel. We select Use reviewed encoding to make this checked version available for evaluation.

We're using the playground's default reasoning settings. The records and rule are now available for evaluation.

In the separate claim field, we write that Parcel A is a member of Dispatch Checklist Met. This evaluates A without adding that conclusion.

The result is FALSE. The packing condition holds for A, but its own label confirmation is missing from the records.

Here FALSE means the claim is not derivable from what we supplied. It does not establish that the physical parcel has no label.

B's label note cannot fill that gap for A. The shared variable keeps those two parcels separate.

| Line | Picture / framing | Spoken emphasis → measured target |
|---|---|---|
| `v02-first-evaluation-01` | `pg-02-rule` · wide | Keep context readable. |
| `v02-first-evaluation-02` | `pg-02-rule` · kb | “The rule” → `kb-rule` |
| `v02-first-evaluation-03` | `pg-03-review` · review | “back-translation” → `review` |
| `v02-first-evaluation-04` | `pg-03-review` · review | “Use reviewed encoding” → `review-action` |
| `v02-first-evaluation-05` | `pg-04-rule-ready` · wide | Keep context readable. |
| `v02-first-evaluation-06` | `pg-04-rule-ready` · claim | “separate claim field” → `claim` |
| `v02-first-evaluation-07` | `pg-05-missing-label` · result | “FALSE” → `false` |
| `v02-first-evaluation-08` | `pg-05-missing-label` · result | Keep context readable. |
| `v02-first-evaluation-09` | `shop` · full illustration | “for A” → `a`; “B's label note” → `b` |

<a id="v02-derivation"></a>
### Add a record and follow the derivation

Later, a colleague adds a label confirmation for Parcel A. Now both confirmations concern A. The information has changed.

We encode that new record as another premise, and review the changed knowledge base before using it.

The claim stays the same. The TRUE result now follows from the added record and the rule we already supplied.

We can now follow the support in the proof panel. Start with the conclusion: A meets the dispatch checklist.

Beside it, the answer cites the recorded rule. That is the requirement connecting the two conditions to this conclusion.

Underneath are the supporting records. One says A is on the packed-parcels list, and is marked given.

The other says A is on the labelled-parcels list, also given. Both records concern A, exactly as our rule requires.

Given means we supplied that record. The checklist conclusion was derived: the engine applied our rule to those two records.

That short path from records, through a rule, to the conclusion is a derivation. It makes the support visible.

Evaluating the claim again gives the same support. It does not add the conclusion to the premises we supplied.

| Line | Picture / framing | Spoken emphasis → measured target |
|---|---|---|
| `v02-derivation-01` | `new-label` · full illustration | “label confirmation” → `label` |
| `v02-derivation-02` | `pg-06-repaired-ready` · kb | “new record” → `a-label` |
| `v02-derivation-03` | `pg-07-derived` · result | “TRUE” → `true` |
| `v02-derivation-04` | `pg-07-derived` · proof | “the conclusion” → `derived` |
| `v02-derivation-05` | `pg-07-derived` · proof | “recorded rule” → `citation` |
| `v02-derivation-06` | `pg-07-derived` · proof | “packed-parcels list” → `packed` |
| `v02-derivation-07` | `pg-07-derived` · proof | “labelled-parcels list” → `labeled` |
| `v02-derivation-08` | `pg-07-derived` · proof | “was derived” → `derived` |
| `v02-derivation-09` | `pg-07-derived` · proof | Keep context readable. |
| `v02-derivation-10` | `pg-08-repeat` · inputs | Keep context readable. |

<a id="v02-controls"></a>
### Compare derived and given support

A small comparison shows what the rule contributes. We'll keep A's two confirmations, but remove the rule.

With both records still present, A’s checklist claim now returns FALSE. The engine has no supplied rule connecting those lists to checklist membership.

The two records are available information. The rule is what makes this particular conclusion follow from them.

For a second comparison, we supply the checklist conclusion itself as a premise. That is a different use of the same statement.

Now the verdict is TRUE, but the support is marked given. There is no rule citation connecting it to the two confirmations.

A TRUE verdict alone does not tell us whether we supplied a conclusion or derived it. Reading the support tells us which happened.

We remove that supplied conclusion and restore our rule. The answer once again derives checklist membership from A’s two records.

| Line | Picture / framing | Spoken emphasis → measured target |
|---|---|---|
| `v02-controls-01` | `compare-rule` · full illustration | Keep context readable. |
| `v02-controls-02` | `pg-09-no-rule` · result | “FALSE” → `false` |
| `v02-controls-03` | `pg-09-no-rule` · kb | Keep context readable. |
| `v02-controls-04` | `compare-given` · full illustration | “conclusion itself” → `point` |
| `v02-controls-05` | `pg-10-given` · proof | “marked given” → `given` |
| `v02-controls-06` | `pg-10-given` · proof | Keep context readable. |
| `v02-controls-07` | `pg-11-restored` · proof | “derives checklist membership” → `derived` |

<a id="v02-some"></a>
### Every case and at least one example

The word all made our rule general. It can apply to each matching parcel, without a new rule written just for that name.

A different claim can require only one qualifying example. We will write: some object meets the dispatch checklist.

First, we explicitly record Parcel A and Parcel B as objects. These are two additional premises, not facts inferred from their names.

Parcel is not a predicate in the supported vocabulary, so we use the predicate object. It has one argument: the thing we are recording. The phrase some object then considers those recorded things.

In the claim, some object takes the first argument position. Dispatch Checklist Met stays in the second position.

This means at least one recorded object belongs to that list. We are evaluating whether such an example is supported.

The result is TRUE because A qualifies. Its object record and the checklist derivation supply the support.

One supported example is enough for this claim. It does not say that every object qualifies, or that exactly one does.

| Line | Picture / framing | Spoken emphasis → measured target |
|---|---|---|
| `v02-some-01` | `objects` · full illustration | “each matching parcel” → `rule-meaning` |
| `v02-some-02` | `objects` · full illustration | “only one” → `some-meaning` |
| `v02-some-03` | `objects` · full illustration | “Parcel A” → `object-a`; “Parcel B” → `object-b` |
| `v02-some-04` | `objects` · full illustration | “predicate object” → `object-word`; “one argument” → `object-arg` |
| `v02-some-05` | `pg-12-objects` · claim | “some object” → `some`; “Dispatch Checklist Met” → `claim-set` |
| `v02-some-06` | `pg-12-objects` · claim | Keep context readable. |
| `v02-some-07` | `pg-13-some-one` · proof | “object record” → `object-leaf`; “checklist derivation” → `derived` |
| `v02-some-08` | `pg-13-some-one` · proof | Keep context readable. |

<a id="v02-parcel-b"></a>
### Apply the same rule to Parcel B

B still has only a label confirmation. Evaluating B's own checklist claim gives FALSE, because its packing record is missing.

Now a packing confirmation arrives for B as well. We add that record and review the changed input.

The same rule now derives B's checklist membership. Its proof uses B's packing and label records. Both confirmations concern B, so neither comes from A.

Both parcels now qualify. Evaluating the some-object claim still returns TRUE. At least one includes the case where both qualify.

In ordinary conversation, some can suggest not all. That extra suggestion is not part of this formal claim.

| Line | Picture / framing | Spoken emphasis → measured target |
|---|---|---|
| `v02-parcel-b-01` | `pg-14-b-missing` · result | “claim gives FALSE” → `false` |
| `v02-parcel-b-02` | `b-record` · full illustration | “packing confirmation” → `packing` |
| `v02-parcel-b-03` | `pg-16-b-derived` · proof | “B's packing” → `packed`; “label records” → `labeled` |
| `v02-parcel-b-04` | `pg-17-some-both` · result | “TRUE” → `true` |
| `v02-parcel-b-05` | `b-record` · full illustration | “not all” → `some` |

<a id="v02-closing"></a>
### What the conclusion rests on

We began with two separate records and one everyday requirement. A shared variable kept each parcel together across both conditions.

Once the right records were present, the rule supported a conclusion we had never supplied as a premise. The derivation showed the connection.

And the some-object claim needed only one supported example, even when more than one was available.

This is the symbolic foundation of our neuro-symbolic course: make the starting information explicit, evaluate a claim, and inspect what supports it.

Next, we'll look at an argument place left empty, and why leaving it out differs from saying some value.

| Line | Picture / framing | Spoken emphasis → measured target |
|---|---|---|
| `v02-closing-01` | `closing` · full illustration | “shared variable” → `one` |
| `v02-closing-02` | `closing` · full illustration | “The derivation” → `two` |
| `v02-closing-03` | `closing` · full illustration | “one supported example” → `three` |
| `v02-closing-04` | `closing` · full illustration | Keep context readable. |
| `v02-closing-05` | `closing` · full illustration | “left empty” → `next` |

<a id="v02-short-story"></a>
### Standalone Short: Records Need a Rule

A shop's dispatch checklist needs packing and label confirmations for the same parcel.

Parcel A has a packing record and the checklist rule, but no label record.

For the claim that A meets the checklist, the verdict is FALSE. That means unsupported by these records, not proof that a label was never attached.

A new label confirmation arrives for A. We add that record as a premise; the checklist rule stays the same.

The TRUE result now follows from both records and the checklist rule.

Without the rule, FALSE returns. Both records remain, but the connection is missing.

A missing record and a missing rule are different gaps in what supports the conclusion.

Follow the support: records supply the information, and the rule connects it to the conclusion.

| Line | Picture / framing | Spoken emphasis → measured target |
|---|---|---|
| `v02-short-story-01` | `short-shop` · full illustration | “packing and label” → `label` |
| `v02-short-story-02` | `short-shop` · full illustration | “Parcel A” → `detail` |
| `v02-short-story-03` | `pg-18-short-missing` · result | “FALSE” → `false` |
| `v02-short-story-04` | `short-new` · full illustration | “for A” → `label` |
| `v02-short-story-05` | `pg-19-short-derived` · short-proof | “TRUE” → `true` |
| `v02-short-story-06` | `pg-20-short-no-rule` · result | “FALSE” → `false` |
| `v02-short-story-07` | `short-rule` · full illustration | “missing record” → `label`; “and a missing rule” → `detail` |
| `v02-short-story-08` | `short-close` · full illustration | “Follow the support” → `label` |

## Source receipts and representation limits

Public technical source revision: `1eab5d1f1d52d1c106145aa0cbc18205ff9b0902`.
No retired Lojban syntax or proposed schema declarations are taught.

| Receipt | Public source | Supports |
|---|---|---|
| R01 | [Language contract](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/NIBLI_KR.md), §§4–6 | Implemented universal variables, conjunction, implication/rule spelling and restricted existential `some object`. |
| R02 | [Current KR cookbook](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/mdbook/src/user/kr-cookbook.md) and [grammar](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/nibli-kr/src/nibli_kr.pest) | Statement period; named variable spelling; current source language. |
| R03 | [Committed predicates](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/nibli-lexicon/src/corpus/predicates.rs) | `member(member, set)` and unary `object(material)`; the vocabulary has no `parcel` predicate. Names of collections supply no built-in parcel semantics. |
| R04 | [Playground guide](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/mdbook/src/user/playground.md) and [review state](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/nibli-ui/src/review.rs) | Separate premises and claim; whole-candidate admission; review invalidated by edits; evaluation does not assert its claim. |
| R05 | [Verdict contract](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/README.md) and [proof renderer](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/nibli-render/src/proof.rs) | FALSE is not derivable, not proof of negation; rule citations and given support. |

The list `DispatchChecklistMet` records the model's conclusion, not an action or
physical inspection. `$parcel` is a chosen variable name, not a built-in parcel
type. `all` generalizes the rule; it does not assert either condition or make all
parcels qualify. Membership does not silently assert `object`, so A and B receive
explicit object records before the restricted existential claim. `parcel` is not
a vocabulary word: the host refuses `parcel(ParcelA).` and `some parcel` as an
unknown predicate, so the claim says `some object`. `some` means at
least one and is compatible with all qualifying. Each demonstrated derivation applies
one rule, first to A and later to B; OR/grouping, conditional comparisons, detailed quantifier
scope/import, rule chains and raw proof internals stay in their assigned lessons.

The browser bundle's shared manifest names revision `36739cca`; it hashes the
separate guided-page module. The actual playground WASM hash is recorded
separately in `build/playground/receipt.json`. Host and browser receipts do not
claim identical builds. Both execute the demonstrated examples independently.
The local bundle retains awkward readings such as “parcela members”. Narration
explains their meaning; no browser wording or output is hand-replaced. FALSE
shots leave internal witness trees collapsed and frame the verdict. The narrator
explains its qualification instead of presenting a fabricated engine message.

## Current coverage mapping

| Curriculum obligation | Main passage / evidence |
|---|---|
| CH02.02 a · mechanical derivation · T | derivation: conclusion, actual cited rule, two given leaves; repeated-evaluation fixture. |
| CH03.01 e · some compatible with all · T | some → parcel-b: A supplies one witness, then both qualify without changing the some verdict. |
| CH04.01 c · basic universal/existential commitments · T | write-rule and some: conditional scope of all, explicit object records, at least one supported witness. |
| CH07.01 a · conjunction · T | write-rule: both member conditions concern the shared parcel; initial near miss and successful repair. |
| CH07.03 a · rule · T | plain-rule through derivation; controls remove/restore the rule. |
| CH08.03 a · universals · T | all applies across matching cases; the same rule derives B when its own record arrives. |
| CH09.01 a,b · facts versus templates · T | plain-rule distinguishes a named record from a reusable pattern; controls separate their contributions. |
| CH09.03 a · condition · T | staged rule construction separates the two conditions from their conclusion. |
| APP-A.01 d · some X restriction · T | some: explicit unary object records; object-restriction fixture proves they are required. |
| APP-B.01 · browser interface reinforcement | first-evaluation and controlled comparisons; KB versus bare claim; fresh review after edits. Ownership remains V03a. |
| APP-B.02 d,f · compact answer parts · T | derivation and controls: verdict, one-line reason, source citation, given leaves. |
| APP-G.01 d · $var · D | write-rule: dollar sign, name, repeated argument positions, one value. |

Coverage family IDs, owners and tiers are retained. The Short reinforces missing
record versus missing rule with its own one-parcel setup; it is not a substitute
for any main-lesson coverage obligation. Document/media checks do not establish
beginner comprehension. Owner review and feedback from a learner remain pending.

## Companion reference

The baseline premises, including the new rule:

```text
member(ParcelA, PackedParcels).
member(ParcelB, LabeledParcels).
all $parcel: member($parcel, PackedParcels) & member($parcel, LabeledParcels) -> member($parcel, DispatchChecklistMet).
```

Evaluate `member(ParcelA, DispatchChecklistMet).` in the separate browser claim
field: FALSE. Add A's newly received `member(ParcelA, LabeledParcels).` record,
review, and evaluate again: TRUE with rule and two given records. The host
interface puts `? ` before a claim to select evaluation; that prefix is not part
of the browser input or the KR statement. Repeated evaluation adds no premise.

For the some-object claim, add `object(ParcelA).` and `object(ParcelB).`, then
evaluate `member(some object, DispatchChecklistMet).` A qualifies. B's own
checklist claim is still FALSE until `member(ParcelB, PackedParcels).` arrives.
The unchanged rule then derives B too. The some-object claim remains TRUE.

**Reference-only encoding error:**

```text
all $parcel, $other: member($parcel, PackedParcels) & member($other, LabeledParcels) -> member($parcel, DispatchChecklistMet).
```

With only A's packing and B's label, this different rule derives A's checklist
membership. The engine follows the supplied rule; the encoding fails to express
the intended same-parcel requirement. Use an isolated session, then reset and
restore one shared variable. This rule is not taught as a second main-video rule.

**Optional written transfer:** A class display requires each drawing's title and
artist-name confirmations. Represent these as `TitledDrawings` and
`ArtistNamedDrawings`, with conclusion list `DisplayChecklistMet`:

```text
member(Drawing1, TitledDrawings).
member(Drawing2, ArtistNamedDrawings).
all $drawing: member($drawing, TitledDrawings) & member($drawing, ArtistNamedDrawings) -> member($drawing, DisplayChecklistMet).
```

Drawing 1's display-checklist claim is FALSE until its own artist-name record is
supplied. This written worked transfer is fixture-checked and does not interrupt
the parcel-only video. Existing companion activity `found.rules-given` remains
optional; no companion application code is changed.

## Evidence and reproducibility

[Parcel fixture notes](../../../nibli/book/editorial/youtube-course/fixtures/V02/parcel-notes.md)
link the new sessions, expectations and both-profile captures. Expectations were
saved before the fixture run; the earlier planning probes are disclosed. The
historical camera/instrument fixtures and their notes are preserved unchanged.

`build/playground/receipt.json` records real browser checks, build hashes, source
text and measured screenshot targets. Each measurement binds exact PNG bytes;
no highlight rectangle is estimated from an image. Authored SVG targets bind
exact text elements. The native renderer applies camera transforms and checks
highlight clipping/caption collisions on every active frame.

The reproducible recipes are archived below because `build/` is generated and
ignored by Git. Restore each block to its named path inside this project. Use
the pinned Vidni Nix environment and run these commands from the Vidni repository:

```sh
export VIDNI_ASSISTANT=1
export VIDNI_VOICES="$VIDNI_PINNED_VOICES"
python3 examples/nibli-v02-rules-and-claims/build/run_fixtures.py
node examples/nibli-v02-rules-and-claims/build/capture_playground.mjs
python3 examples/nibli-v02-rules-and-claims/build/author.py
./target/release/vidni validate examples/nibli-v02-rules-and-claims
./target/release/vidni cues propose examples/nibli-v02-rules-and-claims --voice draft
cargo build --offline --manifest-path examples/nibli-v02-rules-and-claims/build/native/Cargo.toml --target-dir target
./target/debug/vidni-v02-audit dump
python3 examples/nibli-v02-rules-and-claims/build/align_dtw.py
python3 examples/nibli-v02-rules-and-claims/build/prepare_timings.py
./target/debug/vidni-v02-audit apply
./target/debug/vidni-v02-audit frames
./target/release/vidni cues preview examples/nibli-v02-rules-and-claims --voice draft
python3 examples/nibli-v02-rules-and-claims/build/check_delivery.py
```

Run generation only before owner revisions, or reconcile those revisions into
the recipe first. `author.py` writes both the project and the initial notes;
preserve this delivery record before any deliberate regeneration. Never overwrite
later wording edits or recordings by blindly replaying the writing-stage recipe.
The renderer owns voice locks. No recipe writes owner verification or approval.

<a id="synthetic-review"></a>
## Synthetic review delivery

**Synthetic diagnostic previews — 2026-11-20.** Both use QUINE, the existing
Piper `en_US-joe-medium` voice, measured emphasis and continuous narration.
The main stays with the parcels; the independent Short establishes its own shop
situation and explicitly identifies the claim before showing its verdict.

| Cut | Review video | Runtime | Captions |
|---|---|---|---|
| Main · 1920×1080 | [Watch the main review](build/cues/full/long-draft.review.mp4) | 8:47.937 | [SRT](build/cues/full/long-draft.review.en.srt) · [VTT](build/cues/full/long-draft.review.en.vtt) |
| Short · 1080×1920 | [Watch the Short review](build/cues/full/short-draft.review.mp4) | 0:48.796 | [SRT](build/cues/full/short-draft.review.en.srt) · [VTT](build/cues/full/short-draft.review.en.vtt) |

Both videos visibly say **REVIEW COPY — HIGHLIGHT TIMING UNVERIFIED**. This is
the native review status, not an omitted render step. Owner synchronized review
is still pending; automated checks do not approve cues or establish beginner
comprehension. No cue-verification, ratification or release lock was created.
Unmarked exports follow explicit owner confirmation of the exact reviewed cuts,
under [Vidni's assistant protocol](../../docs/assistant.md).

**Measured main chapters** ([text file](build/cues/full/long-draft.review.chapters.txt)):

```text
00:00 The checklist for the same parcel
00:42 Say the rule in ordinary language
01:28 Write one rule with a shared variable
03:00 Supply the rule and evaluate the claim
04:09 Add a record and follow the derivation
05:22 Compare derived and given support
06:17 Every case and at least one example
07:26 Apply the same rule to Parcel B
08:08 What the conclusion rests on
```

**Checks and evidence:**

- All 52 host fixture checks and ten real bundled-playground checks pass.
  Twenty captured states retain original browser text, verdicts and proof trees.
  Host and browser build identities remain separate.
- All 77 synthetic narration lines pass the local speech comparison at or above
  0.90, with no missing or silent clips ([log](build/hear.log)). This is advisory;
  it does not replace listening to the completed cut.
- All 73 cues (64 main, nine Short) have exact-audio advisory timings and remain
  visible for at least three seconds, with no scene/camera cutoffs. Native
  geometry checks cover every active highlight frame. Boundary/midpoint frames
  and the complete render pass checks for clipping and caption collisions;
  [native frame receipt](build/review/native-frames.json).
- Timing uses the pinned local Whisper token-DTW model against the exact audio,
  with flash attention disabled. Recognition substitutions and selected phrase
  occurrences are explicit in the recipe and
  [timing evidence](build/review/advisory-timing-evidence.json). Timings are
  proposals, not owner verification. Recorded narration requires fresh alignment.
- DOM Range fragments that touch on the same baseline are joined into readable
  line outlines. Raw measurements remain in `build/playground/*.raw-targets.json`;
  native receipts retain the unchanged image hash and measured coordinates.
  This removes separate boxes around syntax-colouring fragments without drawing
  guessed rectangles or changing engine output. A verdict highlight outlines the
  measured badge element (`result-verdict`), and the rule-citation highlight is
  the measured derived row (`derived-row`): one rectangle each.
- [Independent media checks](build/review/media-checks.json) pass full decoding,
  exact native MP4 hashes, H.264/30 fps/yuv420p, and AAC/48 kHz/stereo. Main/Short
  audio measures -14.12/-14.16 LUFS and
  -1.81/-1.99 dBTP. The 142/14 caption cues
  are ordered, fit within the measured timelines and stay below 20 characters
  per second (maxima 19.21/18.94).
- No audience questions, prediction tasks or response holds occur in narration.
  Spacing is 0.32 seconds between lines, 0.65 between sections, with a 0.6-second
  lead-in and 1.2-second tail. Highlights hold while narration continues.
- The narration originality screen finds no exact 12-word overlap and one
  ordinary six-word match, “it does not establish that the”; it was reviewed and
  retained. This is a screening result, not proof of independent expression.
  [Narration screen](build/narration-originality.log); document checks are recorded
  in the [course production record](../../../nibli/book/editorial/youtube-course/TODO.md#v02-synthetic-review).

Native video receipts bind these exact files:

| Cut | SHA-256 |
|---|---|
| Main | `f0d3585482252cde15fe05af5893570c3d21022043efc98b483e778ea87915d6` |
| Short | `95a28e79798d0eee2c1dc7bc0a7eb4d01e7f33ee9f1e4da9946dcf7d6d3484a3` |

Remaining review concerns are human: synchronized listen-through, phone-size
legibility, whether a beginner can retell the same-parcel rule and follow the
given/derived comparison. Creator recording and its new cue review are later
production steps. The lesson IDs, route, prerequisites and coverage tiers are
unchanged. No engine or Vidni API code was changed for this lesson.

<a id="review-revision"></a>
**Review revision — 2026-11-20 (Claude, at the owner's request).** An independent
review of the first synthetic copies found a contradiction between narration and
picture in the Short, several highlight-geometry flaws and two unexplained terms.
Both copies were rendered again with these changes:

- *Narration.* `v02-first-evaluation-05` uses Lesson 3's wording, “the
  playground's default reasoning settings”. `v02-some-04` now says why the claim
  uses `object`: the vocabulary has no `parcel` predicate, and the host refuses
  `parcel(ParcelA).` and `some parcel` as an unknown predicate. The main gains
  12 words.
- *Rule card.* The three wrapped lines are indented by position, because SVG
  collapses leading spaces. The outlines of `all`, `&` and `->` leave out the
  separating space.
- *Highlights.* A verdict outline covers the whole badge. The rule citation is
  one rectangle around the derived entry. “same parcel” outlines those two
  words. `v02-write-rule-03` outlines `all` on “every parcel”; it used to outline
  an unrelated caption line. `v02-some-04` outlines the predicate name and then
  its argument, which adds one cue.
- *Short.* The opening card reads “Parcel A has a packing record but no label
  record.” The FALSE shots frame the whole result row, so the claim is readable
  beside its verdict. The TRUE shot frames the proof, with the cited rule and
  both given records, above the result row.

The first copies, their receipts and the project files they were made from are
kept in [build/review/before-review-fixes-2026-11-20](build/review/before-review-fixes-2026-11-20/):
main `083e600e39e9b3ffd55c26cb33bee6e2ae92e3345bf56513de4805d9274a46b8`, Short
`933138bd46d13b25443f407184311108586a3ec7b97dc1a33d292c81546b8844`. The playground
captures and their measurement receipts were not retaken.

`ratified.lock` was written at 13:37 on 2026-11-20, before this revision, and no
take had been recorded. `vidni status` therefore reports both cuts as changed;
the owner ratifies them again before recording.

When the owner later records the final narration, automatically deliver the full
publishing copy package: YouTube title/description with measured chapters,
separate Short copy, LinkedIn/Facebook/X/Threads/Instagram link posts, WhatsApp
Status and Instagram Reels captions. Use `[YouTube link]` until a URL is supplied.
This synthetic pass does not record, ratify, release, upload or post anything.

<a id="recorded-final"></a>
## Owner confirmation and final production — 2026-11-20

The owner watched both recorded review videos and confirmed, in their own
words: “I watched both recorded review videos with audio and the highlights and
timing are correct.” The statement is saved verbatim in
[owner-confirmation.txt](build/review/recorded-voice/owner-confirmation.txt) and
was relayed with Vidni's
[`cues record-confirmation`](../../docs/cues.md#confirming-a-continuous-review)
workflow. The assistant relayed the owner's decision; it did not use the
interactive verifier or substitute its own assessment. Before the confirmation,
the assistant's independent look at the review copies reported one start about
0.2 seconds late (the Short's “Parcel A” at 0:07.2, which appears on the second
syllable), the “B's label note” outline at 4:10 sitting on the Parcel B title, and
code outlines that touch adjacent brackets. The owner confirmed the highlights
as they stand.

The command checked both exact review MP4s and their receipts against every
current audio, visual and timing binding, then saved **64 main and 9 Short cue
records** in `cues.lock` in one write. `vidni cues status` reports 64/64 and 9/9
verified, both cuts remain ratified, and `vidni validate` passes for the
assistant session. No recorded take, narration, scene, camera, cue interval or
reviewed video changed.

`vidni final` then built every export through its normal gate and native QA:

| File | Runtime | Format |
| --- | --- | --- |
| [Full master](out/long.mp4) | 9:02.906 | 1920×1080, separate captions |
| [Captioned main](out/long.captioned.mp4) | 9:02.906 | 1920×1080, burned captions |
| [Vertical Short](out/short.mp4) | 0:51.411 | 1080×1920, burned captions |

The main has [SRT](out/long.en.srt), [VTT](out/long.en.vtt) and
[chapter text](out/long.chapters.txt); the Short has [SRT](out/short.en.srt) and
[VTT](out/short.en.vtt). Vidni also filled its upload folders,
`out/youtube/long/` and `out/youtube-shorts/short/`, each with a `metadata.md`;
neither cut is approved or uploaded.

All three files decode completely as 30 fps H.264 with 48 kHz stereo AAC.
Loudness is −14.11 LUFS and −1.98 dBTP for both main files, and −14.24 LUFS and
−2.15 dBTP for the Short. Compared with the owner-reviewed copies, the decoded
audio is identical and the SRT and VTT files are byte-identical. Every one of
the 73 outlines appears on its first display frame, holds to its last and is
gone on the next, exactly as in the reviewed copy; no final frame carries the
review label. Vidni places each chapter mark at the middle of the pause between
sections, so two chapters start one second earlier than the review-stage list
(7:35 and 8:20 instead of 7:36 and 8:21). The publishing copy below uses the
final chapter file, which is also embedded in both main MP4s.

| File | SHA-256 |
| --- | --- |
| Full master | `48e608ca13e1d3e305c46e25bfd120e741f3345ef79728b491118eb3f3d4ab9c` |
| Captioned main | `9b15c3df1332e8b36332ec36f89df262e1575625fdf163cc0c685e012256c5cf` |
| Vertical Short | `e946371bccca9e79b438dd69b3f199b9982af968469780ee9a3ecaa3cf7f74af` |

The [final checks](build/review/recorded-final/final-checks.json), their
[script](build/review/recorded-final/final_checks.py) and the
[build log](build/review/recorded-final/final-build.log) are kept with these
notes. A first pass of the checks compared exact outline pixel counts and
failed on encoder differences in the thin outline; the receipt keeps those raw
counts and records the on/off comparison that replaced them.

**Remaining:** phone-size viewing, a final listen-through (the raw welcome-line
take, `v02-shop-04` at 0:19, is 3.7 dB below the median take's RMS level) and
beginner feedback.
`vidni approve` and any upload are the owner's; the assistant recorded no
release approval, upload or posting.

<a id="publishing-copy"></a>
## Publishing and sharing copy — published link updated, 2026-11-20

Prepared for the creator-narrated main lesson (**9:02.906**) and independent
Short (**0:51.411**). The chapters below are the nine starts in the delivered
[chapter file](out/long.chapters.txt), which is also embedded in both main MP4s.
The creator reported the main lesson uploaded on 2026-11-20 and supplied this
[published Lesson 4 URL](https://www.youtube.com/watch?v=GgVYSi_rROQ&list=PLAQKTXMK9CyY&index=1).
Every full-lesson link below uses that exact URL, including its playlist context.
No Short publication has been reported. The previous lesson link is the
creator-supplied Lesson 3 URL. This updates sharing copy only: the assistant has
not posted it or written to Vidni's native publication register.

### Main YouTube title

```text
Writing Rules and Evaluating Claims | Neuro-Symbolic Lesson 4
```

### Main YouTube description

```text
A packing note about Parcel A and a label note about Parcel B can't satisfy a checklist that needs both confirmations for the same parcel.

In Neuro-Symbolic Lesson 4, we write that requirement once, as a rule, and follow a conclusion back to the records that support it. We say the rule in ordinary language first, then write it in Nibli KR: all $parcel introduces a variable, the member predicate states each condition, the ampersand joins the two conditions and the arrow leads to the conclusion. The same variable in the packing, label and conclusion positions keeps all three about one parcel.

In the Nibli playground, we supply the rule with the records, review its back-translation and evaluate the claim that Parcel A meets the dispatch checklist. The result is FALSE until A's own label confirmation is recorded. Here FALSE means the claim is not derivable from what we supplied, not that the parcel has no label. With the new record the claim is TRUE, and the proof shows the derivation: the rule it cites and the two given records it rests on.

Two comparisons show what the rule contributes. Without it, the same records give FALSE. Supplying the conclusion itself gives TRUE, but its support is marked given rather than derived. We also compare the rule's "all" with a "some object" claim that needs only one supported example, and apply the same rule to Parcel B.

A conclusion that the checklist is met holds according to our records; it does not dispatch a parcel. The shop is a fictional teaching example. No formal logic or programming background is required.

Chapters
0:00 The checklist for the same parcel
0:43 Say the rule in ordinary language
1:31 Write one rule with a shared variable
3:05 Supply the rule and evaluate the claim
4:16 Add a record and follow the derivation
5:29 Compare derived and given support
6:23 Every case and at least one example
7:35 Apply the same rule to Parcel B
8:20 What the conclusion rests on

Previous lesson — Writing Your First Nibli KR Statements:
https://www.youtube.com/watch?v=ceEup-8bjyk&list=PLAQKTXMK9CyY

Try the playground: https://dhilipsiva.dev/nibli-playground/
Nibli: https://github.com/dhilipsiva/nibli

Next, The Place You Leave Empty: why leaving an argument place empty differs from saying some value.

#SymbolicReasoning #NeuroSymbolicAI #KnowledgeRepresentation
```

### YouTube Short title

```text
Records Need a Rule | Neuro-Symbolic Lesson 4
```

### YouTube Short description

```text
A shop's dispatch checklist needs packing and label confirmations for the same parcel. Parcel A has a packing record and the checklist rule, but no label record, so the claim that A meets the checklist is FALSE: unsupported by these records, not proof that a label was never attached.

Add A's label confirmation and the claim is TRUE. Remove the rule and FALSE returns, even though both records remain. A missing record and a missing rule are different gaps in what supports the conclusion.

A 51-second example from Neuro-Symbolic Lesson 4. No logic or programming background needed.

Full lesson: https://www.youtube.com/watch?v=GgVYSi_rROQ&list=PLAQKTXMK9CyY&index=1

#Shorts #SymbolicReasoning #NeuroSymbolicAI
```

### LinkedIn link post

```text
A TRUE verdict alone doesn't tell you whether a conclusion was derived or simply supplied. Reading its support does.

In Neuro-Symbolic Lesson 4, a shop's dispatch checklist needs a packing and a label confirmation for the same parcel. We write that requirement once, as a Nibli KR rule with a shared variable, evaluate the claim that Parcel A meets it, and follow the derivation back to the two records it rests on.

We also compare the rule's "all" with a "some" claim that needs only one supported example.

This is part of my Neuro-Symbolic Reasoning from First Principles course. No formal logic or programming background is required.

Watch Lesson 4: https://www.youtube.com/watch?v=GgVYSi_rROQ&list=PLAQKTXMK9CyY&index=1

#NeuroSymbolicAI #SymbolicReasoning #KnowledgeRepresentation
```

### Facebook link post

```text
Lesson 4 of my Neuro-Symbolic Reasoning course is ready.

A packing note about one parcel and a label note about another can't satisfy a checklist that needs both for the same parcel. We write that requirement once as a rule, evaluate a claim in the Nibli playground and follow the conclusion back to the records that support it.

Plain English comes first, then the notation step by step. No logic or coding background needed. The lesson runs just over nine minutes.

Watch: https://www.youtube.com/watch?v=GgVYSi_rROQ&list=PLAQKTXMK9CyY&index=1
```

### X.com link post

```text
A packing note about Parcel A and a label note about Parcel B can't meet a checklist for the same parcel.

Neuro-Symbolic Lesson 4 writes that requirement as one rule, evaluates a claim and follows its derivation.

Watch: https://www.youtube.com/watch?v=GgVYSi_rROQ&list=PLAQKTXMK9CyY&index=1
```

### Threads link post

```text
A TRUE verdict can come from a derivation, or from a conclusion we supplied ourselves. The support shows which.

In Neuro-Symbolic Lesson 4, we write a shop's dispatch checklist as one Nibli KR rule with a shared variable, evaluate whether Parcel A meets it and trace the conclusion to its rule and two records.

Everyday example first; notation follows. No formal logic background needed.

Full lesson: https://www.youtube.com/watch?v=GgVYSi_rROQ&list=PLAQKTXMK9CyY&index=1
```

### Instagram link post

```text
Two records. One rule. A conclusion we never supplied as a premise.

Neuro-Symbolic Lesson 4 writes a shop's dispatch checklist as a Nibli KR rule, evaluates the claim that Parcel A meets it and follows the derivation back to the records that support it.

One shared variable keeps both confirmations about the same parcel.

Full lesson on YouTube: https://www.youtube.com/watch?v=GgVYSi_rROQ&list=PLAQKTXMK9CyY&index=1

#NeuroSymbolicAI #SymbolicReasoning #KnowledgeRepresentation
```

### WhatsApp Status caption

```text
Records supply the information; a rule connects it to the conclusion.
Neuro-Symbolic Lesson 4: https://www.youtube.com/watch?v=GgVYSi_rROQ&list=PLAQKTXMK9CyY&index=1
```

### Instagram Reels caption

```text
A missing record and a missing rule are different gaps.

Without its label record, Parcel A's checklist claim is FALSE. Add the record: TRUE. Remove the rule: FALSE again, with both records still there.

A 51-second example from Neuro-Symbolic Lesson 4.
Full lesson: https://www.youtube.com/watch?v=GgVYSi_rROQ&list=PLAQKTXMK9CyY&index=1

#NeuroSymbolicAI #SymbolicReasoning #KnowledgeRepresentation
```

<a id="recorded-voice-review"></a>
## Recorded-voice review — 2026-11-20

The owner ratified both revised cuts and recorded every line in one session:
69 main lines and eight Short lines. Vidni reports 77 current takes, none
stale and none missing. The local speech check hears all 77 as scripted, with a
lowest score of 0.96 ([log](build/review/recorded-voice/hear.log)); it is advisory and
is not a pronunciation or pacing review. Every kept take is 48 kHz mono 16-bit
PCM without a full-scale sample, and peaks run from -12.9 to -2.1 dBFS
([take metrics](build/review/recorded-voice/take-metrics.json)). No retake was requested.

| Cut | Review video | Runtime | Captions |
|---|---|---|---|
| Main · 1920×1080 | [Watch the recorded main review](build/cues/full/long-takes.review.mp4) | 9:02.906 | [SRT](build/cues/full/long-takes.review.en.srt) · [VTT](build/cues/full/long-takes.review.en.vtt) |
| Short · 1080×1920 | [Watch the recorded Short review](build/cues/full/short-takes.review.mp4) | 0:51.411 | [SRT](build/cues/full/short-takes.review.en.srt) · [VTT](build/cues/full/short-takes.review.en.vtt) |

Both videos visibly say **REVIEW COPY — HIGHLIGHT TIMING UNVERIFIED**. They are
diagnostic previews of the recorded narration, not final exports.

**Measured main chapters, recorded voice** ([text file](build/cues/full/long-takes.review.chapters.txt)):

```text
00:00 The checklist for the same parcel
00:43 Say the rule in ordinary language
01:31 Write one rule with a shared variable
03:05 Supply the rule and evaluate the claim
04:16 Add a record and follow the derivation
05:29 Compare derived and given support
06:23 Every case and at least one example
07:36 Apply the same rule to Parcel B
08:21 What the conclusion rests on
```

**Highlights.** Recorded audio needs its own cue bindings; nothing carries over
from the synthetic voice. All 73 highlights (64 main, nine Short) have advisory
intervals from the pinned token alignment of the exact takes, and none is
owner-verified. 65 phrases were recognized as scripted. The other 8 use
explicit substitutions listed in the recipe, such as “per cell” for “parcel”
and “shad variable” for “shared variable”; one of them selects the first of two
mentions of “Packed Parcels”. A token midpoint can fall in the silence beside a
phrase, so that edge is moved to the measured speech boundary of the take: 14
starts and 43 ends moved, each recorded with its before and after value in the
[timing evidence](build/review/recorded-voice/advisory-timing-evidence.json).
Every highlight holds for at least three seconds, with no scene or camera
cutoff, and the native geometry checks pass on the boundary and midpoint frames
([frame receipt](build/review/recorded-voice/native-frames.json)). In the rendered
audio, every highlight starts where its take's interval places it.

**Media.** The [recorded-voice media checks](build/review/recorded-voice/media-checks.json)
pass full decoding, exact MP4 hashes, H.264/30 fps/yuv420p and AAC/48 kHz/stereo.
Main/Short audio measures -14.11/-14.24 LUFS and -1.98/-2.15 dBTP. The
142/14 caption cues are ordered and stay below 20 characters per second
(maxima 19.38/18.25).

| Cut | SHA-256 |
|---|---|
| Main | `4ac75c84131f53188127f28c389125f5b8dcbeffa74a6bf47f1111a251b2bd7c` |
| Short | `3d9de8e5d015bc43dd4fbf28578f2db36a5219cd4438680a8787e7e8319314e9` |

No narration, scene, camera or recorded take was changed, and both cuts remain
ratified. A before/after hash inventory covers all 407 owner files, 401 of
them recordings and their metadata: nothing changed apart from these notes
([after](build/review/recorded-voice/owner-files.after.json)). No cue verification,
release approval, upload or posting was performed.

**Next.** The owner watches both recorded review videos with audio and either
confirms that the highlights and their timing are right or names what to
change. A confirmation is relayed with `vidni cues record-confirmation`, bound
to the two hashes above; `vidni final` then builds the exports, and the full
publishing copy package is prepared against the final timeline.

<a id="production-recipes"></a>
## Archived production recipes

Recorded-voice review, after the owner ratifies and records (never replay
`author.py` at that point):

```sh
export VIDNI_ASSISTANT=1 VIDNI_VOICES="$VIDNI_PINNED_VOICES" V02_VOICE=takes
./target/release/vidni cues propose examples/nibli-v02-rules-and-claims --voice takes
./target/debug/vidni-v02-audit dump takes
python3 examples/nibli-v02-rules-and-claims/build/align_dtw.py
python3 examples/nibli-v02-rules-and-claims/build/prepare_timings.py
./target/debug/vidni-v02-audit apply takes
./target/debug/vidni-v02-audit frames takes
./target/release/vidni cues preview examples/nibli-v02-rules-and-claims --voice takes
python3 examples/nibli-v02-rules-and-claims/build/check_delivery.py
python3 examples/nibli-v02-rules-and-claims/build/recorded_voice_notes.py
```

<details>
<summary>build/run_fixtures.py</summary>

````python
import hashlib, importlib.util, json, os, subprocess, sys
from pathlib import Path
sys.dont_write_bytecode=True
root=Path('/home/dhilipsiva/projects/dhilipsiva')
engine=root/'nibli'; course=engine/'book/editorial/youtube-course'; folder=course/'fixtures/V02'
spec=importlib.util.spec_from_file_location('fixture_runner',course/'tools/run_fixtures.py')
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
prior=json.loads((course/'fixtures/V03a/parcel-results.json').read_text())
manifest=json.loads((folder/'parcel-expectations.json').read_text())
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
results=[]
for name,hashes in manifest['sessions'].items():
    p=folder/(name+'.nibli'); x=p.with_suffix('.expect.json')
    assert sha(p)==hashes['input'] and sha(x)==hashes['expectation']
    expect=json.loads(x.read_text())
    for profile in ['off','on']:
        env={k:v for k,v in os.environ.items() if not k.startswith('NIBLI_')}
        env.update(NIBLI_WASM_PATH=str(engine/'target/wasm32-wasip2/debug/nibli.wasm'),NIBLI_QUIET='1',NIBLI_EXISTENTIAL_IMPORT='0',NIBLI_MAX_CHAIN_DEPTH='10',NIBLI_MATERIALIZE={'off':'0','on':'1'}[profile])
        run=subprocess.run([prior['loader'],'--library-path',prior['library_path'],str(engine/'target/debug/nibli-host')],input=p.read_text()+':quit\n',text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,cwd=engine,env=env,timeout=300)
        start=run.stdout.find('nibli> ');out=run.stdout[start:] if start>=0 else run.stdout
        p.with_suffix('.'+profile+'.txt').write_text(out)
        found=m.blocks(out,'nibli> ');problems=m.check(expect,found,profile)
        if run.returncode: problems.append('host exited '+str(run.returncode))
        facts=[b['text'] for b in found if b['input']==':facts']
        if facts and (len(facts)!=2 or facts[0]!=facts[1]): problems.append('repeated evaluation changed the assertion listing')
        results.append({'session':name,'profile':profile,'expectations':len(expect['expect']),'repeated_facts_unchanged':not facts or facts[0]==facts[1],'problems':problems})
        print(name,profile,'PASS' if not problems else problems,flush=True)
result={'date':'2026-11-20','engine':m.engine_identity(engine),'binary_execution':'Existing unchanged host and component, using installed compatible Nix glibc loader; not freshly rebuilt.','loader':prior['loader'],'library_path':prior['library_path'],'expectations_written_before_fixture_execution':True,'planning_probes_preceded_fixture_authoring':True,'profiles':{'materialization':['off','on'],'existential_import':False,'depth':10},'results':results,'checks':sum(x['expectations'] for x in results),'all_passed':all(not x['problems'] for x in results)}
(folder/'parcel-results.json').write_text(json.dumps(result,indent=2)+'\n')
assert result['all_passed'],result
print('PASS',result['checks'],'expectations across both profiles')
````

</details>

<details>
<summary>build/capture_playground.mjs</summary>

````javascript
// Capture the real bundled Dioxus/WASM playground. No engine or result mocks.
import assert from 'node:assert/strict';
import {chromium} from '/home/dhilipsiva/projects/dhilipsiva/nibli-learn/node_modules/playwright/index.mjs';
import {createServer} from 'node:http';
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {extname,resolve} from 'node:path';
import {createHash} from 'node:crypto';
const project='/home/dhilipsiva/projects/dhilipsiva/vidni/examples/nibli-v02-rules-and-claims';
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
const packing='member(ParcelA, DispatchChecklistMet).';const label='member(ParcelA, LabeledParcels).';
const rule='all $parcel: member($parcel, PackedParcels) & member($parcel, LabeledParcels) -> member($parcel, DispatchChecklistMet).';
const ruleKB=baseline+'\n'+rule;const repaired=ruleKB+'\n'+label;const objects='object(ParcelA).\nobject(ParcelB).';
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
 if(name.includes('missing')||name.includes('no-rule')){const row=page.locator('.proof-tree > .proof-node > .proof-row[aria-expanded="true"]').first();if(await row.count())await row.click();assert.notEqual(await page.locator('.proof-tree > .proof-node > .proof-row').first().getAttribute('aria-expanded'),'true');}
 await page.mouse.move(0,0);
 await page.evaluate(()=>document.activeElement?.blur());
 await page.evaluate(async()=>{await document.fonts.ready;await Promise.all(document.getAnimations().map(a=>a.finished.catch(()=>{}))); await new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r)));});
 const regions={workspace:'.app-shell',header:'.app-header',custom:'.app-header__examples',kb:'.main-row .col-tabs', 'kb-tabs':'.main-row .tab-bar','kb-code':'#kb-kr-highlight',claim:'.query-row .col-tabs','claim-tabs':'.query-row .tab-bar','claim-field':'.query-bar','claim-code':'#query-input-hl',proof:'.main-row .col-proof','proof-content':'.proof-tree-container',log:'.output-log-container',history:'#output-log',result:'#output-log .output-entry:last-child',profile:'.output-profile',review:'.back-translation',status:'.kb-status-bar'};
 const scopes={'kb-text':'#kb-kr-highlight','claim-text':'#query-input-hl',result:'#output-log .output-entry:last-child',history:'#output-log','proof-line':'.proof-tree > .proof-node > .proof-row > .proof-label','proof-verdict':'.proof-tree > .proof-node > .proof-row','proof-all':'.proof-tree','why':'.proof-why-text','review-text':'.back-translation',status:'.kb-status-bar',error:'.translate-error'};
 const terms=[rule,label,'member(ParcelB, PackedParcels).','object(ParcelA).','object(ParcelB).','ParcelA','ParcelB','PackedParcels','LabeledParcels','DispatchChecklistMet','object','some object','all $parcel:','$parcel','&','->','member','set','TRUE','FALSE','[given]','Use reviewed encoding','2 asserted','3 asserted','4 asserted','6 asserted','7 asserted','0 asserted','unknown predicate','too many arguments','arity 2','expected'];
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

  function ranges(e,start,end){
   let offset=0;const boxes=[];const walker=document.createTreeWalker(e,NodeFilter.SHOW_TEXT);
   while(walker.nextNode()){const n=walker.currentNode,lo=Math.max(0,start-offset),hi=Math.min(n.length,end-offset);if(lo<hi){const r=document.createRange();r.setStart(n,lo);r.setEnd(n,hi);boxes.push(...[...r.getClientRects()].filter(b=>b.width>0&&b.height>0).map(rect));}offset+=n.length;}
   return boxes;
  }
  for(const e of document.querySelectorAll('.proof-label')){
   if(!visible(e))continue;const t=e.textContent;
   if(t.includes('[by recorded source')){
    targets['derived-conclusion']=ranges(e,0,t.indexOf('[by recorded source'));
    targets['rule-citation']=ranges(e,t.indexOf('[by recorded source'),t.length);
    targets['derived-row']=[rect(e.getBoundingClientRect())];
   }
   if(t.includes('[given]')&&t.toLowerCase().includes('packedparcels'))targets['packed-leaf']=ranges(e,0,t.length);
   if(t.includes('[given]')&&t.toLowerCase().includes('labeledparcels'))targets['labeled-leaf']=ranges(e,0,t.length);
   if(t.includes('[given]')&&t.toLowerCase().includes('dispatchchecklistmet'))targets['given-conclusion']=ranges(e,0,t.length);
   if(t.includes('[given]')&&t.toLowerCase().includes('object'))targets['object-leaf']=ranges(e,0,t.length);
  }
  const verdict=first('#output-log .output-entry:last-child .output-result');if(verdict)targets['result-verdict']=[rect(verdict.getBoundingClientRect())];
  // Syntax colouring splits one selected phrase into adjacent DOM text nodes.
  // Join only touching fragments on the same measured baseline; keep separate
  // rows and genuine gaps separate. Preserve raw measurements as evidence.
  const rawTargets=structuredClone(targets);
  for(const [key,boxes] of Object.entries(targets)){
   const joined=[];
   for(const b of [...boxes].sort((a,b)=>Math.abs(a[1]-b[1])<=1.5?a[0]-b[0]:a[1]-b[1])){
    const a=joined.at(-1);
    if(a&&Math.abs(a[1]-b[1])<=1.5&&Math.abs(a[3]-b[3])<=3&&b[0]<=a[0]+a[2]+3){
     const right=Math.max(a[0]+a[2],b[0]+b[2]),bottom=Math.max(a[1]+a[3],b[1]+b[3]);
     a[0]=Math.min(a[0],b[0]);a[1]=Math.min(a[1],b[1]);a[2]=right-a[0];a[3]=bottom-a[1];
    }else joined.push([...b]);
   }
   targets[key]=joined;
  }
  return {targets,rawTargets,contents};
 },{regions,scopes,terms});
 const file=name+'.png';await page.screenshot({path:out+'/'+file});
 const bytes=await readFile(out+'/'+file);const text=await page.locator('body').innerText();
 await writeFile(out+'/'+name+'.txt',text);
 const receipt={schema:1,image:'build/playground/'+file,sha256:createHash('sha256').update(bytes).digest('hex'),width:3840,height:2160,browser:await browser.version(),targets:measured.targets};
 await writeFile(out+'/'+name+'.raw-targets.json',JSON.stringify({image:receipt.image,sha256:receipt.sha256,raw_targets:measured.rawTargets,measurement:'DOM Ranges at 3x; adjacent same-baseline fragments joined in the native receipt'},null,2)+'\n');
 await writeFile(out+'/'+name+'.targets.json',JSON.stringify(receipt,null,2)+'\n');
 captures.push({name,text,file,contents:measured.contents,receipt:name+'.targets.json'});console.log('captured',name,Object.keys(measured.targets).length,'measured targets');
}

async function reviewed(kb){await fresh(kb);await approve();await kr();}
try{
 await fresh(baseline);await shot('01-baseline');
 await fresh(ruleKB);await shot('02-rule');
 await page.getByRole('button',{name:'Query',exact:true}).click();assert.match(await page.locator('body').innerText(),/Use reviewed encoding before querying/);checks.push('unreviewed rule KB blocks evaluation');
 await fresh(ruleKB);await back();await shot('03-review');await approve();await kr();await shot('04-rule-ready');
 await query(packing,'FALSE');await shot('05-missing-label');checks.push('A missing label FALSE, original two records and one rule');
 await page.locator('#kb-kr-editor').fill(repaired);await page.getByRole('button',{name:'Query',exact:true}).click();assert.match(await page.locator('body').innerText(),/Use reviewed encoding before querying/);checks.push('adding A label requires fresh review');
 await reviewed(repaired);await shot('06-repaired-ready');await query(packing,'TRUE');
 let proof=await page.locator('.proof-tree').innerText();assert.match(proof,/by recorded source/);assert.match(proof,/packedparcels \[given\]/i);assert.match(proof,/labeledparcels \[given\]/i);assert.ok(!proof.includes('FALSE'));await shot('07-derived');checks.push('one-rule TRUE cites rule and both A records; no false supported step');
 const before=await page.locator('#kb-kr-editor').inputValue();await query(packing,'TRUE');assert.equal(await page.locator('#kb-kr-editor').inputValue(),before);await shot('08-repeat');checks.push('repeated evaluation leaves premise pane unchanged');
 await reviewed(baseline+'\n'+label);await query(packing,'FALSE');await shot('09-no-rule');checks.push('both A confirmations without rule FALSE');
 await reviewed(baseline+'\n'+label+'\n'+packing);await query(packing,'TRUE');proof=await page.locator('.proof-tree').innerText();assert.match(proof,/\[given\]/);assert.ok(!proof.includes('by recorded source'));await shot('10-given');checks.push('direct conclusion TRUE with given support and no rule citation');
 await reviewed(repaired);await query(packing,'TRUE');await shot('11-restored');
 await reviewed(repaired+'\n'+objects);await page.locator('#query-input').fill('member(some object, DispatchChecklistMet).');await shot('12-objects');await query('member(some object, DispatchChecklistMet).','TRUE');await shot('13-some-one');checks.push('some object TRUE with A as recorded object and rule-derived member');
 await query('member(ParcelB, DispatchChecklistMet).','FALSE');await shot('14-b-missing');
 await reviewed(repaired+'\n'+objects+'\nmember(ParcelB, PackedParcels).');await page.locator('#query-input').fill('member(ParcelB, DispatchChecklistMet).');await shot('15-b-ready');await query('member(ParcelB, DispatchChecklistMet).','TRUE');await shot('16-b-derived');await query('member(some object, DispatchChecklistMet).','TRUE');await shot('17-some-both');checks.push('same rule derives B after B packing; some remains TRUE with both');
 await reviewed('member(ParcelA, PackedParcels).\n'+rule);await query(packing,'FALSE');await shot('18-short-missing');
 await reviewed('member(ParcelA, PackedParcels).\n'+rule+'\n'+label);await query(packing,'TRUE');await shot('19-short-derived');
 await reviewed('member(ParcelA, PackedParcels).\n'+label);await query(packing,'FALSE');await shot('20-short-no-rule');checks.push('standalone Short fresh states: missing note FALSE, record plus rule TRUE, missing rule FALSE');
 assert.deepEqual(pageErrors,[]);
 const receipt={date:'2026-11-20',surface:'Real bundled Dioxus playground served on loopback; public deployment not contacted.',manifest,playgroundWasmSha256,fontconfigSha256,blockedRemoteOrigins:[...blockedOrigins],viewport:[1280,720],scale:3,browser:await browser.version(),checks,pageErrors,captures,allPassed:true};
 await writeFile(out+'/receipt.json',JSON.stringify(receipt,null,2)+'\n');console.log('PASS',checks.length,'browser checks;',captures.length,'captures');
}catch(e){if(page){await page.screenshot({path:out+'/failure.png'});await writeFile(out+'/failure.txt',await page.locator('body').innerText());}throw e;}
finally{await browser.close();await new Promise(r=>server.close(r));}
````

</details>

<details>
<summary>build/author.py</summary>

````python
"""Rebuild generated artwork and the authored Lesson 4 package. Recipe is archived in notes.md."""
import base64, hashlib, html, json, re
from pathlib import Path

P=Path(__file__).resolve().parents[1]
OUT=P/'build/cards';OUT.mkdir(parents=True,exist_ok=True)
J=lambda v:json.dumps(v,ensure_ascii=False)
E=lambda s:html.escape(s,quote=True)
BG='#0b0b10';INK='#f4f3ef';MUTED='#908f9a';ACC='#f2542d';PANEL='#15151e';EDGE='#3a3a4a'
scenes={};lines=[];sections={}
def text(id,x,y,s,size=32,color=INK,font='IBM Plex Sans',anchor='start'):
    return f'<text id="{id}" x="{x}" y="{y}" font-family="{font}" font-size="{size}" font-weight="700" fill="{color}" text-anchor="{anchor}">{E(s)}</text>'
def rect(id,x,y,w,h,color=PANEL,stroke=EDGE,r=16):
    return f'<rect id="{id}" x="{x}" y="{y}" width="{w}" height="{h}" rx="{r}" fill="{color}" stroke="{stroke}" stroke-width="2"/>'
def group(id,s):return f'<g id="{id}">{s}</g>'
def heading(kicker,title,sub=''):
    return text('kicker',60,58,'// '+kicker,21,ACC)+text('title',60,120,title,45,font='Space Grotesk')+(text('subtitle',60,166,sub,25,MUTED) if sub else '')
def art(name,body,targets,shape=(1280,720)):
    w,h=shape
    svg=f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">'+rect('background',0,0,w,h,BG,BG,0)+body+'</svg>\n'
    (OUT/(name+'.svg')).write_text(svg)
    scenes[name]={'url':'build/cards/'+name+'.svg','targets':{k:{'element':v,'continuity':name+'-'+k} for k,v in targets.items()}}
def block(id,x,y,title,body):
    return group(id,rect(id+'-box',x,y,540,225)+text(id+'-title',x+32,y+52,title,35,font='Space Grotesk')+text(id+'-body',x+32,y+119,body,29)+text(id+'-note',x+32,y+176,'A record about this parcel',22,MUTED))

body=heading('AT THE PACKING TABLE','Two parcels. Separate records.')
body+=block('a',60,210,'Parcel A','Packing confirmed')+block('b',680,210,'Parcel B','Label confirmed')
body+=text('same',60,535,'Both confirmations must concern the same parcel.',35)
body+=text('limit',60,603,'Our model records confirmations. It does not dispatch parcels.',24,MUTED)
art('shop',body,{'a':'a-title','b':'b-title','same':'same','limit':'limit'})
# The narration names two words of this sentence; outline those words.
scenes['shop']['targets']['same']['span']={'text':'same parcel'}
body=heading('NEURO-SYMBOLIC LESSON 4','Writing Rules and Evaluating Claims','Reasoning from First Principles')
for i,(id,title,desc) in enumerate([('records','Records','Supplied confirmations'),('rule','Rule','Supplied requirement'),('conclusion','Conclusion','Supported result')]):
    x=60+i*400;body+=rect(id+'-box',x,255,360,235)+text(id,x+26,315,title,38,font='Space Grotesk')+text(id+'-a',x+26,379,desc[:25],23,MUTED)
body+=text('goal',60,590,'Follow a conclusion back to its support.',37)
body+=text('limit',60,664,'The model checks records. Physical dispatch is separate.',25,MUTED)
art('purpose',body,{'records':'records','rule':'rule','conclusion':'conclusion','goal':'goal','limit':'limit'})

body=heading('THE SHOP’S REQUIREMENT','Say the rule before writing the symbols')
body+=rect('english-panel',60,220,1160,350)
body+=text('every',92,286,'For any parcel:',37)
body+=text('condition-a',92,365,'if that parcel has a packing confirmation',34)
body+=text('condition-b',92,423,'and that parcel has a label confirmation,',34)
body+=text('conclusion',92,503,'then that parcel meets the dispatch checklist.',34)
body+=text('same',60,634,'“That parcel” refers to the same thing throughout.',28,MUTED)
art('english-rule',body,{k:k for k in ['every','condition-a','condition-b','conclusion','same']})

body=heading('OUR CHOSEN REPRESENTATION','Three lists, with a clear purpose')
for i,(id,label,meaning) in enumerate([('packed','Packed parcels','Packing confirmations'),('labeled','Labelled parcels','Label confirmations'),('checklist','Dispatch checklist met','The model’s conclusion')]):
    y=215+i*120;body+=rect(id+'-panel',60,y,1160,95)+text(id,90,y+61,label,34)+text(id+'-meaning',720,y+59,meaning,25,MUTED)
body+=text('meaning',60,636,'A list name does not itself supply a record.',29,MUTED)
art('lists',body,{k:k for k in ['packed','labeled','checklist','meaning']})

def code_row(id,y,parts,indent=0):
    s=f'<text id="{id}" x="{92+indent}" y="{y}" font-family="IBM Plex Mono" font-size="29" font-weight="700" fill="{INK}">'
    for token,content in parts:s+=f'<tspan id="{token}">{E(content)}</tspan>'
    return s+'</text>'
# SVG collapses leading spaces, so the wrapped lines are indented by position:
# four IBM Plex Mono advances at this size (4 x 0.6 x 29, rounded).
INDENT=70
rows=[
 ('binder',275,0,[('all','all '),('var-binding','$parcel'),('colon',':')]),
 ('packing-condition',350,INDENT,[('p-prefix','member('),('var-packed','$parcel'),('packed-set',', PackedParcels)')]),
 ('label-condition',425,INDENT,[('and','& '),('l-prefix','member('),('var-labeled','$parcel'),('labeled-set',', LabeledParcels)')]),
 ('rule-conclusion',500,INDENT,[('arrow','-> '),('c-prefix','member('),('var-conclusion','$parcel'),('dispatch-set',', DispatchChecklistMet)'),('period','.')])]
# A token that carries a separating space is outlined without that space.
token_spans={'all':'all','and':'&','arrow':'->','packed-set':'PackedParcels','labeled-set':'LabeledParcels','dispatch-set':'DispatchChecklistMet'}
for stage in range(1,5):
    body=heading('ONE SHARED VARIABLE','The same parcel in both conditions','A rule is a reusable pattern.')
    body+=rect('code-panel',60,210,1160,340)
    ids=[]
    for id,y,indent,parts in rows[:stage]:body+=code_row(id,y,parts,indent);ids+=[id]+[x for x,_ in parts]
    body+=text('reading',60,625,['Choose one parcel at a time.','Check its packing record.','Check its label record too.','Both conditions support this conclusion.'][stage-1],30,MUTED)
    art('rule-'+str(stage),body,{k:k for k in ids+['reading']})
    for key,value in scenes['rule-'+str(stage)]['targets'].items():
        value['continuity']='rule-build-'+key
        if key in token_spans:value['span']={'text':token_spans[key]}
    if stage==4:scenes['rule-4']['targets']['shared']={'elements':['var-packed','var-labeled','var-conclusion'],'continuity':'shared-parcel-arguments'}

body=heading('A NEW RECORD ARRIVES','Parcel A now has its own label confirmation')
body+=block('packing',60,230,'Parcel A','Packing confirmed')+block('label',680,230,'Parcel A','Label confirmed')
body+=text('change',60,572,'The information changed. The checklist rule stayed the same.',30)
art('new-label',body,{'packing':'packing-body','label':'label-body','change':'change'})
for name,title,sub in [('compare-rule','Keep the records. Remove the rule.','A comparison using a fresh knowledge base.'),('compare-given','Supply the conclusion directly.','A comparison of given and derived support.')]:
    body=heading('A CONTROLLED COMPARISON',title,sub)+text('point',60,340,'Parcel A meets the dispatch checklist.',39)+text('footer',60,555,'Read what supports the verdict.',36,ACC)
    art(name,body,{'point':'point','footer':'footer'})

body=heading('EVERY AND SOME','One supported example is enough')
body+=text('rule-meaning',60,246,'The rule applies to every matching case.',33)
body+=text('some-meaning',60,310,'The “some” claim needs at least one.',33)
body+=rect('objects-panel',60,370,1160,175)
body+=text('object-a',92,428,'object(ParcelA).',31,font='IBM Plex Mono')+text('object-b',92,493,'object(ParcelB).',31,font='IBM Plex Mono')
body+=text('reading',60,622,'Explicit records: A is an object. B is an object.',29,MUTED)
art('objects',body,{k:k for k in ['rule-meaning','some-meaning','object-a','object-b','reading']})
# The predicate name and its one argument, for the line that introduces them.
scenes['objects']['targets']['object-word']={'element':'object-a','span':{'text':'object'},'continuity':'objects-object-word'}
scenes['objects']['targets']['object-arg']={'element':'object-a','span':{'text':'ParcelA'},'continuity':'objects-object-arg'}
body=heading('THE SAME RULE WORKS AGAIN','Parcel B receives its missing packing record')
body+=block('packing',60,230,'Parcel B','Packing confirmed')+block('label',680,230,'Parcel B','Label confirmed')
body+=text('some',60,574,'“Some” remains true when both parcels qualify.',35)
art('b-record',body,{'packing':'packing-body','label':'label-body','some':'some'})
body=heading('NEURO-SYMBOLIC LESSON 4','A claim, its rule, and its records')
for i,(id,s) in enumerate([('one','One variable keeps one parcel together.'),('two','A derived conclusion cites its rule and records.'),('three','“Some” needs at least one supported example.')]):body+=text(id,60,270+i*85,s,34)
body+=text('next',60,608,'Next: the place you leave empty.',31,ACC)
art('closing',body,{k:k for k in ['one','two','three','next']})
for name,title,label,detail in [('short-shop','One parcel. One checklist.','Packing + label','Parcel A has a packing record but no label record.'),('short-new','A new label record','Parcel A','The same checklist rule now applies.'),('short-rule','Two different gaps','A missing record','A missing rule'),('short-close','Records need a rule.','Follow the support.','Neuro-Symbolic Lesson 4')]:
    b=text('kicker',44,48,'// SYMBOLIC REASONING',20,ACC)
    words=title.split();width=25;parts=[];current=''
    for w in words:
        if len(current+' '+w)>width:parts.append(current);current=w
        else:current=(current+' '+w).strip()
    if current:parts.append(current)
    for i,s in enumerate(parts):b+=text('title-'+str(i),44,115+i*53,s,45,font='Space Grotesk')
    b+=rect('panel',44,235,632,270)+text('label',78,315,label,39)
    # Two short lines, natural wrapping independent of highlight geometry.
    wrapped=[];current=''
    for w in detail.split():
        if len(current+' '+w)>29:wrapped.append(current);current=w
        else:current=(current+' '+w).strip()
    if current:wrapped.append(current)
    for i,s in enumerate(wrapped):b+=text('detail-'+str(i),78,391+i*47,s,30,MUTED)
    if name!='short-close':b+=text('footer',44,578,'Neuro-Symbolic Lesson 4',27)
    art(name,b,{'label':'label','detail':'detail-0'},(720,620))
    if name=='short-shop':scenes[name]['targets']['detail']['span']={'text':'Parcel A'}

def sec(id,title):sections[id]=title
def add(section,text_,scene,cues=(),camera=None,cut='long'):
    n=1+sum(l['section']==section for l in lines)
    d={'id':f'v02-{section}-{n:02d}','section':section,'show':text_,'scene':scene,'source':'notes.md#v02-'+section,'only':[cut]}
    if cues:d['emphasis']=[{'id':'focus-'+str(i+1),'target':target,'phrase':phrase} for i,(target,phrase) in enumerate(cues)]
    if camera:d['camera']=camera;d['camera_vertical']=camera
    lines.append(d)

# NARRATION_START
sec('shop','The checklist for the same parcel')
add('shop','Back at our small shop, Parcel A has a packing record, and Parcel B has a label record.','shop',[('a','Parcel A'),('b','Parcel B')])
add('shop',"The shop's dispatch checklist needs both confirmations for the same parcel.",'shop',[('same','same parcel')])
add('shop','A packing note about A and a label note about B do not satisfy that requirement for either parcel.','shop')
add('shop',"Welcome to Neuro-Symbolic Lesson 4. We're learning to make reasoning explicit, one step at a time.",'purpose')
add('shop','Last time we wrote individual records and evaluated claims against them.','purpose',[('records','individual records')])
add('shop',"This time we'll write the shop's requirement once, as a rule, and follow a conclusion back to the records that support it.",'purpose',[('rule','as a rule'),('conclusion','a conclusion')])
add('shop','That conclusion means the checklist is met according to our records. It does not send a parcel anywhere.','purpose',[('limit','according to our records')])

sec('plain-rule','Say the rule in ordinary language')
add('plain-rule',"The requirement starts with any parcel, without choosing A or B in advance.",'english-rule',[('every','any parcel')])
add('plain-rule','If that parcel has a packing confirmation, and that parcel has a label confirmation, then that parcel meets the dispatch checklist.','english-rule',[('condition-a','packing confirmation'),('condition-b','label confirmation'),('conclusion','meets the dispatch checklist')])
add('plain-rule','Each time we say that parcel, we mean the same thing. We cannot take one confirmation from A and the other from B.','english-rule',[('same','the same thing')])
add('plain-rule','This is a reusable pattern. A particular record concerns one named parcel; the rule can apply whenever its conditions are met.','english-rule')
add('plain-rule',"Our records already use two lists: packed parcels and labelled parcels. We'll represent the conclusion with a third list, Dispatch Checklist Met.",'lists',[('checklist','third list')])
add('plain-rule','That third name is our label for parcels whose records satisfy this checklist. The name itself does not make anything qualify.','lists')

sec('write-rule','Write one rule with a shared variable')
add('write-rule',"In Nibli KR, we give the placeholder a name: dollar parcel. The dollar sign marks a variable, and parcel reminds us what we're tracking.",'rule-1',[('var-binding','dollar parcel')])
add('write-rule','The beginning reads all dollar parcel, followed by a colon. It says to consider each possible value for that variable.','rule-1',[('binder','all dollar parcel')])
add('write-rule','This does not say that every parcel meets the checklist. The conditions that follow still have to hold.','rule-1',[('all','every parcel')])
add('write-rule','The first condition uses our familiar member predicate. Its first argument is now the variable, instead of a particular parcel name.','rule-2',[('var-packed','the variable')])
add('write-rule','The second argument is Packed Parcels. Together, those arguments say that this parcel is on the packed-parcels list.','rule-2',[('packed-set','Packed Parcels')])
add('write-rule','Next comes the ampersand, read as and. Both conditions must hold for the rule to apply.','rule-3',[('and','the ampersand')])
add('write-rule','The second condition puts the same variable on the labelled-parcels list. We have changed the list, while keeping the parcel together.','rule-3',[('var-labeled','same variable'),('labeled-set','labelled-parcels list')])
add('write-rule','The arrow is written with a hyphen and a greater-than sign. Here we read it as then: it leads to the conclusion.','rule-4',[('arrow','The arrow')])
add('write-rule','The conclusion puts that parcel on Dispatch Checklist Met. A period finishes the complete rule.','rule-4',[('dispatch-set','Dispatch Checklist Met'),('period','A period')])
add('write-rule','Notice the same variable in all three argument positions: packing, label, and conclusion. The engine must use one value throughout.','rule-4',[('shared','same variable')])
add('write-rule',"We've written the shop's requirement, rather than a separate checklist result for each parcel. Now we can evaluate a particular claim.",'rule-4')
add('write-rule',"This is one complete statement, wrapped here for reading. In the playground input, we keep that statement together.",'rule-4')

sec('first-evaluation','Supply the rule and evaluate the claim')
add('first-evaluation',"Here is the playground with our two original records and the completed rule in the knowledge-base pane.",'pg-02-rule',camera='wide')
add('first-evaluation','The rule goes with the premises. Rules are starting assumptions too: this is the checklist we have chosen for our example.','pg-02-rule',[('kb-rule','The rule')],camera='kb')
add('first-evaluation','Before using it, we compare the back-translation with our intended requirement. Its repeated X stands for the same shared variable.','pg-03-review',[('review','back-translation')],camera='review')
add('first-evaluation','The reading must retain both conditions and the same parcel. We select Use reviewed encoding to make this checked version available for evaluation.','pg-03-review',[('review-action','Use reviewed encoding')],camera='review')
add('first-evaluation',"We're using the playground's default reasoning settings. The records and rule are now available for evaluation.",'pg-04-rule-ready',camera='wide')
add('first-evaluation','In the separate claim field, we write that Parcel A is a member of Dispatch Checklist Met. This evaluates A without adding that conclusion.','pg-04-rule-ready',[('claim','separate claim field')],camera='claim')
add('first-evaluation','The result is FALSE. The packing condition holds for A, but its own label confirmation is missing from the records.','pg-05-missing-label',[('false','FALSE')],camera='result')
add('first-evaluation',"Here FALSE means the claim is not derivable from what we supplied. It does not establish that the physical parcel has no label.",'pg-05-missing-label',camera='result')
add('first-evaluation',"B's label note cannot fill that gap for A. The shared variable keeps those two parcels separate.",'shop',[('a','for A'),('b',"B's label note")])

sec('derivation','Add a record and follow the derivation')
add('derivation','Later, a colleague adds a label confirmation for Parcel A. Now both confirmations concern A. The information has changed.','new-label',[('label','label confirmation')])
add('derivation','We encode that new record as another premise, and review the changed knowledge base before using it.','pg-06-repaired-ready',[('a-label','new record')],camera='kb')
add('derivation','The claim stays the same. The TRUE result now follows from the added record and the rule we already supplied.','pg-07-derived',[('true','TRUE')],camera='result')
add('derivation','We can now follow the support in the proof panel. Start with the conclusion: A meets the dispatch checklist.','pg-07-derived',[('derived','the conclusion')],camera='proof')
add('derivation','Beside it, the answer cites the recorded rule. That is the requirement connecting the two conditions to this conclusion.','pg-07-derived',[('citation','recorded rule')],camera='proof')
add('derivation','Underneath are the supporting records. One says A is on the packed-parcels list, and is marked given.','pg-07-derived',[('packed','packed-parcels list')],camera='proof')
add('derivation','The other says A is on the labelled-parcels list, also given. Both records concern A, exactly as our rule requires.','pg-07-derived',[('labeled','labelled-parcels list')],camera='proof')
add('derivation','Given means we supplied that record. The checklist conclusion was derived: the engine applied our rule to those two records.','pg-07-derived',[('derived','was derived')],camera='proof')
add('derivation','That short path from records, through a rule, to the conclusion is a derivation. It makes the support visible.','pg-07-derived',camera='proof')
add('derivation','Evaluating the claim again gives the same support. It does not add the conclusion to the premises we supplied.','pg-08-repeat',camera='inputs')

sec('controls','Compare derived and given support')
add('controls',"A small comparison shows what the rule contributes. We'll keep A's two confirmations, but remove the rule.",'compare-rule')
add('controls','With both records still present, A’s checklist claim now returns FALSE. The engine has no supplied rule connecting those lists to checklist membership.','pg-09-no-rule',[('false','FALSE')],camera='result')
add('controls','The two records are available information. The rule is what makes this particular conclusion follow from them.','pg-09-no-rule',camera='kb')
add('controls',"For a second comparison, we supply the checklist conclusion itself as a premise. That is a different use of the same statement.",'compare-given',[('point','conclusion itself')])
add('controls','Now the verdict is TRUE, but the support is marked given. There is no rule citation connecting it to the two confirmations.','pg-10-given',[('given','marked given')],camera='proof')
add('controls','A TRUE verdict alone does not tell us whether we supplied a conclusion or derived it. Reading the support tells us which happened.','pg-10-given',camera='proof')
add('controls','We remove that supplied conclusion and restore our rule. The answer once again derives checklist membership from A’s two records.','pg-11-restored',[('derived','derives checklist membership')],camera='proof')

sec('some','Every case and at least one example')
add('some',"The word all made our rule general. It can apply to each matching parcel, without a new rule written just for that name.",'objects',[('rule-meaning','each matching parcel')])
add('some','A different claim can require only one qualifying example. We will write: some object meets the dispatch checklist.','objects',[('some-meaning','only one')])
add('some','First, we explicitly record Parcel A and Parcel B as objects. These are two additional premises, not facts inferred from their names.','objects',[('object-a','Parcel A'),('object-b','Parcel B')])
add('some','Parcel is not a predicate in the supported vocabulary, so we use the predicate object. It has one argument: the thing we are recording. The phrase some object then considers those recorded things.','objects',[('object-word','predicate object'),('object-arg','one argument')])
add('some','In the claim, some object takes the first argument position. Dispatch Checklist Met stays in the second position.','pg-12-objects',[('some','some object'),('claim-set','Dispatch Checklist Met')],camera='claim')
add('some','This means at least one recorded object belongs to that list. We are evaluating whether such an example is supported.','pg-12-objects',camera='claim')
add('some','The result is TRUE because A qualifies. Its object record and the checklist derivation supply the support.','pg-13-some-one',[('object-leaf','object record'),('derived','checklist derivation')],camera='proof')
add('some','One supported example is enough for this claim. It does not say that every object qualifies, or that exactly one does.','pg-13-some-one',camera='proof')

sec('parcel-b','Apply the same rule to Parcel B')
add('parcel-b',"B still has only a label confirmation. Evaluating B's own checklist claim gives FALSE, because its packing record is missing.",'pg-14-b-missing',[('false','claim gives FALSE')],camera='result')
add('parcel-b','Now a packing confirmation arrives for B as well. We add that record and review the changed input.','b-record',[('packing','packing confirmation')])
add('parcel-b',"The same rule now derives B's checklist membership. Its proof uses B's packing and label records. Both confirmations concern B, so neither comes from A.",'pg-16-b-derived',[('packed',"B's packing"),('labeled','label records')],camera='proof')
add('parcel-b','Both parcels now qualify. Evaluating the some-object claim still returns TRUE. At least one includes the case where both qualify.','pg-17-some-both',[('true','TRUE')],camera='result')
add('parcel-b','In ordinary conversation, some can suggest not all. That extra suggestion is not part of this formal claim.','b-record',[('some','not all')])

sec('closing','What the conclusion rests on')
add('closing','We began with two separate records and one everyday requirement. A shared variable kept each parcel together across both conditions.','closing',[('one','shared variable')])
add('closing','Once the right records were present, the rule supported a conclusion we had never supplied as a premise. The derivation showed the connection.','closing',[('two','The derivation')])
add('closing','And the some-object claim needed only one supported example, even when more than one was available.','closing',[('three','one supported example')])
add('closing',"This is the symbolic foundation of our neuro-symbolic course: make the starting information explicit, evaluate a claim, and inspect what supports it.",'closing')
add('closing',"Next, we'll look at an argument place left empty, and why leaving it out differs from saying some value.",'closing',[('next','left empty')])

sec('short-story','')
add('short-story',"A shop's dispatch checklist needs packing and label confirmations for the same parcel.",'short-shop',[('label','packing and label')],cut='short')
add('short-story','Parcel A has a packing record and the checklist rule, but no label record.','short-shop',[('detail','Parcel A')],cut='short')
add('short-story',"For the claim that A meets the checklist, the verdict is FALSE. That means unsupported by these records, not proof that a label was never attached.",'pg-18-short-missing',[('false','FALSE')],camera='result',cut='short')
add('short-story',"A new label confirmation arrives for A. We add that record as a premise; the checklist rule stays the same.",'short-new',[('label','for A')],cut='short')
add('short-story','The TRUE result now follows from both records and the checklist rule.','pg-19-short-derived',[('true','TRUE')],camera='short-proof',cut='short')
add('short-story','Without the rule, FALSE returns. Both records remain, but the connection is missing.','pg-20-short-no-rule',[('false','FALSE')],camera='result',cut='short')
add('short-story','A missing record and a missing rule are different gaps in what supports the conclusion.','short-rule',[('label','missing record'),('detail','and a missing rule')],cut='short')
add('short-story','Follow the support: records supply the information, and the rule connects it to the conclusion.','short-close',[('label','Follow the support')],cut='short')
# NARRATION_END

# Import measured screenshot geometry; preserve the whole real playground image.
rule='all $parcel: member($parcel, PackedParcels) & member($parcel, LabeledParcels) -> member($parcel, DispatchChecklistMet).'
raw_names={'kb-rule':'kb-text/'+rule+'/1','a-label':'kb-text/member(ParcelA, LabeledParcels)./1','claim':'claim-field','false':'result-verdict','true':'result-verdict','review':'review','review-action':'review-action','derived':'derived-conclusion','citation':'derived-row','packed':'packed-leaf','labeled':'labeled-leaf','given':'given-conclusion','some':'claim-text/some object/1','claim-set':'claim-text/DispatchChecklistMet/1','object-leaf':'object-leaf'}
views={'kb':['kb'],'claim':['claim-field'],'result':['result'],'proof':['proof-content'],'review':['review','review-action'],'inputs':['kb','claim-field'],'short-proof':['proof-all-ink','result']}
for name in sorted({l['scene'] for l in lines if l['scene'].startswith('pg-')}):
    state=name[3:];path=P/f'build/playground/{state}.targets.json';receipt=json.loads(path.read_text());available=receipt['targets']
    image=(P/receipt['image']).read_bytes()
    assert hashlib.sha256(image).hexdigest()==receipt['sha256']
    encoded=base64.b64encode(image).decode()
    (OUT/(name+'.svg')).write_text(f'<svg xmlns="http://www.w3.org/2000/svg" width="1280" height="720" viewBox="0 0 1280 720"><image id="screen" x="0" y="0" width="1280" height="720" preserveAspectRatio="none" href="data:image/png;base64,{encoded}"/></svg>\n')
    t={};cams={}
    wanted={c['target'] for l in lines if l['scene']==name for c in l.get('emphasis',[])}
    for key in sorted(wanted):
        raw=raw_names[key];assert raw in available,(state,key,raw)
        t[key]={'element':'screen','capture':{'receipt':str(path.relative_to(P)),'target':raw},'continuity':state+'-'+key}
    for cam in sorted({l.get('camera','wide') for l in lines if l['scene']==name}):
        if cam=='wide':cams[cam]={'viewport':True};continue
        names=[]
        for raw in views[cam]:
            assert raw in available,(state,cam,raw)
            key='region-'+raw;names.append(key)
            t[key]={'element':'screen','capture':{'receipt':str(path.relative_to(P)),'target':raw}}
        cams[cam]={'union':names}
    scenes[name]={'url':'build/cards/'+name+'.svg','targets':t,'cameras':cams}

header='''schema = 1
[meta]
title = "Writing Rules and Evaluating Claims"
subtitle = "Reasoning from First Principles"
language = "en"
strict = true
default_scene = "shop"

[voice]
draft = "piper:en_US-joe-medium"
length_scale = 1.04
respell = [{ text = "Nibli KR", say = "Nibbly K R" }, { text = "Nibli", say = "Nibbly" }, { text = "Parcel A", say = "Parcel, A" }]
heard_as = [{ heard = "nibbly", means = "nibli" }, { heard = "nibley", means = "nibli" }]

[emphasis]
min_duration = 3.0

[timing]
lead_in = 0.6
line_gap = 0.32
section_gap = 0.65
tail = 1.2

[style]
theme = "../themes/quine"

[lint]
banned = ['\\bask(?:s|ed|ing)?\\b', '\\bquestions?\\b', '\\bpause and\\b', '\\bpredict\\b', 'UNDETERMINED']

[[cuts]]
id = "long"
aspect = "16:9"
captions = "both"
platforms = ["youtube"]

[[cuts]]
id = "short"
aspect = "9:16"
captions = "burned"
platforms = ["youtube-shorts"]

[music]
track = "none"

[publish]
description = "Neuro-Symbolic Lesson 4 continues the parcel example with one shared variable, a checklist rule, claim evaluation and supporting derivations. Compare derived and given conclusions, then introduce every and some. The shop and its records are fictional."
tags = ["neuro-symbolic reasoning", "Nibli KR", "symbolic reasoning", "knowledge representation"]

[publish.titles]
youtube = "Writing Rules and Evaluating Claims | Neuro-Symbolic Lesson 4"
youtube-shorts = "Records Need a Rule | Neuro-Symbolic Lesson 4"
'''
def toml_value(v):
    if isinstance(v,dict):return '{ '+', '.join(k+' = '+toml_value(x) for k,x in v.items())+' }'
    if isinstance(v,list):return '['+', '.join(toml_value(x) for x in v)+']'
    return J(v)
s=header
for key,title in sections.items():s+=f'\n[sections.{key}]\nlabel = ""\nchapter = {J(title)}\n'
for line in lines:
    s+='\n[[lines]]\n'+''.join(k+' = '+toml_value(v)+'\n' for k,v in line.items())
for name,scene in scenes.items():
    s+=f'\n[scenes.{name}]\nkind = "image"\nurl = {J(scene["url"])}\nscale = 3\n'
    for key,target in scene['targets'].items():s+=f'\n[scenes.{name}.targets.{key}]\n'+''.join(k+' = '+toml_value(v)+'\n' for k,v in target.items())
    for key,cam in scene.get('cameras',{}).items():s+=f'\n[scenes.{name}.cameras.{key}]\n'+''.join(k+' = '+toml_value(v)+'\n' for k,v in cam.items())
(P/'vidni.toml').write_text(s)
counts={cut:sum(len(re.findall(r"\b[\w]+(?:['’-][\w]+)*\b",l['show'])) for l in lines if cut in l['only']) for cut in ['long','short']}
(P/'build/writing.json').write_text(json.dumps({'words':counts,'lines':len(lines),'cuts':{cut:{'lines':sum(cut in l['only'] for l in lines),'cues':sum(len(l.get('emphasis',[])) for l in lines if cut in l['only'])} for cut in counts},'storyboard':lines},indent=2)+'\n')

notes='''# Writing Rules and Evaluating Claims

**Neuro-Symbolic Lesson 4 · stable curriculum ID V02 · 2026-11-20**

This package follows the published Lesson 3 parcel handoff. Its one objective is
to write a reusable rule, evaluate a specific claim and follow its derivation.
The owner selected synthetic main and Short previews and a parcel-only narration.
The drawing transfer remains optional written practice. The shop and records are
original fictional examples, not observations of real parcels.

## Production status

Script, source mapping and generated QUINE artwork are prepared. The main has
{long} scripted words and the Short {short}. Aim for 8–10 minutes (hard ceiling
13 minutes, no minimum) and 45–60 seconds respectively; actual measured runtimes
belong in the synthetic-review receipt below after rendering.

Use the existing Piper en_US-joe-medium voice at length_scale 1.04. Every Vidni
command uses VIDNI_ASSISTANT=1. Native measured emphasis has a 3.0-second minimum;
several highlights may overlap. Natural speech continues while a highlight holds.
No audience questions, prediction tasks or response gaps belong in either cut.
All drafts use QUINE and Neuro-Symbolic Lesson 4 branding.

The existing page.html and .gitignore were created by `vidni new` and are unused
but preserved. Generated graphics, source DOM measurement receipts, scripts and
diagnostic media live in build/. No ownership, ratification or cue-verification
lock is written by hand. Synthetic review videos are diagnostic previews; they
are not owner-approved exports or a substitute for review of recorded narration.

## Narration and storyboard

Read these paragraphs as connected prose before recording individual stable
lines. The source for the fictional situation is this authored scenario; the
technical claims are grounded in R01–R05 below and the fixture/browser receipts.
Illustrations explain the model; playground pictures and verdicts are real.

'''.format(**counts)
for section,title in sections.items():
    notes+=f'<a id="v02-{section}"></a>\n### {title or "Standalone Short: Records Need a Rule"}\n\n'
    for l in [x for x in lines if x['section']==section]:notes+=l['show']+'\n\n'
    notes+='| Line | Picture / framing | Spoken emphasis → measured target |\n|---|---|---|\n'
    for l in [x for x in lines if x['section']==section]:
        cue='; '.join('“'+c['phrase']+'” → `'+c['target']+'`' for c in l.get('emphasis',[])) or 'Keep context readable.'
        notes+=f'| `{l["id"]}` | `{l["scene"]}` · {l.get("camera","full illustration")} | {cue} |\n'
    notes+='\n'
notes+='''## Source receipts and representation limits

Public technical source revision: `1eab5d1f1d52d1c106145aa0cbc18205ff9b0902`.
No retired Lojban syntax or proposed schema declarations are taught.

| Receipt | Public source | Supports |
|---|---|---|
| R01 | [Language contract](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/NIBLI_KR.md), §§4–6 | Implemented universal variables, conjunction, implication/rule spelling and restricted existential `some object`. |
| R02 | [Current KR cookbook](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/mdbook/src/user/kr-cookbook.md) and [grammar](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/nibli-kr/src/nibli_kr.pest) | Statement period; named variable spelling; current source language. |
| R03 | [Committed predicates](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/nibli-lexicon/src/corpus/predicates.rs) | `member(member, set)` and unary `object(material)`; the vocabulary has no `parcel` predicate. Names of collections supply no built-in parcel semantics. |
| R04 | [Playground guide](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/mdbook/src/user/playground.md) and [review state](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/nibli-ui/src/review.rs) | Separate premises and claim; whole-candidate admission; review invalidated by edits; evaluation does not assert its claim. |
| R05 | [Verdict contract](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/README.md) and [proof renderer](https://github.com/dhilipsiva/nibli/blob/1eab5d1f1d52d1c106145aa0cbc18205ff9b0902/nibli-render/src/proof.rs) | FALSE is not derivable, not proof of negation; rule citations and given support. |

The list `DispatchChecklistMet` records the model's conclusion, not an action or
physical inspection. `$parcel` is a chosen variable name, not a built-in parcel
type. `all` generalizes the rule; it does not assert either condition or make all
parcels qualify. Membership does not silently assert `object`, so A and B receive
explicit object records before the restricted existential claim. `parcel` is not
a vocabulary word: the host refuses `parcel(ParcelA).` and `some parcel` as an
unknown predicate, so the claim says `some object`. `some` means at
least one and is compatible with all qualifying. This lesson teaches only one
rule application; OR/grouping, conditional comparisons, detailed quantifier
scope/import, rule chains and raw proof internals stay in their assigned lessons.

The browser bundle's shared manifest names revision `36739cca`; it hashes the
separate guided-page module. The actual playground WASM hash is recorded
separately in `build/playground/receipt.json`. Host and browser receipts do not
claim identical builds. Both execute the demonstrated examples independently.
The local bundle retains awkward readings such as “parcela members”. Narration
explains their meaning; no browser wording or output is hand-replaced. FALSE
shots leave internal witness trees collapsed and frame the verdict. The narrator
explains its qualification instead of presenting a fabricated engine message.

## Current coverage mapping

| Curriculum obligation | Main passage / evidence |
|---|---|
| CH02.02 a · mechanical derivation · T | derivation: conclusion, actual cited rule, two given leaves; repeated-evaluation fixture. |
| CH03.01 e · some compatible with all · T | some → parcel-b: A supplies one witness, then both qualify without changing the some verdict. |
| CH04.01 c · basic universal/existential commitments · T | write-rule and some: conditional scope of all, explicit object records, at least one supported witness. |
| CH07.01 a · conjunction · T | write-rule: both member conditions concern the shared parcel; initial near miss and successful repair. |
| CH07.03 a · rule · T | plain-rule through derivation; controls remove/restore the rule. |
| CH08.03 a · universals · T | all applies across matching cases; the same rule derives B when its own record arrives. |
| CH09.01 a,b · facts versus templates · T | plain-rule distinguishes a named record from a reusable pattern; controls separate their contributions. |
| CH09.03 a · condition · T | staged rule construction separates the two conditions from their conclusion. |
| APP-A.01 d · some X restriction · T | some: explicit unary object records; object-restriction fixture proves they are required. |
| APP-B.01 · browser interface reinforcement | first-evaluation and controlled comparisons; KB versus bare claim; fresh review after edits. Ownership remains V03a. |
| APP-B.02 d,f · compact answer parts · T | derivation and controls: verdict, one-line reason, source citation, given leaves. |
| APP-G.01 d · $var · D | write-rule: dollar sign, name, repeated argument positions, one value. |

Coverage family IDs, owners and tiers are retained. The Short reinforces missing
record versus missing rule with its own one-parcel setup; it is not a substitute
for any main-lesson coverage obligation. Document/media checks do not establish
beginner comprehension. Owner review and feedback from a learner remain pending.

## Companion reference

The baseline premises, including the new rule:

```text
member(ParcelA, PackedParcels).
member(ParcelB, LabeledParcels).
all $parcel: member($parcel, PackedParcels) & member($parcel, LabeledParcels) -> member($parcel, DispatchChecklistMet).
```

Evaluate `member(ParcelA, DispatchChecklistMet).` in the separate browser claim
field: FALSE. Add A's newly received `member(ParcelA, LabeledParcels).` record,
review, and evaluate again: TRUE with rule and two given records. The host
interface puts `? ` before a claim to select evaluation; that prefix is not part
of the browser input or the KR statement. Repeated evaluation adds no premise.

For the some-object claim, add `object(ParcelA).` and `object(ParcelB).`, then
evaluate `member(some object, DispatchChecklistMet).` A qualifies. B's own
checklist claim is still FALSE until `member(ParcelB, PackedParcels).` arrives.
The unchanged rule then derives B too. The some-object claim remains TRUE.

**Reference-only encoding error:**

```text
all $parcel, $other: member($parcel, PackedParcels) & member($other, LabeledParcels) -> member($parcel, DispatchChecklistMet).
```

With only A's packing and B's label, this different rule derives A's checklist
membership. The engine follows the supplied rule; the encoding fails to express
the intended same-parcel requirement. Use an isolated session, then reset and
restore one shared variable. This rule is not taught as a second main-video rule.

**Optional written transfer:** A class display requires each drawing's title and
artist-name confirmations. Represent these as `TitledDrawings` and
`ArtistNamedDrawings`, with conclusion list `DisplayChecklistMet`:

```text
member(Drawing1, TitledDrawings).
member(Drawing2, ArtistNamedDrawings).
all $drawing: member($drawing, TitledDrawings) & member($drawing, ArtistNamedDrawings) -> member($drawing, DisplayChecklistMet).
```

Drawing 1's display-checklist claim is FALSE until its own artist-name record is
supplied. This written worked transfer is fixture-checked and does not interrupt
the parcel-only video. Existing companion activity `found.rules-given` remains
optional; no companion application code is changed.

## Evidence and reproducibility

[Parcel fixture notes](../../../nibli/book/editorial/youtube-course/fixtures/V02/parcel-notes.md)
link the new sessions, expectations and both-profile captures. Expectations were
saved before the fixture run; the earlier planning probes are disclosed. The
historical camera/instrument fixtures and their notes are preserved unchanged.

`build/playground/receipt.json` records real browser checks, build hashes, source
text and measured screenshot targets. Each measurement binds exact PNG bytes;
no highlight rectangle is estimated from an image. Authored SVG targets bind
exact text elements. The native renderer applies camera transforms and checks
highlight clipping/caption collisions on every active frame.

Rebuild with the pinned Vidni Nix environment, after restoring the scripts below:

```sh
python3 build/run_fixtures.py
node build/capture_playground.mjs
python3 build/author.py
VIDNI_ASSISTANT=1 vidni validate .
VIDNI_ASSISTANT=1 vidni cues propose . --voice draft
VIDNI_ASSISTANT=1 vidni cues preview . --voice draft
```

Run the generation scripts only before owner revisions, or reconcile those
revisions into the recipe first. Never overwrite later edits or recordings by
blindly replaying a writing-stage generator. The renderer owns generated locks.

<a id="synthetic-review"></a>
## Synthetic review delivery

Pending the render and synchronized inspection. No owner approval is implied by
technical checks. Cue proposals are advisory. The native diagnostic previews
remain available for review without writing cue verification. Unmarked exports
follow an explicit owner confirmation of the exact reviewed cuts, as required by
[Vidni's assistant protocol](../../docs/assistant.md).

When the owner later records the final narration, automatically deliver the full
publishing copy package: YouTube title/description with measured chapters,
separate Short copy, LinkedIn/Facebook/X/Threads/Instagram link posts, WhatsApp
Status and Instagram Reels captions. Use `[YouTube link]` until a URL is supplied.
This synthetic pass does not record, ratify, release, upload or post anything.
'''
(P/'notes.md').write_text(notes)
print('Authored',len(lines),'lines;',counts,'words;',len(scenes),'scenes')
````

</details>

<details>
<summary>build/finalize_notes.py</summary>

````python
"""Save delivery notes only after the actual two-cut media checks pass."""
import json
from pathlib import Path
P=Path(__file__).resolve().parents[1]
root=P.parents[2];course=root/'nibli/book/editorial/youtube-course'
qa=json.loads((P/'build/review/media-checks.json').read_text());assert qa['all_passed']
long,short=qa['cuts'];assert long['cut']=='long' and short['cut']=='short'
writing=json.loads((P/'build/writing.json').read_text());words=writing['words']
clock=lambda t:f"{int(t//60)}:{t%60:06.3f}"
number={0:'no',1:'one',2:'two',3:'three',4:'four',5:'five',6:'six',7:'seven',8:'eight',9:'nine'}
lc,sc=long['cues'],short['cues'];assert lc==writing['cuts']['long']['cues'] and sc==writing['cuts']['short']['cues']
def swap(text,old,new):
    assert text.count(old)==1,old[:60]
    return text.replace(old,new)
notes=(P/'notes.md').read_text()
notes=swap(notes,f'Script, source mapping and generated QUINE artwork are prepared. The main has\n{words["long"]} scripted words and the Short {words["short"]}. Aim for 8–10 minutes (hard ceiling\n13 minutes, no minimum) and 45–60 seconds respectively; actual measured runtimes\nbelong in the synthetic-review receipt below after rendering.',
f'''Both synthetic diagnostic review videos are rendered and checked. The main has
{words["long"]:,} scripted words and measures **{clock(long["timeline_seconds"])}**; the standalone Short has {words["short"]} words
and measures **{clock(short["timeline_seconds"])}**. They meet the 8–10 minute and 45–60 second targets;
the course's main-lesson ceiling remains 13 minutes, with no minimum.
The [delivery record](#synthetic-review) links the videos, captions and evidence.
Both were rendered again on 2026-11-20 after an independent review; the
[review revision](#review-revision) lists what changed.''')
notes=swap(notes,'This lesson teaches only one\nrule application;', 'Each demonstrated derivation applies\none rule, first to A and later to B;')
start=notes.index('Rebuild with the pinned Vidni Nix environment, after restoring the scripts below:')
end=notes.index('<a id="synthetic-review"></a>',start)
notes=notes[:start]+'''The reproducible recipes are archived below because `build/` is generated and
ignored by Git. Restore each block to its named path inside this project. Use
the pinned Vidni Nix environment and run these commands from the Vidni repository:

```sh
export VIDNI_ASSISTANT=1
export VIDNI_VOICES="$VIDNI_PINNED_VOICES"
python3 examples/nibli-v02-rules-and-claims/build/run_fixtures.py
node examples/nibli-v02-rules-and-claims/build/capture_playground.mjs
python3 examples/nibli-v02-rules-and-claims/build/author.py
./target/release/vidni validate examples/nibli-v02-rules-and-claims
./target/release/vidni cues propose examples/nibli-v02-rules-and-claims --voice draft
cargo build --offline --manifest-path examples/nibli-v02-rules-and-claims/build/native/Cargo.toml --target-dir target
./target/debug/vidni-v02-audit dump
python3 examples/nibli-v02-rules-and-claims/build/align_dtw.py
python3 examples/nibli-v02-rules-and-claims/build/prepare_timings.py
./target/debug/vidni-v02-audit apply
./target/debug/vidni-v02-audit frames
./target/release/vidni cues preview examples/nibli-v02-rules-and-claims --voice draft
python3 examples/nibli-v02-rules-and-claims/build/check_delivery.py
```

Run generation only before owner revisions, or reconcile those revisions into
the recipe first. `author.py` writes both the project and the initial notes;
preserve this delivery record before any deliberate regeneration. Never overwrite
later wording edits or recordings by blindly replaying the writing-stage recipe.
The renderer owns voice locks. No recipe writes owner verification or approval.

'''+notes[end:]
start=notes.index('Pending the render and synchronized inspection.')
end=notes.index('When the owner later records the final narration',start)
chapters='\n'.join(f"{c['timestamp']} {c['title']}" for c in long['chapters'])
notes=notes[:start]+f'''**Synthetic diagnostic previews — 2026-11-20.** Both use QUINE, the existing
Piper `en_US-joe-medium` voice, measured emphasis and continuous narration.
The main stays with the parcels; the independent Short establishes its own shop
situation and explicitly identifies the claim before showing its verdict.

| Cut | Review video | Runtime | Captions |
|---|---|---|---|
| Main · 1920×1080 | [Watch the main review](build/cues/full/long-draft.review.mp4) | {clock(long['timeline_seconds'])} | [SRT](build/cues/full/long-draft.review.en.srt) · [VTT](build/cues/full/long-draft.review.en.vtt) |
| Short · 1080×1920 | [Watch the Short review](build/cues/full/short-draft.review.mp4) | {clock(short['timeline_seconds'])} | [SRT](build/cues/full/short-draft.review.en.srt) · [VTT](build/cues/full/short-draft.review.en.vtt) |

Both videos visibly say **REVIEW COPY — HIGHLIGHT TIMING UNVERIFIED**. This is
the native review status, not an omitted render step. Owner synchronized review
is still pending; automated checks do not approve cues or establish beginner
comprehension. No cue-verification, ratification or release lock was created.
Unmarked exports follow explicit owner confirmation of the exact reviewed cuts,
under [Vidni's assistant protocol](../../docs/assistant.md).

**Measured main chapters** ([text file](build/cues/full/long-draft.review.chapters.txt)):

```text
{chapters}
```

**Checks and evidence:**

- All 52 host fixture checks and ten real bundled-playground checks pass.
  Twenty captured states retain original browser text, verdicts and proof trees.
  Host and browser build identities remain separate.
- All {writing['lines']} synthetic narration lines pass the local speech comparison at or above
  0.90, with no missing or silent clips ([log](build/hear.log)). This is advisory;
  it does not replace listening to the completed cut.
- All {lc+sc} cues ({lc} main, {number.get(sc,sc)} Short) have exact-audio advisory timings and remain
  visible for at least three seconds, with no scene/camera cutoffs. Native
  geometry checks cover every active highlight frame. Boundary/midpoint frames
  and the complete render pass checks for clipping and caption collisions;
  [native frame receipt](build/review/native-frames.json).
- Timing uses the pinned local Whisper token-DTW model against the exact audio,
  with flash attention disabled. Recognition substitutions and selected phrase
  occurrences are explicit in the recipe and
  [timing evidence](build/review/advisory-timing-evidence.json). Timings are
  proposals, not owner verification. Recorded narration requires fresh alignment.
- DOM Range fragments that touch on the same baseline are joined into readable
  line outlines. Raw measurements remain in `build/playground/*.raw-targets.json`;
  native receipts retain the unchanged image hash and measured coordinates.
  This removes separate boxes around syntax-colouring fragments without drawing
  guessed rectangles or changing engine output. A verdict highlight outlines the
  measured badge element (`result-verdict`), and the rule-citation highlight is
  the measured derived row (`derived-row`): one rectangle each.
- [Independent media checks](build/review/media-checks.json) pass full decoding,
  exact native MP4 hashes, H.264/30 fps/yuv420p, and AAC/48 kHz/stereo. Main/Short
  audio measures {long['integrated_lufs']:.2f}/{short['integrated_lufs']:.2f} LUFS and
  {long['true_peak_dbtp']:.2f}/{short['true_peak_dbtp']:.2f} dBTP. The {long['captions']}/{short['captions']} caption cues
  are ordered, fit within the measured timelines and stay below 20 characters
  per second (maxima {long['max_caption_cps']:.2f}/{short['max_caption_cps']:.2f}).
- No audience questions, prediction tasks or response holds occur in narration.
  Spacing is 0.32 seconds between lines, 0.65 between sections, with a 0.6-second
  lead-in and 1.2-second tail. Highlights hold while narration continues.
- The narration originality screen finds no exact 12-word overlap and one
  ordinary six-word match, “it does not establish that the”; it was reviewed and
  retained. This is a screening result, not proof of independent expression.
  [Narration screen](build/narration-originality.log); document checks are recorded
  in the [course production record](../../../nibli/book/editorial/youtube-course/TODO.md#v02-synthetic-review).

Native video receipts bind these exact files:

| Cut | SHA-256 |
|---|---|
| Main | `{long['sha256']}` |
| Short | `{short['sha256']}` |

Remaining review concerns are human: synchronized listen-through, phone-size
legibility, whether a beginner can retell the same-parcel rule and follow the
given/derived comparison. Creator recording and its new cue review are later
production steps. The lesson IDs, route, prerequisites and coverage tiers are
unchanged. No engine or Vidni API code was changed for this lesson.

<a id="review-revision"></a>
**Review revision — 2026-11-20 (Claude, at the owner's request).** An independent
review of the first synthetic copies found a contradiction between narration and
picture in the Short, several highlight-geometry flaws and two unexplained terms.
Both copies were rendered again with these changes:

- *Narration.* `v02-first-evaluation-05` uses Lesson 3's wording, “the
  playground's default reasoning settings”. `v02-some-04` now says why the claim
  uses `object`: the vocabulary has no `parcel` predicate, and the host refuses
  `parcel(ParcelA).` and `some parcel` as an unknown predicate. The main gains
  12 words.
- *Rule card.* The three wrapped lines are indented by position, because SVG
  collapses leading spaces. The outlines of `all`, `&` and `->` leave out the
  separating space.
- *Highlights.* A verdict outline covers the whole badge. The rule citation is
  one rectangle around the derived entry. “same parcel” outlines those two
  words. `v02-write-rule-03` outlines `all` on “every parcel”; it used to outline
  an unrelated caption line. `v02-some-04` outlines the predicate name and then
  its argument, which adds one cue.
- *Short.* The opening card reads “Parcel A has a packing record but no label
  record.” The FALSE shots frame the whole result row, so the claim is readable
  beside its verdict. The TRUE shot frames the proof, with the cited rule and
  both given records, above the result row.

The first copies, their receipts and the project files they were made from are
kept in [build/review/before-review-fixes-2026-11-20](build/review/before-review-fixes-2026-11-20/):
main `083e600e39e9b3ffd55c26cb33bee6e2ae92e3345bf56513de4805d9274a46b8`, Short
`933138bd46d13b25443f407184311108586a3ec7b97dc1a33d292c81546b8844`. The playground
captures and their measurement receipts were not retaken.

`ratified.lock` was written at 13:37 on 2026-11-20, before this revision, and no
take had been recorded. `vidni status` therefore reports both cuts as changed;
the owner ratifies them again before recording.

'''+notes[end:]
files=[('build/run_fixtures.py','python'),('build/capture_playground.mjs','javascript'),('build/author.py','python'),('build/native/Cargo.toml','toml'),('build/native/src/main.rs','rust'),('build/align_dtw.py','python'),('build/prepare_timings.py','python'),('build/check_art.mjs','javascript'),('build/check_delivery.py','python')]
notes+='\n<a id="production-recipes"></a>\n## Archived production recipes\n\n'
for name,lang in files:
    notes+=f'<details>\n<summary>{name}</summary>\n\n````{lang}\n'+(P/name).read_text().rstrip()+'\n````\n\n</details>\n\n'
(P/'notes.md').write_text(notes)
print('Saved complete delivery notes and',len(files),'reproducible recipes.')
````

</details>

<details>
<summary>build/native/Cargo.toml</summary>

````toml
[package]
name = "vidni-v02-audit"
version = "0.0.0"
edition = "2024"

[workspace]

[dependencies]
vidni-core = { path = "/home/dhilipsiva/projects/dhilipsiva/vidni/crates/vidni-core" }
vidni-media = { path = "/home/dhilipsiva/projects/dhilipsiva/vidni/crates/vidni-media" }
serde_json = "1.0.154"

[profile.dev.package."*"]
opt-level = 2
````

</details>

<details>
<summary>build/native/src/main.rs</summary>

````rust
use std::{fs,path::Path};
use vidni_core::{validate,release::Voice,voices,timeline,cues::{self,Record,Records,Interval}};
use vidni_media::build::{self,Setup};
use serde_json::{json,Value};

fn main()->Result<(),String>{
 let mode=std::env::args().nth(1).unwrap_or("dump".into());
 // Second argument: `takes` audits the recorded narration; the default is the draft voice.
 let takes=std::env::args().nth(2).as_deref()==Some("takes");
 let label=if takes {"takes"} else {"draft"};
 let voice=||if takes {Voice::Takes} else {Voice::Draft};
 let dir=Path::new("/home/dhilipsiva/projects/dhilipsiva/vidni/examples/nibli-v02-rules-and-claims");
 let out=dir.join(if takes {"build/review/recorded-voice"} else {"build/review"});fs::create_dir_all(&out).map_err(|e|e.to_string())?;
 let p=validate::check(&fs::read_to_string(dir.join("vidni.toml")).unwrap(),Some(dir)).map_err(|e|format!("{e:?}"))?;
 let setup=Setup::find(&p,dir)?;let pack=voices::find(&p.voice.draft,voices::store().as_deref())?;
 let overrides:Value=fs::read(if takes {out.join("advisory-overrides.json")} else {dir.join("build/advisory-overrides.json")}).ok().and_then(|b|serde_json::from_slice(&b).ok()).unwrap_or(json!({}));
 let mut cuts=Vec::new();let mut evidence=Vec::new();
 for cut in &p.cuts {
  if mode=="apply" {
   let plan=build::plan_with_review(&p,dir,cut,voice(),pack.as_ref(),&setup,true).map_err(|e|format!("{e:?}"))?;
   let mut records=Records::read(&dir.join(cues::PROPOSALS))?;
   for cue in &plan.emphasis {
    if let Some(v)=overrides.get(&cue.key) {
     if v["audio"].as_str()!=Some(cue.binding.audio.as_str()){return Err(format!("override audio mismatch: {}",cue.key));}
     let interval=Interval{start:(v["start"].as_f64().ok_or("missing start")?*f64::from(cue.binding.rate)).round() as u64,end:((v["end"].as_f64().ok_or("missing end")?*f64::from(cue.binding.rate)).round() as u64).min(cue.binding.samples)};
     interval.check(&cue.binding)?;
     records.entries.insert(cue.key.clone(),Record{binding:cue.binding.clone(),interval});
     evidence.push(json!({"key":cue.key,"binding":cue.binding,"interval":interval,"basis":v,"status":"advisory only; no owner verification"}));
    }
   }
   records.write(&dir.join(cues::PROPOSALS))?;
  }
  let plan=build::plan_with_review(&p,dir,cut,voice(),pack.as_ref(),&setup,true).map_err(|e|format!("{e:?}"))?;
  fs::write(out.join(format!("{}-cues.json",cut.id)),serde_json::to_vec_pretty(&plan.emphasis).unwrap()).unwrap();
  fs::write(out.join(format!("{}-timeline.json",cut.id)),serde_json::to_vec_pretty(plan.timeline()).unwrap()).unwrap();
  let full=dir.join("build/cues/full");fs::create_dir_all(&full).map_err(|e|e.to_string())?;
  fs::write(full.join(format!("{}-{label}.review.en.srt",cut.id)),timeline::srt(plan.timeline())).unwrap();
  fs::write(full.join(format!("{}-{label}.review.en.vtt",cut.id)),timeline::vtt(plan.timeline())).unwrap();
  let mut frames=Vec::new();
  if mode=="frames" || mode=="sample" {
   let mut composer=plan.composer(true)?;
   for cue in plan.emphasis.iter().filter(|_|mode=="frames") {
    let d=cue.display.as_ref().ok_or_else(||format!("unset cue {}",cue.key))?;
    let mut samples=vec![d.start.saturating_sub(1),d.start,(d.start+d.end)/2,d.end-1,d.end];
    for e in cue.events(){samples.push(e.start);samples.push(e.end-1);}
    samples.sort();samples.dedup();
    for frame in samples {
     if frame>=composer.frames(){continue;}
     let image=composer.frame(frame)?;
     if frame==(d.start+d.end)/2 {
      let name=format!("{}-{}-{}.png",cut.id,cue.line_id,cue.key.rsplit('/').next().unwrap());
      image.save_png(&out.join(&name))?;frames.push(json!({"line":cue.line_id,"key":cue.key,"frame":frame,"file":name}));
     }
    }
   }
   for (i,s) in plan.timeline().sentences.iter().enumerate(){
    let frame=(((s.start+s.end)/2.0)*30.0).floor() as u64;
    let name=format!("{}-line-{i:03}.png",cut.id);composer.frame(frame)?.save_png(&out.join(&name))?;
   }
  }
  cuts.push(json!({"cut":cut.id,"duration":plan.timeline().total,"cues":plan.emphasis.len(),"proposed":plan.emphasis.iter().filter(|c|c.interval.is_some()).count(),"verified":plan.emphasis.iter().filter(|c|c.verified).count(),"full_minimum":plan.emphasis.iter().filter(|c|c.display.as_ref().is_some_and(|d|d.end-d.start>=90)).count(),"cutoffs":plan.emphasis.iter().filter_map(|c|c.display.as_ref().filter(|d|d.cutoff.is_some()).map(|d|json!({"key":c.key,"seconds":(d.end-d.start) as f64/30.0,"reason":d.cutoff}))).collect::<Vec<_>>(),"frames":frames}));
 }
 fs::write(out.join(format!("native-{mode}.json")),serde_json::to_vec_pretty(&cuts).unwrap()).unwrap();
 if mode=="apply" {fs::write(out.join("advisory-timing-evidence.json"),serde_json::to_vec_pretty(&evidence).unwrap()).unwrap();}
 println!("{}",serde_json::to_string(&cuts.iter().map(|v|json!({"cut":v["cut"],"duration":v["duration"],"proposed":v["proposed"],"cues":v["cues"],"full_minimum":v["full_minimum"]})).collect::<Vec<_>>()).unwrap());Ok(())
}
````

</details>

<details>
<summary>build/align_dtw.py</summary>

````python
import concurrent.futures,hashlib,json,os,subprocess
from pathlib import Path
P=Path(__file__).resolve().parents[1]
MODEL_SHA='a03779c86df3323075f5e796cb2ce5029f00ec8869eee3fdfb897afe36c6d002'
assert hashlib.sha256(Path(os.environ['VIDNI_WHISPER_MODEL']).read_bytes()).hexdigest()==MODEL_SHA
# V02_VOICE=takes aligns the recorded narration; the default is the draft voice.
TAKES=os.environ.get('V02_VOICE')=='takes'
REVIEW=P/('build/review/recorded-voice' if TAKES else 'build/review')
out=(REVIEW/'dtw') if TAKES else P/'build/cues/dtw-aligned';out.mkdir(parents=True,exist_ok=True)
items={}
for cut in ['long','short']:
    for c in json.loads((REVIEW/f'{cut}-cues.json').read_text()):items[c['binding']['audio']]=c
def run(item):
    sha,c=item;dest=out/sha
    assert hashlib.sha256(Path(c['audio']).read_bytes()).hexdigest()==sha
    if not dest.with_suffix('.json').exists():
        cmd=[os.environ.get('VIDNI_WHISPER','whisper-cli'),'-m',os.environ['VIDNI_WHISPER_MODEL'],'-f',c['audio'],'-ojf','-sow','-ml','1','-ng','-nfa','-t','4','-dtw','base.en','-of',str(dest)]
        r=subprocess.run(cmd,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,timeout=180)
        dest.with_suffix('.log').write_text(r.stdout)
        if r.returncode:raise RuntimeError(c['line_id']+': '+r.stdout[-1000:])
    d=json.loads(dest.with_suffix('.json').read_text())
    tokens=[t for s in d['transcription'] for t in s.get('tokens',[]) if t.get('t_dtw',-1)>=0]
    assert tokens,c['line_id']
    return {'line':c['line_id'],'audio':sha,'wav':c['audio'],'receipt':str(dest.with_suffix('.json').relative_to(P)),'tokens':len(tokens),'model_sha256':MODEL_SHA,'method':'whisper token DTW base.en, flash attention disabled, lexical-token centres'}
with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
    result=[]
    for row in pool.map(run,items.items()):result.append(row);print(row['line'],row['tokens'],flush=True)
(out/'index.json').write_text(json.dumps(result,indent=2)+'\n')
print('Aligned',len(result),'exact audio clips with the pinned local DTW model.')
````

</details>

<details>
<summary>build/prepare_timings.py</summary>

````python
"""Advisory phrase intervals from the exact synthetic audio's pinned DTW tokens.
Never verifies cues. Explicit recognition substitutions are evidence, not script edits.
"""
import json,os,re
from pathlib import Path
P=Path(__file__).resolve().parents[1]
# V02_VOICE=takes prepares timings for the recorded narration; the default is the draft voice.
TAKES=os.environ.get('V02_VOICE')=='takes'
REVIEW=P/('build/review/recorded-voice' if TAKES else 'build/review')
DTW=(REVIEW/'dtw') if TAKES else P/'build/cues/dtw-aligned'
clean=lambda s:re.sub(r'[^a-z0-9]','',s.lower())
TAKES_SUBSTITUTIONS={
 'long/takes/v02-write-rule-02/focus-1':('all dollar per cell',1,'parcel recognized as per cell.'),
 'long/takes/v02-write-rule-05/focus-1':('packed parcels',1,'Select the first mention naming the second argument; the phrase is repeated later in the explanation.'),
 'long/takes/v02-write-rule-07/focus-2':('labeled parcels list',1,'US spelling labeled corresponds to spoken labelled.'),
 'long/takes/v02-derivation-04/focus-1':('conclusion',1,'The recognizer has no token for the article; the noun occurs once in the clip.'),
 'long/takes/v02-derivation-07/focus-1':('labeled parcels list',1,'US spelling labeled corresponds to spoken labelled.'),
 'long/takes/v02-parcel-b-01/focus-1':('game gives false',1,'claim recognized as game.'),
 'long/takes/v02-parcel-b-03/focus-2':('labelled records',1,'label recognized as labelled.'),
 'long/takes/v02-closing-01/focus-1':('shad variable',1,'shared recognized as shad.'),
}
def silences(path,min_gap=0.2):
    """Measured quiet stretches of a take: 10 ms RMS windows below 6% of its 95th-percentile level."""
    import array,math,wave
    with wave.open(path) as h:
        assert h.getnchannels()==1 and h.getsampwidth()==2
        rate=h.getframerate();a=array.array('h');a.frombytes(h.readframes(h.getnframes()))
    win=rate//100;env=[math.sqrt(sum(x*x for x in a[i:i+win])/win) for i in range(0,len(a)-win,win)]
    level=sorted(env)[int(len(env)*.95)];quiet=level*.06;out=[];begin=None
    for i,e in enumerate(env+[level]):
        if e<quiet and begin is None:begin=i
        if e>=quiet and begin is not None:
            if (i-begin)/100>=min_gap:out.append((begin/100,i/100))
            begin=None
    return out
substitutions={
 'long/draft/v02-shop-01/focus-1':('parcel',1,'Recognizer omitted the letter A; the first parcel-name occurrence is A in the exact scripted clip.'),
 'long/draft/v02-plain-rule-02/focus-1':('packing conformation',1,'confirmation recognized as conformation.'),
 'long/draft/v02-plain-rule-02/focus-2':('label conformation',1,'confirmation recognized as conformation.'),
 'long/draft/v02-write-rule-01/focus-1':('DollarParsal',1,'dollar parcel recognized as a phonetic compound.'),
 'long/draft/v02-write-rule-05/focus-1':('packed parcels',1,'Select the first mention naming the second argument; the phrase is repeated later in the explanation.'),
 'long/draft/v02-write-rule-07/focus-2':('labeled parcels list',1,'US spelling labeled corresponds to spoken labelled.'),
 'long/draft/v02-first-evaluation-09/focus-1':('for it',1,'The spoken letter A was recognized as it.'),
 'long/draft/v02-first-evaluation-09/focus-2':('these label no',1,'The recognizer substitutes these label no for the scripted B’s label note.'),
 'long/draft/v02-derivation-06/focus-1':('pack parcels list',1,'Phonetic recognition of packed-parcels list.'),
 'long/draft/v02-derivation-07/focus-1':('labeled parsals list',1,'Phonetic recognition of labelled-parcels list; DTW corrects early segment boundaries.'),
 'long/draft/v02-controls-07/focus-1':('derides checklist membership',1,'derives recognized as derides.'),
 'long/draft/v02-some-03/focus-1':('parcel',1,'Recognizer omitted A in the first parcel-name occurrence.'),
 'long/draft/v02-some-03/focus-2':('parcel be',1,'B recognized as be.'),
 'short/draft/v02-short-story-02/focus-1':('parcel',1,'Recognizer omitted A after the single parcel-name occurrence.'),
 'short/draft/v02-short-story-04/focus-1':('for her',1,'The spoken letter A was recognized as her.'),
}
# Recognition of the recorded voice differs from the draft voice, so it has its own table.
if TAKES:substitutions=TAKES_SUBSTITUTIONS
result={};unresolved=[];comparisons=[]
for cut in ['long','short']:
 for c in json.loads((REVIEW/f'{cut}-cues.json').read_text()):
  path=DTW/f"{c['binding']['audio']}.json"
  data=json.loads(path.read_text())
  tokens=[{'text':t['text'],'norm':clean(t['text']),'at':t['t_dtw']/100} for s in data['transcription'] for t in s['tokens'] if t.get('t_dtw',-1)>=0 and t['id']<50257 and clean(t['text'])]
  def find(phrase):
   wanted=clean(phrase);matches=[]
   for i in range(len(tokens)):
    text=''
    for j in range(i,len(tokens)):
     text+=tokens[j]['norm']
     if text==wanted:matches.append((i,j));break
     if not wanted.startswith(text):break
   return matches
  heard=c['phrase'];matches=find(heard);occ=c.get('occurrence') or 1;basis='Exact phrase in pinned DTW recognition.'
  if len(matches)!=1 and c['key'] in substitutions:
   heard,occ,basis=substitutions[c['key']];matches=find(heard)
  if not matches or (len(matches)!=1 and c['key'] not in substitutions and c.get('occurrence') is None) or len(matches)<occ:
   unresolved.append({'key':c['key'],'phrase':c['phrase'],'tried':heard,'text':''.join(t['text'] for t in tokens),'matches':matches});continue
  i,j=matches[occ-1]
  first,last=tokens[i]['at'],tokens[j]['at']
  start=(tokens[i-1]['at']+first)/2 if i else max(0,first-.12)
  end=(last+tokens[j+1]['at'])/2 if j+1<len(tokens) else last+.16
  note=''
  if TAKES:
   # A token midpoint can fall in the silence beside the phrase. Move that edge to the
   # measured speech boundary: the phrase starts when the silence before it ends, and
   # ends when the silence after it starts. Only a silence that touches the phrase counts.
   quiet=silences(c['audio'])
   for a,b in quiet:
    if a<=start<b and b<=first+.05:note+=f' Start moved from {start:.2f} to the end of the measured silence {a:.2f}-{b:.2f}.';start=b
    if a<end<=b and a>=last-.05 and a>start:note+=f' End moved from {end:.2f} to the start of the measured silence {a:.2f}-{b:.2f}.';end=a
  duration=c['binding']['samples']/c['binding']['rate'];start=max(0,min(start,duration-.03));end=min(duration,max(end,start+.03))
  result[c['key']]={'audio':c['binding']['audio'],'start':round(start,4),'end':round(end,4),'phrase':c['phrase'],'recognized':heard,'tokens':tokens[i:j+1],'receipt':str(path.relative_to(P)),'reason':basis+' Boundaries are midpoints between adjacent lexical token centres; no uniform reading-speed estimate.'+note+' Advisory only.'}
  old=c.get('interval');old_start=old['start']/c['binding']['rate'] if old else None
  comparisons.append({'key':c['key'],'native_start':old_start,'dtw_start':start,'delta':None if old_start is None else round(start-old_start,3)})
((REVIEW/'advisory-overrides.json') if TAKES else (P/'build/advisory-overrides.json')).write_text(json.dumps(result,indent=2)+'\n')
(REVIEW/'timing-comparison.json').write_text(json.dumps(comparisons,indent=2)+'\n')
(REVIEW/'timing-unresolved.json').write_text(json.dumps(unresolved,indent=2)+'\n')
print('Prepared',len(result),'advisory timings. Unresolved:',json.dumps(unresolved,indent=2))
````

</details>

<details>
<summary>build/check_art.mjs</summary>

````javascript
import {chromium} from '/home/dhilipsiva/projects/dhilipsiva/nibli-learn/node_modules/playwright/index.mjs';
import {readdir,readFile,writeFile} from 'node:fs/promises';
import {fileURLToPath} from 'node:url';
const dir=fileURLToPath(new URL('./cards/',import.meta.url));
const browser=await chromium.launch({executablePath:'/nix/store/fqriy9vihl23l4md56qgiks8z0d2xdlw-chromium-156.0.8078.79/bin/chromium',headless:true});
const page=await browser.newPage();const results=[];
try{for(const name of await readdir(dir)){
 if(!name.endsWith('.svg')||name.startsWith('pg-'))continue;
 await page.goto('file://'+dir+name);await page.evaluate(()=>document.fonts.ready);
 const result=await page.evaluate(()=>{const svg=document.querySelector('svg');const w=svg.viewBox.baseVal.width,h=svg.viewBox.baseVal.height;return [...document.querySelectorAll('text')].map(e=>{const b=e.getBBox();return {id:e.id,text:e.textContent,box:[b.x,b.y,b.width,b.height],outside:b.x<0||b.y<0||b.x+b.width>w||b.y+b.height>h};});});
 results.push({name,text:result,problems:result.filter(x=>x.outside)});
}
await writeFile(new URL('./art-checks.json',import.meta.url),JSON.stringify(results,null,2)+'\n');console.log(JSON.stringify(results.filter(x=>x.problems.length),null,2));
}finally{await browser.close();}
````

</details>

<details>
<summary>build/check_delivery.py</summary>

````python
"""Inspect the actual diagnostic MP4s, native receipts and caption timelines."""
import hashlib,json,math,os,re,subprocess,tomllib
from pathlib import Path
P=Path(__file__).resolve().parents[1]
# V02_VOICE=takes inspects the recorded-voice review copies; the default is the draft voice.
VOICE='takes' if os.environ.get('V02_VOICE')=='takes' else 'draft'
out=P/('build/review/recorded-voice' if VOICE=='takes' else 'build/review');config=tomllib.loads((P/'vidni.toml').read_text())
def run(args):
    r=subprocess.run(args,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
    if r.returncode:raise RuntimeError(' '.join(args)+': '+r.stderr[-2000:])
    return r
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
reports=[]
for cut,size in [('long',[1920,1080]),('short',[1080,1920])]:
    video=P/f'build/cues/full/{cut}-{VOICE}.review.mp4'
    receipt=json.loads(video.with_suffix('.json').read_text())
    timeline=json.loads((out/f'{cut}-timeline.json').read_text())
    probe=json.loads(run(['ffprobe','-v','error','-show_streams','-show_format','-of','json',str(video)]).stdout)
    vs=next(s for s in probe['streams'] if s['codec_type']=='video')
    au=next(s for s in probe['streams'] if s['codec_type']=='audio')
    assert vs['codec_name']=='h264' and [vs['width'],vs['height']]==size
    assert vs['avg_frame_rate']=='30/1' and vs['pix_fmt']=='yuv420p'
    assert au['codec_name']=='aac' and au['sample_rate']=='48000' and au['channels']==2
    assert abs(float(probe['format']['duration'])-timeline['total'])<.06
    assert receipt['frames']==math.ceil(timeline['total']*30)
    # The native encoder uses -shortest. Audio can end between video frames,
    # trimming one final tail frame; require every caption and cue to finish
    # before the actual encoded video ends.
    trim=receipt['frames']-int(vs['nb_frames']);assert trim in (0,1)
    assert sha(video)==receipt['sha256'] and receipt['kind']=='diagnostic-review' and receipt['voice']==VOICE
    assert len(receipt['emphasis'])=={'long':64,'short':9}[cut]
    for cue in receipt['emphasis']:
        assert cue['interval'] is not None and not cue['verified']
        assert cue['display']['cutoff'] is None
        assert cue['display']['end']-cue['display']['start']>=90
        assert cue['display']['end']<=int(vs['nb_frames'])
        assert sha(Path(cue['audio']))==cue['binding']['audio']
    captions=[c for s in timeline['sentences'] for c in s['cues']]
    last=0
    for c in captions:
        assert last<=c['start']<c['end']<=float(vs['duration'])
        assert len(c['text'])/(c['end']-c['start'])<=20
        last=c['end']
    for ext in ['srt','vtt']:
        f=video.with_suffix('.en.'+ext)
        content=f.read_text()
        assert content.count(' --> ')==len(captions)
        for c in captions:assert c['text'] in content
    run(['ffmpeg','-v','error','-i',str(video),'-f','null','-'])
    level=run(['ffmpeg','-hide_banner','-nostats','-i',str(video),'-vn','-af','loudnorm=I=-14:TP=-2:LRA=11:print_format=json','-f','null','-']).stderr
    loud=json.loads(level[level.rfind('{'):level.rfind('}')+1])
    assert -14.8<=float(loud['input_i'])<=-13.2,loud
    assert float(loud['input_tp'])<=-1.5,loud
    chapters=[];seen=set()
    for s in timeline['sentences']:
        if s['segment'] in seen:continue
        seen.add(s['segment']);title=config['sections'][s['segment']]['chapter']
        if title:
            at=0 if not chapters else math.floor(s['start'])
            chapters.append({'seconds':at,'timestamp':f'{at//60:02}:{at%60:02}','title':title,'exact_start':s['start']})
    (out/f'{cut}-chapters.json').write_text(json.dumps(chapters,indent=2)+'\n')
    if chapters:(P/f'build/cues/full/{cut}-{VOICE}.review.chapters.txt').write_text('\n'.join(f"{c['timestamp']} {c['title']}" for c in chapters)+'\n')
    reports.append({'cut':cut,'path':str(video.relative_to(P)),'sha256':receipt['sha256'],'bytes':video.stat().st_size,'timeline_seconds':timeline['total'],'container_seconds':float(probe['format']['duration']),'size':size,'fps':30,'rendered_frames':receipt['frames'],'encoded_frames':int(vs['nb_frames']),'mux_trimmed_tail_frames':trim,'video':'h264/yuv420p','audio':'aac/48000Hz/stereo','integrated_lufs':float(loud['input_i']),'true_peak_dbtp':float(loud['input_tp']),'captions':len(captions),'max_caption_cps':max(len(c['text'])/(c['end']-c['start']) for c in captions),'chapters':chapters,'cues':len(receipt['emphasis']),'all_highlights_at_least_3_seconds':True,'full_decode':True,'owner_verified':False})
    print(cut,'PASS',timeline['total'],'seconds,',len(captions),'captions,',loud['input_i'],'LUFS',flush=True)
assert not (P/'cues.lock').exists()
(out/'media-checks.json').write_text(json.dumps({'all_passed':True,'kind':('recorded-voice' if VOICE=='takes' else 'synthetic')+' diagnostic review, not owner approval','cuts':reports},indent=2)+'\n')
````

</details>

<details>
<summary>build/recorded_voice_notes.py</summary>

````python
"""Record the recorded-voice review in notes.md, in place, from its receipts.

The writing-stage generator (author.py) is not replayed once the owner has
ratified and recorded. This edits notes.md only: the status paragraph, the
recorded-voice section and the recipe archive. Running it again is a no-op.
"""
import json
from pathlib import Path
P=Path(__file__).resolve().parents[1];R=P/'build/review/recorded-voice'
qa=json.loads((R/'media-checks.json').read_text());assert qa['all_passed'] and qa['kind'].startswith('recorded-voice')
long,short=qa['cuts'];assert long['cut']=='long' and short['cut']=='short'
metrics=json.loads((R/'take-metrics.json').read_text())
after=json.loads((R/'owner-files.after.json').read_text());assert set(after['changed'])<={'notes.md'} and not after['added']
overrides=json.loads((R/'advisory-overrides.json').read_text())
native=json.loads((R/'native-frames.json').read_text())
assert all(not c['cutoffs'] and c['full_minimum']==c['cues']==c['proposed'] for c in native)
hear=[l.split() for l in (R/'hear.log').read_text().splitlines() if l.startswith('v02-')]
assert len(hear)==len(metrics) and all(float(h[2])>=.9 for h in hear)
clock=lambda t:f"{int(t//60)}:{t%60:06.3f}"
number={8:'eight',9:'nine'}
main=sum('short-story' not in m['id'] for m in metrics);vertical=len(metrics)-main
assert {(m['rate'],m['channels'],m['width']) for m in metrics}=={(48000,1,2)} and not sum(m['clipped'] for m in metrics)
peaks=sorted(m['peak_dbfs'] for m in metrics)
moved_start=sum(' Start moved' in v['reason'] for v in overrides.values());moved_end=sum(' End moved' in v['reason'] for v in overrides.values())
exact=sum(v['reason'].startswith('Exact phrase') for v in overrides.values());cues=long['cues']+short['cues'];assert len(overrides)==cues
chapters='\n'.join(f"{c['timestamp']} {c['title']}" for c in long['chapters'])
section=f'''<a id="recorded-voice-review"></a>
## Recorded-voice review — 2026-11-20

The owner ratified both revised cuts and recorded every line in one session:
{main} main lines and {number.get(vertical,vertical)} Short lines. Vidni reports {len(metrics)} current takes, none
stale and none missing. The local speech check hears all {len(metrics)} as scripted, with a
lowest score of {min(float(h[2]) for h in hear):.2f} ([log](build/review/recorded-voice/hear.log)); it is advisory and
is not a pronunciation or pacing review. Every kept take is 48 kHz mono 16-bit
PCM without a full-scale sample, and peaks run from {peaks[0]:.1f} to {peaks[-1]:.1f} dBFS
([take metrics](build/review/recorded-voice/take-metrics.json)). No retake was requested.

| Cut | Review video | Runtime | Captions |
|---|---|---|---|
| Main · 1920×1080 | [Watch the recorded main review](build/cues/full/long-takes.review.mp4) | {clock(long['timeline_seconds'])} | [SRT](build/cues/full/long-takes.review.en.srt) · [VTT](build/cues/full/long-takes.review.en.vtt) |
| Short · 1080×1920 | [Watch the recorded Short review](build/cues/full/short-takes.review.mp4) | {clock(short['timeline_seconds'])} | [SRT](build/cues/full/short-takes.review.en.srt) · [VTT](build/cues/full/short-takes.review.en.vtt) |

Both videos visibly say **REVIEW COPY — HIGHLIGHT TIMING UNVERIFIED**. They are
diagnostic previews of the recorded narration, not final exports.

**Measured main chapters, recorded voice** ([text file](build/cues/full/long-takes.review.chapters.txt)):

```text
{chapters}
```

**Highlights.** Recorded audio needs its own cue bindings; nothing carries over
from the synthetic voice. All {cues} highlights ({long['cues']} main, {number.get(short['cues'],short['cues'])} Short) have advisory
intervals from the pinned token alignment of the exact takes, and none is
owner-verified. {exact} phrases were recognized as scripted. The other {cues-exact} use
explicit substitutions listed in the recipe, such as “per cell” for “parcel”
and “shad variable” for “shared variable”; one of them selects the first of two
mentions of “Packed Parcels”. A token midpoint can fall in the silence beside a
phrase, so that edge is moved to the measured speech boundary of the take: {moved_start}
starts and {moved_end} ends moved, each recorded with its before and after value in the
[timing evidence](build/review/recorded-voice/advisory-timing-evidence.json).
Every highlight holds for at least three seconds, with no scene or camera
cutoff, and the native geometry checks pass on the boundary and midpoint frames
([frame receipt](build/review/recorded-voice/native-frames.json)). In the rendered
audio, every highlight starts where its take's interval places it.

**Media.** The [recorded-voice media checks](build/review/recorded-voice/media-checks.json)
pass full decoding, exact MP4 hashes, H.264/30 fps/yuv420p and AAC/48 kHz/stereo.
Main/Short audio measures {long['integrated_lufs']:.2f}/{short['integrated_lufs']:.2f} LUFS and {long['true_peak_dbtp']:.2f}/{short['true_peak_dbtp']:.2f} dBTP. The
{long['captions']}/{short['captions']} caption cues are ordered and stay below 20 characters per second
(maxima {long['max_caption_cps']:.2f}/{short['max_caption_cps']:.2f}).

| Cut | SHA-256 |
|---|---|
| Main | `{long['sha256']}` |
| Short | `{short['sha256']}` |

No narration, scene, camera or recorded take was changed, and both cuts remain
ratified. A before/after hash inventory covers all {after['files']} owner files, {sum(1 for _ in (P/'takes').rglob('*') if _.is_file())} of
them recordings and their metadata: nothing changed apart from these notes
([after](build/review/recorded-voice/owner-files.after.json)). No cue verification,
release approval, upload or posting was performed.

**Next.** The owner watches both recorded review videos with audio and either
confirms that the highlights and their timing are right or names what to
change. A confirmation is relayed with `vidni cues record-confirmation`, bound
to the two hashes above; `vidni final` then builds the exports, and the full
publishing copy package is prepared against the final timeline.

'''
recipe='''Recorded-voice review, after the owner ratifies and records (never replay
`author.py` at that point):

```sh
export VIDNI_ASSISTANT=1 VIDNI_VOICES="$VIDNI_PINNED_VOICES" V02_VOICE=takes
./target/release/vidni cues propose examples/nibli-v02-rules-and-claims --voice takes
./target/debug/vidni-v02-audit dump takes
python3 examples/nibli-v02-rules-and-claims/build/align_dtw.py
python3 examples/nibli-v02-rules-and-claims/build/prepare_timings.py
./target/debug/vidni-v02-audit apply takes
./target/debug/vidni-v02-audit frames takes
./target/release/vidni cues preview examples/nibli-v02-rules-and-claims --voice takes
python3 examples/nibli-v02-rules-and-claims/build/check_delivery.py
python3 examples/nibli-v02-rules-and-claims/build/recorded_voice_notes.py
```

'''
notes=(P/'notes.md').read_text()
# Status paragraph: one sentence, replaced on a rerun.
mark='The owner has since ratified both cuts and recorded all'
status=f"{mark} {len(metrics)} lines. The\n[recorded-voice review](#recorded-voice-review) copies measure **{clock(long['timeline_seconds'])}** and\n**{clock(short['timeline_seconds'])}** and await the owner's highlight confirmation.\n"
anchor='[review revision](#review-revision) lists what changed.\n'
assert notes.count(anchor)==1
head,tail=notes.split(anchor)
if tail.startswith(mark):tail=tail[tail.index('\n\n'):].lstrip('\n');tail='\n'+tail
notes=head+anchor+status+tail
# Recorded-voice section and recipe: placed before the recipe archive, replaced on a rerun.
archive='<a id="production-recipes"></a>\n## Archived production recipes\n\n'
assert notes.count(archive)==1
body=notes[:notes.index(archive)]
if '<a id="recorded-voice-review"></a>' in body:body=body[:body.index('<a id="recorded-voice-review"></a>')]
start='Recorded-voice review, after the owner ratifies and records'
assert start not in body
files=[('build/run_fixtures.py','python'),('build/capture_playground.mjs','javascript'),('build/author.py','python'),('build/finalize_notes.py','python'),('build/native/Cargo.toml','toml'),('build/native/src/main.rs','rust'),('build/align_dtw.py','python'),('build/prepare_timings.py','python'),('build/check_art.mjs','javascript'),('build/check_delivery.py','python'),('build/recorded_voice_notes.py','python')]
notes=body.rstrip('\n')+'\n\n'+section+archive+recipe
for name,lang in files:
    notes+=f'<details>\n<summary>{name}</summary>\n\n````{lang}\n'+(P/name).read_text().rstrip()+'\n````\n\n</details>\n\n'
(P/'notes.md').write_text(notes)
print('Recorded-voice review saved in notes.md with',len(files),'archived recipes.')
````

</details>

