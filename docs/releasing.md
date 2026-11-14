# Releasing vidni

How a release is made, and what its version promises. A release is built from a tag by a workflow; nobody uploads a file by hand.

## Versions

vidni has one version, the workspace's in `Cargo.toml`. The flake, the binaries (`vidni --version`) and the archives' names all read it from there. It follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html), and these are what a version is a promise about:

- **The project file.** A `vidni.toml` that validates keeps validating, and keeps meaning the same video. Its own `schema` number changes only with a major version.
- **The commands:** their names, their flags and their exit codes. What a command prints for people isn't part of the promise.
- **The files vidni writes and reads back:** `ratified.lock`, `release.json`, `voice.lock`, `assets.lock`, the take store.

Before 1.0.0, a minor version (0.2.0) may break any of these, and says so in the changelog; a patch version (0.1.1) never does.

**The bytes of a video aren't promised across versions.** A new version may pin other tools, and a cut built with it then has other hashes. Within one version, they are: that is what the pins are for. A release that was approved stays approved, and `vidni status` says when a cut no longer builds to the bytes that were approved.

## The changelog

`CHANGELOG.md` follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). What a change means for someone who uses vidni goes under **Unreleased** in the commit that makes it. `just release` refuses a version that has no section.

## What a release is made of

`just release` builds four files into `release/v<version>/`:

| File | What it is |
|---|---|
| `vidni-<version>-x86_64-linux.tar.gz` | The portable bundle ([install.md](install.md)) |
| `vidni-tools-x86_64-linux.tar.zst` | The tools that `vidni setup` fetches |
| `SOURCES.txt` | The vidni commit and the nixpkgs revision both were built from, and every package in them |
| `SHA256SUMS` | The SHA-256 of the three files above |

Each is built by Nix from the committed tree and the pinned nixpkgs, with nothing of the machine or the moment in it: sorted archives, one time, one owner, one compression thread. So two runs give the same bytes. `just release --rebuild` shows it: it builds vidni, the tools' archive and the bundle a second time, and fails if any differs from the first.

`just release` refuses a tree with changes that aren't committed.

## Making a release

1. **Decide the version,** and set it in `Cargo.toml` (`[workspace.package] version`). Run `cargo check` so that `Cargo.lock` follows.
2. **Move the changelog's Unreleased entries** under `## [<version>] - <date>`, and leave an empty Unreleased above it.
3. **Check it:** `just lint`, `just test`, `just test-build`, `just test-app`, `just test-bundle`, and `just release --rebuild`.
4. **Commit, and tag the commit** `v<version>`. Push both.
5. **The `release` workflow** runs on the tag. It builds the four files with `just release`, attests where they were built, and makes a draft release with the changelog's section as its text.
6. **Read the draft, and publish it.** Check that its `SHA256SUMS` is the one `just release` gives on your machine for the same tag.

## Checking a release you downloaded

```sh
sha256sum --check SHA256SUMS
gh attestation verify vidni-<version>-x86_64-linux.tar.gz --repo dhilipsiva/vidni
```

The first holds the files to the list. The second asks GitHub whether the file was built by this repository's release workflow, from which commit. The release's files aren't signed with a key of the maintainer's: the attestation is the signature (DECISIONS.md, R5).

`vidni setup` checks the tools' archive against the SHA-256 that the bundle holds, so a bundle that is what `SHA256SUMS` says fetches the tools it was built with or nothing.
