//! The recorder, driven by a made-up input (TODO.md, P4-VIDNI-01): a
//! normal take, a silent take, a short take and room tone, each recorded
//! through the same path a microphone's samples take.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use vidni_record::clean::{self, Refused};
use vidni_record::input::{Input, Synthetic};
use vidni_record::room;
use vidni_record::take::{self, Recording};

const RATE: u32 = 48000;

fn at(seconds: f64) -> usize {
    (seconds * f64::from(RATE)).round() as usize
}

/// A take as a person makes one: the start key's click, a breath of room
/// noise, `speech` seconds of a voice-like tone, room noise again, and the
/// stop key's click. The room is a hum far below the voice.
fn spoken(speech: f64) -> Vec<i16> {
    let room = |n: usize| ((n as f64 / 480.0 * std::f64::consts::TAU).sin() * 20.0) as i16;
    let voice = |n: usize| ((n as f64 / 218.0 * std::f64::consts::TAU).sin() * 9000.0) as i16;
    let mut samples: Vec<i16> = vec![30000, -30000, 20000, -20000, 9000, -9000];
    samples.extend((0..at(0.6)).map(room));
    samples.extend((0..at(speech)).map(voice));
    samples.extend((0..at(0.7)).map(room));
    samples.extend([30000, -30000, 20000, -20000, 9000, -9000]);
    samples
}

/// Records `seconds` of `input` into `raw`, and stops.
fn recorded(input: &Synthetic, raw: &Path, seconds: f64) -> PathBuf {
    let recording = take::record(&input.clone().lasting(seconds), raw).unwrap();
    wait_for(&recording, seconds);
    recording.stop().unwrap()
}

/// Waits until the recording holds `seconds` of sound.
fn wait_for(recording: &Recording, seconds: f64) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while recording.level().seconds < seconds - 1e-9 {
        assert!(Instant::now() < deadline, "{:?}", recording.level());
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn sha(path: &Path) -> String {
    vidni_core::capture::sha256_hex(&fs::read(path).unwrap())
}

#[test]
fn a_normal_take_is_trimmed_and_kept_and_its_raw_take_is_left_as_it_was() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    let (raw, kept) = (
        dir.join("takes/main-1/raw/abcd-1.wav"),
        dir.join("takes/main-1/abcd.wav"),
    );
    let samples = spoken(2.0);
    let seconds = samples.len() as f64 / f64::from(RATE);
    let input = Synthetic::samples(RATE, samples.clone());
    let raw = recorded(&input, &raw, seconds);
    // The raw take is what the input gave, sample for sample.
    assert_eq!(take::read(&raw).unwrap(), (samples.clone(), RATE));
    let before = sha(&raw);

    // Trimmed: the words, with their margins, and neither key's click.
    let candidate = take::candidate(&raw, &kept).unwrap().unwrap();
    assert!(
        (candidate.seconds - (2.0 + clean::LEAD + clean::TAIL)).abs() < 0.02,
        "{candidate:?}"
    );
    assert!(candidate.peak < -10.0, "{candidate:?}");
    assert!((candidate.from as f64 / f64::from(RATE) - (0.6 - clean::LEAD)).abs() < 0.01);
    // It waits beside the kept take, under a name no take has.
    assert_eq!(candidate.path, dir.join("takes/main-1/abcd.wav.new"));
    assert!(candidate.path.is_file() && !kept.exists());
    // Kept, it is the line's take; the candidate is gone.
    let path = candidate.clone().keep().unwrap();
    assert_eq!(path, kept);
    assert!(!candidate.path.exists());
    let (trimmed, rate) = take::read(&kept).unwrap();
    assert_eq!((trimmed.len(), rate), (candidate.to - candidate.from, RATE));
    // The raw take is as it was recorded.
    assert_eq!(sha(&raw), before);

    // A second take of the line: discarded, the kept take stays; kept, it
    // takes the first one's place. No raw take is ever written over.
    let again = recorded(
        &Synthetic::samples(RATE, spoken(1.5)),
        &dir.join("takes/main-1/raw/abcd-2.wav"),
        3.0,
    );
    let first = sha(&kept);
    take::candidate(&again, &kept)
        .unwrap()
        .unwrap()
        .discard()
        .unwrap();
    assert_eq!(sha(&kept), first);
    take::candidate(&again, &kept)
        .unwrap()
        .unwrap()
        .keep()
        .unwrap();
    assert_ne!(sha(&kept), first);
    assert_eq!(sha(&raw), before);
    let error = take::record(&input, &raw).err().unwrap();
    assert!(
        error.contains("abcd-1.wav") && error.contains("exists"),
        "{error}"
    );
    assert_eq!(sha(&raw), before);
}

