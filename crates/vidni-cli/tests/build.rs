//! A small project from draft to approved through the `vidni` command
//! (TODO.md, P3-VIDNI-06), with the real tools: Piper and its pinned voice,
//! Chromium, ffmpeg, unshare and taskset, as the dev shell has them
//! (`just test-build`).

use std::fs;
use std::path::Path;
use std::process::Command;
use vidni_core::{takes, voices};

/// Three lines on two cards. The long cut has them all, and the short the
/// last two.
const PROJECT: &str = r#"schema = 1

[meta]
title = "Hello vidni"
subtitle = "a small project"
default_scene = "card"

[[cuts]]
id = "long"
aspect = "16:9"

[[cuts]]
id = "short"
aspect = "9:16"
lines = ["main-2", "main-3"]

[sections.main]
label = "hello"

[[lines]]
id = "main-1"
section = "main"
show = "This is a small project, made to test a build."

[[lines]]
id = "main-2"
section = "main"
show = "Each of its lines is spoken by the draft voice first."
scene = "other"

[[lines]]
id = "main-3"
section = "main"
show = "Then a take is recorded, and the final cut is built from it."

[scenes.card]
kind = "page"
url = "card.html?c=1&r={aspect}"

[scenes.other]
kind = "page"
url = "card.html?c=2&r={aspect}"
"#;

