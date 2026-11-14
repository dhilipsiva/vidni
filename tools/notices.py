#!/usr/bin/env python3
"""Regenerate THIRD-PARTY-NOTICES.md (TODO.md, P8-VIDNI-04): `just notices`.

Everything in the notices is read from where vidni pins it:
 - the Rust crates linked into vidni, from `cargo metadata` (Cargo.lock);
 - the fonts, from crates/vidni-core/src/fonts.toml, with each font's
   copyright line read out of the pinned file itself ($VIDNI_FONTS);
 - the draft voices, from crates/vidni-core/src/voices.toml;
 - the tools vidni runs, from the nixpkgs revision in flake.lock;
 - the listening model, from flake.nix.

usage: notices.py write|check
"""
import json
import os
import re
import struct
import subprocess
import sys
import tomllib

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "THIRD-PARTY-NOTICES.md")

# The programs vidni runs, as nixpkgs names them, and what vidni uses each for.
TOOLS = [
    ("ffmpeg-full", "ffmpeg", "encodes each cut, measures it, and normalizes loudness"),
    ("x264", "x264", "the H.264 encoder inside that ffmpeg"),
    ("chromium", "Chromium", "captures web and page scenes"),
    ("piper-tts", "Piper", "speaks the draft voice"),
    ("espeak-ng", "eSpeak NG", "turns text into phonemes for Piper"),
    ("onnxruntime", "ONNX Runtime", "runs Piper's voice models"),
    ("whisper-cpp", "whisper.cpp", "listens to takes for `vidni hear`"),
    ("curl", "curl", "downloads a voice for `vidni voices fetch`, and talks to YouTube for `vidni upload`"),
    ("git", "git", "reads a code scene's file at a commit"),
    ("mermaid-cli", "Mermaid CLI", "draws a Mermaid diagram for an image scene"),
]

# The listening model, as flake.nix fetches it.
WHISPER = {
    "name": "ggml-base.en.bin",
    "url": "https://huggingface.co/ggerganov/whisper.cpp/resolve/5359861c739e955e79d9a303bcbc70fb988958b1/ggml-base.en.bin",
    "sha256": "a03779c86df3323075f5e796cb2ce5029f00ec8869eee3fdfb897afe36c6d002",
    "licence": "MIT",
    "notes": "OpenAI's Whisper base.en model, converted for whisper.cpp.",
    "source": "https://github.com/openai/whisper",
}


def read(*parts):
    with open(os.path.join(ROOT, *parts), "rb") as f:
        return f.read()


def run(*args):
    return subprocess.run(args, cwd=ROOT, check=True, capture_output=True, text=True).stdout


def crates():
    """The crates linked into the vidni binaries on Linux: every package a
    workspace member reaches through normal dependencies."""
    meta = json.loads(run("cargo", "metadata", "--format-version", "1", "--locked",
                          "--filter-platform", "x86_64-unknown-linux-gnu"))
    packages = {p["id"]: p for p in meta["packages"]}
    nodes = {n["id"]: n for n in meta["resolve"]["nodes"]}
    members = set(meta["workspace_members"])
    # vidni-parity is a development tool, and is in no release.
    roots = [i for i in members if packages[i]["name"] != "vidni-parity"]
    seen, todo = set(), list(roots)
    while todo:
        node = todo.pop()
        if node in seen:
            continue
        seen.add(node)
        for dep in nodes[node]["deps"]:
            if any(kind["kind"] is None for kind in dep["dep_kinds"]):
                todo.append(dep["pkg"])
    linked = [packages[i] for i in seen - members]
    return sorted(linked, key=lambda p: (p["name"], p["version"]))


def font_copyright(path):
    """The copyright line (name ID 0) of an OpenType file."""
    data = open(path, "rb").read()
    tables = struct.unpack(">H", data[4:6])[0]
    for i in range(tables):
        tag, _, offset, _ = struct.unpack(">4sIII", data[12 + 16 * i:28 + 16 * i])
        if tag != b"name":
            continue
        _, count, strings = struct.unpack(">HHH", data[offset:offset + 6])
        found = {}
        for j in range(count):
            platform, _, _, name, length, at = struct.unpack(
                ">HHHHHH", data[offset + 6 + 12 * j:offset + 18 + 12 * j])
            if name == 0:
                raw = data[offset + strings + at:offset + strings + at + length]
                found[platform] = raw.decode("utf-16-be" if platform in (0, 3) else "mac-roman")
        # Windows names first, as the fuller ones.
        for platform in (3, 0, 1):
            if platform in found:
                return " ".join(found[platform].split())
    raise SystemExit(f"{path}: no copyright in its name table")


