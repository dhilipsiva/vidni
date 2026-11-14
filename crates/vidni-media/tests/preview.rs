//! The preview's decoder gives the frame that is asked for (TODO.md,
//! P7-VIDNI-03): each frame of a test cut carries its own number, and
//! every frame decoded is read back as the one requested.

use std::path::Path;
use std::process::Command;
use vidni_media::encode::{Encoder, Settings};
use vidni_media::preview::Video;

const W: u32 = 320;
const H: u32 = 192;
const FRAMES: u64 = 400;

/// A frame that carries `n`: ten bars across, the bar for bit `b` white
/// when that bit of `n` is set, on a mid grey. Bars survive compression.
fn numbered(n: u64) -> Vec<u8> {
    let mut out = Vec::with_capacity((W * H * 3) as usize);
    for _ in 0..H {
        for x in 0..W {
            let bit = (x / 32) as u64;
            let level = if n >> bit & 1 == 1 { 235 } else { 40 };
            out.extend([level, level, level]);
        }
    }
    out
}

/// The number a decoded frame carries, read at the middle of each bar.
fn number_of(image: &vidni_media::compose::image::Image) -> u64 {
    let bar = image.width / 10;
    (0..10u64)
        .filter(|bit| image.pixel(*bit as u32 * bar + bar / 2, image.height / 2)[0] > 128)
        .map(|bit| 1 << bit)
        .sum()
}

fn cut(dir: &Path) -> Video {
    let audio = dir.join("tone.wav");
    let made = Command::new("ffmpeg")
        .args(["-nostdin", "-loglevel", "error", "-y", "-f", "lavfi", "-i"])
        .arg(format!(
            "sine=frequency=440:duration={}:sample_rate=48000",
            FRAMES / 30 + 1
        ))
        .args(["-ac", "2"])
        .arg(&audio)
        .status()
        .unwrap();
    assert!(made.success());
    let mp4 = dir.join("cut.mp4");
    let mut encoder = Encoder::start(&Settings::new(W, H), &audio, &mp4).unwrap();
    for n in 0..FRAMES {
        encoder.push(&numbered(n)).unwrap();
    }
    encoder.finish().unwrap();
    Video::open(&mp4).unwrap()
}

#[test]
fn a_frame_asked_for_by_number_is_that_frame() {
    let dir = tempfile::tempdir().unwrap();
    let video = cut(dir.path());
    assert_eq!((video.width, video.height, video.frames), (W, H, FRAMES));
    assert_eq!(video.fps, 30.0);
    assert!((video.seconds() - FRAMES as f64 / 30.0).abs() < 1e-9);
    // The first, the last, frames either side of a keyframe (x264 starts
    // one every 250 frames at most), and a scatter of others.
    for n in [
        0, 1, 2, 29, 30, 31, 123, 248, 249, 250, 251, 252, 333, 398, 399,
    ] {
        let frame = video.frame(n, W).unwrap();
        assert_eq!((frame.width, frame.height), (W, H));
        assert_eq!(number_of(&frame), n, "frame {n}");
    }
    // Asked for smaller, it is the same frame; past the end is the last one.
    let small = video.frame(250, 160).unwrap();
    assert_eq!((small.width, small.height), (160, 96));
    assert_eq!(number_of(&small), 250);
    assert_eq!(number_of(&video.frame(10_000, 160).unwrap()), FRAMES - 1);
    // A moment in the cut is the frame that has begun by then.
    assert_eq!(video.frame_at(0.0), 0);
    assert_eq!(video.frame_at(1.0), 30);
    assert_eq!(video.frame_at(1.0 + 1.0 / 60.0), 30);
    assert_eq!(video.frame_at(video.time_of(251)), 251);
    assert_eq!(video.frame_at(1e9), FRAMES - 1);
    assert_eq!(video.size(10_000), (W, H));
    assert_eq!(video.size(101), (100, 60));
}

#[test]
fn playing_gives_every_frame_in_order_from_where_it_starts() {
    let dir = tempfile::tempdir().unwrap();
    let video = cut(dir.path());
    let played: Vec<u64> = video
        .play(245, 160)
        .unwrap()
        .take(12)
        .map(|(index, frame)| {
            assert_eq!(number_of(&frame), index);
            index
        })
        .collect();
    assert_eq!(played, (245..257).collect::<Vec<u64>>());
    // It ends with the cut.
    assert_eq!(video.play(FRAMES - 3, 160).unwrap().count(), 3);
    assert!(Video::open(&dir.path().join("none.mp4")).is_err());
}
