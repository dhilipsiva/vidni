//! The narration track: every line's clip at its start on the timeline,
//! silence between, as one 16-bit mono WAV at the clips' rate.
//!
//! It is assembled as the reference assembles it, so its samples equal the
//! golden narration exactly (`just parity narration`): a clip starts at
//! sample `floor(start × rate)`, taking the timeline's rounded start, and the
//! track lasts `floor(total × rate)` samples.

use std::path::Path;
use vidni_core::timeline::Timeline;

/// Writes the narration of `timeline` to `out`, reading each line's clip
/// from `clips`, in script order.
pub fn assemble(timeline: &Timeline, clips: &[&Path], out: &Path) -> Result<(), String> {
    if clips.len() != timeline.sentences.len() {
        return Err(format!(
            "{} clips for {} lines",
            clips.len(),
            timeline.sentences.len()
        ));
    }
    let rate = timeline.rate;
    let mut samples: Vec<i16> = vec![0; (timeline.total * f64::from(rate)) as usize];
    for (entry, clip) in timeline.sentences.iter().zip(clips) {
        let mut reader =
            hound::WavReader::open(clip).map_err(|e| format!("{}: {e}", clip.display()))?;
        let spec = reader.spec();
        if spec.channels != 1
            || spec.bits_per_sample != 16
            || spec.sample_format != hound::SampleFormat::Int
        {
            return Err(format!(
                "{}: the narration needs 16-bit mono clips",
                clip.display()
            ));
        }
        if spec.sample_rate != rate {
            return Err(format!(
                "{}: sample rate {} differs from {rate}",
                clip.display(),
                spec.sample_rate
            ));
        }
        let data: Vec<i16> = reader
            .samples::<i16>()
            .collect::<Result<_, _>>()
            .map_err(|e| format!("{}: {e}", clip.display()))?;
        let at = (entry.start * f64::from(rate)) as usize;
        if samples.len() < at + data.len() {
            samples.resize(at + data.len(), 0);
        }
        samples[at..at + data.len()].copy_from_slice(&data);
    }
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer =
        hound::WavWriter::create(out, spec).map_err(|e| format!("{}: {e}", out.display()))?;
    for s in samples {
        writer.write_sample(s).map_err(|e| e.to_string())?;
    }
    writer
        .finalize()
        .map_err(|e| format!("{}: {e}", out.display()))
}
