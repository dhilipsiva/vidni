# Quickstart: your first video on Linux

This takes you from nothing to a draft video you can watch, in about 15 minutes, most of it downloads. Then it shows the steps from that draft to a finished video in your own voice.

This page builds vidni from its source, inside a shell that provides the exact tools it needs. To use vidni without building it, [install it](install.md) and start at [step 4](#4-start-a-project-and-draft-it).

**In a hurry, and you have Nix?** `nix run github:dhilipsiva/vidni -- <command>` is vidni with every tool it needs, built for you: `nix run github:dhilipsiva/vidni -- new my-video`, then `… -- voices fetch piper:en_US-joe-medium` and `… -- draft my-video`. `nix run github:dhilipsiva/vidni#app` is the desktop app. The steps below build vidni from its source, which is what you want for working on it.

## What you need

- **Linux on x86_64.**
- **[Nix](https://nixos.org/download/), with flakes on.** If `nix develop` says `experimental Nix feature 'nix-command' is disabled`, turn them on:

  ```sh
  mkdir -p ~/.config/nix
  echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf
  ```

- **git.**
- **About 12 GB of disk, and a network connection** for the first run.

## 1. Get vidni and its tools

```sh
git clone https://github.com/dhilipsiva/vidni
cd vidni
nix develop
```

`nix develop` opens a shell with the Rust compiler and the pinned ffmpeg, Chromium, Piper and whisper.cpp, and the fonts vidni draws with. The first time, it downloads them and builds Piper, which takes a few minutes and prints little but `copying path …` lines. Every later step runs inside this shell.

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

A project is a folder of its own, anywhere you like. Leave vidni's folder first, so that your video isn't inside its repository:

```sh
cd ~
vidni new my-video
vidni draft my-video
```

In a container, or as root, Chromium needs `VIDNI_NO_SANDBOX=1 vidni draft my-video` ([When something goes wrong](#when-something-goes-wrong)).

`vidni new` makes a folder with a `vidni.toml`, which is the whole video as text, and `page.html`, the page its one scene shows. `vidni draft` speaks each line in the draft voice, captures the page, and renders the cuts:

```text
my-video/build/draft/long.mp4       a 16:9 cut, for YouTube
my-video/build/draft/vertical.mp4   a 9:16 cut, for Shorts, Reels and WhatsApp Status
```

Play either file with any player; `ffplay my-video/build/draft/long.mp4` works from this shell. That is your first video.

Beside the two files are each cut's captions (`.srt` and `.vtt`), and `review/`, a frame of each section to look at. `voice.lock`, in the project folder, keeps the draft voice's take of each line; commit it with `vidni.toml`.

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
vidni draft my-video       # speaks and captures only what changed, then renders the cuts
```

A new line needs only the three keys above. `highlight`, which the first lines have, boxes parts of the page while a line plays.

- **A line's `id` never changes.** Your takes are filed under it.
- **To start from an outline,** write your points in a Markdown file and run `vidni new my-video --from outline.md`. Its headings become sections, and its paragraphs and list items become lines.
- **Every key** is described in [the spec](spec.md).

## 6. From the draft to a finished video

| Step | Command | What it does |
|---|---|---|
| Freeze the script | `vidni ratify my-video` | Records what each cut shows and says. After this, `vidni status` reports any change to a cut, and `vidni final` refuses a changed cut. |
| See what to read | `vidni plan my-video` | Prints the recording sheet: each line, and how long the draft voice took over it. |
| Record | `vidni record my-video` | Records your takes in the terminal, one line at a time. Space starts, stops and retakes; Enter keeps. `--test` checks the microphone first. |
| Build | `vidni final my-video` | Builds each cut from your takes into `my-video/out/`, and checks it. |
| Release | `vidni approve my-video` | Records the build in `release.json`, with the hashes of its files. |

`vidni status my-video` says where each cut stands at any time.

**To change a ratified cut,** edit the file and draft as before; a draft is always built. When the draft is right, `vidni ratify my-video` freezes the cut as it is now. A line whose words changed needs a new take, and `vidni record` goes straight to it.

## An example to read

`examples/hello`, in vidni's folder, is a one-minute video about vidni, made of cards and one picture. It starts no browser and needs no network:

```sh
cd ~/vidni                      # wherever you cloned it
vidni draft examples/hello
```

`vidni draft examples/hello --offline` proves the second part: it runs the build where there is no network. That needs user namespaces, which most containers withhold.

Its `vidni.toml` shows cards, a picture under the camera, two cuts and chapters, with a comment on each.

## When something goes wrong

- **`missing or changed; run vidni voices fetch …`:** the draft voice isn't fetched yet. Run step 3.
- **Chromium doesn't start, in a container or as root:** Chromium's own sandbox needs kernel features that containers often withhold. Set `VIDNI_NO_SANDBOX=1` for the commands that capture (`vidni draft`, `vidni capture`), and capture only pages you trust that way.
- **`--offline` fails:** it runs the build in a network namespace, which needs unprivileged user namespaces. Leave `--offline` out; it is a check, and the build is the same without it.
- **`vidni: command not found` in a new terminal:** run `nix develop` in the vidni folder again, then the `export PATH` line of step 2.
