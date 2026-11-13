//! `vidni-parity selftest`: the harness must pass the reference's own outputs
//! and catch a deliberate change to each kind of output (TODO.md, P1-VIDNI-04).
//!
//! The reference's outputs are laid out as a candidate. Every stage must pass
//! on them. Then each perturbation changes one candidate file, its stage is
//! checked, and the file is restored. The stage must fail, and every failure
//! must name the changed file.

use crate::{Sources, check_stage, reference_as_candidate, run};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;

/// One deliberate change to one candidate file, and the stage that must catch it.
struct Perturbation {
    file: String,
    stage: &'static str,
    what: &'static str,
    apply: fn(&Path, &Path) -> Result<(), String>,
}

fn perturbations(candidate: &Path) -> Result<Vec<Perturbation>, String> {
    let mut stills: Vec<String> = fs::read_dir(candidate.join("stills"))
        .map_err(|e| format!("{}: {e}", candidate.join("stills").display()))?
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter(|n| n.ends_with(".png"))
        .collect();
    stills.sort();
    let still = stills.first().ok_or("the candidate has no stills")?;
    let mut frames: Vec<String> = fs::read_dir(candidate.join("frames"))
        .map_err(|e| format!("{}: {e}", candidate.join("frames").display()))?
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter(|n| n.ends_with(".png"))
        .collect();
    frames.sort();
    let frame = frames.first().ok_or("the candidate has no cut frames")?;
    Ok(vec![
        Perturbation {
            file: "timeline-synthetic.json".into(),
            stage: "timeline",
            what: "one caption cue ends 1 ms later",
            apply: later_cue,
        },
        Perturbation {
            file: "nibli-syllogism-voice.en.srt".into(),
            stage: "timeline",
            what: "one caption word misspelled",
            apply: misspelled_caption,
        },
        Perturbation {
            file: "narration-voice.wav".into(),
            stage: "narration",
            what: "one sample off by one",
            apply: one_sample_off,
        },
        Perturbation {
            file: "nibli-syllogism-9x16.mp4".into(),
            stage: "encode",
            what: "the audio re-encoded, the pictures untouched",
            apply: reencoded_audio,
        },
        Perturbation {
            file: format!("frames/{frame}"),
            stage: "cuts",
            what: "a box drawn over a cut's frame",
            apply: boxed_still,
        },
        Perturbation {
            file: format!("stills/{still}"),
            stage: "stills",
            what: "a box drawn over the capture",
            apply: boxed_still,
        },
        Perturbation {
            file: format!("stills/{}", still.replace(".png", ".json")),
            stage: "stills",
            what: "a camera's box moved by 5 pixels",
            apply: moved_camera,
        },
        Perturbation {
            file: "cards/verdicts-16x9.png".into(),
            stage: "stills",
            what: "an overlay made opaque, its colours untouched",
            apply: opaque_overlay,
        },
    ])
}

/// Runs the self-test in `scratch`; true when the harness behaves.
pub fn selftest(sources: &Sources, scratch: &Path) -> Result<bool, String> {
    let candidate = scratch.join("candidate");
    fs::create_dir_all(&candidate).map_err(|e| e.to_string())?;
    reference_as_candidate(sources, &candidate, scratch)?;
    let mut good = true;

    let clean = check_stage("all", &candidate, sources, scratch)?;
    if clean.failures.is_empty() && clean.checks > 0 {
        println!("the reference itself: all {} checks pass", clean.checks);
    } else {
        good = false;
        println!(
            "the reference itself: {} of {} checks FAIL",
            clean.failures.len(),
            clean.checks
        );
        for f in &clean.failures {
            println!("  {f}");
        }
    }

    for p in perturbations(&candidate)? {
        let path = candidate.join(&p.file);
        let backup = scratch.join("backup");
        fs::copy(&path, &backup).map_err(|e| format!("{}: {e}", path.display()))?;
        let report = (p.apply)(&path, scratch)
            .and_then(|_| check_stage(p.stage, &candidate, sources, scratch));
        fs::rename(&backup, &path).map_err(|e| format!("{}: {e}", path.display()))?;
        let report = report?;
        let named = report.failures.iter().all(|f| f.contains(&p.file));
        let caught = !report.failures.is_empty() && named;
        let verdict = if caught { "caught" } else { "MISSED" };
        println!(
            "{} ({}): {verdict}, {} fails {} of {} checks",
            p.file,
            p.what,
            p.stage,
            report.failures.len(),
            report.checks
        );
        if !named {
            for f in report.failures.iter().filter(|f| !f.contains(&p.file)) {
                println!("  unexpected: {f}");
            }
        }
        good &= caught;
    }
    Ok(good)
}

