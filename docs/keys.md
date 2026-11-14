# Every key of `vidni.toml`

This page lists each key vidni reads from a project file, with its type and its meaning. It is generated from the project file's JSON Schema ([vidni.schema.json](vidni.schema.json)), which vidni writes from its own model of a project: `just keys` writes this page again, and `just keys check` fails when it is out of date. Don't edit it by hand.

[The spec](spec.md) explains how the keys work together, with examples. A key that isn't on this page is an error: `vidni validate` names it, with its line and column.

A project file starts with `schema = 1`, the version of this layout.

## `[meta]`

What the video is.

| Key | Value | Required | Meaning |
|---|---|---|---|
| `title` | string | yes | The video's title. |
| `default_scene` | string |  | The scene a line shows when it names none. |
| `language` | string |  | The script's language, as a BCP 47 tag. |
| `strict` | boolean |  | Whether every factual line must name its source (P2-VIDNI-06). |
| `subtitle` | string |  | A short line shown with the title, such as a series name. |

## `[voice]`

The voices that speak the lines.

| Key | Value | Required | Meaning |
|---|---|---|---|
| `draft` | string |  | The synthetic draft voice: a voice pack, as `<engine>:<model>`. |
| `heard_as` | array of [HeardAs](#heardas) |  | Words the listening check hears spelled differently from the script for the same sound. They make its scores fairer and change nothing else: no output depends on what the check hears. |
| `length_scale` | number |  | The draft voice's pace: above 1 is slower. |
| `read_as` | array of [Respelling](#respelling) |  | How a reader says symbols that the captions show. The prompter shows these, and takes are scored against them. |
| `respell` | array of [Respelling](#respelling) |  | Spellings the draft voice says better, applied in order to what it speaks. |
| `seed` | whole number |  | Where a line's first draft takes its seeds from. The seed it keeps goes into `voice.lock`, so changing this changes only the lines drafted afresh. |

## `[timing]`

The pauses around the lines.

| Key | Value | Required | Meaning |
|---|---|---|---|
| `lead_in` | number |  | Silence before the first line. |
| `line_gap` | number |  | The pause between two lines of one section. |
| `section_gap` | number |  | The pause between sections. |
| `tail` | number |  | Silence after the last line. |

## `[lint]`

Wording checks.

| Key | Value | Required | Meaning |
|---|---|---|---|
| `banned` | array of string |  | Regular expressions, matched without regard to case, that no line may show or say. |

## `[style]`

The look of every cut.

| Key | Value | Required | Meaning |
|---|---|---|---|
| `theme` | string |  | `default`, or the path of a theme folder. |

## `[[cuts]]`

The videos made from the script.

| Key | Value | Required | Meaning |
|---|---|---|---|
| `aspect` | `"16:9"` or `"9:16"` | yes | `16:9` or `9:16`. |
| `id` | string | yes | A stable ID, like a line's. |
| `captions` | `"burned"` or `"file"` or `"both"` |  | `burned`, `file` or `both`. Default: `both` for 16:9, `burned` for 9:16. |
| `end_hold` | number |  | How long a 16:9 cut holds its last picture after the narration ends, in seconds from 5 to 20, for an end screen to go over. It takes the place of `timing.tail` in this cut. |
| `lines` | any or array of string |  | `all`, or the IDs of the lines the cut uses, in script order. |
| `platforms` | array of string |  | The platform profiles the cut is made for. Default: `youtube` for 16:9; `youtube-shorts`, `instagram-reels` and `whatsapp-status` for 9:16. |
| `size` | array of whole number |  | The frame size in pixels. Default `[1920, 1080]` or `[1080, 1920]`. |

## `[captions]`

How captions are cut into cues.

| Key | Value | Required | Meaning |
|---|---|---|---|
| `split` | `"readable"` or `"reference"` |  | `readable`: cues of at most two lines of 42 characters, each shown for 0.83 s or more where the next cue leaves room. `reference`: cues as the first pipeline cut them, of up to 108 characters. |

## `[music]`

Background music.

| Key | Value | Required | Meaning |
|---|---|---|---|
| `library` | array of [Track](#track) |  | The tracks the project may use, each with its licence. |
| `track` | string |  | `none`, or the ID of a track in `library`. |
| `under` | number |  | How far under the narration the track plays, in LU. Default 20. |

## `[publish]`

What goes with the video where it is uploaded: its description, tags, links, titles and thumbnails.

| Key | Value | Required | Meaning |
|---|---|---|---|
| `description` | string |  | The description: what the video is about, in your own words. Chapters, links and credits are added after it. |
| `links` | array of [Link](#link) |  | Links to list under the description. |
| `tags` | array of string |  | Tags, most telling first. |
| `thumbnails` | array of [Thumbnail](#thumbnail) |  | Up to three thumbnails for a 16:9 cut, each a frame of a line. With none, there is one: the cut's first line, with the title. |
| `titles` | table of string, by name |  | The title on a platform, by profile ID, where it isn't `meta.title`. |

## `[[errata]]`

Corrections to what a published video says, by line.

| Key | Value | Required | Meaning |
|---|---|---|---|
| `line` | string | yes | The line that is wrong. |
| `on` | string | yes | The day of the correction, as `2026-11-20`. |
| `text` | string | yes | What is right. |

## `[sections.<id>]`

The sections, by ID. They come in the order of their first line.

| Key | Value | Required | Meaning |
|---|---|---|---|
| `chapter` | string |  | The section's title in a chapter list, when it isn't the label. Empty to stay in the chapter before. |
| `label` | string |  | Shown on screen while the section plays, and used for chapters. Empty for none. |

## `[[lines]]`

The script: every narrated line, in order.

| Key | Value | Required | Meaning |
|---|---|---|---|
| `id` | string | yes | A stable ID: lowercase letters and digits in words joined by hyphens. |
| `section` | string | yes | The section the line belongs to. |
| `show` | string | yes | The caption: the line as it is shown. |
| `camera` | string |  | The web scene's camera for landscape cuts; the scene's `default_camera` when absent. |
| `camera_vertical` | string |  | The camera for vertical cuts, when it differs. |
| `claim` | boolean |  | Whether the line states a fact. In strict projects, every line that does needs a `source`; set `claim = false` on a line that states none, such as a greeting or a transition. |
| `do` | array of [Action](#action) |  | Actions on a web scene before the line is captured. |
| `highlight` | array of string |  | Targets of a web scene to box while the line plays. |
| `only` | array of string |  | The cuts the line is only for, such as a short's hook. A cut that uses every line leaves out the lines that are only for others. |
| `overlay` | string |  | A page scene drawn over the line's picture. |
| `say` | string |  | What is said, when it differs from what is shown. |
| `scene` | string |  | The scene the line shows; `meta.default_scene` when absent. |
| `source` | string |  | Where the line's facts come from. |

## `[scenes.<id>]`

What the lines show, by ID.

| Key | Value | Required | Meaning |
|---|---|---|---|
| `kind` | `"web"` or `"page"` or `"card"` or `"image"` or `"terminal"` or `"code"` | yes | `web`: a page captured at a fixed viewport and framed by the virtual camera. `page`: an HTML file rendered at each cut's own size, for cards and overlays. `card`: a card drawn from one of the theme's templates. `image`: a picture in the project folder, PNG or SVG, framed by the virtual camera. `terminal`: a transcript of commands and what they printed, drawn as a terminal. `code`: an excerpt of a source file, drawn with its syntax in colour. |
| `by` | string |  | card: whose words a `quote` card's are. |
| `cameras` | table of [Camera](#camera), by name |  | web: named framings. `viewport` and `highlights` are always defined. |
| `color_scheme` | `"light"` or `"dark"` |  | web: the colour scheme the page is asked for. |
| `columns` | whole number |  | terminal, code: how many characters wide the text is; a longer line goes on in the next row. Default: 80 for a terminal, and the longest line's length for code, between 40 and 100. |
| `default_camera` | string |  | web: the camera of a line that names none. Default `viewport`. |
| `items` | array of string |  | card: the points of a `bullets` card. |
| `language` | string |  | code: the language its syntax is coloured as. Default: by the file's name. |
| `lines` | array of whole number |  | terminal, code: the first and last line of the file to show, counted from 1. Default: all of it. |
| `prompt` | string |  | terminal: what a command's line starts with. Default `$ `. |
| `quote` | string |  | card: the words of a `quote` card. |
| `repo` | string |  | code: the git repository `rev` is in, from the project folder. Default: the project folder's own. |
| `rev` | string |  | code: the git revision the file is read at, as a commit's hash. Without it the file is read as it is in the project folder. |
| `scale` | whole number |  | web: device pixels per CSS pixel. Default 1. image, terminal, code: how many times its own size the picture is drawn. |
| `setup` | array of [Action](#action) |  | web: actions run once, after the page loads. |
| `size` | [PageSize](#pagesize) |  | page: the rendered size for each orientation. Default: the cut's size. |
| `subtitle` | string |  | card: a second, smaller line (`title`, `section`, `end`). |
| `targets` | table of [Target](#target), by name |  | web: named parts of the page, for highlights, cameras, actions and expectations. |
| `template` | `"title"` or `"section"` or `"bullets"` or `"quote"` or `"end"` |  | card: the template it is drawn from. |
| `title` | string |  | card: its title (`title`, `section`, `bullets`, `end`). terminal, code: the words in the window's bar; default: the file's name. |
| `transparent` | boolean |  | page: rendered on a transparent background, as an overlay. |
| `url` | string |  | web: the page's address. page: a file in the project folder; `{aspect}` becomes `16x9` or `9x16`. image: the picture's file. terminal: the transcript's file. code: the source file, in the project folder, or in `repo` at `rev`. |
| `vertical_follows_highlights` | number |  | web: in vertical cuts, a line whose highlights are at most this many CSS pixels wide is framed on its highlights. |
| `viewport` | array of whole number |  | web: the viewport in CSS pixels. Default `[1280, 720]`. |

## `[[expectations]]`

Text that must be on screen when a scene is captured.

| Key | Value | Required | Meaning |
|---|---|---|---|
| `at` | string | yes | `setup`, or the ID of a line on that scene: checked after its actions. |
| `scene` | string | yes | The web scene. |
| `target` | string | yes | The target whose text is read. |
| `text` | string |  | The text of the target's single match. |
| `texts` | array of string |  | The texts of all its matches, in page order. |

## Values with keys of their own

These are written inline, as `{ key = value }`, or as a table of their own.

### HeardAs

One word as the listening check hears it, and the script's word for the same sound.

| Key | Value | Required | Meaning |
|---|---|---|---|
| `heard` | string | yes | The word as whisper writes it. Case is ignored. |
| `means` | string | yes | The script's word for it. |

### Respelling

One text and how it is said.

| Key | Value | Required | Meaning |
|---|---|---|---|
| `say` | string | yes | How it is said. |
| `text` | string | yes | The text as written. |

### Track

A piece of music the project may use, with what allows it.

| Key | Value | Required | Meaning |
|---|---|---|---|
| `attribution` | string | yes | The credit the licence asks for, as it goes into a description. |
| `file` | string | yes | The track's file in the project folder: a 16-bit WAV. |
| `id` | string | yes | A stable ID, like a line's. |
| `licence` | string | yes | The licence it is used under, such as `CC-BY-4.0`, or the name of a paid licence. |
| `platforms` | array of string | yes | The platforms the licence covers. |
| `sha256` | string | yes | The file's SHA-256, so that the licence is of this very file. |
| `content_id` | boolean |  | Whether the track is registered with YouTube's Content ID. A Short over a minute with a Content ID claim is blocked worldwide. |

### Link

A link under a description.

| Key | Value | Required | Meaning |
|---|---|---|---|
| `label` | string | yes | What the link is, such as `The code`. |
| `url` | string | yes | Where it goes: an `http` or `https` address. |

### Thumbnail

A thumbnail: the picture of a line, with words over it.

| Key | Value | Required | Meaning |
|---|---|---|---|
| `line` | string | yes | The line whose picture it is. |
| `title` | string |  | The words drawn over the picture. Default: none. |

### Action

One step on a web page. Exactly one verb is set: `click`, `select`, `expand`, `collapse`, `scroll`, `scroll_to` or `wait`, each naming a target.

| Key | Value | Required | Meaning |
|---|---|---|---|
| `click` | string |  | Click the target. |
| `collapse` | string |  | Close the target if it is open, by clicking `control`. |
| `control` | string |  | For `expand` and `collapse`: the CSS selector, inside the target, to click. |
| `expand` | string |  | Open the target if it is closed (`aria-expanded`), by clicking `control`. |
| `offset` | number |  | For `scroll_to`: the gap left above the target, in CSS pixels. |
| `scroll` | string |  | Scroll the target, a scroll container, to `top`. |
| `scroll_to` | string |  | Scroll `within` until the target is at its top, less `offset`. |
| `select` | string |  | Choose `value` in the target, a `<select>`. |
| `top` | number |  | For `scroll`: the scroll position, in CSS pixels. |
| `value` | string |  | The option value for `select`. |
| `wait` | string |  | Wait until the target is visible. |
| `within` | string |  | For `scroll_to`: the scroll container, a target. |

### Camera

A named framing of a web scene.

| Key | Value | Required | Meaning |
|---|---|---|---|
| `union` | array of string |  | Frame the union of these targets. |
| `vertical` | string |  | The camera vertical cuts use instead, when this one is too wide. |
| `viewport` | boolean |  | Frame the whole viewport. |

### PageSize

A page scene's size for each orientation, in pixels.

| Key | Value | Required | Meaning |
|---|---|---|---|
| `landscape` | array of whole number | yes |  |
| `portrait` | array of whole number | yes |  |

### Target

A named part of a web page: the visible elements that match.

| Key | Value | Required | Meaning |
|---|---|---|---|
| `any` | array of [Target](#target) |  | The union of other matchers, instead of `css`. |
| `attr` | [AttrMatch](#attrmatch) |  | Keep the elements whose attribute matches. |
| `css` | string |  | A CSS selector. |
| `exclude` | string or [TextRule](#textrule) |  | Drop the elements whose text matches. |
| `grow` | [Grow](#grow) |  | Grow each match to its largest enclosing block that still ends with the same text. |
| `line` | whole number |  | One line of the element's text, counted from 0. |
| `lines` | array of whole number |  | On a terminal or code scene: the first and last line of the file that the target is, counted from 1. |
| `merge` | boolean |  | Draw one box around all the matches. |
| `pick` | `"all"` or `"first"` or `"last"` |  | Which matches to keep. Default `all`. |
| `rect` | array of number |  | On an image scene: a part of the picture, as `[x, y, width, height]` in its own pixels (an SVG's, in its own units). |
| `text` | string or [TextRule](#textrule) |  | Keep the elements whose text matches. |
| `text_of` | string |  | Match `text` against this child (a CSS selector) instead of the whole element. |
| `unique` | boolean |  | When reading texts, keep each text once. |

### AttrMatch

An attribute test.

| Key | Value | Required | Meaning |
|---|---|---|---|
| `matches` | string | yes | A regular expression its value must match. |
| `name` | string | yes | The attribute's name. |

### TextRule

A text test other than an exact match. Exactly one key is set.

| Key | Value | Required | Meaning |
|---|---|---|---|
| `contains` | string |  |  |
| `matches` | string |  | A regular expression. |
| `starts_with` | string |  |  |

### Grow

How a match grows to its enclosing block.

| Key | Value | Required | Meaning |
|---|---|---|---|
| `max_height` | number | yes | The tallest block it may grow to, in CSS pixels. |
| `within` | string | yes | The container the match grows within (a CSS selector). |
| `margin` | number |  | The block must stay this many CSS pixels narrower than the container. |
