# Changelog

What changed in each release of vidni, for the people who use it. The format is [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html): see [docs/releasing.md](docs/releasing.md) for what counts as a breaking change here.

## [Unreleased]

## [0.1.0] - not yet released

The first release.

### Added

- **A video as one text file.** `vidni.toml` holds the script, the scenes, the cuts and how the video is published. `vidni validate` checks it and reports each problem at its line and column, `vidni fmt` lays it out, and a JSON Schema describes it to editors. `vidni new` starts a project that drafts as it is, or from a Markdown outline.
- **Drafts with no recording.** `vidni draft` speaks each line in a seeded Piper voice, captures web pages and local pages with a pinned Chromium, composes cards, pictures and captures, and renders every cut. A second draft rebuilds only what changed.
- **Several cuts from one script:** 16:9 and 9:16, with lines of their own, laid out clear of what each platform's app covers.
- **Gates.** `vidni ratify` freezes what a cut shows and says, by hash. `vidni final` builds only a ratified cut, from your takes. `vidni approve` records the build as a release that is never rewritten. `vidni status`, `vidni changes` and `vidni frozen` say where things stand.
- **Recording.** `vidni record` records takes in the terminal, one line at a time, with a meter, a trim to your words and a listening check. `vidni takes` picks another take or trims one elsewhere. Room tone fills the pauses.
- **The narration's sound.** Each take is cleaned, and the whole is levelled, compressed and limited to the loudness its platforms play at, measured as BS.1770 says. Music can play under it.
- **Captions and chapters:** SRT and VTT, a clean master and a captioned picture, captions held to what can be read, and chapters from the sections.
- **Exports per platform.** For YouTube, Shorts, Reels, WhatsApp Status, TikTok, LinkedIn and X: a folder to upload from, with the video, captions, thumbnails, `metadata.md` and `UPLOAD.md`, after the cut is held to the platform's limits. Corrections after publishing go through `[[errata]]`, `vidni published` and `vidni where`.
- **Determinism you can check.** A build records every input and tool. `vidni repro` builds a cut twice in clean environments and compares every hash; `--offline` builds with no network. Frames, sound and captions are the same bytes on any host with the pinned tools.
- **Assets out of git:** takes and captures are locked by hash, and `vidni backup`, `verify`, `restore` and `gc` look after them.
- **A desktop app,** `vidni-app`: projects, a script editor that keeps every comment, a preview player, review notes, the gates, a recording studio and an export panel. Everything it does is a `vidni` command, shown in its help.
- **Two ways to install on Linux:** `nix run github:dhilipsiva/vidni`, and a portable bundle whose `vidni setup` fetches the pinned tools by their SHA-256, with no root. Both draft the same bytes.
- **Scenes of many kinds:** web pages, local pages, cards from templates, pictures (PNG and SVG), a terminal session from a transcript, and code at a git commit with its syntax in colour, all under the same virtual camera.
- **Themes and card templates,** with a neutral default in IBM Plex.
- **Documentation:** a quickstart, guides to recording and to platforms, the full specification, and the licences of everything vidni runs or fetches.
