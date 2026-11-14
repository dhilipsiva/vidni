vidni, the portable bundle for Linux (x86_64)
=============================================

vidni makes narrated tutorial videos from a text file. This folder is
vidni's command line with everything it needs to start, for a Linux that
has no Nix. Nothing here needs root, and nothing is written outside
this folder, your cache (~/.cache/vidni) and your voice store
(~/.local/share/vidni).

1. Put this folder where you want it, and its bin/ on your PATH:

       export PATH="$PWD/bin:$PATH"

2. Fetch the tools vidni drives (ffmpeg, Chromium, Piper, whisper.cpp
   and the fonts), the draft voice and the listening model:

       vidni setup

   It downloads about 1 GB, checks each file's SHA-256, and unpacks into
   ~/.cache/vidni (or $XDG_CACHE_HOME/vidni). It needs about 4.5 GB of
   disk while it works, and 3 GB afterwards. `vidni setup --from DIR`
   takes the tools' archive from a folder; the voice and the listening
   model (about 210 MB) still come from the network.

3. Make a video:

       vidni new my-video
       vidni draft my-video

   my-video/build/draft/long.mp4 is your first video.

4. Or draft the example, a one-minute video about vidni itself:

       cp -r share/vidni/examples/hello .      (from this folder)
       vidni draft hello

docs: https://github.com/dhilipsiva/vidni/blob/main/docs/quickstart.md

What isn't in the bundle: the desktop app (`nix run
github:dhilipsiva/vidni#app` has it). In a container, or as root,
Chromium needs VIDNI_NO_SANDBOX=1.

Licences: share/doc/vidni/ has vidni's two licences (MIT and Apache-2.0,
at your choice) and THIRD-PARTY-NOTICES.md, for the programs, fonts,
voices and models that vidni runs or fetches. Some of them are under
the GPL; the notices say where their source is.
