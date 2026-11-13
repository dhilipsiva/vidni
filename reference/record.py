"""Prompter and recorder for your own narration: one WAV per script line.

Run it in a terminal (it is interactive):
    ./build.sh record --test        check the mic and the room first
    ./build.sh record               record every line not recorded yet
    ./build.sh record --only 12,23  redo lines 012 and 023
    ./build.sh record --from 30     start at line 030
Then build the voiced cuts with ./build.sh own.

Each line shows its section, the line before it and a pace guide (how long
the synthetic voice took). Press Enter, pause a beat, read, pause a beat,
press Enter. The take is trimmed (the keypresses and the silence around the
words), level-checked and transcribed, so a misread line shows up straight
away. Takes go to recordings/NNN.wav (48 kHz, 16-bit mono) and every raw take
stays in recordings/raw/. recordings/ is kept out of git, so back it up.
Set MIC to a PipeWire node name to use a microphone other than the default.
"""

import argparse
import json
import os
import re
import signal
import subprocess
import sys
import textwrap
import wave

ROOT = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, ROOT)
sys.dont_write_bytecode = True
from tts import SPOKEN, closeness, heard, spoken  # noqa: E402  (the narration build's whisper check)

REC = os.path.join(ROOT, "recordings")
RAW = os.path.join(REC, "raw")
GUIDE = os.path.join(ROOT, "build", "audio")
RATE = 48000
# Drop the start and stop keypresses, then the silence around the words.
CLEAN = ("atrim=start=0.20,areverse,atrim=start=0.30,areverse,highpass=f=70,"
         "silenceremove=start_periods=1:start_threshold=-45dB:start_silence=0.15,"
         "areverse,silenceremove=start_periods=1:start_threshold=-45dB:start_silence=0.25,areverse")

BOLD, DIM, GREEN, AMBER, RED, RESET = "\033[1m", "\033[2m", "\033[32m", "\033[33m", "\033[31m", "\033[0m"


def script_lines():
    spec = json.load(open(os.path.join(ROOT, "segments.json")))
    out = []
    for n, seg in enumerate(spec["segments"]):
        label = seg["label"] or ("opening" if n == 0 else "closing")
        out += [(label, s["show"]) for s in seg["sentences"]]
    return out


def take_path(k):
    return os.path.join(REC, f"{k:03d}.wav")


def seconds(path):
    with wave.open(path) as w:
        return w.getnframes() / w.getframerate()


def levels(path, start=None, length=None):
    cmd = ["ffmpeg", "-hide_banner", "-nostats"]
    if start is not None:
        cmd += ["-ss", str(start), "-t", str(length)]
    err = subprocess.run(cmd + ["-i", path, "-af", "volumedetect", "-f", "null", "-"],
                         capture_output=True, text=True).stderr
    peak = re.search(r"max_volume: (-?[\d.]+) dB", err)
    mean = re.search(r"mean_volume: (-?[\d.]+) dB", err)
    return (float(peak.group(1)) if peak else -99.0, float(mean.group(1)) if mean else -99.0)


