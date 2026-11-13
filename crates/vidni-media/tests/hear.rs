//! The listening check with the real whisper-cli and its model
//! (P3-VIDNI-05). These need both, so they are ignored by `cargo test`;
//! `just test-hear` runs them, and so does CI.

use std::path::{Path, PathBuf};
use vidni_media::hear::{Heard, Listener, closeness};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/hear")
        .join(name)
}

/// A WAV of `seconds` of digital silence at 22.05 kHz.
fn silence(dir: &Path, name: &str, seconds: f64) -> PathBuf {
    let path = dir.join(name);
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 22050,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(&path, spec).unwrap();
    for _ in 0..(seconds * 22050.0) as u32 {
        writer.write_sample(0i16).unwrap();
    }
    writer.finalize().unwrap();
    path
}

#[test]
#[ignore = "needs whisper-cli and its model: just test-hear"]
fn a_spoken_clip_is_heard_close_to_its_script() {
    let listener = Listener::find().unwrap();
    let Heard::Words(words) = listener.hear(&fixture("spoken.wav")) else {
        panic!("the spoken clip must be heard");
    };
    let score = closeness("Language models are good at reading prose.", &words, &[]);
    assert!(score >= 0.9, "heard {words:?}, which scores {score}");
    // What it hears has nothing to do with another line.
    let other = closeness("Adam is a dog, so Adam is an animal.", &words, &[]);
    assert!(other < 0.5, "{other}");
}

#[test]
#[ignore = "needs whisper-cli and its model: just test-hear"]
fn empty_and_silent_clips_are_heard_as_nothing() {
    let listener = Listener::find().unwrap();
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(
        listener.hear(&silence(dir.path(), "empty.wav", 0.0)),
        Heard::Nothing
    );
    assert_eq!(
        listener.hear(&silence(dir.path(), "silent.wav", 3.0)),
        Heard::Nothing
    );
}
