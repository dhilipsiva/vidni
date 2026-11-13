//! Candidates built by the Rust port, for the stages ported so far.
//!
//! Each builder runs a vidni stage on the same inputs the reference used, and
//! writes what the reference wrote, so `vidni-parity <stage>` can compare
//! the two. The nibli example is the project; the reference's synthetic
//! clips and the example's takes are the audio.

use crate::{CUTS, VARIANTS};
use std::fs;
use std::path::Path;
use vidni_core::project::Project;
use vidni_core::takes::{self, Status};
use vidni_core::timeline::{self, Clip};

/// Writes the Rust port's outputs for `stage` into `dst`.
pub fn build(stage: &str, example: &Path, reference: &Path, dst: &Path) -> Result<(), String> {
    match stage {
        "timeline" => timelines(example, reference, dst),
        "narration" | "encode" => Err(format!(
            "the Rust port of {stage} isn't built yet (P3-VIDNI-01)"
        )),
        "cuts" => Err(format!(
            "the Rust port of cuts isn't built yet (P3-VIDNI-02); there are {} cuts to match",
            CUTS.len()
        )),
        "stills" => Err("the Rust port of stills isn't built yet (P3-VIDNI-04)".into()),
        _ => Err(format!("no Rust port to build for {stage}")),
    }
}

fn project(example: &Path) -> Result<Project, String> {
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

/// Both variants' timelines and captions: the synthetic one from the
/// reference's cached Piper takes, the creator's from the example's takes.
fn timelines(example: &Path, reference: &Path, dst: &Path) -> Result<(), String> {
    let project = project(example)?;
    let synthetic = (0..project.lines.len())
        .map(|k| {
            let name = format!("build/audio/{k:03}.wav");
            clip(&reference.join(&name), name)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let voice = takes::status(&project, example)
        .into_iter()
        .enumerate()
        .map(|(k, line)| match line.status {
            Status::Current(path) => clip(&path, format!("recordings/{k:03}.wav")),
            _ => Err(format!("line {} has no current take", line.id)),
        })
        .collect::<Result<Vec<_>, _>>()?;
    for ((variant, suffix), clips) in VARIANTS.into_iter().zip([synthetic, voice]) {
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
