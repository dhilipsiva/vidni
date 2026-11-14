//! Captures: the stills of web scenes and the renders of page scenes, filed
//! by what they show, so that an unchanged scene is never captured again.
//!
//! A web line's capture is keyed by its scene (all of it but the cameras,
//! which only frame the still), every action that has run on that page by
//! then (the scene's setup, then each earlier line's actions in script
//! order, then its own), the expectations capture has waited for by then,
//! and its highlights. A page scene's render is keyed
//! by the scene, the size it is rendered at, and the cut's aspect, which
//! `{aspect}` in its address becomes.
//!
//! A key covers what the project file says, not what a page holds. So a
//! capture of one of the project's own pages comes with a record of the
//! files the page loaded, each with its SHA-256 (`<key>.files.json`), and
//! the capture is out of date once one of them has changed (TODO.md,
//! P2-VIDNI-09). A page on the web has no such record: whether it has
//! changed can't be told without loading it, so its capture is kept until
//! it is deleted.

use crate::project::{Aspect, Cut, Project, Scene, SceneKind};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

/// Where captures live, relative to the project folder (out of git).
pub const DIR: &str = "build/captures";

/// A capture a cut needs.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Capture {
    pub key: String,
    /// What it shows, for people: `line <id>` or `page <scene> at <w>x<h>`.
    pub what: String,
    pub source: Source,
    /// The file in the project folder that its page is, with `{aspect}`
    /// filled in. None for a page on the web.
    pub file: Option<String>,
}

impl Capture {
    /// Whether the capture is of a page on the web, which needs the
    /// network, rather than of a file in the project folder.
    pub fn on_the_web(&self) -> bool {
        self.file.is_none()
    }
}

/// What a capture is taken of.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Source {
    /// A web line's still, by line ID.
    Line(String),
    /// A page scene rendered at a size, in pixels, for a cut's aspect:
    /// `16x9` or `9x16`, which `{aspect}` in its address becomes.
    Page {
        scene: String,
        size: [u32; 2],
        aspect: &'static str,
    },
}

/// The file of the capture with `key`.
pub fn path(project_dir: &Path, key: &str) -> PathBuf {
    project_dir.join(DIR).join(format!("{key}.png"))
}

/// The file of a web still's frames: the boxes of its scene's targets and
/// of its highlights, which a cut's cameras are framed from.
pub fn frames_path(project_dir: &Path, key: &str) -> PathBuf {
    project_dir.join(DIR).join(format!("{key}.json"))
}

/// The file of the record of the project's files that a capture's page
/// loaded.
pub fn files_path(project_dir: &Path, key: &str) -> PathBuf {
    project_dir.join(DIR).join(format!("{key}.files.json"))
}

/// The project's files that a page loaded while it was captured: each by
/// its path from the project folder, with its SHA-256 then, or `None` for
/// a file that the page asked for and that wasn't there.
pub type Files = BTreeMap<String, Option<String>>;

/// A record of files as its file holds it.
pub fn files_json(files: &Files) -> String {
    serde_json::to_string_pretty(files).expect("a map of strings serializes") + "\n"
}

/// Reads a capture's record of its files. None when it has none, or one
/// that can't be read, or one that names a file outside the project
/// folder, which no capture writes.
pub fn files(project_dir: &Path, key: &str) -> Option<Files> {
    let text = std::fs::read_to_string(files_path(project_dir, key)).ok()?;
    let files: Files = serde_json::from_str(&text).ok()?;
    let inside = |name: &String| {
        !name.is_empty()
            && Path::new(name)
                .components()
                .all(|part| matches!(part, Component::Normal(_)))
    };
    files.keys().all(inside).then_some(files)
}

/// The SHA-256 of the file `name` in the project folder; none when it
/// isn't there or can't be read.
pub fn file_sha256(project_dir: &Path, name: &str) -> Option<String> {
    std::fs::read(project_dir.join(name))
        .ok()
        .map(|bytes| sha256_hex(&bytes))
}

