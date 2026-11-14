# The desktop app

`vidni-app` is vidni in a window. It does nothing the command line can't: every action is a `vidni` command, run for you, and the app's own help lists each with the command that does the same.

**Status:** the app is being built (milestone P7 of [TODO.md](../TODO.md)). This page describes what is there.

## Running it

From vidni's folder, in its Nix shell ([quickstart](quickstart.md)):

```sh
just app
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

## Accessibility

Every control has a name that a screen reader says, through AccessKit. The app's tests find the controls by those names, and fail if one has none.
