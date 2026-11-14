# The desktop app

`vidni-app` is vidni in a window. It does nothing the command line can't: every action is a `vidni` command, run for you, and the app's own help lists each with the command that does the same.

**Status:** the app is being built (milestone P7 of [TODO.md](../TODO.md)). This page describes what is there.

## Running it

From vidni's folder, in its Nix shell ([quickstart](quickstart.md)):

```sh
just app                     # opens on the list of projects
just app examples/hello      # opens a project
just app examples/hello --preview   # on its preview; --script opens its script
```

The app keeps the list of projects you opened in `recent`, in its folder of settings: `$VIDNI_CONFIG`, or `vidni` under `$XDG_CONFIG_HOME`, or `~/.config/vidni`.

## What it does

| In the app | On the command line | What it does |
|---|---|---|
| New project | `vidni new <name> --in <folder>` | Starts a project that drafts as it is |
| Open project | `vidni status <project>` | Reads a project, and says where each cut stands |
| Check | `vidni validate <project>` | Checks the project's file, and lists each problem at its line |
| Draft every cut | `vidni draft <project>` | Speaks the lines in the draft voice, takes the captures, and renders every cut |
| Draft `<cut>` | `vidni draft <project> --cut <cut>` | Drafts one cut |
| Save note | `vidni notes <project> --line <line> --set <note>` | Writes what to change about a line into notes.md |
| What changed | `vidni changes <project>` | Lists the lines that changed since the last build, and the cuts that changed since they were ratified |
| Ratify every cut | `vidni ratify <project>` | Freezes what every cut shows and says, with the hashes of its captures |
| Ratify `<cut>` | `vidni ratify <project> --cut <cut>` | Freezes one cut |
| Unratify `<cut>` | `vidni unratify <project> --cut <cut>` | Unfreezes one cut, and keeps its captures |
| Frozen `<cut>` | `vidni frozen <project> --cut <cut>` | Lists what a ratified cut froze: its lines, scenes and capture files, each with its hash |
| Final build of every cut | `vidni final <project>` | Builds every ratified cut from your takes into out/, and checks it |
| Final `<cut>` | `vidni final <project> --cut <cut>` | Builds one cut from your takes |
| Approve every cut | `vidni approve <project>` | Records every cut's final build in release.json, as a release with its hashes |
| Approve `<cut>` | `vidni approve <project> --cut <cut>` | Releases one cut's final build |

- **The projects.** The first screen starts a project, opens one by its folder, and lists the ones opened lately.
- **A project.** Its title, its folder, and each cut with where it stands, in `vidni status`'s own words.
- **Jobs.** A command that takes time runs as a job, one at a time. The job's panel shows the command as you would type it, a bar for the frames of a render, and everything the command says. When it ends, the project is read again.
- **Help.** The table above, drawn from the same list the app builds its commands from, so the two can't differ.

## The gates

The **Cuts** tab has the three gates a cut goes through, for every cut at once and beside each cut: ratify, the final build, and approval.

- **A gate is its command.** A button runs `vidni ratify`, `vidni final` or `vidni approve` as a job, so a gate allows what the command allows and refuses what it refuses, in the same words: a cut that isn't ratified, a line with no take, a cut that changed since it was ratified. The refusal is in the job's log, under the command that was run, and the cut's row says where it stands afterwards.
- **Frozen `<cut>`** lists what a ratified cut froze, under the table: the hash of its content, each of its lines and scenes with its hash, and each capture file and frames file with its SHA-256. These are the hashes in `ratified.lock`. The list is read again when the lock changes.
- **Unratify `<cut>`** makes a cut a draft again, and keeps its captures.
- **Approving is for good.** A release is never rewritten, in the app as on the command line: approving a later build of a cut makes its next release.

## The script editor

An open project has two tabs: **Cuts**, and **Script**, which is its file in an editor.

- **Forms.** The title; every line under its section, each shown with its ID; and the scenes. Pick a line to open its form: what it shows, what is said when that differs, its section, scene and cameras, and the source of its facts. Apply saves the fields that changed. A line can be moved up or down, removed, or followed by a new one.
- **Source.** The file's own text, to edit as text. Save source saves it when it is a valid project.
- **Nothing invalid is saved.** An edit that would leave the file invalid, in a form or in the source, isn't written: what is wrong is listed under **Not saved**, each problem at its line and column, in `vidni validate`'s words. The file stays as it was.
- **Comments are kept.** A form changes the value it names and leaves every other byte of the file where it was: every comment, blank line and key, and the file's line endings. A comment written above a line belongs to that line, and moves with it.
- **What would lose a comment is refused,** and said: clearing a key that has a comment above or beside it, and removing a line that has comments of its own. Those are done in the source view, where the comment is in front of you.
- **Editing by hand at the same time is safe.** Every edit is made to the file as it is on disk at that moment, so a change made in another editor in between is kept, and shows in the app at once. What you have typed in a form and not applied stays typed. Text typed in the source view is of the file as it was when it was read: if the file has changed since, Save source refuses, and Read from disk takes the file in again.
- **An empty field** is a key that isn't in the file: clearing "Said, when it differs" removes `say`. A scene's key is taken out in the source view.
- **A line with tables of its own,** written as `[[lines.do]]` under it, is edited in place by the forms, and added after, moved or removed only in the source view: the forms don't move a line's tables with it.

The forms cover what a script is mostly made of. Everything else in a project file (targets, cameras, cuts, the publishing table) is edited in the source view for now.

## Notes, and what changed

Both are in the **Script** tab.

- **A note on a line.** Pick a line, write what to change about it under **Note**, and press Save note. The line's row then ends in `(note)`. An emptied note, saved, is taken away.
- **Notes are a file.** They are in `notes.md`, in the project folder, each under a heading that is its line's ID. So they are there the next time the app is started, they go into git with the script, and they can be written by hand or with `vidni notes`; the app shows such a change at once. A note is no part of the video: writing one changes no cut.
- **What changed** lists, in `vidni changes`'s words, each line that is new, gone or changed since the last build, with the keys that changed, and for each ratified cut what has changed since it was ratified. While it is open it follows the file, so an edit shows in it as it is made.

## The preview

The **Preview** tab plays a built cut, and goes to any line of it.

- **Which picture.** Pick the cut, and its draft or its final build. A build that made two pictures, one with its captions drawn in and one without, has a Captions box to switch between them, at the same place in the cut. A draft is one picture, so the box is greyed there.
- **The lines.** Under the picture, every line with its ID and the time it starts. A click on one goes to the frame its first caption starts on. Previous line and Next line step through them.
- **The position.** A slider over the whole cut, and above it, on the same scale, a tick where each line starts. A click or a drag on the ticks goes there too, and the pointer over them names the line.
- **Contact sheet.** A picture of every line, from halfway through it. A click on one goes to its line.
- **Play** plays from where you are, with the cut's sound through `ffplay`.

Frames are decoded by the same ffmpeg that built the cut, on a thread of the player's own, so the window never waits for one. While you drag the position, the picture follows at the decoder's pace, and only the newest place asked for is decoded next. On the five-minute nibli example at 1080p, a frame comes up in about 0.13 s wherever it is in the cut, and a drag across the whole cut in five seconds shows 37 pictures on the way.

A cut that is built again while it is open, in the app or on the command line, is opened afresh at the place the player was. Playing stops when you leave the tab.

## Accessibility

Every control has a name that a screen reader says, through AccessKit. The app's tests find the controls by those names, and fail if one has none.
