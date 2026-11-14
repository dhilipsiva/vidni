//! The product against the reference (TODO.md, P3-VIDNI-06): the final
//! build of the nibli example, as `vidni final` plans it from the example's
//! `vidni.toml`, its takes and vidni's own captures, against the reference's
//! cuts in the creator's voice.
//!
//! The stage checks compare one Rust stage on the reference's inputs. This
//! compares the stages wired together on the product's own: its timeline,
//! captions and narration must equal golden's, and its frames must match
//! the reference's frames before encoding.
//!
//! Golden was made with the maintainer's fonts, so this check uses them
//! too, where the product pins its own: the captures are taken afresh into
//! a copy of the example, with the host's fontconfig (the `parity` recipe
//! sets the flake's aside), and the frames are drawn with the reference's
//! three faces.

use crate::compose::reference_faces;
use crate::{
    CutSamples, Report, Sources, cut_samples, json_diff, largest_difference, load_json, pcm_sha256,
    reference_frames, ssim,
};
use std::fs;
use std::path::Path;
use vidni_core::project::Aspect;
use vidni_core::release::Voice;
use vidni_core::timeline;
use vidni_media::build::{self, Faces, Setup, Stop};

/// Checks each cut of the example in `example` against the reference's
/// voice variant.
pub fn check(example: &Path, sources: &Sources, scratch: &Path, report: &mut Report) {
    if let Err(e) = compare(example, sources, scratch, report) {
        report.fail(e);
    }
}

fn compare(
    example: &Path,
    sources: &Sources,
    scratch: &Path,
    report: &mut Report,
) -> Result<(), String> {
    // A copy of the example with fresh captures and the creator's takes.
    let work = scratch.join("product");
    let (project, _) = crate::capture::fresh(example, &work)?;
    let takes = example.join("takes");
    let takes = fs::canonicalize(&takes).map_err(|e| format!("{}: {e}", takes.display()))?;
    std::os::unix::fs::symlink(takes, work.join("takes")).map_err(|e| format!("takes: {e}"))?;
    let faces = reference_faces()?;
    let setup = Setup::with(
        Faces {
            caption: faces.caption,
            mono: faces.mono,
            display: faces.display,
            script: None,
        },
        vidni_core::theme::load(example, &project.style.theme)?,
    )?;
    let hashes = load_json(&sources.golden.join("hashes.json"))?;
    let golden_timeline = load_json(&sources.golden.join("timeline-voice.json"))?;
    let golden_srt = fs::read_to_string(sources.golden.join("nibli-syllogism-voice.en.srt"))
        .map_err(|e| format!("the golden captions: {e}"))?;
    let golden_pcm = hashes["outputs"]["narration-voice.wav (pcm)"]
        .as_str()
        .ok_or("narration-voice.wav: no golden hash")?;
    let samples: Vec<CutSamples> = cut_samples(&sources.reference.join("out"))?
        .into_iter()
        .filter(|s| s.variant == "voice")
        .collect();
    let theirs = scratch.join("reference-frames");
    reference_frames(&sources.reference, &samples, &theirs)?;
    let ours = scratch.join("product-frames");
    fs::create_dir_all(&ours).map_err(|e| e.to_string())?;

    for cut in &project.cuts {
        let shape = match cut.aspect {
            Aspect::Landscape => "16x9",
            Aspect::Portrait => "9x16",
        };
        let plan = build::plan(&project, &work, cut, Voice::Takes, None, &setup).map_err(
            |stop| match stop {
                Stop::Refused(reasons) => reasons.join("; "),
                Stop::Failed(e) => e,
            },
        )?;
        // The timeline, apart from two things the reference writes its own
        // way: it numbers the clips, and it writes what its synthetic voice
        // was given into both variants, where vidni's final build writes
        // what the creator reads.
        let mut timeline =
            serde_json::to_value(plan.timeline()).map_err(|e| format!("the timeline: {e}"))?;
        let mut golden_timeline = golden_timeline.clone();
        for (k, entry) in timeline["sentences"]
            .as_array_mut()
            .into_iter()
            .flatten()
            .enumerate()
        {
            entry["wav"] = format!("recordings/{k:03}.wav").into();
            if let Some(theirs) = golden_timeline["sentences"].get_mut(k) {
                theirs["say"] = entry["say"].clone();
            }
        }
        match json_diff(&golden_timeline, &timeline, "") {
            None => report.pass(),
            Some(d) => report.fail(format!("cut {}: its timeline: {d}", cut.id)),
        }
        report.check(timeline::srt(plan.timeline()) == golden_srt, || {
            format!("cut {}: its captions differ from golden", cut.id)
        });
        let narration = scratch.join(format!("product-narration-{}.wav", cut.id));
        vidni_media::narration::assemble(plan.timeline(), &plan.clips(), &narration)?;
        report.check(pcm_sha256(&narration)? == golden_pcm, || {
            format!("cut {}: its narration differs from golden", cut.id)
        });
        // Its frames, at the samples of the reference's cut of this shape.
        let sample = samples
            .iter()
            .find(|s| s.cut == shape)
            .ok_or_else(|| format!("no reference cut is {shape}"))?;
        let mut composer = plan.composer(true)?;
        for (t, f) in &sample.frames {
            let file = format!("{}-{f}.png", sample.name);
            let what = format!("cut {}, frame {f} at {t:.2} s", cut.id);
            let (a, b) = (theirs.join(&file), ours.join(&file));
            composer.frame(*f)?.save_png(&b)?;
            match ssim(&a, &b).and_then(|score| Ok((score, largest_difference(&a, &b)?))) {
                Ok((score, difference)) => {
                    report.difference(difference, &what);
                    report.score(score, what);
                }
                Err(e) => report.fail(format!("{what}: {e}")),
            }
        }
    }
    Ok(())
}
