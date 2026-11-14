//! The timeline: when each line plays and when each of its captions shows.
//!
//! Lines play one after another from `timing.lead_in`, each as long as its
//! clip, with `timing.line_gap` between lines of a section and
//! `timing.section_gap` between sections. A long line's caption is split
//! into cues (see [`cues_for`]), and the cues share the line's time in
//! proportion to their length. Times are rounded to milliseconds the way the
//! reference pipeline rounds them, so that the timeline matches its golden
//! output exactly (TODO.md, P2-VIDNI-04).

use crate::project::{Line, Project, SceneKind};
use crate::release::Voice;
use crate::spoken::{spoken, synthetic};
use serde::{Deserialize, Serialize};

/// The longest caption cue, in characters, before a line is split.
pub const CUE_CHARS: usize = 92;

/// A line's audio clip: its name and its length in samples.
#[derive(Debug, Clone, PartialEq)]
pub struct Clip {
    /// How the timeline names the clip.
    pub wav: String,
    pub frames: u64,
    pub rate: u32,
}

impl Clip {
    fn seconds(&self) -> f64 {
        self.frames as f64 / f64::from(self.rate)
    }
}

/// The whole timeline, as the reference pipeline writes it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Timeline {
    /// The length in seconds, from the start to the end of the tail.
    pub total: f64,
    /// The clips' sample rate.
    pub rate: u32,
    pub sentences: Vec<Entry>,
}

/// One line on the timeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    /// The line's position in the script, from 0.
    pub k: usize,
    /// The line's section.
    pub segment: String,
    /// The section's label.
    pub label: String,
    /// The line's position in its section, from 0.
    pub index: usize,
    pub start: f64,
    pub end: f64,
    pub wav: String,
    pub show: String,
    /// What the line's voice is given: the draft voice's text, with its
    /// respellings, or the words the creator reads.
    pub say: String,
    /// The page scene the line shows full-frame, if any.
    pub card: Option<String>,
    pub cues: Vec<Cue>,
}

/// One caption cue.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cue {
    pub start: f64,
    pub end: f64,
    pub text: String,
}

/// `x` rounded to `places` decimals as Python's `round` does: the exact
/// binary value, rounded half to even, then the nearest double.
pub fn round(x: f64, places: usize) -> f64 {
    format!("{x:.places$}").parse().unwrap_or(x)
}

/// Builds the timeline of `project`'s lines from their clips, one per line
/// in script order, as the reference writes it: `say` is what the draft
/// voice is given, whoever speaks.
pub fn build(project: &Project, clips: &[Clip]) -> Result<Timeline, String> {
    let lines: Vec<&Line> = project.lines.iter().collect();
    build_for(project, &lines, clips, Voice::Draft)
}

/// Builds the timeline of `lines`, the lines of one cut in script order,
/// from their clips, spoken by `voice`. A line's position, and a section's
/// start and end, are its place among these lines.
pub fn build_for(
    project: &Project,
    lines: &[&Line],
    clips: &[Clip],
    voice: Voice,
) -> Result<Timeline, String> {
    if clips.len() != lines.len() {
        return Err(format!("{} clips for {} lines", clips.len(), lines.len()));
    }
    let rate = clips.first().map_or(0, |c| c.rate);
    if let Some(c) = clips.iter().find(|c| c.rate != rate) {
        return Err(format!(
            "{}: sample rate {} differs from {rate}",
            c.wav, c.rate
        ));
    }
    let timing = &project.timing;
    let mut t = timing.lead_in;
    let mut entries: Vec<Entry> = Vec::with_capacity(clips.len());
    let mut index = 0;
    for (k, (line, clip)) in lines.iter().zip(clips).enumerate() {
        if k > 0 && lines[k - 1].section != line.section {
            index = 0;
        }
        let dur = clip.seconds();
        let (start, end) = (t, t + dur);
        let last_in_section = lines
            .get(k + 1)
            .is_none_or(|next| next.section != line.section);
        t = end
            + if last_in_section {
                timing.section_gap
            } else {
                timing.line_gap
            };
        let chunks = cues_for(&line.show);
        let total_chars: usize = chunks.iter().map(|c| c.chars().count()).sum();
        let mut cues = Vec::with_capacity(chunks.len());
        let mut c0 = start;
        for chunk in chunks {
            let c1 = c0 + dur * chunk.chars().count() as f64 / total_chars as f64;
            cues.push(Cue {
                start: round(c0, 3),
                end: round(c1, 3),
                text: chunk,
            });
            c0 = c1;
        }
        let card = line
            .scene_in(project)
            .filter(|id| {
                project
                    .scenes
                    .get(*id)
                    .is_some_and(|s| s.kind == SceneKind::Page)
            })
            .map(str::to_string);
        entries.push(Entry {
            k,
            segment: line.section.clone(),
            label: project
                .sections
                .get(&line.section)
                .map(|s| s.label.clone())
                .unwrap_or_default(),
            index,
            start: round(start, 3),
            end: round(end, 3),
            wav: clip.wav.clone(),
            show: line.show.clone(),
            // A final cut's timeline holds nothing of the draft voice's
            // settings, so a respelling changes no release.
            say: match voice {
                Voice::Draft => synthetic(line, &project.voice),
                Voice::Takes => spoken(line, &project.voice),
            },
            card,
            cues,
        });
        index += 1;
    }
    let total = entries
        .last()
        .map_or(0.0, |e| round(e.end + timing.tail, 3));
    Ok(Timeline {
        total,
        rate,
        sentences: entries,
    })
}

