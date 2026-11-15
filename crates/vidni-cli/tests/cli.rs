//! The `vidni` binary's validate and schema commands.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[path = "shared/gates.rs"]
mod gates;
use gates::project_with_voice;

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
        "examples/nibli-syllogism/vidni.toml: ok: 59 lines in 10 sections, 7 scenes, 4 cuts\n"
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
    gates::status_ratify_and_unratify_work_from_the_hashes(vidni_without_fonts);
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
    gates::the_gates_refuse_before_any_tool_is_needed(vidni_without_fonts);
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
    gates::approve_exits_2_when_the_releases_cant_be_read(vidni_without_fonts);
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
    // A path isn't a name; the answer says how to make it there.
    let (code, _, stderr) = vidni_in(work.path(), &["new", "videos/tour"]);
    assert_eq!(code, Some(1));
    assert!(
        stderr.ends_with("vidni: to make tour in videos, run `vidni new tour --in videos`\n"),
        "{stderr}"
    );
    // And what it says to run does make it there.
    fs::create_dir(work.path().join("videos")).unwrap();
    let (code, _, stderr) = vidni_in(work.path(), &["new", "tour", "--in", "videos"]);
    assert_eq!(code, Some(0), "{stderr}");
    assert!(work.path().join("videos/tour/vidni.toml").is_file());
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

#[test]
fn a_command_run_in_process_says_the_same_and_gives_the_same_code() {
    // P7: the desktop app runs commands this way, on a thread of its own.
    use std::cell::RefCell;
    use std::rc::Rc;
    use vidni_cli::console::{self, Stream};
    let heard: Rc<RefCell<Vec<(Stream, String)>>> = Rc::default();
    let run = |args: &[&str]| {
        heard.borrow_mut().clear();
        let sink = heard.clone();
        let listening = console::listen(move |stream, line| {
            sink.borrow_mut().push((stream, line.to_string()));
        });
        let code = vidni_cli::run(args);
        drop(listening);
        (code, heard.borrow().clone())
    };
    let example = root().join("examples/hello");
    let example = example.to_str().unwrap();
    let (code, said) = run(&["validate", example]);
    let out = vidni(&["validate", example]);
    assert_eq!(Some(i32::from(code)), out.status.code());
    assert_eq!(said.len(), 1);
    assert_eq!(said[0].0, Stream::Out);
    assert_eq!(
        format!("{}\n", said[0].1),
        String::from_utf8_lossy(&out.stdout)
    );
    // A refusal: the same words on stderr, and the same code.
    let broken = root().join("crates/vidni-core/tests/fixtures/invalid/09-unknown-scene.toml");
    let broken = broken.to_str().unwrap();
    let (code, said) = run(&["validate", broken]);
    let out = vidni(&["validate", broken]);
    assert_eq!((code, out.status.code()), (1, Some(1)));
    let lines: Vec<String> = said.iter().map(|(_, line)| format!("{line}\n")).collect();
    assert!(said.iter().any(|(stream, _)| *stream == Stream::Err));
    assert_eq!(
        lines.concat(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stderr),
            String::from_utf8_lossy(&out.stdout)
        )
    );
    // What isn't a command is said, with the code the shell would get.
    let (code, said) = run(&["nonsense"]);
    assert_eq!(code, 2);
    assert!(said[0].1.contains("unrecognized subcommand"), "{said:?}");
    // Help is said on stdout, and is no failure.
    let (code, said) = run(&["platforms", "--help"]);
    assert_eq!(code, 0);
    assert!(said.iter().all(|(stream, _)| *stream == Stream::Out));
    assert!(
        said[0].1.starts_with("List the platform profiles"),
        "{said:?}"
    );
}

