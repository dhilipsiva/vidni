"""Fixtures for vidni-media's Pillow-exact image operations (P3-VIDNI-02).

The compositor reproduces the reference's frames, which Pillow drew. This
script asks Pillow itself for the expected results of each operation the
reference uses, so the Rust port can be tested against them byte for byte.
Run it in the reference's nix shell, from vidni's root:

    nix develop ./reference --command python3 -I tools/pillow_fixtures.py
"""

import hashlib
import json
import os

from PIL import Image, ImageDraw, ImageFont, features

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "crates", "vidni-media", "tests", "fixtures", "pillow")
FONT = os.path.join(ROOT, "crates", "vidni-media", "tests", "fixtures", "fonts", "IBMPlexSans-Bold.otf")


def pattern(w, h, seed):
    img = Image.new("RGB", (w, h))
    img.putdata([((x * 7 + y * 3 + seed) % 256, (x * x + y * 5 + seed * 3) % 256, ((x ^ y) * 11 + seed) % 256)
                 for y in range(h) for x in range(w)])
    return img


def save(img, name):
    img.save(os.path.join(OUT, name))


def main():
    os.makedirs(OUT, exist_ok=True)
    src = pattern(97, 61, 0)
    save(src, "src.png")
    other = pattern(97, 61, 101)
    save(other, "other.png")
    cases = {"resize": [], "blend": []}
    # Crops and resizes as compose.py's view() does: BICUBIC with a float box.
    for i, (size, box) in enumerate([((40, 25), (10.5, 7.25, 80.75, 50.5)),
                                     ((64, 40), (3.3, 2.2, 23.9, 15.1)),
                                     ((50, 31), (0.0, 0.0, 97.0, 61.0)),
                                     ((97, 61), (0.1, 0.2, 96.7, 60.9)),
                                     ((33, 80), (12.4, 0.0, 45.6, 61.0))]):
        save(src.resize(size, Image.BICUBIC, box=box), f"resize-{i}.png")
        cases["resize"].append({"size": size, "box": box, "file": f"resize-{i}.png"})
    # Crossfades as Image.blend makes them.
    for i, alpha in enumerate([0.0, 0.1, 1 / 3, 0.5, 0.77, 0.999, 1.0]):
        save(Image.blend(src, other, alpha), f"blend-{i}.png")
        cases["blend"].append({"alpha": alpha, "file": f"blend-{i}.png"})
    # An RGBA piece pasted with its own alpha, as decorate() does, one in and one partly out of the frame.
    piece = Image.new("RGBA", (30, 20))
    piece.putdata([((x * 9) % 256, (y * 13) % 256, 200, (x * 8 + y * 5) % 256) for y in range(20) for x in range(30)])
    save(piece, "piece.png")
    for i, pos in enumerate([(5, 7), (80, 50)]):
        frame = src.copy()
        frame.paste(piece, pos, piece)
        save(frame, f"paste-{i}.png")
    cases["paste"] = [{"pos": [5, 7], "file": "paste-0.png"}, {"pos": [80, 50], "file": "paste-1.png"}]
    # The 16:9 caption box and section label: rounded rectangles on a transparent piece.
    for i, (size, radius, fill) in enumerate([((120, 54), 10, (7, 7, 10, 210)), ((200, 90), 14, (7, 7, 10, 222))]):
        img = Image.new("RGBA", size, (0, 0, 0, 0))
        ImageDraw.Draw(img).rounded_rectangle((0, 0, size[0] - 1, size[1] - 1), radius=radius, fill=fill,
                                              outline=(58, 58, 74, 255))
        save(img, f"rounded-{i}.png")
    cases["rounded"] = [{"size": [120, 54], "radius": 10, "fill": [7, 7, 10, 210], "file": "rounded-0.png"},
                        {"size": [200, 90], "radius": 14, "fill": [7, 7, 10, 222], "file": "rounded-1.png"}]
    # The 9:16 view's edge lines (width 2) and the overlay band.
    frame = Image.new("RGB", (60, 40), (11, 11, 16))
    d = ImageDraw.Draw(frame)
    d.line((0, 9, 60, 9), fill=(42, 42, 56), width=2)
    d.line((0, 30, 60, 30), fill=(42, 42, 56), width=2)
    d.rectangle((0, 0, 60, 4), fill=(200, 100, 50))
    save(frame, "lines.png")
    # Text coverage drawn into RGB and RGBA pieces, as ImageDraw.text draws its mask.
    mask = Image.frombytes("L", (6, 1), bytes([0, 1, 64, 128, 254, 255]))
    bitmap = {}
    for alpha in [0, 1, 50, 210, 222, 255, "rgb"]:
        if alpha == "rgb":
            img = Image.new("RGB", (6, 1), (7, 7, 10))
        else:
            img = Image.new("RGBA", (6, 1), (7, 7, 10, alpha))
        ImageDraw.Draw(img).bitmap((0, 0), mask, fill=(244, 243, 239))
        bitmap[str(alpha)] = [list(img.getpixel((x, 0))) for x in range(6)]
    with open(os.path.join(OUT, "bitmap.json"), "w") as f:
        json.dump(bitmap, f)
    cases["text"] = text()
    with open(os.path.join(OUT, "cases.json"), "w") as f:
        json.dump(cases, f, indent=1)
    print(f"wrote fixtures to {OUT}")


def text():
    """Every caption cue of the example, measured at both caption sizes, and
    two lines drawn: one with kerning across a hyphen, one starting at a
    fraction of a pixel."""
    assert features.check("raqm"), "Pillow lays text out with libraqm in the reference"
    with open(FONT, "rb") as f:
        sha256 = hashlib.sha256(f.read()).hexdigest()
    cues = set()
    for variant in ("synthetic", "voice"):
        with open(os.path.join(ROOT, "golden", f"timeline-{variant}.json")) as f:
            for sentence in json.load(f)["sentences"]:
                cues.update(cue["text"] for cue in sentence["cues"])
    lengths = []
    for size in (42, 50):
        font = ImageFont.truetype(FONT, size)
        for cue in sorted(cues):
            lengths.append({"size": size, "text": cue, "length26": round(font.getlength(cue) * 64)})
    drawn = []
    for i, (size, line, xy, anchor) in enumerate([
            (42, "A neuro-symbolic system pairs a neural front end", (600.5, 60.0), "mm"),
            (50, "Half-pixel start: y is 80.5", (100.25, 80.5), "ls")]):
        img = Image.new("RGB", (1200, 120), (11, 11, 16))
        ImageDraw.Draw(img).text(xy, line, font=ImageFont.truetype(FONT, size), fill=(244, 243, 239),
                                 anchor=anchor)
        save(img, f"text-{i}.png")
        drawn.append({"size": size, "text": line, "xy": xy, "anchor": anchor, "file": f"text-{i}.png"})
    return {"font": "IBMPlexSans-Bold.otf", "sha256": sha256, "lengths": lengths, "drawn": drawn}


if __name__ == "__main__":
    main()
