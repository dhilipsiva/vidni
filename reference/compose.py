"""Render the 16:9 and 9:16 cuts from the stills, cards and narration timeline.

Reads segments.json, build/timeline<-VARIANT>.json, build/capture.json,
build/stills, build/cards and build/narration<-VARIANT>.wav. Writes
out/nibli-syllogism<-VARIANT>-16x9.mp4 and -9x16.mp4. Pass "16x9" or "9x16" to
render one cut, --remux to replace only the audio of the rendered cuts, or
--plan to print which files a build would read and write, without rendering.
"""

import json
import math
import os
import subprocess
import sys
from functools import lru_cache

from PIL import Image, ImageDraw, ImageFont

ROOT = os.path.dirname(os.path.abspath(__file__))
BUILD = os.path.join(ROOT, "build")
OUT = os.path.join(ROOT, "out")
SUFFIX = f"-{os.environ['VARIANT']}" if os.environ.get("VARIANT") else ""
TIMELINE = os.path.join(BUILD, f"timeline{SUFFIX}.json")
NARRATION = os.path.join(BUILD, f"narration{SUFFIX}.wav")
NARRATION_NORM = os.path.join(BUILD, f"narration{SUFFIX}-norm.wav")
FPS = 30
CAM_EASE = 0.7      # seconds for a camera move
XFADE = 0.35        # seconds for a crossfade between stills or cards

VOID = (11, 11, 16)
LINE = (42, 42, 56)
INK = (244, 243, 239)
MUTED = (144, 143, 154)
EMBER = (242, 84, 45)

# Each cut: output size, where the screen view sits, the smallest crop width
# (in CSS px; 3x stills keep crops at or above 1:1), and the share of the
# view kept clear for captions.
CUTS = {
    "16x9": {"size": (1920, 1080), "view": (0, 0, 1920, 1080), "min_w": 640, "reserve": 0.2},
    "9x16": {"size": (1080, 1920), "view": (0, 330, 1080, 1170), "min_w": 400, "reserve": 0.0},
}
WIDE = {"app", "full", "top"}
OVERLAY_Y = 100     # 16:9 overlays sit below the section label


def font_path(pattern):
    path = subprocess.run(["fc-match", "-f", "%{file}", pattern], capture_output=True, text=True).stdout.strip()
    if not path or not os.path.exists(path):
        sys.exit(f"font not found: {pattern}")
    return path


FONTS = {
    "caption": font_path("IBM Plex Sans:weight=500"),
    "mono": font_path("IBM Plex Mono:weight=400"),
    "display": font_path("Space Grotesk:weight=500"),
}


@lru_cache(maxsize=None)
def font(kind, size):
    return ImageFont.truetype(FONTS[kind], size)


def ease(x):
    x = min(max(x, 0.0), 1.0)
    return x * x * (3 - 2 * x)


def wrap(text, fnt, width):
    lines, cur = [], ""
    for word in text.split():
        cand = f"{cur} {word}".strip()
        if cur and fnt.getlength(cand) > width:
            lines.append(cur)
            cur = word
        else:
            cur = cand
    if cur:
        lines.append(cur)
    return lines


def normalized_audio():
    """build/narration<-VARIANT>-norm.wav: two-pass loudnorm to -16 LUFS. One pass undershoots on
    speech with pauses (it measured -18.1 LUFS on this narration)."""
    src, dst = NARRATION, NARRATION_NORM
    if os.path.exists(dst) and os.path.getmtime(dst) >= os.path.getmtime(src):
        return dst
    target = "I=-16:TP=-1.5:LRA=11"
    err = subprocess.run(["ffmpeg", "-hide_banner", "-nostats", "-i", src, "-af",
                          f"loudnorm={target}:print_format=json", "-f", "null", "-"],
                         capture_output=True, text=True).stderr
    m = json.loads(err[err.rindex("{"):err.rindex("}") + 1])
    af = (f"loudnorm={target}:linear=true:measured_I={m['input_i']}:measured_TP={m['input_tp']}:"
          f"measured_LRA={m['input_lra']}:measured_thresh={m['input_thresh']}:offset={m['target_offset']},"
          "aresample=48000")
    subprocess.run(["ffmpeg", "-loglevel", "error", "-y", "-i", src, "-af", af, "-ac", "2", dst], check=True)
    return dst


