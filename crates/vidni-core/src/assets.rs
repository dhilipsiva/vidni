//! A project's large assets, and their backups (DECISIONS.md, D4; TODO.md,
//! P4-VIDNI-05).
//!
//! The assets are what nothing can make again: the creator's takes, and
//! the captures a ratified cut froze. They live in the project folder, out
//! of git, and `assets.lock`, which is committed, lists each with its
//! SHA-256, as `sha256sum` writes and checks such a list.
//!
//! A backup is a copy of them in another place: a folder named as the
//! project's folder is, under a root such as another disk or a folder that
//! is mounted from another machine. It holds the assets at their paths,
//! `assets.lock`, and each release's delivered files under
//! `releases/<tag>/`, which are archived because a cut's file isn't
//! promised to come out the same on another machine (docs/determinism.md,
//! T3). A backup only ever gains files: nothing in it is deleted.
//!
//! Everything else vidni makes can be made again, and `rebuildable` lists
//! it for `vidni gc`.

use crate::capture;
use crate::project::Project;
use crate::ratify;
use crate::release::{self, Release, Releases};
use crate::takes;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

/// The list of assets, at the project folder's top (committed).
pub const LOCK: &str = "assets.lock";
/// Where a backup keeps each release's delivered files.
pub const RELEASES: &str = "releases";

/// A file and what it holds.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Asset {
    /// Its path from the project folder, with `/` between its parts.
    pub path: String,
    pub sha256: String,
}

/// The SHA-256 of a file, read a piece at a time.
pub fn sha256_of(path: &Path) -> Result<String, String> {
    let shown = |e: std::io::Error| format!("{}: {e}", path.display());
    let mut file = File::open(path).map_err(shown)?;
    let mut hash = Sha256::new();
    let mut piece = vec![0u8; 1 << 16];
    loop {
        match file.read(&mut piece).map_err(shown)? {
            0 => break,
            n => hash.update(&piece[..n]),
        }
    }
    Ok(hash.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

/// Every file under `folder`, as paths from `base`, in no order.
fn files_under(base: &Path, folder: &Path, found: &mut Vec<String>) -> Result<(), String> {
    let entries = match fs::read_dir(folder) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(format!("{}: {e}", folder.display())),
    };
    for entry in entries {
        let path = entry
            .map_err(|e| format!("{}: {e}", folder.display()))?
            .path();
        if path.is_dir() {
            files_under(base, &path, found)?;
        } else if path.is_file() {
            found.push(release::relative(base, &path));
        }
    }
    Ok(())
}

/// Whether a file among the takes is one a session left behind: a trimmed
/// take that was never kept, or a recording of the mic check.
fn left_behind(path: &str) -> bool {
    [".new", ".raw", ".part"]
        .iter()
        .any(|ending| path.ends_with(ending))
}

/// The paths of the project's assets, sorted: every take, raw takes and
/// room tone among them, and every capture file a ratified cut froze.
pub fn paths(project_dir: &Path, lock: &ratify::Lock) -> Result<Vec<String>, String> {
    let mut found = Vec::new();
    files_under(project_dir, &project_dir.join(takes::DIR), &mut found)?;
    found.retain(|path| !left_behind(path));
    let mut paths: BTreeSet<String> = found.into_iter().collect();
    for locked in lock.cuts.values() {
        for key in locked.captures.keys() {
            paths.insert(release::relative(
                project_dir,
                &capture::path(project_dir, key),
            ));
            // The record of a project page's files goes with its capture.
            let files = capture::files_path(project_dir, key);
            if files.is_file() {
                paths.insert(release::relative(project_dir, &files));
            }
        }
        for key in locked.frames.keys() {
            paths.insert(release::relative(
                project_dir,
                &capture::frames_path(project_dir, key),
            ));
        }
    }
    Ok(paths.into_iter().collect())
}

/// The project's assets as they are now, each with its hash. An asset that
/// isn't there, as a frozen capture that went missing, is an error: it
/// can't be listed, let alone backed up.
pub fn snapshot(project_dir: &Path, lock: &ratify::Lock) -> Result<Vec<Asset>, String> {
    paths(project_dir, lock)?
        .into_iter()
        .map(|path| {
            let sha256 = sha256_of(&project_dir.join(&path))?;
            Ok(Asset { path, sha256 })
        })
        .collect()
}

/// `assets` as `assets.lock` holds them: a line for each, as `sha256sum`
/// writes it, sorted by path.
pub fn lock_text(assets: &[Asset]) -> String {
    let mut sorted: Vec<&Asset> = assets.iter().collect();
    sorted.sort_by(|a, b| a.path.cmp(&b.path));
    sorted
        .iter()
        .map(|asset| format!("{}  {}\n", asset.sha256, asset.path))
        .collect()
}

/// Reads an `assets.lock`.
fn parse_lock(path: &Path) -> Result<Vec<Asset>, String> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(format!("{}: {e}", path.display())),
    };
    text.lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(n, line)| {
            // `<64 hex digits>  <path>`; sha256sum writes ` *` for a file
            // read as binary.
            let asset = line.split_once("  ").or_else(|| line.split_once(" *"));
            match asset {
                Some((sha256, path))
                    if sha256.len() == 64
                        && sha256.chars().all(|c| c.is_ascii_hexdigit())
                        && inside(path) =>
                {
                    Ok(Asset {
                        path: path.to_string(),
                        sha256: sha256.to_ascii_lowercase(),
                    })
                }
                _ => Err(format!(
                    "{}:{}: not a line of a list of assets: a SHA-256, two spaces, and a path in the project folder",
                    path.display(),
                    n + 1
                )),
            }
        })
        .collect()
}

