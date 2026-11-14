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

use crate::project::{Aspect, Cut, Project, Scene, SceneKind};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Where captures live, relative to the project folder (out of git).
pub const DIR: &str = "build/captures";

/// A capture a cut needs.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Capture {
    pub key: String,
    /// What it shows, for people: `line <id>` or `page <scene> at <w>x<h>`.
    pub what: String,
    pub source: Source,
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
        let Some(scene) = project.scenes.get(id).filter(|s| s.kind == SceneKind::Web) else {
            continue;
        };
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
            add(Capture {
                key: key.clone(),
                what: format!("line {}", line.id),
                source: Source::Line(line.id.clone()),
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

/// Whether `capture` is of a page on the web, which needs the network,
/// rather than of a file in the project folder.
pub fn on_the_web(project: &Project, capture: &Capture) -> bool {
    let scene = match &capture.source {
        Source::Line(id) => project
            .lines
            .iter()
            .find(|line| &line.id == id)
            .and_then(|line| line.scene_in(project)),
        Source::Page { scene, .. } => Some(scene.as_str()),
    };
    scene
        .and_then(|id| project.scenes.get(id))
        .is_some_and(Scene::on_the_web)
}

/// Whether the project folder holds `capture`: its picture and, for a web
/// still, frames that a cut can be framed from. A still without them can't
/// be used, so it counts as not taken.
pub fn taken(project_dir: &Path, capture: &Capture) -> bool {
    path(project_dir, &capture.key).is_file()
        && match capture.source {
            Source::Line(_) => measured(&frames_path(project_dir, &capture.key)),
            Source::Page { .. } => true,
        }
}

/// Whether a still's frames file holds the boxes of its scene's targets,
/// which cameras are framed from. A file from before vidni measured them,
/// or one that is cut short, doesn't.
fn measured(frames: &Path) -> bool {
    std::fs::read_to_string(frames)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .is_some_and(|frames| frames["targets"].is_object())
}

/// The captures that `cuts` need and the project folder doesn't hold yet:
/// what a draft would capture.
pub fn plan(project: &Project, project_dir: &Path, cuts: &[&Cut]) -> Vec<Capture> {
    let mut seen = BTreeSet::new();
    cuts.iter()
        .flat_map(|cut| for_cut(project, cut))
        .filter(|c| seen.insert(c.key.clone()))
        .filter(|c| !taken(project_dir, c))
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
    fn a_capture_is_on_the_web_when_its_scenes_address_is() {
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
url = "card.html"
"#;
        let project = crate::validate::check(text, None).unwrap();
        let web: Vec<(String, bool)> = for_cut(&project, &project.cuts[0])
            .into_iter()
            .map(|c| (c.what.clone(), on_the_web(&project, &c)))
            .collect();
        assert_eq!(
            web,
            [
                ("line main-1".to_string(), true),
                ("line main-2".to_string(), false),
                ("page card at 1920x1080 for 16x9".to_string(), false),
            ]
        );
    }
}
