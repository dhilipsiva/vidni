//! Review notes and what changed (TODO.md, P7-VIDNI-04): a note written
//! in the app is in `notes.md` and is there when the app is opened again,
//! and what changed since the last build, and since a cut was ratified,
//! names the lines that changed.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use std::fs;
use std::path::{Path, PathBuf};
use vidni_app::actions::Action;
use vidni_app::app::App;
use vidni_app::jobs;
use vidni_core::ratify;

const FIRST: &str = "[intro-1]  Hello, and welcome.";

/// A new project in `dir`.
fn project(dir: &Path) -> PathBuf {
    assert_eq!(
        vidni_cli::run(&["new", "demo", "--in", dir.to_str().unwrap()]),
        0
    );
    dir.join("demo")
}

/// The app, started on `folder`, at its script.
fn start<'a>(folder: &Path) -> Harness<'a, App> {
    let mut app = App::new(None);
    app.open(folder);
    let mut harness = Harness::builder()
        .with_size([1100.0, 2400.0])
        .build_ui_state(|ui, app: &mut App| app.show(ui), app);
    harness.run();
    harness.get_by_label("Script").click();
    harness.run();
    harness
}

fn click(harness: &mut Harness<'_, App>, label: &str) {
    harness.get_by_label(label).click();
    harness.run();
}

fn fill(harness: &mut Harness<'_, App>, label: &str, text: &str) {
    harness.get_by_label(label).focus();
    harness.run();
    harness.key_combination_modifiers(egui::Modifiers::COMMAND, &[egui::Key::A]);
    harness.run();
    match text.is_empty() {
        true => harness.key_press(egui::Key::Backspace),
        false => harness.get_by_label(label).type_text(text),
    }
    harness.run();
}

/// What the field named `label` holds.
fn held(harness: &Harness<'_, App>, label: &str) -> String {
    harness.get_by_label(label).value().unwrap_or_default()
}

/// What "What changed" lists.
fn listed(harness: &Harness<'_, App>) -> Vec<String> {
    harness
        .state()
        .editor
        .as_ref()
        .unwrap()
        .changes
        .clone()
        .unwrap_or_default()
}

#[test]
fn a_note_written_in_the_app_is_there_when_it_is_opened_again() {
    let dir = tempfile::tempdir().unwrap();
    let folder = project(dir.path());
    let noted = format!("{FIRST}  (note)");
    {
        let mut app = start(&folder);
        // No line has a note yet, and there is no file of them.
        assert!(app.query_by_label(&noted).is_none());
        assert!(!folder.join("notes.md").exists());
        click(&mut app, FIRST);
        assert_eq!(held(&app, "Note"), "");
        fill(&mut app, "Note", "Say it slower.");
        click(&mut app, "Save note");
        // It is in notes.md, under the line's ID, and the line says it has one.
        let text = fs::read_to_string(folder.join("notes.md")).unwrap();
        assert!(text.contains("## intro-1\n\nSay it slower.\n"), "{text}");
        assert!(app.query_by_label(&noted).is_some());
        // The script itself is as it was: a note is no part of the video.
        assert_eq!(vidni_cli::run(&["validate", folder.to_str().unwrap()]), 0);
    }
    // The app is closed, and started again on the same folder.
    let mut app = start(&folder);
    assert!(app.query_by_label(&noted).is_some());
    assert!(app.query_by_label(FIRST).is_none());
    click(&mut app, &noted);
    assert_eq!(held(&app, "Note"), "Say it slower.");
    // The other line has none.
    click(&mut app, "[intro-2]  This video was made with vidni.");
    assert_eq!(held(&app, "Note"), "");

    // A note written by hand, or by `vidni notes`, shows in the open app.
    assert_eq!(
        vidni_cli::run(&[
            "notes",
            folder.to_str().unwrap(),
            "--line",
            "intro-2",
            "--set",
            "Name the tool sooner.",
        ]),
        0
    );
    app.run();
    assert!(
        app.query_by_label("[intro-2]  This video was made with vidni.  (note)")
            .is_some()
    );
    assert_eq!(held(&app, "Note"), "Name the tool sooner.");

    // An emptied note is taken out, and the first is kept.
    fill(&mut app, "Note", "");
    click(&mut app, "Save note");
    let text = fs::read_to_string(folder.join("notes.md")).unwrap();
    assert!(!text.contains("intro-2"), "{text}");
    assert!(text.contains("## intro-1\n\nSay it slower.\n"), "{text}");
    drop(app);
    let app = start(&folder);
    assert!(app.query_by_label(&noted).is_some());
    assert!(
        app.query_by_label("[intro-2]  This video was made with vidni.")
            .is_some()
    );
}

