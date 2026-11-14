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
            if matches!(c.source, vidni_core::capture::Source::Line(_)) {
                let frames = vidni_core::capture::frames_path(dir.path(), &c.key);
                fs::write(frames, br#"{"targets":{}}"#).unwrap();
            }
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

#[test]
fn platforms_lists_every_profile_with_its_date_and_sources() {
    let out = vidni(&["platforms"]);
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    let blocks: Vec<&str> = text
        .split("\n\n")
        .filter(|b| !b.starts_with("Platforms change"))
        .collect();
    let ids: Vec<&str> = blocks
        .iter()
        .map(|b| b.split_whitespace().next().unwrap())
        .collect();
    assert_eq!(
        ids,
        [
            "youtube",
            "youtube-shorts",
            "instagram-reels",
            "tiktok",
            "whatsapp-status",
            "linkedin",
            "x"
        ]
    );
    for block in blocks {
        let lines: Vec<&str> = block.lines().collect();
        let at = lines
            .iter()
            .position(|l| l.trim_start().starts_with("verified "))
            .unwrap_or_else(|| panic!("{block}"));
        let date = lines[at].trim_start()["verified ".len()..].trim_end_matches(" against:");
        assert_eq!(date.len(), 10, "{}", lines[at]);
        let sources = &lines[at + 1..];
        assert!(
            !sources.is_empty()
                && sources
                    .iter()
                    .all(|s| s.trim_start().starts_with("https://")),
            "{block}"
        );
    }
    assert!(text.ends_with("confirm one in the app before relying on it.\n"));
}

/// Runs vidni with an empty voice store.
fn vidni_without_voices(args: &[&str], store: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vidni"))
        .args(args)
        .current_dir(root())
        .env("VIDNI_VOICES", store)
        .output()
        .unwrap()
}

/// The minimal fixture with `draft` as its draft voice.
fn project_with_voice(dir: &Path, draft: &str) -> PathBuf {
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

#[test]
fn validate_warns_about_a_draft_voice_that_forbids_commercial_use() {
    let dir = tempfile::tempdir().unwrap();
    let file = project_with_voice(dir.path(), "piper:en_US-ryan-medium");
    let out = vidni_without_voices(&["validate", file.to_str().unwrap()], dir.path());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{stderr}");
    assert!(
        stderr.contains(&format!(
            "{}:9:9: warning: the draft voice piper:en_US-ryan-medium is licensed \
             CC-BY-NC-SA-4.0, which doesn't allow commercial use",
            file.display()
        )),
        "{stderr}"
    );
    let file = project_with_voice(dir.path(), "piper:en_US-joe-medium");
    let out = vidni_without_voices(&["validate", file.to_str().unwrap()], dir.path());
    assert!(out.status.success() && out.stderr.is_empty());
}

#[test]
fn validate_warns_when_a_draft_voices_licence_is_unknown() {
    let dir = tempfile::tempdir().unwrap();
    let file = project_with_voice(dir.path(), "piper:someone-elses");
    let out = vidni_without_voices(&["validate", file.to_str().unwrap()], dir.path());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{stderr}");
    assert!(stderr.contains("so its licence is unknown"), "{stderr}");
}

#[test]
fn voices_lists_each_pack_with_its_licence() {
    let store = tempfile::tempdir().unwrap();
    let out = vidni_without_voices(&["voices"], store.path());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(
        stdout.contains(
            "piper:en_US-joe-medium (en_US): CC0-1.0, commercial use allowed; not fetched"
        )
    );
    assert!(stdout.contains(
        "piper:en_US-ryan-medium (en_US): CC-BY-NC-SA-4.0, no commercial use; not fetched"
    ));
}

#[test]
fn speak_says_how_to_fetch_a_missing_voice() {
    let store = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let file = project_with_voice(dir.path(), "piper:en_US-joe-medium");
    let out = vidni_without_voices(&["speak", file.to_str().unwrap()], store.path());
    let stderr = String::from_utf8_lossy(&out.stderr);
    // A voice that isn't fetched is something missing, as a font is.
    assert_eq!(out.status.code(), Some(2), "{stderr}");
    assert!(
        stderr.contains("run `vidni voices fetch piper:en_US-joe-medium`"),
        "{stderr}"
    );
}

#[test]
fn capture_names_a_chromium_that_isnt_there() {
    let dir = tempfile::tempdir().unwrap();
    let file = project_with_voice(dir.path(), "piper:en_US-joe-medium");
    let out = Command::new(env!("CARGO_BIN_EXE_vidni"))
        .args(["capture", file.to_str().unwrap()])
        .env("VIDNI_CHROMIUM", "/nonexistent/chromium")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "{stderr}");
    assert!(
        stderr.contains("vidni: /nonexistent/chromium: "),
        "{stderr}"
    );
    // Nothing was kept: no still, and no staging folder.
    let left = fs::read_dir(dir.path().join("build/captures"))
        .map(|d| d.count())
        .unwrap_or(0);
    assert_eq!(left, 0);
}

#[test]
fn hear_says_why_it_cant_listen() {
    let dir = tempfile::tempdir().unwrap();
    let file = project_with_voice(dir.path(), "piper:en_US-joe-medium");
    let out = Command::new(env!("CARGO_BIN_EXE_vidni"))
        .args(["hear", file.to_str().unwrap()])
        .env("VIDNI_WHISPER", "/nonexistent/whisper-cli")
        .env("VIDNI_WHISPER_MODEL", dir.path().join("no-model.bin"))
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(2), "{stderr}");
    assert!(
        stderr.starts_with("vidni: the listening check can't run: /nonexistent/whisper-cli: "),
        "{stderr}"
    );
    // Both missing things are named at once.
    assert!(stderr.contains("no-model.bin: "), "{stderr}");
    assert!(out.stdout.is_empty());
    // And the check leaves the project folder as it was.
    let left: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(left, ["vidni.toml"]);
}