#[test]
fn notes_are_kept_by_line_and_changes_name_the_lines() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    assert!(vidni(&["new", "demo", "--in", path]).status.success());
    let project = dir.path().join("demo");
    let demo = project.to_str().unwrap();
    let said = |args: &[&str]| {
        let out = vidni(args);
        (
            out.status.code(),
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )
    };
    // No notes yet, and listing them writes no file.
    assert_eq!(
        said(&["notes", demo]),
        (
            Some(0),
            "no notes; `vidni notes --line ID --set TEXT` writes one\n".into(),
            String::new()
        )
    );
    assert!(!project.join("notes.md").exists());
    // A note on a line; a second on another; the first written again.
    for (line, note) in [
        ("intro-2", "Name the tool sooner."),
        ("intro-1", "Say it slower."),
        ("intro-2", "Name the tool first.\nThen what it does."),
    ] {
        let (code, out, err) = said(&["notes", demo, "--line", line, "--set", note]);
        assert_eq!((code, err.as_str()), (Some(0), ""), "{out}");
        assert_eq!(out, format!("{line}: noted in notes.md\n"));
    }
    assert_eq!(
        fs::read_to_string(project.join("notes.md")).unwrap(),
        "# Notes\n\nWhat to change, line by line. Each heading is a line's ID.\n\n\
         ## intro-2\n\nName the tool first.\nThen what it does.\n\n\
         ## intro-1\n\nSay it slower.\n"
    );
    // Listed in the file's order, a line each.
    assert_eq!(
        said(&["notes", demo]).1,
        "intro-2: Name the tool first. Then what it does.\nintro-1: Say it slower.\n"
    );
    // A note with a line that reads as a heading stays one note.
    let (code, ..) = said(&[
        "notes",
        demo,
        "--line",
        "intro-1",
        "--set",
        "Say it slower.\n## intro-2",
    ]);
    assert_eq!(code, Some(0));
    assert_eq!(
        said(&["notes", demo]).1,
        "intro-2: Name the tool first. Then what it does.\nintro-1: Say it slower.  ## intro-2\n"
    );
    let (code, ..) = said(&[
        "notes",
        demo,
        "--line",
        "intro-1",
        "--set",
        "Say it slower.",
    ]);
    assert_eq!(code, Some(0));
    // A line that isn't in the script has no note to write.
    let (code, _, err) = said(&["notes", demo, "--line", "nope", "--set", "x"]);
    assert_eq!(code, Some(1));
    assert_eq!(err, "vidni: there is no line \"nope\"\n");
    // An empty note takes the line's note away; what was written by hand stays.
    let by_hand = fs::read_to_string(project.join("notes.md"))
        .unwrap()
        .replace("# Notes\n", "# Notes\n\nWatched on the 14th.\n");
    fs::write(project.join("notes.md"), by_hand).unwrap();
    let (code, out, _) = said(&["notes", demo, "--line", "intro-2", "--set", ""]);
    assert_eq!(
        (code, out.as_str()),
        (Some(0), "intro-2: its note is taken out of notes.md\n")
    );
    let text = fs::read_to_string(project.join("notes.md")).unwrap();
    assert!(text.contains("Watched on the 14th.\n"), "{text}");
    assert!(!text.contains("intro-2"), "{text}");
    assert_eq!(said(&["notes", demo]).1, "intro-1: Say it slower.\n");

    // Before a build, there is nothing to compare with.
    let (code, out, _) = said(&["changes", demo]);
    assert_eq!(code, Some(0));
    assert_eq!(
        out,
        "since the last build: there is no build yet to compare with\n\
         since each cut was ratified:\n  long: not ratified\n  vertical: not ratified\n"
    );
}

