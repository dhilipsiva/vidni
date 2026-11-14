# Code at a commit

A code scene: an excerpt of a source file, with its syntax in colour and its lines numbered as the file numbers them. Narrated lines mark the lines they are about, and cameras frame them.

```sh
vidni draft examples/code
```

- The file is `crates/vidni-core/src/notes.rs` of vidni's own repository, read at commit `897a8bb` with `git show`. So the example needs git, and vidni's history: a copy of this folder without the repository around it can't draw the scene.
- `rev` is a commit's hash, never a branch: the excerpt is the same however the file changes later.
- Without `rev`, a code scene shows a file of the project folder as it is now, and is drawn again when the file changes.
- The window is drawn by vidni in DejaVu Sans Mono, one of the fonts it pins, with the colours of syntect's `base16-ocean.dark`.
