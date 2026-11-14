//! Ratification, per cut.
//!
//! Ratifying a cut freezes what it shows and says: its lines, their
//! sections, the scenes they use, and its captures. `ratified.lock` keeps the
//! hash of each, taken from the resolved project (`build/resolved.json`),
//! and the SHA-256 of each capture file. A cut's status is never stored: it
//! is worked out from those hashes each time. A cut is `draft` until
//! ratified, `ratified` while every hash still holds, and `changed` once one
//! doesn't, or once a file of the project's that one of its captures was
//! made from has changed.
//!
//! Gaps, caption style, platforms and the theme aren't part of a cut's
//! content; only the release hashes them. Changing them leaves a cut ratified.

use crate::capture::{self, Capture, canonical, sha256_hex};
use crate::project::{Aspect, Cut, Project};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

/// The lock, at the project folder's top (committed with the project).
pub const LOCK: &str = "ratified.lock";
/// The resolved project, the only file the engine reads.
pub const RESOLVED: &str = "build/resolved.json";

const LOCK_HEADER: &str = "# Written by `vidni ratify` and `vidni unratify`; don't edit it by hand.\n\
                           # Each cut's status is worked out from these hashes (docs/spec.md, Ratification).\n";

/// What `ratified.lock` holds: the ratified cuts.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Lock {
    #[serde(default)]
    pub cuts: BTreeMap<String, Locked>,
}

/// One ratified cut's hashes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Locked {
    /// Everything the cut shows and says.
    pub content: String,
    /// Each line, by ID.
    pub lines: BTreeMap<String, String>,
    /// Each scene the lines use, by ID.
    pub scenes: BTreeMap<String, String>,
    /// Each capture's file, by capture key.
    pub captures: BTreeMap<String, String>,
    /// Each web still's frames file, by capture key: the boxes its cameras
    /// and highlights are drawn from.
    #[serde(default)]
    pub frames: BTreeMap<String, String>,
}

/// A cut's status, worked out from the lock.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Draft,
    Ratified,
    /// Ratified once, but something it froze has changed since.
    Changed(Vec<String>),
}

/// The project with every default filled in: each line's scene, and each
/// cut's lines, captions, platforms and size.
pub fn resolved(project: &Project) -> Value {
    let mut v = serde_json::to_value(project).expect("a project serializes");
    for (i, line) in project.lines.iter().enumerate() {
        v["lines"][i]["scene"] = json!(line.scene_in(project));
    }
    for (c, cut) in project.cuts.iter().enumerate() {
        let r = &mut v["cuts"][c];
        r["lines"] = json!(cut.line_ids(project));
        r["captions"] = json!(cut.captions());
        r["platforms"] = json!(cut.platforms());
        r["size"] = json!(cut.size());
    }
    v
}

/// Writes `build/resolved.json`, unless it holds the same already.
pub fn write_resolved(project: &Project, project_dir: &Path) -> Result<PathBuf, String> {
    let path = project_dir.join(RESOLVED);
    let text = serde_json::to_string_pretty(&resolved(project)).expect("JSON serializes") + "\n";
    if fs::read_to_string(&path).is_ok_and(|held| held == text) {
        return Ok(path);
    }
    fs::create_dir_all(path.parent().expect("a parent folder"))
        .and_then(|_| fs::write(&path, text))
        .map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(path)
}

/// What ratifying `cut` would freeze.
#[derive(Debug, Clone, PartialEq)]
pub struct Content {
    pub content: String,
    pub lines: BTreeMap<String, String>,
    pub scenes: BTreeMap<String, String>,
    pub captures: Vec<Capture>,
}

fn hash(value: &Value) -> String {
    sha256_hex(canonical(value).as_bytes())
}