#[test]
fn a_silent_take_is_refused_and_nothing_is_kept() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    let (raw, kept) = (dir.join("raw/abcd-1.wav"), dir.join("abcd.wav"));
    // Three seconds of a room with nobody speaking.
    let hum = Synthetic::tone(RATE, 100.0, 0.002);
    let raw = recorded(&hum, &raw, 3.0);
    let refused = take::candidate(&raw, &kept).unwrap().unwrap_err();
    let Refused::Silent(peak) = refused else {
        panic!("{refused:?}");
    };
    assert!((peak - -54.0).abs() < 0.5, "{peak}");
    assert_eq!(
        refused.to_string(),
        "there was no speech in that take (its loudest point was -54 dB), so it was discarded; \
         if you did speak, check the mic with `vidni record --test`"
    );
    // No candidate and no kept take; the raw take stays, as every one does.
    assert!(!kept.exists() && !take::candidate_path(&kept).exists());
    assert!(raw.is_file());
    // Digital silence is refused the same way.
    let raw = recorded(&Synthetic::silence(RATE), &dir.join("raw/abcd-2.wav"), 2.0);
    assert_eq!(
        take::candidate(&raw, &kept).unwrap(),
        Err(Refused::Silent(clean::FLOOR))
    );
}

#[test]
fn a_short_take_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    let (raw, kept) = (dir.join("raw/abcd-1.wav"), dir.join("abcd.wav"));
    // A double press: half a second, loud as it may be.
    let raw = recorded(&Synthetic::tone(RATE, 220.0, 0.3), &raw, 0.5);
    let refused = take::candidate(&raw, &kept).unwrap().unwrap_err();
    assert_eq!(refused, Refused::TooShort(0.5));
    assert_eq!(
        refused.to_string(),
        "that take was only 0.5 s, so it was discarded"
    );
    assert!(!kept.exists() && !take::candidate_path(&kept).exists());
    // A take that is cancelled leaves no file at all.
    let cancelled = dir.join("raw/abcd-2.wav");
    let recording =
        take::record(&Synthetic::tone(RATE, 220.0, 0.3).lasting(1.0), &cancelled).unwrap();
    wait_for(&recording, 1.0);
    recording.cancel().unwrap();
    assert!(!cancelled.exists());
}

#[test]
fn room_tone_is_recorded_and_measured() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    // A quiet room: a hum at -60 dBFS, with the start key's click in it.
    let mut quiet: Vec<i16> = (0..at(room::SECONDS))
        .map(|n| ((n as f64 / 480.0 * std::f64::consts::TAU).sin() * 46.0) as i16)
        .collect();
    quiet[..6].copy_from_slice(&[30000, -30000, 20000, -20000, 9000, -9000]);
    let raw = dir.join("takes/_room/tone.wav.raw");
    let raw = recorded(&Synthetic::samples(RATE, quiet), &raw, room::SECONDS);
    let tone = room::keep(&raw, dir).unwrap();
    assert_eq!(tone.path, dir.join("takes/_room/tone.wav"));
    // Its level is the hum's, without the click: a sine's RMS is 3 dB under its peak.
    assert!((tone.level - -60.1).abs() < 0.3, "{}", tone.level);
    assert!(!tone.noisy());
    // What is kept is the middle of the recording, and the recording is gone.
    let (kept, rate) = take::read(&tone.path).unwrap();
    assert_eq!(
        (kept.len(), rate),
        (at(room::SECONDS - room::SETTLE - room::END), RATE)
    );
    assert!(clean::peak_db(&kept) < -55.0);
    assert!(!raw.exists());

    // A noisy room says so, and takes the quiet one's place.
    let noisy = Synthetic::tone(RATE, 120.0, 0.01);
    let raw = recorded(&noisy, &dir.join("takes/_room/tone.wav.raw"), room::SECONDS);
    let tone = room::keep(&raw, dir).unwrap();
    assert!((tone.level - -43.0).abs() < 0.3, "{}", tone.level);
    assert!(tone.noisy());
    // A recording too short to measure is refused, and the tone stays.
    let before = sha(&tone.path);
    let raw = recorded(&noisy, &dir.join("takes/_room/tone.wav.raw"), 0.8);
    let error = room::keep(&raw, dir).unwrap_err();
    assert_eq!(
        error,
        "the room was recorded for 0.8 s, which is too short to measure"
    );
    assert_eq!(sha(&tone.path), before);
}

