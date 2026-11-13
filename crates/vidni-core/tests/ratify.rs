//! Ratification per cut: each cut's status is worked out from hashes, so an
//! edit regresses exactly the cuts that show or say what changed.

use std::fs;
use std::path::Path;
use vidni_core::capture;
use vidni_core::edit::{LineField, ProjectFile};
use vidni_core::project::Project;
use vidni_core::ratify::{self, Lock, Status};
use vidni_core::validate::check;

/// A long cut and a short that share one line, and each have a line of their own.
const PROJECT: &str = r#"schema = 1

[meta]
title = "Ratify"
default_scene = "page"

[[cuts]]
id = "long"
aspect = "16:9"
lines = ["main-1", "main-2", "main-3"]

[[cuts]]
id = "short"
aspect = "9:16"
lines = ["main-2", "hook-1"]

[sections.main]
label = "Main"

[sections.hook]
label = "Hook"

[[lines]]
id = "main-1"
section = "main"
show = "First, the title."
do = [{ click = "tab" }]
highlight = ["title"]

[[lines]]
id = "main-2"
section = "main"
show = "Both cuts use this line."

[[lines]]
id = "main-3"
section = "main"
show = "Only the long cut has this card."
scene = "card"

[[lines]]
id = "hook-1"
section = "hook"
show = "Only the short has this line."
highlight = ["tab"]

[scenes.page]
kind = "web"
url = "https://example.com"

[scenes.page.targets]
title = { css = "h1" }
tab = { css = "button" }

[scenes.page.cameras]
heading = { union = ["title"] }

[scenes.card]
kind = "page"
url = "card.html"
"#;

fn project(dir: &Path, text: &str) -> Project {
    check(text, Some(dir)).unwrap_or_else(|p| panic!("{p:?}"))
}

/// What a draft would do: take every missing capture (here, a stand-in file).
fn draft(dir: &Path, p: &Project) -> usize {
    let cuts: Vec<_> = p.cuts.iter().collect();
    let planned = capture::plan(p, dir, &cuts);
    for c in &planned {
        let path = capture::path(dir, &c.key);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, c.key.as_bytes()).unwrap();
    }
    planned.len()
}

/// A drafted project with both cuts ratified.
fn ratified() -> (tempfile::TempDir, Lock) {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("card.html"), "<!doctype html>").unwrap();
    let p = project(dir.path(), PROJECT);
    assert_eq!(
        draft(dir.path(), &p),
        4,
        "three web lines and the long cut's card"
    );
    let mut lock = Lock::default();
    ratify::ratify(&p, dir.path(), &mut lock, &[]).unwrap();
    ratify::write_lock(dir.path(), &lock).unwrap();
    let lock = ratify::read_lock(dir.path()).unwrap();
    (dir, lock)
}

fn states(dir: &Path, text: &str, lock: &Lock) -> Vec<(String, Status)> {
    ratify::status(&project(dir, text), dir, lock)
}

fn edited(edit: impl FnOnce(&mut ProjectFile)) -> String {
    let mut f = ProjectFile::parse(PROJECT).unwrap();
    edit(&mut f);
    f.text()
}

fn ok(id: &str) -> (String, Status) {
    (id.to_string(), Status::Ratified)
}

fn changed(id: &str, reasons: &[&str]) -> (String, Status) {
    (
        id.to_string(),
        Status::Changed(reasons.iter().map(|r| r.to_string()).collect()),
    )
}

#[test]
fn a_cut_is_draft_until_ratified_and_the_lock_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("card.html"), "x").unwrap();
    let p = project(dir.path(), PROJECT);
    let lock = Lock::default();
    assert_eq!(ratify::status(&p, dir.path(), &lock)[0].1, Status::Draft);
    let mut lock = Lock::default();
    let refused = ratify::ratify(&p, dir.path(), &mut lock, &["long".into()]).unwrap_err();
    assert!(refused.contains("captures aren't taken yet"), "{refused}");
    let (dir, lock) = ratified();
    assert_eq!(
        states(dir.path(), PROJECT, &lock),
        [ok("long"), ok("short")]
    );
    assert!(
        fs::read_to_string(dir.path().join(ratify::LOCK))
            .unwrap()
            .starts_with("# Written by `vidni ratify`")
    );
}

#[test]
fn editing_the_short_leaves_the_long_cut_ratified() {
    let (dir, lock) = ratified();
    let text = edited(|f| {
        f.set_line("hook-1", LineField::Show, Some("A better hook."))
            .unwrap()
    });
    assert_eq!(
        states(dir.path(), &text, &lock),
        [ok("long"), changed("short", &["line hook-1 changed"])]
    );
    // Choosing other lines for the short doesn't touch the long cut either.
    let text = PROJECT.replace(
        r#"lines = ["main-2", "hook-1"]"#,
        r#"lines = ["main-1", "main-2", "hook-1"]"#,
    );
    let states = states(dir.path(), &text, &lock);
    assert_eq!(states[0], ok("long"));
    assert!(
        matches!(&states[1].1, Status::Changed(r) if r.contains(&"line main-1 was added".to_string()))
    );
}

#[test]
fn editing_a_shared_line_regresses_every_cut_that_uses_it() {
    let (dir, lock) = ratified();
    let text = edited(|f| {
        f.set_line(
            "main-2",
            LineField::Show,
            Some("Both cuts use these words."),
        )
        .unwrap()
    });
    assert_eq!(
        states(dir.path(), &text, &lock),
        [
            changed("long", &["line main-2 changed"]),
            changed("short", &["line main-2 changed"])
        ]
    );
}

