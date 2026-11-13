//! T2 for the encoder (TODO.md, P3-VIDNI-01): encode the golden frames and
//! narration twice, and compare the two files' packets and bytes.
//!
//! cargo run --release -p vidni-media --example t2_golden -- <golden cut.mp4> <narration.wav> <scratch dir>
//!
//! The frames are the golden cut decoded to rgb24 (H.264 decoding is bit
//! exact), and the narration is the golden narration PCM, loudness
//! normalized as the reference does.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use vidni_media::encode::{Encoder, Settings, Target, loudnorm};
use vidni_media::framemd5;

fn size(cut: &Path) -> Result<(u32, u32), String> {
    let out = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height",
            "-of",
            "csv=p=0",
        ])
        .arg(cut)
        .output()
        .map_err(|e| e.to_string())?;
    let text = String::from_utf8_lossy(&out.stdout);
    let mut parts = text.trim().split(',').map(|n| n.parse::<u32>());
    match (parts.next(), parts.next()) {
        (Some(Ok(w)), Some(Ok(h))) => Ok((w, h)),
        _ => Err(format!("{}: no video size", cut.display())),
    }
}

/// Decodes `cut` and encodes its frames with `audio` into `out`; returns the frame count.
fn encode(cut: &Path, audio: &Path, out: &Path, (w, h): (u32, u32)) -> Result<u64, String> {
    let mut decoder = Command::new("ffmpeg")
        .args(["-nostdin", "-loglevel", "error", "-i"])
        .arg(cut)
        .args(["-map", "0:v:0", "-f", "rawvideo", "-pix_fmt", "rgb24", "-"])
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let mut frames = decoder.stdout.take().ok_or("no decoder output")?;
    let mut encoder = Encoder::start(&Settings::new(w, h), audio, out)?;
    let mut frame = vec![0u8; (w * h * 3) as usize];
    loop {
        match frames.read_exact(&mut frame) {
            Ok(()) => encoder.push(&frame)?,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.to_string()),
        }
    }
    decoder.wait().map_err(|e| e.to_string())?;
    encoder.finish()
}

fn run() -> Result<bool, String> {
    let args: Vec<PathBuf> = std::env::args().skip(1).map(PathBuf::from).collect();
    let [cut, narration, scratch] = args.as_slice() else {
        return Err("usage: t2_golden <golden cut.mp4> <narration.wav> <scratch dir>".into());
    };
    std::fs::create_dir_all(scratch).map_err(|e| e.to_string())?;
    let audio = scratch.join("narration-norm.wav");
    loudnorm(narration, &audio, Target::default())?;
    let dims = size(cut)?;
    let mut logs = Vec::new();
    for run in ["a", "b"] {
        let out = scratch.join(format!("{run}.mp4"));
        let started = std::time::Instant::now();
        let frames = encode(cut, &audio, &out, dims)?;
        let log = framemd5(&out)?;
        let bytes = std::fs::read(&out).map_err(|e| e.to_string())?;
        println!(
            "run {run}: {frames} frames at {}x{} in {:.0} s; {} packet-log lines; {} bytes",
            dims.0,
            dims.1,
            started.elapsed().as_secs_f64(),
            log.split(|b| *b == b'\n')
                .filter(|l| !l.is_empty() && l[0] != b'#')
                .count(),
            bytes.len()
        );
        logs.push((log, bytes));
    }
    let same_packets = logs[0].0 == logs[1].0;
    let same_bytes = logs[0].1 == logs[1].1;
    println!("same framemd5: {same_packets}; same file bytes: {same_bytes}");
    Ok(same_packets && same_bytes)
}

fn main() -> ExitCode {
    match run() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("t2_golden: {e}");
            ExitCode::from(2)
        }
    }
}
