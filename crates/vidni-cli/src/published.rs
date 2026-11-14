//! Where the cuts are published, and where a line is (TODO.md,
//! P6-VIDNI-07): `vidni published` and `vidni where`.

use crate::{Loaded, load};
use std::path::Path;
use std::process::ExitCode;
use vidni_core::platforms;
use vidni_core::published::{self, Upload};
use vidni_core::release::{self, Voice};
use vidni_core::timeline::Timeline;

fn failed(why: String) -> ExitCode {
    tell!("vidni: {why}");
    ExitCode::FAILURE
}

/// Records that `cut` is published on `platform` at `url`.
pub fn record(path: &Path, cut: &str, platform: &str, url: &str, tag: Option<&str>) -> ExitCode {
    let Loaded { dir, project, .. } = match load(path) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let Some(found) = project.cuts.iter().find(|c| c.id == cut) else {
        return failed(format!("there is no cut {cut:?}"));
    };
    if platforms::get(platform).is_none() {
        return failed(format!(
            "there is no platform {platform:?}; `vidni platforms` lists them"
        ));
    }
    if !found.platforms().iter().any(|p| p == platform) {
        return failed(format!(
            "cut {cut} isn't made for {platform}; its platforms are in vidni.toml"
        ));
    }
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return failed(format!("{url:?} isn't an http or https address"));
    }
    let done = release::read(&dir).and_then(|releases| {
        // What was uploaded is a release: the one named, or the newest.
        let of_cut = releases.cuts.get(cut).map(Vec::as_slice).unwrap_or_default();
        let uploaded = match tag {
            Some(tag) => of_cut.iter().find(|r| r.tag == tag).ok_or_else(|| {
                format!("cut {cut} has no release {tag:?}; `vidni where` lists its releases")
            })?,
            None => of_cut.last().ok_or_else(|| {
                format!("cut {cut} has no release yet; `vidni approve --cut {cut}` makes one, and a release is what is published")
            })?,
        };
        let mut list = published::read(&dir)?;
        list.add(Upload {
            cut: cut.to_string(),
            platform: platform.to_string(),
            release: uploaded.tag.clone(),
            url: url.to_string(),
            on: release::today(),
        });
        published::write(&dir, &list)?;
        Ok(uploaded.tag.clone())
    });
    match done {
        Ok(tag) => {
            say!("{cut}: {tag} is published on {platform} at {url}");
            say!("  `vidni final` writes it into the platform's UPLOAD.md");
            ExitCode::SUCCESS
        }
        Err(why) => failed(why),
    }
}

/// Lists what is published where.
pub fn list(path: &Path) -> ExitCode {
    let Loaded { dir, .. } = match load(path) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    match published::read(&dir) {
        Ok(list) => {
            for upload in &list.uploads {
                say!(
                    "{}  {}  {}  {}  ({})",
                    upload.cut,
                    upload.platform,
                    upload.release,
                    upload.url,
                    upload.on
                );
            }
            if list.uploads.is_empty() {
                say!(
                    "nothing is published yet; `vidni published --cut ID --platform ID --url ADDRESS` records an upload"
                );
            }
            ExitCode::SUCCESS
        }
        Err(why) => failed(why),
    }
}

/// Where in a cut's newest final build line `position` starts, in seconds.
fn starts(dir: &Path, cut: &str, position: usize) -> Option<f64> {
    let file = dir
        .join(Voice::Takes.dir())
        .join(format!("{cut}.timeline.json"));
    let timeline = Timeline::read(&file)?;
    timeline.sentences.get(position).map(|entry| entry.start)
}

/// Says where line `id` is: each cut that says it, when, its releases
/// and where it is published.
pub fn whereabouts(path: &Path, id: &str) -> ExitCode {
    let Loaded { dir, project, .. } = match load(path) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let Some(line) = project.lines.iter().find(|l| l.id == id) else {
        return failed(format!("there is no line {id:?}"));
    };
    let read = release::read(&dir).and_then(|releases| Ok((releases, published::read(&dir)?)));
    let (releases, list) = match read {
        Ok(read) => read,
        Err(why) => return failed(why),
    };
    say!("{id}: {}", line.show);
    let mut said = 0;
    for cut in &project.cuts {
        let Some(position) = cut.line_ids(&project).iter().position(|l| l == id) else {
            continue;
        };
        said += 1;
        let when = match starts(&dir, &cut.id, position) {
            Some(start) => format!("at {}", platforms::clock(start as u32)),
            None => "not built yet".to_string(),
        };
        let release = match releases.latest(&cut.id) {
            Some(newest) => format!("newest release {}", newest.tag),
            None => "no release".to_string(),
        };
        say!("  {}: {when}; {release}", cut.id);
        let mut anywhere = false;
        for upload in list.of(&cut.id) {
            anywhere = true;
            say!(
                "    {} on {}: {}",
                upload.release,
                upload.platform,
                upload.url
            );
        }
        if !anywhere {
            say!("    not published");
        }
    }
    if said == 0 {
        say!("  no cut says it");
    }
    let corrections = project.errata.iter().filter(|e| e.line == id).count();
    if corrections > 0 {
        say!(
            "  {corrections} correction{} to it in [[errata]]; `vidni final` writes {} into each description",
            if corrections == 1 { "" } else { "s" },
            if corrections == 1 { "it" } else { "them" }
        );
    }
    ExitCode::SUCCESS
}