#[test]
fn gaps_caption_style_and_platforms_need_no_unratify() {
    let (dir, lock) = ratified();
    let text = PROJECT
        .replace("schema = 1\n", "schema = 1\n\n[timing]\nline_gap = 0.5\nsection_gap = 1.2\n\n[style]\ntheme = \"themes/other\"\n")
        .replace("aspect = \"16:9\"\n", "aspect = \"16:9\"\ncaptions = \"file\"\nplatforms = [\"youtube\", \"linkedin\"]\n");
    assert_eq!(states(dir.path(), &text, &lock), [ok("long"), ok("short")]);
}

#[test]
fn a_redraft_captures_nothing() {
    let (dir, mut lock) = ratified();
    let p = project(dir.path(), PROJECT);
    assert_eq!(draft(dir.path(), &p), 0);
    // Unratifying keeps the captures: the next draft takes none, and the cut ratifies again as it was.
    ratify::unratify(&p, dir.path(), &mut lock, &["long".into()], false).unwrap();
    assert_eq!(ratify::status(&p, dir.path(), &lock)[0].1, Status::Draft);
    assert_eq!(draft(dir.path(), &p), 0);
    ratify::ratify(&p, dir.path(), &mut lock, &["long".into()]).unwrap();
    assert_eq!(
        ratify::status(&p, dir.path(), &lock),
        [ok("long"), ok("short")]
    );
}

#[test]
fn a_changed_scene_is_captured_again_and_a_changed_camera_is_not() {
    let (dir, lock) = ratified();
    let reframed = PROJECT.replace(
        r#"heading = { union = ["title"] }"#,
        r#"heading = { union = ["title", "tab"] }"#,
    );
    let p = project(dir.path(), &reframed);
    assert_eq!(draft(dir.path(), &p), 0, "cameras only frame a still");
    assert_eq!(
        states(dir.path(), &reframed, &lock)[0],
        changed("long", &["scene page changed"])
    );
    let moved = PROJECT.replace("https://example.com", "https://example.org");
    let p = project(dir.path(), &moved);
    let cuts: Vec<_> = p.cuts.iter().collect();
    let what: Vec<String> = capture::plan(&p, dir.path(), &cuts)
        .into_iter()
        .map(|c| c.what)
        .collect();
    let mut what = what;
    what.sort();
    assert_eq!(
        what,
        ["line hook-1", "line main-1", "line main-2"],
        "the card isn't on the page"
    );
}

#[test]
fn refresh_retakes_only_captures_no_ratified_cut_shares() {
    let (dir, mut lock) = ratified();
    let p = project(dir.path(), PROJECT);
    let deleted = ratify::unratify(&p, dir.path(), &mut lock, &["short".into()], true).unwrap();
    assert_eq!(deleted.len(), 1, "main-2's capture is the long cut's too");
    let short: Vec<_> = p.cuts.iter().filter(|c| c.id == "short").collect();
    let what: Vec<String> = capture::plan(&p, dir.path(), &short)
        .into_iter()
        .map(|c| c.what)
        .collect();
    assert_eq!(what, ["line hook-1"]);
    assert_eq!(
        ratify::status(&p, dir.path(), &lock),
        [ok("long"), (String::from("short"), Status::Draft)]
    );
}

#[test]
fn a_capture_changed_on_disk_regresses_the_cuts_that_froze_it() {
    let (dir, lock) = ratified();
    let p = project(dir.path(), PROJECT);
    let hook = capture::line_keys(&p)["hook-1"].clone();
    fs::write(capture::path(dir.path(), &hook), b"a new screenshot").unwrap();
    let states = ratify::status(&p, dir.path(), &lock);
    assert_eq!(states[0], ok("long"));
    assert_eq!(
        states[1],
        changed(
            "short",
            &[&format!("capture {} changed on disk", &hook[..12])]
        )
    );
}

#[test]
fn the_resolved_project_fills_in_every_default() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("card.html"), "x").unwrap();
    let p = project(dir.path(), PROJECT);
    let path = ratify::write_resolved(&p, dir.path()).unwrap();
    let r: serde_json::Value = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(r["cuts"][0]["captions"], "both");
    assert_eq!(r["cuts"][1]["captions"], "burned");
    assert_eq!(
        r["cuts"][1]["platforms"],
        serde_json::json!(["youtube-shorts", "instagram-reels", "whatsapp-status"])
    );
    assert_eq!(r["cuts"][1]["size"], serde_json::json!([1080, 1920]));
    assert_eq!(r["lines"][1]["scene"], "page");
    assert_eq!(r["timing"]["line_gap"], 0.28);
}

#[test]
fn lines_that_show_the_same_page_share_one_capture() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("card.html"), "x").unwrap();
    let same = PROJECT.replace(
        "show = \"Only the short has this line.\"\nhighlight = [\"tab\"]\n",
        "show = \"Only the short has this line.\"\n",
    );
    let p = project(dir.path(), &same);
    let keys = capture::line_keys(&p);
    assert_eq!(
        keys["main-2"], keys["hook-1"],
        "the same actions so far, and no highlights"
    );
    assert_ne!(keys["main-1"], keys["main-2"]);
    assert_eq!(draft(dir.path(), &p), 3);
}
