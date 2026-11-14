//! What happens to a raw take before it is kept (TODO.md, P4-VIDNI-01): the
//! guard against a take with nothing in it, and the trim of the keypresses
//! and of the silence around the words.
//!
//! The raw take is never changed. What is kept is made from it, as the
//! reference's recorder made it with ffmpeg's `atrim`, `highpass` and
//! `silenceremove` filters (`CLEAN` in reference/record.py). The filter and
//! the level are worked out in ffmpeg's own arithmetic, so that the same
//! raw take gives the same samples and is cut at the same two places.
//! Where ffmpeg splices the silence it keeps, it writes one sample from
//! long before and repeats another; this trim keeps the stretch whole
//! (DECISIONS.md, T10).

use std::fmt;

/// A raw take shorter than this is discarded, in seconds.
pub const MIN_TAKE: f64 = 0.8;
/// A take that never gets louder than this has no speech in it, in dBFS.
pub const SILENT_PEAK: f64 = -40.0;
/// How much is dropped from the start of a take, where the key that
/// started it is heard, and from its end, where the key that stopped it
/// is. Seconds.
pub const KEY_DOWN: f64 = 0.20;
pub const KEY_UP: f64 = 0.10;
/// Rumble below this is filtered out of what is kept, in Hz, by a filter
/// of this sharpness (ffmpeg's default for `highpass`).
pub const HIGH_PASS: f64 = 70.0;
const Q: f64 = 0.707;
/// The words begin where the level first rises above this, and end where
/// it last does: dBFS, as the RMS over the [`WINDOW`] seconds up to there.
pub const SILENCE: f64 = -45.0;
pub const WINDOW: f64 = 0.02;
/// How much of the silence before the words is kept, and after them, in
/// seconds.
pub const LEAD: f64 = 0.15;
pub const TAIL: f64 = 0.25;
/// Less than this left after the trim is nothing, in seconds.
pub const MIN_KEPT: f64 = 0.4;

/// The level of digital silence, for a meter: no sound reads lower.
pub const FLOOR: f64 = -96.0;

/// Why a take was discarded.
#[derive(Debug, Clone, PartialEq)]
pub enum Refused {
    /// It was shorter than [`MIN_TAKE`]: this long, in seconds.
    TooShort(f64),
    /// It never got louder than [`SILENT_PEAK`]: its peak, in dBFS.
    Silent(f64),
    /// The trim left less than [`MIN_KEPT`].
    NothingLeft,
}

impl fmt::Display for Refused {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Refused::TooShort(seconds) => {
                write!(f, "that take was only {seconds:.1} s, so it was discarded")
            }
            Refused::Silent(peak) => write!(
                f,
                "there was no speech in that take (its loudest point was {peak:.0} dB), so it was discarded; if you did speak, check the mic with `vidni record --test`"
            ),
            Refused::NothingLeft => write!(
                f,
                "nothing was left after the silence was trimmed, so the take was discarded"
            ),
        }
    }
}

/// What is kept of a raw take.
#[derive(Debug, Clone, PartialEq)]
pub struct Kept {
    /// The samples to keep: the raw take's from `from` up to `to`, with
    /// the rumble filtered out.
    pub samples: Vec<i16>,
    /// Where in the raw take they begin and end, in samples.
    pub from: usize,
    pub to: usize,
    /// Where in the raw take the words begin and end: the first sample,
    /// and the last, at which the level is above [`SILENCE`]. Around them
    /// are the margins of silence that are kept.
    pub words: (usize, usize),
}

/// A level in dBFS: `amplitude` of 1 is full scale. Never below [`FLOOR`].
pub fn db(amplitude: f64) -> f64 {
    match amplitude > 0.0 {
        true => (20.0 * amplitude.log10()).max(FLOOR),
        false => FLOOR,
    }
}

/// The loudest sample, in dBFS.
pub fn peak_db(samples: &[i16]) -> f64 {
    let peak = samples
        .iter()
        .map(|s| i32::from(*s).unsigned_abs())
        .max()
        .unwrap_or(0);
    db(f64::from(peak) / 32768.0)
}

/// The level of `samples` as a whole, in dBFS: their root mean square.
pub fn rms_db(samples: &[i16]) -> f64 {
    if samples.is_empty() {
        return FLOOR;
    }
    let squares: f64 = samples.iter().map(|s| f64::from(*s).powi(2)).sum();
    db((squares / samples.len() as f64).sqrt() / 32768.0)
}

/// How many samples `seconds` are at `rate`.
fn samples_in(seconds: f64, rate: u32) -> usize {
    (seconds * f64::from(rate)).round() as usize
}

