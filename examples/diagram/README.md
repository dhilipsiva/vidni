# A diagram from text

A Mermaid diagram as an image scene. The diagram is the text in `gates.mmd`; vidni has Mermaid's command-line program draw it, in the browser and the fonts it pins, and frames the picture with the virtual camera like any other.

```sh
vidni draft examples/diagram
```

- It needs `mmdc`, Mermaid's program, which vidni's flake provides; `VIDNI_MERMAID` names another. The portable bundle doesn't carry it.
- `scale = 2` draws the diagram at twice its size, so that it stays sharp when the camera moves in. A target's `rect` is in the diagram's own pixels, before the scale.
- `color_scheme = "light"` draws it in Mermaid's light theme, on white. The default is the dark one.
- The diagram is a capture: drawn once, frozen when its cut is ratified, and drawn again when the file changes.
