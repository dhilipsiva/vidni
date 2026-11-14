#!/usr/bin/env python3
"""Gives an installed Piper its `--seed` flag (nix/piper-seed.patch), for a
system where Piper comes from pip and not from vidni's flake: macOS and
Windows.

    python -m pip install piper-tts==1.8.0
    python packaging/seed-piper.py

vidni refuses a Piper that can't be seeded: without the flag a draft's
voice would be another take every time. This makes the same two changes
as the patch, to the installed `piper/__main__.py`, once.
"""
import importlib.util
import sys

FLAG = '''    parser.add_argument(
        "--seed",
        type=int,
        help="Seed for the voice's sampling noise, so that a run repeats",
    )
'''
SEED = '''    if args.seed is not None:
        # The noise is drawn inside the ONNX graph, from generators that
        # onnxruntime seeds when the voice's session is created.
        import onnxruntime

        onnxruntime.set_seed(args.seed)
'''
BEFORE_FLAG = "    args, unknown_args = parser.parse_known_args()\n"
BEFORE_SEED = "    voice = PiperVoice.load(model_path, use_cuda=args.cuda)\n"


def main():
    spec = importlib.util.find_spec("piper")
    if spec is None or not spec.submodule_search_locations:
        sys.exit("piper isn't installed: python -m pip install piper-tts==1.8.0")
    path = list(spec.submodule_search_locations)[0] + "/__main__.py"
    with open(path, encoding="utf-8") as f:
        text = f.read()
    if '"--seed"' in text:
        print(f"{path}: has --seed already")
        return
    for anchor in (BEFORE_FLAG, BEFORE_SEED):
        if text.count(anchor) != 1:
            sys.exit(f"{path} isn't the piper-tts 1.8.0 this was written for: {anchor.strip()!r} isn't in it once")
    text = text.replace(BEFORE_FLAG, FLAG + BEFORE_FLAG).replace(BEFORE_SEED, SEED + BEFORE_SEED)
    with open(path, "w", encoding="utf-8") as f:
        f.write(text)
    print(f"{path}: --seed added")


if __name__ == "__main__":
    main()