#[test]
fn exports_lists_each_cuts_platforms_and_adds_and_removes_one() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    assert!(vidni(&["new", "demo", "--in", path]).status.success());
    let project = dir.path().join("demo");
    let demo = project.to_str().unwrap();
    let said = |args: &[&str]| {
        let out = vidni(args);
        (
            out.status.code(),
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )
    };
    let before = fs::read_to_string(project.join("vidni.toml")).unwrap();
    assert_eq!(
        said(&["exports", demo]).1,
        "long (16:9), for youtube\n  \
         no final build yet; `vidni final --cut long` builds and exports it\n\
         vertical (9:16), for youtube-shorts, instagram-reels, whatsapp-status\n  \
         no final build yet; `vidni final --cut vertical` builds and exports it\n"
    );
    // One platform more: that key, and every other byte as it was.
    assert_eq!(
        said(&["exports", demo, "--cut", "long", "--add", "linkedin"]),
        (
            Some(0),
            "long: now for youtube, linkedin; the next `vidni final` exports it for linkedin\n"
                .into(),
            String::new()
        )
    );
    assert_eq!(
        fs::read_to_string(project.join("vidni.toml")).unwrap(),
        before.replace(
            "id = \"long\"\naspect = \"16:9\"\n",
            "id = \"long\"\naspect = \"16:9\"\nplatforms = [\"youtube\", \"linkedin\"]\n"
        )
    );
    assert_eq!(
        said(&["exports", demo, "--cut", "long", "--add", "linkedin"]).1,
        "long: already for linkedin\n"
    );
    // One less; and a cut may be for none.
    assert_eq!(
        said(&["exports", demo, "--cut", "long", "--remove", "youtube"]).1,
        "long: now for linkedin; the next `vidni final` removes out/youtube/long\n"
    );
    assert_eq!(
        said(&["exports", demo, "--cut", "long", "--remove", "linkedin"]).1,
        "long: now for no platform; the next `vidni final` removes out/linkedin/long\n"
    );
    assert!(
        said(&["exports", demo, "--cut", "long"])
            .1
            .starts_with("long (16:9), for no platform\n")
    );
    // What can't be is refused, and the file stays.
    let held = fs::read_to_string(project.join("vidni.toml")).unwrap();
    for (args, why) in [
        (
            vec!["--cut", "long", "--add", "tiktok"],
            "vidni: tiktok takes 9:16 and cut long is 16:9\n",
        ),
        (
            vec!["--cut", "wide", "--add", "x"],
            "vidni: there is no cut \"wide\"\n",
        ),
    ] {
        let (code, _, err) = said(&[&["exports", demo][..], &args[..]].concat());
        assert_eq!((code, err.as_str()), (Some(1), why));
    }
    let (code, _, err) = said(&["exports", demo, "--cut", "long", "--add", "nope"]);
    assert_eq!(code, Some(1));
    assert!(
        err.starts_with("vidni: there is no platform \"nope\"; the platforms are youtube, "),
        "{err}"
    );
    assert_eq!(
        fs::read_to_string(project.join("vidni.toml")).unwrap(),
        held
    );
    assert_eq!(said(&["exports", demo, "--add", "x"]).0, Some(2));
}

