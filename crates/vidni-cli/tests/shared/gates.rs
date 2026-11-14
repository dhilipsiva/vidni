//! The gates' tests: ratify, unratify, final, approve and status, by what
//! they say and the codes they give. They are run twice: by the `vidni`
//! binary (`tests/cli.rs`), and through the desktop app's command layer
//! (`crates/vidni-app/tests/gates.rs`), which must give the same words
//! and the same codes (TODO.md, P7-VIDNI-05).

use std::fs;
use std::path::{Path, PathBuf};

/// Runs `vidni <args>`; its exit code, stdout and stderr.
pub type Run = fn(&[&str]) -> (Option<i32>, String, String);

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The minimal fixture with `draft` as its draft voice.
pub fn project_with_voice(dir: &Path, draft: &str) -> PathBuf {
    let minimal =
        fs::read_to_string(root().join("crates/vidni-core/tests/fixtures/valid/minimal.toml"))
            .unwrap()
            .replace(
                "[sections.main]",
                &format!("[voice]\ndraft = \"{draft}\"\n\n[sections.main]"),
            );
    let file = dir.join("vidni.toml");
    fs::write(&file, minimal).unwrap();
    file
}

/// Stand-ins for the captures a draft of the project `text` in `dir`
/// would take, so that its cuts can be ratified with no browser.
pub fn stand_in_captures(dir: &Path, text: &str) {
    let parsed = vidni_core::validate::check(text, None).unwrap();
    for cut in &parsed.cuts {
        for c in vidni_core::capture::for_cut(&parsed, cut) {
            let path = vidni_core::capture::path(dir, &c.key);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, c.key.as_bytes()).unwrap();
            if matches!(c.source, vidni_core::capture::Source::Line(_)) {
                let frames = vidni_core::capture::frames_path(dir, &c.key);
                fs::write(frames, br#"{"targets":{}}"#).unwrap();
            }
            // The page is the project's own, so its capture comes with
            // the record of the files it was made from.
            let file = c.file.clone().unwrap();
            let sha = vidni_core::capture::file_sha256(dir, &file);
            let made_from = [(file, sha)].into();
            fs::write(
                vidni_core::capture::files_path(dir, &c.key),
                vidni_core::capture::files_json(&made_from),
            )
            .unwrap();
        }
    }
}

pub fn status_ratify_and_unratify_work_from_the_hashes(run: Run) {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().to_str().unwrap();
    fs::write(dir.path().join("vidni.toml"), vidni_core::TEMPLATE).unwrap();
    fs::write(dir.path().join("page.html"), "<!doctype html>").unwrap();
    let out = run;
    assert_eq!(
        out(&["status", project]).1,
        "long      draft\nvertical  draft\n"
    );
    let (code, _, err) = out(&["ratify", project]);
    assert_eq!(code, Some(1));
    assert!(err.contains("captures aren't taken yet"), "{err}");
    stand_in_captures(dir.path(), vidni_core::TEMPLATE);
    let (code, stdout, _) = out(&["ratify", project, "--cut", "long"]);
    assert_eq!(code, Some(0));
    assert!(stdout.starts_with("long: ratified (content "), "{stdout}");
    assert!(dir.path().join("build/resolved.json").is_file());
    assert_eq!(
        out(&["status", project]).1,
        "long      ratified\nvertical  draft\n"
    );
    // What the cut froze is listed with the lock's hashes; a draft has none.
    let (code, frozen, _) = out(&["frozen", project]);
    assert_eq!(code, Some(0));
    let lock = vidni_core::ratify::read_lock(dir.path()).unwrap();
    let locked = &lock.cuts["long"];
    let rows: Vec<&str> = frozen.lines().collect();
    assert_eq!(rows[0], format!("long: content {}", locked.content));
    assert_eq!(rows.last(), Some(&"vertical: not ratified"));
    assert_eq!(
        rows.len(),
        2 + locked.lines.len() + locked.scenes.len() + locked.captures.len() + locked.frames.len()
    );
    for (key, sha) in &locked.captures {
        let row = format!("  capture build/captures/{key}.png  sha256 {sha}");
        assert!(rows.contains(&row.as_str()), "{row}\n{frozen}");
    }
    let (code, _, err) = out(&["frozen", project, "--cut", "nope"]);
    assert_eq!(
        (code, err.as_str()),
        (Some(1), "vidni: there is no cut \"nope\"\n")
    );
    // New words for a line regress the ratified cut, and say which line.
    let edited = vidni_core::TEMPLATE.replace("Hello, and welcome.", "Hello, and good evening.");
    fs::write(dir.path().join("vidni.toml"), edited).unwrap();
    assert_eq!(
        out(&["status", project]).1,
        "long      changed: line intro-1 changed\nvertical  draft\n"
    );
    let (code, stdout, _) = out(&["unratify", project, "--cut", "long"]);
    assert_eq!(code, Some(0));
    assert!(stdout.contains("the captures stay"), "{stdout}");
    assert_eq!(
        out(&["status", project]).1,
        "long      draft\nvertical  draft\n"
    );
    assert_eq!(out(&["ratify", project, "--cut", "nope"]).0, Some(1));
}

pub fn the_gates_refuse_before_any_tool_is_needed(run: Run) {
    let dir = tempfile::tempdir().unwrap();
    let file = project_with_voice(dir.path(), "piper:en_US-joe-medium");
    let project = file.to_str().unwrap();
    let unratified = "vidni: cut long isn't ratified; `vidni ratify --cut long` freezes it\n\
                      vidni: cut long: 1 of its 1 lines has no take: main-1\n";
    // A final build and an approval of a cut that was never ratified or recorded.
    for command in ["final", "approve"] {
        let (code, stdout, stderr) = run(&[command, project]);
        assert_eq!(
            (code, stdout.as_str()),
            (Some(1), ""),
            "{command}: {stderr}"
        );
        assert_eq!(stderr, unratified, "{command}");
    }
    // A repro builds from the captures it finds, and there are none yet.
    let (code, _, stderr) = run(&["repro", project]);
    assert_eq!(code, Some(1), "{stderr}");
    assert!(
        stderr.starts_with("vidni: 1 of 1 captures isn't taken yet (the first: line main-1); "),
        "{stderr}"
    );
    // A cut that doesn't exist.
    for command in ["draft", "final", "approve", "repro"] {
        let (code, _, stderr) = run(&[command, project, "--cut", "nope"]);
        assert_eq!(code, Some(1), "{command}");
        assert_eq!(stderr, "vidni: there is no cut \"nope\"\n", "{command}");
    }
    // Nothing was made on the way.
    let left: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(left, ["vidni.toml"]);
}

pub fn approve_exits_2_when_the_releases_cant_be_read(run: Run) {
    let dir = tempfile::tempdir().unwrap();
    let file = project_with_voice(dir.path(), "piper:en_US-joe-medium");
    fs::write(dir.path().join("release.json"), "{\"cuts\": {}}").unwrap();
    for command in ["approve", "status"] {
        let (code, _, stderr) = run(&[command, file.to_str().unwrap()]);
        assert_eq!(code, Some(2), "{command}: {stderr}");
        assert!(
            stderr.ends_with("release.json: it names no schema\n"),
            "{command}: {stderr}"
        );
    }
}