/// `samples` with what is below `cutoff` Hz filtered out: a second-order
/// high-pass, as ffmpeg's `highpass` filter is by default, and in its
/// arithmetic for 16-bit samples: single precision, its order of
/// operations, and a result cut off towards zero.
pub fn high_pass(samples: &[i16], rate: u32, cutoff: f64) -> Vec<i16> {
    // The biquad of the Audio EQ Cookbook, worked out in double precision.
    let w = std::f64::consts::TAU * cutoff / f64::from(rate);
    let alpha = w.sin() / (2.0 * Q);
    let a0 = 1.0 + alpha;
    let (b0, b2) = (
        ((1.0 + w.cos()) / 2.0 / a0) as f32,
        ((1.0 + w.cos()) / 2.0 / a0) as f32,
    );
    let b1 = (-(1.0 + w.cos()) / a0) as f32;
    // The feedback terms are added, so they are kept with their signs turned.
    let (a1, a2) = (
        -((-2.0 * w.cos() / a0) as f32),
        -(((1.0 - alpha) / a0) as f32),
    );
    let (mut i1, mut i2, mut o1, mut o2) = (0f32, 0f32, 0f32, 0f32);
    samples
        .iter()
        .map(|s| {
            let x = f32::from(*s);
            let o = i2 * b2 + i1 * b1 + x * b0 + o2 * a2 + o1 * a1;
            (i2, i1, o2, o1) = (i1, x, o1, o);
            // A float becomes an integer by losing its fraction, and stays
            // within 16 bits.
            o as i16
        })
        .collect()
}

/// The first of `samples` at which the sound has begun: where the level
/// over the [`WINDOW`] up to it first rises above [`SILENCE`]. None when
/// it never does.
///
/// The level is ffmpeg's `silenceremove` level: the RMS of the samples in
/// the window so far, from a running sum of squares in single precision.
pub fn begins<'s>(samples: impl Iterator<Item = &'s i16>, rate: u32) -> Option<usize> {
    let size = samples_in(WINDOW, rate).max(1);
    let threshold = 10f64.powf(SILENCE / 20.0) as f32;
    // The squares in the window, oldest first, and their sum.
    let mut window = std::collections::VecDeque::with_capacity(size);
    let mut sum = 0f32;
    for (i, sample) in samples.enumerate() {
        let x = f32::from(*sample) / 32768.0;
        let leaving = match window.len() == size {
            true => window.pop_front().unwrap_or(0.0),
            false => 0.0,
        };
        window.push_back(x);
        sum += x * x;
        sum -= leaving * leaving;
        sum = sum.max(0.0);
        if (sum / window.len() as f32).sqrt() > threshold {
            return Some(i);
        }
    }
    None
}

