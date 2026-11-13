"""P1-VIDNI-05: the reference pipeline's behaviour, written as tests before the Rust port.

Covered: caption cue splitting, timeline assembly, camera framing, and the
recorder's terminal scenarios (a silent take, a double press, Esc, a kept
take). The tests need no microphone and no network: the recorder reads an
ffmpeg test source through the NIBLI_RECORD_SOURCE hook.

Run them with `just test-reference` from vidni's root, which runs every test in
this folder in the reference's nix shell, without network or a microphone. To
run only these, inside that shell:

    python3 -I -B tests/test_behaviour.py

Golden data is read from GOLDEN_DIR (default: ../golden, which is vidni's
golden/).
"""

import json
import math
import os
import pty
import re
import select
import shutil
import subprocess
import sys
import tempfile
import time
import unittest
import wave

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, ROOT)
sys.dont_write_bytecode = True
import compose  # noqa: E402
import tts  # noqa: E402

GOLDEN = os.path.abspath(os.environ.get("GOLDEN_DIR", os.path.join(ROOT, "..", "golden")))
TOL = 0.0015  # timeline values are rounded to milliseconds


def golden_timeline(variant="synthetic"):
    with open(os.path.join(GOLDEN, f"timeline-{variant}.json")) as f:
        return json.load(f)


def clip_seconds(rel):
    with wave.open(os.path.join(ROOT, rel)) as w:
        return w.getnframes() / w.getframerate()


class CueSplitting(unittest.TestCase):
    def test_every_sentence_splits_as_the_golden_captions_do(self):
        for e in golden_timeline()["sentences"]:
            self.assertEqual(tts.cues_for(e["show"]), [c["text"] for c in e["cues"]], e["show"])

    def test_splitting_loses_nothing_and_stays_bounded(self):
        for e in golden_timeline()["sentences"]:
            cues = tts.cues_for(e["show"])
            self.assertEqual(" ".join(cues), e["show"])
            self.assertTrue(all(len(c) <= tts.CUE_CHARS + 16 for c in cues), cues)

    def test_a_short_sentence_is_one_cue(self):
        self.assertEqual(tts.cues_for("Adam is a dog."), ["Adam is a dog."])

    def test_a_long_sentence_splits_at_punctuation_near_the_middle(self):
        text = ("The engine starts from the claim and works backward through every rule it knows, "
                "and it stops when the chain closes on a stored fact.")
        self.assertEqual(tts.cues_for(text), [
            "The engine starts from the claim and works backward through every rule it knows,",
            "and it stops when the chain closes on a stored fact."])


class TimelineAssembly(unittest.TestCase):
    def setUp(self):
        with open(os.path.join(ROOT, "segments.json")) as f:
            self.voice = json.load(f)["voice"]
        self.timeline = golden_timeline()
        self.sentences = self.timeline["sentences"]

    def test_the_first_line_starts_after_the_lead_in(self):
        self.assertAlmostEqual(self.sentences[0]["start"], self.voice["lead_in"], delta=TOL)

    def test_each_line_lasts_exactly_as_long_as_its_clip(self):
        for e in self.sentences:
            self.assertAlmostEqual(e["end"] - e["start"], clip_seconds(e["wav"]), delta=TOL, msg=e["wav"])

    def test_gaps_are_the_sentence_gap_within_a_segment_and_the_segment_gap_between(self):
        for a, b in zip(self.sentences, self.sentences[1:]):
            gap = self.voice["sentence_gap"] if a["segment"] == b["segment"] else self.voice["segment_gap"]
            self.assertAlmostEqual(b["start"] - a["end"], gap, delta=TOL, msg=b["show"][:40])

    def test_the_total_adds_the_tail(self):
        self.assertAlmostEqual(self.timeline["total"], self.sentences[-1]["end"] + self.voice["tail"], delta=TOL)

    def test_cues_tile_their_line_in_proportion_to_their_length(self):
        for e in self.sentences:
            cues = e["cues"]
            self.assertAlmostEqual(cues[0]["start"], e["start"], delta=TOL)
            self.assertAlmostEqual(cues[-1]["end"], e["end"], delta=TOL)
            chars = sum(len(c["text"]) for c in cues)
            for c in cues:
                share = (e["end"] - e["start"]) * len(c["text"]) / chars
                self.assertAlmostEqual(c["end"] - c["start"], share, delta=2 * TOL)
            for a, b in zip(cues, cues[1:]):
                self.assertAlmostEqual(a["end"], b["start"], delta=TOL)


