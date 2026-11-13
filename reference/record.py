"""Prompter and recorder for your own narration: one WAV per script line.

Run it in a terminal; it reads single keys, no Enter needed:
    ./build.sh record --test        mic and room check
    ./build.sh record               start at the first line not recorded yet
    ./build.sh record --only 12,23  redo lines 012 and 023
    ./build.sh record --from 30     start at line 030
Then build the voiced cuts with ./build.sh own.

Keys
    ready      space record · p play the saved take · ← → move between lines · q quit
    recording  space or enter stop · esc cancel
    review     enter keep and go on · p play · r record again · esc discard · q keep and quit

While recording, a meter shows your level and a timer runs against the pace
guide (how long the synthetic voice took). A take that is too short or has
no speech is discarded with a message and you stay on the same line. Kept
takes are trimmed (keypresses and the silence around the words), checked
for level and transcribed, so a misread line shows up straight away. Takes
go to recordings/NNN.wav (48 kHz, 16-bit mono); every raw take stays in
recordings/raw/. recordings/ is kept out of git, so back it up.
Set MIC to a PipeWire node name to use a microphone other than the default.
"""

import argparse
import array
import json
import math
import os
import re
import select
import shutil
import signal
import subprocess
import sys
import termios
import textwrap
import time
import traceback
import tty
import wave

ROOT = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, ROOT)
sys.dont_write_bytecode = True
from tts import SPOKEN, closeness, heard, spoken  # noqa: E402  (the narration build's whisper check)

REC = os.path.join(ROOT, "recordings")
RAW = os.path.join(REC, "raw")
GUIDE = os.path.join(ROOT, "build", "audio")
RATE = 48000
SETTLE = 0.4          # seconds after a state change during which keys are ignored
MIN_TAKE = 0.8        # seconds; shorter raw takes are discarded
SILENT_PEAK = -40.0   # dB; a take that never gets louder than this has no speech
# Drop the start and stop keypresses, then the silence around the words.
CLEAN = ("atrim=start=0.20,areverse,atrim=start=0.30,areverse,highpass=f=70,"
         "silenceremove=start_periods=1:start_threshold=-45dB:start_silence=0.15,"
         "areverse,silenceremove=start_periods=1:start_threshold=-45dB:start_silence=0.25,areverse")
# Test hook: NIBLI_RECORD_SOURCE=lavfi:<filter> records an ffmpeg source instead of the mic.
SOURCE = os.environ.get("NIBLI_RECORD_SOURCE", "")

BOLD, DIM, GREEN, AMBER, RED, RESET = "\033[1m", "\033[2m", "\033[32m", "\033[33m", "\033[31m", "\033[0m"


class Keys:
    """Single keypresses without Enter. The terminal is restored on exit."""

    def __init__(self):
        if not sys.stdin.isatty():
            sys.exit("record.py is interactive: run it in a terminal.")
        self.fd = sys.stdin.fileno()
        self.saved = termios.tcgetattr(self.fd)

    def __enter__(self):
        tty.setcbreak(self.fd)
        sys.stdout.write("\033[?25l")  # hide the cursor
        return self

    def __exit__(self, *exc):
        termios.tcsetattr(self.fd, termios.TCSADRAIN, self.saved)
        sys.stdout.write("\033[?25h\n")
        sys.stdout.flush()

    def settle(self):
        """Ignore anything pressed in the next moment (a double press, a held key)."""
        time.sleep(SETTLE)
        termios.tcflush(self.fd, termios.TCIFLUSH)

    def get(self, timeout=None):
        ready, _, _ = select.select([self.fd], [], [], timeout)
        if not ready:
            return None
        ch = os.read(self.fd, 1).decode(errors="ignore")
        if ch == "\x1b":
            more, _, _ = select.select([self.fd], [], [], 0.03)
            if not more:
                return "esc"
            seq = os.read(self.fd, 2).decode(errors="ignore")
            return {"[C": "right", "[D": "left", "[A": "up", "[B": "down"}.get(seq, "esc")
        if ch in ("\r", "\n"):
            return "enter"
        return "space" if ch == " " else ch.lower()


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
    try:
        with wave.open(path) as w:
            return w.getnframes() / w.getframerate()
    except (OSError, EOFError, wave.Error):
        return 0.0


def levels(path, start=None, length=None):
    """(peak, mean) in dB; (-99, -99) when the file has no audio."""
    cmd = ["ffmpeg", "-hide_banner", "-nostats"]
    if start is not None:
        cmd += ["-ss", str(start), "-t", str(length)]
    err = subprocess.run(cmd + ["-i", path, "-af", "volumedetect", "-f", "null", "-"],
                         capture_output=True, text=True).stderr
    peak = re.search(r"max_volume: (-?[\d.]+) dB", err)
    mean = re.search(r"mean_volume: (-?[\d.]+) dB", err)
    return (float(peak.group(1)) if peak else -99.0, float(mean.group(1)) if mean else -99.0)


