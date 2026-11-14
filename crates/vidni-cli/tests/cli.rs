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
        "examples/nibli-syllogism/vidni.toml: ok: 59 lines in 10 sections, 7 scenes, 3 cuts\n"
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
    fs::write(dir.path().join("page.html"), "<!doctype html>").unwrap();
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
fn takes_of_one_cut_leave_out_the_lines_only_for_others() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().to_str().unwrap();
    // A hook that only the vertical cut says, ahead of the shared lines.
    let with_a_hook = vidni_core::TEMPLATE.replace(
        "[[lines]]\nid = \"intro-1\"",
        "[[lines]]\nid = \"hook\"\nsection = \"intro\"\nshow = \"Watch this.\"\n\
         only = [\"vertical\"]\n\n[[lines]]\nid = \"intro-1\"",
    );
    assert_ne!(with_a_hook, vidni_core::TEMPLATE);
    fs::write(dir.path().join("vidni.toml"), with_a_hook).unwrap();
    fs::write(dir.path().join("page.html"), "<!doctype html>").unwrap();
    let ids = |args: &[&str]| -> Vec<String> {
        let out = String::from_utf8(vidni(args).stdout).unwrap();
        let mut lines: Vec<&str> = out.lines().collect();
        lines.pop();
        lines
            .iter()
            .map(|l| l.split_whitespace().next().unwrap().to_string())
            .collect()
    };
    assert_eq!(ids(&["takes", project]), ["hook", "intro-1", "intro-2"]);
    assert_eq!(
        ids(&["takes", project, "--cut", "long"]),
        ["intro-1", "intro-2"]
    );
    assert_eq!(
        ids(&["takes", project, "--cut", "vertical"]),
        ["hook", "intro-1", "intro-2"]
    );
    // The long cut's takes are laid out although the hook has none.
    for id in ["intro-1", "intro-2"] {
        let listing = String::from_utf8(vidni(&["takes", project]).stdout).unwrap();
        let row = listing.lines().find(|l| l.starts_with(id)).unwrap();
        let file = dir.path().join(row.split_whitespace().nth(2).unwrap());
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, id.as_bytes()).unwrap();
    }
    let links = dir.path().join("links");
    let link = |cut: &str, to: &Path| {
        vidni(&[
            "takes",
            project,
            "--cut",
            cut,
            "--link",
            to.to_str().unwrap(),
        ])
    };
    let out = link("long", &links);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(fs::read_dir(&links).unwrap().count(), 2);
    assert_eq!(fs::read(links.join("000.wav")).unwrap(), b"intro-1");
    let out = link("vertical", &dir.path().join("links2"));
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).ends_with("no current take: hook\n"));
    let out = vidni(&["takes", project, "--cut", "wide"]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "vidni: there is no cut \"wide\"\n"
    );
}

#[test]
fn status_ratify_and_unratify_work_from_the_hashes() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().to_str().unwrap();
    fs::write(dir.path().join("vidni.toml"), vidni_core::TEMPLATE).unwrap();
    fs::write(dir.path().join("page.html"), "<!doctype html>").unwrap();
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
            // The page is the project's own, so its capture comes with
            // the record of the files it was made from.
            let file = c.file.clone().unwrap();
            let sha = vidni_core::capture::file_sha256(dir.path(), &file);
            let made_from = [(file, sha)].into();
            fs::write(
                vidni_core::capture::files_path(dir.path(), &c.key),
                vidni_core::capture::files_json(&made_from),
            )
            .unwrap();
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
        stderr.starts_with("vidni: 1 of 1 captures isn't taken yet (the first: line main-1); "),
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