#[test]
fn hear_refuses_a_script_that_isnt_english() {
    let dir = tempfile::tempdir().unwrap();
    let file = project_with_voice(dir.path(), "piper:en_US-joe-medium");
    let tamil = fs::read_to_string(&file)
        .unwrap()
        .replace("[meta]", "[meta]\nlanguage = \"ta\"");
    fs::write(&file, tamil).unwrap();
    let out = vidni(&["hear", file.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "vidni: the listening check can't run: it knows English only, and this script's language is ta\n"
    );
}

#[test]
fn hear_names_a_line_that_isnt_there() {
    let dir = tempfile::tempdir().unwrap();
    let file = project_with_voice(dir.path(), "piper:en_US-joe-medium");
    let out = vidni(&["hear", file.to_str().unwrap(), "--line", "nope"]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        format!("vidni: {}: no line nope\n", file.display())
    );
}

/// Runs vidni with no fonts to be found, so that anything it does needs none.
fn vidni_without_fonts(args: &[&str]) -> (Option<i32>, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_vidni"))
        .args(args)
        .env("VIDNI_FONTS", "/nonexistent/fonts")
        .output()
        .unwrap();
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn the_gates_refuse_before_any_tool_is_needed() {
    let dir = tempfile::tempdir().unwrap();
    let file = project_with_voice(dir.path(), "piper:en_US-joe-medium");
    let project = file.to_str().unwrap();
    let unratified = "vidni: cut long isn't ratified; `vidni ratify --cut long` freezes it\n\
                      vidni: cut long: 1 of its 1 lines has no take: main-1\n";
    // A final build and an approval of a cut that was never ratified or recorded.
    for command in ["final", "approve"] {
        let (code, stdout, stderr) = vidni_without_fonts(&[command, project]);
        assert_eq!(
            (code, stdout.as_str()),
            (Some(1), ""),
            "{command}: {stderr}"
        );
        assert_eq!(stderr, unratified, "{command}");
    }
    // A repro builds from the captures it finds, and there are none yet.
    let (code, _, stderr) = vidni_without_fonts(&["repro", project]);
    assert_eq!(code, Some(1), "{stderr}");
    assert!(
        stderr.starts_with("vidni: 1 of the captures isn't taken yet (the first: line main-1); "),
        "{stderr}"
    );
    // A cut that doesn't exist.
    for command in ["draft", "final", "approve", "repro"] {
        let (code, _, stderr) = vidni_without_fonts(&[command, project, "--cut", "nope"]);
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

#[test]
fn a_build_says_which_font_it_cant_find() {
    let dir = tempfile::tempdir().unwrap();
    let file = project_with_voice(dir.path(), "piper:en_US-joe-medium");
    let (code, stdout, stderr) = vidni_without_fonts(&["draft", file.to_str().unwrap()]);
    assert_eq!((code, stdout.as_str()), (Some(2), ""), "{stderr}");
    assert_eq!(
        stderr,
        "vidni: the font IBMPlexSans-Bold.otf (IBM Plex Sans Bold 1.1.0) wasn't found: looked in \
         /nonexistent/fonts; VIDNI_FONTS names the folders that hold vidni's fonts\n"
    );
}

#[test]
fn a_command_marked_offline_must_have_no_network() {
    let dir = tempfile::tempdir().unwrap();
    let file = project_with_voice(dir.path(), "piper:en_US-joe-medium");
    let out = Command::new(env!("CARGO_BIN_EXE_vidni"))
        .args(["validate", file.to_str().unwrap()])
        .env("VIDNI_OFFLINE", "1")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    // Where these tests run with a route to the network, the mark alone is
    // refused. (A build sandbox has none, and there the mark is true.)
    let routed = |file: &str, name: fn(&str) -> Option<&str>| {
        fs::read_to_string(file)
            .unwrap_or_default()
            .lines()
            .filter_map(name)
            .any(|name| name != "Iface" && name != "lo")
    };
    let networked = routed("/proc/self/net/route", |l| l.split_whitespace().next())
        || routed("/proc/self/net/ipv6_route", |l| {
            l.split_whitespace().next_back()
        });
    if networked {
        assert_eq!(out.status.code(), Some(2), "{stderr}");
        assert!(
            stderr.starts_with(
                "vidni: VIDNI_OFFLINE is set, but this process has a route to the network (through "
            ),
            "{stderr}"
        );
    } else {
        assert!(out.status.success(), "{stderr}");
    }
}

#[test]
fn approve_exits_2_when_the_releases_cant_be_read() {
    let dir = tempfile::tempdir().unwrap();
    let file = project_with_voice(dir.path(), "piper:en_US-joe-medium");
    fs::write(dir.path().join("release.json"), "{\"cuts\": {}}").unwrap();
    for command in ["approve", "status"] {
        let (code, _, stderr) = vidni_without_fonts(&[command, file.to_str().unwrap()]);
        assert_eq!(code, Some(2), "{command}: {stderr}");
        assert!(
            stderr.ends_with("release.json: it names no schema\n"),
            "{command}: {stderr}"
        );
    }
}