class CameraFraming(unittest.TestCase):
    """compose.crop_rect: the crop frames a padded target between the top reserve
    (an overlay) and the bottom reserve (captions), and never leaves the screen."""

    VIEW = (1280, 720)
    PAD = 16
    TARGETS = [(646, 123, 622, 303), (12, 123, 622, 303), (676, 261, 554, 50), (0, 0, 1280, 720),
               (1000, 600, 200, 100), (29, 495, 588, 36)]
    EPS = 1e-6

    def framings(self):
        """(cut, top reserve, target, crop) for every target, cut and reserve."""
        for name, cut in compose.CUTS.items():
            aspect = cut["view"][2] / cut["view"][3]
            for top in ((0.0, 0.28) if name == "16x9" else (0.0,)):
                for t in self.TARGETS:
                    yield cut, top, aspect, t, compose.crop_rect(t, aspect, cut, self.VIEW, top)

    def clear_of_the_edges(self, crop):
        x, y, w, h = crop
        return (x > self.EPS and y > self.EPS and x + w < self.VIEW[0] - self.EPS
                and y + h < self.VIEW[1] - self.EPS)

    def test_every_crop_has_the_cut_aspect_and_stays_on_screen(self):
        for cut, top, aspect, t, (x, y, w, h) in self.framings():
            self.assertAlmostEqual(w / h, aspect, places=6)
            self.assertGreaterEqual(x, -self.EPS)
            self.assertGreaterEqual(y, -self.EPS)
            self.assertLessEqual(x + w, self.VIEW[0] + self.EPS)
            self.assertLessEqual(y + h, self.VIEW[1] + self.EPS)

    def test_crops_never_zoom_past_the_minimum_width(self):
        for cut, top, aspect, t, (x, y, w, h) in self.framings():
            self.assertGreaterEqual(w + self.EPS, min(cut["min_w"], self.VIEW[0], self.VIEW[1] * aspect))

    def test_a_crop_clear_of_the_edges_centres_the_target_in_its_free_band(self):
        centred = 0
        for cut, top, aspect, t, crop in self.framings():
            if not self.clear_of_the_edges(crop):
                continue
            x, y, w, h = crop
            tx, ty, tw, th = t
            above = (ty - self.PAD) - (y + top * h)
            below = (y + h * (1 - cut["reserve"])) - (ty + th + self.PAD)
            left, right = (tx - self.PAD) - x, (x + w) - (tx + tw + self.PAD)
            self.assertGreaterEqual(min(above, below, left, right), -self.EPS, t)
            self.assertAlmostEqual(above, below, places=6, msg=t)
            self.assertAlmostEqual(left, right, places=6, msg=t)
            centred += 1
        self.assertGreaterEqual(centred, 3, "too few crops clear of the edges to test")

    def test_a_crop_pinned_to_an_edge_still_holds_the_target(self):
        for cut, top, aspect, t, (x, y, w, h) in self.framings():
            tx, ty, tw, th = t
            if tw <= w and th <= h:
                self.assertTrue(x - self.EPS <= tx and tx + tw <= x + w + self.EPS, t)
                self.assertTrue(y - self.EPS <= ty and ty + th <= y + h + self.EPS, t)

    def test_at_the_bottom_edge_the_crop_stays_on_screen_and_captions_may_cover_the_target(self):
        # The reference's trade-off, kept as it is so the port matches it or changes it on purpose.
        cut = compose.CUTS["16x9"]
        x, y, w, h = compose.crop_rect((1000, 600, 200, 100), 16 / 9, cut, self.VIEW)
        self.assertAlmostEqual(y + h, self.VIEW[1], places=6)
        self.assertGreater(600 + 100, y + h * (1 - cut["reserve"]))

    def test_an_overlay_reserve_pushes_the_target_below_it(self):
        for cut, top, aspect, t, (x, y, w, h) in self.framings():
            if top and y > 0:
                self.assertGreaterEqual(t[1] - self.PAD, y + h * top - self.EPS, t)

    def test_known_framings(self):
        known = [("16x9", (646, 123, 622, 303), [535.556, 107.0, 744.444, 418.75]),
                 ("16x9", (676, 261, 554, 50), [633.0, 142.0, 640.0, 360.0]),
                 ("16x9", (1000, 600, 200, 100), [640.0, 360.0, 640.0, 360.0]),
                 ("9x16", (676, 261, 554, 50), [660.0, 0.0, 586.0, 634.833])]
        for name, target, want in known:
            cut = compose.CUTS[name]
            crop = compose.crop_rect(target, cut["view"][2] / cut["view"][3], cut, self.VIEW)
            self.assertEqual([round(v, 3) for v in crop], want, (name, target))