/// Runs vidni in the folder `dir`; its exit code, stdout and stderr.
fn vidni_in(dir: &Path, args: &[&str]) -> (Option<i32>, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_vidni"))
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn new_starts_a_project_that_is_valid_and_in_the_canonical_layout() {
    let work = tempfile::tempdir().unwrap();
    let (code, stdout, stderr) = vidni_in(work.path(), &["new", "my-demo"]);
    assert_eq!(code, Some(0), "{stderr}");
    assert_eq!(
        stdout,
        "my-demo: a new project, \"My demo\"\n  \
         my-demo/vidni.toml  2 lines in 1 section, 1 scene, 2 cuts\n  \
         my-demo/page.html   the page its scene shows\n  \
         my-demo/.gitignore  keeps build/, out/ and takes/ out of git\n\
         next: `vidni draft my-demo` renders it as it is\n"
    );
    let (code, stdout, _) = vidni_in(work.path(), &["validate", "my-demo"]);
    assert_eq!(code, Some(0));
    assert_eq!(
        stdout,
        "my-demo/vidni.toml: ok: 2 lines in 1 section, 1 scene, 2 cuts\n"
    );
    assert_eq!(
        vidni_in(work.path(), &["fmt", "--check", "my-demo"]).0,
        Some(0)
    );
    // Its sheet lists the lines to read; nothing is spoken or recorded yet.
    let (code, sheet, stderr) = vidni_in(work.path(), &["plan", "my-demo"]);
    assert_eq!(code, Some(0), "{stderr}");
    assert_eq!(
        sheet,
        "My demo: the recording sheet\n\
         2 lines in 1 section, 9 words; 0 recorded, 0 stale, 2 to record; \
         the draft voice hasn't spoken them yet\n\
         \n\
         Introduction\n  \
         [ ] intro-1        -  Hello, and welcome.\n  \
         [ ] intro-2        -  This video was made with vidni.\n"
    );
    // A folder that is there is left alone, and a name must be an ID.
    let (code, _, stderr) = vidni_in(work.path(), &["new", "my-demo"]);
    assert_eq!(
        (code, stderr.as_str()),
        (Some(1), "vidni: my-demo already exists\n")
    );
    let (code, _, stderr) = vidni_in(work.path(), &["new", "My Demo"]);
    assert_eq!(code, Some(1));
    assert!(stderr.ends_with("\"My Demo\" isn't\n"), "{stderr}");
    assert!(!work.path().join("My Demo").exists());
}

#[test]
fn new_takes_its_script_from_an_outline() {
    let work = tempfile::tempdir().unwrap();
    fs::write(
        work.path().join("outline.md"),
        "# A tour\n\nWelcome.\n\n## The parts\n\n- The first part.\n- The second part.\n",
    )
    .unwrap();
    let (code, stdout, stderr) = vidni_in(work.path(), &["new", "tour", "--from", "outline.md"]);
    assert_eq!(code, Some(0), "{stderr}");
    assert!(
        stdout.starts_with(
            "tour: a new project, \"A tour\"\n  tour/vidni.toml  3 lines in 2 sections, 1 scene, 2 cuts\n"
        ),
        "{stdout}"
    );
    let (_, sheet, _) = vidni_in(work.path(), &["plan", "tour", "--cut", "long"]);
    assert!(
        sheet.ends_with(
            "(intro)\n  [ ] intro-1            -  Welcome.\n\n\
             The parts\n  [ ] the-parts-1        -  The first part.\n  \
             [ ] the-parts-2        -  The second part.\n"
        ),
        "{sheet}"
    );
    // An outline that can't be read, or that has no line, makes no project.
    let (code, _, stderr) = vidni_in(work.path(), &["new", "other", "--from", "gone.md"]);
    assert_eq!(code, Some(2), "{stderr}");
    assert!(stderr.starts_with("vidni: gone.md: "), "{stderr}");
    fs::write(work.path().join("empty.md"), "# Only a title\n").unwrap();
    let (code, _, stderr) = vidni_in(work.path(), &["new", "other", "--from", "empty.md"]);
    assert_eq!(
        (code, stderr.as_str()),
        (
            Some(1),
            "vidni: the outline has no paragraph or list item to make a line of\n"
        )
    );
    assert!(!work.path().join("other").exists());
}

#[test]
fn plan_marks_what_is_recorded_and_lists_the_cuts_asked_for() {
    let work = tempfile::tempdir().unwrap();
    assert_eq!(vidni_in(work.path(), &["new", "demo"]).0, Some(0));
    let dir = work.path().join("demo");
    // A take where `vidni takes` says the first line's goes, and one of
    // other words for the second.
    let (_, takes, _) = vidni_in(&dir, &["takes"]);
    let first = takes.lines().next().unwrap();
    let take = dir.join(first.split_whitespace().nth(2).unwrap());
    fs::create_dir_all(take.parent().unwrap()).unwrap();
    fs::write(&take, b"a take").unwrap();
    let other = dir.join("takes/intro-2/0000000000000000.wav");
    fs::create_dir_all(other.parent().unwrap()).unwrap();
    fs::write(&other, b"a take of other words").unwrap();
    let (code, sheet, stderr) = vidni_in(&dir, &["plan", "--cut", "vertical"]);
    assert_eq!(code, Some(0), "{stderr}");
    assert_eq!(
        sheet,
        "Demo: the recording sheet\n\
         2 lines in 1 section, 9 words; 1 recorded, 1 stale, 0 to record; \
         the draft voice hasn't spoken them yet\n\
         \n\
         Introduction\n  \
         [x] intro-1        -  Hello, and welcome.\n  \
         [~] intro-2        -  This video was made with vidni.\n"
    );
    // The sheet is only read: it leaves the project folder as it was.
    assert!(!dir.join("build").exists() && !dir.join("voice.lock").exists());
    let (code, _, stderr) = vidni_in(&dir, &["plan", "--cut", "short"]);
    assert_eq!(
        (code, stderr.as_str()),
        (Some(1), "vidni: there is no cut \"short\"\n")
    );
    assert_eq!(vidni_in(work.path(), &["plan", "gone"]).0, Some(2));
}

#[test]
fn record_says_what_it_cant_do_before_it_takes_the_terminal() {
    let work = tempfile::tempdir().unwrap();
    assert_eq!(vidni_in(work.path(), &["new", "demo"]).0, Some(0));
    let dir = work.path().join("demo");
    for (args, said) in [
        (
            &["record", "--section", "outro"][..],
            "vidni: there is no section \"outro\"\n",
        ),
        (
            &["record", "--only", "intro-9"][..],
            "vidni: there is no line \"intro-9\" in the cuts to record\n",
        ),
        (
            &["record", "--cut", "short"][..],
            "vidni: there is no cut \"short\"\n",
        ),
    ] {
        let (code, _, stderr) = vidni_in(&dir, args);
        assert_eq!((code, stderr.as_str()), (Some(1), said), "{args:?}");
    }
    // It reads single keys, so it needs a terminal; here it has none.
    let (code, _, stderr) = vidni_in(&dir, &["record"]);
    assert_eq!(
        (code, stderr.as_str()),
        (
            Some(2),
            "vidni: vidni record reads single keys: run it in a terminal\n"
        )
    );
    assert!(!dir.join("takes").exists());
}

#[test]
fn a_backup_lists_the_assets_copies_them_twice_and_restores_them() {
    let work = tempfile::tempdir().unwrap();
    assert_eq!(vidni_in(work.path(), &["new", "demo"]).0, Some(0));
    let dir = work.path().join("demo");
    // Takes where `vidni takes` says they go, a raw take, and what vidni
    // could make again.
    let (_, takes, _) = vidni_in(&dir, &["takes"]);
    for line in takes.lines().take(2) {
        let take = dir.join(line.split_whitespace().nth(2).unwrap());
        fs::create_dir_all(take.parent().unwrap().join("raw")).unwrap();
        fs::write(&take, line.as_bytes()).unwrap();
        fs::write(take.parent().unwrap().join("raw/abcd-1.wav"), b"raw").unwrap();
    }
    fs::create_dir_all(dir.join("build/draft")).unwrap();
    fs::write(dir.join("build/draft/long.mp4"), b"a draft").unwrap();

    // A backup goes to two places, and each must be there already.
    let (near, far) = (work.path().join("disk"), work.path().join("elsewhere"));
    let (near, far) = (near.to_str().unwrap(), far.to_str().unwrap());
    let (code, _, stderr) = vidni_in(&dir, &["backup", "--to", near]);
    assert_eq!(code, Some(1));
    assert!(
        stderr.starts_with("vidni: a backup goes to two places, "),
        "{stderr}"
    );
    let (code, _, stderr) = vidni_in(&dir, &["backup", "--to", near, "--to", near]);
    assert_eq!(code, Some(1), "{stderr}");
    fs::create_dir(near).unwrap();
    let (code, _, stderr) = vidni_in(&dir, &["backup", "--to", near, "--to", far]);
    assert_eq!(code, Some(2));
    assert_eq!(
        stderr,
        format!("vidni: {far} isn't there: is the disk plugged in, or the folder mounted?\n")
    );
    assert!(!dir.join("assets.lock").exists() && !Path::new(far).exists());
    // Nothing is listed yet, so there is nothing to check against.
    let (code, _, stderr) = vidni_in(&dir, &["verify"]);
    assert_eq!(code, Some(2));
    assert!(
        stderr.ends_with("assets.lock lists no asset; `vidni backup` writes it\n"),
        "{stderr}"
    );

    fs::create_dir(far).unwrap();
    let (code, stdout, stderr) = vidni_in(&dir, &["backup", "--to", near, "--to", far]);
    assert_eq!(code, Some(0), "{stderr}");
    assert_eq!(
        stdout,
        format!(
            "./assets.lock: 4 assets, written afresh\n\
             {near}/demo: 4 assets, 4 copied; 0 delivered files, 0 copied\n\
             {far}/demo: 4 assets, 4 copied; 0 delivered files, 0 copied\n"
        )
    );
    // Here both places are on the project's disk, which is said.
    assert!(
        stderr.contains("warning: every place is on the disk the project is on"),
        "{stderr}"
    );
    // The list is one sha256sum can check.
    let lock = fs::read_to_string(dir.join("assets.lock")).unwrap();
    assert_eq!(lock.lines().count(), 4);
    assert!(
        lock.lines().all(|l| l.len() > 66 && &l[64..66] == "  "),
        "{lock}"
    );
    // The places can come from the environment instead.
    let backup = format!("{near}:{far}");
    let with_places = |args: &[&str]| {
        let out = Command::new(env!("CARGO_BIN_EXE_vidni"))
            .args(args)
            .current_dir(&dir)
            .env("VIDNI_BACKUP", &backup)
            .output()
            .unwrap();
        (
            out.status.code(),
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )
    };
    let (code, stdout, _) = with_places(&["verify"]);
    assert_eq!(code, Some(0));
    assert_eq!(
        stdout,
        format!(
            ".: 4 of 4 assets as listed\n\
             {near}/demo: 4 of 4 assets as listed\n\
             {far}/demo: 4 of 4 assets as listed\n"
        )
    );

    // A take is lost, and another is recorded since the backup.
    let lost = lock.lines().last().unwrap()[66..].to_string();
    fs::remove_file(dir.join(&lost)).unwrap();
    fs::write(
        dir.join("takes/intro-1/raw/abcd-2.wav"),
        b"a newer raw take",
    )
    .unwrap();
    let (code, stdout, stderr) = with_places(&["verify"]);
    assert_eq!(code, Some(1));
    assert_eq!(stderr, format!("vidni: missing: ./{lost}\n"));
    assert!(
        stdout.starts_with(".: 3 of 4 assets as listed\n"),
        "{stdout}"
    );
    assert!(
        stdout.contains(
            ".: 1 asset isn't listed yet (the first: takes/intro-1/raw/abcd-2.wav); \
             `vidni backup` lists and copies them\n"
        ),
        "{stdout}"
    );
    // Restored, it is back; restoring needs to be told from where.
    let (code, _, stderr) = vidni_in(&dir, &["restore"]);
    assert_eq!(code, Some(1));
    assert!(
        stderr.starts_with("vidni: name the backup to restore from"),
        "{stderr}"
    );
    let (code, stdout, stderr) = vidni_in(&dir, &["restore", "--from", far]);
    assert_eq!(code, Some(0), "{stderr}");
    assert_eq!(stdout, ".: 1 file restored, 3 there already\n");
    assert!(dir.join(&lost).is_file());

    // gc says what it would delete, then deletes it: the draft, and no take.
    let (code, stdout, _) = vidni_in(&dir, &["gc", "--dry-run"]);
    assert_eq!(code, Some(0));
    assert_eq!(
        stdout,
        "would delete 1 file of drafts, 0.0 MB (`vidni draft` builds them again)\n\
         .: would delete 1 file, 0.0 MB; takes, frozen captures and delivered files are never touched\n"
    );
    assert!(dir.join("build/draft/long.mp4").is_file());
    let (code, stdout, _) = vidni_in(&dir, &["gc"]);
    assert_eq!(code, Some(0));
    assert!(stdout.starts_with("deleted 1 file of drafts, "), "{stdout}");
    assert!(!dir.join("build").exists(), "the emptied folders go too");
    assert_eq!(with_places(&["verify"]).0, Some(0));
    let (_, stdout, _) = vidni_in(&dir, &["gc"]);
    assert_eq!(
        stdout,
        ".: deleted 0 files, 0.0 MB; takes, frozen captures and delivered files are never touched\n"
    );
}

#[test]
fn nothing_vidni_ships_runs_python_or_node() {
    // P3-VIDNI-07: the reference pipeline is retired. The crates that make
    // a video start no interpreter and no script of the reference's; only
    // the parity harness, a development tool, still runs the reference.
    fn sources(dir: &Path, found: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                sources(&path, found);
            } else if path.extension().is_some_and(|e| e == "rs") {
                found.push(path);
            }
        }
    }
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut files = Vec::new();
    for shipped in [
        "vidni-core",
        "vidni-media",
        "vidni-record",
        "vidni-cli",
        "vidni-app",
    ] {
        sources(&crates.join(shipped).join("src"), &mut files);
    }
    assert!(files.len() > 30, "{} source files", files.len());
    let started = [
        "\"python",
        "\"node\"",
        "\"nodejs\"",
        "\"npm\"",
        "\"npx\"",
        ".py\"",
        ".mjs\"",
        "build.sh",
    ];
    for file in files {
        let text = fs::read_to_string(&file).unwrap();
        for (n, line) in text.lines().enumerate() {
            // A comment may name the reference; code may not start it.
            let code = line.split("//").next().unwrap_or_default();
            for word in started {
                assert!(
                    !code.contains(word),
                    "{}:{}: {word} in {line:?}",
                    file.display(),
                    n + 1
                );
            }
        }
    }
}

