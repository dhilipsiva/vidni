#!/usr/bin/env python3
"""Measures what an app covers of the grid clip, from a screenshot.

`vidni platforms --grid DIR` makes a picture of a grid and a clip of it.
Upload the clip privately, play it in the app, and take a screenshot. This
finds where the app drew the clip on the screen, compares the screenshot
with the grid there, and prints the edges of the clip that hold something
else: the app's buttons and words, or nothing at all where the app cut the
clip off (TODO.md, P6-VIDNI-02; DECISIONS.md, A2).

    python3 tools/zones_measure.py GRID.png SCREENSHOT.png [--mask OUT.png]

It needs Pillow and numpy, which the development shell has.
"""

import argparse
import sys

import numpy as np
from PIL import Image, ImageFilter

# How far a pixel may be from the grid's, of 255, and still be the grid's:
# the app's encoder softens the lines and the figures.
FAR = 70
# The share of a row or a column of the clip that must be something else
# before the app is said to cover it there.
SHARE = 0.02


def gray(image):
    return np.asarray(image.convert("L"), dtype=np.float32)


def placed(grid, shot):
    """Where the app drew the clip: its scale, and its top left corner on the screen."""
    width, height = shot.size
    # A strip across the middle of the screen, where no app draws anything.
    top, bottom = height * 2 // 5, height * 3 // 5
    strip = gray(shot.crop((width // 4, top, width * 3 // 4, bottom)))

    def score(scale, left, y):
        size = (round(grid.width * scale), round(grid.height * scale))
        if top - y < 0 or bottom - y > size[1] or width // 4 - left < 0 or width * 3 // 4 - left > size[0]:
            return None
        scaled = scales.get(scale)
        if scaled is None:
            scaled = scales[scale] = gray(grid.resize(size, Image.LANCZOS))
        part = scaled[top - y : bottom - y, width // 4 - left : width * 3 // 4 - left]
        return float(np.abs(part - strip).mean())

    scales, best = {}, None
    # First roughly, with the clip centred across the screen; then closely.
    rough = [(n / 1000, None, y) for n in range(900, 1301, 2) for y in range(-200, height, 2)]
    for scale, left, y in rough:
        left = (width - round(grid.width * scale)) // 2
        found = score(scale, left, y)
        if found is not None and (best is None or found < best[0]):
            best = (found, scale, left, y)
    if best is None:
        return None
    _, scale, left, y = best
    for n in range(-8, 9):
        close = round(scale + n / 2000, 4)
        for dx in range(-6, 7):
            for dy in range(-6, 7):
                found = score(close, left + dx, y + dy)
                if found is not None and found < best[0]:
                    best = (found, close, left + dx, y + dy)
        scales.pop(close, None)
    return best


def bands(covered, share):
    """The runs of rows of `covered` (rows by columns) of which `share` is covered."""
    rows = covered.mean(axis=1) >= share
    runs, start = [], None
    for i, on in enumerate(rows):
        if on and start is None:
            start = i
        if not on and start is not None:
            runs.append((start, i))
            start = None
    if start is not None:
        runs.append((start, len(rows)))
    return runs


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("grid")
    parser.add_argument("screenshot")
    parser.add_argument("--mask", help="write the covered pixels of the clip here, as a picture")
    args = parser.parse_args()
    grid, shot = Image.open(args.grid), Image.open(args.screenshot)
    found = placed(grid, shot)
    if found is None:
        sys.exit("the grid isn't on this screenshot")
    score, scale, left, top = found
    print(f"screen {shot.width}x{shot.height}; the clip is drawn at {scale:.3f} to 1, "
          f"its corner at ({left}, {top}); the middle differs by {score:.1f} of 255")

    # The screen, as the clip's own pixels see it.
    size = (round(grid.width * scale), round(grid.height * scale))
    canvas = Image.new("RGB", size, (255, 0, 255))
    canvas.paste(shot, (-left, -top))
    seen = canvas.resize(grid.size, Image.LANCZOS)
    # What the screen doesn't hold of the clip is cut off.
    inside = Image.new("L", size, 0)
    inside.paste(255, (max(0, left) - left, max(0, top) - top,
                       min(shot.width, left + size[0]) - left, min(shot.height, top + size[1]) - top))
    cut = np.asarray(inside.resize(grid.size, Image.NEAREST)) == 0
    soft = lambda image: gray(image.filter(ImageFilter.GaussianBlur(2)))
    covered = (np.abs(soft(seen) - soft(grid)) > FAR) & ~cut
    if args.mask:
        Image.fromarray((covered * 255).astype(np.uint8)).save(args.mask)

    w, h = grid.size
    print(f"cut off: {int(cut[:, : w // 2].all(axis=0).sum())} px left, "
          f"{int(cut[:, w // 2 :].all(axis=0).sum())} right, "
          f"{int(cut[: h // 2].all(axis=1).sum())} top, {int(cut[h // 2 :].all(axis=1).sum())} bottom")
    print("rows the app draws on (y from, to; the columns it draws on there):")
    for a, b in bands(covered, SHARE):
        columns = np.flatnonzero(covered[a:b].any(axis=0))
        print(f"  y {a:4}..{b:4}  x {columns.min():4}..{columns.max() + 1:4}")
    print("columns the app draws on (x from, to; the rows it draws on there):")
    for a, b in bands(covered.T, SHARE):
        rows = np.flatnonzero(covered[:, a:b].any(axis=1))
        print(f"  x {a:4}..{b:4}  y {rows.min():4}..{rows.max() + 1:4}")


if __name__ == "__main__":
    main()
