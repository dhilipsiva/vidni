//! Candidates built by the Rust port, for the stages ported so far.
//!
//! Each builder runs a vidni stage on the same inputs the reference used, and
//! writes what the reference wrote, so `vidni-parity <stage>` can compare
//! the two. The nibli example is the project; the reference's synthetic
//! clips and the example's takes are the audio.

use crate::VARIANTS;
use std::fs;
use std::path::{Path, PathBuf};
use vidni_core::project::Project;
use vidni_core::takes::{self, Status};
use vidni_core::timeline::{self, Clip};

/// Writes the Rust port's outputs for `stage` into `dst`.
pub fn build(stage: &str, example: &Path, reference: &Path, dst: &Path) -> Result<(), String> {
    match stage {
        "timeline" => timelines(example, reference, dst),
        "narration" => narrations(example, reference, dst),
        "encode" => Err(
            "the Rust encoder is held to T2, not to the reference's packets: its flags \
                         differ on purpose (bitexact, fixed threads, -cpuflags 0). Run \
                         `just encode-t2` (P3-VIDNI-01)"
                .into(),
        ),
        "cuts" => cuts(example, reference, dst),
        "stills" => Err("the Rust port of stills isn't built yet (P3-VIDNI-04)".into()),
        _ => Err(format!("no Rust port to build for {stage}")),
    }
}

pub(crate) fn project(example: &Path) -> Result<Project, String> {
    let file = example.join("vidni.toml");
    let text = fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
    vidni_core::validate::check(&text, Some(example)).map_err(|problems| {
        let first = problems
            .first()
            .map(ToString::to_string)
            .unwrap_or_default();
        format!("{}: {first}", file.display())
    })
}

/// A clip's length from its WAV header, named as the reference names it.
fn clip(path: &Path, wav: String) -> Result<Clip, String> {
    let reader = hound::WavReader::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(Clip {
        wav,
        frames: u64::from(reader.duration()),
        rate: reader.spec().sample_rate,
    })
}

/// Each variant's clips, as files and as the reference names them: the
/// synthetic ones from the reference's cached Piper takes, the creator's
/// from the example's takes.
pub(crate) fn variants(
    project: &Project,
    example: &Path,
    reference: &Path,
) -> Result<[Vec<(PathBuf, Clip)>; 2], String> {
    let synthetic = (0..project.lines.len())
        .map(|k| {
            let name = format!("build/audio/{k:03}.wav");
            let path = reference.join(&name);
            clip(&path, name).map(|c| (path, c))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let voice = takes::status(project, example)
        .into_iter()
        .enumerate()
        .map(|(k, line)| match line.status {
            Status::Current(path) => {
                clip(&path, format!("recordings/{k:03}.wav")).map(|c| (path, c))
            }
            _ => Err(format!("line {} has no current take", line.id)),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok([synthetic, voice])
}

/// Both variants' narration tracks, assembled on their timelines.
fn narrations(example: &Path, reference: &Path, dst: &Path) -> Result<(), String> {
    let project = project(example)?;
    for ((variant, _), clips) in VARIANTS
        .into_iter()
        .zip(variants(&project, example, reference)?)
    {
        let infos: Vec<Clip> = clips.iter().map(|(_, c)| c.clone()).collect();
        let paths: Vec<&Path> = clips.iter().map(|(p, _)| p.as_path()).collect();
        let timeline = timeline::build(&project, &infos)?;
        let out = dst.join(format!("narration-{variant}.wav"));
        vidni_media::narration::assemble(&timeline, &paths, &out)?;
    }
    Ok(())
}

/// Both variants' timelines and captions.
fn timelines(example: &Path, reference: &Path, dst: &Path) -> Result<(), String> {
    let project = project(example)?;
    for ((variant, suffix), clips) in VARIANTS
        .into_iter()
        .zip(variants(&project, example, reference)?)
    {
        let clips: Vec<Clip> = clips.into_iter().map(|(_, c)| c).collect();
        let timeline = timeline::build(&project, &clips)?;
        let json = serde_json::to_string_pretty(&timeline).map_err(|e| e.to_string())?;
        let write = |name: String, text: String| {
            fs::write(dst.join(&name), text).map_err(|e| format!("{name}: {e}"))
        };
        write(format!("timeline-{variant}.json"), json)?;
        write(
            format!("nibli-syllogism{suffix}.en.srt"),
            timeline::srt(&timeline),
        )?;
    }
    Ok(())
}

/// Every cut's sample frames, composited by the Rust port from the
/// reference's captures, as `frames/NAME-F.png`.
fn cuts(example: &Path, reference: &Path, dst: &Path) -> Result<(), String> {
    let faces = crate::compose::reference_faces()?;
    let samples = crate::cut_samples(&reference.join("out"))?;
    let frames = dst.join("frames");
    fs::create_dir_all(&frames).map_err(|e| format!("{}: {e}", frames.display()))?;
    for mut job in crate::compose::jobs(example, reference, &faces)? {
        let cut = samples
            .iter()
            .find(|s| s.name == job.name)
            .ok_or_else(|| format!("{}: no samples", job.name))?;
        for (_, f) in &cut.frames {
            let png = frames.join(format!("{}-{f}.png", job.name));
            job.composer.frame(*f)?.save_png(&png)?;
        }
    }
    Ok(())
}
