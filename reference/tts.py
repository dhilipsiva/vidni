"""Narration: lint the script, synthesize one clip per sentence, build the timeline.

Writes build/audio/NNN.wav, build/narration.wav, build/timeline.json,
script.md and out/nibli-syllogism<-VARIANT>.en.srt. Set OWN_AUDIO_DIR to a
folder of NNN.wav files (16-bit mono, one per sentence, numbered in script
order, as record.py writes them) to use your own recordings instead of Piper;
VARIANT names that build's outputs (build.sh's "own" step uses "voice").

Piper samples differently on every run, so when whisper-cli and
build/models/ggml-base.en.bin are present each sentence is synthesized up to
TAKES times and the take whisper transcribes closest to the script is kept.
Own recordings are transcribed too, to flag lines read differently.
"""

import difflib
import hashlib
import json
import math
import os
import re
import subprocess
import sys
import tempfile
import wave

ROOT = os.path.dirname(os.path.abspath(__file__))
BUILD = os.path.join(ROOT, "build")
AUDIO = os.path.join(BUILD, "audio")
OUT = os.path.join(ROOT, "out")
VARIANT = os.environ.get("VARIANT", "")
SUFFIX = f"-{VARIANT}" if VARIANT else ""

# Wording rules from canonical-facts.md (B, E, G) and the book repo's CLAUDE.md.
BANNED = [
    r"zero[- ]hallucination", r"no hallucination", r"cannot hallucinate",
    r"guarantee", r"machine[- ]checked", r"verified engine", r"high[- ]performance",
    r"\bpublished\b", r"UNDETERMINED", r"sub[- ]millisecond",
    r"LLM[^.]*\bcompil", r"\bcompil\w*[^.]*\bLLM",
]
CUE_CHARS = 92
TAKES = 4
GOOD_ENOUGH = 0.97
WHISPER_MODEL = os.path.join(BUILD, "models", "ggml-base.en.bin")
# Words whisper writes differently for the same sound.
HOMOPHONES = {"atom": "adam", "atoms": "adams", "roll": "role", "y": "why",
              "nibbly": "nibli", "nibley": "nibli", "nibali": "nibli", "site's": "cites",
              "sites": "cites"}


def lint(sentences):
    hits = []
    for k, s in enumerate(sentences):
        for text in {s["show"], s.get("say", s["show"])}:
            for pattern in BANNED:
                if re.search(pattern, text, re.IGNORECASE):
                    hits.append(f"sentence {k}: /{pattern}/ in {text!r}")
    if hits:
        sys.exit("script lint failed:\n  " + "\n  ".join(hits))


# How a person reads the symbols the captions show (record.py prints these as hints).
SPOKEN = [("dhilipsiva.dev/nibli-playground", "dhilipsiva dot dev slash nibli playground"),
          ("RESOURCE_EXCEEDED", "resource exceeded"), ("#1", "number one"), ("#0", "number zero")]


def spoken(text):
    for symbol, words in SPOKEN:
        text = text.replace(symbol, words)
    return text


def respell(text, pairs):
    for old, new in pairs:
        text = text.replace(old, new)
    return text


def cues_for(text):
    """Split a long sentence into the fewest caption cues of about CUE_CHARS
    characters, cutting at punctuation near each ideal point when there is any."""
    if len(text) <= CUE_CHARS:
        return [text]
    words = text.split(" ")
    n = math.ceil(len(text) / CUE_CHARS)
    ends, pos = [], 0
    for i, w in enumerate(words[:-1]):
        pos += len(w) + 1
        ends.append((pos, i, w))
    bonus = {".": 30, "?": 30, "!": 30, ":": 26, ";": 26, ",": 24}
    cuts = []
    for j in range(1, n):
        ideal = len(text) * j / n
        start = cuts[-1][0] if cuts else 0
        options = [(abs(p - ideal) - bonus.get(w[-1], 0), p, i) for p, i, w in ends
                   if (not cuts or i > cuts[-1][1]) and p - start <= CUE_CHARS + 16]
        _, p, i = min(options)
        cuts.append((p, i))
    chunks, prev = [], 0
    for _, i in cuts + [(None, len(words) - 1)]:
        chunks.append(" ".join(words[prev:i + 1]))
        prev = i + 1
    return chunks


