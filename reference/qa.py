"""Check both cuts: streams, durations, loudness, captions; extract review frames.

Writes build/qa/<cut>-<NN>-<segment>.png (one frame from the middle of each
segment) and prints a summary. Exits non-zero when a check fails.
"""

import json
import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.abspath(__file__))
BUILD = os.path.join(ROOT, "build")
OUT = os.path.join(ROOT, "out")
CUTS = {"16x9": (1920, 1080), "9x16": (1080, 1920)}


def probe(path):
    raw = subprocess.run(["ffprobe", "-v", "error", "-show_streams", "-show_format", "-of", "json", path],
                         capture_output=True, text=True, check=True).stdout
    return json.loads(raw)


def loudness(path):
    err = subprocess.run(["ffmpeg", "-hide_banner", "-nostats", "-i", path, "-map", "0:a", "-af", "ebur128",
                          "-f", "null", "-"], capture_output=True, text=True).stderr
    m = re.findall(r"I:\s+(-?[\d.]+) LUFS", err)
    return float(m[-1]) if m else None


def main():
    timeline = json.load(open(os.path.join(BUILD, "timeline.json")))
    total = timeline["total"]
    segments = []
    for e in timeline["sentences"]:
        if not segments or segments[-1][0] != e["segment"]:
            segments.append([e["segment"], e["start"], e["end"]])
        segments[-1][2] = e["end"]
    srt = open(os.path.join(OUT, "nibli-syllogism.en.srt")).read()
    cues = len(re.findall(r"-->", srt))
    expected_cues = sum(len(e["cues"]) for e in timeline["sentences"])
    problems = []
    if cues != expected_cues:
        problems.append(f"SRT has {cues} cues, timeline has {expected_cues}")
    os.makedirs(os.path.join(BUILD, "qa"), exist_ok=True)
    for cut, (w, h) in CUTS.items():
        path = os.path.join(OUT, f"nibli-syllogism-{cut}.mp4")
        info = probe(path)
        v = next(s for s in info["streams"] if s["codec_type"] == "video")
        a = next((s for s in info["streams"] if s["codec_type"] == "audio"), None)
        dur = float(info["format"]["duration"])
        fps = v["r_frame_rate"]
        lufs = loudness(path)
        size_mb = os.path.getsize(path) / 1e6
        print(f"{cut}: {v['width']}x{v['height']} {v['codec_name']} {fps} fps, "
              f"audio {a['codec_name'] if a else 'MISSING'} {a['channels'] if a else ''}ch, "
              f"{dur:.2f} s ({int(dur // 60)}:{dur % 60:05.2f}), {lufs} LUFS, {size_mb:.1f} MB")
        if (v["width"], v["height"]) != (w, h):
            problems.append(f"{cut}: size {v['width']}x{v['height']}")
        if fps != "30/1":
            problems.append(f"{cut}: frame rate {fps}")
        if not a:
            problems.append(f"{cut}: no audio")
        if abs(dur - total) > 0.5:
            problems.append(f"{cut}: duration {dur:.2f} s, timeline {total:.2f} s")
        if lufs is None or abs(lufs + 16) > 1.5:
            problems.append(f"{cut}: loudness {lufs} LUFS")
        for n, (seg, start, end) in enumerate(segments):
            t = (start + end) / 2
            shot = os.path.join(BUILD, "qa", f"{cut}-{n:02d}-{seg}.png")
            subprocess.run(["ffmpeg", "-loglevel", "error", "-y", "-ss", f"{t:.2f}", "-i", path,
                            "-frames:v", "1", shot], check=True)
    print(f"captions: {cues} cues; review frames in build/qa/")
    if problems:
        sys.exit("QA failed:\n  " + "\n  ".join(problems))
    print("QA passed")


if __name__ == "__main__":
    main()
