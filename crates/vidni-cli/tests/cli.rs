//! The `vidni` binary's validate and schema commands.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn vidni(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vidni"))
        .args(args)
        .current_dir(root())
        .output()
        .unwrap()
}

#[test]
fn validate_accepts_the_nibli_example() {
    let out = vidni(&["validate", "examples/nibli-syllogism"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "examples/nibli-syllogism/vidni.toml: ok: 57 lines in 10 sections, 5 scenes, 2 cuts\n"
    );
}

#[test]
fn validate_reports_each_broken_fixture_at_its_line_and_column() {
    let dir = "crates/vidni-core/tests/fixtures/invalid";
    for entry in fs::read_dir(root().join(dir)).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|x| x != "toml") {
            continue;
        }
        let name = format!("{dir}/{}", path.file_name().unwrap().to_string_lossy());
        let expected = fs::read_to_string(&path)
            .unwrap()
            .lines()
            .next()
            .unwrap()
            .replacen("# expect: ", "", 1);
        let out = vidni(&["validate", &name]);
        assert_eq!(out.status.code(), Some(1), "{name}");
        assert_eq!(
            String::from_utf8_lossy(&out.stderr),
            format!("{name}:{expected}\n1 problem\n"),
            "{name}"
        );
    }
}

#[test]
fn validate_exits_2_when_there_is_no_project() {
    let out = vidni(&["validate", "no-such-folder"]);
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn schema_prints_the_committed_schema() {
    let out = vidni(&["schema"]);
    assert!(out.status.success());
    let committed = fs::read_to_string(root().join("docs/vidni.schema.json")).unwrap_or_default();
    assert!(
        String::from_utf8_lossy(&out.stdout) == committed,
        "docs/vidni.schema.json is stale; run `just schema`"
    );
}

#[test]
fn fmt_check_passes_the_canonical_example_and_changes_nothing() {
    let before = fs::read_to_string(root().join("examples/nibli-syllogism/vidni.toml")).unwrap();
    let out = vidni(&["fmt", "--check", "examples/nibli-syllogism"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        fs::read_to_string(root().join("examples/nibli-syllogism/vidni.toml")).unwrap(),
        before
    );
}

#[test]
fn fmt_rewrites_a_messy_file_and_check_flags_it_first() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("vidni.toml");
    let messy =
        vidni_core::TEMPLATE.replace("title = \"My first video\"", "title=\"My first video\"   ");
    fs::write(&file, &messy).unwrap();
    let path = file.to_str().unwrap();
    let check = vidni(&["fmt", "--check", path]);
    assert_eq!(check.status.code(), Some(1));
    assert_eq!(fs::read_to_string(&file).unwrap(), messy);
    let out = vidni(&["fmt", path]);
    assert!(out.status.success());
    assert_eq!(fs::read_to_string(&file).unwrap(), vidni_core::TEMPLATE);
}

#[test]
fn fmt_refuses_an_invalid_file_with_its_problems() {
    let out = vidni(&[
        "fmt",
        "crates/vidni-core/tests/fixtures/invalid/09-unknown-scene.toml",
    ]);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains(":14:9: unknown scene \"pgae\""));
}

#[test]
fn takes_lists_each_lines_take_and_links_the_current_ones() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().to_str().unwrap();
    fs::write(dir.path().join("vidni.toml"), vidni_core::TEMPLATE).unwrap();
    let listing = |args: &[&str]| String::from_utf8(vidni(args).stdout).unwrap();
    let before = listing(&["takes", project]);
    assert!(
        before.ends_with("0 current, 0 stale, 2 missing\n"),
        "{before}"
    );
    // Record a take where the listing says each one goes.
    let paths: Vec<String> = before
        .lines()
        .take(2)
        .map(|l| l.split_whitespace().nth(2).unwrap().to_string())
        .collect();
    for path in &paths {
        let file = dir.path().join(path);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, path.as_bytes()).unwrap();
    }
    assert!(listing(&["takes", project]).ends_with("2 current, 0 stale, 0 missing\n"));
    let links = dir.path().join("links");
    let out = vidni(&["takes", project, "--link", links.to_str().unwrap()]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    for (k, path) in paths.iter().enumerate() {
        let link = links.join(format!("{k:03}.wav"));
        assert_eq!(fs::read_link(&link).unwrap(), dir.path().join(path));
    }
    // New words for the second line: its take goes stale, and --link refuses.
    let edited = vidni_core::TEMPLATE.replace(
        "This video was made with vidni.",
        "This video was made by hand.",
    );
    fs::write(dir.path().join("vidni.toml"), edited).unwrap();
    assert!(listing(&["takes", project]).contains("stale"));
    let out = vidni(&[
        "takes",
        project,
        "--link",
        dir.path().join("links2").to_str().unwrap(),
    ]);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("intro-2"));
}

#[test]
fn status_ratify_and_unratify_work_from_the_hashes() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().to_str().unwrap();
    fs::write(dir.path().join("vidni.toml"), vidni_core::TEMPLATE).unwrap();
    let out = |args: &[&str]| {
        let o = vidni(args);
        (
            o.status.code(),
            String::from_utf8_lossy(&o.stdout).into_owned(),
            String::from_utf8_lossy(&o.stderr).into_owned(),
        )
    };
    assert_eq!(
        out(&["status", project]).1,
        "long      draft\nvertical  draft\n"
    );
    let (code, _, err) = out(&["ratify", project]);
    assert_eq!(code, Some(1));
    assert!(err.contains("captures aren't taken yet"), "{err}");
    // Stand-ins for the captures a draft would take.
    let parsed = vidni_core::validate::check(vidni_core::TEMPLATE, None).unwrap();
    for cut in &parsed.cuts {
        for c in vidni_core::capture::for_cut(&parsed, cut) {
            let path = vidni_core::capture::path(dir.path(), &c.key);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, c.key.as_bytes()).unwrap();
        }
    }
    let (code, stdout, _) = out(&["ratify", project, "--cut", "long"]);
    assert_eq!(code, Some(0));
    assert!(stdout.starts_with("long: ratified (content "), "{stdout}");
    assert!(dir.path().join("build/resolved.json").is_file());
    assert_eq!(
        out(&["status", project]).1,
        "long      ratified\nvertical  draft\n"
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
