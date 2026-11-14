# Draft voices

A draft voice speaks your script until you record it yourself. This page records how candidate voices were measured, which were accepted, and why. `vidni voices` lists the accepted ones with their licences.

**What is measured, and what isn't.** Three things decide whether a voice can be a draft voice:

1. **It repeats.** The same words, with the same seed, give the same samples every time. Without that, a draft isn't reproducible.
2. **It can be understood.** A listener hears the words it was given.
3. **Its licence allows it,** and says whether the audio may be used commercially.

**How natural a voice sounds is not judged here** (DECISIONS.md, A7). That is a matter of taste and of the listener, and a measurement of it would be a number about nothing. Listen to a voice before you choose it.

## The measurement

`tools/voices_eval.py` speaks 15 sentences (155 words) of the kind a tutorial has: plain statements, numbers, names, a version number. For each voice:

- **Repeatability:** every sentence is spoken twice, in two separate processes, with the seed 1292372942. The two takes' samples are hashed. The score is how many of the 15 are the same, sample for sample.
- **Intelligibility:** whisper.cpp, with the pinned `base.en` model, writes down what it hears of each first take. The score is the share of the words that it heard right, after lower-casing, dropping punctuation, and reading numbers as figures on both sides (whisper writes "37" as often as "thirty-seven").

Both were run on one machine, the maintainer's desktop (x86_64 Linux, CPU only), on 2026-11-14. Whether a voice gives the same samples on another machine is a separate question: Piper's does within vidni's pins ([determinism.md](determinism.md)); Kokoro's wasn't tried elsewhere.

The listener is a small model, and it mishears. Its mistakes are in the scores: "a proof" heard as "proof", "Adam" as "an atom". A score a few words below another isn't a worse voice. A score far below would be.

## Results

| Voice | Engine | Same in two processes | Words heard | Licence of the voice | Commercial use | Accepted |
|---|---|---|---|---|---|---|
| `en_US-joe-medium` | Piper 1.8.0 | 15 of 15 | 151 of 155 (97.4%) | CC0-1.0 | yes | yes (the default, since P3) |
| `en_US-kristin-medium` | Piper 1.8.0 | 15 of 15 | 152 of 155 (98.1%) | public domain (LibriVox) | yes | **yes** |
| `en_US-john-medium` | Piper 1.8.0 | 15 of 15 | 152 of 155 (98.1%) | public domain (LibriVox) | yes | **yes** |
| `en_US-norman-medium` | Piper 1.8.0 | 15 of 15 | 149 of 155 (96.1%) | public domain (LibriVox) | yes | no |
| `en_US-bryce-medium` | Piper 1.8.0 | 15 of 15 | 153 of 155 (98.7%) | "public domain", with no source named | unclear | no |
| Kokoro v1.0, `af_heart` | Kokoro (PyTorch) | 15 of 15 | 155 of 155 (100%) | Apache-2.0 | yes | not yet: see below |
| Kokoro v1.0, `am_michael` | Kokoro (PyTorch) | 15 of 15 | 152 of 155 (98.1%) | Apache-2.0 | yes | not yet: see below |

## What was accepted

**`piper:en_US-kristin-medium` and `piper:en_US-john-medium`** are draft voices now, pinned in `crates/vidni-core/src/voices.toml` by each file's size and SHA-256, at the same revision of `rhasspy/piper-voices` as the others:

| File | SHA-256 |
|---|---|
| `en_US-kristin-medium.onnx` | `5849957f929cbf720c258f8458692d6103fff2f0e3d3b19c8259474bb06a18d4` |
| `en_US-kristin-medium.onnx.json` | `5681426d4aead22195de70531eeeeddb46493cfaffc5764b2ea3db73428b651c` |
| `en_US-john-medium.onnx` | `789c6c875726e627ddee93d51d8727859abe9c091c3d141591f4b83c2072e988` |
| `en_US-john-medium.onnx.json` | `af60f177b6b550f3d7a302720c0fb89e7f94a82b5dca464775ef63b1c69ba09a` |

`vidni voices fetch piper:en_US-kristin-medium` fetches one, and `[voice] draft = "piper:en_US-kristin-medium"` uses it. They give a project a woman's voice and a second man's, both free for any use.

- **Their licence.** Each model card gives the training data as recordings from LibriVox, which are in the public domain, and says how many hours. That is the card's statement, read on 2026-11-14; vidni repeats it and hasn't checked the recordings.
- **`norman` wasn't accepted:** it repeats and is free to use, but the listener lost more of its words than of the others', in the same places each time ("a reasoner derives" heard as "a reason of drives").
- **`bryce` wasn't accepted:** its card says "public domain" and names no dataset, so there is nothing to check the statement against.

## Kokoro

Kokoro passed all three measurements:

- **It repeats.** With PyTorch's seed set and its deterministic algorithms asked for, two processes gave the same samples for all 15 sentences, for both voices tried.
- **It loads with no network.** The weights and the voices were read from files, with `HF_HUB_OFFLINE=1` set while it spoke.
- **It is understood** as well as any voice here.
- **Its licence** is Apache-2.0 for the weights, and its model card says it was trained only on permissive and non-copyrighted audio, with the CC BY sets it used listed by name.

Its files, at revision `f3ff3571791e39611d31c381e3a41a3af07b4987` of `hexgrad/Kokoro-82M`:

| File | SHA-256 |
|---|---|
| `kokoro-v1_0.pth` | `496dba118d1a58f5f3db2efc88dbdc216e0483fc89fe6e47ee1f2c53f18ad1e4` (the hash its model card states) |
| `config.json` | `5abb01e2403b072bf03d04fde160443e209d7a0dad49a423be15196b9b43c17f` |
| `voices/af_heart.pt` | `0ab5709b8ffab19bfd849cd11d98f75b60af7733253ad0d67b12382a102cb4ff` |
| `voices/am_michael.pt` | `9a443b79a4b22489a5b0ab7c651a0bcd1a30bef675c28333f06971abbd47bd37` |

**It isn't a vidni voice yet,** for a reason that has nothing to do with the voice: vidni speaks through Piper, and Kokoro is another engine. Taking it in means a second engine among vidni's pinned tools, which is PyTorch and its text front end (a spaCy model among them), several gigabytes beside Piper's few hundred megabytes, and the same repeatability shown on a second machine. That is a piece of work of its own; this page is the evidence that it would be worth doing.

## Running it again

```sh
# A Piper voice: its model and its config.
python3 tools/voices_eval.py piper en_US-kristin-medium.onnx en_US-kristin-medium.onnx.json out/kristin

# Kokoro: a Python with the kokoro package and spaCy's small English model.
nix build --impure --no-link --print-out-paths --inputs-from . --expr \
  'let p = (builtins.getFlake "nixpkgs").legacyPackages.x86_64-linux;
   in p.python3.withPackages (q: [ q.kokoro q.soundfile q.spacy-models.en_core_web_sm ])'
<that path>/bin/python3 tools/voices_eval.py kokoro <folder with the files above> af_heart out/kokoro
```

Both run inside `nix develop`, which has Piper, whisper.cpp, its model and ffmpeg. Each writes `result.json` in its folder, with what was heard of every sentence.