def find_font(name):
    for folder in os.environ.get("VIDNI_FONTS", "").split(":"):
        for base, _, files in os.walk(folder) if folder else ():
            if name in files:
                return os.path.join(base, name)
    raise SystemExit(f"{name} isn't in $VIDNI_FONTS; run `just notices` in vidni's nix shell")


def nix(package, attribute, apply=None):
    args = ["nix", "eval", "--json", "--inputs-from", ".", f"nixpkgs#{package}.{attribute}"]
    if apply:
        args += ["--apply", apply]
    return json.loads(run(*args))


def tools():
    spdx = "l: map (x: x.spdxId or x.shortName) (if builtins.isList l then l else [ l ])"
    return [
        {
            "name": name,
            "use": use,
            "version": nix(package, "version"),
            "licences": nix(package, "meta.license", spdx),
            "home": nix(package, "meta.homepage"),
        }
        for package, name, use in TOOLS
    ]


def table(head, rows):
    lines = ["| " + " | ".join(head) + " |", "|" + "---|" * len(head)]
    lines += ["| " + " | ".join(str(cell).replace("|", "\\|") for cell in row) + " |" for row in rows]
    return "\n".join(lines)


def notices():
    flake = read("flake.nix").decode()
    for pinned in (WHISPER["url"], WHISPER["sha256"]):
        if pinned not in flake:
            raise SystemExit(f"flake.nix no longer fetches {pinned}; update tools/notices.py")
    lock = json.loads(read("flake.lock"))
    nixpkgs = lock["nodes"][lock["nodes"]["root"]["inputs"]["nixpkgs"]]["locked"]["rev"]
    fonts = tomllib.loads(read("crates/vidni-core/src/fonts.toml").decode())["font"]
    voices = tomllib.loads(read("crates/vidni-core/src/voices.toml").decode())["voice"]
    linked = crates()
    programs = tools()
    gpl = [t for t in programs if any(l.startswith("GPL") for l in t["licences"])]

    parts = [f"""# Third-party notices

vidni itself is licensed under MIT OR Apache-2.0 ([LICENSE-MIT](LICENSE-MIT), [LICENSE-APACHE](LICENSE-APACHE)).

This file lists what vidni is built from, what it runs, and what it reads, each with its licence. It is generated: `just notices` writes it again from `Cargo.lock`, `fonts.toml`, `voices.toml`, `flake.nix` and `flake.lock`, and `just notices check` fails when it is out of date. Don't edit it by hand.

A video you make with vidni is yours. Two things in it come from others: the fonts its text is drawn in, and, until you record your own takes, the draft voice. Their licences are below.

## Programs vidni runs

vidni runs these as separate programs. None of them is part of the `vidni` binary, and vidni's own licence doesn't change theirs. The versions are the ones vidni's flake pins (nixpkgs `{nixpkgs}`).

""" + table(
        ["Program", "Version", "Licence", "What vidni uses it for", "Home"],
        [[t["name"], t["version"], ", ".join(t["licences"]), t["use"], t["home"]] for t in programs],
    )]

    names = ", ".join(t["name"] for t in gpl)
    parts.append(f"""### The GPL programs, and their source

{names} are under the GNU General Public License. The ffmpeg that vidni pins is built with `--enable-gpl` and libx264, which makes the whole of that ffmpeg GPL-3.0-or-later.

- **Running them changes nothing for you.** vidni starts each as a program of its own, so their licence covers neither vidni nor your videos.
- **Passing them on does.** Anyone who gives another person a copy of one of these programs, alone or inside a bundle with vidni, must give that person its complete corresponding source as well, or a written offer of it, under the same licence (GPL version 3, section 6).
- **Where the source is.** Each program is built from the recipe in nixpkgs at revision `{nixpkgs}`. `nix build --inputs-from . nixpkgs#<name>.src` fetches the exact source, with <name> one of: {", ".join("`" + p + "`" for p, n, _ in TOOLS if any(t["name"] == n for t in gpl))}. The recipes and their patches are at <https://github.com/NixOS/nixpkgs/tree/{nixpkgs}>, and vidni's one patch to Piper is [`nix/piper-seed.patch`](nix/piper-seed.patch).

Chromium is under a BSD-style licence and includes many other projects; its own list is at `chrome://credits` in the browser.""")

    parts.append(f"""## The portable bundle

The portable bundle for Linux ([docs/install.md](docs/install.md)) is two archives, and both pass programs on:

- **`vidni-<version>-x86_64-linux.tar.gz`** holds vidni, and what starts it on a Linux with no Nix: the GNU C Library and GCC's runtime libraries, ALSA's library, curl with the libraries it loads (OpenSSL and MIT Kerberos among them) and Mozilla's list of certificate authorities, patchelf, Bash and GNU coreutils, and the libraries those load in turn.
- **`vidni-tools-x86_64-linux.tar.zst`**, which `vidni setup` fetches, holds the programs of the table above in the builds the bundle uses (ffmpeg without what plays and what reads the network, Piper without what trains voices), the fonts, and everything those programs load: some 400 packages, from the C library to Python.

Every package in either archive is in it exactly as Nix built it from nixpkgs at revision `{nixpkgs}`, by vidni's `flake.nix`. Their folders keep the names Nix gave them, `<hash>-<name>-<version>`, so each can be traced:

- **What is in an archive:** its packages are the folders under `store/`. `tar tf vidni-tools-x86_64-linux.tar.zst | cut -d/ -f2 | sort -u` lists the tools' archive's, and `tar tzf vidni-<version>-x86_64-linux.tar.gz | grep /store/ | cut -d/ -f3 | sort -u` the other's.
- **A package's licence** is in its recipe in nixpkgs, under `meta.license`: <https://search.nixos.org/packages> finds the recipe by name.
- **A package's source:** `nix derivation show /nix/store/<its folder>` names the source its build fetched, pinned by hash, and its patches; `nix build --inputs-from . nixpkgs#<name>.src` fetches it. The recipes are at <https://github.com/NixOS/nixpkgs/tree/{nixpkgs}>.

Many of these packages are under the GPL or the LGPL (the C library, GCC's runtime, Bash, coreutils, patchelf, ffmpeg, x264, Piper and eSpeak NG among them). **Whoever gives either archive to someone else must offer that person the source of each of them,** as the section above says. The lines above are how the source is found; a release of vidni that carries the archives names the vidni commit and the nixpkgs revision they were built from, which fixes every source.""")

    parts.append("""## Fonts

vidni draws text in these fonts, and their outlines end up in your video's frames. Their licences allow that, for any purpose.

- **OFL-1.1** is the SIL Open Font License 1.1 (<https://openfontlicense.org>). It asks that the font files themselves are passed on only with their licence and under their own names.
- **Bitstream-Vera** is the licence of the Bitstream Vera fonts, which DejaVu is made from (<https://dejavu-fonts.github.io/License.html>). It asks the same of the font files, and that a changed font isn't called Bitstream or Vera. DejaVu's own changes are in the public domain.

""" + table(
        ["Font", "Version", "Licence", "Copyright", "Source"],
        [[f'{f["family"]} {f["style"]}', f["version"], f["licence"],
          font_copyright(find_font(f["file"])), f["source"]] for f in fonts],
    ))

    parts.append("""## Draft voices

A draft voice is a model trained on recordings of a person, and its licence is its training data's. It decides what you may do with audio the voice speaks. `vidni voices` prints this list, and `vidni validate` warns when a project's draft voice doesn't allow commercial use.

""" + table(
        ["Voice", "Licence", "Commercial use", "Notes", "Model card"],
        [[v["id"], v["licence"], "yes" if v["commercial"] else "no", v["notes"], v["source"]]
         for v in voices],
    ))

    parts.append("""## Models

""" + table(
        ["Model", "Licence", "Notes", "Source", "SHA-256"],
        [[WHISPER["name"], WHISPER["licence"], WHISPER["notes"], WHISPER["source"], WHISPER["sha256"]]],
    ) + """

`vidni hear` listens with this model. Nothing it hears changes an output.""")

    licences = sorted({p.get("license") or "not stated" for p in linked})
    parts.append(f"""## Rust crates

The `vidni` binaries are built from these {len(linked)} crates, each under the licence its authors chose ({len(licences)} licence expressions in all). Each crate's own licence files are in its source, at the address given.

""" + table(
        ["Crate", "Version", "Licence", "Source"],
        [[p["name"], p["version"], p.get("license") or "not stated",
          p.get("repository") or f'https://crates.io/crates/{p["name"]}'] for p in linked],
    ))
    return "\n\n".join(parts) + "\n"


def main():
    if len(sys.argv) != 2 or sys.argv[1] not in ("write", "check"):
        sys.exit(__doc__)
    text = notices()
    if sys.argv[1] == "write":
        with open(OUT, "w") as f:
            f.write(text)
        print(f"{os.path.relpath(OUT, ROOT)}: written")
        return
    current = open(OUT).read() if os.path.exists(OUT) else ""
    if current != text:
        sys.exit("THIRD-PARTY-NOTICES.md is out of date; run `just notices`")
    print("THIRD-PARTY-NOTICES.md: up to date")


if __name__ == "__main__":
    main()
