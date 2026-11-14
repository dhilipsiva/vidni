//! Room tone: a few seconds of the room with nobody speaking, recorded
//! during the mic check. Its level says how noisy the room or the
//! microphone is, and it is kept as the sound of the project's silence.

use crate::clean;
use crate::take;
use std::fs;
use std::path::{Path, PathBuf};

/// How long the room is recorded for, in seconds.
pub const SECONDS: f64 = 3.0;
/// What is left out of the recording at its start, where the key that
/// started it may sound and the input settles, and at its end. Seconds.
pub const SETTLE: f64 = 0.3;
pub const END: f64 = 0.2;
/// A room louder than this is noisy, in dBFS.
pub const NOISY: f64 = -55.0;

/// The project's room tone, relative to its takes.
pub const FILE: &str = "_room/tone.wav";

/// A room, as recorded.
#[derive(Debug, Clone, PartialEq)]
pub struct Room {
    /// The room tone's file.
    pub path: PathBuf,
    /// Its level, in dBFS: the RMS of the whole of it.
    pub level: f64,
}

impl Room {
    /// Whether the room, or the microphone, is noisy.
    pub fn noisy(&self) -> bool {
        self.level > NOISY
    }
}

/// Where a project's room tone is.
pub fn path(project_dir: &Path) -> PathBuf {
    project_dir.join(vidni_core::takes::DIR).join(FILE)
}

/// Keeps the recording of the room at `raw` as the project's room tone:
/// its middle, without [`SETTLE`] at the start and [`END`] at the end. The
/// recording itself is removed. One that is too short to have a middle is
/// refused, and the room tone from before stays.
pub fn keep(raw: &Path, project_dir: &Path) -> Result<Room, String> {
    let (samples, rate) = take::read(raw)?;
    let at = |seconds: f64| (seconds * f64::from(rate)).round() as usize;
    let middle = samples
        .get(at(SETTLE)..samples.len().saturating_sub(at(END)))
        .filter(|middle| middle.len() >= at(1.0))
        .ok_or_else(|| {
            format!(
                "the room was recorded for {:.1} s, which is too short to measure",
                samples.len() as f64 / f64::from(rate.max(1))
            )
        })?;
    let tone = path(project_dir);
    let part = take::candidate_path(&tone);
    take::write(&part, middle, rate)?;
    fs::rename(&part, &tone).map_err(|e| format!("{}: {e}", tone.display()))?;
    fs::remove_file(raw).map_err(|e| format!("{}: {e}", raw.display()))?;
    Ok(Room {
        path: tone,
        level: clean::rms_db(middle),
    })
}
