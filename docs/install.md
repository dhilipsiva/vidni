# Installing vidni

vidni is made for Linux, x86_64, where there are two ways to get it. For macOS and Windows there are unsigned builds of the command line: see [the end of this page](#macos-and-windows).

On Linux there are two ways to get it, and they give the same vidni with the same tools: a video drafted by one is, byte for byte, the video drafted by the other.

| | With Nix | The portable bundle |
|---|---|---|
| For | NixOS, or any Linux with [Nix](https://nixos.org/download/) | any other Linux |
| Needs root | no | no |
| Download | Nix fetches what it needs, about 6 GB | 40 MB, then about 1 GB for the tools |
| The desktop app | yes | no |

## With Nix

```sh
nix run github:dhilipsiva/vidni -- new my-video
nix run github:dhilipsiva/vidni -- voices fetch piper:en_US-joe-medium
nix run github:dhilipsiva/vidni -- draft my-video
nix run github:dhilipsiva/vidni#app          # the desktop app
```

`nix run` starts vidni with the tools it drives on its path: the pinned ffmpeg, Chromium, Piper and whisper.cpp, the fonts and the listening model. Nix needs flakes on; [the quickstart](quickstart.md#what-you-need) says how. To keep the command, `nix profile install github:dhilipsiva/vidni`.

## The portable bundle

For a Linux that has no Nix. Nothing is installed outside the bundle's folder, your cache and your voice store, and nothing needs root.

1. **Download** `vidni-<version>-x86_64-linux.tar.gz` from [the releases](https://github.com/dhilipsiva/vidni/releases), and check it against the release's `SHA256SUMS`.
2. **Unpack it, and put its `bin` on your path:**

   ```sh
   tar xzf vidni-*-x86_64-linux.tar.gz
   cd vidni-*-x86_64-linux
   export PATH="$PWD/bin:$PATH"
   ```

3. **Fetch the tools:**

   ```sh
   vidni setup
   ```

   It downloads one archive of about 1 GB with the tools (ffmpeg, Chromium, Piper, whisper.cpp, the fonts), then the draft voice and the listening model. Each file is checked against the SHA-256 that this vidni pins, and one that differs is refused. It takes a few minutes, most of it the download. It needs about 4.5 GB of disk while it works, and 3 GB afterwards.
4. **Make a video:**

   ```sh
   vidni new my-video
   vidni draft my-video
   ```

   Or draft the bundle's example: `cp -r share/vidni/examples/hello . && vidni draft hello`.

From here on, [the quickstart](quickstart.md#5-make-it-yours) applies as it is.

### What `vidni setup` does

- **Where things go.** The tools are unpacked into `~/.cache/vidni/tools/` (under `$XDG_CACHE_HOME` when that is set), the listening model into `~/.cache/vidni/models/`, and the draft voice into the voice store, `~/.local/share/vidni/voices/`. Each vidni names its tools' folder by their SHA-256, and reads the `env.sh` in it each time it starts, which puts the tools on its path. So a vidni never runs with another version's tools.
- **The cache's path must be plain:** letters, digits and `/ . _ - + @`. The tools' scripts name each other by their paths, and a space or a quote would break them, so `vidni setup` refuses such a folder; set `XDG_CACHE_HOME` to another.
- **The tools are the flake's own.** They are the programs that Nix built for vidni's flake, packed with everything they load. `vidni setup` moves them to your cache: it sets each program's loader and library paths to where they are now, with the `patchelf` the bundle carries, and rewrites the same paths in scripts and links. That is why the two ways of installing draft the same bytes.
- **From a mirror:** `vidni setup --mirror <address>` fetches the tools' archive from that address's folder, and holds it to the same SHA-256.
- **With no network:** copy the tools' archive (`vidni-tools-x86_64-linux.tar.zst`, from the same release) to a folder, and run `vidni setup --from <folder>`. The voice and the listening model still come from the network; a machine that can't reach it takes them from another's `~/.local/share/vidni/voices` and `~/.cache/vidni/models`.
- **Run again,** it finds what is in place and fetches only what is missing. A newer vidni that pins other tools needs its own `vidni setup`, which unpacks them beside the old ones; delete the old folder under `~/.cache/vidni/tools/` when you no longer need it.
- **What isn't moved.** A few libraries hold a folder's path inside themselves, where it can't be rewritten: the C library's character-set converters and locales, OpenSSL's configuration, and fontconfig's default file. vidni's tools don't need the first three, and are told where the fonts' file and the certificates are. A program you run from the tools' folders yourself may miss them.
- **To remove vidni:** delete the bundle's folder, `~/.cache/vidni` and `~/.local/share/vidni`.

### What the bundle doesn't have

- **The desktop app.** It opens its window with the system's graphics libraries, which a bundle can't carry for every system. `nix run github:dhilipsiva/vidni#app` has it.
- **A sure microphone.** `vidni record` in the bundle finds sound devices through the bundle's own ALSA library and its own configuration, without your system's PipeWire or PulseAudio plugins. `ALSA_CONFIG_PATH` names another configuration. `VIDNI_MIC` names a device when the default one isn't found. Recording through the bundle hasn't been tried on real hardware yet.
- **Mermaid and git.** A Mermaid diagram needs `mmdc`, which the bundle doesn't carry (`VIDNI_MERMAID` names yours), and a code scene at a commit uses the system's git.
- **Chromium's sandbox, in some places.** In a container, as root, or on a system that forbids unprivileged user namespaces (Ubuntu 24.04 does by default), Chromium can't make its sandbox. Set `VIDNI_NO_SANDBOX=1` for the commands that capture, and capture only pages you trust that way.
- **`--offline` and `vidni repro`** need unprivileged user namespaces too.

### How it was tried

`just test-bundle` builds the bundle, follows the steps above in a clean container of a Linux with no Nix, as a user who isn't root, and drafts a new project and the example. It then drafts the same two with the flake's tools and compares: the build records and the videos are the same bytes. It has passed on Debian stable, Ubuntu 24.04 and Fedora. Until there is a release to download from, the test fetches the tools' archive from a folder that stands in for the release's address.

### Passing the bundle on

The two archives hold programs under the GPL and the LGPL, among other licences. Whoever gives them to someone else takes on those licences' duties, the offer of source above all. [THIRD-PARTY-NOTICES.md](../THIRD-PARTY-NOTICES.md#the-portable-bundle) says what is in the archives and where each program's source is; the bundle carries that file and vidni's two licences in `share/doc/vidni/`.

## macOS and Windows

The `platforms` workflow builds vidni's command line for macOS (Apple silicon) and Windows (x86_64) and packs each with its licences, the hello example and two setup scripts. The archives are on each run of that workflow, and on a release when it has them. `README.txt` in the archive has the steps; in short:

1. **Let the system run it.** The builds aren't signed, since signing needs certificates that nobody has paid for.
   - **macOS** says the program "cannot be opened because the developer cannot be verified". Take the quarantine mark off the folder once: `xattr -dr com.apple.quarantine <the folder>`.
   - **Windows** shows "Windows protected your PC". Choose **More info**, then **Run anyway**.
   - Check the archive against the release's `SHA256SUMS` before you do either.
2. **Install what vidni drives:** ffmpeg (`brew install ffmpeg`, or `choco install ffmpeg`), and Piper with its seed flag (`python -m pip install piper-tts==1.8.0`, then `python setup/seed-piper.py`).
3. **Fetch the fonts:** `bash setup/fonts.sh` (on Windows, in Git Bash). vidni checks each file against its pinned SHA-256.
4. **Fetch the draft voice and draft:** `bin/vidni voices fetch piper:en_US-joe-medium`, then `bin/vidni draft examples/hello`.

**What differs from Linux:**

- **The tools are your system's, not vidni's pins.** A video builds the same way twice on your machine. It isn't byte for byte the video Linux builds: another ffmpeg encodes other bytes, and the draft voice's takes come out a little longer or shorter on another system's maths library.
- **Web and page scenes** need Chromium. On macOS, `VIDNI_CHROMIUM` names its program; this wasn't tried on a runner. On Windows they aren't supported: vidni hands Chromium its pipes in a way only Unix has. Cards, pictures, terminal and code scenes work on both.
- **`--offline`, `vidni repro` and `vidni setup`** are Linux's.
- **The desktop app and recording** weren't tried on either system.

**How it was tried:** on GitHub's `macos-latest` and `windows-latest` runners, the workflow builds the command, packs it, unpacks the archive elsewhere, sets up as above, and drafts the hello example from the package. The 16:9 cut came to 2,131 frames (71.0 s) on Windows and 2,120 frames (70.6 s) on macOS: the draft voice's takes differ a little in length between systems, which is the first of the differences above.
