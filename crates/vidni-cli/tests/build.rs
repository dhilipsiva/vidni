//! A small project from draft to approved through the `vidni` command
//! (TODO.md, P3-VIDNI-06), with the real tools: Piper and its pinned voice,
//! Chromium, ffmpeg, unshare and taskset, as the dev shell has them
//! (`just test-build`).

use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};
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
    assert_eq!(names(&dir.join("out")).len(), 7);
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
/// A PNG's size and its pixels as rgb24, read by ffmpeg.
fn image_of(png: &Path) -> ((u32, u32), Vec<u8>) {
    let probe = Command::new("ffprobe")
        .args(["-v", "error", "-show_entries", "stream=width,height"])
        .args(["-of", "csv=p=0"])
        .arg(png)
        .output()
        .unwrap();
    let size: Vec<u32> = String::from_utf8_lossy(&probe.stdout)
        .trim()
        .split(',')
        .map(|n| n.parse().unwrap())
        .collect();
    let raw = Command::new("ffmpeg")
        .args(["-v", "error", "-i"])
        .arg(png)
        .args(["-f", "rawvideo", "-pix_fmt", "rgb24", "-"])
        .output()
        .unwrap();
    assert!(raw.status.success(), "{}", png.display());
    ((size[0], size[1]), raw.stdout)
}

#[test]
#[ignore = "needs ffmpeg; run `just test-build`"]
fn the_grid_is_a_picture_and_a_clip_to_measure_an_app_with() {
    let work = tempfile::tempdir().unwrap();
    let said = ok_in(work.path(), &["platforms", "--grid", "grid"]);
    assert!(
        said.starts_with("grid/grid-1080x1920.png: 1080x1920"),
        "{said}"
    );
    let picture = image_of(&work.path().join("grid/grid-1080x1920.png"));
    assert_eq!(picture.0, (1080, 1920));
    let probe = Command::new("ffprobe")
        .args(["-v", "error", "-show_entries"])
        .args(["stream=codec_name,width,height,nb_frames", "-of", "csv=p=0"])
        .arg(work.path().join("grid/grid-1080x1920.mp4"))
        .output()
        .unwrap();
    let streams = String::from_utf8_lossy(&probe.stdout);
    let streams: Vec<&str> = streams.lines().collect();
    // 15 s at 30 fps, and a sound track for apps that want one.
    assert_eq!(streams[0], "h264,1080,1920,450", "{streams:?}");
    assert!(streams[1].starts_with("aac,"), "{streams:?}");
    // Nothing but the two files is left.
    let mut left: Vec<String> = fs::read_dir(work.path().join("grid"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    left.sort();
    assert_eq!(left, ["grid-1080x1920.mp4", "grid-1080x1920.png"]);
}

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
    // P6-VIDNI-02: each cut has a review frame for its one section, for
    // each platform it is made for, and nothing it draws is under an app.
    assert!(!drafted.contains("warning"), "{drafted}");
    let review = work.path().join("demo/build/draft/review");
    for frame in [
        "long/youtube/00-intro.png",
        "vertical/youtube-shorts/00-intro.png",
        "vertical/instagram-reels/00-intro.png",
        "vertical/whatsapp-status/00-intro.png",
    ] {
        assert!(review.join(frame).is_file(), "{frame}");
    }
    // Shorts' app covers the frame's edges and WhatsApp's none, so the
    // two frames differ, and only at the edges.
    let png = |frame: &str| image_of(&review.join(frame));
    let (shorts, plain) = (
        png("vertical/youtube-shorts/00-intro.png"),
        png("vertical/whatsapp-status/00-intro.png"),
    );
    assert_eq!(shorts.0, (1080, 1920));
    let pixel = |image: &((u32, u32), Vec<u8>), x: usize, y: usize| {
        let at = (y * 1080 + x) * 3;
        image.1[at..at + 3].to_vec()
    };
    assert_ne!(pixel(&shorts, 540, 100), pixel(&plain, 540, 100));
    assert_ne!(pixel(&shorts, 540, 1600), pixel(&plain, 540, 1600));
    for (x, y) in [(100, 300), (540, 700), (880, 1240)] {
        assert_eq!(pixel(&shorts, x, y), pixel(&plain, x, y), "({x}, {y})");
    }

    // Its sheet now has the draft voice's pace for each line.
    let sheet = ok_in(work.path(), &["plan", "demo"]);
    assert!(
        sheet.contains(" in the draft voice, without the pauses\n"),
        "{sheet}"
    );

    // P2-VIDNI-09: a second draft with nothing changed renders nothing.
    let started = Instant::now();
    let again = ok_in(work.path(), &["draft", "demo"]);
    assert!(started.elapsed() < Duration::from_secs(10));
    assert!(again.contains("2 from the cache"), "{again}");
    assert!(
        again.contains("demo/vidni.toml: every capture is taken\n"),
        "{again}"
    );
    // Its page is a file of the project's: once that changes, the stills
    // are taken again, and the cuts that show them rendered.
    let page = work.path().join("demo/page.html");
    let text = fs::read_to_string(&page).unwrap();
    fs::write(&page, text.replace("<h1>Demo</h1>", "<h1>A demo</h1>")).unwrap();
    let before = fs::read(work.path().join("demo/build/draft/long.mp4")).unwrap();
    let redrafted = ok_in(work.path(), &["draft", "demo"]);
    assert!(
        redrafted.contains("demo/vidni.toml: 2 captures taken\n"),
        "{redrafted}"
    );
    assert_ne!(
        fs::read(work.path().join("demo/build/draft/long.mp4")).unwrap(),
        before
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