def remux(name):
    """Swap in the normalized narration without re-encoding the video."""
    out = os.path.join(OUT, f"nibli-syllogism{SUFFIX}-{name}.mp4")
    tmp = out + ".remux.mp4"
    subprocess.run(["ffmpeg", "-loglevel", "error", "-y", "-i", out, "-i", normalized_audio(),
                    "-map", "0:v", "-map", "1:a", "-c:v", "copy", "-c:a", "aac", "-b:a", "160k",
                    "-movflags", "+faststart", "-shortest", tmp], check=True)
    os.replace(tmp, out)
    print(f"{out}: audio replaced")


def crop_rect(target, aspect, cut, view_wh, top_reserve=0.0):
    """A crop (CSS px) with the cut's aspect that frames `target` between the
    top reserve (an overlay) and the bottom reserve (captions)."""
    vw, vh = view_wh
    pad = 16
    x, y, w, h = target[0] - pad, target[1] - pad, target[2] + 2 * pad, target[3] + 2 * pad
    r = cut["reserve"]
    free = 1 - r - top_reserve
    H = max(h / free, w / aspect)
    W = H * aspect
    if W < cut["min_w"]:
        W, H = cut["min_w"], cut["min_w"] / aspect
    if W > vw:
        W, H = vw, vw / aspect
    if H > vh:
        H, W = vh, vh * aspect
    cx = x + w / 2
    top = y - top_reserve * H - max(0.0, (H * free - h) / 2)
    X = min(max(cx - W / 2, 0), vw - W)
    Y = min(max(top, 0), vh - H)
    return (X, Y, W, H)


