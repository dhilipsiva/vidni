# vidni, shown by vidni

A walkthrough of vidni of about two and a half minutes, and a short of under a minute from the same script. It was made with vidni, to try vidni on itself: [DOGFOOD.md](DOGFOOD.md) lists what that found.

```sh
vidni draft examples/vidni-demo      # build/draft/long.mp4 and short.mp4, in the draft voice
```

Both cuts are ratified, and neither has takes yet. To finish it in your own voice:

```sh
vidni record examples/vidni-demo
vidni final examples/vidni-demo
vidni approve examples/vidni-demo
```

## What it shows, and where each picture comes from

| Scene | Kind | From |
|---|---|---|
| `script`, `scenes` | code | `examples/hello/vidni.toml` at the commit of v0.1.0 |
| `first`, `refused`, `sheet`, `repro` | terminal | `sessions/*.txt`: what the commands printed on a new project, `my-video`, in a scratch folder on 2026-11-15. Only the lines that count frames were left out |
| `first-narrow`, `refused-narrow`, `sheet-narrow`, `repro-narrow` | terminal | the same sessions in a window 46 columns wide, which the short swaps in |
| `flow` | image | `gates.mmd`, a Mermaid diagram |
| `zones` | image | `zones.png`, a review frame of the nibli example's short, as `vidni draft` writes it |
| `window` | image | `app.png`, the app's window on this project, from `just app-shot` |
| `keys`, `keys-two` | overlay | drawn by vidni from their parts: three chips for the walkthrough, two for the short |
| the rest | cards | drawn by vidni from templates |

The short says nine of the walkthrough's lines, with an opening and a closing line of its own. A 9:16 cut can show some fifty columns at a size that reads. So the short's `swap` gives it each terminal session in a narrower window, and its line on the code scene has a `camera_vertical` that frames the first 68 columns.