#[test]
fn the_meter_reads_the_level_as_the_take_is_recorded() {
    let dir = tempfile::tempdir().unwrap();
    // A second of a loud tone, then a second of a soft one.
    let input = Synthetic::new(RATE, |n| {
        let level = if n < u64::from(RATE) { 0.5 } else { 0.05 };
        ((n as f64 / 218.0 * std::f64::consts::TAU).sin() * level * 32767.0) as i16
    });
    let raw = dir.path().join("raw.wav");
    let recording = take::record(&input.clone().lasting(1.0), &raw).unwrap();
    wait_for(&recording, 1.0);
    let loud = recording.level();
    assert_eq!(loud.seconds, 1.0);
    assert!(
        (loud.now - -9.0).abs() < 0.2 && (loud.peak - -6.0).abs() < 0.1,
        "{loud:?}"
    );
    recording.cancel().unwrap();
    let recording = take::record(&input.lasting(2.0), &raw).unwrap();
    wait_for(&recording, 2.0);
    let soft = recording.level();
    // The level is the newest sound's; the peak is the whole take's.
    assert!(
        (soft.now - -29.0).abs() < 0.2 && (soft.peak - -6.0).abs() < 0.1,
        "{soft:?}"
    );
    assert_eq!(recording.failed(), None);
    recording.stop().unwrap();
    // Before anything arrives, the meter reads the floor.
    let recording = take::record(
        &Synthetic::silence(RATE).lasting(0.0),
        &dir.path().join("empty.wav"),
    )
    .unwrap();
    assert_eq!(recording.level().now, clean::FLOOR);
    recording.cancel().unwrap();
}

#[test]
fn an_input_that_stops_on_its_own_is_reported() {
    let dir = tempfile::tempdir().unwrap();
    let unplugged = Synthetic::tone(RATE, 220.0, 0.3)
        .lasting(1.0)
        .failing("the device is gone");
    let raw = dir.path().join("raw.wav");
    let recording = take::record(&unplugged, &raw).unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while recording.failed().is_none() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(
        recording.failed().as_deref(),
        Some("the input stopped on its own: the device is gone")
    );
    // What was recorded until then is still a raw take, and stopping says why it ended.
    let error = recording.stop().unwrap_err();
    assert!(
        error.ends_with("raw.wav: the input stopped on its own: the device is gone"),
        "{error}"
    );
    assert_eq!(take::read(&raw).unwrap().0.len(), at(1.0));
    // A paced input takes as long as a real one.
    let started = Instant::now();
    let paced = Synthetic::silence(RATE).lasting(0.3).paced();
    let recording = take::record(&paced, &dir.path().join("paced.wav")).unwrap();
    wait_for(&recording, 0.3);
    assert!(
        started.elapsed() >= Duration::from_millis(280),
        "{:?}",
        started.elapsed()
    );
    recording.cancel().unwrap();
    assert_eq!(paced.name(), "a made-up input");
}

#[test]
fn the_waveform_of_a_recording_is_the_waveform_of_its_take() {
    let dir = tempfile::tempdir().unwrap();
    let raw = dir.path().join("raw-1.wav");
    let input = Synthetic::samples(RATE, spoken(1.0)).lasting(2.5);
    let recording = take::record(&input, &raw).unwrap();
    wait_for(&recording, 2.5);
    // While it is recorded: a bar for each 20 ms so far.
    let live = recording.envelope();
    assert_eq!(live.len(), 125);
    let raw = recording.stop().unwrap();
    // The stored take's waveform is the same bars.
    let (samples, rate) = take::read(&raw).unwrap();
    assert_eq!(take::envelope(&samples, rate), live);
    // The clicks are the loudest bars, the voice is at its level, and the
    // room is far below it.
    assert!(live[0] > 0.9, "{}", live[0]);
    let voice = live[at(1.0) / at(take::BAR)];
    assert!((voice - 9000.0 / 32768.0).abs() < 0.01, "{voice}");
    assert!(live[10] < 0.001, "{}", live[10]);
    // A take that ends inside a bar has that part as its last bar.
    assert_eq!(take::envelope(&samples[..at(0.05)], rate).len(), 3);
    assert!(take::envelope(&[], rate).is_empty());
}
