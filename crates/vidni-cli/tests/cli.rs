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
