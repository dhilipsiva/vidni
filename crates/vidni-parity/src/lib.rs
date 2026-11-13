//! Parity checks between a candidate (a Rust stage's outputs) and the golden
//! outputs of the reference pipeline (TODO.md, P1-VIDNI-04).
//!
//! Logic outputs must match exactly: the normalized timelines, the captions,
//! the narration PCM and the MP4 packets (T1 and T2). Rendered frames are
//! compared by SSIM (at least [`SSIM_MIN`]) while the port is in progress,
//! against golden media whose hashes are checked first.
//!
//! Packets are compared by [`packet_fingerprint`], which doesn't depend on
//! the ffmpeg version that reads them: the reference's golden logs were
//! written by ffmpeg 8.1, and vidni pins ffmpeg 9.0.
//!
//! A candidate folder holds any of:
//! `timeline-{synthetic,voice}.json`, `nibli-syllogism{,-voice}.en.srt`,
//! `narration-{synthetic,voice}.wav`, `nibli-syllogism{,-voice}-{16x9,9x16}.mp4`,
//! `stills/*.png` and `cards/*.png`. A missing file fails its check.

use flate2::read::GzDecoder;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

pub mod candidate;
pub mod selftest;

/// The lowest SSIM a rendered frame may reach against its golden frame.
pub const SSIM_MIN: f64 = 0.995;
/// How many evenly spaced frames are compared per cut.
pub const SAMPLES_PER_CUT: usize = 20;
/// The variants and the file-name suffix each one uses.
pub const VARIANTS: [(&str, &str); 2] = [("synthetic", ""), ("voice", "-voice")];
/// The two cuts.
pub const CUTS: [&str; 2] = ["16x9", "9x16"];

/// Every check made and every failure found.
#[derive(Debug, Default)]
pub struct Report {
    pub checks: usize,
    pub failures: Vec<String>,
}

impl Report {
    fn pass(&mut self) {
        self.checks += 1;
    }

    fn fail(&mut self, message: String) {
        self.checks += 1;
        self.failures.push(message);
    }

    fn check(&mut self, ok: bool, message: impl FnOnce() -> String) {
        if ok {
            self.pass()
        } else {
            self.fail(message())
        }
    }
}

/// Lowercase hex of SHA-256.
pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn read(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|e| format!("{}: {e}", path.display()))
}

/// Hash of a WAV file's decoded samples, the same scheme as `tools/golden.py`:
/// `"{channels}ch {bits}bit {rate}Hz|"` followed by the interleaved little-endian samples.
pub fn pcm_sha256(path: &Path) -> Result<String, String> {
    let mut reader =
        hound::WavReader::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let spec = reader.spec();
    if spec.bits_per_sample != 16 || spec.sample_format != hound::SampleFormat::Int {
        return Err(format!("{}: expected 16-bit PCM", path.display()));
    }
    let mut data = format!("{}ch 16bit {}Hz|", spec.channels, spec.sample_rate).into_bytes();
    for sample in reader.samples::<i16>() {
        data.extend_from_slice(
            &sample
                .map_err(|e| format!("{}: {e}", path.display()))?
                .to_le_bytes(),
        );
    }
    Ok(sha256_hex(&data))
}

/// Normalizes a timeline as `tools/golden.py` does: whisper's `heard` and
/// `score` are dropped, and each clip path, absolute or relative to `root`,
/// is rewritten so that it doesn't depend on where the takes were laid out.
/// A clip in the reference's `build/` (a synthetic take) is made relative to
/// `root`; any other clip is one of the creator's takes, `recordings/<file>`.
pub fn normalize_timeline(timeline: &mut Value, root: &Path) {
    let root = lexical(root);
    if let Some(sentences) = timeline.get_mut("sentences").and_then(Value::as_array_mut) {
        for sentence in sentences {
            if let Some(map) = sentence.as_object_mut() {
                map.remove("heard");
                map.remove("score");
                if let Some(Value::String(wav)) = map.get_mut("wav") {
                    let clip = lexical(&root.join(wav.as_str()));
                    match clip.strip_prefix(&root) {
                        Ok(rel) if rel.starts_with("build") => {
                            *wav = rel.to_string_lossy().into_owned();
                        }
                        _ => {
                            let name = clip.file_name().unwrap_or_default().to_string_lossy();
                            *wav = format!("recordings/{name}");
                        }
                    }
                }
            }
        }
    }
}