class RecorderScenarios(unittest.TestCase):
    """Drives record.py in a pty with ffmpeg test sources instead of a microphone."""

    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.mkdtemp(prefix="vidni-recorder-")
        for name in ("record.py", "tts.py", "segments.json"):
            shutil.copy(os.path.join(ROOT, name), cls.tmp)
        os.symlink(os.path.join(ROOT, "build"), os.path.join(cls.tmp, "build"))
        os.makedirs(os.path.join(cls.tmp, "recordings"))
        # Line 000 is already recorded, so every session starts at line 001.
        shutil.copy(os.path.join(ROOT, "build", "audio", "000.wav"), os.path.join(cls.tmp, "recordings"))

    @classmethod
    def tearDownClass(cls):
        shutil.rmtree(cls.tmp)

    def session(self, source, keys):
        env = dict(os.environ, NIBLI_RECORD_SOURCE=source, TERM="xterm",
                   RECORDINGS_DIR=os.path.join(self.tmp, "recordings"))
        env.pop("LD_LIBRARY_PATH", None)
        pid, fd = pty.fork()
        if pid == 0:
            os.execvpe(sys.executable, [sys.executable, "-I", os.path.join(self.tmp, "record.py")], env)
        buf = b""

        def read_for(seconds):
            nonlocal buf
            end = time.time() + seconds
            while time.time() < end:
                ready, _, _ = select.select([fd], [], [], 0.05)
                if ready:
                    try:
                        buf += os.read(fd, 65536)
                    except OSError:
                        return

        read_for(2.0)
        for key, wait in keys:
            os.write(fd, key.encode())
            read_for(wait)
        read_for(1.0)
        os.waitpid(pid, 0)
        return re.sub(r"\x1b\[[0-9;?]*[A-Za-z]", "", buf.decode(errors="replace"))

    def test_a_silent_take_is_discarded_and_the_line_stays(self):
        text = self.session("lavfi:anullsrc=r=48000:cl=mono", [(" ", 2.0), (" ", 3.0), ("q", 1.0)])
        self.assertIn("No speech in that take", text)
        self.assertNotIn("Traceback", text)
        self.assertFalse(os.path.exists(os.path.join(self.tmp, "recordings", "001.wav")))

    def test_a_double_press_is_ignored_and_esc_cancels(self):
        text = self.session("lavfi:anullsrc=r=48000:cl=mono",
                            [(" ", 0.1), (" ", 1.5), (" ", 3.0), (" ", 1.2), ("\x1b", 1.5), ("q", 1.0)])
        self.assertIn("No speech in that take", text)
        self.assertIn("Cancelled; nothing was saved", text)

    def test_a_spoken_take_is_kept_and_the_next_line_comes_up(self):
        clip = os.path.join(ROOT, "build", "audio", "001.wav")
        text = self.session(f"lavfi:amovie={clip}:loop=0", [(" ", 10.5), (" ", 14.0), ("\r", 2.0), ("q", 1.5)])
        self.assertIn("matches the script", text)
        self.assertIn("Line 3 of 57", text)
        self.assertTrue(os.path.exists(os.path.join(self.tmp, "recordings", "001.wav")))


if __name__ == "__main__":
    unittest.main(verbosity=2)
