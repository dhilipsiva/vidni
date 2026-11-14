//! `vidni exports` (TODO.md, P7-VIDNI-07): which platforms each cut is
//! made for, what its last final build and export found, and where each
//! platform's upload folder is; and, with `--add` or `--remove`, a cut
//! made for one platform more or less.
//!
//! It reads what `vidni final` left behind and builds nothing.

use crate::{Loaded, load};
use std::fs;
use std::path::Path;
use std::process::ExitCode;
use vidni_core::project::{Aspect, Cut};
use vidni_core::release::{self, Record, Voice};
use vidni_core::{edit, platforms, validate};
use vidni_media::build::Exports;

fn shape(aspect: Aspect) -> &'static str {
    match aspect {
        Aspect::Landscape => "16:9",
        Aspect::Portrait => "9:16",
    }
}

/// Where cut `cut` is exported for `platform`, relative to the project.
pub fn folder(platform: &str, cut: &str) -> String {
    format!("{}/{platform}/{cut}", release::OUT)
}

/// Says where `cut` stands with each platform.
fn of(dir: &Path, cut: &Cut) -> Result<(), String> {
    let wanted = cut.platforms();
    let made_for = match wanted.is_empty() {
        true => "for no platform".to_string(),
        false => format!("for {}", wanted.join(", ")),
    };
    say!("{} ({}), {made_for}", cut.id, shape(cut.aspect));
    let Some(record) = Record::read(dir, Voice::Takes, &cut.id)? else {
        say!(
            "  no final build yet; `vidni final --cut {}` builds and exports it",
            cut.id
        );
        return Ok(());
    };
    // What QA measured of each picture, and what it found wrong.
    for (name, output) in &record.outputs {
        say!("  {name}: {}", output.qa);
        for problem in &output.problems {
            say!("  {name}: QA: {problem}");
        }
    }
    if record.outputs.values().all(|o| o.problems.is_empty()) {
        say!("  QA: nothing wrong");
    }
    let found = Exports::read(dir, &cut.id)?;
    let exported = found.as_ref().map(|found| found.platforms.clone());
    for platform in &wanted {
        let folder = folder(platform, &cut.id);
        let files = fs::read_dir(dir.join(&folder)).map(|entries| entries.count());
        // Why it wasn't exported, when the last export said why.
        let refusal = found.iter().flat_map(|found| &found.refused).find(|said| {
            said.starts_with(&format!("cut {} isn't exported for {platform}: ", cut.id))
        });
        match (files, refusal, &exported) {
            (_, Some(refusal), _) => {
                let why = refusal.split_once(": ").map_or("", |(_, why)| why);
                say!("  {platform}: not exported: {why}");
            }
            // A folder from before exports were noted is one too.
            (Ok(files), None, was) if was.as_ref().is_none_or(|was| was.contains(platform)) => {
                say!("  {platform}: {folder} ({})", crate::count(files, "file"));
            }
            _ => say!("  {platform}: not exported yet; the next `vidni final` exports it"),
        }
    }
    // A folder of a platform the cut is no longer made for.
    for profile in platforms::all() {
        let folder = folder(&profile.id, &cut.id);
        if !wanted.contains(&profile.id) && dir.join(&folder).is_dir() {
            say!(
                "  {}: {folder} is from before; the next `vidni final` removes it",
                profile.id
            );
        }
    }
    for warning in found.iter().flat_map(|found| &found.warnings) {
        say!("  warning: {warning}");
    }
    Ok(())
}

/// Lists where each of `cuts` stands with its platforms; every cut when
/// none is named.
pub fn list(path: &Path, cuts: &[String]) -> ExitCode {
    let Loaded { dir, project, .. } = match load(path) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let cuts = match crate::build::chosen(&project, cuts) {
        Ok(cuts) => cuts,
        Err(code) => return code,
    };
    for cut in cuts {
        if let Err(e) = of(&dir, cut) {
            tell!("vidni: {e}");
            return ExitCode::from(2);
        }
    }
    ExitCode::SUCCESS
}

/// Makes cut `id` for `platform` as well (`on`), or no longer.
pub fn target(path: &Path, id: &str, platform: &str, on: bool) -> ExitCode {
    let Loaded { file, dir, project } = match load(path) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let refused = |why: String| {
        tell!("vidni: {why}");
        ExitCode::FAILURE
    };
    let Some(cut) = project.cuts.iter().find(|cut| cut.id == id) else {
        return refused(format!("there is no cut {id:?}"));
    };
    let Some(profile) = platforms::get(platform) else {
        let known: Vec<String> = platforms::all().into_iter().map(|p| p.id).collect();
        return refused(format!(
            "there is no platform {platform:?}; the platforms are {}",
            known.join(", ")
        ));
    };
    if on && profile.aspect != cut.aspect {
        return refused(format!(
            "{platform} takes {} and cut {id} is {}",
            shape(profile.aspect),
            shape(cut.aspect)
        ));
    }
    let before = cut.platforms();
    if before.contains(&profile.id) == on {
        match on {
            true => say!("{id}: already for {platform}"),
            false => say!("{id}: already not for {platform}"),
        }
        return ExitCode::SUCCESS;
    }
    // In the profiles' own order, whatever order they were added in.
    let after: Vec<String> = platforms::all()
        .into_iter()
        .map(|p| p.id)
        .filter(|p| match *p == profile.id {
            true => on,
            false => before.contains(p),
        })
        .collect();
    let edited = fs::read_to_string(&file)
        .map_err(|e| e.to_string())
        .and_then(|text| edit::ProjectFile::parse(&text).map_err(|_| "it has problems".into()))
        .and_then(|mut text| {
            text.set_cut_platforms(id, &after)
                .map_err(|e| e.to_string())?;
            Ok(text.text())
        })
        .and_then(|text| match validate::check(&text, Some(&dir)) {
            Ok(_) => Ok(text),
            Err(problems) => Err(format!(
                "the file would then have a problem: {}",
                problems
                    .first()
                    .map(|p| p.message.clone())
                    .unwrap_or_default()
            )),
        })
        .and_then(|text| {
            // Written beside the file and moved over it, so that the file
            // is never half of itself.
            let part = file.with_extension("toml.part");
            fs::write(&part, text)
                .and_then(|()| fs::rename(&part, &file))
                .map_err(|e| e.to_string())
        });
    if let Err(why) = edited {
        return refused(format!("{}: {why}", file.display()));
    }
    let now = match after.is_empty() {
        true => "now for no platform".to_string(),
        false => format!("now for {}", after.join(", ")),
    };
    match on {
        true => say!("{id}: {now}; the next `vidni final` exports it for {platform}"),
        false => say!(
            "{id}: {now}; the next `vidni final` removes {}",
            folder(platform, id)
        ),
    }
    ExitCode::SUCCESS
}