/// `path` with `.` and `..` resolved by its components alone, as Python's
/// `os.path.normpath` does; the file system isn't consulted.
fn lexical(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir
                if matches!(out.components().next_back(), Some(Component::Normal(_))) =>
            {
                out.pop();
            }
            other => out.push(other),
        }
    }
    out
}

/// The path of the first difference between two JSON values, if any.
pub fn json_diff(a: &Value, b: &Value, path: &str) -> Option<String> {
    match (a, b) {
        (Value::Object(x), Value::Object(y)) => {
            let mut keys: Vec<&String> = x.keys().chain(y.keys()).collect();
            keys.sort();
            keys.dedup();
            keys.into_iter().find_map(|k| match (x.get(k), y.get(k)) {
                (Some(u), Some(v)) => json_diff(u, v, &format!("{path}/{k}")),
                _ => Some(format!("{path}/{k}: present on one side only")),
            })
        }
        (Value::Array(x), Value::Array(y)) => {
            if x.len() != y.len() {
                return Some(format!("{path}: {} items, golden has {}", y.len(), x.len()));
            }
            x.iter()
                .zip(y)
                .enumerate()
                .find_map(|(i, (u, v))| json_diff(u, v, &format!("{path}[{i}]")))
        }
        _ if a == b => None,
        _ => Some(format!("{path}: golden {a}, candidate {b}")),
    }
}

fn run(cmd: &mut Command) -> Result<std::process::Output, String> {
    let shown = format!("{cmd:?}");
    let out = cmd.output().map_err(|e| format!("{shown}: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "{shown}: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(out)
}

/// `ffmpeg -map 0 -c copy -f framemd5` of a media file: one line per packet.
pub fn framemd5(path: &Path) -> Result<Vec<u8>, String> {
    let out = run(Command::new("ffmpeg")
        .args(["-loglevel", "error", "-i"])
        .arg(path)
        .args(["-map", "0", "-c", "copy", "-f", "framemd5", "-"]))?;
    Ok(out.stdout)
}

/// A framemd5 log without what depends on the ffmpeg that read the file: the
/// `#software` line, and the side data listed after a packet's hash (ffmpeg 9
/// lists the last AAC packet's skip-samples side data, ffmpeg 8 doesn't). The
/// extradata, the stream parameters and every packet's timing, size and
/// payload MD5 stay, so the fingerprint changes only when the streams do.
pub fn packet_fingerprint(framemd5: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(framemd5)
        .lines()
        .filter(|l| !l.starts_with("#software"))
        .map(|l| match l.split_once(", S=") {
            Some((packet, _)) if !l.starts_with('#') => packet.to_string(),
            _ => l.to_string(),
        })
        .collect()
}

/// The golden framemd5 log of a cut, from `golden/framemd5/`, checked
/// against its hash in `hashes.json`.
pub fn golden_framemd5(golden: &Path, name: &str, hashes: &Value) -> Result<Vec<u8>, String> {
    let path = golden.join("framemd5").join(format!("{name}.framemd5.gz"));
    let mut log = Vec::new();
    GzDecoder::new(read(&path)?.as_slice())
        .read_to_end(&mut log)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let want = golden_hash(hashes, "outputs", &format!("{name} (framemd5)"))
        .ok_or_else(|| format!("{name}: no golden hash"))?;
    if sha256_hex(&log) != want {
        return Err(format!("{}: doesn't match hashes.json", path.display()));
    }
    Ok(log)
}

/// Compares two fingerprints; on a difference, names the first differing line.
fn same_packets(golden: &[String], ours: &[String]) -> Result<(), String> {
    match golden.iter().zip(ours).position(|(a, b)| a != b) {
        Some(i) => Err(format!("line {} of the packet log differs", i + 1)),
        None if golden.len() != ours.len() => Err(format!(
            "{} packet-log lines, golden has {}",
            ours.len(),
            golden.len()
        )),
        None => Ok(()),
    }
}

/// A media file's duration in seconds, from ffprobe.
pub fn duration(path: &Path) -> Result<f64, String> {
    let out = run(Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "csv=p=0",
        ])
        .arg(path))?;
    String::from_utf8_lossy(&out.stdout)
        .trim()
        .parse()
        .map_err(|e| format!("{}: duration: {e}", path.display()))
}

