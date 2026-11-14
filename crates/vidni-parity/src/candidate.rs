//! Candidates built by the Rust port, for the stages ported so far.
//!
//! Each builder runs a vidni stage on the same inputs the reference used, and
//! writes what the reference wrote, so `vidni-parity <stage>` can compare
//! the two. The nibli example is the project; the reference's synthetic
//! clips and the example's takes are the audio.

use crate::VARIANTS;
use std::fs;
use std::path::{Path, PathBuf};
use vidni_core::capture::{self, Source};
use vidni_core::project::{Aspect, LineSelection, Project, SceneKind, Split};
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
        "stills" => stills(example, dst),
        _ => Err(format!("no Rust port to build for {stage}")),
    }
}

/// The example as the reference has it: the whole script, in the cuts that
/// say all of it. The reference has no shorts, so a cut that picks its
/// lines is left out, and so are the lines that are only for such cuts.
pub(crate) fn project(example: &Path) -> Result<Project, String> {
    let file = example.join("vidni.toml");
    let text = fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
    let mut project = vidni_core::validate::check(&text, Some(example)).map_err(|problems| {
        let first = problems
            .first()
            .map(ToString::to_string)
            .unwrap_or_default();
        format!("{}: {first}", file.display())
    })?;
    // The reference drew its two overlays as pages, from `cards/card.html`.
    // The example has them as data now (P5-VIDNI-05); here they are pages
    // again, so that the reference's frames are still what is compared.
    for (id, height) in [("verdicts", 150), ("firewall", 190)] {
        let drawn = project
            .scenes
            .get_mut(id)
            .filter(|scene| scene.kind == SceneKind::Overlay);
        if let Some(scene) = drawn {
            let page = serde_json::json!({
                "kind": "page",
                "url": format!("cards/card.html?v={id}&r={{aspect}}"),
                "transparent": true,
                "size": { "landscape": [1600, height], "portrait": [1000, 330] },
            });
            *scene = serde_json::from_value(page).map_err(|e| e.to_string())?;
        }
    }
    // The reference spoke one language: the script's own.
    let own = project.meta.language.clone();
    let mut project = project.in_language(&own);
    project.cuts.retain(|c| c.lines == LineSelection::All);
    project.lines.retain(|l| l.only.is_empty());
    // The reference cut its captions its own way, and held no last picture.
    project.captions.split = Split::Reference;
    for cut in &mut project.cuts {
        cut.end_hold = None;
    }
    Ok(project)
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

/// The example's stills and cards, captured afresh by the Rust port into a
/// copy of the example, and laid out as the reference names them:
/// `stills/NNN.png` by line number, each with its frames as
/// `stills/NNN.json`, and `cards/<scene>-<aspect>.png`.
///
/// The reference's frames hold each camera's box. vidni's hold the boxes
/// of the scene's targets, and a camera is framed from them when a cut is
/// built. So a still's frames are written here with its cameras framed
/// that way, which checks the framing as well as the measuring.
fn stills(example: &Path, dst: &Path) -> Result<(), String> {
    let work = dst.join("project");
    let (project, _) = crate::capture::fresh(example, &work)?;
    let made = |e: std::io::Error| e.to_string();
    let copy = |from: PathBuf, to: PathBuf| {
        fs::copy(&from, &to)
            .map(drop)
            .map_err(|e| format!("{}: {e}", from.display()))
    };
    fs::create_dir_all(dst.join("stills")).map_err(made)?;
    fs::create_dir_all(dst.join("cards")).map_err(made)?;
    let keys = capture::line_keys(&project);
    for (k, line) in project.lines.iter().enumerate() {
        if let Some(key) = keys.get(&line.id) {
            copy(
                capture::path(&work, key),
                dst.join(format!("stills/{k:03}.png")),
            )?;
            let file = vidni_media::capture::frames_path(&work, key);
            let frames: vidni_media::capture::Frames = fs::read_to_string(&file)
                .map_err(|e| e.to_string())
                .and_then(|text| serde_json::from_str(&text).map_err(|e| e.to_string()))
                .map_err(|e| format!("{}: {e}", file.display()))?;
            let scene = line
                .scene_in(&project)
                .and_then(|id| project.scenes.get(id))
                .ok_or_else(|| format!("line {} shows no scene", line.id))?;
            let cameras: serde_json::Map<String, serde_json::Value> = scene
                .cameras
                .keys()
                .map(String::as_str)
                .chain(["viewport", "highlights"])
                .map(|name| {
                    (
                        name.to_string(),
                        serde_json::json!(frames.camera(name, scene)),
                    )
                })
                .collect();
            let framed = serde_json::json!({
                "viewport": frames.viewport,
                "scale": frames.scale,
                "cameras": cameras,
                "highlights": frames.highlights,
                "browser": frames.browser,
            });
            let json = serde_json::to_string_pretty(&framed).map_err(|e| e.to_string())?;
            fs::write(dst.join(format!("stills/{k:03}.json")), json).map_err(made)?;
        }
    }
    for cut in &project.cuts {
        let aspect = match cut.aspect {
            Aspect::Landscape => "16x9",
            Aspect::Portrait => "9x16",
        };
        for c in capture::for_cut(&project, cut) {
            if let Source::Page { scene, .. } = &c.source {
                copy(
                    capture::path(&work, &c.key),
                    dst.join(format!("cards/{scene}-{aspect}.png")),
                )?;
            }
        }
    }
    Ok(())
}
