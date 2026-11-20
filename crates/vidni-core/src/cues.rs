//! Reviewed emphasis intervals. Alignment proposes; only a person verifies.
//! Offsets belong to exact audio bytes, never to a guessed reading speed.

use crate::capture::{canonical, sha256_hex};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

pub const LOCK: &str = "cues.lock";
pub const PROPOSALS: &str = "build/cues/proposals.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub audio: String,
    pub rate: u32,
    pub samples: u64,
    pub words: String,
    pub definition: String,
    /// Includes measured capture, target, camera, theme and output layout.
    pub visual: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Interval {
    pub start: u64,
    pub end: u64,
}

impl Interval {
    pub fn check(self, binding: &Binding) -> Result<Self, String> {
        if binding.rate == 0 || self.start >= self.end || self.end > binding.samples {
            return Err("cue needs start < end within its exact audio take".into());
        }
        Ok(self)
    }

    /// Half-open frame interval. Neither endpoint appears before its sample.
    pub fn frames(self, line_start: f64, rate: u32, fps: u32) -> [u64; 2] {
        let at = |sample| {
            ((line_start + sample as f64 / f64::from(rate)) * f64::from(fps) - 1e-9).ceil() as u64
        };
        [at(self.start), at(self.end)]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub binding: Binding,
    pub interval: Interval,
}

/// An owner's explicit review decision, tied to the exact continuous video.
/// This records who made the decision separately from who saved it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Confirmation {
    pub cut: String,
    pub voice: crate::release::Voice,
    pub owner_statement: String,
    pub recorded_by: String,
    pub review_sha256: String,
    pub receipt_sha256: String,
    pub review_identity: String,
    pub cue_keys: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Records {
    pub schema: u32,
    pub entries: BTreeMap<String, Record>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub confirmations: Vec<Confirmation>,
}

impl Default for Records {
    fn default() -> Self {
        Self {
            schema: 1,
            entries: BTreeMap::new(),
            confirmations: Vec::new(),
        }
    }
}

impl Records {
    pub fn read(path: &Path) -> Result<Self, String> {
        let bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => return Err(format!("{}: {e}", path.display())),
        };
        let records: Self =
            serde_json::from_slice(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
        if records.schema != 1 {
            return Err(format!("{}: unsupported cue record schema", path.display()));
        }
        Ok(records)
    }

    pub fn write(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        let temp = path.with_extension("json.new");
        std::fs::write(&temp, bytes).map_err(|e| e.to_string())?;
        std::fs::rename(temp, path).map_err(|e| e.to_string())
    }

    pub fn current(&self, key: &str, binding: &Binding) -> Result<Interval, String> {
        let record = self
            .entries
            .get(key)
            .ok_or_else(|| format!("{key}: cue is unverified"))?;
        if &record.binding != binding {
            return Err(format!(
                "{key}: cue is stale; its audio, words, target or layout changed"
            ));
        }
        record
            .interval
            .check(binding)
            .map_err(|why| format!("{key}: {why}"))
    }
}

pub fn hash(value: &impl Serialize) -> String {
    sha256_hex(canonical(&serde_json::to_value(value).expect("cue data serializes")).as_bytes())
}

pub fn key(cut: &str, voice: crate::release::Voice, line: &str, cue: &str) -> String {
    let voice = match voice {
        crate::release::Voice::Draft => "draft",
        crate::release::Voice::Takes => "takes",
    };
    format!("{cut}/{voice}/{line}/{cue}")
}

/// Exact substring selection in UTF-8 bytes. No fuzzy or first-match fallback.
pub fn occurrence(text: &str, phrase: &str, which: Option<u32>) -> Result<[usize; 2], String> {
    if phrase.is_empty() {
        return Err("the selected phrase is empty".into());
    }
    let starts: Vec<usize> = text.match_indices(phrase).map(|(at, _)| at).collect();
    let at = match (which, starts.as_slice()) {
        (None, [at]) => *at,
        (None, []) => return Err(format!("{phrase:?} does not occur in the text")),
        (None, _) => return Err(format!("{phrase:?} is ambiguous; specify its occurrence")),
        (Some(n), _) => *n
            .checked_sub(1)
            .and_then(|n| starts.get(n as usize))
            .ok_or_else(|| format!("{phrase:?} has no occurrence {n}"))?,
    };
    Ok([at, at + phrase.len()])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binding() -> Binding {
        Binding {
            audio: "audio".into(),
            rate: 48000,
            samples: 96000,
            words: "words".into(),
            definition: "cue".into(),
            visual: "layout".into(),
        }
    }

    #[test]
    fn repeated_phrases_require_a_choice_and_unicode_is_not_counted_as_bytes() {
        assert!(
            occurrence("P and P", "P", None)
                .unwrap_err()
                .contains("ambiguous")
        );
        assert_eq!(occurrence("é P and P", "P", Some(2)).unwrap(), [9, 10]);
        assert!(occurrence("P", "P", Some(0)).is_err());
    }

    #[test]
    fn every_change_to_reviewed_inputs_invalidates_the_cue() {
        let original = binding();
        let mut records = Records::default();
        records.entries.insert(
            "cue".into(),
            Record {
                binding: original.clone(),
                interval: Interval {
                    start: 1,
                    end: 95000,
                },
            },
        );
        assert!(records.current("cue", &original).is_ok());
        for changed in [
            Binding {
                audio: "retake".into(),
                ..original.clone()
            },
            Binding {
                samples: 95999,
                ..original.clone()
            },
            Binding {
                rate: 24000,
                ..original.clone()
            },
            Binding {
                words: "edited".into(),
                ..original.clone()
            },
            Binding {
                definition: "new target".into(),
                ..original.clone()
            },
            Binding {
                visual: "new font/crop".into(),
                ..original.clone()
            },
        ] {
            assert!(
                records
                    .current("cue", &changed)
                    .unwrap_err()
                    .contains("stale")
            );
        }
        assert!(records.current("absent", &original).is_err());
    }

    #[test]
    fn offsets_are_inside_the_take_and_round_forward_to_frames() {
        let b = binding();
        assert!(Interval { start: 8, end: 8 }.check(&b).is_err());
        assert!(
            Interval {
                start: 8,
                end: 96001
            }
            .check(&b)
            .is_err()
        );
        assert_eq!(
            Interval {
                start: 1,
                end: 48000
            }
            .frames(1.0, 48000, 30),
            [31, 60]
        );
        assert_eq!(
            Interval {
                start: 0,
                end: 1600
            }
            .frames(1.0, 48000, 30),
            [30, 31]
        );
    }
}