const CARD: &str = r##"<!doctype html>
<meta charset="utf-8">
<style>
  html, body { margin: 0; height: 100%; }
  body { display: grid; place-items: center; font: 600 8vmin "IBM Plex Sans", sans-serif; color: #f4f3ef; }
</style>
<body>
<script>
  const q = new URLSearchParams(location.search);
  document.body.style.background = q.get("c") === "2" ? "#1d3557" : "#0b0b10";
  document.body.textContent = "card " + q.get("c") + " · " + q.get("r");
</script>
</body>
"##;

/// Runs vidni on the project in `dir`, or on the project file `dir`; its
/// exit code, stdout and stderr.
fn vidni(dir: &Path, args: &[&str]) -> (Option<i32>, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_vidni"))
        .arg(args[0])
        .arg(dir)
        .args(&args[1..])
        .output()
        .unwrap();
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// Runs vidni and expects it to succeed; its stdout.
fn ok(dir: &Path, args: &[&str]) -> String {
    let (code, stdout, stderr) = vidni(dir, args);
    assert_eq!(code, Some(0), "vidni {args:?}:\n{stdout}\n{stderr}");
    stdout
}

/// Runs vidni and expects it to refuse; its stderr.
fn refused(dir: &Path, args: &[&str]) -> String {
    let (code, stdout, stderr) = vidni(dir, args);
    assert_eq!(code, Some(1), "vidni {args:?}:\n{stdout}\n{stderr}");
    stderr
}

/// Records each line: its draft clip stands in for the creator's take.
fn record(dir: &Path) {
    let project = vidni_core::validate::check(PROJECT, Some(dir)).unwrap();
    let lock = voices::read_lock(dir).unwrap();
    let pack = voices::find(&project.voice.draft, None).unwrap().unwrap();
    for (line, take) in project.lines.iter().zip(takes::status(&project, dir)) {
        let clip = voices::drafted(dir, &lock, &pack, &project.voice, line).unwrap();
        fs::create_dir_all(take.path.parent().unwrap()).unwrap();
        fs::copy(clip, take.path).unwrap();
    }
}

fn names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
#[ignore = "needs Piper and its voice, Chromium, ffmpeg, unshare and taskset; run `just test-build`"]
fn a_project_goes_from_draft_to_approved_and_its_builds_repeat() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    fs::write(dir.join("vidni.toml"), PROJECT).unwrap();
    fs::write(dir.join("card.html"), CARD).unwrap();

    // A draft speaks the lines, takes the captures and renders each cut.
    let drafted = ok(dir, &["draft"]);
    assert!(
        drafted.contains("draft voice piper:en_US-joe-medium: 3 lines; 3 chosen afresh"),
        "{drafted}"
    );
    // Two cards for the long cut, and each again at the short's size.
    assert!(
        drafted.contains("vidni.toml: 4 captures taken\n"),
        "{drafted}"
    );
    for cut in [
        "build/draft/long.mp4: ",
        "build/draft/short.mp4: ",
        " frames; 1920x1080 h264 30/1 fps; aac 48000 Hz 2 ch; ",
        " frames; 1080x1920 h264 30/1 fps; aac 48000 Hz 2 ch; ",
    ] {
        assert!(drafted.contains(cut), "{cut}:\n{drafted}");
    }
    assert!(dir.join("build/draft/long.mp4").is_file());
    assert!(!dir.join("out").exists(), "a draft delivers nothing");
    // Drafted again, nothing is spoken or captured anew.
    let again = ok(dir, &["draft", "--cut", "short"]);
    assert!(again.contains("3 from the cache"), "{again}");
    assert!(again.contains("every capture is taken"), "{again}");
    assert!(!again.contains("long.mp4"), "{again}");

    // The final build has its gates: ratified cuts, then takes.
    let unratified = refused(dir, &["final"]);
    assert!(
        unratified.contains("vidni: cut long isn't ratified; `vidni ratify --cut long` freezes it"),
        "{unratified}"
    );
    ok(dir, &["ratify"]);
    assert_eq!(
        refused(dir, &["final"]),
        "vidni: cut long: 3 of its 3 lines have no take: main-1, main-2, main-3\n\
         vidni: cut short: 2 of its 2 lines have no take: main-2, main-3\n"
    );
    assert!(
        !dir.join("build/final").exists(),
        "nothing is rendered for a refused build"
    );
    record(dir);

    // Built, the cuts are delivered: the long one clean and captioned, as
    // its default asks, each with its caption files.
    let built = ok(dir, &["final"]);
    assert!(
        built.contains(&format!(
            "long: delivered to {}; `vidni approve --cut long` releases it\n",
            dir.join("out").display()
        )),
        "{built}"
    );
    assert_eq!(
        names(&dir.join("out")),
        [
            "long.captioned.mp4",
            "long.en.srt",
            "long.en.vtt",
            "long.mp4",
            "short.en.srt",
            "short.en.vtt",
            "short.mp4"
        ]
    );
    assert_eq!(ok(dir, &["status"]), "long   ratified\nshort  ratified\n");

    // Approved, each cut is a release with its hashes.
    let approved = ok(dir, &["approve"]);
    let today = vidni_core::release::today();
    assert!(
        approved.contains(&format!("long: approved as long-r1, on {today}\n")),
        "{approved}"
    );
    assert!(
        approved.contains(&format!("short: approved as short-r1, on {today}\n")),
        "{approved}"
    );
    assert!(approved.contains("  t2 file         "), "{approved}");
    let releases = vidni_core::release::read(dir).unwrap();
    assert_eq!(releases.latest("long").unwrap().build.outputs.len(), 2);
    assert_eq!(
        ok(dir, &["status"]),
        "long   ratified; approved as long-r1\nshort  ratified; approved as short-r1\n"
    );
    let already = ok(dir, &["approve", "--cut", "long"]);
    assert!(
        already.starts_with("long: already approved as long-r1, on "),
        "{already}"
    );

    // Two clean builds, at other paths and on other cores, make the same
    // files, and so did the build above.
    let repeated = ok(dir, &["repro"]);
    for line in [
        "long: the two builds agree\n",
        "short: the two builds agree\n",
        "  and they agree with the build in build/final\n",
    ] {
        assert!(repeated.contains(line), "{line}:\n{repeated}");
    }
    let verdict = repeated.lines().last().unwrap();
    assert!(
        verdict.starts_with("repro: T1 and T2 are equal across two builds, on cores "),
        "{repeated}"
    );
    // The drafts repeat too, spoken again from voice.lock.
    let drafts = ok(dir, &["repro", "--draft"]);
    assert!(
        drafts.contains("  and they agree with the build in build/draft\n"),
        "{drafts}"
    );

    // A change to what only the listening check reads is no change to a
    // cut: its release stands, and its build still repeats.
    let heard = PROJECT.replace(
        "[[cuts]]\nid = \"long\"",
        "[voice]\nheard_as = [{ heard = \"vidney\", means = \"vidni\" }]\n\n[[cuts]]\nid = \"long\"",
    );
    fs::write(dir.join("vidni.toml"), heard).unwrap();
    assert_eq!(
        ok(dir, &["status"]),
        "long   ratified; approved as long-r1\nshort  ratified; approved as short-r1\n"
    );
    let unchanged = ok(dir, &["repro", "--cut", "short"]);
    assert!(
        unchanged.contains("  and they agree with the build in build/final\n"),
        "{unchanged}"
    );
    fs::write(dir.join("vidni.toml"), PROJECT).unwrap();

    // None of it needs the network.
    let offline = ok(dir, &["final", "--offline", "--cut", "short"]);
    assert!(offline.contains("short: delivered to "), "{offline}");
    let (code, stdout, stderr) = vidni(dir, &["repro", "--offline", "--cut", "short"]);
    assert_eq!(code, Some(0), "{stdout}\n{stderr}");
    assert!(
        stderr.contains("repro: building twice, with no network: on cores "),
        "{stderr}"
    );
    assert!(
        stdout.contains("  and they agree with the build in build/final\n"),
        "{stdout}"
    );

    // repro builds the project file it is given, not the one beside it.
    let alt = dir.join("alt.toml");
    let slower = PROJECT.replace(
        "default_scene = \"card\"",
        "default_scene = \"card\"\n\n[timing]\ntail = 1.2",
    );
    fs::write(&alt, slower).unwrap();
    ok(&alt, &["final", "--cut", "short"]);
    let other = ok(&alt, &["repro", "--cut", "short"]);
    assert!(
        other.contains("  and they agree with the build in build/final\n"),
        "{other}"
    );
    // And it keeps its copies out of the project they are copies of.
    let kept = dir.join("kept");
    let (code, _, stderr) = vidni(
        dir,
        &["repro", "--cut", "short", "--keep", kept.to_str().unwrap()],
    );
    assert_eq!(code, Some(2), "{stderr}");
    assert!(
        stderr.contains("the copies can't be kept inside the project they are copies of"),
        "{stderr}"
    );
    assert!(!kept.exists());

    // New words for a line: the cuts that say it can't be built or
    // approved until they are ratified again, and a take is recorded.
    let reworded = PROJECT.replace("Then a take is recorded", "Then every take is recorded");
    fs::write(dir.join("vidni.toml"), reworded).unwrap();
    let changed = refused(dir, &["final", "--cut", "short"]);
    assert_eq!(
        changed,
        "vidni: cut short has changed since it was ratified: line main-3 changed\n\
         vidni: cut short: 1 of its 2 lines has a take only for other words: main-3\n"
    );
    assert_eq!(refused(dir, &["approve", "--cut", "short"]), changed);
    let status = ok(dir, &["status"]);
    assert!(
        status.contains(
            "short  changed: line main-3 changed; approved as short-r1, and changed since in "
        ),
        "{status}"
    );
}

#[test]
#[ignore = "needs Piper and its voice, ffmpeg and unshare; run `just test-build`"]
fn offline_a_page_on_the_web_isnt_captured() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    let web = PROJECT.replace(
        "kind = \"page\"\nurl = \"card.html?c=1&r={aspect}\"",
        "kind = \"web\"\nurl = \"https://example.com\"",
    );
    fs::write(dir.join("vidni.toml"), web).unwrap();
    fs::write(dir.join("card.html"), CARD).unwrap();
    let (code, stdout, stderr) = vidni(dir, &["draft", "--offline"]);
    assert_eq!(code, Some(1), "{stdout}\n{stderr}");
    assert!(
        stderr.ends_with(
            "vidni: offline, and 1 capture of web pages isn't taken yet (the first: line main-1)\n"
        ),
        "{stderr}"
    );
    assert!(!dir.join("build/captures").exists());
}