/// Whether `path` names a file inside the folder it is read from.
fn inside(path: &str) -> bool {
    use std::path::Component;
    !path.is_empty()
        && Path::new(path)
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
}

/// The assets `assets.lock` lists; none when the project has no such file.
pub fn read_lock(project_dir: &Path) -> Result<Vec<Asset>, String> {
    parse_lock(&project_dir.join(LOCK))
}

/// Writes `assets.lock` unless it holds the same already; true when it
/// was written.
pub fn write_lock(folder: &Path, assets: &[Asset]) -> Result<bool, String> {
    let path = folder.join(LOCK);
    let text = lock_text(assets);
    if fs::read_to_string(&path).is_ok_and(|held| held == text) {
        return Ok(false);
    }
    fs::create_dir_all(folder)
        .and_then(|()| fs::write(&path, text))
        .map(|()| true)
        .map_err(|e| format!("{}: {e}", path.display()))
}

/// How a folder's copy of a file stands against what the file should hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Same,
    Missing,
    /// It is there, and holds something else.
    Changed,
}

/// How `folder`'s copy of `asset` stands.
pub fn state(folder: &Path, asset: &Asset) -> State {
    let path = folder.join(&asset.path);
    if !path.is_file() {
        return State::Missing;
    }
    match sha256_of(&path) {
        Ok(sha256) if sha256 == asset.sha256 => State::Same,
        _ => State::Changed,
    }
}

/// The assets of `assets` that `folder` doesn't hold as listed, each with
/// how it stands.
pub fn faults(folder: &Path, assets: &[Asset]) -> Vec<(String, State)> {
    assets
        .iter()
        .map(|asset| (asset.path.clone(), state(folder, asset)))
        .filter(|(_, state)| *state != State::Same)
        .collect()
}

/// Copies the file at `from`, which should hold `sha256`, to `to`, unless
/// `to` holds it already. The copy is written aside, checked, and moved
/// in, so that a copy cut short never stands for the file. True when it
/// was copied.
pub fn put(from: &Path, to: &Path, sha256: &str) -> Result<bool, String> {
    if to.is_file() && sha256_of(to).is_ok_and(|held| held == sha256) {
        return Ok(false);
    }
    let folder = to.parent().ok_or("a file has a folder")?;
    fs::create_dir_all(folder).map_err(|e| format!("{}: {e}", folder.display()))?;
    let mut name = to.file_name().unwrap_or_default().to_os_string();
    name.push(".part");
    let part = to.with_file_name(name);
    fs::copy(from, &part).map_err(|e| format!("{}: {e}", to.display()))?;
    let copied = sha256_of(&part)?;
    if copied != sha256 {
        let _ = fs::remove_file(&part);
        return Err(format!(
            "{} isn't the file that was listed: it has changed since",
            from.display()
        ));
    }
    fs::rename(&part, to).map_err(|e| format!("{}: {e}", to.display()))?;
    Ok(true)
}

