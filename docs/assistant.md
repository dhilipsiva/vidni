# The assistant protocol

vidni needs no assistant: every step is a command, and a person can do all of it. An assistant, a language model of any make, may help with the writing: drafting lines and scenes, and applying the changes a person asked for. This page is the whole of what it may do. It is written for the assistant to read, and for the person to hold it to.

The rule under all of it: **the assistant writes the script; the person decides what is true, what is said in their voice, and what is released.**

## A session

1. **A person starts the session,** in the project's folder:

   ```sh
   vidni assistant
   ```

   It notes every file of the project as it is, in `build/assistant.json`.
2. **The assistant runs every vidni command with `VIDNI_ASSISTANT=1`** in its environment.
3. **After every change, the assistant runs `vidni validate`,** and goes on only when it passes.

With `VIDNI_ASSISTANT` set, vidni holds the assistant to this page: commands that are a person's refuse, and `vidni validate` fails when anything was changed that an assistant may not change.

## What the assistant may change

Two files, and no others:

| File | What it holds |
|---|---|
| `vidni.toml` | The script, the scenes, the cuts ([the spec](spec.md)) |
| `notes.md` | The person's review notes, one section a line; the assistant takes a note out once it has applied it |

- **Every other file is the person's or vidni's.** The assistant doesn't edit, add or remove a page, a picture, a transcript, a theme, a lock or a record. When a scene needs a file that isn't there, it asks the person for it.
- **A line's `id` never changes.** The person's takes are filed under it. A line that is rewritten keeps its ID; a line that is dropped takes its ID with it.
- **`source` says where a fact comes from.** A line that states a fact names the place in the person's own material that says so. The assistant states nothing it wasn't given.
- **Comments and layout stay.** The file is edited in place, not written out afresh, and `vidni fmt` lays it out.

## What the assistant runs

| Command | For |
|---|---|
| `vidni new` | Starting a project, when asked to |
| `vidni validate`, `vidni fmt` | After every change |
| `vidni draft` | Speaking and rendering a draft for the person to watch |
| `vidni status`, `vidni changes`, `vidni frozen`, `vidni exports` | Seeing where things stand |
| `vidni notes` | Reading the person's notes, and taking out the ones it applied |
| `vidni plan`, `vidni takes` (listing), `vidni hear`, `vidni where`, `vidni published` (listing), `vidni platforms`, `vidni voices` | Reading |
| `vidni final` | Building what the person ratified and recorded, when asked to; it refuses anything else by itself |

## What is the person's

These refuse when `VIDNI_ASSISTANT` is set. The assistant asks the person to run them, and says why.

| Command | Why it is the person's |
|---|---|
| `vidni ratify`, `vidni unratify` | Ratifying says "this is what I mean to say and show". |
| `vidni record`, `vidni takes --pick`, `--trim` | The narration is the person's own voice. |
| `vidni approve` | Approving says "this is what I release". |
| `vidni published` (recording an upload), `vidni upload` | Only the person uploads, and only they know what they uploaded. |
| `vidni backup`, `vidni restore`, `vidni gc`, `vidni setup` | They move and delete what can't be made again. |

- **A changed cut stays changed.** When the assistant's edit regresses a ratified cut, `vidni status` says so, and that is the right outcome. The assistant doesn't work round it: it tells the person which cut changed and why, and the person ratifies again or doesn't.
- **The assistant never uploads.** vidni uploads nothing by itself. `vidni upload` is the person's, with a confirmation for each upload, and it refuses for an assistant.
- **The assistant doesn't unset `VIDNI_ASSISTANT`,** start a session over, or edit `build/assistant.json`. A session started again by the assistant is refused.

## What `vidni validate` checks for an assistant

Everything it checks for a person, and then the session:

```console
$ VIDNI_ASSISTANT=1 vidni validate
vidni.toml: ok: 12 lines in 3 sections, 4 scenes, 2 cuts
vidni: page.html was changed: an assistant changes vidni.toml and notes.md only
vidni: ratified.lock was changed: only `vidni ratify`, run by a person, writes it
2 changes that the assistant protocol forbids (docs/assistant.md)
```

- **Held to the session's start:** every file in the project folder but `vidni.toml` and `notes.md`. One that is new, gone or different fails the check, and is named.
- **Not held:** `build/`, `out/` and `takes/`, which commands fill and a person's recorder fills.
- **A command's own file is noted again by that command.** `vidni draft` writes `voice.lock`; a person who ratifies or approves while the assistant works writes `ratified.lock` or `release.json`. Those don't fail the check: the command that wrote the file says so to the session.
- **It exits 1** when the protocol was broken, so a script or an agent's own loop stops there.

The check can be got round by an assistant that sets out to: it is a guard against mistakes and a statement of the rules, not a sandbox. For a hard limit, run the assistant where it can write only those two files.

## A reference implementation

[`assistants/claude-code/vidni/SKILL.md`](../assistants/claude-code/vidni/SKILL.md) is a skill for Claude Code that follows this protocol: copy its folder into a project's `.claude/skills/`, or your own `~/.claude/skills/`. It drafts lines and card scenes from a topic and a facts file, and applies the notes in `notes.md`. Another assistant needs the same instructions in its own form; nothing in vidni is specific to one.

## A round of work

1. The person runs `vidni assistant`, and says what they want: a topic and their material, or "apply my notes".
2. The assistant edits `vidni.toml`, runs `vidni fmt` and `vidni validate`, and then `vidni draft`.
3. The person watches the draft, and writes what to change with `vidni notes --line <id> --set "…"`, or in the app.
4. The assistant reads `vidni notes`, changes those lines, takes each applied note out, validates and drafts again, and says what it changed (`vidni changes` lists it).
5. When the draft is right, the person ratifies, records, builds and approves.
