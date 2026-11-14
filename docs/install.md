# Installing vidni on Linux

vidni runs on Linux, x86_64. There are two ways to get it, and they give the same vidni with the same tools: a video drafted by one is, byte for byte, the video drafted by the other.

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
- **Chromium's sandbox, in some places.** In a container, as root, or on a system that forbids unprivileged user namespaces (Ubuntu 24.04 does by default), Chromium can't make its sandbox. Set `VIDNI_NO_SANDBOX=1` for the commands that capture, and capture only pages you trust that way.
- **`--offline` and `vidni repro`** need unprivileged user namespaces too.

### How it was tried

`just test-bundle` builds the bundle, follows the steps above in a clean container of a Linux with no Nix, as a user who isn't root, and drafts a new project and the example. It then drafts the same two with the flake's tools and compares: the build records and the videos are the same bytes. It has passed on Debian stable, Ubuntu 24.04 and Fedora. Until there is a release to download from, the test fetches the tools' archive from a folder that stands in for the release's address.

### Passing the bundle on

The two archives hold programs under the GPL and the LGPL, among other licences. Whoever gives them to someone else takes on those licences' duties, the offer of source above all. [THIRD-PARTY-NOTICES.md](../THIRD-PARTY-NOTICES.md#the-portable-bundle) says what is in the archives and where each program's source is; the bundle carries that file and vidni's two licences in `share/doc/vidni/`.
