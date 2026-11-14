//! The microphone's path through cpal, on ALSA's `null` device: an input
//! that every ALSA has and that gives zeros, so the test hears nobody. It
//! is ignored by `cargo test`; `just test-record` runs it.

use std::time::{Duration, Instant};
use vidni_record::clean::{self, Refused};
use vidni_record::input::Input;
use vidni_record::mic::Mic;
use vidni_record::take;

/// What ALSA calls its `null` device.
const NULL: &str = "generate zero samples";

#[test]
#[ignore = "needs ALSA: just test-record"]
fn the_microphones_path_records_what_the_device_gives() {
    assert!(
        Mic::all().unwrap().iter().any(|name| name.contains(NULL)),
        "{:?}",
        Mic::all()
    );
    let mic = Mic::find(Some(NULL)).unwrap();
    assert!(mic.name().contains(NULL), "{}", mic.name());
    assert!(mic.rate() > 0);
    // A second of what it gives: silence, through the same recorder.
    let dir = tempfile::tempdir().unwrap();
    let raw = dir.path().join("takes/main-1/raw/abcd-1.wav");
    let recording = take::record(&mic, &raw).unwrap();
    let deadline = Instant::now() + Duration::from_secs(20);
    while recording.level().seconds < 1.0 {
        assert_eq!(recording.failed(), None);
        assert!(Instant::now() < deadline, "{:?}", recording.level());
        std::thread::sleep(Duration::from_millis(5));
    }
    let level = recording.level();
    assert_eq!((level.now, level.peak), (clean::FLOOR, clean::FLOOR));
    let raw = recording.stop().unwrap();
    let (samples, rate) = take::read(&raw).unwrap();
    assert_eq!(rate, mic.rate());
    assert!(samples.len() >= rate as usize && samples.iter().all(|s| *s == 0));
    // And it is refused as any silent take is.
    let kept = dir.path().join("takes/main-1/abcd.wav");
    assert_eq!(
        take::candidate(&raw, &kept).unwrap(),
        Err(Refused::Silent(clean::FLOOR))
    );
    // An input that isn't there is named.
    let error = Mic::find(Some("no such microphone")).err().unwrap();
    assert_eq!(error, "no input's name holds \"no such microphone\"");
}
