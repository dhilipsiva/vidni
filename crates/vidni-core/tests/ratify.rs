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
        // A web still comes with its frames.
        if matches!(c.source, capture::Source::Line(_)) {
            fs::write(capture::frames_path(dir, &c.key), br#"{"targets":{}}"#).unwrap();
        }
        record(dir, c);
    }
    planned.len()
}

/// Writes the record that a capture of one of the project's own pages
/// comes with: the files it was made from, here the page's own file only.
fn record(dir: &Path, c: &capture::Capture) {
    if let Some(file) = &c.file {
        let made_from = [(file.clone(), capture::file_sha256(dir, file))].into();
        fs::write(
            capture::files_path(dir, &c.key),
            capture::files_json(&made_from),
        )
        .unwrap();
    }
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
fn a_stills_frames_are_frozen_with_it() {
    let (dir, lock) = ratified();
    let p = project(dir.path(), PROJECT);
    let hook = capture::line_keys(&p)["hook-1"].clone();
    // The boxes a still is framed by are part of what the cut shows.
    fs::write(
        capture::frames_path(dir.path(), &hook),
        b"{\"moved\": true}",
    )
    .unwrap();
    let states = ratify::status(&p, dir.path(), &lock);
    assert_eq!(states[0], ok("long"));
    assert_eq!(
        states[1],
        changed(
            "short",
            &[&format!(
                "the frames of capture {} changed on disk",
                &hook[..12]
            )]
        )
    );
    // A cut with many files gone says how many, not which.
    let long = &p.cuts[0];
    let mut big = lock.clone();
    let frozen = big.cuts.get_mut(&long.id).unwrap();
    for n in 0..5 {
        frozen.captures.insert(format!("{n:064}"), "00".into());
    }
    let Status::Changed(reasons) = ratify::status_of(&p, dir.path(), &big, long) else {
        panic!("captures that are gone change the cut");
    };
    assert_eq!(reasons, ["5 of its 8 captures went missing"]);
    // A still without its frames can't be framed, so it is taken again.
    fs::remove_file(capture::frames_path(dir.path(), &hook)).unwrap();
    let cuts: Vec<_> = p.cuts.iter().collect();
    let what: Vec<String> = capture::plan(&p, dir.path(), &cuts)
        .into_iter()
        .map(|c| c.what)
        .collect();
    assert_eq!(what, ["line hook-1"]);
}

#[test]
fn a_frozen_still_isnt_taken_again_for_want_of_its_frames() {
    let (dir, lock) = ratified();
    let p = project(dir.path(), PROJECT);
    let cuts: Vec<_> = p.cuts.iter().collect();
    let hook = capture::line_keys(&p)["hook-1"].clone();
    // Its frames are gone, so the still counts as not taken; but the
    // picture is the one the short cut froze, and a draft must leave it.
    fs::remove_file(capture::frames_path(dir.path(), &hook)).unwrap();
    let plan = capture::plan(&p, dir.path(), &cuts);
    assert_eq!(plan.len(), 1);
    assert_eq!(
        ratify::frozen(dir.path(), &lock, &plan),
        [format!(
            "the capture of line hook-1 is frozen by cut short, and its frames can't be read; restore build/captures/{hook}.json from a backup, or run `vidni unratify --cut short --refresh` to take it afresh"
        )]
    );
    // Frames from before targets were measured can't be framed from either.
    fs::write(capture::frames_path(dir.path(), &hook), b"{\"cameras\":{}}").unwrap();
    assert_eq!(capture::plan(&p, dir.path(), &cuts).len(), 1);
    // A picture that is no longer the frozen one may be taken again, and
    // so may one that no ratified cut froze.
    fs::write(capture::path(dir.path(), &hook), b"another screenshot").unwrap();
    assert_eq!(
        ratify::frozen(dir.path(), &lock, &plan),
        Vec::<String>::new()
    );
    assert_eq!(
        ratify::frozen(dir.path(), &Lock::default(), &plan),
        Vec::<String>::new()
    );
}

#[test]
fn a_lock_from_before_frames_were_frozen_cant_vouch_for_them() {
    let (dir, mut lock) = ratified();
    let p = project(dir.path(), PROJECT);
    lock.cuts.get_mut("long").unwrap().frames.clear();
    let states = ratify::status(&p, dir.path(), &lock);
    assert_eq!(
        states[0],
        changed(
            "long",
            &["the frames of 2 of its stills were never frozen; ratify it again"]
        )
    );
    assert_eq!(states[1], ok("short"));
    // A key that isn't a hash, as a lock edited by hand may hold, is named whole.
    lock.cuts
        .get_mut("short")
        .unwrap()
        .captures
        .insert("abc".into(), "00".into());
    let states = ratify::status(&p, dir.path(), &lock);
    assert_eq!(states[1], changed("short", &["capture abc went missing"]));
}

#[test]
fn a_vertical_cut_freezes_the_subtitle_its_title_band_shows() {
    let (dir, lock) = ratified();
    let text = PROJECT.replace(
        "title = \"Ratify\"",
        "title = \"Ratify\"\nsubtitle = \"a series\"",
    );
    // The 16:9 cut draws no subtitle; the 9:16 cut's band does.
    assert_eq!(
        states(dir.path(), &text, &lock),
        [
            ok("long"),
            changed(
                "short",
                &["its sections, subtitle, size or line order changed"]
            )
        ]
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

#[test]
fn an_expectation_retakes_the_stills_from_its_line_on() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("card.html"), "x").unwrap();
    let before = capture::line_keys(&project(dir.path(), PROJECT));
    // Capture waits for an expectation before a still, so adding one to
    // fix a stale still has to retake it, and the stills after it.
    let expecting = format!(
        "{PROJECT}\n[[expectations]]\nscene = \"page\"\nat = \"main-2\"\ntarget = \"title\"\ntext = \"Ready\"\n"
    );
    let after = capture::line_keys(&project(dir.path(), &expecting));
    assert_eq!(before["main-1"], after["main-1"], "the line before it");
    assert_ne!(before["main-2"], after["main-2"], "its own line");
    assert_ne!(before["hook-1"], after["hook-1"], "a line after it");
}

#[test]
fn a_capture_of_the_projects_own_page_is_out_of_date_once_its_file_changes() {
    use capture::Unready;
    let (dir, mut lock) = ratified();
    let dir = dir.path();
    let p = project(dir, PROJECT);
    let cuts: Vec<_> = p.cuts.iter().collect();
    let card = capture::page_capture(&p, &p.cuts[0], "card").unwrap();
    assert_eq!(card.file.as_deref(), Some("card.html"));
    assert_eq!(capture::unready(dir, &card), None);

    // The card's file changes: its render shows the file as it was. The
    // long cut shows the card, so it is no longer the cut that was
    // ratified; the short doesn't, and stays.
    fs::write(dir.join("card.html"), "<!doctype html><p>new").unwrap();
    assert_eq!(
        capture::unready(dir, &card),
        Some(Unready::Stale(vec!["card.html".into()]))
    );
    assert_eq!(
        ratify::status(&p, dir, &lock),
        [
            changed(
                "long",
                &["card.html has changed since 1 of its captures was taken"]
            ),
            ok("short")
        ]
    );
    // A draft would render it again, and until then the cut can't be ratified.
    let what: Vec<String> = capture::plan(&p, dir, &cuts)
        .into_iter()
        .map(|c| c.what)
        .collect();
    assert_eq!(what, ["page card at 1920x1080 for 16x9"]);
    assert_eq!(
        ratify::ratify(&p, dir, &mut lock.clone(), &["long".into()]).unwrap_err(),
        "long: 1 of 3 captures is out of date (the first: page card at 1920x1080 for 16x9): \
         card.html has changed since; draft the cut first"
    );
    // With the file as it was, nothing is out of date.
    fs::write(dir.join("card.html"), "<!doctype html>").unwrap();
    assert_eq!(ratify::status(&p, dir, &lock), [ok("long"), ok("short")]);
    assert!(capture::plan(&p, dir, &cuts).is_empty());

    // A file the page asked for and that wasn't there is one it was made
    // without: once it is there, the render is out of date too.
    let made_from = |also: &[(&str, Option<String>)]| {
        let mut files: capture::Files = [(
            "card.html".to_string(),
            capture::file_sha256(dir, "card.html"),
        )]
        .into();
        files.extend(
            also.iter()
                .map(|(name, sha)| (name.to_string(), sha.clone())),
        );
        fs::write(
            capture::files_path(dir, &card.key),
            capture::files_json(&files),
        )
        .unwrap();
    };
    made_from(&[("card.css", None)]);
    assert_eq!(capture::unready(dir, &card), None);
    fs::write(dir.join("card.css"), "body {}").unwrap();
    assert_eq!(
        capture::unready(dir, &card),
        Some(Unready::Stale(vec!["card.css".into()]))
    );
    // And one that was there and is gone.
    made_from(&[("card.css", capture::file_sha256(dir, "card.css"))]);
    assert_eq!(capture::unready(dir, &card), None);
    fs::remove_file(dir.join("card.css")).unwrap();
    assert_eq!(
        capture::refusals(dir, std::slice::from_ref(&card)),
        [
            "1 of 1 captures is out of date (the first: page card at 1920x1080 for 16x9): \
          card.css has changed since"
        ]
    );

    // Without its record, a render can't be told to be out of date, so it
    // counts as not taken.
    fs::remove_file(capture::files_path(dir, &card.key)).unwrap();
    assert_eq!(capture::unready(dir, &card), Some(Unready::Untaken));
    assert_eq!(
        ratify::status(&p, dir, &lock)[0],
        changed(
            "long",
            &["1 of its captures has no record of the files it was made from; draft it again"]
        )
    );
    // A record that names a file outside the project folder isn't one a
    // capture wrote, and isn't read.
    for name in ["../card.html", "/etc/hostname", "a/../../b", ""] {
        made_from(&[(name, None)]);
        assert_eq!(capture::files(dir, &card.key), None, "{name:?}");
        assert_eq!(capture::unready(dir, &card), Some(Unready::Untaken));
    }

    // A capture that is deleted to be taken afresh takes its record along.
    made_from(&[]);
    assert_eq!(capture::unready(dir, &card), None);
    let deleted = ratify::unratify(&p, dir, &mut lock, &["long".into()], true).unwrap();
    assert!(
        deleted.contains(&capture::path(dir, &card.key)),
        "{deleted:?}"
    );
    assert!(!capture::files_path(dir, &card.key).exists());
}

#[test]
fn a_frozen_still_of_the_projects_own_page_is_taken_again_once_the_page_changes() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    for file in ["card.html", "site.html"] {
        fs::write(dir.join(file), "<!doctype html>").unwrap();
    }
    // The web scene is a page in the project folder now.
    let text = PROJECT.replace("https://example.com", "site.html?tab=1#top");
    let p = project(dir, &text);
    assert_eq!(draft(dir, &p), 4);
    let mut lock = Lock::default();
    ratify::ratify(&p, dir, &mut lock, &[]).unwrap();
    let cuts: Vec<_> = p.cuts.iter().collect();
    assert_eq!(ratify::status(&p, dir, &lock), [ok("long"), ok("short")]);

    // The page changes. Its three stills are frozen, and out of date: a
    // draft may take them again, since what changed is the creator's own
    // file, to change back.
    fs::write(dir.join("site.html"), "<!doctype html><h1>new").unwrap();
    let plan = capture::plan(&p, dir, &cuts);
    assert_eq!(plan.len(), 3);
    assert_eq!(ratify::frozen(dir, &lock, &plan), Vec::<String>::new());
    let reason = "site.html has changed since 2 of its captures were taken";
    assert_eq!(
        ratify::status(&p, dir, &lock),
        [changed("long", &[reason]), changed("short", &[reason])]
    );
    // A still whose frames can't be read is left alone all the same.
    let hook = capture::line_keys(&p)["hook-1"].clone();
    fs::remove_file(capture::frames_path(dir, &hook)).unwrap();
    let refusals = ratify::frozen(dir, &lock, &plan);
    assert_eq!(refusals.len(), 1, "{refusals:?}");
    assert!(refusals[0].starts_with("the capture of line hook-1 is frozen by cut short"));
}
