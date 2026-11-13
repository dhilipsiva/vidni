"""Golden outputs of the reference pipeline (TODO.md, P1-VIDNI-01).

Run inside the reference's nix shell, as `just golden` and `just golden-update` do:

    python3 -I tools/golden.py update <reference-dir>   re-derive and write golden/
    python3 -I tools/golden.py verify <reference-dir>   re-derive and compare with golden/

Re-deriving runs the reference's deterministic stages from its frozen inputs,
once for the synthetic variant and once for the creator's voice ("voice"):

1. tts.py builds the timeline, the narration and the captions. Synthetic audio
   comes from the cached Piper takes in build/audio; voice audio comes from
   the creator's takes in recordings/.
2. compose.py renders both cuts (16x9 and 9x16).

Some things are frozen inputs rather than re-made outputs: the live page's
captures and Chromium's renders (stills, cards), the synthetic takes, the
creator's takes and the script. They are hashed, never regenerated.
Whisper's `heard` and `score` fields are dropped from the timeline, because
whisper never feeds an output (TODO.md, rule 2).
"""

import gzip
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
import wave

from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
GOLDEN = os.path.join(os.path.dirname(HERE), "golden")
VARIANTS = {"synthetic": "", "voice": "voice"}
CUTS = ("16x9", "9x16")


def sha256_bytes(data):
    return hashlib.sha256(data).hexdigest()


def sha256_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def pcm_sha256(path):
    """Hash of the decoded samples, independent of the WAV header layout."""
    with wave.open(path) as w:
        head = f"{w.getnchannels()}ch {w.getsampwidth() * 8}bit {w.getframerate()}Hz|".encode()
        return sha256_bytes(head + w.readframes(w.getnframes()))


def pixels_sha256(path):
    """Hash of the decoded RGBA pixels, independent of the PNG encoder."""
    with Image.open(path) as im:
        rgba = im.convert("RGBA")
        return sha256_bytes(f"{rgba.width}x{rgba.height}|".encode() + rgba.tobytes())


def normalized_json(path, drop=()):
    with open(path) as f:
        data = json.load(f)
    for key in drop:
        data.pop(key, None)
    return data


def normalized_timeline(path, ref):
    data = normalized_json(path)
    for e in data["sentences"]:
        e["wav"] = os.path.relpath(e["wav"], ref)
        e.pop("heard", None)
        e.pop("score", None)
    return data


def canonical(data):
    return json.dumps(data, indent=1, sort_keys=True, ensure_ascii=False) + "\n"


def run(cmd, ref, env):
    print("  $ " + " ".join(cmd), flush=True)
    subprocess.run(cmd, cwd=ref, env=env, check=True, stdout=subprocess.DEVNULL)


def framemd5(path):
    out = subprocess.run(["ffmpeg", "-loglevel", "error", "-i", path, "-map", "0", "-c", "copy",
                          "-f", "framemd5", "-"], capture_output=True, check=True).stdout
    return out


