//! QA of an encoded cut: its streams, duration, loudness, captions and chapters.

use crate::run;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use vidni_core::chapters::{self, Chapter};
use vidni_core::platforms::Upload;

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
    /// The highest true peak, in dBTP.
    pub true_peak: f64,
    /// The caption file, and the number of cues the timeline has.
    pub captions: Option<(PathBuf, usize)>,
    /// The cut's chapters; none when it has no list.
    pub chapters: Vec<Chapter>,
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
    let sound = measure(mp4)?;
    let lufs = sound.loudness;
    match lufs {
        Some(l) if (l - expect.loudness).abs() <= expect.tolerance => {}
        Some(l) => problems.push(format!(
            "its loudness is {l} LUFS, more than {} LU from {} LUFS",
            expect.tolerance, expect.loudness
        )),
        None => problems.push("its loudness couldn't be measured".to_string()),
    }
    match sound.true_peak {
        Some(peak) if peak <= expect.true_peak => {}
        Some(peak) => problems.push(format!(
            "its true peak is {peak} dBTP, above {} dBTP",
            expect.true_peak
        )),
        None => problems.push("its true peak couldn't be measured".to_string()),
    }
    if let Some((at, lasts)) = sound.silences.first() {
        problems.push(format!(
            "it has {} of digital silence longer than {} ms; the first is {lasts:.2} s long, at {at:.2} s",
            match sound.silences.len() {
                1 => "a stretch".to_string(),
                n => format!("{n} stretches"),
            },
            (SILENCE * 1000.0) as u32
        ));
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
        problems.extend(captions(&text, duration));
    }
    // The file holds the cut's chapters and no others, and the list keeps
    // YouTube's rules in a video as long as this file.
    {
        let want = &expect.chapters;
        problems.extend(chapters::problems(want, duration));
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
                .all(|(g, w)| (g.0 - f64::from(w.start)).abs() < 0.001 && g.1 == w.title);
        if got.len() != want.len() {
            problems.push(format!(
                "it has {} chapters, not the {} the cut defines",
                got.len(),
                want.len()
            ));
        } else if !same {
            // The first one that differs says what went wrong.
            let (n, (g, w)) = got
                .iter()
                .zip(want)
                .enumerate()
                .find(|(_, (g, w))| (g.0 - f64::from(w.start)).abs() >= 0.001 || g.1 != w.title)
                .expect("a chapter differs");
            problems.push(format!(
                "its chapter {} is {:?} at {} s, not {:?} at {} s",
                n + 1,
                g.1,
                g.0,
                w.title,
                w.start
            ));
        }
    }
    let summary = format!(
        "{video_line}; {audio_line}; {duration:.2} s; {}; {}",
        lufs.map_or("loudness unknown".to_string(), |l| format!("{l} LUFS")),
        sound
            .true_peak
            .map_or("peak unknown".to_string(), |p| format!("{p} dBTP"))
    );
    Ok(Report { summary, problems })
}

/// A file as a platform would see it: its shape, its codecs, its length,
/// its size and its sound.
pub fn upload(mp4: &Path, aspect: vidni_core::project::Aspect) -> Result<Upload, String> {
    let probe = run(Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_streams",
            "-show_format",
            "-of",
            "json",
        ])
        .arg(mp4))?;
    let info: Value = serde_json::from_slice(&probe.stdout).map_err(|e| e.to_string())?;
    let streams = info["streams"].as_array().cloned().unwrap_or_default();
    let of = |kind: &str| {
        streams
            .iter()
            .find(|s| s["codec_type"] == kind)
            .cloned()
            .unwrap_or(Value::Null)
    };
    let (video, audio) = (of("video"), of("audio"));
    let text = |value: &Value| value.as_str().unwrap_or("none").to_string();
    let sound = measure(mp4)?;
    Ok(Upload {
        aspect,
        size: [
            video["width"].as_u64().unwrap_or(0) as u32,
            video["height"].as_u64().unwrap_or(0) as u32,
        ],
        duration: info["format"]["duration"]
            .as_str()
            .and_then(|d| d.parse().ok())
            .unwrap_or(0.0),
        bytes: fs::metadata(mp4).map_or(0, |m| m.len()),
        video: [
            text(&video["codec_name"]),
            text(&video["profile"]),
            text(&video["pix_fmt"]),
        ],
        audio: text(&audio["codec_name"]),
        loudness: sound.loudness,
        true_peak: sound.true_peak,
    })
}

