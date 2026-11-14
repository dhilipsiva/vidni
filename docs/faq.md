# FAQ

## What is vidni for?

Narrated videos that explain something: tutorials, walkthroughs, talks to camera without the camera. You write the script as lines in a text file, name what each line shows, and vidni makes the video: first in a synthetic voice, to watch and change, then in your own.

It isn't a video editor. There is no timeline to drag things on, and no footage to cut.

## Does it need an AI model or an account?

No. Every step works on your machine with no language model and no account. Two small models do narrow jobs: Piper speaks the draft voice, and whisper listens to takes for `vidni hear`, which only advises. An assistant may help you write `vidni.toml`, but vidni doesn't need one.

## Why a text file?

Because a video you can't change is a video you'll redo. In `vidni.toml` a line is three or four rows of text. Changing its words changes one take and one stretch of the video, and the rest stays as it was, to the frame. The file goes in git, so a video has a history, and a review of a change to a video is a diff.

## Do I have to record my own voice?

No, but vidni is built for it. `vidni draft` gives you the whole video in a synthetic voice, and you can stop there: a draft is a real MP4. `vidni final` builds from your takes, and refuses until every line has one.

A synthetic voice has a licence. `vidni voices` lists each voice's, and `vidni validate` warns when a project's draft voice doesn't allow commercial use.

## What does "deterministic" mean here?

The same script, takes and captures give the same video. vidni promises it in tiers ([determinism.md](determinism.md)):
- **T1, on any machine:** the frames before encoding, the narration's samples, the captions and the timeline are the same bytes.
- **T2, on one machine:** the MP4 files are the same bytes.
- **T3:** where bytes can't be promised, pictures are compared by how alike they look.

`vidni repro` builds a cut twice, in clean environments, at two paths and on two sets of cores, and compares every hash.

## What happens when the page I'm showing changes?

A video about a live web page narrates what the page showed when it was captured. vidni guards that two ways:
- **Expectations.** You say what text a target must show at a point in the script. A capture stops when the page shows anything else, so a video never narrates a page that changed underneath it.
- **Ratifying.** `vidni ratify` freezes a cut's captures. From then on the cut is built from those pictures, whatever the page does later.

## Can one script make several videos?

Yes. Each `[[cuts]]` table is a video: a 16:9 cut and a 9:16 cut of the whole script, and shorts that pick some of its lines. A line can be only for some cuts, such as a short's hook. A line is recorded once, whichever cuts use it ([Several cuts](spec.md#several-cuts)).

## Why is the picture so small in my vertical cut?

Shorts and Reels cover about 15 % of the top and 35 % of the bottom of a video with their own buttons and labels, and a strip down the right. vidni keeps the title, the view and the captions inside what is left. The review frames show it: `build/draft/review/<cut>/<platform>/`. A cut with `platforms = []` uses the frame's full height ([What the apps cover](spec.md#what-the-apps-cover)).

## Does vidni upload my videos?

No. It writes files into `out/` and stops. Uploading is yours ([platforms.md](platforms.md)).

## Where are my recordings, and are they safe?

In `takes/` inside the project folder, out of git. They are the one thing vidni can't make again, so back them up: `vidni backup --to <place> --to <another>` copies them to two places and lists them, with their hashes, in `assets.lock` ([recording.md](recording.md)).

## What can I delete?

`vidni gc` deletes what can be made again: draft clips, drafts, final builds, and captures that no cut shows. It never touches takes, frozen captures or delivered files. `vidni gc --dry-run` lists what it would delete.

## Which fonts, and can I change the look?

The default theme draws in IBM Plex Sans and IBM Plex Mono. A theme is a small file of colours, fonts and the mark before a label; `[style] theme` names it ([Themes](spec.md#themes)). vidni draws only with fonts it pins by hash, so that text is the same on every machine.

## Does it run on macOS or Windows?

Not yet. vidni is built and tested on Linux. Builds for macOS and Windows are planned ([TODO.md](../TODO.md), P9-VIDNI-02).

## What are the licences?

vidni is MIT OR Apache-2.0. It runs other programs, some under the GPL, and draws with other people's fonts and voices; [THIRD-PARTY-NOTICES.md](../THIRD-PARTY-NOTICES.md) lists each. A video you make is yours.

## Something is wrong. Where do I look?

- `vidni validate` says what is wrong with a project file, at its line and column.
- `vidni status` says where each cut stands, and what changed since it was ratified.
- A build's problems are printed by `vidni draft` as warnings, and stop `vidni final` from delivering.
- [quickstart.md](quickstart.md) lists the first-run problems.
