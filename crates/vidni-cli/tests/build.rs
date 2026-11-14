//! A small project from draft to approved through the `vidni` command
//! (TODO.md, P3-VIDNI-06), with the real tools: Piper and its pinned voice,
//! Chromium, ffmpeg, unshare and taskset, as the dev shell has them
//! (`just test-build`).

use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

#[path = "shared/flow.rs"]
mod flow;
use flow::{CARD, PROJECT, ok_in, vidni};

/// Runs the `vidni` binary.
fn binary(args: &[&str], cwd: Option<&Path>) -> (Option<i32>, String, String) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vidni"));
    command.args(args);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    let out = command.output().unwrap();
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
#[ignore = "needs Piper and its voice, Chromium, ffmpeg, unshare and taskset; run `just test-build`"]
fn a_project_goes_from_draft_to_approved_and_its_builds_repeat() {
    flow::a_project_goes_from_draft_to_approved_and_its_builds_repeat(binary);
}

#[test]
#[ignore = "needs Piper and its voice, ffmpeg and unshare; run `just test-build`"]
fn offline_a_page_on_the_web_isnt_captured() {
    flow::through(binary);
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
    flow::through(binary);
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

#[test]
#[ignore = "needs Piper and its voice, Chromium, ffmpeg and unshare; run `just test-build`"]
fn a_new_project_drafts_as_it_is() {
    flow::through(binary);
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