fn later_cue(path: &Path, _: &Path) -> Result<(), String> {
    let mut timeline: Value = serde_json::from_slice(&crate::read(path)?)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let end = timeline
        .pointer_mut("/sentences/10/cues/0/end")
        .ok_or_else(|| format!("{}: no sentence 10 cue", path.display()))?;
    let later = ((end.as_f64().unwrap_or_default() + 0.001) * 1000.0).round() / 1000.0;
    *end = Value::from(later);
    fs::write(
        path,
        serde_json::to_vec_pretty(&timeline).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("{}: {e}", path.display()))
}

fn misspelled_caption(path: &Path, _: &Path) -> Result<(), String> {
    let text = String::from_utf8(crate::read(path)?).map_err(|e| e.to_string())?;
    if !text.contains(" the ") {
        return Err(format!("{}: no \" the \" to misspell", path.display()));
    }
    fs::write(path, text.replacen(" the ", " teh ", 1))
        .map_err(|e| format!("{}: {e}", path.display()))
}

fn one_sample_off(path: &Path, _: &Path) -> Result<(), String> {
    let mut reader =
        hound::WavReader::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let spec = reader.spec();
    let mut samples = reader
        .samples::<i16>()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let mid = samples.len() / 2;
    samples[mid] = if samples[mid] == i16::MAX {
        i16::MAX - 1
    } else {
        samples[mid] + 1
    };
    let mut writer =
        hound::WavWriter::create(path, spec).map_err(|e| format!("{}: {e}", path.display()))?;
    for s in samples {
        writer.write_sample(s).map_err(|e| e.to_string())?;
    }
    writer.finalize().map_err(|e| e.to_string())
}

/// Runs ffmpeg from `path` into a scratch file with `args` between input and
/// output, then moves the result over `path`.
fn ffmpeg_over(path: &Path, scratch: &Path, args: &[&str]) -> Result<(), String> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("out");
    let tmp = scratch.join(format!("perturbed.{ext}"));
    run(Command::new("ffmpeg")
        .args(["-loglevel", "error", "-y", "-i"])
        .arg(path)
        .args(args)
        .arg(&tmp))?;
    fs::rename(&tmp, path).map_err(|e| format!("{}: {e}", path.display()))
}

fn reencoded_audio(path: &Path, scratch: &Path) -> Result<(), String> {
    ffmpeg_over(
        path,
        scratch,
        &["-map", "0", "-c:v", "copy", "-c:a", "aac", "-b:a", "96k"],
    )
}

fn boxed_still(path: &Path, scratch: &Path) -> Result<(), String> {
    ffmpeg_over(
        path,
        scratch,
        &["-vf", "drawbox=x=0:y=0:w=iw/4:h=ih/4:color=red@1:t=fill"],
    )
}

fn moved_camera(path: &Path, _: &Path) -> Result<(), String> {
    let mut frames: Value = serde_json::from_slice(&crate::read(path)?)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let x = frames
        .pointer_mut("/cameras/app/0")
        .ok_or_else(|| format!("{}: no app camera", path.display()))?;
    *x = Value::from(x.as_f64().unwrap_or_default() + 5.0);
    fs::write(
        path,
        serde_json::to_vec_pretty(&frames).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("{}: {e}", path.display()))
}

fn opaque_overlay(path: &Path, _: &Path) -> Result<(), String> {
    let mut image = vidni_media::compose::image::Image::load_png(path)?.to_rgba();
    for pixel in image.data.chunks_mut(4) {
        pixel[3] = 255;
    }
    image.save_png(path)
}
