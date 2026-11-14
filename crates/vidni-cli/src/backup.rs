//! `vidni backup`, `verify`, `restore` and `gc` (TODO.md, P4-VIDNI-05):
//! the project's assets, which nothing can make again, copied to two
//! places and checked against `assets.lock`; and the files that can be
//! made again, deleted to make room.

use crate::build::cant;
use crate::{Loaded, count, load};
use std::collections::BTreeMap;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use vidni_core::assets::{self, Asset, Rebuildable, State};
use vidni_core::release::Releases;
use vidni_core::{ratify, release};

/// The places backups are in: the ones named, or else the ones
/// `$VIDNI_BACKUP` lists, separated by `:`.
fn places(named: &[PathBuf]) -> Vec<PathBuf> {
    if !named.is_empty() {
        return named.to_vec();
    }
    std::env::var_os("VIDNI_BACKUP")
        .map(|listed| std::env::split_paths(&listed).collect())
        .unwrap_or_default()
}

/// The project's locks: what it has ratified and what it has released.
fn locks(dir: &Path) -> Result<(ratify::Lock, Releases), ExitCode> {
    let lock = ratify::read_lock(dir).map_err(cant)?;
    let releases = release::read(dir).map_err(cant)?;
    Ok((lock, releases))
}

/// The first few of `paths`, each on a line of its own after `what`.
fn list(what: &str, paths: &[String]) {
    for path in paths.iter().take(10) {
        tell!("vidni: {what}: {path}");
    }
    if paths.len() > 10 {
        tell!("vidni: and {} more", paths.len() - 10);
    }
}

pub fn backup(path: &Path, to: &[PathBuf]) -> ExitCode {
    let loaded = match load(path) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let Loaded { dir, project, .. } = &loaded;
    let places = places(to);
    let distinct: std::collections::BTreeSet<&PathBuf> = places.iter().collect();
    if distinct.len() < 2 {
        tell!(
            "vidni: a backup goes to two places, one of them off this machine: name both with --to, or in $VIDNI_BACKUP"
        );
        return ExitCode::FAILURE;
    }
    // A place that isn't there is a disk that isn't plugged in, or a
    // folder that isn't mounted: making it would put the backup on this
    // disk, where it isn't one.
    if let Some(gone) = places.iter().find(|place| !place.is_dir()) {
        return cant(format!(
            "{} isn't there: is the disk plugged in, or the folder mounted?",
            gone.display()
        ));
    }
    let done = locks(dir).and_then(|(lock, releases)| {
        let assets = assets::snapshot(dir, &lock).map_err(cant)?;
        let written = assets::write_lock(dir, &assets).map_err(cant)?;
        say!(
            "{}: {}{}",
            dir.join(assets::LOCK).display(),
            count(assets.len(), "asset"),
            if written { ", written afresh" } else { "" }
        );
        for place in &places {
            let folder = assets::folder(place, dir).map_err(cant)?;
            let backed = assets::backup(project, dir, &assets, &releases, &folder).map_err(cant)?;
            say!(
                "{}: {}, {} copied; {}, {} copied",
                backed.folder.display(),
                count(backed.assets, "asset"),
                backed.copied,
                count(backed.delivered, "delivered file"),
                backed.delivered_copied
            );
        }
        Ok(())
    });
    if let Err(code) = done {
        return code;
    }
    // On one disk with the project, a backup goes when the disk goes.
    #[cfg(unix)]
    let disk = |path: &Path| std::fs::metadata(path).map(|meta| meta.dev()).ok();
    // Elsewhere a disk can't be told from a path: no two are called the same.
    #[cfg(not(unix))]
    let disk = |path: &Path| Some(path.to_path_buf());
    if places.iter().all(|place| disk(place) == disk(dir)) {
        tell!(
            "vidni: warning: every place is on the disk the project is on; one of them should be off this machine"
        );
    }
    ExitCode::SUCCESS
}

/// Says how `folder` stands against `assets`; how many don't stand.
fn check(folder: &Path, assets: &[Asset]) -> usize {
    let faults = assets::faults(folder, assets);
    let of = |state: State| -> Vec<String> {
        faults
            .iter()
            .filter(|(_, s)| *s == state)
            .map(|(path, _)| folder.join(path).display().to_string())
            .collect()
    };
    list("missing", &of(State::Missing));
    list("not the file that is listed", &of(State::Changed));
    faults.len()
}

pub fn verify(path: &Path, to: &[PathBuf]) -> ExitCode {
    let loaded = match load(path) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let Loaded { dir, project, .. } = &loaded;
    let checked = locks(dir).and_then(|(lock, releases)| {
        let assets = assets::read_lock(dir).map_err(cant)?;
        if assets.is_empty() {
            return Err(cant(format!(
                "{} lists no asset; `vidni backup` writes it",
                dir.join(assets::LOCK).display()
            )));
        }
        let mut faults = check(dir, &assets);
        say!(
            "{}: {} of {} as listed",
            dir.display(),
            assets.len() - faults,
            count(assets.len(), "asset")
        );
        // What is there and isn't listed was recorded or frozen since.
        let listed: Vec<&str> = assets.iter().map(|asset| asset.path.as_str()).collect();
        let unlisted: Vec<String> = assets::paths(dir, &lock)
            .map_err(cant)?
            .into_iter()
            .filter(|path| !listed.contains(&path.as_str()))
            .collect();
        if let Some(first) = unlisted.first() {
            say!(
                "{}: {} listed yet (the first: {first}); `vidni backup` lists and copies them",
                dir.display(),
                match unlisted.len() {
                    1 => "1 asset isn't".to_string(),
                    n => format!("{n} assets aren't"),
                }
            );
        }
        for place in places(to) {
            let folder = assets::folder(&place, dir).map_err(cant)?;
            if !folder.is_dir() {
                tell!("vidni: {} holds no backup of this project", place.display());
                faults += 1;
                continue;
            }
            let missing = check(&folder, &assets);
            // Each cut's newest release, where it was archived.
            let mut archived = Vec::new();
            for release in releases.cuts.values().filter_map(|of_cut| of_cut.last()) {
                let archive = folder.join(assets::RELEASES).join(&release.tag);
                if !archive.is_dir() {
                    continue;
                }
                faults += check(&archive, &assets::delivered(project, release));
                archived.push(release.tag.as_str());
            }
            say!(
                "{}: {} of {} as listed{}",
                folder.display(),
                assets.len() - missing,
                count(assets.len(), "asset"),
                match archived.is_empty() {
                    true => String::new(),
                    false => format!("; holds {}", archived.join(" and ")),
                }
            );
            faults += missing;
        }
        Ok(faults)
    });
    match checked {
        Ok(0) => ExitCode::SUCCESS,
        Ok(_) => ExitCode::FAILURE,
        Err(code) => code,
    }
}