/// A backup's folder under `root`: named as the project's folder is.
pub fn folder(root: &Path, project_dir: &Path) -> Result<PathBuf, String> {
    let dir =
        fs::canonicalize(project_dir).map_err(|e| format!("{}: {e}", project_dir.display()))?;
    let name = dir
        .file_name()
        .ok_or_else(|| format!("{} has no name to file its backup under", dir.display()))?;
    Ok(root.join(name))
}

/// The files a release delivered, as they are named in `out/`: each
/// picture, and the captions in the project's language.
pub fn delivered(project: &Project, release: &Release) -> Vec<Asset> {
    let build = &release.build;
    let stem = format!("{}.{}", build.cut, project.meta.language);
    let mut files: Vec<Asset> = build
        .outputs
        .iter()
        .map(|(name, output)| Asset {
            path: name.clone(),
            sha256: output.mp4.clone(),
        })
        .collect();
    files.push(Asset {
        path: format!("{stem}.srt"),
        sha256: build.t1.srt.clone(),
    });
    files.push(Asset {
        path: format!("{stem}.vtt"),
        sha256: build.t1.vtt.clone(),
    });
    files
}

/// What a backup to one place did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Backed {
    /// The backup's folder.
    pub folder: PathBuf,
    /// How many assets it holds now, and how many of them were copied.
    pub assets: usize,
    pub copied: usize,
    /// The delivered files of releases it holds now, and how many of them
    /// were copied.
    pub delivered: usize,
    pub delivered_copied: usize,
}

/// Backs up `assets`, the project's assets as `assets.lock` lists them, and
/// the delivered files of each cut's newest release, into the backup's
/// `folder`. A delivered file is archived when `out/` holds the very file
/// the release records.
pub fn backup(
    project: &Project,
    project_dir: &Path,
    assets: &[Asset],
    releases: &Releases,
    folder: &Path,
) -> Result<Backed, String> {
    let mut backed = Backed {
        folder: folder.to_path_buf(),
        assets: assets.len(),
        copied: 0,
        delivered: 0,
        delivered_copied: 0,
    };
    for asset in assets {
        let copied = put(
            &project_dir.join(&asset.path),
            &folder.join(&asset.path),
            &asset.sha256,
        )?;
        backed.copied += usize::from(copied);
    }
    for release in releases.cuts.values().filter_map(|of_cut| of_cut.last()) {
        let archive = folder.join(RELEASES).join(&release.tag);
        for file in delivered(project, release) {
            let out = project_dir.join(release::OUT).join(&file.path);
            if state(&project_dir.join(release::OUT), &file) != State::Same {
                continue;
            }
            let copied = put(&out, &archive.join(&file.path), &file.sha256)?;
            backed.delivered += 1;
            backed.delivered_copied += usize::from(copied);
        }
    }
    // The list goes in last: a backup that was cut short keeps the list
    // of the one before.
    write_lock(folder, assets)?;
    Ok(backed)
}

/// What a restore did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Restored {
    /// How many files were copied back, and how many were there already.
    pub restored: usize,
    pub already: usize,
    /// Files that are there and hold something else: left as they are.
    pub differing: Vec<String>,
    /// Files that no backup holds as listed.
    pub missing: Vec<String>,
}