/// Splits a caption longer than [`CUE_CHARS`] into the fewest cues of about
/// that length, cutting at punctuation near each ideal point when there is
/// any. Lengths are in characters, and words are separated by single spaces.
pub fn cues_for(text: &str) -> Vec<String> {
    let len = text.chars().count();
    if len <= CUE_CHARS {
        return vec![text.to_string()];
    }
    let words: Vec<&str> = text.split(' ').collect();
    let n = len.div_ceil(CUE_CHARS);
    // Where each word but the last ends, counting the space after it.
    let mut ends: Vec<(usize, usize, &str)> = Vec::new();
    let mut pos = 0;
    for (i, w) in words[..words.len() - 1].iter().enumerate() {
        pos += w.chars().count() + 1;
        ends.push((pos, i, w));
    }
    let bonus = |w: &str| match w.chars().last() {
        Some('.' | '?' | '!') => 30.0,
        Some(':' | ';') => 26.0,
        Some(',') => 24.0,
        _ => 0.0,
    };
    let mut cuts: Vec<(usize, usize)> = Vec::new();
    for j in 1..n {
        let ideal = (len * j) as f64 / n as f64;
        let start = cuts.last().map_or(0, |c| c.0);
        let best = ends
            .iter()
            .filter(|(p, i, _)| cuts.last().is_none_or(|c| *i > c.1) && p - start <= CUE_CHARS + 16)
            .map(|(p, i, w)| (((*p as f64) - ideal).abs() - bonus(w), *p, *i))
            .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
        match best {
            Some((_, p, i)) => cuts.push((p, i)),
            // No word ends in reach (one very long word): stop splitting.
            None => break,
        }
    }
    let mut chunks = Vec::with_capacity(cuts.len() + 1);
    let mut prev = 0;
    for i in cuts.iter().map(|c| c.1).chain([words.len() - 1]) {
        chunks.push(words[prev..=i].join(" "));
        prev = i + 1;
    }
    chunks
}

fn clock(t: f64, separator: char) -> String {
    let ms = (t * 1000.0).round_ties_even() as i64;
    let (h, ms) = (ms / 3_600_000, ms % 3_600_000);
    let (m, ms) = (ms / 60_000, ms % 60_000);
    let (s, ms) = (ms / 1000, ms % 1000);
    format!("{h:02}:{m:02}:{s:02}{separator}{ms:03}")
}

/// The captions as SubRip (SRT), numbered from 1.
pub fn srt(timeline: &Timeline) -> String {
    let mut out = String::new();
    let cues = timeline.sentences.iter().flat_map(|e| &e.cues);
    for (n, cue) in cues.enumerate() {
        out.push_str(&format!(
            "{}\n{} --> {}\n{}\n\n",
            n + 1,
            clock(cue.start, ','),
            clock(cue.end, ','),
            cue.text
        ));
    }
    out
}

/// The captions as WebVTT.
pub fn vtt(timeline: &Timeline) -> String {
    let mut out = String::from("WEBVTT\n\n");
    for cue in timeline.sentences.iter().flat_map(|e| &e.cues) {
        out.push_str(&format!(
            "{} --> {}\n{}\n\n",
            clock(cue.start, '.'),
            clock(cue.end, '.'),
            cue.text
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounding_matches_python() {
        // Values from Python 3: round(x, 3) and round(x, 2).
        assert_eq!(round(0.6, 3), 0.6);
        assert_eq!(round(2.675, 2), 2.67); // 2.675 is just below 2.675
        assert_eq!(round(0.125, 2), 0.12); // an exact tie goes to even
        assert_eq!(round(0.375, 2), 0.38);
        assert_eq!(round(1.0005, 3), 1.0);
        assert_eq!(round(12.3456789, 3), 12.346);
    }

    #[test]
    fn short_captions_stay_whole_and_long_ones_split_at_punctuation() {
        assert_eq!(cues_for("Adam is a dog."), ["Adam is a dog."]);
        let text = "The engine starts from the claim and works backward through every rule it knows, \
                    and it stops when the chain closes on a stored fact.";
        assert_eq!(
            cues_for(text),
            [
                "The engine starts from the claim and works backward through every rule it knows,",
                "and it stops when the chain closes on a stored fact."
            ]
        );
    }

    #[test]
    fn srt_and_vtt_clocks() {
        assert_eq!(clock(3723.0456, ','), "01:02:03,046");
        assert_eq!(clock(0.0005, '.'), "00:00:00.000"); // 0.5 ms is a tie, rounded to even
        assert_eq!(clock(0.0015, '.'), "00:00:00.002");
    }
}
