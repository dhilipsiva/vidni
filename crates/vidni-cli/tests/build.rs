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

/// A made-up YouTube on this machine: it answers each request with the
/// next reply, and hands over each request's first line.
fn made_up_youtube(replies: usize) -> (String, std::sync::mpsc::Receiver<String>) {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let session = format!("{base}/upload/session/1");
    let (seen, requests) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for n in 0..replies {
            let Ok((mut stream, _)) = listener.accept() else {
                return;
            };
            let mut request = Vec::new();
            let mut block = [0u8; 65536];
            let (mut head_end, mut length) = (None, 0usize);
            loop {
                let got = stream.read(&mut block).unwrap_or(0);
                if got == 0 {
                    break;
                }
                request.extend_from_slice(&block[..got]);
                if head_end.is_none()
                    && let Some(at) = request.windows(4).position(|w| w == b"\r\n\r\n")
                {
                    head_end = Some(at + 4);
                    let head = String::from_utf8_lossy(&request[..at]).to_lowercase();
                    length = head
                        .lines()
                        .find_map(|l| l.strip_prefix("content-length:"))
                        .and_then(|v| v.trim().parse().ok())
                        .unwrap_or(0);
                    if head.contains("expect: 100-continue") {
                        let _ = stream.write_all(b"HTTP/1.1 100 Continue\r\n\r\n");
                    }
                }
                if head_end.is_some_and(|end| request.len() >= end + length) {
                    break;
                }
            }
            let first = String::from_utf8_lossy(&request);
            let _ = seen.send(format!(
                "{} ({} bytes)",
                first.lines().next().unwrap_or_default(),
                length
            ));
            let (headers, body) = match n % 3 {
                0 => (String::new(), r#"{"access_token":"t"}"#),
                1 => (format!("Location: {session}\r\n"), ""),
                _ => (String::new(), r#"{"id":"vid123"}"#),
            };
            let reply = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n{headers}Connection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(reply.as_bytes());
        }
    });
    (base, requests)
}

