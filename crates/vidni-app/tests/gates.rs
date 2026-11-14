//! The gates in the app (TODO.md, P7-VIDNI-05): the command line's own
//! gate tests, with every ratify, unratify, final, approve, draft and
//! status run through the app's command layer: the action the app's
//! buttons make, run as the job the app runs. They must say the same
//! words and give the same codes as the `vidni` binary does.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use vidni_app::actions::Action;
use vidni_app::jobs::{self, Job};
use vidni_cli::console::Stream;

#[path = "../../vidni-cli/tests/shared/flow.rs"]
mod flow;
#[path = "../../vidni-cli/tests/shared/gates.rs"]
mod gates;

/// How many commands went through an action of the app's.
static THROUGH_THE_APP: AtomicUsize = AtomicUsize::new(0);

/// The app's action that is `vidni <args>`, when it has one.
fn action(args: &[&str]) -> Option<Action> {
    let project = PathBuf::from(args.get(1)?);
    let cut = match &args[2..] {
        [] => None,
        ["--cut", id] => Some(id.to_string()),
        _ => return None,
    };
    Some(match (args[0], cut) {
        ("status", None) => Action::Status(project),
        ("draft", cut) => Action::Draft { project, cut },
        ("ratify", cut) => Action::Ratify { project, cut },
        ("unratify", Some(cut)) => Action::Unratify { project, cut },
        ("final", cut) => Action::Final { project, cut },
        ("approve", cut) => Action::Approve { project, cut },
        _ => return None,
    })
}

/// What was said on `stream`, as the text a terminal would have had.
fn text(said: &[(Stream, String)], stream: Stream) -> String {
    said.iter()
        .filter(|(s, _)| *s == stream)
        .map(|(_, line)| format!("{line}\n"))
        .collect()
}

/// Runs `action` as the app does: as a job, on a thread of its own.
fn job(action: &Action, args: &[&str]) -> (Option<i32>, String, String) {
    // The action is the very command the test asked for.
    assert_eq!(action.args(), args);
    THROUGH_THE_APP.fetch_add(1, Ordering::SeqCst);
    let mut job = Job::start(action.args(), || {});
    let code = job.wait();
    (
        Some(i32::from(code)),
        text(&job.lines, Stream::Out),
        text(&job.lines, Stream::Err),
    )
}

/// Runs a command the way the app would: its action as a job, or, where
/// the app has no action for it, the command in this process.
fn app(args: &[&str]) -> (Option<i32>, String, String) {
    match action(args) {
        Some(action) => job(&action, args),
        None => {
            let args: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
            let (code, said) = jobs::now(&args);
            (
                Some(i32::from(code)),
                text(&said, Stream::Out),
                text(&said, Stream::Err),
            )
        }
    }
}

