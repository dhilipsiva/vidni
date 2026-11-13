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
| `language` | string | `"en"` | The script's language, as a BCP 47 tag |
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
| `theme` | string | `"default"` | `default`, or the path of a theme folder (P5-VIDNI-01) |

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

`vidni takes --link DIR` lays the current takes out as `000.wav`, `001.wav` … in script order, as symbolic links, for tools that read numbered files.

## The draft voice

`vidni draft` speaks every line in the draft voice, `voice.draft`, so that a cut can be watched before any take is recorded.

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

**Licences.** A voice whose licence doesn't allow commercial use may still be drafted with, but its audio can't go into a cut that is used commercially. `vidni validate` and `vidni draft` warn about such a voice, and about one whose licence vidni can't find.

## `[scenes.<id>]`

| Key | Type | Kinds | Meaning |
|---|---|---|---|
| `kind` | `"web"` or `"page"` | both | `web`: a page captured at a fixed viewport and framed by the virtual camera. `page`: an HTML file rendered at each cut's own size, for cards and overlays. |
| `url` | string | both | `web`: the page's address, or a file in the project folder. `page`: a file in the project folder, where `{aspect}` becomes `16x9` or `9x16` |
| `viewport` | `[width, height]` | web | CSS pixels. Default `[1280, 720]`. |
| `scale` | integer | web | Device pixels per CSS pixel. Default 1. |
| `color_scheme` | `"light"` or `"dark"` | web | The colour scheme the page is asked for |
| `setup` | array of actions | web | Run once, after the page loads |
| `targets` | table of targets | web | Named parts of the page; see [Targets](#targets) |
| `cameras` | table of cameras | web | Named framings; see [Cameras](#cameras) |
| `default_camera` | string | web | The camera of a line that names none. Default `viewport`. |
| `vertical_follows_highlights` | number | web | In 9:16 cuts, a line whose highlights are at most this many CSS pixels wide is framed on them |
| `transparent` | boolean | page | Rendered on a transparent background, as an overlay |
| `size` | `{ landscape = [w, h], portrait = [w, h] }` | page | The rendered size for each orientation. Default: the cut's size. |

A key that belongs to the other kind is an error. A page scene's file must exist in the project folder, and so must a web scene's file when its `url` isn't an `http` or `https` address. The check runs for both orientations when the name holds `{aspect}`.

### Targets

A target names the visible elements of a web page that match it. Highlights, cameras, actions and expectations refer to targets by name.

| Key | Type | Meaning |
|---|---|---|
| `css` | string | A CSS selector. A target needs `css` or `any`. |
| `text` | string or rule | Keep elements whose text matches. A string matches exactly, after whitespace is collapsed. A rule is one of `{ starts_with = "…" }`, `{ contains = "…" }` or `{ matches = "regex" }`. |
| `text_of` | string | Match `text` and `exclude` against this child (a CSS selector) instead of the whole element |
| `exclude` | string or rule | Drop elements whose text matches |
| `attr` | `{ name, matches }` | Keep elements whose attribute `name` matches the regular expression |
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

Capture waits for the page to settle after every action: it takes screenshots until two in a row are identical (P3-VIDNI-04).

## `[[cuts]]`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `id` | string | required | The cut's ID |
| `aspect` | `"16:9"` or `"9:16"` | required | The frame's shape |
| `lines` | `"all"` or array of line IDs | `"all"` | The lines the cut uses, in script order |
| `captions` | `"burned"`, `"file"` or `"both"` | `both` for 16:9, `burned` for 9:16 | `burned` draws captions into the picture. `file` keeps the picture clean and writes SRT and VTT files. `both` makes a clean master with caption files, plus a burned-in variant (DECISIONS.md, D7). |
| `platforms` | array of strings | the profiles on by default for the cut's aspect: `["youtube"]` for 16:9; `["youtube-shorts", "instagram-reels", "whatsapp-status"]` for 9:16 | The platform profiles the cut is made for: `youtube`, `youtube-shorts`, `instagram-reels`, `tiktok`, `whatsapp-status`, `linkedin` or `x`. `vidni platforms` lists each profile's limits, with the date they were checked and their sources. |
| `size` | `[width, height]` | `[1920, 1080]` or `[1080, 1920]` | The frame size: even numbers in exactly the cut's aspect |

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

A capture reads the target's texts at that point and writes them to `build/onscreen.json`. Texts are compared with their runs of whitespace collapsed. Each one that differs is reported with the point in the script, the target, and what was shown in place of what was expected, for example: `at line "false-1", target "result" shows "UNKNOWN", not "FALSE"`.

## Ratification

Ratifying a cut freezes what it shows and says. `vidni ratify [--cut ID]` records the following in `ratified.lock`, which is committed with the project:
- **One hash of the cut's content:** its line order, its lines (every key, with the default scene filled in), their sections, the scenes they use, and the keys of its captures. All of it is taken from `build/resolved.json`, the project with every default filled in.
- **A hash for each line and each scene,** so that a changed cut can say what changed.
- **The SHA-256 of each capture file.**

A cut's status is never stored; `vidni status` works it out from those hashes each time:
- **draft:** not ratified.
- **ratified:** every hash still holds.
- **changed:** something the cut froze has changed since: a line, a scene, its line list or order, or a capture file on disk. The status names what changed.

Gaps (`[timing]`), caption style, platforms and the theme aren't part of a cut's content; only the release hashes them. So changing them leaves a cut ratified, and editing a line regresses exactly the cuts that use it.

**Captures** live in `build/captures/`, out of git, each named by a key:
- **A web line's capture** is keyed by three things:
  - its scene, all of it except the cameras, which only frame the still;
  - every action that has run on the page by then: the setup, then each earlier line's `do` in script order, then its own;
  - its highlights.

  Lines that show the same page with the same highlights share one capture.
- **A page scene's render** is keyed by the scene and the size it is rendered at.

A draft takes only the captures no file holds yet, so a redraft with no scene changes takes none. `vidni unratify` keeps a cut's captures. `--refresh` deletes the ones no other ratified cut uses, so that they are taken afresh.

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
   - Regular expressions compile.
   - No line shows or says wording that `lint.banned` forbids.
   - In a strict project, every line that states a fact names its `source`.
   - Every file a scene names exists.
   - Expectations name a web scene, a target, and a line on that scene.

The broken fixtures in `crates/vidni-core/tests/fixtures/invalid` show one problem of each kind, with the message it produces.

It also warns, without failing, when the draft voice's licence doesn't allow commercial use or isn't known ([The draft voice](#the-draft-voice)). A warning reads `<file>:<line>:<column>: warning: <message>`.