#[test]
fn what_changed_since_the_last_build_names_the_lines() {
    let dir = tempfile::tempdir().unwrap();
    let folder = project(dir.path());
    let mut app = start(&folder);
    // Before any build there is nothing to compare with, and it says so.
    click(&mut app, "What changed");
    assert_eq!(
        listed(&app)[0],
        "since the last build: there is no build yet to compare with"
    );
    click(&mut app, "Hide what changed");
    assert!(app.state().editor.as_ref().unwrap().changes.is_none());

    // What a build leaves behind: the project as it was built.
    let text = fs::read_to_string(folder.join("vidni.toml")).unwrap();
    let built = vidni_core::validate::check(&text, Some(&folder)).unwrap();
    ratify::write_resolved(&built, &folder).unwrap();
    click(&mut app, "What changed");
    assert_eq!(listed(&app)[0], "since the last build: nothing");

    // A line's words change in the app: the list names the line, and what
    // of it changed. The line that wasn't touched isn't named.
    click(&mut app, FIRST);
    fill(&mut app, "Shown", "Hello, everyone.");
    click(&mut app, "Apply to intro-1");
    let list = listed(&app);
    assert_eq!(list[0], "since the last build:");
    assert_eq!(list[1], "  line intro-1 changed: show");
    assert!(
        !list.iter().any(|line| line.contains("intro-2")),
        "{list:?}"
    );
    // It is on the screen, as it is on the command line.
    assert!(app.query_by_label("  line intro-1 changed: show").is_some());
    let (code, said) = jobs::now(&Action::Changes(folder.clone()).args());
    assert_eq!(code, 0);
    let said: Vec<String> = said.into_iter().map(|(_, line)| line).collect();
    assert_eq!(said, list);

    // A hand edit is listed too: a line added, and one taken out.
    let text = fs::read_to_string(folder.join("vidni.toml")).unwrap();
    let cut = text.find("[[lines]]\nid = \"intro-2\"").unwrap();
    let rest = &text[cut..];
    let end = rest[1..].find("\n[").map_or(rest.len(), |i| i + 2);
    let added = "[[lines]]\nid = \"intro-3\"\nsection = \"intro\"\nshow = \"A third line.\"\n";
    fs::write(
        folder.join("vidni.toml"),
        format!("{}{added}{}", &text[..cut], &rest[end..]),
    )
    .unwrap();
    app.run();
    let list = listed(&app);
    for named in [
        "  line intro-1 changed: show",
        "  line intro-2 is gone",
        "  line intro-3 is new",
    ] {
        assert!(list.iter().any(|line| line == named), "{named}: {list:#?}");
    }
}

#[test]
#[ignore = "needs Piper and its voice, Chromium and ffmpeg; run `just test-app`"]
fn what_changed_since_a_draft_and_since_ratification_names_the_lines() {
    let dir = tempfile::tempdir().unwrap();
    let folder = project(dir.path());
    let path = folder.to_str().unwrap();
    assert_eq!(vidni_cli::run(&["draft", path]), 0);
    assert_eq!(vidni_cli::run(&["ratify", path, "--cut", "long"]), 0);
    let mut app = start(&folder);
    click(&mut app, "What changed");
    assert_eq!(
        listed(&app),
        [
            "since the last build: nothing",
            "since each cut was ratified:",
            "  long: nothing",
            "  vertical: not ratified",
        ]
    );
    click(&mut app, FIRST);
    fill(&mut app, "Shown", "Hello, everyone.");
    click(&mut app, "Apply to intro-1");
    let list = listed(&app);
    assert_eq!(list[1], "  line intro-1 changed: show", "{list:#?}");
    let long = list
        .iter()
        .find(|line| line.starts_with("  long: "))
        .unwrap();
    assert!(long.contains("intro-1"), "{long}");
    assert!(!long.contains("intro-2"), "{long}");
    // Drafted again, the build is of the file as it is; the ratified cut
    // still differs from what was frozen.
    assert_eq!(vidni_cli::run(&["draft", path]), 0);
    click(&mut app, "Hide what changed");
    click(&mut app, "What changed");
    let list = listed(&app);
    assert_eq!(list[0], "since the last build: nothing");
    assert!(
        list.iter()
            .any(|line| line.starts_with("  long: ") && line.contains("intro-1")),
        "{list:#?}"
    );
}
