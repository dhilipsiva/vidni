# Fixtures for the listening check

- `spoken.wav`: "Language models are good at reading prose.", spoken by the draft voice piper:en_US-joe-medium (CC0) at the seed the nibli example's `voice.lock` keeps for line `neurosymbolic-2`. `vidni draft examples/nibli-syllogism` makes it again.
- `closeness.json`: for each of the reference's 57 lines, what its synthetic voice was given, what whisper heard, and the score `reference/tts.py` gave the pair, with the words tts.py treats as heard alike. The Rust port must give the same scores.