/// Runs vidni in the folder `dir` and expects it to succeed; its stdout.
fn ok_in(dir: &Path, args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_vidni"))
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(
        out.status.success(),
        "vidni {args:?}:\n{stdout}\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    stdout
}

#[test]
#[ignore = "needs Piper and its voice, Chromium, ffmpeg and unshare; run `just test-build`"]
fn a_new_project_drafts_as_it_is() {
    // P2-VIDNI-08: `vidni new demo && vidni draft demo` renders with no edits.
    let work = tempfile::tempdir().unwrap();
    ok_in(work.path(), &["new", "demo"]);
    let drafted = ok_in(work.path(), &["draft", "demo"]);
    // Its page is captured once for each of its two lines' highlights.
    assert!(
        drafted.contains("demo/vidni.toml: 2 captures taken\n"),
        "{drafted}"
    );
    for cut in [
        "demo/build/draft/long.mp4: ",
        "demo/build/draft/vertical.mp4: ",
    ] {
        assert!(drafted.contains(cut), "{cut}:\n{drafted}");
    }
    for file in ["long.mp4", "long.en.srt", "vertical.mp4"] {
        assert!(
            work.path().join("demo/build/draft").join(file).is_file(),
            "{file}"
        );
    }
    // Its sheet now has the draft voice's pace for each line.
    let sheet = ok_in(work.path(), &["plan", "demo"]);
    assert!(
        sheet.contains(" in the draft voice, without the pauses\n"),
        "{sheet}"
    );

    // And it needs no network: a second new project, from an outline,
    // drafts in a namespace that has none.
    fs::write(
        work.path().join("outline.md"),
        "# A tour\n\nWelcome to the tour.\n\n## The end\n\nThat was all of it.\n",
    )
    .unwrap();
    ok_in(work.path(), &["new", "tour", "--from", "outline.md"]);
    let offline = ok_in(work.path(), &["draft", "tour", "--offline"]);
    assert!(offline.contains("tour/build/draft/long.mp4: "), "{offline}");
}
