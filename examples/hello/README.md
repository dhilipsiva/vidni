# Example: hello, vidni

A one-minute video about vidni itself, made of four cards and one picture. It needs no browser and no network, and nothing in it is anyone's in particular: copy the folder to start a video of your own.

```sh
just example hello                      # drafts both cuts with no network, into build/draft/
vidni draft examples/hello --offline    # the same, with vidni on your path
```

What it shows of vidni:
- **Cards** drawn from the theme's templates: `title`, `bullets`, `section` and `end` ([Cards](../../docs/spec.md#cards)).
- **A picture under the camera:** `flow.svg`, whose four boxes are targets, with a camera on each step ([Images](../../docs/spec.md#images)).
- **Two cuts from one script:** a 16:9 cut for YouTube and a 9:16 cut for Shorts, Reels and WhatsApp Status. The vertical cut is laid out clear of what those apps cover, and `build/draft/review/vertical/` shows it ([What the apps cover](../../docs/spec.md#what-the-apps-cover)).
- **Chapters:** the long cut's four sections become `build/draft/long.chapters.txt` ([Chapters](../../docs/spec.md#chapters)).
- **What goes with an upload:** `[publish]` holds the description, the tags, a link and a thumbnail, and the long cut holds its last card six seconds for an end screen. `vidni final` writes them into `out/<platform>/<cut>/` once the lines are recorded ([`[publish]`](../../docs/spec.md#publish)).

Every line is spoken by the draft voice. `voice.lock` keeps the seed of each line's take, so a draft on another day has the same audio. To make the final cuts, record the lines with `vidni record`, ratify the cuts and run `vidni final`.