def tail_level(path, span=0.12):
    """Level of the newest audio in a file still being written, in dB."""
    try:
        size = os.path.getsize(path)
        n = int(RATE * span) * 2
        if size < 512 + n:
            return None
        with open(path, "rb") as f:
            f.seek(size - n - (size % 2))
            data = array.array("h", f.read(n))
    except OSError:
        return None
    if not data:
        return None
    rms = math.sqrt(sum(x * x for x in data) / len(data)) / 32768
    return 20 * math.log10(rms) if rms > 0 else -96.0


def meter(db, width=24):
    if db is None:
        return DIM + "·" * width + RESET
    filled = max(0, min(width, int((db + 60) / 60 * width)))
    colour = RED if db > -3 else AMBER if db > -12 else GREEN
    return colour + "█" * filled + RESET + DIM + "·" * (width - filled) + RESET


def status(text):
    sys.stdout.write("\r\033[2K" + text)
    sys.stdout.flush()


def recorder(path):
    if SOURCE.startswith("lavfi:"):
        cmd = ["ffmpeg", "-loglevel", "error", "-y", "-re", "-f", "lavfi", "-i", SOURCE[6:],
               "-ac", "1", "-ar", str(RATE), "-c:a", "pcm_s16le", path]
        return subprocess.Popen(cmd, stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
    cmd = ["pw-record", "--rate", str(RATE), "--channels", "1", "--format", "s16"]
    if os.environ.get("MIC"):
        cmd += ["--target", os.environ["MIC"]]
    return subprocess.Popen(cmd + [path], stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)


def capture(keys, path, guide=None, duration=None, label="REC"):
    """Record until space or enter (or for `duration` seconds). Returns 'ok', 'cancel' or an error."""
    proc = recorder(path)
    t0 = time.monotonic()
    outcome = "ok"
    termios.tcflush(keys.fd, termios.TCIFLUSH)
    try:
        while True:
            elapsed = time.monotonic() - t0
            if proc.poll() is not None:
                err = proc.stderr.read().decode(errors="replace").strip()
                outcome = f"The recorder stopped on its own{': ' + err if err else ''}. Check the mic with --test."
                break
            if duration is not None and elapsed >= duration:
                break
            pace = f" of ~{guide:.0f} s" if guide else ""
            hint = "esc cancel" if duration is not None else "space stop · esc cancel"
            status(f"  {RED}● {label}{RESET} {elapsed:4.1f} s{pace}  {meter(tail_level(path))}  {DIM}{hint}{RESET}")
            key = keys.get(0.1)
            if key is None or elapsed < SETTLE:
                continue
            if key == "esc":
                outcome = "cancel"
                break
            if duration is None and key in ("space", "enter"):
                break
    finally:
        if proc.poll() is None:
            proc.send_signal(signal.SIGINT)
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                proc.kill()
        status("")
    return outcome


def clean(raw, out):
    subprocess.run(["ffmpeg", "-loglevel", "error", "-y", "-i", raw, "-af", CLEAN,
                    "-ar", str(RATE), "-ac", "1", "-c:a", "pcm_s16le", out], check=True)


def check_take(raw, new):
    """Trim the take into `new`. Returns None when it is usable, else why it was discarded."""
    length = seconds(raw)
    if length < MIN_TAKE:
        return f"That take was only {length:.1f} s, so it was discarded."
    peak, _ = levels(raw)
    if peak < SILENT_PEAK:
        return (f"No speech in that take (its loudest point was {peak:.0f} dB), so it was discarded. "
                "If you did speak, run ./build.sh record --test to check the mic.")
    try:
        clean(raw, new)
    except subprocess.CalledProcessError:
        return "That take could not be trimmed, so it was discarded."
    if seconds(new) < 0.4:
        return "Nothing was left after trimming the silence, so the take was discarded."
    return None


def review_lines(path, text):
    peak, _ = levels(path)
    lines = [f"{seconds(path):.1f} s · peak {peak:.1f} dB"]
    if peak > -1.0:
        lines.append(f"{RED}Too hot: lower the input gain or move back a little, then record again.{RESET}")
    elif peak < -20:
        lines.append(f"{AMBER}Quiet: move closer or raise the input gain.{RESET}")
    transcript = heard(path)
    if transcript is not None:
        score = closeness(spoken(text), transcript)
        lines[0] += f" · matches the script {GREEN if score >= 0.9 else AMBER}{score:.2f}{RESET}"
        if score < 0.9:
            lines.append(f"{AMBER}Heard: {transcript or '(nothing)'}{RESET}")
    return lines


def play(keys, path):
    """Play a file; any key stops it."""
    proc = subprocess.Popen(["pw-play", path], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    status(f"  {DIM}▶ playing · any key stops{RESET}")
    try:
        while proc.poll() is None:
            if keys.get(0.1) is not None:
                proc.terminate()
                break
    finally:
        proc.wait()
        status("")


def draw(k, script, recorded, panel=(), legend=""):
    width = min(max(shutil.get_terminal_size((90, 24)).columns - 4, 40), 80)
    label, text = script[k]
    guide = seconds(os.path.join(GUIDE, f"{k:03d}.wav"))
    out = ["\033[2J\033[H"]
    out.append(f"{DIM}nibli narration · {recorded} of {len(script)} recorded{RESET}")
    out.append(f"{BOLD}Line {k + 1} of {len(script)}{RESET}  {DIM}[{k:03d}] · {label}"
               + (f" · pace guide {guide:.0f} s" if guide else "") + RESET)
    out.append(DIM + "─" * width + RESET)
    if k > 0:
        out.append(textwrap.fill(script[k - 1][1], width, initial_indent=f"{DIM}before  ",
                                 subsequent_indent="        ") + RESET)
    out.append("")
    out.append(BOLD + textwrap.fill(text, width, initial_indent="  ", subsequent_indent="  ") + RESET)
    for symbol, words in SPOKEN:
        if symbol in text:
            out.append(f"  {AMBER}say {symbol} as “{words}”{RESET}")
    out.append("")
    if os.path.exists(take_path(k)):
        out.append(f"  {GREEN}✓ saved take, {seconds(take_path(k)):.1f} s{RESET}")
    for line in panel:
        out.append("  " + line)
    out.append(DIM + "─" * width + RESET)
    out.append(legend)
    sys.stdout.write("\n".join(out) + "\n")
    sys.stdout.flush()


READY = f"  {BOLD}space{RESET} record · {BOLD}p{RESET} play saved take · {BOLD}← →{RESET} lines · {BOLD}q{RESET} quit"
REVIEW = (f"  {BOLD}enter{RESET} keep and go on · {BOLD}p{RESET} play · {BOLD}r{RESET} record again · "
          f"{BOLD}esc{RESET} discard · {BOLD}q{RESET} keep and quit")


def next_take_name(k):
    n = 1
    while os.path.exists(os.path.join(RAW, f"{k:03d}-take{n}.wav")):
        n += 1
    return os.path.join(RAW, f"{k:03d}-take{n}.wav")


def record_line(keys, k, script):
    """Record takes for line k until one is kept or the user backs out.
    Returns 'kept', 'kept-quit', 'quit' or 'back' plus a note for the ready screen."""
    text = script[k][1]
    guide = seconds(os.path.join(GUIDE, f"{k:03d}.wav")) or None
    recorded = lambda: sum(os.path.exists(take_path(i)) for i in range(len(script)))  # noqa: E731
    while True:
        draw(k, script, recorded(), [f"{DIM}Read when the meter starts moving; press space when you finish.{RESET}"], "")
        raw = next_take_name(k)
        outcome = capture(keys, raw, guide)
        if outcome == "cancel":
            return "back", f"{DIM}Cancelled; nothing was saved.{RESET}"
        if outcome != "ok":
            return "back", f"{RED}{outcome}{RESET}"
        new = take_path(k) + ".new.wav"
        status(f"  {DIM}checking the take…{RESET}")
        problem = check_take(raw, new)
        if problem:
            if os.path.exists(new):
                os.remove(new)
            return "back", f"{AMBER}{problem}{RESET}"
        panel = review_lines(new, text)
        while True:
            draw(k, script, recorded(), [f"{BOLD}New take{RESET}"] + panel, REVIEW)
            keys.settle()
            key = keys.get()
            if key == "p":
                play(keys, new)
                continue
            if key in ("enter", "q"):
                os.replace(new, take_path(k))
                return ("kept-quit" if key == "q" else "kept"), ""
            if key == "esc":
                os.remove(new)
                return "back", f"{DIM}Discarded; the line is as it was.{RESET}"
            if key == "r":
                os.remove(new)
                break


def mic_test(keys):
    os.makedirs(RAW, exist_ok=True)
    room_wav = os.path.join(RAW, "mic-test-room.wav")
    speech_raw = os.path.join(RAW, "mic-test.wav")
    speech = os.path.join(RAW, "mic-test-speech.wav")
    line = "A theorem prover automates that."
    while True:
        sys.stdout.write(f"\033[2J\033[H{BOLD}Mic check{RESET}\n\n"
                         f"  Step 1 of 2: stay quiet for 3 seconds while the room is measured.\n\n")
        sys.stdout.flush()
        keys.settle()
        if capture(keys, room_wav, duration=3.0, label="ROOM") == "cancel":
            return
        room = levels(room_wav, 0.3, 2.5)[1]
        sys.stdout.write(f"  Step 2 of 2: press {BOLD}space{RESET}, read this line, press {BOLD}space{RESET}.\n\n"
                         f"  {BOLD}{line}{RESET}\n\n")
        sys.stdout.flush()
        keys.settle()
        while keys.get() not in ("space", "enter", "esc", "q"):
            pass
        if capture(keys, speech_raw) == "cancel":
            return
        problem = check_take(speech_raw, speech)
        sys.stdout.write(f"  room noise {room:.0f} dB\n")
        if problem:
            sys.stdout.write(f"  {AMBER}{problem}{RESET}\n")
        else:
            peak, _ = levels(speech)
            sys.stdout.write(f"  speech peak {peak:.1f} dB\n")
            if room > -55:
                sys.stdout.write(f"  {AMBER}The room or the mic is noisy. A quieter room, soft furnishings or a closer mic help.{RESET}\n")
            if peak > -1.0:
                sys.stdout.write(f"  {RED}Speech is clipping. Lower the input gain.{RESET}\n")
            elif peak < -20:
                sys.stdout.write(f"  {AMBER}Speech is quiet. Move closer or raise the input gain.{RESET}\n")
            elif room <= -55:
                sys.stdout.write(f"  {GREEN}Levels look good.{RESET}\n")
        sys.stdout.write(f"\n  {BOLD}p{RESET} play it back · {BOLD}r{RESET} run the check again · {BOLD}q{RESET} done\n")
        sys.stdout.flush()
        keys.settle()
        while True:
            key = keys.get()
            if key == "p" and not problem:
                play(keys, speech)
            elif key == "r":
                break
            elif key in ("q", "esc", "enter"):
                return


def main():
    ap = argparse.ArgumentParser(description="Record your own narration, one line at a time.")
    ap.add_argument("--test", action="store_true", help="check the mic and the room")
    ap.add_argument("--from", dest="start", type=int, default=None, help="line number to start at")
    ap.add_argument("--only", default="", help="comma-separated line numbers to redo")
    args = ap.parse_args()
    os.makedirs(RAW, exist_ok=True)
    for name in os.listdir(REC):  # leftovers of an interrupted take
        if name.endswith(".new.wav"):
            os.remove(os.path.join(REC, name))

    script = script_lines()
    with Keys() as keys:
        if args.test:
            return mic_test(keys)
        order = [int(x) for x in args.only.split(",") if x.strip()] if args.only else list(range(len(script)))
        if args.start is not None:
            pos = order.index(args.start) if args.start in order else 0
        elif args.only:
            pos = 0
        else:
            missing = [i for i, k in enumerate(order) if not os.path.exists(take_path(k))]
            pos = missing[0] if missing else 0
        note = ""
        while True:
            k = order[pos]
            recorded = sum(os.path.exists(take_path(i)) for i in range(len(script)))
            draw(k, script, recorded, [note] if note else [], READY)
            note = ""
            key = keys.get()
            if key in ("q", "esc"):
                break
            if key in ("left", "up", "b"):
                pos = max(pos - 1, 0)
            elif key in ("right", "down", "n"):
                pos = min(pos + 1, len(order) - 1)
            elif key == "p" and os.path.exists(take_path(k)):
                play(keys, take_path(k))
            elif key in ("space", "enter"):
                result, note = record_line(keys, k, script)
                if result == "kept-quit":
                    break
                if result == "kept":
                    later = [i for i in range(pos + 1, len(order)) if not os.path.exists(take_path(order[i]))]
                    if args.only:
                        later = list(range(pos + 1, len(order)))
                    if later:
                        pos = later[0]
                    else:
                        earlier = [i for i in range(len(order)) if not os.path.exists(take_path(order[i]))]
                        if not earlier:
                            break
                        pos = earlier[0]
                        note = f"{AMBER}Back to the first line still missing.{RESET}"

    done = sum(os.path.exists(take_path(i)) for i in range(len(script)))
    print(f"{done} of {len(script)} lines recorded in recordings/.")
    if done == len(script):
        print("Build the voiced cuts with ./build.sh own")
    else:
        missing = [f"{i:03d}" for i in range(len(script)) if not os.path.exists(take_path(i))]
        print("Still to record: " + ", ".join(missing[:12]) + (" …" if len(missing) > 12 else ""))


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("Stopped. Every take you kept is saved in recordings/.")
    except Exception:
        log = os.path.join(REC, "record-error.log")
        os.makedirs(REC, exist_ok=True)
        with open(log, "a") as f:
            f.write(time.strftime("%Y-%m-%d %H:%M:%S ") + traceback.format_exc() + "\n")
        print(f"Something went wrong; the details are in {log}. Every take you kept is saved. "
              "Run ./build.sh record again to continue where you left off.")
        sys.exit(1)