/// Brings the file that should hold `sha256` back to `to`, when it isn't
/// there, from the first of `copies` that holds it. `name` is what the
/// file is called in the report.
fn bring_back(
    name: String,
    sha256: &str,
    to: &Path,
    copies: &[PathBuf],
    restored: &mut Restored,
) -> Result<(), String> {
    if to.is_file() {
        match sha256_of(to)? == sha256 {
            true => restored.already += 1,
            false => restored.differing.push(name),
        }
        return Ok(());
    }
    let holder = copies
        .iter()
        .find(|copy| sha256_of(copy).is_ok_and(|held| held == sha256));
    match holder {
        Some(copy) => {
            put(copy, to, sha256)?;
            restored.restored += 1;
        }
        None => restored.missing.push(name),
    }
    Ok(())
}

/// Restores the project's assets, as `assets.lock` lists them, from the
/// backups in `folders`, and the delivered files of each cut's newest
/// release into `out/`. Only what is missing is copied: a file that is
/// there and holds something else is left as it is, and reported.
pub fn restore(
    project: &Project,
    project_dir: &Path,
    assets: &[Asset],
    releases: &Releases,
    folders: &[PathBuf],
) -> Result<Restored, String> {
    let mut restored = Restored::default();
    for asset in assets {
        let copies: Vec<PathBuf> = folders.iter().map(|f| f.join(&asset.path)).collect();
        let to = project_dir.join(&asset.path);
        bring_back(
            asset.path.clone(),
            &asset.sha256,
            &to,
            &copies,
            &mut restored,
        )?;
    }
    for release in releases.cuts.values().filter_map(|of_cut| of_cut.last()) {
        for file in delivered(project, release) {
            let copies: Vec<PathBuf> = folders
                .iter()
                .map(|f| f.join(RELEASES).join(&release.tag).join(&file.path))
                .collect();
            // A release that was never archived has nothing to bring back.
            if !copies.iter().any(|copy| copy.is_file()) {
                continue;
            }
            let to = project_dir.join(release::OUT).join(&file.path);
            let name = format!("{}/{}", release::OUT, file.path);
            bring_back(name, &file.sha256, &to, &copies, &mut restored)?;
        }
    }
    Ok(restored)
}

/// What kind of file can be made again.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rebuildable {
    /// A draft clip: `vidni draft` speaks it again from `voice.lock`.
    Voice,
    /// A file of a draft: `vidni draft` builds it again.
    Draft,
    /// A file of a final build: `vidni final` builds it again.
    Final,
    /// A capture no cut shows any more, and no ratified cut froze.
    Capture,
    /// What a session left behind: a file written aside and never moved in.
    Leftover,
}

/// The files of the project that can be made again, each with its kind
/// and size, sorted. Takes, the captures a ratified cut froze, the
/// captures the cuts show now and the delivered files are never among
/// them.
pub fn rebuildable(
    project: &Project,
    project_dir: &Path,
    lock: &ratify::Lock,
) -> Result<Vec<(Rebuildable, String, u64)>, String> {
    let mut found: Vec<(Rebuildable, String)> = Vec::new();
    for (kind, folder) in [
        (Rebuildable::Voice, crate::voices::CACHE),
        (Rebuildable::Draft, release::Voice::Draft.dir()),
        (Rebuildable::Final, release::Voice::Takes.dir()),
    ] {
        let mut files = Vec::new();
        files_under(project_dir, &project_dir.join(folder), &mut files)?;
        found.extend(files.into_iter().map(|path| (kind, path)));
    }
    // A capture stays when a cut shows it or a ratified cut froze it.
    let mut kept: BTreeSet<String> = paths(project_dir, lock)?.into_iter().collect();
    for capture in project
        .cuts
        .iter()
        .flat_map(|cut| capture::for_cut(project, cut))
    {
        for file in [
            capture::path(project_dir, &capture.key),
            capture::frames_path(project_dir, &capture.key),
            capture::files_path(project_dir, &capture.key),
        ] {
            kept.insert(release::relative(project_dir, &file));
        }
    }
    let mut captures = Vec::new();
    files_under(project_dir, &project_dir.join(capture::DIR), &mut captures)?;
    for path in captures.into_iter().filter(|path| !kept.contains(path)) {
        let kind = match left_behind(&path) {
            true => Rebuildable::Leftover,
            false => Rebuildable::Capture,
        };
        found.push((kind, path));
    }
    let mut among_takes = Vec::new();
    files_under(project_dir, &project_dir.join(takes::DIR), &mut among_takes)?;
    found.extend(
        among_takes
            .into_iter()
            .filter(|path| left_behind(path))
            .map(|path| (Rebuildable::Leftover, path)),
    );
    found.sort();
    Ok(found
        .into_iter()
        .map(|(kind, path)| {
            let size = fs::metadata(project_dir.join(&path)).map_or(0, |meta| meta.len());
            (kind, path, size)
        })
        .collect())
}

