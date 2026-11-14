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
| Use take `<n>` | `vidni takes <project> --line <line> --pick <n>` | Makes another raw take the line's kept take, trimmed where vidni finds the words |
| Trim | `vidni takes <project> --line <line> --trim <from>:<to>` | Cuts the line's kept take from other seconds of its raw take |
| Exports | `vidni exports <project> --cut <cut>` | Lists a cut's platforms, what its final build's QA and its last export found, and its upload folders |
| Tick a platform | `vidni exports <project> --cut <cut> --add <platform>` | Makes the cut for the platform as well: its next final build exports it there |
| Untick a platform | `vidni exports <project> --cut <cut> --remove <platform>` | Makes the cut for the platform no longer: its next final build removes that folder |

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

## The recording studio

The **Record** tab records your takes, one line at a time. It is `vidni record` in a window: the same recorder, the same folders, the same records beside each take. A session can start in one and go on in the other.

- **The prompter.** The line to read, in large type, with its ID, its section, the cuts it is in, and how long the draft voice takes over it. It opens on the first line that has no take of its present words. Previous line, Next line and Next line to record move it, and so does a click on any line in the list at the bottom.
- **Record a take.** While you read, a meter shows the level, a waveform grows, and a timer runs beside the draft voice's pace. Stop ends the take; Cancel ends it and saves nothing.
- **The new take** is trimmed to your words, measured, and held beside the line's kept take. Play A plays the kept take and Play B the new one, so the two can be compared. Keep makes the new take the line's take and goes on to the next line with none. Record again and Discard leave the line as it was. A take with no speech in it, or one too short to be a take, is refused in the recorder's words.
- **The line's takes.** Every raw take of the line's present words, with its length, its level, and when and from what it was recorded. Play take `<n>` plays one. Use take `<n>` makes it the kept take.
- **Another trim.** Under the takes, the kept take's raw take is drawn with the kept part marked. From and To are its ends in seconds of the raw take; Trim cuts it there, and Trim where vidni finds the words puts it back.
- **Room tone.** Record room tone records three seconds of the room with nobody speaking, and keeps the middle as the sound of the project's pauses. Its level is shown; a room louder than -55 dB is said to be noisy.
- **Nothing is left half done.** A recording is cancelled, and a take that waits to be kept is discarded, when you leave the tab or close the app; a take that waits is discarded when you go to another line too. A cancelled recording leaves no file. A discarded take's raw take stays in the list, as does what was recorded before an input failed.
- **Record in one place at a time.** The terminal and the studio hold a new take in the same place while it waits to be kept, so a line under review in one shouldn't be recorded in the other. Raw takes and kept takes are safe either way.
- **The microphone** is the one `VIDNI_MIC` names, or the system's default input; takes play through the output `VIDNI_SPEAKER` names, or the default one. These are `vidni record`'s settings.

What the terminal has and the studio hasn't yet: the listening check of a new take against the script, the mic check's reading of your level (`vidni record --test` does both), and the warning when a session is louder or quieter than the one before.

## The export panel

The **Export** tab is where a cut's platforms are chosen, and where its upload folders are.

- **A box for each platform** that takes the cut's shape. Ticking one writes the cut's `platforms` in the project's file, and leaves the rest of the file as it is. Nothing is built by a tick.
- **The next final build** (Final `<cut>`, here or on the Cuts tab) makes the folder `out/<platform>/<cut>/` for each ticked platform, and removes the folder of one that was unticked. A cut's platforms aren't part of what it says, so it stays ratified. They do decide how loud the cut is and what its layout keeps clear of, so the cut is built again; where they decide the same, the build is the same bytes, and only the platform's folder changes.
- **What was found.** Under each cut: what QA measured of each picture and anything it found wrong; each platform's folder, or why the cut wasn't exported for it; and what an upload wouldn't be at its best with. These are `vidni exports`'s words.
- **Open `out/<platform>/<cut>`** opens that folder in the system's file manager (`xdg-open`), with the video, its captions, `metadata.md` and its images, ready to upload by hand.

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
