//! Writes frames of a project's final build as PNGs, to look at or to feed
//! to other tools: the frames `vidni final` would encode, before encoding.
//!
//! `cargo run --release -p vidni-parity --example product_frames -- PROJECT CUT OUT FIRST LAST`

use std::path::PathBuf;
use vidni_core::release::Voice;
use vidni_media::build::{self, Setup, Stop};

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [project, cut, out, first, last] = args.as_slice() else {
        return Err("usage: product_frames PROJECT CUT OUT FIRST LAST".into());
    };
    let dir = PathBuf::from(project);
    let text = std::fs::read_to_string(dir.join("vidni.toml")).map_err(|e| e.to_string())?;
    let project = vidni_core::validate::check(&text, Some(&dir)).map_err(|p| format!("{p:?}"))?;
    let cut = project
        .cuts
        .iter()
        .find(|c| &c.id == cut)
        .ok_or("no such cut")?;
    let plan = build::plan(
        &project,
        &dir,
        cut,
        Voice::Takes,
        None,
        &Setup::find(&project, &dir)?,
    )
    .map_err(|stop| match stop {
        Stop::Refused(reasons) => reasons.join("; "),
        Stop::Failed(e) => e,
    })?;
    let mut composer = plan.composer(true)?;
    let out = PathBuf::from(out);
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let (first, last): (u64, u64) = (
        first.parse().map_err(|_| "FIRST is a frame number")?,
        last.parse().map_err(|_| "LAST is a frame number")?,
    );
    for f in first..=last {
        composer
            .frame(f)?
            .save_png(&out.join(format!("{f:06}.png")))?;
    }
    Ok(())
}