#[test]
fn an_assistants_session_is_held_to_the_protocol() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    assert!(vidni(&["new", "demo", "--in", path]).status.success());
    let project = dir.path().join("demo");
    let demo = project.to_str().unwrap();
    // The same command, run by an assistant.
    let assistant = |args: &[&str]| {
        let out = Command::new(env!("CARGO_BIN_EXE_vidni"))
            .args(args)
            .env("VIDNI_ASSISTANT", "1")
            .output()
            .unwrap();
        (
            out.status.code(),
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )
    };
    // Without a session, validate says to start one.
    let (code, _, err) = assistant(&["validate", demo]);
    assert_eq!(code, Some(1));
    assert!(
        err.contains("there is no assistant's session on this project"),
        "{err}"
    );
    let started = vidni(&["assistant", demo]);
    assert!(started.status.success());
    let said = String::from_utf8_lossy(&started.stdout);
    assert!(said.contains("3 files noted as they are"), "{said}");
    assert_eq!(assistant(&["validate", demo]).0, Some(0));

    // What an assistant may change: the script, and the notes.
    let file = project.join("vidni.toml");
    let text = fs::read_to_string(&file).unwrap();
    fs::write(
        &file,
        text.replace("Hello, and welcome.", "Hello, everyone."),
    )
    .unwrap();
    let (code, ..) = assistant(&["notes", demo, "--line", "intro-1", "--set", "Warmer."]);
    assert_eq!(code, Some(0));
    let (code, _, err) = assistant(&["validate", demo]);
    assert_eq!((code, err.as_str()), (Some(0), ""));

    // What it may not: another file changed, one added, one removed, and
    // a gate's own file written by hand. Each is said, and validate fails.
    let page = fs::read_to_string(project.join("page.html")).unwrap();
    fs::write(project.join("page.html"), page.clone() + "<!-- more -->\n").unwrap();
    fs::write(project.join("extra.html"), "<p>new</p>").unwrap();
    fs::write(project.join("ratified.lock"), "cuts = {}\n").unwrap();
    let (code, _, err) = assistant(&["validate", demo]);
    assert_eq!(code, Some(1));
    for said in [
        "vidni: page.html was changed: an assistant changes vidni.toml and notes.md only\n",
        "vidni: extra.html is new, and an assistant adds no file\n",
        "vidni: ratified.lock is new, and an assistant adds no file\n",
        "3 changes that the assistant protocol forbids (docs/assistant.md)\n",
    ] {
        assert!(err.contains(said), "{said}\n{err}");
    }
    // A person's validate doesn't look at any of it.
    assert!(vidni(&["validate", demo]).status.success());
    // Put back, the session is whole again.
    fs::write(project.join("page.html"), page).unwrap();
    fs::remove_file(project.join("extra.html")).unwrap();
    fs::remove_file(project.join("ratified.lock")).unwrap();
    assert_eq!(assistant(&["validate", demo]).0, Some(0));
    let ignore = fs::read(project.join(".gitignore")).unwrap();
    fs::remove_file(project.join(".gitignore")).unwrap();
    let (code, _, err) = assistant(&["validate", demo]);
    assert_eq!(code, Some(1));
    assert!(err.contains("vidni: .gitignore was removed"), "{err}");
    fs::write(project.join(".gitignore"), ignore).unwrap();

    // The commands that are a person's refuse, and say whose they are.
    for (args, why) in [
        (
            vec!["ratify", demo],
            "ratifying a cut is a person's decision",
        ),
        (
            vec!["unratify", demo],
            "ratifying a cut is a person's decision",
        ),
        (
            vec!["approve", demo],
            "approving a release is a person's decision",
        ),
        (vec!["record", demo], "recording is the creator's own voice"),
        (
            vec!["takes", demo, "--line", "intro-1", "--pick", "1"],
            "choosing and trimming takes is the creator's",
        ),
        (
            vec![
                "published",
                demo,
                "--cut",
                "long",
                "--platform",
                "youtube",
                "--url",
                "https://x",
            ],
            "an assistant never uploads",
        ),
        (
            vec!["gc", demo],
            "moving and deleting assets is a person's to do",
        ),
    ] {
        let (code, out, err) = assistant(&args);
        assert_eq!((code, out.as_str()), (Some(1), ""), "{args:?}: {err}");
        assert!(
            err.starts_with(&format!("vidni: {why}")) && err.contains("VIDNI_ASSISTANT is set"),
            "{args:?}: {err}"
        );
    }
    assert!(!project.join("ratified.lock").exists());
    // What only reads is an assistant's too.
    for args in [
        vec!["status", demo],
        vec!["takes", demo],
        vec!["published", demo],
    ] {
        assert_eq!(assistant(&args).0, Some(0), "{args:?}");
    }
    // It can't start the session again, to pass off what it changed.
    let (code, _, err) = assistant(&["assistant", demo]);
    assert_eq!(code, Some(1));
    assert!(err.contains("only a person starts another"), "{err}");

    // A person ratifies meanwhile (here the lock is written as the command
    // would): the session notes the command's file, and stays whole.
    let (code, _, err) = {
        let out = vidni(&["ratify", demo]);
        (
            out.status.code(),
            String::new(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )
    };
    // With no captures the cut can't be ratified; nothing was written.
    assert_eq!(code, Some(1), "{err}");
    assert_eq!(assistant(&["validate", demo]).0, Some(0));
}