def duration(path):
    with wave.open(path) as w:
        return w.getnframes() / w.getframerate(), w.getframerate()


def heard(path):
    """Whisper's transcript of a clip: "" for a clip too short to hold speech,
    None when whisper isn't available or fails. Never raises; whisper-cli
    crashes (exit 11) on empty audio."""
    if not os.path.exists(WHISPER_MODEL):
        return None
    try:
        if duration(path)[0] < 0.3:
            return ""
        with tempfile.TemporaryDirectory() as tmp:
            pcm = os.path.join(tmp, "in.wav")
            subprocess.run(["ffmpeg", "-loglevel", "error", "-y", "-i", path, "-ar", "16000",
                            "-ac", "1", pcm], check=True)
            subprocess.run(["whisper-cli", "-m", WHISPER_MODEL, "-f", pcm, "-nt", "-np", "-otxt",
                            "-of", os.path.join(tmp, "out")], check=True,
                           stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            return open(os.path.join(tmp, "out.txt")).read().strip()
    except (OSError, EOFError, wave.Error, subprocess.CalledProcessError):
        return None


def closeness(script, transcript):
    def letters(s):
        words = re.findall(r"[a-z0-9']+", s.lower())
        return re.sub(r"[^a-z]", "", "".join(HOMOPHONES.get(w, w) for w in words))
    return difflib.SequenceMatcher(a=letters(script), b=letters(transcript), autojunk=False).ratio()


def synthesize(k, text, voice, model):
    """Returns (path, transcript, score); reuses a cached take for unchanged text."""
    path = os.path.join(AUDIO, f"{k:03d}.wav")
    key = hashlib.sha256(json.dumps([text, voice], sort_keys=True).encode()).hexdigest()
    stamp = path + ".json"
    if os.path.exists(path) and os.path.exists(stamp):
        meta = json.load(open(stamp))
        if meta["key"] == key:
            return path, meta["heard"], meta["score"]
    best = None
    for take in range(TAKES):
        cand = f"{path}.take{take}.wav"
        subprocess.run(
            ["piper", "-m", model, "-f", cand, "--length-scale", str(voice["length_scale"])],
            input=text.encode(), check=True, stderr=subprocess.DEVNULL,
        )
        transcript = heard(cand)
        score = closeness(text, transcript) if transcript is not None else 1.0
        if best is None or score > best[2]:
            best = (cand, transcript, score)
        if score >= GOOD_ENOUGH:
            break
    os.replace(best[0], path)
    for take in range(TAKES):
        if os.path.exists(f"{path}.take{take}.wav"):
            os.remove(f"{path}.take{take}.wav")
    with open(stamp, "w") as f:
        json.dump({"key": key, "heard": best[1], "score": round(best[2], 4)}, f)
    return path, best[1], best[2]


def srt_time(t):
    ms = int(round(t * 1000))
    h, ms = divmod(ms, 3600000)
    m, ms = divmod(ms, 60000)
    s, ms = divmod(ms, 1000)
    return f"{h:02d}:{m:02d}:{s:02d},{ms:03d}"


def clock(t):
    return f"{int(t // 60)}:{int(t % 60):02d}"


def write_script(spec, entries, total):
    """script.md: the narration as read aloud, regenerated on every run."""
    lines = [f"# {spec['title']}", "",
             f"Narration script, generated by `tts.py` from `segments.json` (edit that file, not this one). "
             f"Timings are from the last synthesis with `{spec['voice']['model']}`; total {clock(total)}.", ""]
    for seg in spec["segments"]:
        mine = [e for e in entries if e["segment"] == seg["id"]]
        title = seg["label"] or ("opening" if seg is spec["segments"][0] else "closing")
        lines += [f"## {title[:1].upper() + title[1:]} ({clock(mine[0]['start'])}–{clock(mine[-1]['end'])})", ""]
        for e in mine:
            lines.append(f"- `{clock(e['start'])}` {e['show']}")
        lines.append("")
    with open(os.path.join(ROOT, "script.md"), "w") as f:
        f.write("\n".join(lines))


def main():
    spec = json.load(open(os.path.join(ROOT, "segments.json")))
    voice = spec["voice"]
    model = os.path.join(BUILD, "voices", f"{voice['model']}.onnx")
    own = os.environ.get("OWN_AUDIO_DIR")
    os.makedirs(AUDIO, exist_ok=True)
    os.makedirs(OUT, exist_ok=True)

    flat = []
    for seg in spec["segments"]:
        for i, s in enumerate(seg["sentences"]):
            flat.append((seg, i, s))
    lint([s for _, _, s in flat])
    if own:
        missing = [f"{k:03d}" for k in range(len(flat)) if not os.path.exists(os.path.join(own, f"{k:03d}.wav"))]
        if missing:
            sys.exit(f"{own}: {len(missing)} of {len(flat)} lines not recorded yet: {', '.join(missing)}")

    t = voice["lead_in"]
    rate = None
    entries = []
    for k, (seg, i, s) in enumerate(flat):
        say = respell(s.get("say", s["show"]), spec["respell"])
        if own:
            path = os.path.join(own, f"{k:03d}.wav")
            transcript = heard(path)
            score = closeness(spoken(s["show"]), transcript) if transcript is not None else None
        else:
            path, transcript, score = synthesize(k, say, voice, model)
        dur, r = duration(path)
        if rate is None:
            rate = r
        elif r != rate:
            sys.exit(f"{path}: sample rate {r} differs from {rate}")
        start, end = t, t + dur
        last_in_segment = i == len(seg["sentences"]) - 1
        t = end + (voice["segment_gap"] if last_in_segment else voice["sentence_gap"])
        # Caption cues share the clip's time in proportion to their length.
        chunks = cues_for(s["show"])
        total_chars = sum(len(c) for c in chunks)
        cues, c0 = [], start
        for c in chunks:
            c1 = c0 + dur * len(c) / total_chars
            cues.append({"start": round(c0, 3), "end": round(c1, 3), "text": c})
            c0 = c1
        entries.append({
            "k": k, "segment": seg["id"], "label": seg["label"], "index": i,
            "start": round(start, 3), "end": round(end, 3), "wav": path,
            "show": s["show"], "say": say, "card": s.get("card"), "cues": cues,
            "heard": transcript, "score": score,
        })
    total = round(entries[-1]["end"] + voice["tail"], 3)

    # One narration track with every clip at its start time.
    frames = bytearray(int(total * rate) * 2)
    for e in entries:
        with wave.open(e["wav"]) as w:
            if w.getsampwidth() != 2 or w.getnchannels() != 1:
                sys.exit(f"{e['wav']}: need 16-bit mono")
            data = w.readframes(w.getnframes())
        at = int(e["start"] * rate) * 2
        frames[at:at + len(data)] = data
    with wave.open(os.path.join(BUILD, "narration.wav"), "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(rate)
        w.writeframes(bytes(frames))

    with open(os.path.join(BUILD, "timeline.json"), "w") as f:
        json.dump({"total": total, "rate": rate, "sentences": entries}, f, indent=1)
    if not own:
        write_script(spec, entries, total)

    n = 0
    with open(os.path.join(OUT, f"nibli-syllogism{SUFFIX}.en.srt"), "w") as f:
        for e in entries:
            for c in e["cues"]:
                n += 1
                f.write(f"{n}\n{srt_time(c['start'])} --> {srt_time(c['end'])}\n{c['text']}\n\n")

    words = sum(len(e["show"].split()) for e in entries)
    print(f"{len(entries)} sentences, {n} cues, {words} words, {total:.1f} s "
          f"({int(total // 60)}:{total % 60:04.1f}), "
          + ("your recordings" if own else f"voice {voice['model']}"))
    scored = [e for e in entries if e["score"] is not None and e["heard"] is not None]
    if scored:
        worst = sorted(scored, key=lambda e: e["score"])[:5]
        print(f"whisper closeness: mean {sum(e['score'] for e in scored) / len(scored):.3f}; lowest:")
        for e in worst:
            print(f"  {e['k']:03d} {e['score']:.3f} heard {e['heard']!r}")


if __name__ == "__main__":
    main()
