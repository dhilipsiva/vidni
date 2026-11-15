//! The encoder gives the same packets for the same frames and samples (T2),
//! loudnorm reaches its target, and QA names each way a cut is wrong.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use vidni_core::chapters::Chapter;
use vidni_media::encode::{Encoder, Settings, Target, chapter_metadata, loudnorm};
use vidni_media::{framemd5, qa};

const W: u32 = 320;
const H: u32 = 180;
const FRAMES: u32 = 60;

/// A moving pattern, the same every run.
fn frame(n: u32) -> Vec<u8> {
    let mut out = Vec::with_capacity((W * H * 3) as usize);
    for y in 0..H {
        for x in 0..W {
            out.extend([
                (x + n * 4) as u8,
                (y * 2 + n) as u8,
                ((x ^ y) + n * 3) as u8,
            ]);
        }
    }
    out
}

/// Two seconds of a 440 Hz tone, mono, 48 kHz.
fn tone(dir: &Path) -> PathBuf {
    let path = dir.join("tone.wav");
    let status = Command::new("ffmpeg")
        .args([
            "-nostdin",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:duration=2:sample_rate=48000",
            "-ac",
            "1",
        ])
        .arg(&path)
        .status()
        .unwrap();
    assert!(status.success());
    path
}

fn encode(audio: &Path, out: &Path) {
    let mut encoder = Encoder::start(&Settings::new(W, H), audio, out).unwrap();
    for n in 0..FRAMES {
        encoder.push(&frame(n)).unwrap();
    }
    assert_eq!(encoder.finish().unwrap(), u64::from(FRAMES));
}

#[test]
fn the_same_frames_and_samples_give_the_same_packets_and_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let audio = tone(dir.path());
    let (a, b) = (dir.path().join("a.mp4"), dir.path().join("b.mp4"));
    encode(&audio, &a);
    encode(&audio, &b);
    assert_eq!(framemd5(&a).unwrap(), framemd5(&b).unwrap());
    assert_eq!(fs::read(&a).unwrap(), fs::read(&b).unwrap());
}

#[test]
fn the_container_carries_no_dates_or_version_numbers() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("a.mp4");
    encode(&tone(dir.path()), &out);
    no_dates_or_versions(&out);
}

fn no_dates_or_versions(out: &Path) {
    let probe = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format_tags:stream_tags",
            "-of",
            "json",
        ])
        .arg(out)
        .output()
        .unwrap();
    let info: serde_json::Value = serde_json::from_slice(&probe.stdout).unwrap();
    let mut tags: Vec<(String, String)> = Vec::new();
    for owner in std::iter::once(&info["format"]).chain(info["streams"].as_array().unwrap()) {
        for (k, v) in owner["tags"].as_object().into_iter().flatten() {
            tags.push((k.clone(), v.as_str().unwrap_or_default().to_string()));
        }
    }
    assert!(!tags.iter().any(|(k, _)| k == "creation_time"), "{tags:?}");
    // Under +bitexact the encoder tag names the codec ("Lavc libx264")
    // without a version such as 61.19.100.
    let versioned = |v: &str| {
        let b = v.as_bytes();
        b.windows(3)
            .any(|w| w[0].is_ascii_digit() && w[1] == b'.' && w[2].is_ascii_digit())
    };
    for (k, v) in &tags {
        assert!(!versioned(v), "{k} = {v}");
    }
}