/// The hashes of everything `cut` shows and says.
pub fn content(project: &Project, cut: &Cut) -> Content {
    let r = resolved(project);
    let ids: BTreeSet<String> = cut.line_ids(project).into_iter().collect();
    let (mut lines, mut line_values) = (BTreeMap::new(), Vec::new());
    let (mut scenes, mut scene_values) = (BTreeMap::new(), BTreeMap::new());
    let mut sections = BTreeMap::new();
    for (i, line) in project
        .lines
        .iter()
        .enumerate()
        .filter(|(_, l)| ids.contains(&l.id))
    {
        let value = r["lines"][i].clone();
        lines.insert(line.id.clone(), hash(&value));
        line_values.push(value);
        // A section's label is shown; its chapter title isn't, so it is
        // left to the release's hash, with the title of the video.
        let label = json!({ "label": r["sections"][&line.section]["label"] });
        sections.insert(line.section.clone(), label);
        for id in line
            .scene_in(project)
            .into_iter()
            .chain(line.overlay.as_deref())
        {
            if let Some(scene) = r["scenes"].get(id) {
                scenes.insert(id.to_string(), hash(scene));
                scene_values.insert(id.to_string(), scene.clone());
            }
        }
    }
    let captures = capture::for_cut(project, cut);
    let keys: Vec<&str> = captures.iter().map(|c| c.key.as_str()).collect();
    // The title band of a 9:16 cut shows the project's subtitle, so it is
    // part of what that cut shows.
    let subtitle = match cut.aspect {
        Aspect::Portrait => project.meta.subtitle.as_deref(),
        Aspect::Landscape => None,
    };
    let frozen = json!({
        "cut": {
            "id": cut.id,
            "aspect": cut.aspect,
            "size": cut.size(),
            "lines": cut.line_ids(project),
            "subtitle": subtitle,
        },
        "lines": line_values,
        "sections": sections,
        "scenes": scene_values,
        "captures": keys,
    });
    Content {
        content: hash(&frozen),
        lines,
        scenes,
        captures,
    }
}

