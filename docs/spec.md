# `vidni.toml`, schema 1

Each video is one project folder, and its `vidni.toml` describes everything about it:
- the script, line by line
- what each line shows
- the voices
- the cuts made from it

This page describes every table and key. `vidni validate` checks a file against it, and `vidni schema` prints the same rules as a JSON Schema, committed at [`vidni.schema.json`](vidni.schema.json). Editors that read the schema (Taplo, and Even Better TOML in VS Code) offer completion and inline checks when the file starts with:

```toml
#:schema ../../docs/vidni.schema.json
```

A complete example is [`examples/nibli-syllogism/vidni.toml`](../examples/nibli-syllogism/vidni.toml). Here is the smallest valid project:

```toml
schema = 1

[meta]
title = "A minimal project"
default_scene = "page"

[sections.main]

[[lines]]
id = "main-1"
section = "main"
show = "Hello."

[scenes.page]
kind = "web"
url = "https://example.com"

[[cuts]]
id = "long"
aspect = "16:9"
```

Later items make use of keys that schema 1 already fixes. For example, capture (P3-VIDNI-04) acts on targets and cameras, and the compositor (P3-VIDNI-02) frames them. Their meaning is set here, so a project written today keeps working.

## Starting a project: `vidni new`

`vidni new <name>` makes a folder called `<name>` in the current folder, holding a project that renders as it is:

```text
<name>/vidni.toml   two lines in one section, one web scene, a 16:9 cut and a 9:16 cut
<name>/page.html    the page the scene shows
<name>/.gitignore   keeps build/, out/ and takes/ out of git
```

