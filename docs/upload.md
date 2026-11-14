# Uploading to YouTube

vidni uploads nothing by itself. A release is built into a folder, and you upload it by hand ([platforms.md](platforms.md)). `vidni upload` is there for someone who would rather have vidni send a release to YouTube: it is off until you set it up, and it asks before each upload.

> **No live upload has been made with this command.** It was written against YouTube's published Data API, and its tests run against a made-up API on the test's own machine: the sign-in, the upload's two requests, and what is said when the service refuses. Until someone has uploaded with it for real, expect that something may not fit, and check your channel after the first upload.

## What it does

```console
$ vidni upload --cut long
to upload to YouTube, as private:
  release  long-r2
  file     out/youtube/long/long.mp4 (48.2 MB)
  title    The LLM drafts, the engine decides
type the release's name to upload it, or anything else to stop:
long-r2
long: long-r2 is on YouTube, private, at https://youtu.be/…
  it is recorded in published.json; captions and the thumbnail are added by hand, from the upload's folder
```

- **What goes up is a release:** the cut's newest approved build, as `vidni final` delivered it to `out/youtube/<cut>/`. A cut with no release, or whose delivered file isn't the approved one, is refused.
- **You confirm each upload** by typing the release's name. Anything else stops it, and nothing is sent.
- **It goes up private** unless you say `--privacy unlisted` or `--privacy public`. Look at it on YouTube, and change its visibility there.
- **The title, description and tags** are the ones in the upload's `metadata.md`, from `[publish]` in `vidni.toml`.
- **It is recorded** in `published.json` with its release, as `vidni published` would record it, so `vidni where` and the next `UPLOAD.md` know of it.
- **A release is uploaded once.** A second `vidni upload` of the same release to the same platform is refused.
- **A 9:16 cut for Shorts** is `vidni upload --cut vertical --platform youtube-shorts`.
- **Not uploaded:** the captions and the thumbnail. Add them in YouTube Studio from the upload's folder, as its `metadata.md` says.
- **An assistant can't run it** ([assistant.md](assistant.md)).

## Setting it up

YouTube lets a program upload for you only through an OAuth client that you have registered with Google. vidni has none of its own: you make yours, and it stays on your machine.

1. **Make an OAuth client.** In the [Google Cloud console](https://console.cloud.google.com/), make a project, enable the **YouTube Data API v3**, and under **Credentials** create an **OAuth client ID** of the type **Desktop app**. On the consent screen, add yourself as a test user.
2. **Put its ID and secret in a file outside any project,** readable by you alone:

   ```sh
   mkdir -p ~/.config/vidni
   cat > ~/.config/vidni/youtube.json <<'JSON'
   { "client_id": "….apps.googleusercontent.com", "client_secret": "…" }
   JSON
   chmod 600 ~/.config/vidni/youtube.json
   ```

   `VIDNI_YOUTUBE` names another file. vidni refuses one that is inside the project, or that others on the machine can read.
3. **Sign in:**

   ```sh
   vidni upload --login
   ```

   It prints an address. Open it in your browser, choose your channel's account, and allow vidni to upload videos. The browser then comes back to vidni on your own machine, and vidni keeps the token it is given in the same file.

To take the permission back, remove vidni's access in your [Google account's connections](https://myaccount.google.com/connections), and delete the file.

## What it asks of YouTube, and what YouTube limits

- **The permission** is `youtube.upload` alone: vidni can add videos to your channel, and can't read, change or delete anything.
- **Quota.** An upload costs a large part of a project's daily quota, so a new client can upload a handful of videos a day.
- **An unverified client's uploads are locked private** by YouTube until the client has passed its audit. That is YouTube's rule for every API client, and one more reason vidni uploads as private.
- **When YouTube refuses,** vidni says what it said, and records nothing as published.
