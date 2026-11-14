//! The recorder's trim against the reference's (TODO.md, P4-VIDNI-01).
//!
//! Each line of the nibli example has the take the reference's recorder
//! kept, and the raw takes it was recorded as. The kept take is what
//! ffmpeg's filter chain made of one of them (`CLEAN` in
//! reference/record.py): the keypresses dropped, a high-pass filter, and
//! `silenceremove` at each end.
//!
//! vidni's trim (`vidni_record::clean`) is the same chain, with one
//! difference. Where `silenceremove` splices the silence it keeps onto the
//! sound, it writes one sample from long before and repeats the sample the
//! sound begins at. vidni keeps the stretch whole. So what is checked for
//! each line is that vidni's trim of one of its raw takes, spliced as
//! ffmpeg splices it, is the kept take, sample for sample.

use crate::Report;
use std::path::Path;
use vidni_core::takes::{self, Status};
use vidni_record::clean::{self, LEAD, TAIL};
use vidni_record::take;

/// What ffmpeg's `silenceremove` makes of `samples`, a stretch that begins
/// with `keep` samples of silence at most and whose sound begins at
/// `sound`: the stretch as ffmpeg splices it.
///
/// The filter holds the newest `keep + 1` samples in a ring. When the
/// sound begins it writes out as many samples as it has counted, `keep` at
/// most, backwards from the ring's next place. That place holds the oldest
/// sample, which is the stretch's first, or nothing when the take began
/// less than `keep` samples before. Then it carries on from the sample the
/// sound began at, which it has just written.
fn spliced(samples: &[i16], sound: usize, keep: usize) -> Vec<i16> {
    if keep == 0 {
        return samples.to_vec();
    }
    let (kept, oldest) = match sound == keep {
        true => (&samples[2..=sound], samples[0]),
        false => (&samples[1..=sound], 0),
    };
    [kept, &[oldest], &samples[sound..]].concat()
}

/// The reference's kept take of `raw`: vidni's trim of it, spliced at each
/// end as ffmpeg splices it. None when vidni refuses the take.
fn reference(raw: &[i16], rate: u32) -> Option<Vec<i16>> {
    let at = |seconds: f64| (seconds * f64::from(rate)).round() as usize;
    let trimmed = clean::clean(raw, rate).ok()?;
    let (first, last) = trimmed.words;
    let mut reversed = spliced(&trimmed.samples, first - trimmed.from, at(LEAD));
    reversed.reverse();
    // The second pass reads the first one's output from its end.
    let mut kept = spliced(&reversed, trimmed.to - 1 - last, at(TAIL));
    kept.reverse();
    Some(kept)
}

/// Checks each line's kept take against vidni's trim of its raw takes.
pub fn check(example: &Path, report: &mut Report) {
    if let Err(e) = compare(example, report) {
        report.fail(e);
    }
}

fn compare(example: &Path, report: &mut Report) -> Result<(), String> {
    let project = crate::candidate::project(example)?;
    for line in takes::status(&project, example) {
        let Status::Current(kept) = &line.status else {
            return Err(format!("line {} has no current take", line.id));
        };
        let (kept, rate) = take::read(kept)?;
        // One of the line's raw takes is the one its kept take was made from.
        let mut made = false;
        for n in 1.. {
            let raw = takes::raw(example, &line.id, &line.hash, n);
            if made || !raw.is_file() {
                break;
            }
            let (samples, _) = take::read(&raw)?;
            made = reference(&samples, rate).as_ref() == Some(&kept);
        }
        report.check(made, || {
            format!(
                "line {}: no raw take of it trims to its kept take, sample for sample",
                line.id
            )
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_splice_writes_the_oldest_sample_and_repeats_the_first() {
        let samples: Vec<i16> = (10..30).collect();
        // Three samples of silence, and the sound from the fourth: ffmpeg
        // writes two of the three, then the oldest, then the sound.
        assert_eq!(
            spliced(&samples, 3, 3),
            [&[12, 13, 10][..], &samples[3..]].concat()
        );
        // A take that began less than three samples before its sound:
        // the ring's next place holds nothing yet.
        assert_eq!(
            spliced(&samples, 1, 3),
            [&[11, 0][..], &samples[1..]].concat()
        );
        assert_eq!(spliced(&samples, 0, 3), [&[0][..], &samples[..]].concat());
        // With no silence kept, the sound is all there is.
        assert_eq!(spliced(&samples, 0, 0), samples);
    }
}