#[test]
fn signing_in_keeps_the_token_outside_any_project() {
    use std::io::{BufRead, BufReader, Read, Write};
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let secret = dir.path().join("youtube.json");
    fs::write(
        &secret,
        r#"{"client_id":"the client","client_secret":"s3cret"}"#,
    )
    .unwrap();
    fs::set_permissions(&secret, fs::Permissions::from_mode(0o600)).unwrap();
    // A made-up token service: one request, one token.
    let service = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let api = format!("http://{}", service.local_addr().unwrap());
    let asked = std::thread::spawn(move || {
        let (mut stream, _) = service.accept().unwrap();
        let mut request = vec![0u8; 8192];
        let n = stream.read(&mut request).unwrap();
        let body = r#"{"access_token":"a","refresh_token":"kept-for-later"}"#;
        let reply = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(reply.as_bytes()).unwrap();
        String::from_utf8_lossy(&request[..n]).into_owned()
    });
    let mut child = Command::new(env!("CARGO_BIN_EXE_vidni"))
        .args(["upload", "--login"])
        .env("VIDNI_YOUTUBE", &secret)
        .env("VIDNI_YOUTUBE_API", &api)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    // It says where to go, and waits for the browser to come back.
    let mut said = BufReader::new(child.stdout.take().unwrap());
    let mut lines = Vec::new();
    let back = loop {
        let mut line = String::new();
        assert!(said.read_line(&mut line).unwrap() > 0, "{lines:?}");
        if let Some(rest) = line.strip_prefix("waiting for the browser to come back to ") {
            break rest.split_whitespace().next().unwrap().to_string();
        }
        lines.push(line);
    };
    let address = lines
        .iter()
        .find(|l| l.contains("/auth?"))
        .expect("an address");
    assert!(address.contains("client_id=the%20client"), "{address}");
    assert!(
        address.contains("scope=https%3A%2F%2Fwww.googleapis.com%2Fauth%2Fyoutube.upload"),
        "{address}"
    );
    assert!(address.contains("access_type=offline"), "{address}");
    assert!(!address.contains("s3cret"), "{address}");
    // The browser comes back with a code.
    let mut browser = std::net::TcpStream::connect(back.trim_start_matches("http://")).unwrap();
    browser
        .write_all(b"GET /?code=4%2Fthe-code&scope=x HTTP/1.1\r\nHost: localhost\r\n\r\n")
        .unwrap();
    let mut page = String::new();
    browser.read_to_string(&mut page).unwrap();
    assert!(page.contains("vidni is signed in."), "{page}");
    let out = child.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    // The code was exchanged, with the client's secret, for a token …
    let exchange = asked.join().unwrap();
    assert!(exchange.starts_with("POST /token "), "{exchange}");
    assert!(exchange.contains("code=4%2Fthe-code"), "{exchange}");
    assert!(exchange.contains("client_secret=s3cret"), "{exchange}");
    assert!(
        exchange.contains("grant_type=authorization_code"),
        "{exchange}"
    );
    // … which is kept in the file, still the person's alone.
    let kept: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&secret).unwrap()).unwrap();
    assert_eq!(kept["refresh_token"], "kept-for-later");
    assert_eq!(kept["client_id"], "the client");
    assert_eq!(
        fs::metadata(&secret).unwrap().permissions().mode() & 0o777,
        0o600
    );

    // With no credentials file, it says where one goes and how to make it.
    let out = Command::new(env!("CARGO_BIN_EXE_vidni"))
        .args(["upload", "--login"])
        .env("VIDNI_YOUTUBE", dir.path().join("none.json"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("there are no YouTube credentials in"), "{err}");
    assert!(err.contains("docs/upload.md"), "{err}");
}
