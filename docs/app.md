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

- **The projects.** The first screen starts a project, opens one by its folder, and lists the ones opened lately.
- **A project.** Its title, its folder, and each cut with where it stands, in `vidni status`'s own words.
- **Jobs.** A command that takes time runs as a job, one at a time. The job's panel shows the command as you would type it, a bar for the frames of a render, and everything the command says. When it ends, the project is read again.
- **Help.** The table above, drawn from the same list the app builds its commands from, so the two can't differ.

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