/// How the cues of an SRT file miss a picture of `duration` seconds: a
/// cue that ends before it starts, starts before the one before it ends,
/// or runs past the end of the picture.
pub fn captions(srt: &str, duration: f64) -> Vec<String> {
    let seconds = |clock: &str| -> Option<f64> {
        let (hms, ms) = clock.trim().split_once(',')?;
        let mut parts = hms.split(':').map(|p| p.parse::<f64>().ok());
        let (h, m, s) = (parts.next()??, parts.next()??, parts.next()??);
        Some(h * 3600.0 + m * 60.0 + s + ms.parse::<f64>().ok()? / 1000.0)
    };
    let mut problems = Vec::new();
    let mut before = 0.0;
    for (n, line) in srt.lines().filter(|l| l.contains("-->")).enumerate() {
        let n = n + 1;
        let times = line
            .split_once("-->")
            .and_then(|(a, b)| Some((seconds(a)?, seconds(b)?)));
        let Some((start, end)) = times else {
            problems.push(format!("caption {n} has no times: {line:?}"));
            continue;
        };
        if end <= start {
            problems.push(format!("caption {n} ends before it starts"));
        }
        if start < before {
            problems.push(format!("caption {n} starts before caption {} ends", n - 1));
        }
        if end > duration + 0.05 {
            problems.push(format!(
                "caption {n} ends at {end:.2} s, after the picture does at {duration:.2} s"
            ));
        }
        before = end;
    }
    problems
}

/// The longest stretch of digital silence a cut may have, in seconds:
/// longer, and a listener hears the sound drop out between lines.
pub const SILENCE: f64 = 0.05;

/// What a file's audio measures.
#[derive(Debug, Clone, PartialEq)]
pub struct Sound {
    /// Integrated loudness, in LUFS (EBU R 128).
    pub loudness: Option<f64>,
    /// The highest true peak of any channel, in dBTP.
    pub true_peak: Option<f64>,
    /// Each stretch of digital silence longer than [`SILENCE`]: its start
    /// and its length, in seconds.
    pub silences: Vec<(f64, f64)>,
}

/// Measures a file's audio in one pass: its loudness, its true peak, and
/// its stretches of digital silence, which is every sample under one
/// step of 16 bits.
pub fn measure(path: &Path) -> Result<Sound, String> {
    let filters = format!("silencedetect=noise=0.00002:d={SILENCE},ebur128=peak=true");
    let out = Command::new("ffmpeg")
        .args(["-nostdin", "-hide_banner", "-nostats", "-i"])
        .arg(path)
        .args(["-map", "0:a", "-af", &filters, "-f", "null", "-"])
        .output()
        .map_err(|e| format!("ffmpeg: {e}"))?;
    let log = String::from_utf8_lossy(&out.stderr);
    let after = |line: &str, key: &str| -> Option<f64> {
        let rest = &line[line.find(key)? + key.len()..];
        rest.split_whitespace().next()?.parse().ok()
    };
    let loudness = log
        .lines()
        .filter_map(|l| l.trim().strip_prefix("I:"))
        .filter_map(|rest| rest.split_whitespace().next()?.parse().ok())
        .next_back();
    let true_peak = log
        .lines()
        .filter_map(|l| l.trim().strip_prefix("Peak:"))
        .filter_map(|rest| rest.split_whitespace().next()?.parse().ok())
        .next_back();
    // A stretch that runs to the end of the file is reported when it ends, too.
    let silences = log
        .lines()
        .filter(|l| l.contains("silence_end:"))
        .filter_map(|l| {
            let (end, lasts) = (after(l, "silence_end:")?, after(l, "silence_duration:")?);
            Some((end - lasts, lasts))
        })
        .collect();
    Ok(Sound {
        loudness,
        true_peak,
        silences,
    })
}

/// The integrated loudness of a file's audio, in LUFS (EBU R 128).
pub fn loudness(path: &Path) -> Result<Option<f64>, String> {
    measure(path).map(|sound| sound.loudness)
}