def record(path):
    cmd = ["pw-record", "--rate", str(RATE), "--channels", "1", "--format", "s16"]
    if os.environ.get("MIC"):
        cmd += ["--target", os.environ["MIC"]]
    proc = subprocess.Popen(cmd + [path], stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
    try:
        input(f"  {RED}● recording{RESET}  read the line, then press Enter ")
    finally:
        proc.send_signal(signal.SIGINT)
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
    if not os.path.exists(path) or os.path.getsize(path) < 1000:
        err = proc.stderr.read().decode(errors="replace").strip()
        sys.exit(f"pw-record wrote nothing{': ' + err if err else ''}")


def clean(raw, out):
    subprocess.run(["ffmpeg", "-loglevel", "error", "-y", "-i", raw, "-af", CLEAN,
                    "-ar", str(RATE), "-ac", "1", "-c:a", "pcm_s16le", out], check=True)


def play(path):
    subprocess.run(["pw-play", path], check=False)


def report(path, text):
    peak, mean = levels(path)
    line = f"  {seconds(path):.1f} s · peak {peak:.1f} dB · mean {mean:.1f} dB"
    notes = []
    if peak > -1.0:
        notes.append(f"{RED}Too hot (peak {peak:.1f} dB): lower the input gain or move back a little.{RESET}")
    elif mean < -32:
        notes.append(f"{AMBER}Quiet (mean {mean:.1f} dB): move closer or raise the input gain.{RESET}")
    transcript = heard(path)
    if transcript is not None:
        score = closeness(spoken(text), transcript)
        line += f" · matches the script {GREEN if score >= 0.9 else AMBER}{score:.2f}{RESET}"
        if score < 0.9:
            notes.append(f"{AMBER}Heard: {transcript}{RESET}")
    print(line)
    for note in notes:
        print("  " + note)


def mic_test():
    os.makedirs(RAW, exist_ok=True)
    raw = os.path.join(RAW, "mic-test.wav")
    print(f"\n{BOLD}Mic check.{RESET} Press Enter, stay silent for three seconds, read this line, press Enter.\n")
    print(f"  {BOLD}A theorem prover automates that.{RESET}\n")
    input(f"  {DIM}Enter: start{RESET} ")
    record(raw)
    _, room = levels(raw, 0.5, 2.0)
    speech = os.path.join(RAW, "mic-test-speech.wav")
    clean(raw, speech)
    peak, mean = levels(speech)
    print(f"\n  room noise {room:.1f} dB · speech peak {peak:.1f} dB · speech mean {mean:.1f} dB")
    if room > -55:
        print(f"  {AMBER}The room or the mic is noisy. A quieter room, soft furnishings or a closer mic help.{RESET}")
    if peak > -1.0:
        print(f"  {RED}Speech is clipping. Lower the input gain.{RESET}")
    elif mean < -32:
        print(f"  {AMBER}Speech is quiet. Move closer or raise the input gain.{RESET}")
    elif room <= -55:
        print(f"  {GREEN}Levels look good.{RESET}")
    input(f"  {DIM}Enter: play it back{RESET} ")
    play(speech)


def prompt(k, script, guide):
    label, text = script[k]
    print("\033[2J\033[H", end="")
    pace = f" · pace guide {guide:.0f} s" if guide else ""
    print(f"{DIM}[{k:03d}] line {k + 1} of {len(script)} · {label}{pace}{RESET}")
    if k > 0:
        print(textwrap.fill(script[k - 1][1], 78, initial_indent=f"{DIM}  before: ", subsequent_indent="          ") + RESET)
    print()
    print(BOLD + textwrap.fill(text, 72, initial_indent="  ", subsequent_indent="  ") + RESET)
    for symbol, words in SPOKEN:
        if symbol in text:
            print(f"  {AMBER}say {symbol!r} as: {words}{RESET}")
    print()


def main():
    ap = argparse.ArgumentParser(description="Record your own narration, one line at a time.")
    ap.add_argument("--test", action="store_true", help="check the mic and the room")
    ap.add_argument("--from", dest="start", type=int, default=0, help="first line number to record")
    ap.add_argument("--only", default="", help="comma-separated line numbers to redo")
    args = ap.parse_args()
    if args.test:
        return mic_test()

    os.makedirs(RAW, exist_ok=True)
    script = script_lines()
    if args.only:
        todo = [int(x) for x in args.only.split(",") if x.strip()]
    else:
        todo = [k for k in range(args.start, len(script)) if not os.path.exists(take_path(k))]
    if not todo:
        print("Every line is recorded. Build with ./build.sh own, or redo lines with --only N,M.")
        return

    i = 0
    while 0 <= i < len(todo):
        k = todo[i]
        guide_wav = os.path.join(GUIDE, f"{k:03d}.wav")
        prompt(k, script, seconds(guide_wav) if os.path.exists(guide_wav) else None)
        cmd = input(f"  {DIM}Enter: record · s: skip · b: back · q: quit{RESET} ").strip().lower()
        if cmd == "q":
            break
        if cmd in ("s", "b"):
            i += 1 if cmd == "s" else -1
            i = max(i, 0)
            continue
        n = 1
        while os.path.exists(os.path.join(RAW, f"{k:03d}-take{n}.wav")):
            n += 1
        raw = os.path.join(RAW, f"{k:03d}-take{n}.wav")
        record(raw)
        new = take_path(k) + ".new.wav"
        clean(raw, new)
        report(new, script[k][1])
        while True:
            choice = input(f"  {DIM}Enter: keep · p: play · r: redo · q: keep and quit{RESET} ").strip().lower()
            if choice != "p":
                break
            play(new)
        if choice == "r":
            os.remove(new)
            continue
        os.replace(new, take_path(k))
        if choice == "q":
            break
        i += 1

    done = sum(os.path.exists(take_path(k)) for k in range(len(script)))
    print(f"\n{done} of {len(script)} lines recorded in recordings/.")
    if done == len(script):
        print("Build the voiced cuts with ./build.sh own")


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\nStopped. Every take you kept is saved in recordings/.")