- **The name** is an ID ([IDs](#ids)), and the title is made from it: `my-first-video` is titled "My first video".
- **The scene** is a page in the project folder, so `vidni new demo && vidni draft demo` renders with no edits and no network.
- **A folder that is there already** is left as it is, and the command fails.

`vidni new <name> --from outline.md` takes the script from an outline written in Markdown:
- **The title** is the first heading, when it is a `#` heading that comes before everything else.
- **Every other heading** starts a section, and is its label. The section's ID is made from the heading's first four words, so `## How it works, briefly` is `how-it-works-briefly`. A heading with no letter or digit that an ID can hold is `section-<n>`, and a second section of the same name ends in `-2`.
- **Each paragraph** is a line, and so is each list item. A line's ID is its section's ID and its number in the section, from 1: `how-it-works-briefly-1`. Lines that come before any heading go in a section called `intro`.
- **Inline marks** are dropped. A link or an image is its text, and the marks of bold (`**`, `__`) and of code go. Fenced code is left out, and a quote loses its `>`.

Every line shows the starter page. An outline with no paragraph and no list item makes no project.

## Top level

| Key | Type | Required | What it is |
|---|---|---|---|
| `schema` | integer | yes | The schema version, `1` |
| `meta` | table | yes | What the video is |
| `voice` | table | no | The voices that speak the lines |
| `timing` | table | no | The pauses around the lines |
| `lint` | table | no | Wording checks |
| `style` | table | no | The look of every cut |
| `sections.<id>` | tables | no | Runs of lines under one label |
| `lines` | array of tables | yes | The script, in order |
| `scenes.<id>` | tables | no | What lines show |
| `cuts` | array of tables | yes | The videos made from the script |
| `music` | table | no | Background music |
| `expectations` | array of tables | no | Text a web scene must show |

A key that schema 1 doesn't define is an error. A misspelled key never passes silently.

## IDs

Lines, sections, scenes, targets, cameras and cuts have IDs. An ID is lowercase letters and digits in words joined by single hyphens, like `intro-1` or `row-eats`.

A line's ID is permanent. Takes, captions and per-cut state are keyed by it (P2-VIDNI-03), so rename a line only when you mean to discard its takes.

## `[meta]`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `title` | string | required | The video's title. It must not be empty. |
| `subtitle` | string | none | A short line shown with the title, such as a series name |
| `language` | string | `"en"` | The script's language, as a BCP 47 tag of letters, digits and hyphens, such as `en` or `pt-BR`. It becomes part of the caption files' names. |
| `strict` | boolean | `false` | Whether every line that states a fact must name its `source`. Set `claim = false` on a line that states none. |
| `default_scene` | string | none | The scene a line shows when it names none |

## `[voice]`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `draft` | string | `"piper:en_US-joe-medium"` | The synthetic draft voice: a voice pack, as `<engine>:<model>` (see [The draft voice](#the-draft-voice)) |
| `seed` | integer | `0` | Where a line's first draft takes its seeds from. The seed it keeps goes into `voice.lock`, so changing this changes only the lines drafted afresh |
| `length_scale` | number | `1.0` | The draft voice's pace; above 1 is slower. It must be above 0. |
| `read_as` | array of `{ text, say }` | empty | How a reader says symbols that the captions show, such as `{ text = "#1", say = "number one" }`. The recorder's prompter shows these, and takes are scored against them. |
| `respell` | array of `{ text, say }` | empty | Spellings the draft voice says better, applied in order to what it speaks, such as `{ text = "theorem prover", say = "theorem proover" }` |
| `heard_as` | array of `{ heard, means }` | empty | Words the listening check hears spelled differently from the script for the same sound, such as `{ heard = "nibley", means = "nibli" }` (see [The listening check](#the-listening-check-vidni-hear)). `heard` is one word of letters, digits and apostrophes, listed once. No output depends on these. |

## `[timing]`

All values are seconds, zero or more.

| Key | Default | Meaning |
|---|---|---|
| `lead_in` | `0.6` | Silence before the first line |
| `line_gap` | `0.28` | The pause between two lines of one section |
| `section_gap` | `0.75` | The pause between sections |
| `tail` | `2.4` | Silence after the last line |

## `[lint]`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `banned` | array of strings | empty | Regular expressions, matched without regard to case against each line's `show` and `say`. A match is an error that quotes the wording and the pattern. Literal strings (`'\bpublished\b'`) keep backslashes as they are. |

## `[style]`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `theme` | string | `"default"` | `default`, or the path of a theme folder, from the project folder |

### Themes

A theme is the look of a cut: its colours, its fonts, and the mark before a section's label. Where things sit and how big they are is the same in every theme. The built-in `default` theme is dark and plain, in IBM Plex. A theme of your own is a folder with a `theme.toml`:

```toml
name = "quine"

[colors]
frame = "#0b0b10"     # behind everything: cards, and the bands around a 9:16 view
rule = "#2a2a38"      # the rules between the bands
ink = "#f4f3ef"       # text
muted = "#908f9a"     # text that matters less
accent = "#f2542d"    # the mark before a label, and a card's rule
panel = "#07070ade"   # the box behind captions; the last two digits are how solid it is
label = "#07070ad2"   # the box behind a section's label
edge = "#3a3a4a"      # the edge of both boxes

[fonts]
text = "ibm-plex-sans-bold"     # captions, and a card's running text
mono = "ibm-plex-mono-bold"     # labels and small type
display = "space-grotesk-bold"  # titles

[label]
mark = "//"           # written before a label, in the accent colour; "" for none
```

- **Colours** are `#rrggbb`; `panel` and `label` may be `#rrggbbaa`.
- **Fonts** are named by the IDs vidni pins them under ([Fonts](#fonts)): `ibm-plex-sans-bold`, `ibm-plex-mono-bold` and `space-grotesk-bold`.
- **A theme isn't part of what a cut shows and says,** so changing it leaves a cut ratified. It is part of a build's inputs: the cut is built again, and a release of it is a new one.
- **Highlight boxes** on a web scene's stills keep their ember outline in every theme: they are in the still as it was captured.

That file is [`examples/themes/quine`](../examples/themes/quine/theme.toml), the look of the nibli example.

## `[sections.<id>]`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `label` | string | `""` | Shown on screen while the section plays, and used for chapters. Empty for none. |

Sections come in the order of their first line, and a section's lines must be together. A section with no lines is an error.

## `[[lines]]`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `id` | string | required | The line's permanent ID |
| `section` | string | required | The section it belongs to |
| `show` | string | required | The caption: the line as it is shown. It must not be empty. |
| `say` | string | `show` | What is said, when it differs from what is shown |
| `source` | string | none | Where the line's facts come from. It must not be empty. |
| `claim` | boolean | `true` | Whether the line states a fact. In a strict project, a line that does needs a `source`. |
| `scene` | string | `meta.default_scene` | The scene the line shows |
| `do` | array of actions | empty | Web scenes: actions before the line is captured |
| `highlight` | array of strings | empty | Web scenes: targets boxed while the line plays |
| `camera` | string | the scene's `default_camera` | Web scenes: the framing for 16:9 cuts |
| `camera_vertical` | string | see [Cameras](#cameras) | Web scenes: the framing for 9:16 cuts |
| `overlay` | string | none | A transparent page scene drawn over the line's picture |
| `only` | array of cut IDs | empty: every cut | The cuts the line is only for, such as a short's hook. See [Several cuts](#several-cuts). |

`do`, `highlight`, `camera` and `camera_vertical` need a web scene. A line on a page scene shows that page full-frame.

## Takes and spoken words

A line's takes and captions are filed under its ID and a hash of its spoken words. Its draft clips are filed under its ID and a key of what the draft voice is given ([The draft voice](#the-draft-voice)).
- **The spoken form** is `say`, or else `show` with `voice.read_as` applied.
- **Its words** are the spoken form in lowercase, with apostrophes joined (`it's` becomes `its`) and every other punctuation mark removed.
- **The hash** is the first 16 hex digits of the words' SHA-256.

So an edit to a line's punctuation or capitals keeps its audio, and an edit to its words makes the audio stale. Inserting, removing or moving other lines never touches it.

The creator's takes live in the project folder, out of git (DECISIONS.md, D4):

```text
takes/<line-id>/<hash>.wav          the kept take for that wording
takes/<line-id>/raw/<hash>-<n>.wav  every raw take for it, numbered from 1
takes/_room/                        room tone from the mic check
```

`vidni takes` lists each line's take as one of:
- **current:** there is a take for the line's words.
- **stale:** there are takes only for other words. They stay on disk, in case the edit is undone.
- **missing:** there are no takes at all.

`vidni takes --cut ID` lists only that cut's lines; repeat it for more cuts.

`vidni takes --link DIR` lays the current takes out as `000.wav`, `001.wav` … in script order, as symbolic links, for tools that read numbered files. With `--cut`, it lays out that cut's takes.

### What is kept of a take

A recording goes straight into a raw take: 16-bit PCM with one channel, at the microphone's rate, which is 48 kHz where the microphone offers it. A raw take is written once and never changed. The kept take is made from it:
1. **The guard.** A raw take shorter than 0.8 s is discarded, and so is one that never gets louder than −40 dBFS: it has no speech in it. The raw file stays either way.
2. **The keypresses.** The first 0.20 s and the last 0.10 s are dropped, where the keys that started and stopped the take are heard.
3. **The rumble.** What is below 70 Hz is filtered out.
4. **The silence.** The words begin where the level first rises above −45 dBFS, and end where it last does. The level is the RMS over the 20 ms up to each sample. 0.15 s is kept before the words and 0.25 s after them. A take with less than 0.4 s left is discarded.

The trimmed take waits beside the kept one as `<hash>.wav.new` until it is kept or discarded, so a line's take is replaced only by one you chose.

**Room tone.** The mic check records the room for 3 s with nobody speaking. Its middle, without the first 0.3 s and the last 0.2 s, is kept as `takes/_room/tone.wav`, in the place of the one before. A room louder than −55 dBFS is called noisy.

The trim is the one the reference's recorder did with ffmpeg, worked out in ffmpeg's arithmetic, so the same raw take gives the same samples (DECISIONS.md, T10).

## The draft voice

`vidni draft` begins by speaking every line in the draft voice, `voice.draft`, so that a cut can be watched before any take is recorded. `vidni speak` does that step alone.

**Voice packs.** A draft voice is a voice pack: a model's files, pinned by size and SHA-256, plus what its model card says about the licence. `vidni voices` lists the packs vidni ships, each with its licence, whether that allows commercial use, and the model card it was read from. `vidni voices fetch <id>` downloads a pack into the voice store and checks every file. The store is `$VIDNI_VOICES`, else `$XDG_DATA_HOME/vidni/voices`, else `~/.local/share/vidni/voices`, and a pack lives in `<engine>/<model>/` there with a `pack.toml` holding its description. A creator's own voice is a folder of the same shape, with a `pack.toml` they write:

```toml
id = "piper:my-voice"
language = "en_GB"
licence = "proprietary"            # an SPDX ID where there is one
commercial = true                  # whether the licence allows commercial use
notes = "Recorded and trained by me."
source = "notes.md"                # where the licence is stated
checked = "2026-11-13"
files = [
  { name = "my-voice.onnx", size = 63221984, sha256 = "…" },
  { name = "my-voice.onnx.json", size = 5465, sha256 = "…" },
]
```

**Takes and seeds.** Piper samples, so a take depends on a seed as well as on the text. vidni runs Piper with a seed (vidni's flake patches it to take `--seed`), and the same seed gives the same PCM.
- A line's first draft speaks three takes, with seeds made from `voice.seed`, the line's ID and its key. It keeps the take of median length, since a take much longer or shorter than the others has usually stretched or dropped a word.
- The kept seed goes into `voice.lock`, beside `vidni.toml`. Commit it: every later draft speaks the line from that seed.
- A line is drafted afresh when what the voice is given changes: its `say` or `show` (punctuation too, since it changes how the voice reads), `voice.respell`, the voice pack, or `voice.length_scale`.
- To hear another take of a line, write another seed into its entry in `voice.lock`.

**The cache.** Clips are cached in `build/voice/<line-id>/<key>-<seed>.wav`, where the key is the first 16 hex digits of a SHA-256 over what the voice is given (the line's `say` or `show`, with `voice.respell` applied), the pack's ID and file hashes, and `voice.length_scale`. A draft speaks only what the cache doesn't hold. Clearing `build/` and drafting again gives the same PCM, from `voice.lock`.

**Licences.** A voice whose licence doesn't allow commercial use may still be drafted with, but its audio can't go into a cut that is used commercially. `vidni validate`, `vidni speak` and `vidni draft` warn about such a voice, and about one whose licence vidni can't find.

## The recording sheet: `vidni plan`

`vidni plan` prints what there is to record: every line in script order, under its section's label, with the words to say.

```text
My first video: the recording sheet
2 lines in 1 section, 9 words; 0 recorded, 0 stale, 2 to record; about 0:04 in the draft voice, without the pauses

Introduction
  [ ] intro-1    1.7 s  Hello, and welcome.
  [ ] intro-2    2.3 s  This video was made with vidni.
```

- **The words** are the line's spoken form ([Takes and spoken words](#takes-and-spoken-words)): what you read aloud, not what the captions show.
- **The mark** is `[x]` for a line with a current take, `[~]` for a line whose takes are all of other words, and `[ ]` for a line with none.
- **The time** is how long the draft voice took over the line as it is now, as a pace to read at. A line the draft voice hasn't spoken yet has `-`.
- **`--cut ID`**, which can be repeated, lists those cuts' lines only.

The sheet only reads the project. It speaks nothing and writes no file.

## Recording: `vidni record`

`vidni record` is the recorder, in the terminal. It shows one line at a time, with the words to say, and takes single keys, with no Enter.

| When | Keys |
|---|---|
| Ready | `space` record · `p` play the saved take · `s` play the section's takes · `←` `→` move between lines · `q` quit |
| Recording | `space` or `enter` stop · `esc` cancel |
| Review | `enter` keep and go on · `p` play · `r` record again · `esc` discard · `q` keep and quit |

- **While a take is recorded**, a meter shows its level, and a timer runs beside the time the draft voice took over the line.
- **A take with nothing in it** is discarded with a message, and the line stays: one shorter than 0.8 s, or with no speech ([what is kept of a take](#what-is-kept-of-a-take)).
- **A take that passes** is trimmed and shown for review with its length and its peak, and a warning when it is too hot or too quiet. Where the listening check is at hand, the review says how the take matches the script, and what was heard when that isn't the script. It only advises.
- **Nothing replaces a line's take until you keep the new one.** Discarded or cancelled, the line is as it was. Every raw take that was stopped stays in `raw/`.
- **For a moment after the screen changes, keys are ignored** (0.4 s), so that a double press isn't taken for an answer.
- **The session resumes.** It starts at the first line with no take of its present words, and after a take is kept it moves to the next such line. A line whose take is of other words says so. A line that several cuts use is recorded once.

| Flag | What it does |
|---|---|
| `--cut ID` | The lines of that cut only; repeat for more |
| `--section ID` | One section's lines only |
| `--from LINE` | Start at that line |
| `--only LINE` | Record that line again; repeat for more. The session goes through them in script order |
| `--test` | The mic check: the room is recorded for three seconds, then you read a line, and both are measured. It keeps the room tone and records no line |

The microphone is the system's default input, or the one whose name holds `$VIDNI_MIC`. Takes play through the default output, or the one whose name holds `$VIDNI_SPEAKER`. `$VIDNI_RECORD_INPUT` puts a made-up input in the microphone's place, for tests and demonstrations: `silence`, `tone:<hz>:<level>` or `wav:<file>`.

`vidni record` needs a terminal, and exits with 2 without one.

## The listening check: `vidni hear`

`vidni hear` listens to each line's audio with whisper and says how close what it hears is to the script. It points you at a line that may have been read, or synthesized, differently from its words: a dropped word, a stumble, a name the draft voice garbles.

**It only advises.** Nothing it hears is written into an output or decides one, and it writes no file. A low score fails nothing: it means the line is worth a listen.

```text
$ vidni hear examples/nibli-syllogism
intro-1          take   1.00
intro-2          take   0.97
…
syllogism-1      take   0.80
                   heard:  Here is the silage some example 3 lines of Nibali KR.
                   script: Here's the syllogism example: three lines of nibli KR.
…
examples/nibli-syllogism/vidni.toml: 55 heard as scripted, 2 worth a listen (below 0.9), 0 with nothing to hear, 0 with no audio
```

- **What it listens to.** A line's current take, or else the clip the last draft kept for the line's present wording. A stale take isn't the line's audio, and neither is a draft clip spoken from an earlier wording: a line with only those is listed as having no audio. `--draft` listens to the draft clips even where there are takes, and `--line ID`, which can be repeated, picks lines.
- **What it compares.** A take is compared with the line's spoken form: `say`, or `show` with `voice.read_as` applied. A draft clip is compared with what the draft voice was given: `say` or `show`, with `voice.respell` applied.
- **The score** runs from 0 to 1. Both texts are reduced to their letters, so case, spaces, punctuation and digits don't count. The score is the share of those letters that lie in runs the two texts have in common. A line whose score, as shown to two decimals, is below 0.90 is listed with what was heard.
- **`voice.heard_as`** lists words that whisper spells differently from the script for the same sound, as it usually does a made-up name: `{ heard = "nibley", means = "nibli" }`. Each such word is read as what it means, in the script and in what was heard, whatever its case. A number can be given its word the same way: `{ heard = "3", means = "three" }`.
- **Nothing to hear.** A clip that is empty, shorter than 0.3 seconds or silent (never above −60 dBFS) is reported as having nothing to hear. whisper isn't asked about it: it fails on an empty clip, and it hears words in silence. Its notes on sounds that aren't speech, such as `(machine whirring)`, are left out of what was heard, and a clip with nothing but those has nothing to hear either.
- **English only.** The model knows English, and the score compares English letters. For a script in another language (`meta.language`), `vidni hear` says that it can't run.

**What it needs.** `vidni hear` runs whisper.cpp's `whisper-cli` from the path, or the one `VIDNI_WHISPER` names, with whisper.cpp's `ggml-base.en.bin` model (148 MB). The model is pinned by SHA-256, and vidni refuses another file. It is looked for at `$VIDNI_WHISPER_MODEL`, else `$XDG_DATA_HOME/vidni/models/ggml-base.en.bin`, else `~/.local/share/vidni/models/ggml-base.en.bin`. vidni's Nix shell provides both.

**Exit status:**
- 0 when every line that has audio was listened to, whatever the scores
- 1 when the project is invalid, or `--line` names a line it doesn't have
- 2 when the check can't run, or when a clip can't be read. The other lines are still listened to.

## `[scenes.<id>]`

| Key | Type | Kinds | Meaning |
|---|---|---|---|
| `kind` | `"web"`, `"page"`, `"card"` or `"image"` | all | `web`: a page captured at a fixed viewport and framed by the virtual camera. `page`: an HTML file rendered at each cut's own size, for cards and overlays. `card`: a card drawn from one of the theme's templates; see [Cards](#cards). `image`: a picture in the project folder, framed by the virtual camera; see [Images](#images). |
| `url` | string | web, page, image | `web`: the page's address, or a file in the project folder. `page`: a file in the project folder, where `{aspect}` becomes `16x9` or `9x16`. `image`: a PNG or SVG file in the project folder |
| `viewport` | `[width, height]` | web | CSS pixels. Default `[1280, 720]`. |
| `scale` | integer | web, image | `web`: device pixels per CSS pixel. `image`: how many times its size an SVG is drawn at. Default 1. |
| `color_scheme` | `"light"` or `"dark"` | web | The colour scheme the page is asked for |
| `setup` | array of actions | web | Run once, after the page loads |
| `targets` | table of targets | web, image | Named parts of the page; see [Targets](#targets) |
| `cameras` | table of cameras | web, image | Named framings; see [Cameras](#cameras) |
| `default_camera` | string | web, image | The camera of a line that names none. Default `viewport`. |
| `vertical_follows_highlights` | number | web | In 9:16 cuts, a line whose highlights are at most this many CSS pixels wide is framed on them |
| `transparent` | boolean | page | Rendered on a transparent background, as an overlay |
| `size` | `{ landscape = [w, h], portrait = [w, h] }` | page | The rendered size for each orientation. Default: the cut's size. |

| `template` | `"title"`, `"section"`, `"bullets"`, `"quote"` or `"end"` | card | The template the card is drawn from |
| `title` | string | card | The card's title (`title`, `section`, `bullets`, `end`) |
| `subtitle` | string | card | A second, smaller line (`title`, `section`, `end`) |
| `items` | array of strings | card | The points of a `bullets` card |
| `quote` | string | card | The words of a `quote` card |
| `by` | string | card | Whose words they are |

A key that belongs to another kind is an error. A page scene's file must exist in the project folder, and so must a web scene's file when its `url` isn't an `http` or `https` address. The check runs for both orientations when the name holds `{aspect}`.

### Cards

A card scene is drawn by vidni from a template, in the theme's colours and fonts, at each cut's own size. It needs no page, no browser and no capture, and it is the same picture on every machine.

| Template | Shows | Needs | May have |
|---|---|---|---|
| `title` | A short rule, a large title, and a line under it | `title` | `subtitle` |
| `section` | A small line in the accent colour, and a section's name | `title` | `subtitle` |
| `bullets` | A title, and a list of points | `title`, `items` | |
| `quote` | A quotation beside a rule, and whose it is | `quote` | `by` |
| `end` | A title and a line under it, centred | `title` | `subtitle` |

```toml
[scenes.open]
kind = "card"
template = "title"
title = "The LLM drafts, the engine decides"
subtitle = "a five-minute tour"
```

Long text wraps, and the whole card sits in the middle of the frame, or of the part of it that the cut's apps leave free ([What the apps cover](#what-the-apps-cover)). A line on a card scene takes no `do`, `highlight` or `camera`. A card's words are part of what its cut shows, so changing them after the cut is ratified makes it `changed`.

### Images

An image scene is a picture under the virtual camera: a PNG, used as it is, or an SVG, which vidni draws itself. A part of the picture is a target with a rectangle, in the picture's own pixels (an SVG's own units), and cameras frame those parts as they do on a web scene.

```toml
[scenes.diagram]
kind = "image"
url = "diagram.svg"
scale = 2                  # an SVG drawn at twice its size stays sharp when the camera moves in

[scenes.diagram.targets]
draft = { rect = [490, 270, 300, 180] }   # x, y, width, height

[scenes.diagram.cameras]
draft = { union = ["draft"] }
```

- **A line on an image scene** takes `camera` and `camera_vertical`. It takes no `do` and no `highlight`.
- **An SVG's text** is set in the fonts vidni pins ([Fonts](#fonts)), by the family the SVG names, and in no font of the system's. So the picture is the same on every machine.
- **The picture is a capture** like any other: one for all the lines that show it, taken by `vidni draft` or `vidni capture` with no browser, frozen when its cut is ratified, and taken again when its file changes ([When a capture is out of date](#when-a-capture-is-out-of-date)).

[`examples/images`](../examples/images) has an SVG diagram and a PNG chart.

### Targets

A target names the visible elements of a web page that match it. Highlights, cameras, actions and expectations refer to targets by name.

| Key | Type | Meaning |
|---|---|---|
| `css` | string | A CSS selector. A target needs `css` or `any`. |
| `text` | string or rule | Keep elements whose text matches. A string matches exactly, after whitespace is collapsed. A rule is one of `{ starts_with = "…" }`, `{ contains = "…" }` or `{ matches = "regex" }`. The regular expression is JavaScript's, in Unicode mode, because it runs in the page; a leading `(?i)`, `(?m)` or `(?s)` sets that flag. |
| `text_of` | string | Match `text` and `exclude` against this child (a CSS selector) instead of the whole element |
| `exclude` | string or rule | Drop elements whose text matches |
| `attr` | `{ name, matches }` | Keep elements whose attribute `name` matches the regular expression (JavaScript's, as for `text`) |
| `pick` | `"all"`, `"first"` or `"last"` | Which matches to keep. Default `all`. |
| `merge` | boolean | Draw one box around all the matches. `pick` must be `all`. |
| `line` | integer | One line of the element's text, counted from 0 |
| `grow` | `{ within, max_height, margin }` | Grow each match to its largest enclosing block inside `within` that is at most `max_height` CSS pixels tall, at least `margin` pixels narrower than `within`, and whose text still ends with the match's text |
| `unique` | boolean | When reading texts, keep each text once |
| `any` | array of targets | The union of other matchers, instead of `css`. Put the matching keys inside each matcher; `pick`, `merge` and `unique` apply to the union. |

Boxes are clipped to the scroll containers that hold them.

### Cameras

A camera is a framing of a web scene: the union of some targets, or the whole viewport. Two cameras are always defined: `viewport` (the whole page) and `highlights` (the union of the line's highlight boxes).

| Key | Type | Meaning |
|---|---|---|
| `union` | array of strings | Frame the union of these targets |
| `viewport` | boolean | Frame the whole viewport. Set this or `union`. |
| `vertical` | string | The camera that 9:16 cuts use instead, when this one is too wide |

A 9:16 cut chooses a line's camera in this order:
1. The line's `camera_vertical`.
2. `highlights`, when the line has highlights no wider than the scene's `vertical_follows_highlights`.
3. The `vertical` camera of the line's `camera`.
4. The line's `camera`.

### Actions

An action is a table with exactly one verb. Each verb names a target, and some take extra keys.

| Verb | Extra keys | What it does |
|---|---|---|
| `click` | | Click the target |
| `select` | `value` (required) | Choose the option `value` in the target, a `<select>` |
| `expand` | `control` | Open the target if its `aria-expanded` is false, by clicking `control` (a CSS selector inside it) or else the target itself |
| `collapse` | `control` | Close the target if it is open, in the same way |
| `scroll` | `top` (required) | Scroll the target, a scroll container, to `top` CSS pixels |
| `scroll_to` | `within` (required), `offset` | Scroll the target `within` (a scroll container) until this target is at its top, less `offset` CSS pixels |
| `wait` | | Wait until the target is visible |

How actions run:
- **Waiting:** every action first waits, up to 30 seconds, for its target to show.
- **Clicks:** a click moves the mouse to the middle of the target and clicks there, as a person would, so hover styles show as they would for one. A target that isn't wholly in view is first scrolled into view through its scroll containers. A click never lands on something covering its target: it waits, up to 30 seconds, for the cover to go.
- **Scrolls** happen at once, whatever the page's `scroll-behavior` says.
- **Settling:** after every action, capture waits for the page to settle. Its running animations are finished, and endless ones stopped; one the page holds still (paused) is left as it is. Its fonts are loaded. Then two screenshots in a row, a quarter second apart, must be identical, with the network quiet between them. A page that keeps its network busy has settled once five screenshots in a row are identical.
- **Following:** when an action takes the page to another document, capture waits for it to load and carries on there. A page that asks before it is left (`beforeunload`) is told to go ahead.

A change that arrives late, with nothing moving meanwhile, can't be seen coming: a result that shows a second after a click, say. Name what the page must show then in an [expectation](#expectations), or `wait` for its target; capture waits for both.

## Capturing: `vidni capture`

`vidni capture` takes the captures the cuts need that `build/captures/` doesn't hold yet, or holds [out of date](#when-a-capture-is-out-of-date).
- **Each web scene** is loaded in a fresh Chromium, and its `setup` run. Then its lines run in script order. For each line:
  1. Its actions run, and the focus is taken off whatever has it.
  2. Its expectations are waited for, up to 30 seconds each, and read.
  3. When a still is needed, the line's highlights are drawn, the page settles, and the still is taken.
- **Beside each still**, `<key>.json` holds its frames, in CSS pixels: the boxes of every target of the scene, as far as they show; each highlight's boxes; the viewport and scale; and the browser that took it. Cameras aren't in it. A cut frames each camera from its targets' boxes when it is built, so a camera can be changed or added without capturing again.
- **Each page scene** is rendered at each size a cut needs, in its own Chromium, with a transparent background when it says so.
- **Beside each capture of a page in the project folder**, `<key>.files.json` records the project's files that the page had loaded by then, each with its SHA-256: the page itself, and its stylesheets, scripts, pictures and whatever else it asked for. A file it asked for that wasn't there is recorded as missing.
- **A scene's stills are kept only when it passes.** They wait aside until the scene's last line is done. The readings go to `build/onscreen.json`. No still of the scene is kept, and the problem is reported with its scene and line, when:
  - an expectation doesn't hold;
  - a script on the page throws an error;
  - the page opens a dialog, which is dismissed;
  - the page's renderer crashes;
  - a page that is a file of the project's asks the network for a part that doesn't load. Such a page should come out whole every time. A page on the web may lose a load, as pages do, unless the capture runs with [`--offline`](#without-the-network---offline);
  - a page that is a file of the project's loads a file from outside the project folder. Nothing could tell when that file changes, and a copy of the project wouldn't have it;
  - a file the page loaded changes while the page is being captured.
- **A frozen still is never taken again for want of its frames.** When a ratified cut froze a still's picture and only its frames are missing, `vidni capture` and `vidni draft` refuse, and say to restore the frames or to unratify the cut with `--refresh`. A frozen still of a page in the project folder is taken again once it is out of date: the files that changed are yours, to change back.
- **Interrupting** a capture (Ctrl-C) closes Chromium, removes its temporary profile and keeps no still of the unfinished scene.

A web scene with a `scale` above 1 is loaded twice, and the second load is the one captured. Chromium's first load at an emulated scale came out in one of two renderings, launch by launch (DECISIONS.md, T6). The first load is inert: none of the page's scripts run, and the cookies it sets are cleared, so the second load is still a first visit. The server does see two requests.

How Chromium runs:
- It is the `chromium` on the path, or `$VIDNI_CHROMIUM`. vidni's flake pins it, and each still's frames record its version.
- It runs headless, on a fresh profile, with the switches Playwright 1.63 uses for headless Chrome, plus `--font-render-hinting=none` and `--force-color-profile=srgb`.
- It rasters each layer as one tile. Left to itself it placed its tiles differently from one launch to the next, and curves came out a few levels different.
- Its sandbox is on. `VIDNI_NO_SANDBOX=1` turns it off, for hosts whose kernel doesn't let Chromium build one (some CI runners).
- vidni talks to it over pipes, so nothing listens on a network port, and it exits when vidni does.
- vidni's own script runs in an isolated world, where the page's scripts can't see or change it. Its style and highlight layer are in each document from the start.
- Pages are told to reduce motion, and a web scene gets the colour scheme it asks for.
- Fonts are the ones fontconfig offers. vidni's flake pins them (IBM Plex, Space Grotesk, DejaVu, Liberation, Noto and Noto Color Emoji), so a page's fallback fonts don't depend on the host there. Chinese, Japanese and Korean aren't covered yet.

Highlight boxes are drawn with an ember outline, in every theme: they are part of the still.

## `[[cuts]]`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `id` | string | required | The cut's ID |
| `aspect` | `"16:9"` or `"9:16"` | required | The frame's shape |
| `lines` | `"all"` or array of line IDs | `"all"` | The lines the cut uses, in script order |
| `captions` | `"burned"`, `"file"` or `"both"` | `both` for 16:9, `burned` for 9:16 | `burned` draws captions into the picture. `file` keeps the picture clean and writes SRT and VTT files. `both` makes a clean master with caption files, plus a burned-in variant (DECISIONS.md, D7). |
| `platforms` | array of strings | the profiles on by default for the cut's aspect: `["youtube"]` for 16:9; `["youtube-shorts", "instagram-reels", "whatsapp-status"]` for 9:16 | The platform profiles the cut is made for: `youtube`, `youtube-shorts`, `instagram-reels`, `tiktok`, `whatsapp-status`, `linkedin` or `x`. `vidni platforms` lists each profile's limits, with the date they were checked and their sources. |
| `size` | `[width, height]` | `[1920, 1080]` or `[1080, 1920]` | The frame size: even numbers in exactly the cut's aspect. For now, vidni draws only the two default sizes, and a build refuses another. |

### Several cuts

A project can make any number of videos from one script: a long cut, and shorts that tell a part of it.
- **A cut picks its lines** with `lines`, in script order. Without it, a cut uses every line.
- **A line can be only for some cuts.** A short's hook or sign-off sets `only = ["short"]`. A cut that uses every line leaves it out, and a cut it isn't for can't list it.
- **A cut that lists its lines** must list its own lines too. A line that is only for a cut that leaves it out is in no video, so validation reports it.

```toml
[[cuts]]
id = "short"
aspect = "9:16"
lines = ["hook", "syllogism-1", "syllogism-2"]

[[lines]]
id = "hook"
section = "intro"
show = "Here's a proof you can check, in under a minute."
only = ["short"]
```

A line sits at its place in the script whichever cuts use it, so a short's lines go where they are said: the hook before the lines it leads into.
- **What a web line shows** doesn't depend on the cut. Its still is the page after every action of the script up to it, including the actions of lines the cut leaves out. So a cut never needs captures of its own.
- **Takes are shared.** A line has one take, and every cut that uses the line plays it.
- **Each cut is ratified, built and approved by itself.** Adding a short, or a line that is only for it, leaves the other cuts ratified.

### What the apps cover

A phone app draws its own things over a video: buttons down the right, the account and the description along the bottom, tabs across the top. Each platform profile says how much of the frame its app covers at each edge, as its safe zone (`vidni platforms`). A cut is laid out clear of what all its platforms cover together: the most that any of them covers at each edge.

With the default platforms of a 9:16 cut, that is 288 px at the top, 672 at the bottom, 65 at the left and 193 at the right. It leaves 822 by 960 px of the 1080 by 1920 frame free.

**A 9:16 cut whose platforms cover something** keeps everything inside the free part:
- **The title band,** 176 px high, is under the top edge the apps cover: the kicker, and the section's label in up to two lines.
- **The caption band,** 190 px high, is above the bottom edge they cover. A cue of more lines than it holds grows upwards, over the view.
- **The view** is between the two bands, as wide as the free part. A camera frames its targets in it, as in any cut.
- **A card** drawn from a template is laid out in the free part, above the caption band, with smaller type.
- **The rest of the frame** is the theme's frame colour.

**A 9:16 cut whose platforms cover nothing,** or with `platforms = []`, keeps the full-height layout: a 330 px title band, the view, and the captions below.

**A page scene** is your own layout, and vidni can't move what it draws. Keep what such a page draws inside the free part, on a plain background; or use a card template.

**QA checks it.** Each line's card, section label, overlay and captions must sit inside the free part. Ink is every pixel that differs from the card's own background, or from the frame colour for the other three. A line that draws ink under an app is a QA problem, named by its line, the thing and the edge:

```text
line short-hook: its card reaches 288 px into the top edge that the cut's platforms cover
```

A draft prints it as a warning, and a final build that has one isn't delivered.

**Review frames show it.** Every build writes one frame for each section, halfway through its middle line, as `build/draft/review/<cut>/<platform>/<nn>-<section>.png` (or `build/final/…`). Each platform has its own set, with what its app covers shaded red and the pixels named. A cut made for no platform has one set, in `review/<cut>/`.

**Measuring an app.** `vidni platforms --grid DIR` writes `grid-1080x1920.png` and a 15-second clip of it, `grid-1080x1920.mp4`. The picture is in squares of 120 px, each with the place of its top left corner written in it. Upload the clip privately, take a screenshot while it plays, and the squares the app's buttons and labels hide say what it covers.

## `[music]`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `track` | string | `"none"` | Must be `none` in schema 1. Licensed tracks arrive with the music library (P6-VIDNI-06). |

## `[[expectations]]`

An expectation is text a web scene must show. A capture stops when the page shows anything else, so a video never narrates a page that changed underneath it.

| Key | Type | Meaning |
|---|---|---|
| `scene` | string | A web scene |
| `at` | string | `setup`, or the ID of a line on that scene; checked after that line's actions |
| `target` | string | The target whose text is read |
| `text` | string | The text of the target's single match |
| `texts` | array of strings | The texts of all its matches, in page order |

Set `text` or `texts`, not both.

A capture waits at that point, up to 30 seconds, for the target to show the expected texts. Then it reads them and writes them to `build/onscreen.json`. Texts are compared with their runs of whitespace collapsed. Each one that differs is reported with the point in the script, the target, and what was shown in place of what was expected, for example: `at line "false-1", target "result" shows "UNKNOWN", not "FALSE"`.

## Ratification

Ratifying a cut freezes what it shows and says. `vidni ratify [--cut ID]` records the following in `ratified.lock`, which is committed with the project:
- **One hash of the cut's content:** its line order, its lines (every key, with the default scene filled in), their sections, the scenes they use, the keys of its captures and, for a 9:16 cut, the subtitle that its title band shows. All of it is taken from `build/resolved.json`, the project with every default filled in.
- **A hash for each line and each scene,** so that a changed cut can say what changed.
- **The SHA-256 of each capture file,** and of each still's frames file, which holds the boxes the still is framed by.

A cut's status is never stored; `vidni status` works it out from those hashes each time:
- **draft:** not ratified.
- **ratified:** every hash still holds.
- **changed:** something the cut froze has changed since: a line, a scene, its line list or order, a capture file on disk, or a file in the project folder that one of its captures was made from. The status names what changed.

Gaps (`[timing]`), caption style, platforms and the theme aren't part of a cut's content; only the release hashes them. So changing them leaves a cut ratified, and editing a line regresses exactly the cuts that use it.

**Captures** live in `build/captures/`, out of git, each named by a key:
- **A web line's capture** is keyed by four things:
  - its scene, all of it except the cameras, which only frame the still;
  - every action that has run on the page by then: the setup, then each earlier line's `do` in script order, then its own;
  - every expectation capture has waited for by then, on that scene;
  - its highlights.

  Lines that show the same page with the same highlights share one capture.
- **A page scene's render** is keyed by the scene, the size it is rendered at, and the cut's aspect, which `{aspect}` in its address becomes.

A draft takes only the captures that the folder doesn't hold, or holds out of date, so a redraft with nothing changed takes none. `vidni unratify` keeps a cut's captures. `--refresh` deletes the ones no other ratified cut uses, so that they are taken afresh.

### When a capture is out of date

A key covers what `vidni.toml` says about a scene. It can't cover what the page holds.
- **A page in the project folder:** its capture is out of date once a file in its record (`<key>.files.json`) isn't what it was: the file changed, or it is gone, or it is there now and wasn't. The next `vidni draft` or `vidni capture` takes that capture again and says which files changed. A capture with no record counts as not taken.
- **A page on the web:** nothing can tell whether it has changed without loading it. Its capture is kept until `vidni unratify --refresh` deletes it, or you do.

A ratified cut with a capture that is out of date is `changed`, and names the file. It can't have a final build until it is drafted and ratified again. With the file put back as it was, the cut is the ratified one again: at once when no draft took the capture again meanwhile, and otherwise after the next draft, since one machine with one Chromium gives the same capture.

## From draft to release

A cut is drafted, ratified, built from your takes, and approved. Each step is a command, and each refuses to go ahead when an earlier one isn't in place.

| Step | Command | What it makes |
|---|---|---|
| Draft | `vidni draft` | Each cut in the draft voice, in `build/draft/`, to watch |
| Ratify | `vidni ratify` | `ratified.lock`: what each cut shows and says, frozen ([Ratification](#ratification)) |
| Final build | `vidni final` | Each cut from your takes, in `out/` |
| Approve | `vidni approve` | `release.json`: each final cut, as a release |

Each of these takes `--cut ID`, which can be repeated, and works on every cut without it.

**The project folder** holds four kinds of file:

```text
vidni.toml      the project                             committed
voice.lock      the draft voice's kept seeds            committed
ratified.lock   the ratified cuts                       committed
release.json    the approved releases                   committed
assets.lock     the takes and frozen captures, by hash  committed
takes/          your takes                              out of git, backed up
out/            the final cuts, as delivered            out of git, archived with a backup
build/          what vidni makes                        out of git
  resolved.json   the project with every default filled in
  voice/          the draft voice's clips
  captures/       stills and page renders, with each still's frames and each
                  project page's record of the files it loaded
  draft/          each cut's draft
  final/          each cut's final build, before it is delivered
```

Everything in `build/` can be made again, except the captures of a ratified cut. A page on the web may have changed since, so those are frozen: they are assets, as your takes are ([Assets and backups](#assets-and-backups)).

### Drafting: `vidni draft`

`vidni draft` makes each cut watchable before anything is recorded. It does three things, and each does only what is missing:
1. It speaks every line in the draft voice (`vidni speak` does this step alone; see [The draft voice](#the-draft-voice)).
2. It takes the captures the cuts need (`vidni capture` does this step alone).
3. It builds each cut into `build/draft/`.

A draft is one picture for each cut, `build/draft/<cut>.mp4`, with its captions drawn in unless the cut's `captions` is `file`. What QA finds in a draft is printed as a warning. Beside the picture are the cut's review frames, in `build/draft/review/<cut>/` ([What the apps cover](#what-the-apps-cover)).

**What a draft keeps.** Each step keeps what it made, and makes it again only when what it was made from changes. So a second draft with nothing changed speaks, captures and renders nothing: on the nibli example, two cuts of five minutes each, it takes under a second.

| What | Kept as | Made again when |
|---|---|---|
| A line's draft clip | `build/voice/<line-id>/<key>-<seed>.wav` | What the draft voice is given changes: the line's `say` or `show`, `voice.respell`, the voice pack or `voice.length_scale`; or its seed in `voice.lock` does |
| A capture | `build/captures/<key>.png` | Its key changes: the scene, the actions and expectations before it, or its highlights. Or it is [out of date](#when-a-capture-is-out-of-date) |
| A cut's build | `build/draft/` or `build/final/`, with its record | Its `inputs` change, or a file the build made isn't as the build made it ([the record](#what-a-build-is-made-from-cutbuildjson)) |

`vidni draft --again` and `vidni final --again` render each cut all the same.

### The final build: `vidni final`

`vidni final` builds each cut from your takes. It refuses a cut, before rendering anything, unless:
- the cut is ratified, and nothing it froze has changed since;
- every line of the cut has a current take.

Then it builds the cut into `build/final/` and runs QA on each file. A build that is there already and is still the cut's is kept, with what QA found in it ([what a draft keeps](#drafting-vidni-draft)). A cut that fails QA stays there, with its problems listed, and is not delivered: what `out/` held of it before stays as it was. A cut that passes is copied to `out/`:

| File | What it is |
|---|---|
| `<cut>.mp4` | The cut. Its captions are drawn in when `captions` is `burned`. |
| `<cut>.captioned.mp4` | The variant with captions drawn in, when `captions` is `both` |
| `<cut>.<language>.srt`, `.vtt` | The captions as files, to upload |

### What a build is made from: `<cut>.build.json`

Every build writes a record beside its files, `build/draft/<cut>.build.json` or `build/final/<cut>.build.json`. It names what the build was made from and what it made, each by its SHA-256.

| Key | What it holds |
|---|---|
| `sources.content` | What the cut shows and says: the hash that ratifying freezes |
| `sources.audio` | Each line's clip |
| `sources.captures` | Each capture's picture and frames |
| `sources.look` | The gaps, the look, the title and subtitle, and the cut's own settings |
| `tools` | vidni's and ffmpeg's versions, each font file, and the encoder's settings |
| `inputs` | All of the above and the cut's timeline, as one hash |
| `t1` | The timeline, the captions, and the narration before and after loudness normalization |
| `outputs.<file>` | For each encoded file: its frames before encoding (T1), its packets and its bytes (T2), and what QA measured |

Two builds with the same `inputs` must have the same `t1` and `outputs`: that is what [`vidni repro`](#checking-that-a-build-repeats-vidni-repro) checks. The tiers are described in [`determinism.md`](determinism.md).

That is also why a build can be kept. `vidni draft` and `vidni final` work out a cut's `inputs` first. When the record on disk has the same `inputs`, and each file it names still has the SHA-256 it records, a build would only make the same files again, so nothing is rendered. `--again` renders anyway.

What isn't in a cut's record:
- **The resolved project,** `build/resolved.json`. It is T1 as well, but it is the whole project's, so an edit to another cut would change it. `vidni repro` compares it once, for the project.
- **`voice.heard_as`,** which only the listening check reads.
- **The draft voice's settings,** in a final build. `voice.respell`, `voice.seed` and the rest change the draft voice's clips, and no take.

### Approving: `vidni approve`

`vidni approve` writes a cut's final build into `release.json` as a release. It refuses unless:
- a final build would be allowed now (the cut is ratified and unchanged, and every line has a current take);
- the build in `build/final/` is of the cut as it is now, with the tools and fonts in use now, and passed QA;
- the files in `out/` are the ones that build delivered.

A release has a tag, the cut's ID and a number, such as `long-r1`; the day it was approved; and the build's record. `release.json` names its own layout as `"schema": 1`. Releases are never changed. Approving a cut again after a change adds the next one, `long-r2`, and approving the same build twice changes nothing. vidni makes no git tag: commit `release.json`, and tag the commit if you want one.

`vidni status` shows each cut's state, and how it stands against its newest release:

```text
long      ratified; approved as long-r1
vertical  changed: line intro-2 changed; approved as vertical-r1, and changed since in what it shows and says
```

### Checking that a build repeats: `vidni repro`

`vidni repro` builds the final cuts twice and compares every hash of the two records. Each build runs:
- **from its own copy of the project,** at a path of another depth and length. The copy holds what a build reads and nothing else: the project file it was given, the locks, the files its scenes name, each line's current take, and the cuts' captures. Captures are never taken again here: they are frozen inputs.
- **in a clean environment:** only `PATH`, a fixed time zone (`TZ=UTC`), a fixed locale (`LC_ALL` and `LANG` set to `C.UTF-8`), `SOURCE_DATE_EPOCH`, an empty `HOME` and `TMPDIR` of its own, and the font folders. Nothing else of yours is passed on.
- **on its own set of cores:** two cores for one build and three others for the other, where there are at least five; one against the rest, where there are fewer.

It prints each hash the builds share, the resolved project's first, and exits 0 when all are equal. When the project folder has a build of the same inputs, made in your own environment, that build must equal them too. A difference is named by its tier and file, such as `t2 long.mp4 packets`, and the exit code is 1.

| Option | What it does |
|---|---|
| `--draft` | Builds the drafts instead of the final cuts. Each build speaks its lines again, from `voice.lock`, so the voice store is passed on too. |
| `--offline` | Runs both builds with no network |
| `--keep DIR` | Keeps both copies, with their builds and logs, in `DIR`, an empty or new folder outside the project |

Interrupted, it ends both builds and removes its copies. It needs `taskset`, from util-linux, and runs on Linux only.

### Without the network: `--offline`

`vidni draft --offline` and `vidni final --offline` run in a namespace that has no network, so a build that reaches for the network fails instead of passing by luck:
- A page in the project folder is still captured, whether it is a page scene or a web scene's `url`. One that asks the network for a part fails, and no capture of it is kept.
- A web scene on the web whose capture isn't taken yet is refused, by name.
- Nothing can be connected to. A name can still be looked up where the host's resolver answers through a socket in the file system.

The command checks that it has no route to the network before it does anything else. It needs `unshare`, from util-linux, and a kernel that lets a user make namespaces. It runs on Linux only.

### Assets and backups

The assets are what nothing can make again: every take under `takes/` (kept takes, raw takes and room tone), and every capture file a ratified cut froze. They are large and stay out of git. `assets.lock`, which is committed, lists each with its SHA-256, one to a line as `sha256sum` writes them, so `sha256sum -c assets.lock` checks them too.

| Command | What it does |
|---|---|
| `vidni backup --to DIR --to DIR` | Lists the assets in `assets.lock`, and copies them to each place |
| `vidni verify [--to DIR]…` | Checks the assets in the project folder against `assets.lock`, and each backup named |
| `vidni restore --from DIR` | Copies back the assets that are missing |
| `vidni gc [--dry-run]` | Deletes what can be made again |

- **A backup goes to two places,** one of them off the machine: another disk, or a folder mounted from another machine. `vidni backup` refuses fewer, and warns when every place is on the disk the project is on. A place must be there already: a folder that isn't is a disk that isn't plugged in, and vidni doesn't make it. `$VIDNI_BACKUP` names the places, separated by `:`, when no `--to` or `--from` does.
- **In each place** the backup is a folder named as the project's folder is. It holds the assets at their paths, a copy of `assets.lock`, and under `releases/<tag>/` the files each cut's newest release delivered, when `out/` still holds them. Those are archived because a cut's file isn't promised to come out the same on another machine ([`determinism.md`](determinism.md), T3).
- **A backup only gains files.** Each copy is checked against its hash before it stands for the file. Nothing in a backup is deleted, so a take that was replaced is still there.
- **`vidni restore`** copies back each listed file that is missing, from the first place that holds it as listed, and the newest releases' delivered files into `out/`. A file that is there and holds something else is left as it is, and reported: it may be a take recorded since the backup. A project that has lost `assets.lock` takes a backup's.
- **`vidni verify`** exits with 1 when a listed file is missing or changed, here or in a backup. Assets that aren't listed yet are named, and aren't a fault.
- **`vidni gc`** deletes the draft voice's clips, the drafts, the final builds in `build/final/`, captures that no cut shows and no ratified cut froze, and files a session left behind. It never touches a take, a frozen capture, a capture a cut shows now, or `out/`.

After the assets are wiped, `vidni restore` and `vidni final` give the cuts that were approved, with the hashes `release.json` holds.

### Exit codes

For `draft`, `final`, `approve` and `repro`:
- 0 when everything asked for was done
- 1 when a gate refused, a build failed, or two builds differ
- 2 when the project, a lock or `release.json` can't be read, or something a build needs is missing: a font, ffmpeg, the draft voice's files

### Fonts

vidni draws with font files it pins by SHA-256: IBM Plex Sans Bold, IBM Plex Mono Bold and Space Grotesk Bold. A theme names the three it uses ([Themes](#themes)). It looks for them in the folders that `VIDNI_FONTS` names (separated by `:`), else in `$XDG_DATA_HOME/vidni/fonts`, else in `~/.local/share/vidni/fonts`, and never among the system's fonts. A file with the right name and other bytes is refused. vidni's Nix shell sets `VIDNI_FONTS`.

## Layout: `vidni fmt`

`vidni fmt` rewrites a valid file in one canonical layout, and `vidni fmt --check` reports a file that isn't in it. Every comment is kept, and every value is kept as written: strings keep their quotes and escapes, and numbers keep their spelling. The rest is fixed:
- **Tables** come in the order of this page: `meta`, `voice`, `timing`, `lint`, `style`, `cuts`, `music`, the sections in script order, `lines`, `scenes`, `expectations`.
- **Keys** come in the order of this page within each table.
- **Spacing:** `key = value` has single spaces, and inline tables read `{ a = 1, b = 2 }`.
- **Blank lines:** each table header has one blank line before it. Keys have none, except before a comment that starts a group.
- **Arrays** stay on one line when the line fits in 100 columns and holds no comments. Otherwise each element gets its own line, indented two spaces, with a trailing comma.
- **Trailing comments** sit one space after their value.
- **Dotted tables** become headed tables, and the file ends with one newline.

The desktop app edits a file through the same core, and an edit changes only what it names: a changed value keeps its comment, a new key goes where `vidni fmt` would put it, and a moved line takes its comments along.

## What `vidni validate` checks

```sh
vidni validate                     # the vidni.toml in this folder
vidni validate examples/nibli-syllogism
```

It reports every problem as `<file>:<line>:<column>: <message>`, and exits:
- 0 when the file is valid
- 1 when it has problems
- 2 when it can't be read

It checks, in order:
1. The TOML syntax.
2. The schema version.
3. Types, required keys and unknown keys. The parser stops at the first problem of this kind.
4. Every rule on this page that reaches across tables:
   - IDs are well formed and unique.
   - Lines name known sections and scenes, and keep sections together.
   - Highlights, cameras and actions name known targets and cameras, and actions have exactly one verb.
   - Keys match their scene's kind.
   - Overlays are transparent page scenes.
   - Cuts name known lines in script order, and known platforms.
   - Sizes are even and in aspect.
   - Regular expressions compile: a target's as JavaScript, `lint.banned` as Rust's `regex` crate.
   - `scroll`'s `top` and `scroll_to`'s `offset` are finite numbers.
   - Each `voice.heard_as` entry's `heard` is one word, listed once, and its `means` has letters.
   - `meta.language` is a tag of letters, digits and hyphens.
   - No line shows or says wording that `lint.banned` forbids.
   - In a strict project, every line that states a fact names its `source`.
   - Every file a scene names exists.
   - Expectations name a web scene, a target, and a line on that scene.

The broken fixtures in `crates/vidni-core/tests/fixtures/invalid` show one problem of each kind, with the message it produces.

It also warns, without failing, when the draft voice's licence doesn't allow commercial use or isn't known ([The draft voice](#the-draft-voice)). A warning reads `<file>:<line>:<column>: warning: <message>`.
