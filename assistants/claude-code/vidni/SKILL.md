---
name: vidni
description: Draft and revise a narrated tutorial video with vidni. Use when asked to turn a topic and a facts file into a vidni project's script and scenes, or to apply the review notes in a vidni project's notes.md. Edits only vidni.toml and notes.md, and leaves ratifying, recording and approving to the person.
---

# vidni: drafting and revising a video's script

vidni makes a narrated video from one text file, `vidni.toml`. You write and revise that file. The person decides what is true, records the narration in their own voice, and releases. This skill is an implementation of vidni's assistant protocol; when anything here is unclear, `docs/assistant.md` in vidni's repository decides.

## The rules, which vidni checks

- **Change two files only: `vidni.toml` and `notes.md`.** Never edit, add or delete any other file in the project: not a page, a picture, a lock, a record, or anything under `build/`, `out/` or `takes/`. If a scene needs a file that isn't there, ask the person for it.
- **Run every vidni command with `VIDNI_ASSISTANT=1`.** Export it once at the start: `export VIDNI_ASSISTANT=1`. Never unset it.
- **Run `vidni validate <project>` after every change,** and go on only when it exits 0. It also fails when you changed something you may not.
- **Never run** `vidni ratify`, `unratify`, `record`, `approve`, `backup`, `restore`, `gc` or `setup`, nor `vidni takes --pick/--trim`, nor `vidni published --cut`. They refuse for you. When one is needed, say which and why, and stop.
- **Never upload anything, anywhere.**
- **State only what the facts file says.** Every line that states a fact gets `source = "<facts file>: <where>"`. If the person's material doesn't say it, don't say it; ask.
- **A line's `id` never changes** once it has a take, and a line that has one isn't dropped without asking. `vidni takes <project>` shows which lines have takes. In a new project none has: the starter's lines are placeholders, and you may reuse their IDs or drop them.
- **When the request and the facts disagree, the facts win.** Follow the facts, and say in your report what you did differently and why.
- If `vidni validate` says there is no assistant's session, stop and ask the person to run `vidni assistant <project>`. Don't run it yourself over an existing session.

## Before you write

1. `export VIDNI_ASSISTANT=1`
2. `vidni validate <project>`: it must pass before you touch anything.
3. Read `<project>/vidni.toml` as it is, comments included: the starter explains its own keys. Read the facts file the person named.
4. `vidni status <project>` and `vidni notes <project>`: where the cuts stand, and what the person asked for.

## Drafting from a topic and a facts file

Write for the ear. One idea a line, in short sentences a person can say in one breath (about 8 to 20 words). Open with what the viewer will be able to do; end with one line that sums up.

Edit `vidni.toml` in place: keep its comments and its `[meta]`, `[voice]` and `[[cuts]]` tables, and replace the starter's sections, lines and scenes with yours.

```toml
[sections.intro]
label = "what this is"

[[lines]]
id = "intro-1"            # <section>-<n>, lowercase, hyphens; never reused or renamed
section = "intro"
show = "What appears in the captions, and is said."
source = "facts.md: Overview"   # where the fact is from, when the line states one
scene = "open"            # leave out to use [meta] default_scene
```

Scenes you can write without any other file are **cards**:

```toml
[scenes.open]
kind = "card"
template = "title"        # title | section | bullets | quote | end
title = "The video's title"
subtitle = "one line under it"

[scenes.points]
kind = "card"
template = "bullets"
title = "Three things"
items = ["First", "Second", "Third"]
```

- `title`, `section` and `end` take `title` and `subtitle`; `bullets` takes `title` and `items`; `quote` takes `quote` and `by`.
- Use a `title` card to open, `section` cards between parts, `bullets` where the narration lists things, and an `end` card to close.
- Set `[meta] title` and `default_scene`, which must name a scene you define; a line with no `scene` shows it. Define no scene that no line shows.
- **A card's words are facts too.** Titles, subtitles and bullet items come from the facts file like the narration, though a scene has no `source`.
- **A section's `label` is on screen** in a corner while its lines play, and becomes a chapter's name. YouTube shows chapters only when each lasts 10 seconds or more, and `vidni draft` warns about shorter ones. In a short video, use fewer sections (two or three) and let a `section` card open a part without starting a new section; if a warning remains, report it and leave it.
- **A file you no longer use stays.** When you remove the starter's scene, its `page.html` is left where it is: deleting it isn't yours to do.
- A scene that shows a web page, a picture, a terminal session or code needs a file or an address from the person. Use one only when they gave it; `docs/spec.md` has the keys.
- `say = "…"` on a line gives the spoken words when they differ from `show` (a symbol read aloud, a file name such as `ratified.lock` said as "ratified dot lock"). `show` is plain text: no backticks or other markup. A word the draft voice gets wrong everywhere goes in `[voice] respell` once, as the starter does for "vidni".

Then, in this order:

```sh
vidni fmt <project>          # the canonical layout
vidni validate <project>     # fix what it reports, at the line and column it gives
vidni draft <project>        # speaks the lines in the draft voice and renders the cuts
```

`vidni draft` exits 0 when the cuts were rendered; what it prints as `warning:` is advice to pass on, not a failure. Run `vidni validate <project>` once more after it. You can't watch the video: the person does.

Tell the person where the drafts are (`<project>/build/draft/<cut>.mp4`), how many lines and sections you wrote, the warnings, and anything in the topic the facts file didn't cover.

## Applying review notes

The person watches a draft and leaves notes, one per line, in `notes.md` (through `vidni notes --line <id> --set "…"`, or the app).

1. `vidni notes <project>` lists them: `<line-id>: <what to change>`.
2. For each note, change that line in `vidni.toml` as asked, and nothing else. Keep the line's `id`.
   - A note about how a line sounds ("don't read the file name aloud") changes what is said: when the captions should keep the words, set `say`; when the note wants them gone, change `show`.
   - If a note asks for something the facts file doesn't support, don't invent it. Leave the note as the person wrote it, and tell them what the facts would need to say.
3. `vidni fmt <project>` and `vidni validate <project>`.
4. `vidni changes <project>`, before you draft: it names each line that differs from the last build. Keep that list for your report. (After a draft it compares with the new build, and lists nothing.)
5. Take out each note you applied, with the command and not by hand: `vidni notes <project> --line <id> --set ""`.
6. `vidni draft <project>`, then `vidni validate <project>`.
7. Report the lines that changed, and the notes you left and why.

If `vidni status` shows a cut as `changed` after your edit, that is correct: the person ratified other words. Say which cut, and that it needs `vidni ratify` from them. Don't try to make the status go away.

## When something fails

- **`vidni validate` reports a problem:** it names the line and column. Fix the file; don't work round the check.
- **A command says it is a person's:** stop and ask the person to run it.
- **`vidni draft` can't find a voice or a tool:** report the message as it is. Setting vidni up is the person's.
