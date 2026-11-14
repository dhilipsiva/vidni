//! A small project from draft to approved (TODO.md, P3-VIDNI-06), with
//! the real tools. It is run twice: by the `vidni` binary
//! (`tests/build.rs`, `just test-build`), and with its gates driven
//! through the desktop app's command layer
//! (`crates/vidni-app/tests/gates.rs`, `just test-app`), which must give
//! the same words, codes and files (TODO.md, P7-VIDNI-05).

use std::fs;
use std::path::Path;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use vidni_core::{takes, voices};

/// Runs `vidni <args>`, in the folder `cwd` when one is given; its exit
/// code, stdout and stderr.
pub type Run = fn(&[&str], Option<&Path>) -> (Option<i32>, String, String);

static RUN: OnceLock<Run> = OnceLock::new();

/// Has every command of these tests run by `run`.
pub fn through(run: Run) {
    RUN.get_or_init(|| run);
}

fn runner() -> Run {
    *RUN.get().expect("`through` names what runs vidni")
}

/// Three lines on two cards. The long cut has them all, and the short the
/// last two.
pub const PROJECT: &str = r#"schema = 1

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

pub const CARD: &str = r##"<!doctype html>
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
pub fn vidni(dir: &Path, args: &[&str]) -> (Option<i32>, String, String) {
    let mut all = vec![args[0], dir.to_str().unwrap()];
    all.extend(&args[1..]);
    runner()(&all, None)
}

/// Runs vidni in the folder `dir` and expects it to succeed; its stdout.
pub fn ok_in(dir: &Path, args: &[&str]) -> String {
    let (code, stdout, stderr) = runner()(args, Some(dir));
    assert_eq!(code, Some(0), "vidni {args:?}:\n{stdout}\n{stderr}");
    stdout
}

