# nibli explainer video: the Syllogism walkthrough

> **vidni's reference pipeline.** This Python and Node pipeline made the first vidni video. It moved here from the maintainer's workspace with its history (P1-VIDNI-03). It stays the reference implementation until the Rust port matches its golden outputs (`just golden`, `just parity`). Run every command below from this folder. The creator's takes live in [`../examples/nibli-syllogism/recordings/`](../examples/nibli-syllogism/), out of git.

A five-minute narrated video of the live nibli playground. It covers:

- symbolic reasoning
- theorem proving
- the neuro-symbolic architecture
- nibli itself
- the Syllogism example: how the claim `eats(Adam).` is reasoned over, and how its proof trace is built

It renders as two cuts from one script:

- `out/nibli-syllogism-16x9.mp4` (1920×1080)
- `out/nibli-syllogism-9x16.mp4` (1080×1920)
- `out/nibli-syllogism.en.srt`, the captions as a file for upload

The videos are rebuilt from source and stay out of git.

## Build

```sh
./build.sh                  # voices → tts → capture → compose → qa
./build.sh compose 16x9     # one cut
./build.sh compose --remux  # new narration into both cuts, video not re-encoded
```

`tests/` specifies this pipeline's behaviour for the Rust port: cue splitting, timeline assembly, camera framing, the recorder's terminal scenarios, and keeping the variants apart. Run it from vidni's root with `just test-reference`. It takes under a minute, with no microphone and no network.

Narration is normalized in two loudnorm passes. One pass measured −18.1 LUFS on this speech-with-pauses track, below the −16 target.

**Requirements:**
- Nix with flakes. `flake.nix` pins the same nixpkgs revisions as the NixOS config and provides ffmpeg-full, Piper 1.8, whisper-cpp, espeak-ng and Python with Pillow.
- Node and Chrome on `PATH`.
- A Playwright module, which defaults to `nibli-learn/node_modules/playwright`. Override it with `PLAYWRIGHT_MODULE`. Override the browser with `BROWSER_EXECUTABLE`.
- Network access, for the first voice and model download (checked by sha256) and for the live playground.

`build.sh` unsets `LD_LIBRARY_PATH`, because this desktop's session value makes nix-store binaries fail with a GLIBC error.

## How it works

| Step | File | What it does |
|---|---|---|
| script | `segments.json` | Every narrated sentence: `show` (captions) and optional `say` (spelled for the voice), plus what to show. `script.md` is generated from it. |
| voice | `tts.py` | Lints the wording, then runs Piper per sentence. Piper samples differently each run, so each sentence gets up to four takes, and the one whisper transcribes closest to the script is kept. Writes `build/timeline.json`, `build/narration.wav` and the SRT. Your own voice gets its own `-voice` files, so building one variant never changes the other. |
| capture | `capture.mjs` | Drives the live playground at 1280×720 CSS px with a device scale factor of 3, giving 3840×2160 stills. It runs each sentence's actions, draws ember boxes around what is being discussed, and saves one still per sentence. It renders the QUINE cards in `cards/card.html`. **It stops if the playground's text differs from the strings pinned in `nibli-wasm/src/lib.rs` (`syllogism_playground_bytes_are_verbatim`).** |
| compose | `compose.py` | A virtual camera eases between crops of the stills, with crossfades, section labels, overlays and burned-in captions. The 9:16 cut frames tighter crops between a title band and a caption band, because the playground's two-column grid has no portrait layout. |
| check | `qa.py` | Checks size, 30 fps, audio, duration against the timeline, loudness (−16 LUFS ± 1.5) and the SRT cue count. Extracts one frame per segment for each cut into `build/qa/`. |

**Sentence fields in `segments.json`:**

| Field | Meaning | Values |
|---|---|---|
| `do` | Actions to run first | `{"tab": "Source" \| "nibli KR" \| "Back-translation"}`, `{"preset": "2"}`, `{"expand" or "collapse": "given-detail"}`, `{"scroll": px}`, `{"scrollTo": key}` |
| `hl` | Highlight keys | `kr`, `kr-line:N`, `bt-line:N`, `source-text`, `formalize`, `kb-status`, `examples`, `brand`, `footer`, `query-select`, `query-gloss`, `why`, `tree`, `row:eats`, `row:animal`, `row:given`, `row:detail-given`, `rows:roles`, `rows:proved`, `row:failed`, `verdict` |
| `cam` | 16:9 framing | `full`, `app`, `kb`, `proof`, `query`, `log`, `top`, `left`, `right`, `hl` |
| `vcam` | 9:16 framing override | The same values as `cam`. Without it, the vertical cut follows the highlights when they're narrow enough. |
| `card` | Use a card instead of the playground | `intro`, `end` |
| `overlay` | Draw an overlay | `verdicts`, `firewall` |

