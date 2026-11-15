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
| `flow` | image | `gates.mmd`, a Mermaid diagram |
| `zones` | image | `zones.png`, a review frame of the nibli example's short, as `vidni draft` writes it |
| `window` | image | `app.png`, the app's window on this project, from `just app-shot`, with the compositor's bar cut off |
| `keys` | overlay | drawn by vidni from its parts |
| the rest | cards | drawn by vidni from templates |

The short says nine of the walkthrough's lines, with an opening and a closing line of its own. Its lines on terminal and code scenes have a `camera_vertical` that frames some fifty columns, which is what a 9:16 cut can show at a size that reads.
