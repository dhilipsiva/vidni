//! Encoding: rgb24 frames and a narration track in, one MP4 out.
//!
//! Every flag that could vary is fixed, so that the same frames and samples
//! give the same packets on one host (TODO.md, rule 2, tier T2):
//! - libx264 with a fixed thread count, preset and CRF, and its lookahead
//!   run in the encoder's own thread: in a thread of its own, the packets
//!   depended on how the threads happened to be scheduled (DECISIONS.md, T8);
//! - `+bitexact` for the container and both codecs, and `-map_metadata -1`,
//!   so no version string, date or encoder tag enters the file;
//! - an explicit BT.709 conversion from RGB, with the colour tags to match;
//! - `-cpuflags 0`, so filters don't choose SIMD paths by CPU;
//! - loudness normalized in two passes before encoding ([`loudnorm`]).

use crate::run;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};

/// libx264's own settings, beyond the preset. `sync-lookahead=0` runs the
/// lookahead in the encoder's thread. With the threaded lookahead that x264
/// turns on when it has several threads, 40 encodes of one stretch of a cut
/// gave 8 different packet streams; without it, 40 gave one, on 1 to 32
/// cores, and took no longer.
pub const X264_PARAMS: &str = "sync-lookahead=0";

/// What the encoder makes.
#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub crf: u32,
    pub preset: &'static str,
    /// libx264's threads. The packets depend on it, so it never follows the host's core count.
    pub threads: u32,
    pub audio_kbps: u32,
}

impl Settings {
    /// 30 fps H.264 at CRF 18 with AAC at 160 kb/s, as the reference encodes.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            fps: 30,
            crf: 18,
            preset: "slow",
            threads: 4,
            audio_kbps: 160,
        }
    }

    /// The ffmpeg arguments, from the first input to the output.
    pub fn args(&self, audio: &Path, out: &Path) -> Vec<String> {
        let mut args: Vec<String> = [
            "-nostdin",
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-cpuflags",
            "0",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgb24",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        args.extend([
            "-s".into(),
            format!("{}x{}", self.width, self.height),
            "-r".into(),
            self.fps.to_string(),
            "-i".into(),
            "-".into(),
            "-i".into(),
            audio.display().to_string(),
        ]);
        args.extend(
            [
                "-map", "0:v:0", "-map", "1:a:0",
                "-vf", "scale=out_color_matrix=bt709:out_range=tv:flags=bitexact+accurate_rnd+full_chroma_int,format=yuv420p",
                "-color_primaries", "bt709", "-color_trc", "bt709", "-colorspace", "bt709", "-color_range", "tv",
                "-c:v", "libx264", "-preset", self.preset,
                "-x264-params", X264_PARAMS,
            ]
            .iter()
            .map(|s| s.to_string()),
        );
        args.extend([
            "-crf".into(),
            self.crf.to_string(),
            "-threads".into(),
            self.threads.to_string(),
            "-c:a".into(),
            "aac".into(),
            "-b:a".into(),
            format!("{}k", self.audio_kbps),
        ]);
        args.extend(
            [
                "-map_metadata",
                "-1",
                "-map_chapters",
                "-1",
                "-fflags",
                "+bitexact",
                "-flags:v",
                "+bitexact",
                "-flags:a",
                "+bitexact",
                "-movflags",
                "+faststart",
                "-shortest",
            ]
            .iter()
            .map(|s| s.to_string()),
        );
        args.push(out.display().to_string());
        args
    }
}

/// A running encode: push frames, then finish.
pub struct Encoder {
    child: Child,
    stdin: Option<ChildStdin>,
    frame_bytes: usize,
    frames: u64,
    out: PathBuf,
}

