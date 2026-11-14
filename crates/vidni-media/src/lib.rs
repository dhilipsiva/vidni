//! vidni's media stages: compose, encode and QA, TTS, capture and the
//! intelligibility check, and the build that runs them for a cut. They
//! drive external tools pinned by version and sha256. See `TODO.md`,
//! milestone P3.

pub mod build;
pub mod capture;
pub mod cdp;
pub mod compose;
pub mod encode;
pub mod export;
pub mod hear;
pub mod narration;
pub mod preview;
pub mod qa;
pub mod review;
pub mod sheet;
pub mod textscene;
pub mod tts;
pub mod voice;

use std::process::{Command, Output};

/// Runs a command; its stderr becomes the error when it fails.
pub(crate) fn run(cmd: &mut Command) -> Result<Output, String> {
    let shown = format!("{cmd:?}");
    let out = cmd.output().map_err(|e| format!("{shown}: {e}"))?;
    if out.status.success() {
        Ok(out)
    } else {
        Err(format!(
            "{shown}: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
}

/// The packet log of a media file (`ffmpeg -map 0 -c copy -f framemd5`).
pub fn framemd5(path: &std::path::Path) -> Result<Vec<u8>, String> {
    run(Command::new("ffmpeg")
        .args(["-nostdin", "-loglevel", "error", "-i"])
        .arg(path)
        .args(["-map", "0", "-c", "copy", "-f", "framemd5", "-"]))
    .map(|out| out.stdout)
}