class Cut:
    def __init__(self, name, spec, timeline, capture):
        self.name = name
        self.cut = CUTS[name]
        self.W, self.H = self.cut["size"]
        vx, vy, vw, vh = self.cut["view"]
        self.view_box = (vx, vy)
        self.view_size = (vw, vh)
        self.aspect = vw / vh
        self.dpr = capture["dpr"]
        self.css = (capture["view"]["width"], capture["view"]["height"])
        self.images = {}
        self.label_cache = {}
        self.sentences = timeline["sentences"]
        self.total = timeline["total"]
        shots = {s["k"]: s for s in capture["shots"]}
        seg_spec = {s["id"]: s for s in spec["segments"]}
        flat = [s for seg in spec["segments"] for s in seg["sentences"]]
        self.items = []
        for e in self.sentences:
            shot, src = shots[e["k"]], flat[e["k"]]
            item = {"k": e["k"], "label": seg_spec[e["segment"]]["label"], "card": shot.get("card"),
                    "overlay": shot.get("overlay"), "still": shot.get("still"), "cue": e["cues"]}
            if not item["card"]:
                key = self.camera_key(src, shot)
                top = 0.0
                if item["overlay"] and name == "16x9":
                    top = (OVERLAY_Y + self.overlay(item["overlay"]).height + 20) / self.H
                item["rect"] = crop_rect(shot["cameras"][key], self.aspect, self.cut, self.css, top)
            self.items.append(item)
        # A page sentence after a card starts from its own framing.
        for i, it in enumerate(self.items):
            if "rect" not in it:
                continue
            prev = next((p for p in reversed(self.items[:i]) if "rect" in p), None)
            it["from_rect"] = prev["rect"] if prev else it["rect"]
        # Visual changes land halfway through the pause before each sentence.
        self.switch = [0.0]
        for a, b in zip(self.sentences, self.sentences[1:]):
            self.switch.append((a["end"] + b["start"]) / 2)
        self.cues = [c for e in self.sentences for c in e["cues"]]
        self.cue_overlays = [self.render_cue(c["text"]) for c in self.cues]

    def camera_key(self, src, shot):
        if self.name == "16x9":
            return src.get("cam", "app")
        if src.get("vcam"):
            return src["vcam"]
        hl = shot["cameras"].get("hl")
        if hl and hl[2] <= 760:
            return "hl"
        return "proof" if src.get("cam", "app") in WIDE else src.get("cam", "app")

    # --- pre-rendered pieces ---------------------------------------------
    def render_cue(self, text):
        if self.name == "16x9":
            fnt = font("caption", 42)
            lines = wrap(text, fnt, 1560)
            lh = 56
            tw = max(fnt.getlength(l) for l in lines)
            bw, bh = int(tw + 60), int(len(lines) * lh + 36)
            img = Image.new("RGBA", (bw, bh), (0, 0, 0, 0))
            d = ImageDraw.Draw(img)
            d.rounded_rectangle((0, 0, bw - 1, bh - 1), radius=14, fill=(7, 7, 10, 222), outline=(58, 58, 74, 255))
            for i, line in enumerate(lines):
                d.text((bw / 2, 18 + i * lh + lh / 2), line, font=fnt, fill=INK, anchor="mm")
            return img, ((self.W - bw) // 2, self.H - bh - 44)
        fnt = font("caption", 50)
        lines = wrap(text, fnt, 960)
        lh = 66
        img = Image.new("RGBA", (self.W, 420), (0, 0, 0, 0))
        d = ImageDraw.Draw(img)
        y0 = (420 - len(lines) * lh) / 2
        for i, line in enumerate(lines):
            d.text((self.W / 2, y0 + i * lh + lh / 2), line, font=fnt, fill=INK, anchor="mm")
        return img, (0, 1500)

    def label_piece(self, label):
        if label in self.label_cache:
            return self.label_cache[label]
        if self.name == "16x9":
            fnt = font("mono", 26)
            w = int(fnt.getlength("// " + label) + 40)
            img = Image.new("RGBA", (w, 54), (0, 0, 0, 0))
            d = ImageDraw.Draw(img)
            d.rounded_rectangle((0, 0, w - 1, 53), radius=10, fill=(7, 7, 10, 210), outline=(58, 58, 74, 255))
            d.text((20, 27), "//", font=fnt, fill=EMBER, anchor="lm")
            d.text((20 + fnt.getlength("// "), 27), label, font=fnt, fill=INK, anchor="lm")
            piece = (img, (32, 30))
        else:
            img = Image.new("RGBA", (self.W, 330), VOID + (255,))
            d = ImageDraw.Draw(img)
            mono = font("mono", 30)
            d.text((80, 118), "//", font=mono, fill=EMBER, anchor="ls")
            d.text((80 + mono.getlength("// "), 118), "nibli · the syllogism walkthrough", font=mono, fill=MUTED, anchor="ls")
            title = font("display", 74)
            lines = wrap(label[:1].upper() + label[1:], title, 920)
            for i, line in enumerate(lines[:2]):
                d.text((80, 212 + i * 80), line, font=title, fill=INK, anchor="ls")
            piece = (img, (0, 0))
        self.label_cache[label] = piece
        return piece

    def card(self, name):
        key = ("card", name)
        if key not in self.images:
            self.images[key] = Image.open(os.path.join(BUILD, "cards", f"{name}-{self.name}.png")).convert("RGB")
        return self.images[key]

    def overlay(self, name):
        key = ("overlay", name)
        if key not in self.images:
            self.images[key] = Image.open(os.path.join(BUILD, "cards", f"{name}-{self.name}.png")).convert("RGBA")
        return self.images[key]

    def still(self, path):
        key = ("still", path)
        if key not in self.images:
            stills = [k for k in self.images if k[0] == "still"]
            for old in stills[:-3]:
                del self.images[old]
            self.images[key] = Image.open(os.path.join(BUILD, path)).convert("RGB")
        return self.images[key]

    # --- frames ------------------------------------------------------------
    def view(self, item, rect):
        x, y, w, h = rect
        d = self.dpr
        return self.still(item["still"]).resize(self.view_size, Image.BICUBIC, box=(x * d, y * d, (x + w) * d, (y + h) * d))

    def visual(self, item, rect):
        """The item's picture over the full frame (screen view or card), without captions."""
        if item["card"]:
            return self.card(item["card"]).copy()
        v = self.view(item, rect)
        if self.name == "16x9":
            return v
        frame = Image.new("RGB", (self.W, self.H), VOID)
        frame.paste(v, self.view_box)
        d = ImageDraw.Draw(frame)
        vx, vy = self.view_box
        vw, vh = self.view_size
        d.line((0, vy - 1, self.W, vy - 1), fill=LINE, width=2)
        d.line((0, vy + vh, self.W, vy + vh), fill=LINE, width=2)
        return frame

    def decorate(self, frame, item, cue):
        if not item["card"] and (item["label"] or self.name == "9x16"):
            img, pos = self.label_piece(item["label"])
            frame.paste(img, pos, img)
        if item["overlay"] and not item["card"]:
            ov = self.overlay(item["overlay"])
            pos = ((self.W - ov.width) // 2, OVERLAY_Y) if self.name == "16x9" else ((self.W - ov.width) // 2, 0)
            if self.name == "9x16":
                ImageDraw.Draw(frame).rectangle((0, 0, self.W, 329), fill=VOID)
            frame.paste(ov, pos, ov)
        if cue is not None:
            img, pos = self.cue_overlays[cue]
            frame.paste(img, pos, img)
        return frame

    def index_at(self, t):
        i = 0
        while i + 1 < len(self.switch) and self.switch[i + 1] <= t:
            i += 1
        return i

    def cue_at(self, t):
        best = None
        for j, c in enumerate(self.cues):
            if c["start"] - 0.12 <= t:
                best = j
            else:
                break
        if best is None:
            return None
        nxt = self.cues[best + 1]["start"] if best + 1 < len(self.cues) else math.inf
        return best if t < min(nxt, self.cues[best]["end"] + 0.9) else None

    def frames(self):
        n = math.ceil(self.total * FPS)
        last_key, last_bytes = None, None
        for f in range(n):
            t = f / FPS
            i = self.index_at(t)
            item = self.items[i]
            prev = self.items[i - 1] if i > 0 else item
            since = t - self.switch[i]
            cam_p = ease(since / CAM_EASE) if i > 0 else 1.0
            fade_p = ease(since / XFADE) if i > 0 else 1.0
            cue = self.cue_at(t)
            stable = cam_p >= 1.0 and fade_p >= 1.0
            key = ("S", i, cue) if stable else ("T", f)
            if key == last_key:
                yield last_bytes
                continue
            if item["card"]:
                rect = None
            else:
                a, b = item["from_rect"], item["rect"]
                rect = tuple(a[j] + (b[j] - a[j]) * cam_p for j in range(4))
            cur = self.visual(item, rect)
            if fade_p < 1.0 and i > 0:
                prev_rect = rect if (rect and not prev["card"]) else prev.get("rect")
                old = self.visual(prev, prev_rect)
                cur = Image.blend(old, cur, fade_p)
            frame = self.decorate(cur, item, cue)
            last_key, last_bytes = key, frame.tobytes()
            yield last_bytes

    def render(self):
        os.makedirs(OUT, exist_ok=True)
        out = os.path.join(OUT, f"nibli-syllogism{SUFFIX}-{self.name}.mp4")
        cmd = ["ffmpeg", "-loglevel", "error", "-y",
               "-f", "rawvideo", "-pix_fmt", "rgb24", "-s", f"{self.W}x{self.H}", "-r", str(FPS), "-i", "-",
               "-i", normalized_audio(),
               "-c:v", "libx264", "-preset", "slow", "-crf", "18", "-pix_fmt", "yuv420p",
               "-c:a", "aac", "-b:a", "160k",
               "-movflags", "+faststart", "-shortest", out]
        proc = subprocess.Popen(cmd, stdin=subprocess.PIPE)
        count = 0
        for data in self.frames():
            proc.stdin.write(data)
            count += 1
            if count % (FPS * 30) == 0:
                print(f"  {self.name}: {count / FPS:.0f}s of {self.total:.0f}s", flush=True)
        proc.stdin.close()
        if proc.wait():
            sys.exit(f"ffmpeg failed for {out}")
        print(f"{out}: {count} frames")


def main():
    if sys.argv[1:] == ["--plan"]:
        timeline = json.load(open(TIMELINE))
        print(json.dumps({"timeline": os.path.relpath(TIMELINE, ROOT),
                          "narration": os.path.relpath(NARRATION, ROOT),
                          "total": timeline["total"],
                          "outputs": [f"out/nibli-syllogism{SUFFIX}-{c}.mp4" for c in CUTS]}))
        return
    if sys.argv[1:] == ["--remux"]:
        for name in CUTS:
            remux(name)
        return
    spec = json.load(open(os.path.join(ROOT, "segments.json")))
    timeline = json.load(open(TIMELINE))
    capture = json.load(open(os.path.join(BUILD, "capture.json")))
    if len(capture["shots"]) != len(timeline["sentences"]):
        sys.exit("capture.json and timeline.json disagree on the sentence count; re-run both")
    for name in (sys.argv[1:] or ["16x9", "9x16"]):
        Cut(name, spec, timeline, capture).render()


if __name__ == "__main__":
    main()
