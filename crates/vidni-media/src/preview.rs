//! The preview's decoder (TODO.md, P7-VIDNI-03; DECISIONS.md, D9): frames
//! of a built cut, read through the same pinned ffmpeg that built it.
//!
//! A frame is asked for by its number. ffmpeg seeks to the keyframe
//! before it and decodes up to it, so the frame that comes back is that
//! very frame, not the nearest one that is cheap to reach.

use crate::compose::image::Image;
use crate::run;
use serde_json::Value;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

/// A built cut's picture, as ffprobe sees it.
#[derive(Debug, Clone, PartialEq)]
pub struct Video {
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
    /// Frames a second.
    pub fps: f64,
    /// How many frames it has.
    pub frames: u64,
}

impl Video {
    /// Reads what `mp4` is.
    pub fn open(mp4: &Path) -> Result<Self, String> {
        let probe = run(Command::new("ffprobe")
            .args(["-v", "error", "-select_streams", "v:0", "-count_packets"])
            .args([
                "-show_entries",
                "stream=width,height,r_frame_rate,nb_read_packets",
            ])
            .args(["-of", "json"])
            .arg(mp4))?;
        let info: Value = serde_json::from_slice(&probe.stdout).map_err(|e| e.to_string())?;
        let stream = &info["streams"][0];
        let number = |key: &str| stream[key].as_u64();
        let rate = stream["r_frame_rate"].as_str().and_then(|rate| {
            let (n, d) = rate.split_once('/')?;
            let (n, d): (f64, f64) = (n.parse().ok()?, d.parse().ok()?);
            (d > 0.0 && n > 0.0).then_some(n / d)
        });
        let frames = stream["nb_read_packets"]
            .as_str()
            .and_then(|n| n.parse::<u64>().ok());
        match (number("width"), number("height"), rate, frames) {
            (Some(width), Some(height), Some(fps), Some(frames)) if frames > 0 => Ok(Self {
                path: mp4.to_path_buf(),
                width: width as u32,
                height: height as u32,
                fps,
                frames,
            }),
            _ => Err(format!(
                "{}: there is no picture in it to show",
                mp4.display()
            )),
        }
    }

    /// How long it lasts, in seconds.
    pub fn seconds(&self) -> f64 {
        self.frames as f64 / self.fps
    }

    /// The frame that shows at `seconds`: the last one that has begun.
    pub fn frame_at(&self, seconds: f64) -> u64 {
        // A hair is added, so that a time that is a frame's own start,
        // less the last digit of a float, is that frame.
        let frame = (seconds.max(0.0) * self.fps + 1e-6).floor() as u64;
        frame.min(self.frames.saturating_sub(1))
    }

    /// When frame `index` begins, in seconds.
    pub fn time_of(&self, index: u64) -> f64 {
        index as f64 / self.fps
    }

    /// The size a frame is decoded at when it is asked for `width` wide:
    /// no wider than the picture, and even in both directions.
    pub fn size(&self, width: u32) -> (u32, u32) {
        let w = width.clamp(2, self.width.max(2)) / 2 * 2;
        let h = (f64::from(w) * f64::from(self.height) / f64::from(self.width) / 2.0).round()
            as u32
            * 2;
        (w, h.max(2))
    }

    /// The arguments that make ffmpeg write frames from `index` on, raw,
    /// `width` wide, to its stdout.
    fn args(&self, index: u64, width: u32) -> Vec<String> {
        let (w, h) = self.size(width);
        // Half a frame before the frame's start: the first frame at or
        // after that moment is the frame itself, whatever the rounding.
        let from = ((index as f64 - 0.5) / self.fps).max(0.0);
        let mut args: Vec<String> = ["-nostdin", "-hide_banner", "-loglevel", "error"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        args.extend(["-ss".into(), format!("{from:.6}"), "-i".into()]);
        args.push(self.path.display().to_string());
        args.extend(
            ["-an", "-sn", "-vf"]
                .iter()
                .map(|s| s.to_string())
                .chain([format!("scale={w}:{h}:flags=bilinear")])
                .chain(
                    // Each frame once, as it is in the file: none is
                    // repeated or dropped to even out the timing.
                    [
                        "-fps_mode",
                        "passthrough",
                        "-f",
                        "rawvideo",
                        "-pix_fmt",
                        "rgb24",
                    ]
                    .iter()
                    .map(|s| s.to_string()),
                ),
        );
        args
    }

    /// Frame `index`, `width` pixels wide.
    pub fn frame(&self, index: u64, width: u32) -> Result<Image, String> {
        let (w, h) = self.size(width);
        let index = index.min(self.frames.saturating_sub(1));
        let mut command = Command::new("ffmpeg");
        command
            .args(self.args(index, width))
            .args(["-frames:v", "1", "-"]);
        let out = run(&mut command)?;
        let bytes = (w * h * 3) as usize;
        if out.stdout.len() != bytes {
            return Err(format!(
                "{}: ffmpeg gave {} bytes for frame {index}, not the {bytes} of a {w}x{h} picture",
                self.path.display(),
                out.stdout.len()
            ));
        }
        Ok(Image {
            width: w,
            height: h,
            channels: 3,
            data: out.stdout,
        })
    }

    /// The frames from `index` on, one after another, for playing.
    pub fn play(&self, index: u64, width: u32) -> Result<Frames, String> {
        let (w, h) = self.size(width);
        let child = Command::new("ffmpeg")
            .args(self.args(index, width))
            .arg("-")
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("ffmpeg: {e}"))?;
        Ok(Frames {
            child,
            size: (w, h),
            next: index,
        })
    }
}

/// A cut being decoded from some frame on.
pub struct Frames {
    child: Child,
    size: (u32, u32),
    next: u64,
}

impl Iterator for Frames {
    type Item = (u64, Image);

    /// The next frame, with its number; none when the cut has ended.
    fn next(&mut self) -> Option<(u64, Image)> {
        let (w, h) = self.size;
        let mut data = vec![0u8; (w * h * 3) as usize];
        self.child.stdout.as_mut()?.read_exact(&mut data).ok()?;
        let index = self.next;
        self.next += 1;
        Some((
            index,
            Image {
                width: w,
                height: h,
                channels: 3,
                data,
            },
        ))
    }
}

impl Drop for Frames {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