/// Writes the frame at `seconds` as a PNG.
pub fn frame_at(video: &Path, seconds: f64, png: &Path) -> Result<(), String> {
    run(Command::new("ffmpeg")
        .args([
            "-loglevel",
            "error",
            "-y",
            "-ss",
            &format!("{seconds:.3}"),
            "-i",
        ])
        .arg(video)
        .args(["-frames:v", "1"])
        .arg(png))
    .map(|_| ())
}

/// SSIM of two same-sized images, from ffmpeg's `ssim` filter (1.0 is identical).
pub fn ssim(a: &Path, b: &Path) -> Result<f64, String> {
    let out = run(Command::new("ffmpeg")
        .args(["-hide_banner", "-nostats", "-i"])
        .arg(a)
        .arg("-i")
        .arg(b)
        .args(["-lavfi", "ssim", "-f", "null", "-"]))?;
    let log = String::from_utf8_lossy(&out.stderr);
    log.lines()
        .filter_map(|l| l.split_once(" All:"))
        .filter_map(|(_, rest)| rest.split_whitespace().next()?.parse().ok())
        .next_back()
        .ok_or_else(|| format!("no SSIM in ffmpeg's output for {}", a.display()))
}

/// Hash of an image's decoded RGBA pixels, the same scheme as `tools/golden.py`:
/// `"{width}x{height}|"` followed by the pixels.
pub fn pixels_sha256(path: &Path) -> Result<String, String> {
    let probe = run(Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height",
        ])
        .args(["-of", "csv=p=0:s=x"])
        .arg(path))?;
    let size = String::from_utf8_lossy(&probe.stdout).trim().to_string();
    let raw = run(Command::new("ffmpeg")
        .args(["-loglevel", "error", "-i"])
        .arg(path)
        .args(["-f", "rawvideo", "-pix_fmt", "rgba", "-"]))?;
    let mut data = format!("{size}|").into_bytes();
    data.extend_from_slice(&raw.stdout);
    Ok(sha256_hex(&data))
}

fn golden_hash<'a>(hashes: &'a Value, section: &str, key: &str) -> Option<&'a str> {
    let v = hashes.get(section)?.get(key)?;
    v.as_str()
        .or_else(|| v.get("decoded").and_then(Value::as_str))
}

/// Reads a JSON file.
pub fn load_json(path: &Path) -> Result<Value, String> {
    serde_json::from_slice(&read(path)?).map_err(|e| format!("{}: {e}", path.display()))
}

/// Timelines and captions: exact equality after normalization.
pub fn check_timeline(candidate: &Path, golden: &Path, root: &Path, report: &mut Report) {
    for (variant, suffix) in VARIANTS {
        let name = format!("timeline-{variant}.json");
        match (
            load_json(&golden.join(&name)),
            load_json(&candidate.join(&name)),
        ) {
            (Ok(g), Ok(mut c)) => {
                normalize_timeline(&mut c, root);
                match json_diff(&g, &c, "") {
                    None => report.pass(),
                    Some(d) => report.fail(format!("{name}: {d}")),
                }
            }
            (Err(e), _) | (_, Err(e)) => report.fail(e),
        }
        let srt = format!("nibli-syllogism{suffix}.en.srt");
        match (read(&golden.join(&srt)), read(&candidate.join(&srt))) {
            (Ok(g), Ok(c)) => report.check(g == c, || format!("{srt}: differs from golden")),
            (Err(e), _) | (_, Err(e)) => report.fail(e),
        }
    }
}

/// Narration: the decoded PCM must hash as golden does.
pub fn check_narration(candidate: &Path, hashes: &Value, report: &mut Report) {
    for (variant, _) in VARIANTS {
        let name = format!("narration-{variant}.wav");
        let want = golden_hash(hashes, "outputs", &format!("{name} (pcm)"));
        match (want, pcm_sha256(&candidate.join(&name))) {
            (Some(w), Ok(got)) => {
                report.check(w == got, || format!("{name}: PCM differs from golden"))
            }
            (None, _) => report.fail(format!("{name}: no golden hash")),
            (_, Err(e)) => report.fail(e),
        }
    }
}

