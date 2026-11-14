#!/usr/bin/env python3
"""Measure a draft voice (TODO.md, P9-VIDNI-04; docs/voices.md): whether it
says the same thing the same way each time, and whether a listener hears
the words it was given.

It speaks a fixed set of sentences twice, in two separate processes, with
one seed; hashes each take's samples; and has whisper.cpp write down what
it hears of the first take. Nothing here judges how natural a voice sounds.

  voices_eval.py piper <model.onnx> <model.onnx.json> <out>
  voices_eval.py kokoro <folder with config.json, kokoro-v1_0.pth, voices/> <voice> <out>
  voices_eval.py speak-kokoro <folder> <voice> <out> <take>    (run by `kokoro`, in its own process)
  voices_eval.py score <out>

`piper` and `whisper-cli` and `ffmpeg` are the dev shell's; `kokoro` needs
Python with the `kokoro` package (see docs/voices.md for the command).
"""
import hashlib
import json
import os
import re
import subprocess
import sys
import wave

SEED = 1292372942

# What a tutorial says: plain sentences, numbers, names, and the words a
# program is talked about with.
SENTENCES = [
    "Hello, and welcome.",
    "This video shows how a proof is checked, step by step.",
    "Every dog is an animal, every animal eats, and Adam is a dog.",
    "Run it again tomorrow and you get the same answer.",
    "The first cut is five minutes long, and the second is under sixty seconds.",
    "Press the space bar to record, and press it again to stop.",
    "A language model predicts the next word. A reasoner derives the next step.",
    "The file has three sections, twelve lines and two cuts.",
    "Nothing is uploaded unless you ask for it.",
    "In nineteen ninety-five, the answer took four hours; today it takes forty milliseconds.",
    "Open the terminal, type the command, and read what it prints.",
    "If the answer is unknown, the program says so, and says why.",
    "Version two point one fixed thirty-seven bugs.",
    "The camera moves to the part of the picture that the line is about.",
    "That is all. Thank you for watching.",
]


# whisper.cpp writes a number in figures as often as in words. Both are
# the number that was said, so the numbers in these sentences are read as
# figures on both sides before the words are compared.
FIGURES = [
    ("nineteen ninety five", "1995"), ("thirty seven", "37"), ("two point one", "2point1"),
    ("twelve", "12"), ("sixty", "60"), ("forty", "40"), ("five", "5"), ("four", "4"),
    ("three", "3"), ("two", "2"),
]


def pcm_sha(path):
    with wave.open(path, "rb") as w:
        frames = w.readframes(w.getnframes())
        return hashlib.sha256(frames).hexdigest(), w.getnframes() / w.getframerate(), w.getframerate()


def speak_piper(model, config, out, take):
    os.makedirs(os.path.join(out, take), exist_ok=True)
    for n, text in enumerate(SENTENCES):
        wav = os.path.join(out, take, f"{n:02}.wav")
        subprocess.run(
            ["piper", "--model", model, "--config", config, "--seed", str(SEED),
             "--length-scale", "1", "--output-file", wav],
            input=text.encode(), check=True, capture_output=True)


def speak_kokoro(folder, voice, out, take):
    os.environ["HF_HUB_OFFLINE"] = "1"  # nothing is fetched while it speaks
    import numpy
    import torch
    from kokoro import KModel, KPipeline
    torch.manual_seed(SEED)
    torch.use_deterministic_algorithms(True)
    model = KModel(config=os.path.join(folder, "config.json"),
                   model=os.path.join(folder, "kokoro-v1_0.pth")).eval()
    pipeline = KPipeline(lang_code="a", model=model, repo_id="hexgrad/Kokoro-82M")
    pack = torch.load(os.path.join(folder, "voices", f"{voice}.pt"), weights_only=True)
    os.makedirs(os.path.join(out, take), exist_ok=True)
    for n, text in enumerate(SENTENCES):
        parts = [result.audio.numpy() for result in pipeline(text, voice=pack)]
        samples = numpy.clip(numpy.concatenate(parts), -1, 1)
        pcm = (samples * 32767).astype("<i2").tobytes()
        with wave.open(os.path.join(out, take, f"{n:02}.wav"), "wb") as w:
            w.setnchannels(1)
            w.setsampwidth(2)
            w.setframerate(24000)
            w.writeframes(pcm)