## Recording your own narration

`record.py` is a prompter and recorder for the 57 lines. Run it in a terminal, because it's interactive:

```sh
./build.sh record --test        # mic and room check: levels, noise, playback
./build.sh record               # every line not recorded yet; quit and resume any time
./build.sh record --only 12,23  # redo lines 012 and 023
./build.sh own                  # build out/nibli-syllogism-voice-{16x9,9x16}.mp4
```

**Recording a line.** Single keys; no Enter needed.
- Each line shows its section, the line before it, and a pace guide: how long the synthetic voice took.
- Press space, read when the level meter moves, then press space again. Esc cancels the take.
- If a take is too short or has no speech, it's discarded with a message and you stay on the same line.
- A second keypress straight after you start or stop is ignored.
- A take you keep is trimmed: the keypresses and the silence around the words go.
- Its levels are checked and it's transcribed, so a misread line shows up straight away.

**After each take:**
- Enter keeps it and moves to the next missing line.
- p plays it.
- r records it again.
- Esc discards it.

**On a line's ready screen,** ← and → move between lines, and p plays the take saved for that line.

**If something fails,** you get a message instead of a crash, and every take you kept stays saved. Unexpected errors go to `record-error.log` in the takes folder.

**Where takes go:**
- The nibli example's takes live in vidni's take store, `../examples/nibli-syllogism/takes/`, filed by line ID and spoken words (P2-VIDNI-03). `./build.sh own` lays them out as numbered files with `vidni takes --link`.
- This recorder writes the older numbered layout, into the folder `RECORDINGS_DIR` names: kept takes as `NNN.wav` (48 kHz, 16-bit mono), and every raw take in `raw/`.
- Takes are kept out of git, so back them up. `examples/nibli-syllogism/assets.lock` lists their sha256.

**Choosing a microphone.** `wpctl status` lists the microphones. To use one, either make it the default with `wpctl set-default <id>` or set `MIC` to its node name.

**What adapts to your recordings:**
- Timing, captions and camera moves all follow your takes, so read at whatever pace is natural.
- `./build.sh own` writes `nibli-syllogism-voice-*` beside the synthetic cut and leaves the synthetic cut in place.
- It transcribes every take again and lists the lines that differ most from the script.

**Pronunciations:** "nibli" is said NIB-lee. The prompter shows how to read symbols: `#1` as "number one", RESOURCE_EXCEEDED as "resource exceeded", and the URL.

If you record with another app instead, save one 16-bit mono WAV per line, named `000.wav` to `056.wav` in script order. Then run `OWN_AUDIO_DIR=/path ./build.sh own`.

## Voice and licences

| Asset | Source | Licence | Role in the build |
|---|---|---|---|
| Narration voice `en_US-joe-medium` | Piper, from `rhasspy/piper-voices` | Training data from OHF-Voice/voice-datasets, CC0 | Speaks the narration |
| Whisper `ggml-base.en` | whisper.cpp | MIT | Chooses between takes and checks intelligibility; never heard in the video |
| IBM Plex, Space Grotesk | — | OFL | Fonts |

The voice was picked over ryan, whose data is CC BY-NC-SA, because this video promotes a book. john, norman and bryce are public domain. In a whisper intelligibility test, joe tied for best with bryce.

**sha256:**
- `en_US-joe-medium.onnx`: abf4b998e34045cbd00eaf050de2de620f81ded6eb8b85a24b022a1c2b3f08d2
- `en_US-joe-medium.onnx.json`: 7b5f0293c0bf5bda8e377be1696da84af798befeef31038e43beefc01fe858da
- `ggml-base.en.bin`: a03779c86df3323075f5e796cb2ce5029f00ec8869eee3fdfb897afe36c6d002

## Wording

**Lint.** The script follows `applying-nibli-research/review/canonical-facts.md` (B, E, G) and the book repo's wording rules. `tts.py` fails on these phrases:
- zero-hallucination, no hallucination, guaranteed
- machine-checked, verified engine
- high-performance, sub-millisecond
- published
- UNDETERMINED
- "compile" used of the LLM

**Narration rules:**
- The LLM step is "formalize".
- FALSE means "not derivable".
- The book is "scheduled for December 2026".
- Nothing is quoted from the manuscript; the publisher reserves its rights.

**"nibli" pronunciation.** "nibli" is said NIB-lee. espeak-ng phonemizes it as `n'Ibli`. "theorem prover" is respelled "theorem proover" for the voice; plain spelling was transcribed as "their improver".

**Known gap in the playground.** For `bird(Adam).`, the playground's proof tree shows FALSE with "No witness found". It does not show the closed-world note that the engine's `proof_text` carries: "[Note: FALSE is closed-world — not derivable from the KB, not a proof of the negation]". So the narration explains what FALSE means instead of pointing at that note.