pub fn restore(path: &Path, from: &[PathBuf]) -> ExitCode {
    let loaded = match load(path) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let Loaded { dir, project, .. } = &loaded;
    let places = places(from);
    if places.is_empty() {
        tell!("vidni: name the backup to restore from with --from, or in $VIDNI_BACKUP");
        return ExitCode::FAILURE;
    }
    let restored = locks(dir).and_then(|(_, releases)| {
        let mut folders = Vec::new();
        for place in &places {
            let folder = assets::folder(place, dir).map_err(cant)?;
            match folder.is_dir() {
                true => folders.push(folder),
                false => tell!(
                    "vidni: warning: {} holds no backup of this project",
                    place.display()
                ),
            }
        }
        if folders.is_empty() {
            return Err(cant("no backup of this project was found".to_string()));
        }
        // The list is the project's own, which is committed with it; a
        // project that lost it takes the newest backup's.
        let mut assets = assets::read_lock(dir).map_err(cant)?;
        if assets.is_empty() {
            for folder in &folders {
                assets = assets::read_lock(folder).map_err(cant)?;
                if !assets.is_empty() {
                    assets::write_lock(dir, &assets).map_err(cant)?;
                    say!(
                        "{}: taken from {}",
                        dir.join(assets::LOCK).display(),
                        folder.display()
                    );
                    break;
                }
            }
        }
        assets::restore(project, dir, &assets, &releases, &folders).map_err(cant)
    });
    let restored = match restored {
        Ok(restored) => restored,
        Err(code) => return code,
    };
    say!(
        "{}: {} restored, {} there already",
        dir.display(),
        count(restored.restored, "file"),
        restored.already
    );
    list(
        "left as it is, though it isn't the file that is listed",
        &restored.differing,
    );
    list("no backup holds it as listed", &restored.missing);
    match restored.differing.is_empty() && restored.missing.is_empty() {
        true => ExitCode::SUCCESS,
        false => ExitCode::FAILURE,
    }
}

/// A size for people: `12.3 MB`.
fn megabytes(bytes: u64) -> String {
    format!("{:.1} MB", bytes as f64 / 1e6)
}

pub fn gc(path: &Path, dry_run: bool) -> ExitCode {
    let loaded = match load(path) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let Loaded { dir, project, .. } = &loaded;
    let found = ratify::read_lock(dir)
        .and_then(|lock| assets::rebuildable(project, dir, &lock))
        .map_err(cant);
    let found = match found {
        Ok(found) => found,
        Err(code) => return code,
    };
    let mut kinds: BTreeMap<Rebuildable, (usize, u64)> = BTreeMap::new();
    for (kind, file, size) in &found {
        if !dry_run {
            let file = dir.join(file);
            if let Err(e) = std::fs::remove_file(&file) {
                return cant(format!("{}: {e}", file.display()));
            }
            // A folder that is empty now goes too, up to `build/`.
            let mut folder = file.parent();
            while let Some(empty) = folder.filter(|f| *f != dir.as_path()) {
                if std::fs::remove_dir(empty).is_err() {
                    break;
                }
                folder = empty.parent();
            }
        }
        let (files, bytes) = kinds.entry(*kind).or_default();
        *files += 1;
        *bytes += size;
    }
    let verb = if dry_run { "would delete" } else { "deleted" };
    for (kind, (files, bytes)) in &kinds {
        let (one, many, again) = match kind {
            Rebuildable::Voice => (
                "draft clip",
                "draft clips",
                "`vidni draft` speaks them again",
            ),
            Rebuildable::Draft => (
                "file of drafts",
                "files of drafts",
                "`vidni draft` builds them again",
            ),
            Rebuildable::Final => (
                "file of final builds",
                "files of final builds",
                "`vidni final` builds them again",
            ),
            Rebuildable::Capture => (
                "capture no cut shows",
                "captures no cut shows",
                "nothing needs them",
            ),
            Rebuildable::Leftover => (
                "file a session left behind",
                "files a session left behind",
                "nothing needs them",
            ),
        };
        let what = if *files == 1 { one } else { many };
        say!("{verb} {files} {what}, {} ({again})", megabytes(*bytes));
    }
    let total: u64 = kinds.values().map(|(_, bytes)| bytes).sum();
    say!(
        "{}: {verb} {}, {}; takes, frozen captures and delivered files are never touched",
        dir.display(),
        count(found.len(), "file"),
        megabytes(total)
    );
    ExitCode::SUCCESS
}
