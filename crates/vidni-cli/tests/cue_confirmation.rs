//! Recording an explicit owner decision is separate from self-verification.
use std::{fs, process::Command};

#[test]
fn assistant_confirmation_requires_owner_text_and_each_selected_video_hash() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("vidni.toml"),
        r#"schema = 1
[meta]
title = "Review confirmation"
default_scene = "screen"
[sections.main]
[[lines]]
id = "line"
section = "main"
show = "A recorded premise."
[[cuts]]
id = "long"
aspect = "16:9"
[[cuts]]
id = "short"
aspect = "9:16"
[scenes.screen]
kind = "web"
url = "https://example.com"
"#,
    )
    .unwrap();
    let statement = dir.path().join("statement.txt");
    let run = |hashes: &[&str]| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_vidni"));
        command
            .env("VIDNI_ASSISTANT", "1")
            .args(["cues", "record-confirmation"])
            .arg(dir.path())
            .arg("--statement-file")
            .arg(&statement);
        for hash in hashes {
            command.arg("--review-sha256").arg(hash);
        }
        command.output().unwrap()
    };
    fs::write(&statement, "  ").unwrap();
    let out = run(&["long=abc"]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("explicit owner review statement"));
    fs::write(
        &statement,
        "I watched both review videos and confirm the cues are correct.",
    )
    .unwrap();
    for hashes in [
        vec!["long=abc"],
        vec!["long=abc", "long=def"],
        vec!["wrong=abc", "short=def"],
    ] {
        let out = run(&hashes);
        assert!(!out.status.success());
        let error = String::from_utf8_lossy(&out.stderr);
        assert!(
            error.contains("every selected cut") || error.contains("duplicate review hash"),
            "{error}"
        );
        assert!(!dir.path().join("cues.lock").exists());
    }
}

#[test]
fn assistant_still_cannot_use_interactive_verification() {
    let out = Command::new(env!("CARGO_BIN_EXE_vidni"))
        .env("VIDNI_ASSISTANT", "1")
        .args(["cues", "review", "unused-project"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("cue verification belongs to the person")
    );
}