/// Reads `ratified.lock`; a project without one has ratified nothing.
pub fn read_lock(project_dir: &Path) -> Result<Lock, String> {
    let path = project_dir.join(LOCK);
    match fs::read_to_string(&path) {
        Ok(text) => {
            toml::from_str(&text).map_err(|e| format!("{}: {}", path.display(), e.message()))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Lock::default()),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}

/// Writes `ratified.lock`.
pub fn write_lock(project_dir: &Path, lock: &Lock) -> Result<(), String> {
    let path = project_dir.join(LOCK);
    let body = toml::to_string(lock).map_err(|e| e.to_string())?;
    fs::write(&path, format!("{LOCK_HEADER}\n{body}"))
        .map_err(|e| format!("{}: {e}", path.display()))
}

fn file_hash(path: &Path) -> Option<String> {
    fs::read(path).ok().map(|bytes| sha256_hex(&bytes))
}

/// Each cut's status, in the project's order.
pub fn status(project: &Project, project_dir: &Path, lock: &Lock) -> Vec<(String, Status)> {
    project
        .cuts
        .iter()
        .map(|cut| (cut.id.clone(), status_of(project, project_dir, lock, cut)))
        .collect()
}

/// One cut's status.
pub fn status_of(project: &Project, project_dir: &Path, lock: &Lock, cut: &Cut) -> Status {
    let Some(locked) = lock.cuts.get(&cut.id) else {
        return Status::Draft;
    };
    let now = content(project, cut);
    let mut reasons = Vec::new();
    if now.content != locked.content {
        differences("line", &locked.lines, &now.lines, &mut reasons);
        differences("scene", &locked.scenes, &now.scenes, &mut reasons);
        let keys: BTreeSet<&String> = now.captures.iter().map(|c| &c.key).collect();
        if keys != locked.captures.keys().collect() {
            reasons.push("its captures changed".into());
        }
        if reasons.is_empty() {
            reasons.push("its sections, subtitle, size or line order changed".into());
        }
    }
    // A still's frames are frozen with it. A lock that has the still and
    // not its frames was written before they were, and can't vouch for them.
    let unfrozen = now
        .captures
        .iter()
        .filter(|c| matches!(c.source, capture::Source::Line(_)))
        .filter(|c| locked.captures.contains_key(&c.key) && !locked.frames.contains_key(&c.key))
        .count();
    if unfrozen > 0 {
        reasons.push(format!(
            "the frames of {unfrozen} of its stills were never frozen; ratify it again"
        ));
    }
    // Each frozen file, checked on disk. A few are named; many are counted,
    // as when the captures were never restored from a backup.
    let files = [
        (
            "capture",
            "captures",
            &locked.captures,
            capture::path as fn(&Path, &str) -> PathBuf,
        ),
        (
            "the frames of capture",
            "stills' frames",
            &locked.frames,
            capture::frames_path,
        ),
    ];
    for (one, many, frozen, path) in files {
        let (mut changed, mut missing) = (Vec::new(), Vec::new());
        for (key, sha) in frozen {
            // A key is 64 hex digits; one from a lock edited by hand may not be.
            let short = key.get(..12).unwrap_or(key);
            match file_hash(&path(project_dir, key)) {
                Some(now) if now == *sha => {}
                Some(_) => changed.push(short),
                None => missing.push(short),
            }
        }
        for (keys, what) in [(changed, "changed on disk"), (missing, "went missing")] {
            match keys.len() {
                0..=3 => reasons.extend(keys.iter().map(|key| format!("{one} {key} {what}"))),
                n => reasons.push(format!("{n} of its {} {many} {what}", frozen.len())),
            }
        }
    }
    // A capture of one of the project's own pages shows its files as they
    // were. Once one of them changes, the cut no longer shows the project.
    let (mut unrecorded, mut stale) = (0, BTreeMap::new());
    let mut hashes = capture::Hashes::default();
    for c in now.captures.iter().filter(|c| !c.on_the_web()) {
        if capture::path(project_dir, &c.key).is_file()
            && capture::files(project_dir, &c.key).is_none()
        {
            unrecorded += 1;
        }
        for file in capture::changed_in(project_dir, c, &mut hashes) {
            *stale.entry(file).or_insert(0) += 1;
        }
    }
    if unrecorded > 0 {
        reasons.push(format!(
            "{unrecorded} of its captures {} no record of the files {} made from; draft it again",
            if unrecorded == 1 { "has" } else { "have" },
            if unrecorded == 1 {
                "it was"
            } else {
                "they were"
            },
        ));
    }
    for (file, captures) in stale {
        reasons.push(format!(
            "{file} has changed since {captures} of its captures {} taken",
            if captures == 1 { "was" } else { "were" },
        ));
    }
    if reasons.is_empty() {
        Status::Ratified
    } else {
        Status::Changed(reasons)
    }
}

fn differences(
    what: &str,
    locked: &BTreeMap<String, String>,
    now: &BTreeMap<String, String>,
    reasons: &mut Vec<String>,
) {
    for (id, hash) in now {
        match locked.get(id) {
            None => reasons.push(format!("{what} {id} was added")),
            Some(old) if old != hash => reasons.push(format!("{what} {id} changed")),
            Some(_) => {}
        }
    }
    for id in locked.keys().filter(|id| !now.contains_key(*id)) {
        reasons.push(format!("{what} {id} was removed"));
    }
}

/// The captures in `plan` that mustn't be taken again: stills whose
/// picture is still the one a ratified cut froze, and whose frames are
/// missing or unreadable. Taking one again would put another picture in
/// the place of a frozen one, which a page that has changed since can't
/// give back. Each is a refusal that says what to do instead.
///
/// A still of one of the project's own pages that is out of date is taken
/// again all the same: its files are what changed, and they are the
/// creator's to change back.
pub fn frozen(project_dir: &Path, lock: &Lock, plan: &[Capture]) -> Vec<String> {
    let mut refusals = Vec::new();
    for c in plan
        .iter()
        .filter(|c| matches!(c.source, capture::Source::Line(_)))
        .filter(|c| !capture::measured(&capture::frames_path(project_dir, &c.key)))
    {
        let Some(picture) = file_hash(&capture::path(project_dir, &c.key)) else {
            continue;
        };
        let cuts: Vec<&str> = lock
            .cuts
            .iter()
            .filter(|(_, locked)| locked.captures.get(&c.key) == Some(&picture))
            .map(|(id, _)| id.as_str())
            .collect();
        if let Some(first) = cuts.first() {
            refusals.push(format!(
                "the capture of {} is frozen by cut {}, and its frames can't be read; restore {}/{}.json from a backup, or run `vidni unratify --cut {first} --refresh` to take it afresh",
                c.what,
                cuts.join(" and "),
                capture::DIR,
                c.key
            ));
        }
    }
    refusals
}

/// The cuts named, or every cut when none is.
fn chosen<'p>(project: &'p Project, ids: &[String]) -> Result<Vec<&'p Cut>, String> {
    if ids.is_empty() {
        return Ok(project.cuts.iter().collect());
    }
    ids.iter()
        .map(|id| {
            project
                .cuts
                .iter()
                .find(|c| &c.id == id)
                .ok_or_else(|| format!("there is no cut {id:?}"))
        })
        .collect()
}

