# Platform guide

What vidni makes for each platform, and what to check before you upload. The facts are the platform profiles': `vidni platforms` prints each one with the date it was checked and the pages it was checked against. This page explains them; where the two differ, the profile is right.

**Platforms change their limits without notice.** Each profile carries the day it was verified. Confirm a limit in the app before you rely on it.

## Which cut goes where

A cut names the platforms it is made for:

```toml
[[cuts]]
id = "short"
aspect = "9:16"
platforms = ["youtube-shorts", "instagram-reels"]
```

Without `platforms`, a cut is for the profiles that are on by default for its shape.

| Profile | Shape | On by default | Captions | Upload up to |
|---|---|---|---|---|
| `youtube` | 16:9 | yes | a caption file | 15 minutes, or 12 hours on a verified account |
| `youtube-shorts` | 9:16 | yes | drawn in | 3 minutes |
| `instagram-reels` | 9:16 | yes | drawn in | 20 minutes; 3 are recommended |
| `whatsapp-status` | 9:16 | yes | drawn in | 90 seconds |
| `tiktok` | 9:16 | no | drawn in | 30 minutes |
| `linkedin` | 16:9 | no | a caption file | 15 minutes |
| `x` | 16:9 | no | drawn in | 2 minutes 20 seconds on a free account |

`platforms = []` makes a cut for no platform in particular: nothing is assumed about an app, and the 9:16 layout uses the frame's full height.

## One master for every platform

None of these platforms publishes a loudness target for uploads. YouTube's playback normalization has been measured at −14 LUFS, and the other profiles use the same figure with peaks at −1 dBTP, so one master suits them all.

## YouTube, long form

- **Upload the clean master,** `<cut>.mp4`, and its caption file, `<cut>.<language>.srt`, in YouTube Studio. Viewers can turn captions off, and YouTube can translate them. `<cut>.captioned.mp4` is the same picture with the captions drawn in, for places that take no caption file.
- **Paste `<cut>.chapters.txt` into the description.** YouTube shows chapters only when the list starts at 00:00, has three or more, and each lasts 10 seconds or longer; vidni makes no list from fewer than three, and QA fails a chapter that is too short ([Chapters](spec.md#chapters)).
- **15 minutes is the limit until the account verifies a phone number.**

## Shorts, Reels and WhatsApp Status

- **The apps cover part of the picture:** buttons down the right, the account and the description along the bottom, tabs across the top. vidni lays a 9:16 cut out clear of what its platforms cover together, QA lists anything that would sit under an app, and the review frames in `build/draft/review/<cut>/<platform>/` show each app's share shaded ([What the apps cover](spec.md#what-the-apps-cover)).
- **The figures are the platforms' own guidance for ads.** None publishes margins for ordinary posts, and WhatsApp publishes none at all. `vidni platforms --grid DIR` makes a grid clip to upload privately and measure an app with.
- **Captions are drawn in.** Shorts' caption-file upload isn't documented, Reels and Status take no caption file, and vidni's shorts are always captioned.
- **Length:**
  - A Short may run 3 minutes. One over 1 minute with an active Content ID claim is blocked worldwide, so keep music out of longer Shorts.
  - Reels over 3 minutes aren't recommended to new audiences.
  - Of a Status video over 90 seconds, only the first 90 are shared.

## TikTok, LinkedIn and X

These profiles exist, and are off by default. Name one in a cut's `platforms` to make the cut for it.
- **TikTok** covers more of the right edge than the others, so a cut that includes it has a narrower view.
- **LinkedIn** takes an SRT file, attached on the desktop site before posting.
- **X** takes 2 minutes 20 seconds and 512 MB from a free account, at 1280×720; its paid plans take more.

## Where you are matters

- **India: TikTok is banned,** and the profile records that no order has lifted the ban. Its pages don't load from India, so its limits were read from archived copies. A cut made for it can't be checked in the app from there.
- **X blocks automated reading,** so its limits were read from archived copies too. Check them in the app.
- **Account type changes the limits:** YouTube's 15 minutes, X's 2 minutes 20, and what the upload APIs take all differ from what a verified or paying account gets. The profiles give the limit of an ordinary account, and their notes give the others.

## Uploads are yours

vidni writes files and uploads nothing. Each release is in `out/`, recorded in `release.json` with the hashes of its files, so you can tell which file you uploaded where.
