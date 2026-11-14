//! The preview player (TODO.md, P7-VIDNI-03): a click on a line lands on
//! the frame its cue starts at, and going through a five-minute cut never
//! holds the window up. Both need ffmpeg: `just test-app`.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};
use vidni_app::preview::{Line, Player};
use vidni_media::encode::{Encoder, Settings};

/// A cut of `frames` frames, `w` by `h`, in which every frame is another
/// picture: bars that spell the frame's number.
fn cut(dir: &Path, (w, h): (u32, u32), frames: u64) -> PathBuf {
    let audio = dir.join("tone.wav");
    let made = Command::new("ffmpeg")
        .args(["-nostdin", "-loglevel", "error", "-y", "-f", "lavfi", "-i"])
        .arg(format!(
            "sine=frequency=440:duration={}:sample_rate=48000",
            frames / 30 + 1
        ))
        .args(["-ac", "2"])
        .arg(&audio)
        .status()
        .unwrap();
    assert!(made.success());
    let mp4 = dir.join("cut.mp4");
    let mut encoder = Encoder::start(&Settings::new(w, h), &audio, &mp4).unwrap();
    let bar = w / 16;
    for n in 0..frames {
        let mut frame = Vec::with_capacity((w * h * 3) as usize);
        for _ in 0..h {
            for x in 0..w {
                let level = if n >> (x / bar).min(15) & 1 == 1 {
                    235
                } else {
                    40
                };
                frame.extend([level, level, level]);
            }
        }
        encoder.push(&frame).unwrap();
    }
    encoder.finish().unwrap();
    mp4
}

fn line(id: &str, start: f64, end: f64) -> Line {
    Line {
        id: id.into(),
        show: format!("The words of {id}."),
        start,
        end,
        section: "main".into(),
    }
}