/// Encoding: every cut's packets must equal golden's (T2), by [`packet_fingerprint`].
pub fn check_encode(candidate: &Path, golden: &Path, hashes: &Value, report: &mut Report) {
    for (_, suffix) in VARIANTS {
        for cut in CUTS {
            let name = format!("nibli-syllogism{suffix}-{cut}.mp4");
            let compared = golden_framemd5(golden, &name, hashes).and_then(|g| {
                let ours = framemd5(&candidate.join(&name))?;
                Ok(same_packets(
                    &packet_fingerprint(&g),
                    &packet_fingerprint(&ours),
                ))
            });
            match compared {
                Ok(Ok(())) => report.pass(),
                Ok(Err(d)) => report.fail(format!("{name}: packets differ from golden: {d}")),
                Err(e) => report.fail(e),
            }
        }
    }
}

fn verified_golden_cut(
    reference_out: &Path,
    golden: &Path,
    name: &str,
    hashes: &Value,
) -> Result<PathBuf, String> {
    let path = reference_out.join(name);
    let want = packet_fingerprint(&golden_framemd5(golden, name, hashes)?);
    if same_packets(&want, &packet_fingerprint(&framemd5(&path)?)).is_err() {
        return Err(format!(
            "{}: is not the golden media; run `just golden`",
            path.display()
        ));
    }
    Ok(path)
}

/// Cuts: SSIM of evenly spaced frames against the verified golden media.
pub fn check_cuts(
    candidate: &Path,
    reference_out: &Path,
    golden_dir: &Path,
    hashes: &Value,
    scratch: &Path,
    report: &mut Report,
) {
    for (_, suffix) in VARIANTS {
        for cut in CUTS {
            let name = format!("nibli-syllogism{suffix}-{cut}.mp4");
            let golden = match verified_golden_cut(reference_out, golden_dir, &name, hashes) {
                Ok(p) => p,
                Err(e) => {
                    report.fail(e);
                    continue;
                }
            };
            let ours = candidate.join(&name);
            let span = match (duration(&golden), duration(&ours)) {
                (Ok(a), Ok(b)) => a.min(b),
                (Err(e), _) | (_, Err(e)) => {
                    report.fail(e);
                    continue;
                }
            };
            for i in 0..SAMPLES_PER_CUT {
                let t = span * (i as f64 + 0.5) / SAMPLES_PER_CUT as f64;
                let (a, b) = (scratch.join("golden.png"), scratch.join("candidate.png"));
                let score = frame_at(&golden, t, &a)
                    .and_then(|_| frame_at(&ours, t, &b))
                    .and_then(|_| ssim(&a, &b));
                match score {
                    Ok(s) => report.check(s >= SSIM_MIN, || {
                        format!("{name} at {t:.2} s: SSIM {s:.4} < {SSIM_MIN}")
                    }),
                    Err(e) => report.fail(e),
                }
            }
        }
    }
}

/// Stills and cards: SSIM against the reference's frozen captures, verified by pixel hash.
pub fn check_stills(candidate: &Path, reference_build: &Path, hashes: &Value, report: &mut Report) {
    let Some(inputs) = hashes.get("inputs").and_then(Value::as_object) else {
        report.fail("golden hashes have no inputs".into());
        return;
    };
    for (key, entry) in inputs {
        let Some(rel) = key.strip_prefix("build/").filter(|r| r.ends_with(".png")) else {
            continue;
        };
        let golden = reference_build.join(rel);
        let want = entry
            .get("decoded")
            .and_then(Value::as_str)
            .unwrap_or_default();
        match pixels_sha256(&golden) {
            Ok(h) if h == want => {}
            Ok(_) => {
                report.fail(format!("{}: is not the golden capture", golden.display()));
                continue;
            }
            Err(e) => {
                report.fail(e);
                continue;
            }
        }
        match ssim(&golden, &candidate.join(rel)) {
            Ok(s) => report.check(s >= SSIM_MIN, || format!("{rel}: SSIM {s:.4} < {SSIM_MIN}")),
            Err(e) => report.fail(e),
        }
    }
}

