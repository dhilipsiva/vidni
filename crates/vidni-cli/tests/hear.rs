//! `vidni hear` with the real whisper-cli and its model, which the dev
//! shell provides (`just test-hear`): a spoken, an empty and a silent take.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const PROJECT: &str = r#"schema = 1

[meta]
title = "A project to listen to"
default_scene = "page"

[sections.main]

[[lines]]
id = "said"
section = "main"
show = "Language models are good at reading prose."

[[lines]]
id = "misread"
section = "main"
show = "Symbolic systems are good at checking it."

[[lines]]
id = "empty"
section = "main"
show = "This take holds no sound at all."

[[lines]]
id = "silent"
section = "main"
show = "This take holds two seconds of silence."

[[lines]]
id = "unsaid"
section = "main"
show = "Nobody has said this yet."

[scenes.page]
kind = "web"
url = "https://example.com"

[[cuts]]
id = "long"
aspect = "16:9"
"#;

fn hear(file: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vidni"))
        .arg("hear")
        .arg(file)
        .output()
        .unwrap()
}

/// Every file under `dir`, with its bytes.
fn files(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut found = Vec::new();
    let mut folders = vec![dir.to_path_buf()];
    while let Some(folder) = folders.pop() {
        for entry in fs::read_dir(folder).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                folders.push(path);
            } else {
                found.push((path.clone(), fs::read(&path).unwrap()));
            }
        }
    }
    found.sort();
    found
}

/// The score at the end of a line such as `said     take   0.97`.
fn score(line: &str) -> f64 {
    line.rsplit(' ').next().unwrap().parse().unwrap()
}

#[test]
#[ignore = "needs whisper-cli and its model; run `just test-hear`"]
fn hear_scores_each_take_and_changes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    let file = dir.join("vidni.toml");
    fs::write(&file, PROJECT).unwrap();
    let project = vidni_core::validate::check(PROJECT, Some(dir)).unwrap();
    let takes = vidni_core::takes::status(&project, dir);
    let take = |id: &str| {
        let path = &takes.iter().find(|t| t.id == id).unwrap().path;
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        path.clone()
    };
    // One clip of the first line's words stands for a take of it, and for
    // a take of the second line that says something else.
    let spoken =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../vidni-media/tests/fixtures/hear/spoken.wav");
    fs::copy(&spoken, take("said")).unwrap();
    fs::copy(&spoken, take("misread")).unwrap();
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 48000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    hound::WavWriter::create(take("empty"), spec)
        .unwrap()
        .finalize()
        .unwrap();
    let mut silent = hound::WavWriter::create(take("silent"), spec).unwrap();
    for _ in 0..96000 {
        silent.write_sample(0i16).unwrap();
    }
    silent.finalize().unwrap();

    let before = files(dir);
    let out = hear(&file);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(0), "{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 8, "{stdout}");
    assert!(lines[0].starts_with("said     take   "), "{stdout}");
    assert!(score(lines[0]) >= 0.9, "{stdout}");
    // A take that says other words is worth a listen, and is shown with
    // what was heard.
    assert!(lines[1].starts_with("misread  take   "), "{stdout}");
    assert!(score(lines[1]) < 0.9, "{stdout}");
    assert!(lines[2].starts_with("           heard:  "), "{stdout}");
    assert_eq!(
        lines[3],
        "           script: Symbolic systems are good at checking it."
    );
    assert_eq!(lines[4], "empty    take   nothing to hear");
    assert_eq!(lines[5], "silent   take   nothing to hear");
    assert_eq!(lines[6], "unsaid   no audio");
    assert_eq!(
        lines[7],
        format!(
            "{}: 1 heard as scripted, 1 worth a listen (below 0.9), 2 with nothing to hear, \
             1 with no audio",
            file.display()
        )
    );
    // The check wrote nothing into the project.
    assert_eq!(files(dir), before);

    // A take that can't be read is named, and the others are still heard.
    fs::write(take("unsaid"), b"not a sound file").unwrap();
    let out = hear(&file);
    assert_eq!(out.status.code(), Some(2));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("\nunsaid   take   not heard\n"), "{stdout}");
    assert!(
        stdout.contains("1 heard as scripted, 1 worth a listen"),
        "{stdout}"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.starts_with(&format!("vidni: {}: ", take("unsaid").display())),
        "{stderr}"
    );
    assert!(
        stderr.ends_with("vidni: 1 line couldn't be listened to\n"),
        "{stderr}"
    );
}