impl Encoder {
    /// Starts ffmpeg writing `out`, with `audio` as the sound.
    pub fn start(settings: &Settings, audio: &Path, out: &Path) -> Result<Self, String> {
        let mut child = Command::new("ffmpeg")
            .args(settings.args(audio, out))
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("ffmpeg: {e}"))?;
        let stdin = child.stdin.take();
        Ok(Self {
            child,
            stdin,
            frame_bytes: (settings.width * settings.height * 3) as usize,
            frames: 0,
            out: out.to_path_buf(),
        })
    }

    /// Adds one rgb24 frame.
    pub fn push(&mut self, frame: &[u8]) -> Result<(), String> {
        if frame.len() != self.frame_bytes {
            return Err(format!(
                "a frame of {} bytes; this encode takes {}",
                frame.len(),
                self.frame_bytes
            ));
        }
        let stdin = self.stdin.as_mut().ok_or("the encode has finished")?;
        stdin
            .write_all(frame)
            .map_err(|e| format!("{}: ffmpeg stopped reading frames: {e}", self.out.display()))?;
        self.frames += 1;
        Ok(())
    }

    /// Closes the stream and waits for ffmpeg; returns the frame count.
    pub fn finish(mut self) -> Result<u64, String> {
        drop(self.stdin.take());
        let out = self.child.wait_with_output().map_err(|e| e.to_string())?;
        if !out.status.success() {
            return Err(format!(
                "{}: ffmpeg failed: {}",
                self.out.display(),
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
        Ok(self.frames)
    }
}

/// A loudness target for [`loudnorm`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Target {
    /// Integrated loudness, in LUFS.
    pub integrated: f64,
    /// The highest true peak, in dBTP.
    pub true_peak: f64,
    /// Loudness range, in LU.
    pub range: f64,
}

impl Default for Target {
    /// −16 LUFS, −1.5 dBTP, 11 LU: the reference's target.
    fn default() -> Self {
        Self {
            integrated: -16.0,
            true_peak: -1.5,
            range: 11.0,
        }
    }
}

/// Normalizes `src` to `target` in two passes, as the reference does (one
/// pass undershoots on speech with pauses), into a 48 kHz stereo WAV.
pub fn loudnorm(src: &Path, dst: &Path, target: Target) -> Result<(), String> {
    let base = format!(
        "loudnorm=I={}:TP={}:LRA={}",
        target.integrated, target.true_peak, target.range
    );
    let measured = run(Command::new("ffmpeg")
        .args([
            "-nostdin",
            "-hide_banner",
            "-nostats",
            "-cpuflags",
            "0",
            "-i",
        ])
        .arg(src)
        .args([
            "-af",
            &format!("{base}:print_format=json"),
            "-f",
            "null",
            "-",
        ]))?;
    let log = String::from_utf8_lossy(&measured.stderr);
    let json = log
        .rfind('{')
        .and_then(|start| log[start..].find('}').map(|end| &log[start..=start + end]))
        .ok_or("loudnorm printed no measurement")?;
    let m: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let field = |k: &str| {
        m[k].as_str()
            .map(str::to_string)
            .ok_or(format!("loudnorm gave no {k}"))
    };
    // Silence has no loudness to bring to a target, and ffmpeg refuses the
    // measurement it gives for it.
    if field("input_i")?.contains("inf") {
        return Err(format!(
            "{}: there is no sound in it to normalize",
            src.display()
        ));
    }
    let filter = format!(
        "{base}:linear=true:measured_I={}:measured_TP={}:measured_LRA={}:measured_thresh={}:offset={},aresample=48000",
        field("input_i")?,
        field("input_tp")?,
        field("input_lra")?,
        field("input_thresh")?,
        field("target_offset")?
    );
    run(Command::new("ffmpeg")
        .args([
            "-nostdin",
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-cpuflags",
            "0",
            "-i",
        ])
        .arg(src)
        .args([
            "-af",
            &filter,
            "-ac",
            "2",
            "-fflags",
            "+bitexact",
            "-flags:a",
            "+bitexact",
        ])
        .arg(dst))
    .map(|_| ())
}