/// The stages, in the order `all` runs them.
pub const STAGES: [&str; 5] = ["timeline", "narration", "encode", "cuts", "stills"];

/// Where the golden outputs and the reference pipeline are.
pub struct Sources {
    pub golden: PathBuf,
    pub reference: PathBuf,
}

/// Runs one stage's checks, or every stage's for `all`, on a candidate folder.
pub fn check_stage(
    stage: &str,
    candidate: &Path,
    sources: &Sources,
    scratch: &Path,
) -> Result<Report, String> {
    if stage != "all" && !STAGES.contains(&stage) {
        return Err(format!("unknown stage {stage}"));
    }
    let hashes = load_json(&sources.golden.join("hashes.json"))?;
    let on = |s: &str| stage == "all" || stage == s;
    let mut report = Report::default();
    if on("timeline") {
        check_timeline(candidate, &sources.golden, &sources.reference, &mut report);
    }
    if on("narration") {
        check_narration(candidate, &hashes, &mut report);
    }
    if on("encode") {
        check_encode(candidate, &sources.golden, &hashes, &mut report);
    }
    if on("cuts") {
        let out = sources.reference.join("out");
        check_cuts(
            candidate,
            &out,
            &sources.golden,
            &hashes,
            scratch,
            &mut report,
        );
    }
    if on("stills") {
        let build = sources.reference.join("build");
        check_stills(candidate, &build, &hashes, &mut report);
    }
    Ok(report)
}

