# A session in a terminal

A terminal scene: a transcript, drawn as a terminal's window. Narrated lines mark the transcript's lines they are about, and cameras frame them. Nothing is run, and nothing here needs a browser or the network.

```sh
vidni draft examples/terminal            # speaks the lines, draws the window, renders both cuts
vidni repro examples/terminal --draft    # two more builds, compared hash for hash
```

- `session.txt` is the transcript: what was typed and what was printed. A line that starts with the scene's `prompt` is a command.
- The session is the one in [nibli's README](https://github.com/dhilipsiva/nibli/blob/36739cca7060d0758af36e7d56b0623a930943a2/README.md#repl-usage), copied as it is printed there, at commit `36739cc`. The narration says only what the transcript shows.
- The window is drawn by vidni in DejaVu Sans Mono, one of the fonts it pins, so it is the same picture on every machine.
- A transcript that changes is drawn again on the next draft, and a ratified cut that shows it becomes `changed`.