/// As [`app`], for the whole flow: what the app has no action for is run
/// by the `vidni` binary that `VIDNI_BIN` names, as a person would beside
/// the app. A repro and an offline build start vidni again by its own
/// file, which in this process is the test's.
fn app_beside_the_binary(args: &[&str], cwd: Option<&Path>) -> (Option<i32>, String, String) {
    if let (Some(action), None) = (action(args), cwd) {
        return job(&action, args);
    }
    let binary = std::env::var_os("VIDNI_BIN").expect("VIDNI_BIN names the vidni binary");
    let mut command = Command::new(binary);
    command.args(args);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    let out = command.output().unwrap();
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn status_ratify_and_unratify_work_from_the_hashes() {
    gates::status_ratify_and_unratify_work_from_the_hashes(app);
}

#[test]
fn the_gates_refuse_before_any_tool_is_needed() {
    gates::the_gates_refuse_before_any_tool_is_needed(app);
}

#[test]
fn approve_exits_2_when_the_releases_cant_be_read() {
    gates::approve_exits_2_when_the_releases_cant_be_read(app);
}

#[test]
#[ignore = "needs Piper and its voice, Chromium, ffmpeg, unshare and taskset; run `just test-app`"]
fn a_project_goes_from_draft_to_approved_and_its_builds_repeat() {
    let before = THROUGH_THE_APP.load(Ordering::SeqCst);
    flow::a_project_goes_from_draft_to_approved_and_its_builds_repeat(app_beside_the_binary);
    // Every gate of the flow was the app's: its drafts, ratifications,
    // final builds, approvals and readings of the status.
    let through = THROUGH_THE_APP.load(Ordering::SeqCst) - before;
    println!("{through} commands went through the app");
    assert!(through >= 20, "{through} commands went through the app");
}

/// The app on the project in `folder`, at its cuts.
fn window<'a>(folder: &Path) -> egui_kittest::Harness<'a, vidni_app::app::App> {
    let mut app = vidni_app::app::App::new(None);
    app.open(folder);
    let mut harness = egui_kittest::Harness::builder()
        .with_size([1400.0, 1600.0])
        .build_ui_state(|ui, app: &mut vidni_app::app::App| app.show(ui), app);
    harness.run();
    harness
}

/// Presses the button `label`, waits for the job it starts, and gives
/// the job's code and what it said.
fn press(
    harness: &mut egui_kittest::Harness<'_, vidni_app::app::App>,
    label: &str,
) -> (u8, Vec<String>) {
    use egui_kittest::kittest::Queryable as _;
    harness.get_by_label(label).click();
    harness.step();
    let job = harness.state_mut().job.as_mut().expect("a job");
    let code = job.wait();
    let said = job.lines.iter().map(|(_, line)| line.clone()).collect();
    harness.run();
    (code, said)
}

/// Where the cuts stand, as the Cuts tab has them.
fn standing(harness: &egui_kittest::Harness<'_, vidni_app::app::App>) -> Vec<(String, String)> {
    harness.state().project.as_ref().unwrap().cuts.clone()
}

#[test]
fn the_cuts_tab_runs_the_gates_and_lists_what_a_cut_froze() {
    use egui_kittest::kittest::Queryable as _;
    let dir = tempfile::tempdir().unwrap();
    let folder = dir.path();
    std::fs::write(folder.join("vidni.toml"), vidni_core::TEMPLATE).unwrap();
    std::fs::write(folder.join("page.html"), "<!doctype html>").unwrap();
    let path = folder.to_str().unwrap();
    let cuts = |long: &str, vertical: &str| {
        vec![
            ("long".to_string(), long.to_string()),
            ("vertical".to_string(), vertical.to_string()),
        ]
    };
    let mut app = window(folder);
    assert_eq!(standing(&app), cuts("draft", "draft"));

    // Each gate refuses what the command line refuses, in its words.
    let unratified = [
        "vidni: cut long isn't ratified; `vidni ratify --cut long` freezes it",
        "vidni: cut long: 2 of its 2 lines have no take: intro-1, intro-2",
    ];
    for gate in ["Final long", "Approve long"] {
        let (code, said) = press(&mut app, gate);
        assert_eq!(
            (code, said.as_slice()),
            (1, &unratified.map(String::from)[..]),
            "{gate}"
        );
        // The refusal is on the screen, under the command that was run.
        assert!(app.query_by_label("Stopped, with exit code 1.").is_some());
        assert!(app.query_by_label(unratified[0]).is_some());
    }
    let command = format!("$ vidni approve {path} --cut long");
    assert!(app.query_by_label(&command).is_some(), "{command}");
    // A cut whose captures aren't taken can't be ratified.
    let (code, said) = press(&mut app, "Ratify long");
    assert_eq!(code, 1);
    assert!(said[0].contains("captures aren't taken yet"), "{said:?}");
    assert_eq!(standing(&app), cuts("draft", "draft"));

    // With its captures taken, the cut is ratified, and the tab says so.
    gates::stand_in_captures(folder, vidni_core::TEMPLATE);
    let (code, said) = press(&mut app, "Ratify long");
    assert_eq!(code, 0, "{said:?}");
    assert!(said[0].starts_with("long: ratified (content "), "{said:?}");
    assert_eq!(standing(&app), cuts("ratified", "draft"));

    // What it froze: the lock's own hashes, each one on the screen.
    app.get_by_label("Frozen long").click();
    app.run();
    let lock = vidni_core::ratify::read_lock(folder).unwrap();
    let locked = &lock.cuts["long"];
    let (cut, listed) = app.state().frozen.clone().expect("the list");
    assert_eq!(cut, "long");
    assert_eq!(listed[0], format!("long: content {}", locked.content));
    let mut hashes = 1;
    for (id, hash) in &locked.lines {
        assert!(listed.contains(&format!("  line {id}  {hash}")), "{id}");
        hashes += 1;
    }
    for (id, hash) in &locked.scenes {
        assert!(listed.contains(&format!("  scene {id}  {hash}")), "{id}");
        hashes += 1;
    }
    for (key, sha) in &locked.captures {
        let row = format!("  capture build/captures/{key}.png  sha256 {sha}");
        assert!(listed.contains(&row), "{row}");
        hashes += 1;
    }
    for (key, sha) in &locked.frames {
        let row = format!("  frames build/captures/{key}.json  sha256 {sha}");
        assert!(listed.contains(&row), "{row}");
        hashes += 1;
    }
    assert!(!locked.captures.is_empty() && !locked.frames.is_empty());
    // Nothing else is listed, and every row is drawn.
    assert_eq!(listed.len(), hashes);
    for row in &listed {
        assert!(app.query_by_label(row).is_some(), "{row}");
    }
    // It is `vidni frozen`'s list.
    let (code, said, _) = crate::app(&["frozen", path, "--cut", "long"]);
    assert_eq!(code, Some(0));
    assert_eq!(
        said,
        listed
            .iter()
            .map(|row| format!("{row}\n"))
            .collect::<String>()
    );

    // A ratified cut with no takes: the final build says which lines.
    let (code, said) = press(&mut app, "Final long");
    assert_eq!(code, 1);
    assert_eq!(said, [unratified[1]]);

    // New words for a line, written by hand: the cut is changed, the tab
    // says which line, and the gates refuse it as the command line does.
    let edited = vidni_core::TEMPLATE.replace("Hello, and welcome.", "Hello, and good evening.");
    std::fs::write(folder.join("vidni.toml"), edited).unwrap();
    app.get_by_label("Read again").click();
    app.run();
    assert_eq!(
        standing(&app),
        cuts("changed: line intro-1 changed", "draft")
    );
    let (code, said) = press(&mut app, "Final long");
    assert_eq!(code, 1);
    let (cli, _, refusal) = crate::app(&["final", path, "--cut", "long"]);
    assert_eq!(cli, Some(1));
    assert_eq!(
        said.iter().map(|l| format!("{l}\n")).collect::<String>(),
        refusal
    );
    assert!(said[0].contains("line intro-1 changed"), "{said:?}");

    // Ratified again, the list is of the new hashes.
    let before = app.state().frozen.clone().unwrap().1;
    let (code, _) = press(&mut app, "Ratify every cut");
    assert_eq!(code, 0);
    assert_eq!(standing(&app), cuts("ratified", "ratified"));
    let after = app.state().frozen.clone().unwrap().1;
    assert_ne!(before[0], after[0]);
    let lock = vidni_core::ratify::read_lock(folder).unwrap();
    assert_eq!(
        after[0],
        format!("long: content {}", lock.cuts["long"].content)
    );

    // Unratified, it is a draft again and nothing of it is frozen.
    let (code, said) = press(&mut app, "Unratify long");
    assert_eq!(code, 0);
    assert!(said[0].contains("the captures stay"), "{said:?}");
    assert_eq!(standing(&app), cuts("draft", "ratified"));
    assert_eq!(
        app.state().frozen.clone().unwrap().1,
        ["long: not ratified"]
    );
    app.get_by_label("Hide what long froze").click();
    app.run();
    assert!(app.state().frozen.is_none());
}
