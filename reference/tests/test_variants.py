"""P1-VIDNI-02: building one variant never changes another.

The test first builds the synthetic narration. It then builds the creator's
voice narration, as `./build.sh own` does. Finally it asks compose which files
a plain build would read. Before the fix, the voice build overwrote the shared
build/timeline.json and build/narration.wav, so a later plain compose put the
creator's voice into the synthetic-named cut.

Run it from this folder inside its nix shell:

    python3 -I tests/test_variants.py

The creator's takes are read from RECORDINGS_DIR (default: recordings/).
"""

import hashlib
import json
import os
import subprocess
import sys
import unittest

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BUILD = os.path.join(ROOT, "build")
RECORDINGS = os.path.abspath(os.environ.get("RECORDINGS_DIR", os.path.join(ROOT, "recordings")))


def run(args, **env):
    clean = {k: v for k, v in os.environ.items() if k not in ("VARIANT", "OWN_AUDIO_DIR", "LD_LIBRARY_PATH")}
    clean["TTS_SKIP_WHISPER"] = "1"  # the leak doesn't depend on transcripts
    clean.update(env)
    return subprocess.run([sys.executable, "-I", *args], cwd=ROOT, env=clean, check=True,
                          capture_output=True, text=True).stdout


def digest(name):
    with open(os.path.join(BUILD, name), "rb") as f:
        return hashlib.sha256(f.read()).hexdigest()


class VariantsStayApart(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        run(["tts.py"])
        cls.synthetic = (digest("timeline.json"), digest("narration.wav"))
        run(["tts.py"], VARIANT="voice", OWN_AUDIO_DIR=RECORDINGS)

    def test_a_voice_build_leaves_the_synthetic_files_alone(self):
        self.assertEqual((digest("timeline.json"), digest("narration.wav")), self.synthetic)

    def test_each_variant_has_its_own_files(self):
        self.assertNotEqual(digest("timeline-voice.json"), self.synthetic[0])
        self.assertNotEqual(digest("narration-voice.wav"), self.synthetic[1])

    def test_a_plain_compose_reads_the_synthetic_files(self):
        plain = json.loads(run(["compose.py", "--plan"]))
        voice = json.loads(run(["compose.py", "--plan"], VARIANT="voice"))
        with open(os.path.join(BUILD, "timeline.json")) as f:
            synthetic_total = json.load(f)["total"]
        with open(os.path.join(BUILD, "timeline-voice.json")) as f:
            voice_total = json.load(f)["total"]
        self.assertEqual((plain["timeline"], plain["narration"]), ("build/timeline.json", "build/narration.wav"))
        self.assertEqual(plain["total"], synthetic_total)
        self.assertEqual(plain["outputs"], ["out/nibli-syllogism-16x9.mp4", "out/nibli-syllogism-9x16.mp4"])
        self.assertEqual((voice["timeline"], voice["narration"]),
                         ("build/timeline-voice.json", "build/narration-voice.wav"))
        self.assertEqual(voice["total"], voice_total)
        self.assertNotEqual(voice_total, synthetic_total)

    def test_clip_paths_are_relative(self):
        for name in ("timeline.json", "timeline-voice.json"):
            with open(os.path.join(BUILD, name)) as f:
                paths = [e["wav"] for e in json.load(f)["sentences"]]
            self.assertTrue(paths and not any(os.path.isabs(p) for p in paths), name)


if __name__ == "__main__":
    unittest.main(verbosity=2)