def derive(ref, stage):
    """Re-derive every golden artifact into `stage`; return the manifest."""
    ref = os.path.abspath(ref)
    base_env = {k: v for k, v in os.environ.items()
                if k not in ("VARIANT", "OWN_AUDIO_DIR", "LD_LIBRARY_PATH")}
    manifest = {"inputs": {}, "outputs": {}, "commands": {}}

    for variant, tag in VARIANTS.items():
        env = dict(base_env)
        suffix = f"-{tag}" if tag else ""
        if tag:
            env["VARIANT"] = tag
            env["OWN_AUDIO_DIR"] = os.path.join(ref, "recordings")
        how = f"VARIANT={tag} OWN_AUDIO_DIR=recordings " if tag else ""
        print(f"[{variant}] narration, timeline and captions", flush=True)
        run(["python3", "-I", "tts.py"], ref, env)
        timeline = canonical(normalized_timeline(os.path.join(ref, "build", "timeline.json"), ref))
        with open(os.path.join(stage, f"timeline-{variant}.json"), "w") as f:
            f.write(timeline)
        srt = f"nibli-syllogism{suffix}.en.srt"
        shutil.copy(os.path.join(ref, "out", srt), os.path.join(stage, srt))
        manifest["outputs"][f"timeline-{variant}.json"] = sha256_bytes(timeline.encode())
        manifest["outputs"][srt] = sha256_file(os.path.join(stage, srt))
        manifest["outputs"][f"narration-{variant}.wav (pcm)"] = pcm_sha256(
            os.path.join(ref, "build", "narration.wav"))
        manifest["commands"][f"{variant}: timeline, narration, captions"] = f"{how}python3 -I tts.py"

        print(f"[{variant}] both cuts", flush=True)
        run(["python3", "-I", "compose.py"], ref, env)
        manifest["commands"][f"{variant}: cuts"] = f"{how}python3 -I compose.py"
        for cut in CUTS:
            name = f"nibli-syllogism{suffix}-{cut}.mp4"
            fmd5 = framemd5(os.path.join(ref, "out", name))
            with gzip.GzipFile(os.path.join(stage, "framemd5", name + ".framemd5.gz"), "wb", mtime=0) as f:
                f.write(fmd5)
            manifest["outputs"][f"{name} (framemd5)"] = sha256_bytes(fmd5)
            manifest["outputs"][f"{name} (file)"] = sha256_file(os.path.join(ref, "out", name))
        manifest["commands"]["framemd5"] = "ffmpeg -i <cut>.mp4 -map 0 -c copy -f framemd5 -"

    # Leave the reference in its synthetic state (its timeline.json is shared until P1-VIDNI-02).
    run(["python3", "-I", "tts.py"], ref, base_env)

    print("[frozen inputs] hashing", flush=True)
    ins = manifest["inputs"]
    ins["segments.json"] = sha256_file(os.path.join(ref, "segments.json"))
    for folder, kind in (("build/audio", "wav"), ("recordings", "wav"), ("build/stills", "png"),
                         ("build/cards", "png")):
        names = sorted(n for n in os.listdir(os.path.join(ref, folder)) if n.endswith("." + kind))
        for n in names:
            p = os.path.join(ref, folder, n)
            ins[f"{folder}/{n}"] = {"file": sha256_file(p),
                                     "decoded": pcm_sha256(p) if kind == "wav" else pixels_sha256(p)}
    ins["build/capture.json"] = sha256_bytes(canonical(
        normalized_json(os.path.join(ref, "build", "capture.json"))).encode())
    ins["build/onscreen.json (without checkedAt)"] = sha256_bytes(canonical(
        normalized_json(os.path.join(ref, "build", "onscreen.json"), drop=("checkedAt",))).encode())
    return manifest


def compare(old, new, path=""):
    diffs = []
    keys = sorted(set(old) | set(new)) if isinstance(old, dict) and isinstance(new, dict) else None
    if keys is None:
        if old != new:
            diffs.append(f"{path}: golden {str(old)[:20]}… now {str(new)[:20]}…")
        return diffs
    for k in keys:
        if k not in old:
            diffs.append(f"{path}/{k}: new, not in golden")
        elif k not in new:
            diffs.append(f"{path}/{k}: missing now")
        else:
            diffs += compare(old[k], new[k], f"{path}/{k}")
    return diffs


def main():
    if len(sys.argv) != 3 or sys.argv[1] not in ("update", "verify"):
        sys.exit(__doc__)
    mode, ref = sys.argv[1], sys.argv[2]
    started = time.time()
    with tempfile.TemporaryDirectory() as stage:
        os.makedirs(os.path.join(stage, "framemd5"))
        manifest = derive(ref, stage)
        manifest_text = canonical({k: manifest[k] for k in ("inputs", "outputs")})
        if mode == "update":
            if os.path.exists(GOLDEN):
                for n in ("framemd5",):
                    shutil.rmtree(os.path.join(GOLDEN, n), ignore_errors=True)
            os.makedirs(GOLDEN, exist_ok=True)
            for n in os.listdir(stage):
                src = os.path.join(stage, n)
                (shutil.copytree if os.path.isdir(src) else shutil.copy)(src, os.path.join(GOLDEN, n))
            with open(os.path.join(GOLDEN, "hashes.json"), "w") as f:
                f.write(manifest_text)
            with open(os.path.join(GOLDEN, "commands.json"), "w") as f:
                f.write(canonical(manifest["commands"]))
            print(f"golden written: {len(manifest['inputs'])} inputs, {len(manifest['outputs'])} outputs "
                  f"({time.time() - started:.0f} s)")
            return
        with open(os.path.join(GOLDEN, "hashes.json")) as f:
            golden = json.load(f)
        diffs = compare(golden, json.loads(manifest_text))
        if diffs:
            print(f"golden MISMATCH ({len(diffs)}):\n  " + "\n  ".join(diffs[:40]))
            sys.exit(1)
        print(f"golden verified: {len(golden['inputs'])} inputs and {len(golden['outputs'])} outputs match "
              f"({time.time() - started:.0f} s)")


if __name__ == "__main__":
    main()
