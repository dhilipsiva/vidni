vidni for macOS and Windows
===========================

vidni makes narrated tutorial videos from a text file. This folder is
vidni's command line, built for your system, with an example. On Linux,
use Nix or the portable bundle instead: docs/install.md in vidni's
repository.

This build is not signed
------------------------

Nobody has paid Apple or Microsoft to vouch for it, so your system asks
before it runs a program it doesn't know.

- macOS says the program "cannot be opened because the developer cannot
  be verified". Take the mark off the folder you unpacked, once:

      xattr -dr com.apple.quarantine <this folder>

- Windows shows "Windows protected your PC". Choose "More info", then
  "Run anyway".

Check the archive against the SHA256SUMS of the release you took it
from before you do either.

What vidni needs beside it
--------------------------

vidni drives other programs, and on these systems you install them:

1. ffmpeg, on your PATH.
       macOS:   brew install ffmpeg
       Windows: choco install ffmpeg      (or winget install ffmpeg)

2. Piper, the draft voice, with the flag that makes it repeat:
       python -m pip install piper-tts==1.8.0
       python setup/seed-piper.py

3. The fonts vidni draws with:
       bash setup/fonts.sh
   (On Windows, in Git Bash. It needs curl, unzip and tar.)

4. The draft voice:
       bin/vidni voices fetch piper:en_US-joe-medium

Then:

       bin/vidni draft examples/hello

examples/hello/build/draft/long.mp4 is the video.

What differs from Linux
-----------------------

- The tools are whatever your system installs, not the ones vidni pins.
  A video still builds the same way twice on your machine; it isn't
  byte for byte the video that Linux builds.
- Web and page scenes need Chromium. On macOS, set VIDNI_CHROMIUM to
  its program. On Windows they aren't supported yet: cards, pictures,
  terminal and code scenes are.
- `--offline`, `vidni repro` and `vidni setup` are for Linux.
- The desktop app isn't in this build.

Licences: LICENSE-MIT and LICENSE-APACHE are vidni's, at your choice.
THIRD-PARTY-NOTICES.md lists what vidni runs and fetches.
