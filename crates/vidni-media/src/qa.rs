//! QA of an encoded cut: its streams, duration, loudness, captions and chapters.

use crate::run;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// What a cut must be.
#[derive(Debug, Clone, PartialEq)]
pub struct Expect {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    /// The timeline's length, in seconds.
    pub duration: f64,
    /// The loudness target, in LUFS, and how far off it may be, in LU.
    pub loudness: f64,
    pub tolerance: f64,
    /// The caption file, and the number of cues the timeline has.
    pub captions: Option<(PathBuf, usize)>,
    /// Each chapter's start in seconds and its title, when the cut has chapters.
    pub chapters: Option<Vec<(f64, String)>>,
}

/// What QA measured, and what fails.
#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    pub summary: String,
    pub problems: Vec<String>,
}

/// Measures `mp4` with ffprobe and ffmpeg, and lists every way it misses `expect`.
pub fn check(mp4: &Path, expect: &Expect) -> Result<Report, String> {
    let probe = run(Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_streams",
            "-show_format",
            "-show_chapters",
            "-of",
            "json",
        ])
        .arg(mp4))?;
    let info: Value = serde_json::from_slice(&probe.stdout).map_err(|e| e.to_string())?;
    let streams = info["streams"].as_array().cloned().unwrap_or_default();
    let of = |kind: &str| streams.iter().find(|s| s["codec_type"] == kind).cloned();
    let mut problems = Vec::new();
    let (video, audio) = (of("video"), of("audio"));
    let video_line = match &video {
        None => {
            problems.push("there is no video stream".to_string());
            "no video".to_string()
        }
        Some(v) => {
            let (w, h) = (
                v["width"].as_u64().unwrap_or(0),
                v["height"].as_u64().unwrap_or(0),
            );
            let codec = v["codec_name"].as_str().unwrap_or("?");
            let fps = v["r_frame_rate"].as_str().unwrap_or("?");
            if (w, h) != (u64::from(expect.width), u64::from(expect.height)) {
                problems.push(format!(
                    "the picture is {w}x{h}, not {}x{}",
                    expect.width, expect.height
                ));
            }
            if codec != "h264" {
                problems.push(format!("the video codec is {codec}, not h264"));
            }
            if fps != format!("{}/1", expect.fps) {
                problems.push(format!("the frame rate is {fps}, not {}/1", expect.fps));
            }
            format!("{w}x{h} {codec} {fps} fps")
        }
    };
    let audio_line = match &audio {
        None => {
            problems.push("there is no audio stream".to_string());
            "no audio".to_string()
        }
        Some(a) => {
            let codec = a["codec_name"].as_str().unwrap_or("?");
            let rate = a["sample_rate"].as_str().unwrap_or("?");
            let channels = a["channels"].as_u64().unwrap_or(0);
            if codec != "aac" || rate != "48000" || channels != 2 {
                problems.push(format!("the audio is {codec} at {rate} Hz with {channels} channels, not AAC at 48000 Hz in stereo"));
            }
            format!("{codec} {rate} Hz {channels} ch")
        }
    };
    let duration: f64 = info["format"]["duration"]
        .as_str()
        .and_then(|d| d.parse().ok())
        .unwrap_or(0.0);
    if (duration - expect.duration).abs() > 0.5 {
        problems.push(format!(
            "it lasts {duration:.2} s, and the timeline {:.2} s",
            expect.duration
        ));
    }
    let lufs = loudness(mp4)?;
    match lufs {
        Some(l) if (l - expect.loudness).abs() <= expect.tolerance => {}
        Some(l) => problems.push(format!(
            "its loudness is {l} LUFS, more than {} LU from {} LUFS",
            expect.tolerance, expect.loudness
        )),
        None => problems.push("its loudness couldn't be measured".to_string()),
    }
    if let Some((srt, cues)) = &expect.captions {
        let text = fs::read_to_string(srt).map_err(|e| format!("{}: {e}", srt.display()))?;
        let found = text.lines().filter(|l| l.contains("-->")).count();
        if found != *cues {
            problems.push(format!(
                "{} has {found} cues, and the timeline {cues}",
                srt.display()
            ));
        }
    }
    if let Some(want) = &expect.chapters {
        let got: Vec<(f64, String)> = info["chapters"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|c| {
                let start = c["start_time"]
                    .as_str()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(-1.0);
                (
                    start,
                    c["tags"]["title"].as_str().unwrap_or_default().to_string(),
                )
            })
            .collect();
        let same = got.len() == want.len()
            && got
                .iter()
                .zip(want)
                .all(|(g, w)| (g.0 - w.0).abs() < 0.001 && g.1 == w.1);
        if !same {
            problems.push(format!(
                "it has {} chapters, not the {} the cut defines",
                got.len(),
                want.len()
            ));
        }
    }
    let summary = format!(
        "{video_line}; {audio_line}; {duration:.2} s; {}",
        lufs.map_or("loudness unknown".to_string(), |l| format!("{l} LUFS"))
    );
    Ok(Report { summary, problems })
}

/// The integrated loudness of a file's audio, in LUFS (EBU R 128).
pub fn loudness(path: &Path) -> Result<Option<f64>, String> {
    let out = Command::new("ffmpeg")
        .args(["-nostdin", "-hide_banner", "-nostats", "-i"])
        .arg(path)
        .args(["-map", "0:a", "-af", "ebur128", "-f", "null", "-"])
        .output()
        .map_err(|e| format!("ffmpeg: {e}"))?;
    let log = String::from_utf8_lossy(&out.stderr);
    Ok(log
        .lines()
        .filter_map(|l| l.trim().strip_prefix("I:"))
        .filter_map(|rest| rest.split_whitespace().next()?.parse().ok())
        .next_back())
}