/// JSON with every object's keys sorted and no spaces, so that equal values
/// always hash the same.
pub fn canonical(value: &Value) -> String {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let fields: Vec<String> = keys
                .into_iter()
                .map(|k| format!("{}:{}", Value::String(k.clone()), canonical(&map[k])))
                .collect();
            format!("{{{}}}", fields.join(","))
        }
        Value::Array(items) => format!(
            "[{}]",
            items.iter().map(canonical).collect::<Vec<_>>().join(",")
        ),
        other => other.to_string(),
    }
}

/// Lowercase hex SHA-256.
pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn hash(value: &Value) -> String {
    sha256_hex(canonical(value).as_bytes())
}

fn value_of<T: serde::Serialize>(item: &T) -> Value {
    serde_json::to_value(item).expect("project types serialize")
}

/// What a web scene's still depends on: everything but how it is framed.
fn web_spec(scene: &Scene) -> Value {
    let mut spec = value_of(scene);
    if let Some(map) = spec.as_object_mut() {
        for framing in ["cameras", "default_camera", "vertical_follows_highlights"] {
            map.remove(framing);
        }
    }
    spec
}

/// The capture key of every line on a web scene, by line ID.
pub fn line_keys(project: &Project) -> BTreeMap<String, String> {
    // What has happened on each scene's page so far: the actions run, and
    // the expectations waited for. A capture waits for an expectation
    // before it takes a still, so adding one can change the still.
    let mut done: BTreeMap<&str, (Vec<Value>, Vec<Value>)> = BTreeMap::new();
    let expected = |scene: &str, at: &str| -> Vec<Value> {
        project
            .expectations
            .iter()
            .filter(|x| x.scene == scene && x.at == at)
            .map(value_of)
            .collect()
    };
    let mut keys = BTreeMap::new();
    for line in &project.lines {
        let Some(id) = line.scene_in(project) else {
            continue;
        };
        let Some(scene) = project.scenes.get(id) else {
            continue;
        };
        // A picture is one capture, whichever of its lines shows it: the
        // picture itself, with the boxes of its targets.
        if scene.kind == SceneKind::Image {
            keys.insert(line.id.clone(), hash(&json!({ "image": web_spec(scene) })));
            continue;
        }
        if scene.kind != SceneKind::Web {
            continue;
        }
        let (steps, waits) = done
            .entry(id)
            .or_insert_with(|| (Vec::new(), expected(id, "setup")));
        steps.extend(line.actions.iter().map(value_of));
        waits.extend(expected(id, &line.id));
        let key = hash(&json!({
            "scene": web_spec(scene),
            "steps": steps,
            "expects": waits,
            "highlight": line.highlight,
        }));
        keys.insert(line.id.clone(), key);
    }
    keys
}

/// The size a page scene is rendered at in `cut`.
fn page_size(scene: &Scene, cut: &Cut) -> [u32; 2] {
    match (&scene.size, cut.aspect) {
        (Some(size), Aspect::Landscape) => size.landscape,
        (Some(size), Aspect::Portrait) => size.portrait,
        (None, _) => cut.size(),
    }
}

/// The render of the page scene `id` that `cut` uses, or `None` when `id`
/// isn't a page scene.
pub fn page_capture(project: &Project, cut: &Cut, id: &str) -> Option<Capture> {
    let scene = project
        .scenes
        .get(id)
        .filter(|s| s.kind == SceneKind::Page)?;
    let size = page_size(scene, cut);
    let aspect = match cut.aspect {
        Aspect::Landscape => "16x9",
        Aspect::Portrait => "9x16",
    };
    Some(Capture {
        key: hash(&json!({"scene": value_of(scene), "size": size, "aspect": aspect})),
        what: format!("page {id} at {}x{} for {aspect}", size[0], size[1]),
        source: Source::Page {
            scene: id.to_string(),
            size,
            aspect,
        },
        file: scene.file().map(|file| file.replace("{aspect}", aspect)),
    })
}