/// The hash of nothing, for a test of the hashing itself.
#[cfg(test)]
const EMPTY: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::sha256_hex;

    #[test]
    fn a_files_hash_is_the_hash_of_its_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("a");
        fs::write(&file, b"").unwrap();
        assert_eq!(sha256_of(&file).unwrap(), EMPTY);
        // Longer than one piece, so that the pieces are joined.
        let long: Vec<u8> = (0..200_000u32).map(|n| n as u8).collect();
        fs::write(&file, &long).unwrap();
        assert_eq!(sha256_of(&file).unwrap(), sha256_hex(&long));
        assert!(sha256_of(&dir.path().join("gone")).is_err());
    }

    #[test]
    fn the_list_is_what_sha256sum_writes_and_reads_back() {
        let assets = vec![
            Asset {
                path: "takes/b/1.wav".into(),
                sha256: "b".repeat(64),
            },
            Asset {
                path: "build/captures/a b.png".into(),
                sha256: "a".repeat(64),
            },
        ];
        let text = lock_text(&assets);
        assert_eq!(
            text,
            format!(
                "{}  build/captures/a b.png\n{}  takes/b/1.wav\n",
                "a".repeat(64),
                "b".repeat(64)
            )
        );
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(read_lock(dir.path()).unwrap(), []);
        assert!(write_lock(dir.path(), &assets).unwrap());
        assert!(!write_lock(dir.path(), &assets).unwrap(), "written once");
        let mut sorted = assets.clone();
        sorted.sort();
        assert_eq!(read_lock(dir.path()).unwrap(), sorted);
        // A line that isn't a hash and a path inside the folder is refused,
        // by its number.
        for bad in [
            "not a hash  takes/a.wav",
            &format!("{}  ../outside.wav", "a".repeat(64)),
            &format!("{}  /etc/hostname", "a".repeat(64)),
            &format!("{} takes/a.wav", "a".repeat(64)),
        ] {
            fs::write(dir.path().join(LOCK), format!("{text}{bad}\n")).unwrap();
            let error = read_lock(dir.path()).unwrap_err();
            assert!(
                error.contains("assets.lock:3: not a line"),
                "{bad}: {error}"
            );
        }
    }

    #[test]
    fn a_copy_is_checked_before_it_stands_for_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let (from, to) = (dir.path().join("from.wav"), dir.path().join("to/copy.wav"));
        fs::write(&from, b"a take").unwrap();
        let sha256 = sha256_hex(b"a take");
        assert!(put(&from, &to, &sha256).unwrap());
        assert_eq!(fs::read(&to).unwrap(), b"a take");
        assert!(!put(&from, &to, &sha256).unwrap(), "it is there already");
        // A copy that holds something else is replaced.
        fs::write(&to, b"something else").unwrap();
        assert!(put(&from, &to, &sha256).unwrap());
        assert_eq!(fs::read(&to).unwrap(), b"a take");
        // A source that isn't the listed file is refused, and leaves
        // nothing in the copy's place.
        let other = dir.path().join("to/other.wav");
        let error = put(&from, &other, &"0".repeat(64)).unwrap_err();
        assert!(error.ends_with("from.wav isn't the file that was listed: it has changed since"));
        assert_eq!(fs::read_dir(dir.path().join("to")).unwrap().count(), 1);
    }
}