#[test]
#[ignore = "needs Piper and its voice, Chromium, ffmpeg and curl; run `just test-build`"]
fn an_approved_release_is_uploaded_once_its_name_is_typed() {
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    flow::through(binary);
    let work = tempfile::tempdir().unwrap();
    let dir = &work.path().join("video");
    fs::create_dir(dir).unwrap();
    fs::write(dir.join("vidni.toml"), PROJECT).unwrap();
    fs::write(dir.join("card.html"), CARD).unwrap();
    for step in [&["draft"][..], &["ratify"], &[]] {
        match step.is_empty() {
            true => flow::record(dir),
            false => {
                flow::ok(dir, step);
            }
        }
    }
    flow::ok(dir, &["final", "--cut", "long"]);
    flow::ok(dir, &["approve", "--cut", "long"]);
    let video = dir.join("out/youtube/long/long.mp4");
    let size = fs::metadata(&video).unwrap().len();

    // The person's credentials: outside the project, and theirs alone.
    let secret = work.path().join("youtube.json");
    fs::write(
        &secret,
        r#"{"client_id":"c","client_secret":"s","refresh_token":"r"}"#,
    )
    .unwrap();
    fs::set_permissions(&secret, fs::Permissions::from_mode(0o600)).unwrap();
    // `vidni upload`, with what the person types.
    let upload = |args: &[&str], typed: &str, api: &str, more: &[(&str, &str)]| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_vidni"));
        command
            .arg("upload")
            .arg(dir)
            .args(args)
            .env("VIDNI_YOUTUBE", &secret)
            .env("VIDNI_YOUTUBE_API", api)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        for (name, value) in more {
            command.env(name, value);
        }
        let mut child = command.spawn().unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(typed.as_bytes())
            .unwrap();
        let out = child.wait_with_output().unwrap();
        (
            out.status.code(),
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )
    };
    let nobody = "http://127.0.0.1:9";

    // Anything but the release's name stops it, and nothing is sent:
    // nobody listens where it would have gone.
    for typed in ["", "yes\n", "long\n"] {
        let (code, out, err) = upload(&["--cut", "long"], typed, nobody, &[]);
        assert_eq!(code, Some(1), "{typed:?}: {out}{err}");
        assert!(out.contains("  release  long-r1\n"), "{out}");
        assert!(
            out.contains("type the release's name to upload it"),
            "{out}"
        );
        assert_eq!(err, "vidni: nothing was uploaded\n");
    }
    assert!(!dir.join("published.json").exists());

    // The name typed: a token, a session, the file; and the upload is
    // recorded with its release.
    let (api, requests) = made_up_youtube(3);
    let (code, out, err) = upload(&["--cut", "long"], "long-r1\n", &api, &[]);
    assert_eq!(code, Some(0), "{out}{err}");
    assert!(
        out.contains(&format!(
            "long: long-r1 is on YouTube, private, at {api}/watch/vid123\n"
        )),
        "{out}"
    );
    let sent: Vec<String> = requests.iter().take(3).collect();
    assert!(sent[0].starts_with("POST /token "), "{sent:?}");
    assert!(
        sent[1].starts_with(
            "POST /upload/youtube/v3/videos?uploadType=resumable&part=snippet,status "
        ),
        "{sent:?}"
    );
    assert_eq!(
        sent[2],
        format!("PUT /upload/session/1 HTTP/1.1 ({size} bytes)")
    );
    let listed = flow::ok(dir, &["published"]);
    assert!(
        listed.contains("long-r1") && listed.contains("/watch/vid123"),
        "{listed}"
    );
    // The same release isn't uploaded twice.
    let (code, _, err) = upload(&["--cut", "long"], "long-r1\n", nobody, &[]);
    assert_eq!(code, Some(1));
    assert!(
        err.starts_with("vidni: long-r1 is published on youtube already, at "),
        "{err}"
    );

    // What can't be uploaded is said before anything is asked.
    let refused = |args: &[&str], more: &[(&str, &str)]| {
        let (code, out, err) = upload(args, "long-r1\n", nobody, more);
        assert_eq!(code, Some(1), "{args:?}: {out}");
        assert!(!out.contains("type the release's name"), "{args:?}: {out}");
        err
    };
    // A cut with no release.
    let err = refused(&["--cut", "short", "--platform", "youtube-shorts"], &[]);
    assert!(err.starts_with("vidni: cut short has no release;"), "{err}");
    // Another platform, and a visibility that isn't one.
    let err = refused(&["--cut", "long", "--platform", "linkedin"], &[]);
    assert_eq!(
        err,
        "vidni: vidni uploads to YouTube only; linkedin is uploaded by hand\n"
    );
    let err = refused(&["--cut", "long", "--privacy", "everyone"], &[]);
    assert!(err.contains("isn't a visibility"), "{err}");
    // An assistant.
    let err = refused(&["--cut", "long"], &[("VIDNI_ASSISTANT", "1")]);
    assert!(
        err.starts_with("vidni: an assistant never uploads"),
        "{err}"
    );

    // Credentials that others can read, or that are in the project, are
    // refused; so is a delivered file that isn't the approved one.
    fs::remove_file(dir.join("published.json")).unwrap();
    fs::set_permissions(&secret, fs::Permissions::from_mode(0o644)).unwrap();
    let err = refused(&["--cut", "long"], &[]);
    assert!(
        err.contains("can be read by others on this machine"),
        "{err}"
    );
    fs::set_permissions(&secret, fs::Permissions::from_mode(0o600)).unwrap();
    let inside = dir.join("youtube.json");
    fs::copy(&secret, &inside).unwrap();
    let err = refused(
        &["--cut", "long"],
        &[("VIDNI_YOUTUBE", inside.to_str().unwrap())],
    );
    assert!(err.contains("is inside the project"), "{err}");
    fs::remove_file(&inside).unwrap();
    let whole = fs::read(&video).unwrap();
    fs::write(&video, &whole[..whole.len() - 1]).unwrap();
    let err = refused(&["--cut", "long"], &[]);
    assert!(
        err.contains("isn't the file that was approved as long-r1"),
        "{err}"
    );
}