/// Every capture `cut` needs, each once, in script order.
pub fn for_cut(project: &Project, cut: &Cut) -> Vec<Capture> {
    let keys = line_keys(project);
    let ids: BTreeSet<String> = cut.line_ids(project).into_iter().collect();
    let mut seen = BTreeSet::new();
    let mut found = Vec::new();
    let mut add = |capture: Capture| {
        if seen.insert(capture.key.clone()) {
            found.push(capture);
        }
    };
    for line in project.lines.iter().filter(|l| ids.contains(&l.id)) {
        if let Some(key) = keys.get(&line.id) {
            let scene = line.scene_in(project).and_then(|id| project.scenes.get(id));
            add(Capture {
                key: key.clone(),
                what: format!("line {}", line.id),
                source: Source::Line(line.id.clone()),
                file: scene.and_then(Scene::file).map(str::to_string),
            });
        }
        for id in line
            .scene_in(project)
            .into_iter()
            .chain(line.overlay.as_deref())
        {
            if let Some(page) = page_capture(project, cut, id) {
                add(page);
            }
        }
    }
    found
}

/// Whether the project folder holds `capture`: its picture; for a web
/// still, frames that a cut can be framed from; and for one of the
/// project's own pages, the record of its files. A capture without them
/// can't be used, or can't be told to be out of date, so it counts as not
/// taken.
pub fn taken(project_dir: &Path, capture: &Capture) -> bool {
    path(project_dir, &capture.key).is_file()
        && match capture.source {
            Source::Line(_) => measured(&frames_path(project_dir, &capture.key)),
            Source::Page { .. } => true,
        }
        && (capture.on_the_web() || files(project_dir, &capture.key).is_some())
}

/// The SHA-256 of files in the project folder as they are now, each read
/// once, however many captures were made from it.
#[derive(Debug, Default)]
pub(crate) struct Hashes(BTreeMap<String, Option<String>>);

impl Hashes {
    fn of(&mut self, project_dir: &Path, name: &str) -> &Option<String> {
        self.0
            .entry(name.to_string())
            .or_insert_with(|| file_sha256(project_dir, name))
    }
}

/// The project's files that `capture` was made from and that aren't now
/// what they were then: one that has changed, one that is gone, or one
/// that is there now and wasn't. Empty for a capture with no record of its
/// files.
pub fn changed(project_dir: &Path, capture: &Capture) -> Vec<String> {
    changed_in(project_dir, capture, &mut Hashes::default())
}

/// [`changed`], with the files' hashes kept in `now` for the next capture.
pub(crate) fn changed_in(project_dir: &Path, capture: &Capture, now: &mut Hashes) -> Vec<String> {
    files(project_dir, &capture.key)
        .unwrap_or_default()
        .into_iter()
        .filter(|(name, then)| now.of(project_dir, name) != then)
        .map(|(name, _)| name)
        .collect()
}

/// The project's files that have changed under any of `captures`.
pub fn changed_under(project_dir: &Path, captures: &[Capture]) -> BTreeSet<String> {
    let mut now = Hashes::default();
    captures
        .iter()
        .flat_map(|c| changed_in(project_dir, c, &mut now))
        .collect()
}

/// Why a capture can't be used as it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unready {
    /// It isn't taken.
    Untaken,
    /// It is of files that have changed since: these, by name.
    Stale(Vec<String>),
}

/// Why `capture` can't be used as it is; none when it can.
pub fn unready(project_dir: &Path, capture: &Capture) -> Option<Unready> {
    unready_in(project_dir, capture, &mut Hashes::default())
}

fn unready_in(project_dir: &Path, capture: &Capture, now: &mut Hashes) -> Option<Unready> {
    if !taken(project_dir, capture) {
        return Some(Unready::Untaken);
    }
    match changed_in(project_dir, capture, now) {
        files if files.is_empty() => None,
        files => Some(Unready::Stale(files)),
    }
}

/// Why `captures` can't all be used as they are, in words for a refusal:
/// how many aren't taken yet, and which files have changed under how
/// many. Empty when every one can be used.
pub fn refusals(project_dir: &Path, captures: &[Capture]) -> Vec<String> {
    let of = captures.len();
    let (mut untaken, mut stale) = (Vec::new(), Vec::new());
    let (mut files, mut now) = (BTreeSet::new(), Hashes::default());
    for capture in captures {
        match unready_in(project_dir, capture, &mut now) {
            None => {}
            Some(Unready::Untaken) => untaken.push(capture),
            Some(Unready::Stale(changed)) => {
                files.extend(changed);
                stale.push(capture);
            }
        }
    }
    let mut refusals = Vec::new();
    if let Some(first) = untaken.first() {
        refusals.push(format!(
            "{} of {of} captures {} taken yet (the first: {})",
            untaken.len(),
            if untaken.len() == 1 {
                "isn't"
            } else {
                "aren't"
            },
            first.what
        ));
    }
    if let Some(first) = stale.first() {
        refusals.push(format!(
            "{} of {of} captures {} out of date (the first: {}): {} changed since",
            stale.len(),
            if stale.len() == 1 { "is" } else { "are" },
            first.what,
            have(&files)
        ));
    }
    refusals
}

