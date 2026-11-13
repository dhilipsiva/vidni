//! Ratification, per cut.
//!
//! Ratifying a cut freezes what it shows and says: its lines, their
//! sections, the scenes they use, and its captures. `ratified.lock` keeps the
//! hash of each, taken from the resolved project (`build/resolved.json`),
//! and the SHA-256 of each capture file. A cut's status is never stored: it
//! is worked out from those hashes each time. A cut is `draft` until
//! ratified, `ratified` while every hash still holds, and `changed` once one
//! doesn't.
//!
//! Gaps, caption style, platforms and the theme aren't part of a cut's
//! content; only the release hashes them. Changing them leaves a cut ratified.

use crate::capture::{self, Capture, canonical, sha256_hex};
use crate::project::{Cut, Project};
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

/// Writes `build/resolved.json`.
pub fn write_resolved(project: &Project, project_dir: &Path) -> Result<PathBuf, String> {
    let path = project_dir.join(RESOLVED);
    let text = serde_json::to_string_pretty(&resolved(project)).expect("JSON serializes") + "\n";
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
        sections.insert(line.section.clone(), r["sections"][&line.section].clone());
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
    let frozen = json!({
        "cut": {"id": cut.id, "aspect": cut.aspect, "size": cut.size(), "lines": cut.line_ids(project)},
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
        .map(|cut| {
            let Some(locked) = lock.cuts.get(&cut.id) else {
                return (cut.id.clone(), Status::Draft);
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
                    reasons.push("its sections, size or line order changed".into());
                }
            }
            for (key, sha) in &locked.captures {
                match file_hash(&capture::path(project_dir, key)) {
                    Some(now) if now == *sha => {}
                    Some(_) => reasons.push(format!("capture {} changed on disk", &key[..12])),
                    None => reasons.push(format!("capture {} is missing", &key[..12])),
                }
            }
            let status = if reasons.is_empty() {
                Status::Ratified
            } else {
                Status::Changed(reasons)
            };
            (cut.id.clone(), status)
        })
        .collect()
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
        let missing: Vec<&Capture> = content
            .captures
            .iter()
            .filter(|c| !capture::path(project_dir, &c.key).is_file())
            .collect();
        if !missing.is_empty() {
            return Err(format!(
                "{}: {} of {} captures aren't taken yet (the first: {}); draft the cut first",
                cut.id,
                missing.len(),
                content.captures.len(),
                missing[0].what
            ));
        }
        let captures = content
            .captures
            .iter()
            .map(|c| {
                (
                    c.key.clone(),
                    file_hash(&capture::path(project_dir, &c.key)).unwrap_or_default(),
                )
            })
            .collect();
        lock.cuts.insert(
            cut.id.clone(),
            Locked {
                content: content.content.clone(),
                lines: content.lines.clone(),
                scenes: content.scenes.clone(),
                captures,
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
            deleted.push(path);
        }
    }
    Ok(deleted)
}