#[test]
fn loudnorm_reaches_its_target_and_qa_passes_a_good_cut() {
    let dir = tempfile::tempdir().unwrap();
    let normalized = dir.path().join("tone-norm.wav");
    loudnorm(&tone(dir.path()), &normalized, Target::default()).unwrap();
    let out = dir.path().join("cut.mp4");
    encode(&normalized, &out);
    let srt = dir.path().join("cut.srt");
    fs::write(
        &srt,
        "1\n00:00:00,000 --> 00:00:01,000\nOne.\n\n2\n00:00:01,000 --> 00:00:02,000\nTwo.\n\n",
    )
    .unwrap();
    let expect = qa::Expect {
        width: W,
        height: H,
        fps: 30,
        duration: 2.0,
        loudness: -16.0,
        tolerance: 1.5,
        true_peak: -1.0,
        captions: Some((srt.clone(), 2)),
        chapters: Vec::new(),
    };
    let report = qa::check(&out, &expect).unwrap();
    assert_eq!(report.problems, Vec::<String>::new(), "{}", report.summary);
    // Each wrong expectation is named on its own.
    let wrong = qa::Expect {
        width: 1920,
        duration: 5.0,
        loudness: -23.0,
        captions: Some((srt, 3)),
        chapters: vec![Chapter {
            start: 0,
            title: "Intro".into(),
        }],
        ..expect
    };
    let problems = qa::check(&out, &wrong).unwrap().problems;
    assert_eq!(problems.len(), 7, "{problems:#?}");
    assert!(problems[0].starts_with("the picture is 320x180, not 1920x180"));
    assert!(problems[1].starts_with("it lasts 2.0"));
    assert!(problems[2].contains("more than 1.5 LU from -23 LUFS"));
    assert!(problems[3].ends_with("has 2 cues, and the timeline 3"));
    assert_eq!(
        problems[4],
        "the list has 1 of the 3 chapters that YouTube shows a list from"
    );
    assert!(problems[5].starts_with("chapter \"Intro\" lasts 2 s;"));
    assert_eq!(problems[6], "it has 0 chapters, not the 1 the cut defines");

    // Captions that a picture of this length can't show are named too.
    fs::write(
        dir.path().join("late.srt"),
        "1\n00:00:00,500 --> 00:00:01,500\nOne.\n\n2\n00:00:01,000 --> 00:00:03,000\nTwo.\n\n",
    )
    .unwrap();
    let late = qa::Expect {
        captions: Some((dir.path().join("late.srt"), 2)),
        ..qa::Expect {
            chapters: Vec::new(),
            ..wrong.clone()
        }
    };
    let problems = qa::check(&out, &late).unwrap().problems;
    assert_eq!(
        problems[3..],
        [
            "caption 2 starts before caption 1 ends",
            "caption 2 ends at 3.00 s, after the picture does at 2.00 s"
        ],
        "{problems:#?}"
    );
}

#[test]
fn chapters_are_written_into_the_file_and_checked_against_its_length() {
    let dir = tempfile::tempdir().unwrap();
    // 31 seconds of a tone and of the pattern.
    let audio = dir.path().join("long.wav");
    let made = Command::new("ffmpeg")
        .args(["-nostdin", "-loglevel", "error", "-y", "-f", "lavfi", "-i"])
        .arg("sine=frequency=440:duration=31:sample_rate=48000")
        .arg(&audio)
        .status()
        .unwrap();
    assert!(made.success());
    let normalized = dir.path().join("norm.wav");
    loudnorm(&audio, &normalized, Target::default()).unwrap();
    let chapter = |start, title: &str| Chapter {
        start,
        title: title.into(),
    };
    // A title with the marks the metadata format keeps for itself.
    let chapters = vec![
        chapter(0, "Intro"),
        chapter(10, "What = which; #2 \\ more"),
        chapter(20, "The end"),
    ];
    let metadata = dir.path().join("chapters.ffmeta");
    fs::write(&metadata, chapter_metadata(&chapters, 31.0)).unwrap();
    let encode = |out: &Path| {
        let settings = Settings::new(W, H);
        let mut encoder =
            Encoder::start_with(&settings, &normalized, Some(&metadata), out).unwrap();
        for n in 0..31 * 30 {
            encoder.push(&frame(n % 7)).unwrap();
        }
        encoder.finish().unwrap();
    };
    let (a, b) = (dir.path().join("a.mp4"), dir.path().join("b.mp4"));
    encode(&a);
    encode(&b);
    // The chapters don't cost the file its repeatability.
    assert_eq!(fs::read(&a).unwrap(), fs::read(&b).unwrap());
    no_dates_or_versions(&a);
    let expect = qa::Expect {
        width: W,
        height: H,
        fps: 30,
        duration: 31.0,
        loudness: -16.0,
        tolerance: 1.5,
        true_peak: -1.0,
        captions: None,
        chapters: chapters.clone(),
    };
    let report = qa::check(&a, &expect).unwrap();
    assert_eq!(report.problems, Vec::<String>::new(), "{}", report.summary);
    // Another list than the file's is a problem, and so is a file with
    // chapters when the cut has none.
    let mut other = chapters.clone();
    other[1].title = "Something else".into();
    let wrong = qa::Expect {
        chapters: other,
        ..expect.clone()
    };
    assert_eq!(
        qa::check(&a, &wrong).unwrap().problems,
        ["its chapter 2 is \"What = which; #2 \\\\ more\" at 10 s, not \"Something else\" at 10 s"]
    );
    let none = qa::Expect {
        chapters: Vec::new(),
        ..expect
    };
    assert_eq!(
        qa::check(&a, &none).unwrap().problems,
        ["it has 3 chapters, not the 0 the cut defines"]
    );
}

#[test]
fn a_frame_of_the_wrong_size_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let mut encoder = Encoder::start(
        &Settings::new(W, H),
        &tone(dir.path()),
        &dir.path().join("x.mp4"),
    )
    .unwrap();
    assert!(
        encoder
            .push(&[0u8; 12])
            .unwrap_err()
            .contains("a frame of 12 bytes")
    );
}