/// `names` as the subject of a sentence, three of them at most by name:
/// `a.css has`, `a.css and b.css have`, `a.css, b.css, c.js and 2 more have`.
pub fn have(names: &BTreeSet<String>) -> String {
    let named: Vec<&str> = names.iter().take(3).map(String::as_str).collect();
    let listed = match (names.len() - named.len(), named.split_last()) {
        (0, Some((last, rest))) if !rest.is_empty() => {
            format!("{} and {last}", rest.join(", "))
        }
        (0, _) => named.concat(),
        (more, _) => format!("{} and {more} more", named.join(", ")),
    };
    format!("{listed} {}", if names.len() == 1 { "has" } else { "have" })
}

/// Whether a still's frames file holds the boxes of its scene's targets,
/// which cameras are framed from. A file from before vidni measured them,
/// or one that is cut short, doesn't.
pub fn measured(frames: &Path) -> bool {
    std::fs::read_to_string(frames)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .is_some_and(|frames| frames["targets"].is_object())
}

/// The captures that `cuts` need and can't use as the project folder has
/// them: the ones it doesn't hold, and the ones that are out of date. They
/// are what a draft would capture.
pub fn plan(project: &Project, project_dir: &Path, cuts: &[&Cut]) -> Vec<Capture> {
    let (mut seen, mut now) = (BTreeSet::new(), Hashes::default());
    cuts.iter()
        .flat_map(|cut| for_cut(project, cut))
        .filter(|c| seen.insert(c.key.clone()))
        .filter(|c| unready_in(project_dir, c, &mut now).is_some())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_json_sorts_keys_at_every_level() {
        let a = json!({"b": 1, "a": {"y": [true, null], "x": "é"}});
        assert_eq!(canonical(&a), r#"{"a":{"x":"é","y":[true,null]},"b":1}"#);
    }

    #[test]
    fn changed_files_are_named_three_at_most() {
        let names = |n: usize| -> BTreeSet<String> {
            ["a.css", "b.css", "c.js", "d.png", "e.svg"][..n]
                .iter()
                .map(|name| name.to_string())
                .collect()
        };
        assert_eq!(have(&names(1)), "a.css has");
        assert_eq!(have(&names(2)), "a.css and b.css have");
        assert_eq!(have(&names(3)), "a.css, b.css and c.js have");
        assert_eq!(have(&names(5)), "a.css, b.css, c.js and 2 more have");
    }

    #[test]
    fn a_capture_names_its_pages_file_unless_the_page_is_on_the_web() {
        let text = r#"schema = 1

[meta]
title = "Where the pages are"
default_scene = "site"

[[cuts]]
id = "long"
aspect = "16:9"

[sections.main]

[[lines]]
id = "main-1"
section = "main"
show = "A page on the web."

[[lines]]
id = "main-2"
section = "main"
show = "A page in the project folder."
scene = "local"

[[lines]]
id = "main-3"
section = "main"
show = "A card."
scene = "card"

[scenes.site]
kind = "web"
url = "https://example.com/a?b=1"

[scenes.local]
kind = "web"
url = "pages/local.html?tab=2"

[scenes.card]
kind = "page"
url = "cards/card-{aspect}.html#top"
"#;
        let project = crate::validate::check(text, None).unwrap();
        let web: Vec<(String, Option<String>)> = for_cut(&project, &project.cuts[0])
            .into_iter()
            .map(|c| {
                assert_eq!(c.on_the_web(), c.file.is_none());
                (c.what, c.file)
            })
            .collect();
        assert_eq!(
            web,
            [
                ("line main-1".to_string(), None),
                ("line main-2".to_string(), Some("pages/local.html".into())),
                (
                    "page card at 1920x1080 for 16x9".to_string(),
                    Some("cards/card-16x9.html".into())
                ),
            ]
        );
    }
}
