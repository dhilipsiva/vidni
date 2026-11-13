//! The encoder gives the same packets for the same frames and samples (T2),
//! loudnorm reaches its target, and QA names each way a cut is wrong.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use vidni_media::encode::{Encoder, Settings, Target, loudnorm};
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
    let probe = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format_tags:stream_tags",
            "-of",
            "json",
        ])
        .arg(&out)
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
        captions: Some((srt.clone(), 2)),
        chapters: None,
    };
    let report = qa::check(&out, &expect).unwrap();
    assert_eq!(report.problems, Vec::<String>::new(), "{}", report.summary);
    // Each wrong expectation is named on its own.
    let wrong = qa::Expect {
        width: 1920,
        duration: 5.0,
        loudness: -23.0,
        captions: Some((srt, 3)),
        chapters: Some(vec![(0.0, "Intro".into())]),
        ..expect
    };
    let problems = qa::check(&out, &wrong).unwrap().problems;
    assert_eq!(problems.len(), 5, "{problems:#?}");
    assert!(problems[0].starts_with("the picture is 320x180, not 1920x180"));
    assert!(problems[1].starts_with("it lasts 2.0"));
    assert!(problems[2].contains("more than 1.5 LU from -23 LUFS"));
    assert!(problems[3].ends_with("has 2 cues, and the timeline 3"));
    assert_eq!(problems[4], "it has 0 chapters, not the 1 the cut defines");
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