/// Ratifies the cuts named (all when none is): every capture they need must
/// be in the project folder. Returns the cuts ratified, with their hashes.
pub fn ratify(
    project: &Project,
    project_dir: &Path,
    lock: &mut Lock,
    ids: &[String],
) -> Result<Vec<(String, Content)>, String> {
    let cuts = chosen(project, ids)?;
    let mut done = Vec::new();
    for cut in cuts {
        let content = content(project, cut);
        let unready = capture::refusals(project_dir, &content.captures);
        if !unready.is_empty() {
            return Err(format!(
                "{}: {}; draft the cut first",
                cut.id,
                unready.join("; ")
            ));
        }
        let hashes = |path: fn(&Path, &str) -> PathBuf, web_only: bool| {
            content
                .captures
                .iter()
                .filter(|c| !web_only || matches!(c.source, capture::Source::Line(_)))
                .map(|c| {
                    let sha = file_hash(&path(project_dir, &c.key)).unwrap_or_default();
                    (c.key.clone(), sha)
                })
                .collect()
        };
        lock.cuts.insert(
            cut.id.clone(),
            Locked {
                content: content.content.clone(),
                lines: content.lines.clone(),
                scenes: content.scenes.clone(),
                captures: hashes(capture::path, false),
                frames: hashes(capture::frames_path, true),
            },
        );
        done.push((cut.id.clone(), content));
    }
    Ok(done)
}

/// Unratifies the cuts named (all when none is). Their captures stay, so the
/// next draft takes none again unless a scene changed; with `refresh`, the
/// captures no other ratified cut uses are deleted, to be taken afresh.
/// Returns the captures deleted.
pub fn unratify(
    project: &Project,
    project_dir: &Path,
    lock: &mut Lock,
    ids: &[String],
    refresh: bool,
) -> Result<Vec<PathBuf>, String> {
    let cuts = chosen(project, ids)?;
    for cut in &cuts {
        lock.cuts.remove(&cut.id);
    }
    if !refresh {
        return Ok(Vec::new());
    }
    let kept: BTreeSet<String> = project
        .cuts
        .iter()
        .filter(|c| lock.cuts.contains_key(&c.id))
        .flat_map(|c| capture::for_cut(project, c))
        .map(|c| c.key)
        .collect();
    let mut deleted = Vec::new();
    for c in cuts.iter().flat_map(|cut| capture::for_cut(project, cut)) {
        let path = capture::path(project_dir, &c.key);
        if !kept.contains(&c.key) && path.is_file() {
            fs::remove_file(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            // A still's frames go with it, and the record of its files.
            for beside in [
                capture::frames_path(project_dir, &c.key),
                capture::files_path(project_dir, &c.key),
            ] {
                if beside.is_file() {
                    fs::remove_file(&beside).map_err(|e| format!("{}: {e}", beside.display()))?;
                }
            }
            deleted.push(path);
        }
    }
    Ok(deleted)
}
