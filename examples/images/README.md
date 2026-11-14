# Pictures under the camera

An SVG diagram and a PNG chart as scenes, with cards around them. Each picture names its parts by their rectangles, and cameras frame those parts, as on a web scene. Nothing here needs a browser or the network.

```sh
vidni draft examples/images            # speaks the lines, reads the pictures, renders both cuts
vidni repro examples/images --draft    # two more builds, compared hash for hash
```

- `diagram.svg` is drawn by vidni itself (resvg), at twice its size, with its text in IBM Plex Sans from the fonts vidni pins.
- `chart.png` is used as it is.
- A picture that changes is read again on the next draft, and a ratified cut that shows it becomes `changed`.