/// Lays the reference's own outputs out as a candidate, so the harness can be
/// shown to pass on the reference itself.
pub fn reference_as_candidate(reference: &Path, dst: &Path) -> Result<(), String> {
    let copy = |from: PathBuf, to: PathBuf| -> Result<(), String> {
        fs::copy(&from, &to)
            .map(|_| ())
            .map_err(|e| format!("{}: {e}", from.display()))
    };
    for (variant, suffix) in VARIANTS {
        copy(
            reference.join(format!("build/timeline{suffix}.json")),
            dst.join(format!("timeline-{variant}.json")),
        )?;
        copy(
            reference.join(format!("build/narration{suffix}.wav")),
            dst.join(format!("narration-{variant}.wav")),
        )?;
        copy(
            reference.join(format!("out/nibli-syllogism{suffix}.en.srt")),
            dst.join(format!("nibli-syllogism{suffix}.en.srt")),
        )?;
        for cut in CUTS {
            let name = format!("nibli-syllogism{suffix}-{cut}.mp4");
            copy(reference.join("out").join(&name), dst.join(&name))?;
        }
    }
    for dir in ["stills", "cards"] {
        fs::create_dir_all(dst.join(dir)).map_err(|e| e.to_string())?;
        for entry in fs::read_dir(reference.join("build").join(dir)).map_err(|e| e.to_string())? {
            let path = entry.map_err(|e| e.to_string())?.path();
            if path.extension().is_some_and(|x| x == "png") {
                copy(
                    path.clone(),
                    dst.join(dir).join(path.file_name().unwrap_or_default()),
                )?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn json_diff_names_the_first_difference() {
        let a = json!({"total": 306.3, "sentences": [{"start": 0.6}, {"start": 5.1}]});
        let b = json!({"total": 306.3, "sentences": [{"start": 0.6}, {"start": 5.2}]});
        assert_eq!(json_diff(&a, &a, ""), None);
        assert_eq!(
            json_diff(&a, &b, ""),
            Some("/sentences[1]/start: golden 5.1, candidate 5.2".into())
        );
        let c = json!({"total": 306.3});
        assert!(
            json_diff(&a, &c, "")
                .unwrap()
                .contains("present on one side only")
        );
    }

    #[test]
    fn normalizing_drops_whisper_fields_and_makes_paths_relative() {
        let mut t = json!({"sentences": [
            {"wav": "/x/ref/build/audio/000.wav", "heard": "hi", "score": 0.9, "k": 0},
            {"wav": "build/audio/001.wav", "k": 1},
            {"wav": "../../tmp/tmpab12/takes/002.wav", "k": 2},
            {"wav": "/x/examples/nibli-syllogism/recordings/003.wav", "k": 3},
            {"wav": "./build/../build/audio/004.wav", "k": 4},
        ]});
        normalize_timeline(&mut t, Path::new("/x/ref"));
        assert_eq!(
            t,
            json!({"sentences": [
                {"wav": "build/audio/000.wav", "k": 0},
                {"wav": "build/audio/001.wav", "k": 1},
                {"wav": "recordings/002.wav", "k": 2},
                {"wav": "recordings/003.wav", "k": 3},
                {"wav": "build/audio/004.wav", "k": 4},
            ]})
        );
    }

    #[test]
    fn packet_fingerprints_ignore_the_reader_but_not_the_streams() {
        let v8 = b"#format: frame checksums\n#software: Lavf62.12.103\n#tb 1: 1/48000\n\
                   1,   14701568,   14701568,      160,        6, 70b4c3b27e78e2e4690c8e80b0e6e388\n";
        let v9 = b"#format: frame checksums\n#software: Lavf63.1.102\n#tb 1: 1/48000\n\
                   1,   14701568,   14701568,      160,        6, 70b4c3b27e78e2e4690c8e80b0e6e388, S=1,       10, c7b0ebf7fc3a41f7ce4472543a20acef\n";
        let changed = b"#format: frame checksums\n#software: Lavf62.12.103\n#tb 1: 1/48000\n\
                   1,   14701568,   14701568,      160,        6, 00b4c3b27e78e2e4690c8e80b0e6e388\n";
        assert_eq!(packet_fingerprint(v8), packet_fingerprint(v9));
        assert_ne!(packet_fingerprint(v8), packet_fingerprint(changed));
        assert_eq!(
            same_packets(&packet_fingerprint(v8), &packet_fingerprint(changed)),
            Err("line 3 of the packet log differs".into())
        );
    }

    #[test]
    fn lexical_paths_resolve_dots_like_normpath() {
        assert_eq!(lexical(Path::new("/a/b/../c/./d")), PathBuf::from("/a/c/d"));
        assert_eq!(lexical(Path::new("../../a")), PathBuf::from("../../a"));
        assert_eq!(lexical(Path::new("a/../../b")), PathBuf::from("../b"));
    }

    #[test]
    fn pcm_hash_matches_the_golden_scheme() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.wav");
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 22050,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut w = hound::WavWriter::create(&path, spec).unwrap();
        for s in [0i16, 1, -1, 32767] {
            w.write_sample(s).unwrap();
        }
        w.finalize().unwrap();
        let mut expected = b"1ch 16bit 22050Hz|".to_vec();
        for s in [0i16, 1, -1, 32767] {
            expected.extend_from_slice(&s.to_le_bytes());
        }
        assert_eq!(pcm_sha256(&path).unwrap(), sha256_hex(&expected));
    }

    fn testsrc(path: &Path, extra: &str) {
        let filter = format!("testsrc2=size=320x240:rate=1{extra}");
        run(Command::new("ffmpeg")
            .args([
                "-loglevel",
                "error",
                "-y",
                "-f",
                "lavfi",
                "-i",
                &filter,
                "-frames:v",
                "1",
            ])
            .arg(path))
        .unwrap();
    }

    #[test]
    fn ssim_passes_identical_frames_and_fails_a_perturbed_one() {
        // ffmpeg is in every shell and in the flake check, so this never skips.
        let dir = tempfile::tempdir().unwrap();
        let (a, b, c) = (
            dir.path().join("a.png"),
            dir.path().join("b.png"),
            dir.path().join("c.png"),
        );
        testsrc(&a, "");
        testsrc(&b, "");
        testsrc(&c, ",drawbox=x=40:y=40:w=120:h=90:color=red@1:t=fill");
        assert!(ssim(&a, &b).unwrap() >= 0.9999);
        assert!(ssim(&a, &c).unwrap() < SSIM_MIN);
        assert_eq!(pixels_sha256(&a).unwrap(), pixels_sha256(&b).unwrap());
        assert_ne!(pixels_sha256(&a).unwrap(), pixels_sha256(&c).unwrap());
    }
}