/// A raw take: half a second of silence, `seconds` of a tone at `hz` and
/// `level` of full scale, and half a second of silence, at 48 kHz.
fn raw_take(path: &Path, hz: f64, level: f64, seconds: f64) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 48000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec).unwrap();
    let tone = (seconds * 48000.0) as u32;
    for n in 0..24000 + tone + 24000 {
        let t = f64::from(n) / 48000.0;
        let sample = match (24000..24000 + tone).contains(&n) {
            true => (t * hz * std::f64::consts::TAU).sin() * level * 32767.0,
            false => 0.0,
        };
        writer.write_sample(sample as i16).unwrap();
    }
    writer.finalize().unwrap();
}

#[test]
fn picking_an_older_take_changes_only_its_line() {
    // P4-VIDNI-04: a kept take is cut from a raw one, by a record beside it.
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().to_str().unwrap();
    fs::write(dir.path().join("vidni.toml"), vidni_core::TEMPLATE).unwrap();
    fs::write(dir.path().join("page.html"), "<!doctype html>").unwrap();
    let run = |args: &[&str]| {
        let mut all = vec!["takes", project];
        all.extend(args);
        let out = vidni(&all);
        (
            out.status.code(),
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )
    };
    // Where each line's kept take goes says where its raw takes go.
    let listing = run(&[]).1;
    let kept: Vec<PathBuf> = listing
        .lines()
        .take(2)
        .map(|l| dir.path().join(l.split_whitespace().nth(2).unwrap()))
        .collect();
    let raw = |line: usize, n: u32| {
        let stem = kept[line].file_stem().unwrap().to_str().unwrap();
        kept[line]
            .parent()
            .unwrap()
            .join(format!("raw/{stem}-{n}.wav"))
    };
    // Two raw takes of the first line, and one of the second.
    raw_take(&raw(0, 1), 300.0, 0.3, 1.0);
    raw_take(&raw(0, 2), 500.0, 0.3, 1.5);
    raw_take(&raw(1, 1), 400.0, 0.3, 1.0);
    let (code, said, err) = run(&["--line", "intro-1", "--pick", "2"]);
    assert_eq!(code, Some(0), "{err}");
    // vidni keeps 0.15 s before the words and 0.25 s after them.
    assert!(
        said.starts_with("intro-1: its kept take is raw take 2, from 0.35 s to 2.25 s: 1.90 s\n"),
        "{said}"
    );
    assert_eq!(run(&["--line", "intro-2", "--pick", "1"]).0, Some(0));
    assert!(run(&[]).1.ends_with("2 current, 0 stale, 0 missing\n"));

    // Every file among the takes, with its bytes.
    let files = || -> Vec<(String, Vec<u8>)> {
        let mut found = Vec::new();
        let mut folders = vec![dir.path().join("takes")];
        while let Some(folder) = folders.pop() {
            for entry in fs::read_dir(&folder).unwrap() {
                let path = entry.unwrap().path();
                match path.is_dir() {
                    true => folders.push(path),
                    false => found.push((
                        path.strip_prefix(dir.path()).unwrap().display().to_string(),
                        fs::read(&path).unwrap(),
                    )),
                }
            }
        }
        found.sort();
        found
    };
    let changed = |before: &[(String, Vec<u8>)], after: &[(String, Vec<u8>)]| -> Vec<String> {
        assert_eq!(before.len(), after.len());
        before
            .iter()
            .zip(after)
            .filter(|(b, a)| b != a)
            .map(|(b, _)| b.0.rsplit('/').next().unwrap().to_string())
            .collect()
    };
    let with_the_second = files();
    assert_eq!(
        with_the_second.len(),
        7,
        "three raw takes, two kept, two records"
    );
    let name = |path: &Path| path.file_name().unwrap().to_str().unwrap().to_string();
    let (take, record) = (name(&kept[0]), name(&kept[0].with_extension("json")));

    // Pick the older take: the line's kept take and its record change,
    // and nothing else among the takes does.
    let (code, said, _) = run(&["--line", "intro-1", "--pick", "1"]);
    assert_eq!(code, Some(0));
    assert!(
        said.contains("raw take 1, from 0.35 s to 1.75 s: 1.40 s"),
        "{said}"
    );
    let with_the_first = files();
    assert_eq!(
        changed(&with_the_second, &with_the_first),
        [record.clone(), take.clone()]
    );
    // The kept take is the raw one's samples, with nothing done to them
    // but the cut and the rumble filter: picked again, it is the same bytes.
    assert_eq!(run(&["--line", "intro-1", "--pick", "2"]).0, Some(0));
    assert_eq!(files(), with_the_second);

    // Cut it elsewhere: the same raw take, other ends, on record as set.
    let (code, said, _) = run(&["--line", "intro-1", "--trim", "0.6:1.4"]);
    assert_eq!(code, Some(0));
    assert!(
        said.contains("raw take 2, from 0.60 s to 1.40 s: 0.80 s"),
        "{said}"
    );
    assert_eq!(
        changed(&with_the_second, &files()),
        [record.clone(), take.clone()]
    );
    let pick = fs::read_to_string(kept[0].with_extension("json")).unwrap();
    assert!(
        pick.contains("\"raw\": 2") && pick.contains("\"trim\": \"set\""),
        "{pick}"
    );
    let (_, said, _) = run(&["--line", "intro-1"]);
    let rows: Vec<&str> = said.lines().collect();
    assert!(rows[0].starts_with("intro-1: "), "{said}");
    assert!(rows[1].starts_with("   1  takes/intro-1/raw/") && rows[1].contains("2.00 s, peak "));
    assert!(rows[2].starts_with("*  2  takes/intro-1/raw/") && rows[2].contains("2.50 s, peak "));
    assert!(
        rows[2].ends_with("recorded before sessions were noted"),
        "{said}"
    );
    assert!(
        rows[3].ends_with("raw take 2 from 0.60 s to 1.40 s (where --trim put its ends)"),
        "{said}"
    );

    // What can't be done says why, and changes nothing.
    let before = files();
    let (code, _, err) = run(&["--line", "intro-1", "--pick", "9"]);
    assert_eq!(code, Some(1));
    assert_eq!(err, "vidni: line intro-1 has no raw take 9; it has 1, 2\n");
    let (code, _, err) = run(&["--line", "intro-1", "--trim", "soon"]);
    assert_eq!(code, Some(1));
    assert!(err.contains("--trim takes FROM:TO in seconds"), "{err}");
    let (code, _, err) = run(&["--line", "intro-1", "--trim", "1.0:1.2"]);
    assert_eq!(code, Some(1));
    assert!(err.contains("nothing was left"), "{err}");
    let (code, _, err) = run(&["--line", "nowhere"]);
    assert_eq!(
        (code, err.as_str()),
        (Some(1), "vidni: there is no line \"nowhere\"\n")
    );
    assert_eq!(files(), before);

    // Sessions: a line recorded on another day, 12 dB quieter, is named.
    let session = |day: &str| {
        format!(
            "{{\"session\": \"{day}T10:00:00Z\", \"recorded\": \"{day}T10:05:00Z\", \
             \"microphone\": \"a microphone\", \"rate\": 48000}}"
        )
    };
    fs::write(raw(0, 2).with_extension("json"), session("2026-11-10")).unwrap();
    raw_take(&raw(1, 2), 400.0, 0.075, 1.0);
    fs::write(raw(1, 2).with_extension("json"), session("2026-11-12")).unwrap();
    assert_eq!(run(&["--line", "intro-2", "--pick", "2"]).0, Some(0));
    let (code, _, err) = run(&[]);
    assert_eq!(code, Some(0));
    assert!(
        err.starts_with(
            "vidni: warning: the takes of the session of 2026-11-12T10:00:00Z are 12.1 dB \
             quieter than those of 2026-11-10T10:00:00Z: intro-2; "
        ),
        "{err}"
    );
    let (_, said, _) = run(&["--line", "intro-2"]);
    assert!(
        said.contains("recorded 2026-11-12T10:05:00Z from a microphone"),
        "{said}"
    );
}
