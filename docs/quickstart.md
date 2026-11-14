# Quickstart: your first video on Linux

This takes you from nothing to a draft video you can watch, in about 15 minutes, most of it downloads. Then it shows the steps from that draft to a finished video in your own voice.

vidni has no installer yet. For now you build it from its source, inside a shell that provides the exact tools it needs.

## What you need

- **Linux on x86_64.**
- **[Nix](https://nixos.org/download/), with flakes on.** Flakes are on when `~/.config/nix/nix.conf` or `/etc/nix/nix.conf` has the line `experimental-features = nix-command flakes`.
- **git.**
- **About 8 GB of disk, and a network connection** for the first run.

## 1. Get vidni and its tools

```sh
git clone https://github.com/dhilipsiva/vidni
cd vidni
nix develop
```

`nix develop` opens a shell with the Rust compiler and the pinned ffmpeg, Chromium, Piper and whisper.cpp, and the fonts vidni draws with. The first time, it downloads them. Every later step runs inside this shell.

## 2. Build vidni

```sh
cargo build --release -p vidni-cli
export PATH="$PWD/target/release:$PATH"
vidni --version
```

## 3. Fetch the draft voice

```sh
vidni voices fetch piper:en_US-joe-medium
```

The draft voice speaks your script until you record your own takes. This one is 63 MB, is checked by its SHA-256, and may be used commercially. `vidni voices` lists the voices and their licences.

## 4. Start a project and draft it

```sh
vidni new my-video
vidni draft my-video
```

`vidni new` makes a folder with a `vidni.toml`, which is the whole video as text, and `page.html`, the page its one scene shows. `vidni draft` speaks each line in the draft voice, captures the page, and renders the cuts:

```text
my-video/build/draft/long.mp4       a 16:9 cut, for YouTube
my-video/build/draft/vertical.mp4   a 9:16 cut, for Shorts, Reels and WhatsApp Status
```

Play either file. That is your first video.

## 5. Make it yours

Open `my-video/vidni.toml`. Each `[[lines]]` table is one narrated line:

```toml
[[lines]]
id = "intro-1"
section = "intro"
show = "Hello, and welcome."
```

Change the words of `show`, add lines, and draft again:

```sh
vidni validate my-video    # says what is wrong, at its line and column
vidni draft my-video       # renders only what changed
```

- **A line's `id` never changes.** Your takes are filed under it.
- **To start from an outline,** write your points in a Markdown file and run `vidni new my-video --from outline.md`. Its headings become sections, and its paragraphs and list items become lines.
- **Every key** is described in [the spec](spec.md).

## 6. From the draft to a finished video

| Step | Command | What it does |
|---|---|---|
| Freeze the script | `vidni ratify my-video` | Records what each cut shows and says. After this, a change to a cut is reported, not silently built. |
| See what to read | `vidni plan my-video` | Prints the recording sheet: each line, and how long the draft voice took over it. |
| Record | `vidni record my-video` | Records your takes in the terminal, one line at a time. Space starts a take and stops it. `--test` checks the microphone first. |
| Build | `vidni final my-video` | Builds each cut from your takes into `my-video/out/`, and checks it. |
| Release | `vidni approve my-video` | Records the build in `release.json`, with the hashes of its files. |

`vidni status my-video` says where each cut stands at any time.

## An example to read

`examples/hello` is a one-minute video about vidni, made of cards and one picture. It needs no browser and no network:

```sh
vidni draft examples/hello --offline
```

Its `vidni.toml` shows cards, a picture under the camera, two cuts and chapters, with a comment on each.

## When something goes wrong

- **`missing or changed; run vidni voices fetch …`:** the draft voice isn't fetched yet. Run step 3.
- **Chromium doesn't start, in a container or as root:** Chromium's own sandbox needs kernel features that containers often withhold. Set `VIDNI_NO_SANDBOX=1` for the commands that capture (`vidni draft`, `vidni capture`), and capture only pages you trust that way.
- **`--offline` fails:** it runs the build in a network namespace, which needs unprivileged user namespaces. Leave `--offline` out; it is a check, and the build is the same without it.
- **`vidni: command not found` in a new terminal:** run `nix develop` in the vidni folder again, then the `export PATH` line of step 2.