/// Steps the window until the player shows the frame it is at.
fn settle(harness: &mut Harness<'_, Player>) -> Duration {
    let began = Instant::now();
    loop {
        harness.step();
        let player = harness.state();
        if player.shown == Some(player.position) {
            return began.elapsed();
        }
        assert!(player.failed.is_none(), "{:?}", player.failed);
        assert!(began.elapsed() < Duration::from_secs(20), "no frame came");
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn window<'a>(player: Player) -> Harness<'a, Player> {
    let mut harness = Harness::builder()
        .with_size([1100.0, 1400.0])
        .build_ui_state(|ui, player: &mut Player| player.ui(ui), player);
    settle(&mut harness);
    harness
}

#[test]
#[ignore = "needs ffmpeg; run `just test-app`"]
fn a_click_on_a_line_lands_within_a_frame_of_its_cue() {
    let dir = tempfile::tempdir().unwrap();
    let mp4 = cut(dir.path(), (320, 192), 600);
    // Cues at moments that are no frame's own start.
    let lines = vec![
        line("a-1", 0.6, 3.2),
        line("a-2", 3.4871, 6.9),
        line("b-1", 7.013, 12.0),
        line("b-2", 12.5049, 19.0),
    ];
    let mut harness = window(Player::open(&mp4, lines.clone(), || {}).unwrap());
    assert_eq!(harness.state().position, 0);
    for (i, line) in lines.iter().enumerate().rev() {
        let name = format!("[{}]", line.id);
        harness
            .query_by_label_contains(&name)
            .unwrap_or_else(|| panic!("no row for {}", line.id))
            .click();
        settle(&mut harness);
        let player = harness.state();
        let at = player.video.time_of(player.position);
        // The frame that is showing when the cue starts: it began no
        // later than the cue, and less than a frame before it.
        assert!(
            at <= line.start + 1e-6 && line.start - at < 1.0 / 30.0,
            "{}: frame {} begins at {at}, and the cue at {}",
            line.id,
            player.position,
            line.start
        );
        assert_eq!(player.shown, Some(player.position));
        assert_eq!(player.line_at(), Some(i));
    }
    // Before the first line, what is before is the start of the cut.
    harness.state_mut().seek(5);
    settle(&mut harness);
    assert_eq!(harness.state().line_at(), None);
    harness.get_by_label("Previous line").click();
    settle(&mut harness);
    assert_eq!(harness.state().position, 0);
    harness.get_by_label("Next line").click();
    settle(&mut harness);
    assert_eq!(harness.state().line_at(), Some(0));
    // From the first line, on to the next and back.
    harness.get_by_label("Next line").click();
    settle(&mut harness);
    assert_eq!(harness.state().line_at(), Some(1));
    assert_eq!(harness.state().position, (3.4871f64 * 30.0).floor() as u64);
    harness.get_by_label("Previous line").click();
    settle(&mut harness);
    assert_eq!(harness.state().line_at(), Some(0));
    // The contact sheet has a picture of every line, and a click on one goes there.
    harness.get_by_label("Contact sheet").click();
    harness.step();
    let began = Instant::now();
    let button = egui::accesskit::Role::Button;
    // The pictures come one at a time; the last line's comes last.
    while harness.query_by_role_and_label(button, "[b-2]").is_none() {
        harness.step();
        assert!(
            began.elapsed() < Duration::from_secs(20),
            "the contact sheet didn't fill"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(harness.state().sheet);
    harness.get_by_label("[b-1]").click();
    settle(&mut harness);
    assert!(!harness.state().sheet);
    assert_eq!(harness.state().line_at(), Some(2));

    // Playing goes on from there with the clock, and Pause stops it.
    let from = harness.state().position;
    harness.get_by_label("Play").click();
    let began = Instant::now();
    while began.elapsed() < Duration::from_millis(1200) {
        harness.step();
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(harness.state().playing());
    harness.get_by_label("Pause").click();
    harness.step();
    let played = harness.state().position - from;
    assert!(!harness.state().playing());
    // 1.2 seconds is 36 frames; the first frame takes a moment to come.
    assert!((20..=40).contains(&played), "{played} frames in 1.2 s");
    assert_eq!(harness.state().shown, Some(harness.state().position));
}

#[test]
#[ignore = "needs ffmpeg; run `just test-app`"]
fn going_through_a_five_minute_cut_never_holds_the_window_up() {
    let dir = tempfile::tempdir().unwrap();
    // Five minutes: 9,000 frames. `VIDNI_PREVIEW_CUT` names a real cut to
    // go through in its place.
    let mp4 = match std::env::var_os("VIDNI_PREVIEW_CUT") {
        Some(real) => PathBuf::from(real),
        None => cut(dir.path(), (640, 360), 9000),
    };
    let mut harness = window(Player::open(&mp4, vec![line("a-1", 0.0, 300.0)], || {}).unwrap());
    let frames = harness.state().video.frames;
    assert!(frames >= 8990, "{frames} frames isn't five minutes");

    // A drag across the whole cut in five seconds: a new position and a
    // frame of the window every 16 ms. No request and no frame of the
    // window waits for the decoder, and the picture follows the drag at
    // the decoder's pace.
    let mut slowest = Duration::ZERO;
    let before = harness.state().decodes;
    let mut pictures = std::collections::BTreeSet::new();
    for step in 0..300u64 {
        let began = Instant::now();
        harness.state_mut().seek(step * (frames - 1) / 299);
        harness.step();
        slowest = slowest.max(began.elapsed());
        pictures.extend(harness.state().shown);
        std::thread::sleep(Duration::from_millis(16).saturating_sub(began.elapsed()));
    }
    assert!(
        slowest < Duration::from_millis(50),
        "a seek and a frame of the window took {slowest:?}"
    );
    // A picture several times a second, each of a place the drag passed.
    assert!(
        pictures.len() >= 15,
        "{} pictures were shown during a drag of five seconds",
        pictures.len()
    );
    // It ends on the last position asked for, having decoded far fewer
    // frames than were asked for.
    settle(&mut harness);
    assert_eq!(harness.state().shown, Some(frames - 1));
    let decoded = harness.state().decodes - before;
    assert!(
        (15..150).contains(&decoded),
        "{decoded} frames were decoded for 300 requests"
    );
    println!(
        "{}: {} pictures during a five-second drag, {decoded} frames decoded",
        mp4.display(),
        pictures.len()
    );

    // Each place in the cut comes up within half a second, wherever it is.
    let mut waits = Vec::new();
    for n in 0..30u64 {
        let at = (n * 2_654_435_761) % frames;
        harness.state_mut().seek(at);
        waits.push(settle(&mut harness));
        assert_eq!(harness.state().shown, Some(at));
    }
    waits.sort();
    let (median, worst) = (waits[waits.len() / 2], waits[waits.len() - 1]);
    println!(
        "{}: median {median:?}, worst {worst:?} to show a frame",
        mp4.display()
    );
    assert!(
        worst < Duration::from_millis(500),
        "the slowest frame took {worst:?}"
    );
}