/// Checks a raw take and trims it: drops the keypresses at its ends,
/// filters out the rumble, and trims the silence around the words, leaving
/// [`LEAD`] before them and [`TAIL`] after. A take that is too short, or
/// has no speech in it, is refused.
pub fn clean(raw: &[i16], rate: u32) -> Result<Kept, Refused> {
    let seconds = raw.len() as f64 / f64::from(rate.max(1));
    if seconds < MIN_TAKE {
        return Err(Refused::TooShort(seconds));
    }
    let peak = peak_db(raw);
    if peak < SILENT_PEAK {
        return Err(Refused::Silent(peak));
    }
    let (down, up) = (samples_in(KEY_DOWN, rate), samples_in(KEY_UP, rate));
    let spoken = raw
        .get(down..raw.len().saturating_sub(up))
        .unwrap_or_default();
    let filtered = high_pass(spoken, rate, HIGH_PASS);
    // Where the words begin, read from the start, and where they end,
    // read from the end.
    let (Some(first), Some(last)) = (
        begins(filtered.iter(), rate),
        begins(filtered.iter().rev(), rate),
    ) else {
        return Err(Refused::NothingLeft);
    };
    let from = first.saturating_sub(samples_in(LEAD, rate));
    let to = filtered
        .len()
        .saturating_sub(last.saturating_sub(samples_in(TAIL, rate)));
    if to.saturating_sub(from) < samples_in(MIN_KEPT, rate) {
        return Err(Refused::NothingLeft);
    }
    Ok(Kept {
        samples: filtered[from..to].to_vec(),
        from: down + from,
        to: down + to,
        words: (down + first, down + filtered.len() - 1 - last),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const RATE: u32 = 48000;

    /// `seconds` of a tone at `hz`, at `level` of full scale.
    fn tone(seconds: f64, hz: f64, level: f64) -> Vec<i16> {
        (0..samples_in(seconds, RATE))
            .map(|n| {
                let t = n as f64 / f64::from(RATE);
                ((t * hz * std::f64::consts::TAU).sin() * level * 32767.0) as i16
            })
            .collect()
    }

    fn silence(seconds: f64) -> Vec<i16> {
        vec![0; samples_in(seconds, RATE)]
    }

    /// A click, as a key makes: a few loud samples.
    fn click() -> Vec<i16> {
        vec![30000, -30000, 20000, -20000, 9000, -9000]
    }

    fn seconds(samples: usize) -> f64 {
        samples as f64 / f64::from(RATE)
    }

    #[test]
    fn levels_are_in_dbfs() {
        assert_eq!(peak_db(&[]), FLOOR);
        assert_eq!(peak_db(&[0, 0]), FLOOR);
        assert!((peak_db(&[0, 16384, -3]) - -6.02).abs() < 0.01);
        assert!(peak_db(&[i16::MIN]).abs() < 1e-9);
        // A full-scale sine's RMS is 3 dB under its peak.
        assert!((rms_db(&tone(1.0, 440.0, 1.0)) - -3.01).abs() < 0.01);
        assert_eq!(rms_db(&silence(0.1)), FLOOR);
    }

    #[test]
    fn a_take_is_trimmed_to_its_words() {
        // A key click, a pause, 2 s of speech, a pause, and the stop key.
        let mut raw = click();
        raw.extend(silence(0.6));
        let words = raw.len();
        raw.extend(tone(2.0, 220.0, 0.3));
        raw.extend(silence(0.7));
        raw.extend(click());
        let kept = clean(&raw, RATE).unwrap();
        // The words, with 0.15 s before them and 0.25 s after.
        let from = seconds(words) - seconds(kept.from);
        let after = seconds(kept.to) - (seconds(words) + 2.0);
        assert!((from - LEAD).abs() < 0.01, "{from}");
        assert!((after - TAIL).abs() < 0.01, "{after}");
        assert_eq!(kept.samples.len(), kept.to - kept.from);
        // The words are found to within the window the level is read over.
        // (The filter rings on for a moment after the tone stops.)
        let (first, last) = kept.words;
        let window = samples_in(WINDOW, RATE);
        assert!(first.abs_diff(words) < window, "{first} {words}");
        let end = words + samples_in(2.0, RATE) - 1;
        assert!(last.abs_diff(end) < window, "{last} {end}");
        // Neither click is in it, and the tone is, about as loud as it was
        // (the filter overshoots a little where the tone starts and stops).
        let (peak, tone) = (peak_db(&kept.samples), 20.0 * 0.3f64.log10());
        assert!((peak - tone).abs() < 1.0, "{peak} {tone}");
        // The raw take is as it was.
        assert_eq!(&raw[..6], click());
    }

    #[test]
    fn a_take_too_short_or_with_no_speech_is_refused() {
        let short = tone(0.5, 220.0, 0.3);
        assert_eq!(clean(&short, RATE), Err(Refused::TooShort(0.5)));
        assert_eq!(
            Refused::TooShort(0.5).to_string(),
            "that take was only 0.5 s, so it was discarded"
        );
        // Silence, and a room's hum far below speech.
        assert_eq!(clean(&silence(3.0), RATE), Err(Refused::Silent(FLOOR)));
        let hum = tone(3.0, 100.0, 0.003);
        let Err(Refused::Silent(peak)) = clean(&hum, RATE) else {
            panic!("a hum isn't speech");
        };
        assert!((peak - -50.5).abs() < 0.2, "{peak}");
        assert!(Refused::Silent(peak).to_string().starts_with(
            "there was no speech in that take (its loudest point was -50 dB), so it was discarded"
        ));
        // Only the keypresses are loud: nothing is left once they are dropped.
        let mut keys = click();
        keys.extend(silence(2.0));
        keys.extend(click());
        assert_eq!(clean(&keys, RATE), Err(Refused::NothingLeft));
    }

    #[test]
    fn rumble_is_filtered_out_and_speech_is_left() {
        let rumble = high_pass(&tone(1.0, 20.0, 0.5), RATE, HIGH_PASS);
        let speech = high_pass(&tone(1.0, 220.0, 0.5), RATE, HIGH_PASS);
        // After the filter settles: 20 Hz is down by some 22 dB, 220 Hz by next to nothing.
        let settled = RATE as usize / 2;
        assert!(rms_db(&rumble[settled..]) < rms_db(&tone(1.0, 20.0, 0.5)) - 20.0);
        assert!((rms_db(&speech[settled..]) - rms_db(&tone(1.0, 220.0, 0.5))).abs() < 0.1);
        // A steady offset, as some inputs have, goes too.
        let offset = high_pass(&vec![5000; RATE as usize], RATE, HIGH_PASS);
        assert!(offset[settled..].iter().all(|s| s.abs() < 3));
    }
}