/// Runs vidni and expects it to succeed; its stdout.
pub fn ok(dir: &Path, args: &[&str]) -> String {
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
pub fn record(dir: &Path) {
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

pub fn a_project_goes_from_draft_to_approved_and_its_builds_repeat(run: Run) {
    through(run);
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
    // Drafted again with nothing changed, nothing is spoken, captured or
    // rendered, and it takes no time to speak of (P2-VIDNI-09).
    let long = dir.join("build/draft/long.mp4");
    let written = fs::metadata(&long).unwrap().modified().unwrap();
    let started = Instant::now();
    let (code, again, said) = vidni(dir, &["draft"]);
    let took = started.elapsed();
    assert_eq!(code, Some(0), "{again}\n{said}");
    assert!(took < Duration::from_secs(10), "{took:?}");
    assert!(again.contains("3 from the cache"), "{again}");
    assert!(again.contains("every capture is taken"), "{again}");
    assert!(again.contains("build/draft/long.mp4: "), "{again}");
    assert_eq!(
        said,
        "long: nothing has changed since it was built; nothing to render\n\
         short: nothing has changed since it was built; nothing to render\n"
    );
    assert_eq!(fs::metadata(&long).unwrap().modified().unwrap(), written);
    // One cut can be drafted alone, and rendered again when that is asked for.
    let (_, again, said) = vidni(dir, &["draft", "--cut", "short", "--again"]);
    assert!(!again.contains("long.mp4"), "{again}");
    assert!(said.contains("short: short.mp4: "), "{said}");
    assert_eq!(fs::metadata(&long).unwrap().modified().unwrap(), written);

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
            "instagram-reels",
            "long.captioned.mp4",
            "long.en.srt",
            "long.en.vtt",
            "long.mp4",
            "short.en.srt",
            "short.en.vtt",
            "short.mp4",
            "whatsapp-status",
            "youtube",
            "youtube-shorts"
        ]
    );
    // P6-VIDNI-04: each platform has the upload of each cut made for it.
    assert_eq!(
        names(&dir.join("out/youtube/long")),
        [
            "long.en.srt",
            "long.en.vtt",
            "long.mp4",
            "metadata.md",
            "thumbnail-1.png"
        ]
    );
    assert_eq!(
        names(&dir.join("out/instagram-reels/short")),
        ["cover.png", "metadata.md", "short.mp4"]
    );
    assert!(
        built.contains(
            "out/youtube-shorts/short: the upload for youtube-shorts, with its metadata.md\n"
        ),
        "{built}"
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
    // P6-VIDNI-07: once a cut is published, what a line got wrong is
    // corrected beside the video. Both cuts say main-2.
    let said = ok(
        dir,
        &[
            "published",
            "--cut",
            "long",
            "--platform",
            "youtube",
            "--url",
            "https://youtu.be/abc",
        ],
    );
    assert_eq!(
        said,
        "long: long-r1 is published on youtube at https://youtu.be/abc\n  \
         `vidni final` writes it into the platform's UPLOAD.md\n"
    );
    assert!(
        refused(
            dir,
            &[
                "published",
                "--cut",
                "long",
                "--platform",
                "tiktok",
                "--url",
                "https://x"
            ]
        )
        .contains("cut long isn't made for tiktok")
    );
    let before = fs::read_to_string(dir.join("vidni.toml")).unwrap();
    let unchanged = |name: &str| {
        fs::metadata(dir.join("build/final").join(name))
            .unwrap()
            .modified()
            .unwrap()
    };
    let rendered = (unchanged("long.mp4"), unchanged("short.mp4"));
    let corrected = format!(
        "{before}\n[[errata]]\nline = \"main-2\"\non = \"2026-11-20\"\ntext = \"The draft voice speaks second, not first.\"\n"
    );
    fs::write(dir.join("vidni.toml"), corrected).unwrap();
    ok(dir, &["final"]);
    // Nothing is rendered, and both cuts are still what was approved.
    assert_eq!((unchanged("long.mp4"), unchanged("short.mp4")), rendered);
    assert_eq!(
        ok(dir, &["status"]),
        "long   ratified; approved as long-r1\nshort  ratified; approved as short-r1\n"
    );
    // Every description of a cut that says the line has the correction,
    // with where in that cut the line is.
    let block = |file: &str| {
        let text = fs::read_to_string(dir.join("out").join(file)).unwrap();
        let at = text
            .find("Corrections:\n")
            .unwrap_or_else(|| panic!("{file}:\n{text}"));
        text[at..].lines().nth(1).unwrap().to_string()
    };
    let in_long = block("youtube/long/metadata.md");
    assert!(
        in_long.ends_with(" (2026-11-20): The draft voice speaks second, not first."),
        "{in_long}"
    );
    for short in ["youtube-shorts", "instagram-reels", "whatsapp-status"] {
        let in_short = block(&format!("{short}/short/metadata.md"));
        assert!(
            in_short.starts_with("0:00 (2026-11-20): The draft voice"),
            "{in_short}"
        );
        // The line comes later in the long cut, after main-1.
        assert_ne!(in_short, in_long);
    }
    // Where the line is, and where it is published.
    let whereabouts = ok_in(dir, &["where", "main-2"]);
    let lines: Vec<&str> = whereabouts.lines().collect();
    assert_eq!(
        lines[0],
        "main-2: Each of its lines is spoken by the draft voice first."
    );
    assert!(
        lines[1].starts_with("  long: at 0:0") && lines[1].ends_with("; newest release long-r1"),
        "{whereabouts}"
    );
    assert_eq!(lines[2], "    long-r1 on youtube: https://youtu.be/abc");
    assert_eq!(lines[3], "  short: at 0:00; newest release short-r1");
    assert_eq!(lines[4], "    not published");
    assert!(
        lines[5].starts_with("  1 correction to it in [[errata]]"),
        "{whereabouts}"
    );
    assert_eq!(ok_in(dir, &["where", "main-1"]).lines().count(), 3);
    // Each platform's UPLOAD.md lists its cuts, and says what YouTube can't do.
    let upload = fs::read_to_string(dir.join("out/youtube/UPLOAD.md")).unwrap();
    assert!(
        upload.contains(
            "- `long/`: release long-r1\n  - published as long-r1: https://youtu.be/abc\n"
        ),
        "{upload}"
    );
    assert!(
        upload.contains("YouTube can't replace the file of a published video"),
        "{upload}"
    );
    assert!(
        fs::read_to_string(dir.join("out/whatsapp-status/UPLOAD.md"))
            .unwrap()
            .contains("- `short/`: release short-r1\n  - not published yet\n")
    );
    assert_eq!(ok(dir, &["published"]).lines().count(), 1);
    // The rest of this test goes on from the project as it was.
    fs::write(dir.join("vidni.toml"), before).unwrap();
    fs::remove_file(dir.join("published.json")).unwrap();
    ok(dir, &["final"]);

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
    let offline = ok(dir, &["final", "--offline", "--again", "--cut", "short"]);
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

    // The card's file is what its renders were made from. Once it changes,
    // the cuts that show it aren't the ratified ones, and a draft renders
    // the card again.
    fs::write(
        dir.join("card.html"),
        CARD.replace("\"card \"", "\"Card \""),
    )
    .unwrap();
    let status = ok(dir, &["status"]);
    assert!(
        status.starts_with(
            "long   changed: card.html has changed since 2 of its captures were taken; "
        ),
        "{status}"
    );
    assert_eq!(
        refused(dir, &["final", "--cut", "long"]),
        "vidni: cut long has changed since it was ratified: \
         card.html has changed since 2 of its captures were taken\n"
    );
    let (code, redrafted, said) = vidni(dir, &["draft"]);
    assert_eq!(code, Some(0), "{redrafted}\n{said}");
    assert!(
        redrafted.contains("vidni.toml: 4 captures taken\n"),
        "{redrafted}"
    );
    assert!(
        said.contains("long: long.mp4: ") && said.contains("short: short.mp4: "),
        "{said}"
    );
    let status = ok(dir, &["status"]);
    assert!(status.starts_with("long   changed: capture "), "{status}");
    assert!(status.contains(" changed on disk"), "{status}");
    // With the file as it was, a draft renders the card as it was: the
    // cuts are the ratified ones again, and their releases stand.
    fs::write(dir.join("card.html"), CARD).unwrap();
    let restored = ok(dir, &["draft"]);
    assert!(
        restored.contains("vidni.toml: 4 captures taken\n"),
        "{restored}"
    );
    assert_eq!(
        ok(dir, &["status"]),
        "long   ratified; approved as long-r1\nshort  ratified; approved as short-r1\n"
    );

    // P4-VIDNI-05: the assets are backed up to two places. With what can
    // be made again deleted, and then the assets wiped too, a restore and
    // a final build give the very cuts that were approved.
    ok(dir, &["final"]);
    let places = tempfile::tempdir().unwrap();
    let (near, far) = (places.path().join("disk"), places.path().join("elsewhere"));
    for place in [&near, &far] {
        fs::create_dir(place).unwrap();
    }
    let to = [
        "--to",
        near.to_str().unwrap(),
        "--to",
        far.to_str().unwrap(),
    ];
    let backed = ok(dir, &[&["backup"][..], &to[..]].concat());
    // Three takes, and the four card renders the cuts froze, each with
    // the record of its file; and what the two releases delivered.
    assert!(
        backed.contains("assets.lock: 11 assets, written afresh\n"),
        "{backed}"
    );
    let name = dir.file_name().unwrap();
    assert!(
        backed.contains(&format!(
            "{}: 11 assets, 11 copied; 7 delivered files, 7 copied\n",
            far.join(name).display()
        )),
        "{backed}"
    );
    let verified = ok(dir, &[&["verify"][..], &to[..]].concat());
    assert!(
        verified.contains("11 of 11 assets as listed; holds long-r1 and short-r1\n"),
        "{verified}"
    );
    // What can be made again goes, and nothing else.
    let would = ok(dir, &["gc", "--dry-run"]);
    assert!(would.contains("would delete 3 draft clips, "), "{would}");
    assert!(dir.join("build/voice").is_dir());
    let deleted = ok(dir, &["gc"]);
    assert!(deleted.contains("deleted 3 draft clips, "), "{deleted}");
    assert!(deleted.contains(" files of final builds, "), "{deleted}");
    for gone in ["build/voice", "build/draft", "build/final"] {
        assert!(!dir.join(gone).exists(), "{gone}");
    }
    ok(dir, &["verify"]);
    // The seven delivered files, and the four platforms' folders.
    assert_eq!(names(&dir.join("out")).len(), 11);
    // Then the assets themselves, and what was delivered.
    for wiped in ["takes", "build", "out"] {
        fs::remove_dir_all(dir.join(wiped)).unwrap();
    }
    let (code, _, missing) = vidni(dir, &["verify"]);
    assert_eq!(code, Some(1), "{missing}");
    assert!(missing.contains("vidni: missing: "), "{missing}");
    assert!(
        refused(dir, &["final", "--cut", "short"]).contains("went missing"),
        "a cut whose frozen captures are gone can't be built"
    );
    // One backup is enough to restore from; the other has lost a take.
    let lost = far.join(name).join("takes");
    fs::remove_dir_all(&lost).unwrap();
    let from = [
        "--from",
        far.to_str().unwrap(),
        "--from",
        near.to_str().unwrap(),
    ];
    let restored = ok(dir, &[&["restore"][..], &from[..]].concat());
    assert!(
        restored.contains(": 18 files restored, 0 there already\n"),
        "{restored}"
    );
    ok(dir, &["verify"]);
    // Built again from what was restored, each cut is the release that
    // was approved: the same T1 and T2 hashes as release.json holds.
    let rebuilt = ok(dir, &["final"]);
    assert!(rebuilt.contains("build/final/long.mp4: "), "{rebuilt}");
    for cut in ["long", "short"] {
        let already = ok(dir, &["approve", "--cut", cut]);
        assert!(
            already.starts_with(&format!("{cut}: already approved as {cut}-r1, on ")),
            "{already}"
        );
    }
    assert_eq!(vidni_core::release::read(dir).unwrap(), releases);

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