def words(text):
    # A point between figures is part of the number.
    text = re.sub(r"(?<=\d)\.(?=\d)", "point", text.lower().replace("-", " "))
    text = " ".join(re.sub(r"[^a-z0-9 ]", " ", text).split())
    for said, figures in FIGURES:
        text = re.sub(rf"\b{said}\b", figures, text)
    return text.split()


def distance(a, b):
    row = list(range(len(b) + 1))
    for i, x in enumerate(a, 1):
        last, row[0] = row[0], i
        for j, y in enumerate(b, 1):
            last, row[j] = row[j], min(row[j] + 1, row[j - 1] + 1, last + (x != y))
    return row[-1]


def heard(wav, out):
    """What whisper.cpp hears in `wav`, which it wants at 16 kHz."""
    low = os.path.join(out, "heard.wav")
    subprocess.run(["ffmpeg", "-v", "error", "-y", "-i", wav, "-ar", "16000", "-ac", "1", low], check=True)
    said = subprocess.run(
        ["whisper-cli", "--model", os.environ["VIDNI_WHISPER_MODEL"], "--file", low,
         "--no-timestamps", "--threads", "1", "--language", "en"],
        check=True, capture_output=True, text=True).stdout
    return " ".join(said.split())


def score(out):
    takes = [[pcm_sha(os.path.join(out, take, f"{n:02}.wav")) for n in range(len(SENTENCES))]
             for take in ("a", "b")]
    same = sum(a[0] == b[0] for a, b in zip(*takes))
    rows, wrong, total = [], 0, 0
    for n, text in enumerate(SENTENCES):
        hears = heard(os.path.join(out, "a", f"{n:02}.wav"), out)
        want, got = words(text), words(hears)
        missed = distance(want, got)
        wrong += missed
        total += len(want)
        rows.append({"text": text, "heard": hears, "words": len(want), "wrong": missed})
    os.remove(os.path.join(out, "heard.wav"))
    result = {
        "seed": SEED,
        "sentences": len(SENTENCES),
        "same_samples_in_two_processes": same,
        "seconds": round(sum(t[1] for t in takes[0]), 2),
        "rate": takes[0][0][2],
        "words": total,
        "words_heard_wrong": wrong,
        "word_accuracy": round(1 - wrong / total, 4),
        "lines": rows,
    }
    with open(os.path.join(out, "result.json"), "w") as f:
        json.dump(result, f, indent=2, ensure_ascii=False)
        f.write("\n")
    print(f"{out}: {same} of {len(SENTENCES)} takes the same in two processes; "
          f"{total - wrong} of {total} words heard ({result['word_accuracy']:.1%}); "
          f"{result['seconds']} s at {result['rate']} Hz")
    for row in rows:
        if row["wrong"]:
            print(f"  {row['wrong']} wrong: {row['text']!r} heard as {row['heard']!r}")


def main(args):
    if args[:1] == ["piper"] and len(args) == 4:
        for take in ("a", "b"):
            subprocess.run([sys.executable, __file__, "speak-piper", args[1], args[2], args[3], take], check=True)
        score(args[3])
    elif args[:1] == ["speak-piper"] and len(args) == 5:
        speak_piper(*args[1:])
    elif args[:1] == ["kokoro"] and len(args) == 4:
        for take in ("a", "b"):
            subprocess.run([sys.executable, __file__, "speak-kokoro", args[1], args[2], args[3], take], check=True)
        score(args[3])
    elif args[:1] == ["speak-kokoro"] and len(args) == 5:
        speak_kokoro(*args[1:])
    elif args[:1] == ["score"] and len(args) == 2:
        score(args[1])
    else:
        raise SystemExit(__doc__)


if __name__ == "__main__":
    main(sys.argv[1:])
