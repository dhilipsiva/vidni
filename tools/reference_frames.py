"""The reference's own frames of its cuts, before encoding (P3-VIDNI-02).

golden/ pins the reference's MP4s, which are x264 at CRF 18. Their decoded
frames carry that compression, most of all while the camera moves, so the
parity harness compares the Rust compositor with the frames those MP4s were
encoded from instead: reference/compose.py's own. `just golden` shows they
are what golden pins, by re-deriving the MP4s byte for byte. Run in the
reference's nix shell:

    nix develop ./reference --command python3 -I -B tools/reference_frames.py \\
        --reference REFERENCE --variant VARIANT --fonts CAPTION MONO DISPLAY \\
        (--out OUT CUT:F,F,... [CUT:F,F,...] | --stream CUT)

VARIANT is "synthetic" or "voice" and CUT is "16x9" or "9x16". The fonts are
the sha256 each of compose.py's three faces must have; it finds them with
fc-match, which falls back to some other font rather than fail.

With --out, writes the frames F as OUT/nibli-syllogism[-voice]-CUT-F.png.
With --stream, writes every frame that differs from the one before it to
stdout: its index (8 bytes, little-endian) and its rgb24 bytes. Then 2^64 - 1
and the cut's frame count, the same way.
"""

import argparse
import hashlib
import json
import os
import struct
import sys


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--reference", required=True)
    parser.add_argument("--variant", required=True, choices=["synthetic", "voice"])
    parser.add_argument("--fonts", required=True, nargs=3, metavar=("CAPTION", "MONO", "DISPLAY"))
    parser.add_argument("--out")
    parser.add_argument("--stream", choices=["16x9", "9x16"])
    parser.add_argument("jobs", nargs="*", metavar="CUT:F,F,...")
    args = parser.parse_args()
    if bool(args.out) == bool(args.stream) or bool(args.out) != bool(args.jobs):
        parser.error("give --out with CUT:F,... jobs, or --stream CUT")
    # compose.py reads its fonts and its variant from the environment when it
    # is imported. The golden build found its fonts with the host's
    # fontconfig, not with a dev shell's FONTCONFIG_FILE.
    os.environ.pop("FONTCONFIG_FILE", None)
    if args.variant == "voice":
        os.environ["VARIANT"] = "voice"
    else:
        os.environ.pop("VARIANT", None)
    sys.path.insert(0, os.path.abspath(args.reference))
    import compose
    from PIL import Image

    for kind, want in zip(["caption", "mono", "display"], args.fonts):
        with open(compose.FONTS[kind], "rb") as f:
            got = hashlib.sha256(f.read()).hexdigest()
        if got != want:
            sys.exit(f"compose.py's {kind} font is {compose.FONTS[kind]} (sha256 {got}), not {want}")
    spec = json.load(open(os.path.join(compose.ROOT, "segments.json")))
    timeline = json.load(open(compose.TIMELINE))
    capture = json.load(open(os.path.join(compose.BUILD, "capture.json")))

    def cut(name):
        return compose.Cut(name, spec, timeline, capture)

    if args.stream:
        out = sys.stdout.buffer
        count, last = 0, None
        for f, data in enumerate(cut(args.stream).frames()):
            count += 1
            # A frame equal to the one before is the same bytes object.
            if data is not last:
                out.write(struct.pack("<Q", f))
                out.write(data)
                last = data
        out.write(struct.pack("<QQ", 2**64 - 1, count))
        out.flush()
        return
    os.makedirs(args.out, exist_ok=True)
    for job in args.jobs:
        name, frames = job.split(":")
        todo = {int(f) for f in frames.split(",")}
        frames = cut(name)
        stem = f"nibli-syllogism{compose.SUFFIX}-{name}"
        for f, data in enumerate(frames.frames()):
            if f in todo:
                Image.frombytes("RGB", (frames.W, frames.H), data).save(os.path.join(args.out, f"{stem}-{f}.png"))
                todo.discard(f)
                if not todo:
                    break
        if todo:
            sys.exit(f"{stem} has no frame {min(todo)}")


if __name__ == "__main__":
    main()
