//! The capture checks (TODO.md, P3-VIDNI-04): the Rust port captures the
//! nibli example afresh into a copy of it, and five captures of its web
//! scene must give the same pixels.

use crate::candidate::project;
use crate::sha256_hex;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use vidni_core::capture::{self, Capture, Source};
use vidni_core::project::Project;
use vidni_media::compose::image::Image;

/// Copies what capturing the example reads, its vidni.toml and cards, into
/// `work`, so that every capture there is taken afresh.
pub fn copy_example(example: &Path, work: &Path) -> Result<(), String> {
    let made = |e: std::io::Error| format!("{}: {e}", work.display());
    fs::create_dir_all(work.join("cards")).map_err(made)?;
    fs::copy(example.join("vidni.toml"), work.join("vidni.toml")).map_err(made)?;
    for entry in fs::read_dir(example.join("cards")).map_err(made)? {
        let path = entry.map_err(made)?.path();
        if path.is_file() {
            let name = path.file_name().ok_or("a file name")?;
            fs::copy(&path, work.join("cards").join(name)).map_err(made)?;
        }
    }
    Ok(())
}

/// Captures what the example's cuts need into a fresh copy of it, in `work`.
pub fn fresh(example: &Path, work: &Path) -> Result<(Project, Vec<Capture>), String> {
    let project = project(example)?;
    copy_example(example, work)?;
    let cuts: Vec<_> = project.cuts.iter().collect();
    let plan = capture::plan(&project, work, &cuts);
    let taken = vidni_media::capture::capture(&project, work, &plan)?;
    Ok((project, taken))
}

/// Captures the example's web scene `times` times, each from scratch in its
/// own folder and browser, and compares every still's pixels and frames
/// across the runs. Prints each run and the verdict; true when every run
/// agrees. `$VIDNI_KEEP_RUNS` names a folder to keep the runs in, to look at.
pub fn repeat(example: &Path, scratch: &Path, times: usize) -> Result<bool, String> {
    let keep = std::env::var_os("VIDNI_KEEP_RUNS").map(std::path::PathBuf::from);
    // Each run: every still's pixels and frames, by capture key.
    let mut runs: Vec<BTreeMap<String, (String, String)>> = Vec::new();
    for run in 1..=times {
        let work = keep
            .as_deref()
            .unwrap_or(scratch)
            .join(format!("run-{run}"));
        let (_, taken) = fresh(example, &work)?;
        let mut stills = BTreeMap::new();
        for c in taken.iter().filter(|c| matches!(c.source, Source::Line(_))) {
            let image = Image::load_png(&capture::path(&work, &c.key))?;
            let frames = vidni_media::capture::frames_path(&work, &c.key);
            let frames = fs::read(&frames).map_err(|e| format!("{}: {e}", frames.display()))?;
            stills.insert(
                c.key.clone(),
                (sha256_hex(&image.data), sha256_hex(&frames)),
            );
        }
        println!("capture {run} of {times}: {} stills", stills.len());
        runs.push(stills);
        if keep.is_none() {
            fs::remove_dir_all(&work).map_err(|e| format!("{}: {e}", work.display()))?;
        }
    }
    let first = &runs[0];
    let mut same = !first.is_empty();
    for (i, run) in runs.iter().enumerate().skip(1) {
        let differing = |pick: fn(&(String, String)) -> &String| {
            first
                .iter()
                .filter(|(key, still)| run.get(*key).map(pick) != Some(pick(still)))
                .count()
        };
        let (pixels, frames) = (differing(|s| &s.0), differing(|s| &s.1));
        if pixels > 0 || frames > 0 || run.len() != first.len() {
            same = false;
            println!(
                "  capture {} differs from capture 1: the pixels of {pixels} stills, the frames of {frames}",
                i + 1
            );
        }
    }
    println!(
        "{}",
        if same {
            format!("all {times} captures gave the same pixels and frames for every still")
        } else {
            "the captures DIFFER".to_string()
        }
    );
    Ok(same)
}