/// A narration whose sibilant ffmpeg's AAC encoder spikes on, when it is
/// left to substitute noise: the first 18.5 s of a draft in the voice
/// `piper:en_US-kristin-medium`, as `voice::narrate` wrote it (22.05 kHz,
/// mono, its peaks held at -2.5 dBFS). The spike depends on everything
/// the encoder has heard before it, so the fixture is the narration from
/// its start.
fn sibilant() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/encode/sibilant.flac")
}

#[test]
fn the_encoder_adds_no_peak_to_a_narration() {
    let dir = tempfile::tempdir().unwrap();
    // The narration as a cut encodes it: 48 kHz, the same in both channels.
    let narration = dir.path().join("narration.wav");
    vidni_media::encode::stereo(&sibilant(), &narration).unwrap();
    let before = qa::measure(&narration).unwrap().true_peak.unwrap();
    assert!(
        (-2.5..=-1.5).contains(&before),
        "it goes in at {before} dBTP"
    );

    // P10-VIDNI-01: encoded as a cut is, it comes out as high as it went in.
    let out = dir.path().join("cut.mp4");
    let mut encoder = Encoder::start(&Settings::new(W, H), &narration, &out).unwrap();
    let still = frame(0);
    for _ in 0..30 * 19 {
        encoder.push(&still).unwrap();
    }
    encoder.finish().unwrap();
    let after = qa::measure(&out).unwrap().true_peak.unwrap();
    assert!(
        (after - before).abs() <= 0.5,
        "{before} dBTP went in and {after} came out"
    );

    // And it is this narration that the setting is for: with ffmpeg's
    // encoder as it comes, the same sound comes out over full scale. If
    // this stops holding, ffmpeg's encoder has changed, and the fixture
    // tells nothing any more.
    let plain = dir.path().join("plain.m4a");
    let made = Command::new("ffmpeg")
        .args(["-nostdin", "-loglevel", "error", "-y", "-i"])
        .arg(&narration)
        .args(["-c:a", "aac", "-b:a", "160k"])
        .arg(&plain)
        .status()
        .unwrap();
    assert!(made.success());
    let spiked = qa::measure(&plain).unwrap().true_peak.unwrap();
    assert!(
        spiked > before + 2.0,
        "{before} dBTP went in and {spiked} came out"
    );
}

#[test]
fn qa_names_digital_silence_and_a_peak_that_is_too_high() {
    let dir = tempfile::tempdir().unwrap();
    // Two seconds: a tone driven into clipping, with a quarter of a second of
    // digital silence cut into its middle.
    let audio = dir.path().join("gap.wav");
    let made = Command::new("ffmpeg")
        .args(["-nostdin", "-loglevel", "error", "-y", "-f", "lavfi", "-i"])
        .arg("sine=frequency=440:duration=2:sample_rate=48000")
        .args([
            "-af",
            "volume=20,volume=enable='between(t,1,1.25)':volume=0",
            "-ac",
            "2",
        ])
        .arg(&audio)
        .status()
        .unwrap();
    assert!(made.success());
    let out = dir.path().join("gap.mp4");
    encode(&audio, &out);
    let sound = qa::measure(&out).unwrap();
    assert_eq!(sound.silences.len(), 1, "{sound:?}");
    let (at, lasts) = sound.silences[0];
    assert!(
        (at - 1.0).abs() < 0.03 && (lasts - 0.25).abs() < 0.03,
        "{sound:?}"
    );
    assert!(sound.true_peak.unwrap() > -1.0, "{sound:?}");
    let expect = qa::Expect {
        width: W,
        height: H,
        fps: 30,
        duration: 2.0,
        loudness: sound.loudness.unwrap(),
        tolerance: 1.0,
        true_peak: -1.0,
        captions: None,
        chapters: Vec::new(),
    };
    let problems = qa::check(&out, &expect).unwrap().problems;
    assert_eq!(problems.len(), 2, "{problems:#?}");
    assert!(problems[0].starts_with("its true peak is ") && problems[0].ends_with("above -1 dBTP"));
    assert!(
        problems[1]
            .starts_with("it has a stretch of digital silence longer than 50 ms; the first is 0.2"),
        "{problems:#?}"
    );
    // The tone alone, normalized, has neither.
    let normalized = dir.path().join("norm.wav");
    loudnorm(&tone(dir.path()), &normalized, Target::default()).unwrap();
    let good = dir.path().join("good.mp4");
    encode(&normalized, &good);
    let sound = qa::measure(&good).unwrap();
    assert_eq!(sound.silences, []);
    assert!(sound.true_peak.unwrap() <= -1.0, "{sound:?}");
}
