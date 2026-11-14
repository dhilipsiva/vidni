//! The gates of a final build, what a release hashes, and how a cut stands
//! against its newest release (TODO.md, rule 4 and P3-VIDNI-06).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use vidni_core::capture;
use vidni_core::edit::{LineField, ProjectFile};
use vidni_core::project::{Cut, Project};
use vidni_core::ratify::{self, Lock};
use vidni_core::release::{self, Output, Record, Releases, Sources, Standing, T1, Tools, Voice};
use vidni_core::takes;
use vidni_core::validate::check;

/// A long cut and a short that share one line, and each have a line of their own.
const PROJECT: &str = r#"schema = 1

[meta]
title = "Release"
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

[scenes.page]
kind = "web"
url = "https://example.com"

[scenes.page.targets]
title = { css = "h1" }

[scenes.card]
kind = "page"
url = "card.html"
"#;

fn project(dir: &Path, text: &str) -> Project {
    check(text, Some(dir)).unwrap_or_else(|p| panic!("{p:?}"))
}

fn edited(edit: impl FnOnce(&mut ProjectFile)) -> String {
    let mut f = ProjectFile::parse(PROJECT).unwrap();
    edit(&mut f);
    f.text()
}

fn cut<'p>(p: &'p Project, id: &str) -> &'p Cut {
    p.cuts.iter().find(|c| c.id == id).unwrap()
}

/// A project folder with every capture taken (stand-in files).
fn drafted() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("card.html"), "<!doctype html>").unwrap();
    let p = project(dir.path(), PROJECT);
    let cuts: Vec<_> = p.cuts.iter().collect();
    for c in capture::plan(&p, dir.path(), &cuts) {
        let path = capture::path(dir.path(), &c.key);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, c.key.as_bytes()).unwrap();
        if matches!(c.source, capture::Source::Line(_)) {
            fs::write(
                capture::frames_path(dir.path(), &c.key),
                br#"{"targets":{}}"#,
            )
            .unwrap();
        }
        // The card is the project's own page: its render comes with the
        // record of the files it was made from.
        if let Some(file) = &c.file {
            let made_from = [(file.clone(), capture::file_sha256(dir.path(), file))].into();
            fs::write(
                capture::files_path(dir.path(), &c.key),
                capture::files_json(&made_from),
            )
            .unwrap();
        }
    }
    dir
}

/// Records a take of `line`'s present words: a stand-in file holding `sound`.
fn record(dir: &Path, p: &Project, line: &str, sound: &str) -> PathBuf {
    let take = takes::status(p, dir)
        .into_iter()
        .find(|t| t.id == line)
        .unwrap();
    fs::create_dir_all(take.path.parent().unwrap()).unwrap();
    fs::write(&take.path, sound).unwrap();
    take.path
}

fn ratify_all(dir: &Path, p: &Project) -> Lock {
    let mut lock = Lock::default();
    ratify::ratify(p, dir, &mut lock, &[]).unwrap();
    lock
}

#[test]
fn a_final_build_is_refused_until_its_cut_is_ratified_and_every_line_has_a_take() {
    let dir = drafted();
    let dir = dir.path();
    let p = project(dir, PROJECT);
    let long = cut(&p, "long");
    assert_eq!(
        release::refusals(&p, dir, &Lock::default(), long),
        [
            "cut long isn't ratified; `vidni ratify --cut long` freezes it",
            "cut long: 3 of its 3 lines have no take: main-1, main-2, main-3"
        ]
    );
    // Ratified, it still needs its takes: each line's, and only its own lines'.
    let mut lock = Lock::default();
    ratify::ratify(&p, dir, &mut lock, &["long".into()]).unwrap();
    record(dir, &p, "main-1", "one");
    record(dir, &p, "main-2", "two");
    record(dir, &p, "hook-1", "the short's line");
    assert_eq!(
        release::refusals(&p, dir, &lock, long),
        ["cut long: 1 of its 3 lines has no take: main-3"]
    );
    record(dir, &p, "main-3", "three");
    assert_eq!(
        release::refusals(&p, dir, &lock, long),
        Vec::<String>::new()
    );
    // The short has its takes, and was never ratified.
    assert_eq!(
        release::refusals(&p, dir, &lock, cut(&p, "short")),
        ["cut short isn't ratified; `vidni ratify --cut short` freezes it"]
    );
}

#[test]
fn a_final_build_is_refused_when_its_cut_changed_after_it_was_ratified() {
    let dir = drafted();
    let dir = dir.path();
    let p = project(dir, PROJECT);
    for line in ["main-1", "main-2", "main-3", "hook-1"] {
        record(dir, &p, line, line);
    }
    let lock = ratify_all(dir, &p);
    assert!(release::refusals(&p, dir, &lock, cut(&p, "long")).is_empty());

    // New words for a line: the cut has changed, and the line's take is for the old ones.
    let text = edited(|f| {
        f.set_line(
            "main-2",
            LineField::Show,
            Some("Both cuts use these new words."),
        )
        .unwrap();
    });
    let reworded = project(dir, &text);
    assert_eq!(
        release::refusals(&reworded, dir, &lock, cut(&reworded, "long")),
        [
            "cut long has changed since it was ratified: line main-2 changed",
            "cut long: 1 of its 3 lines has a take only for other words: main-2"
        ]
    );
    // A capture retaken behind the lock's back.
    let key = capture::line_keys(&p)["main-1"].clone();
    fs::write(capture::path(dir, &key), b"another screenshot").unwrap();
    assert_eq!(
        release::refusals(&p, dir, &lock, cut(&p, "long")),
        [format!(
            "cut long has changed since it was ratified: capture {} changed on disk",
            &key[..12]
        )]
    );
    // The short doesn't show that still, so it may still be built.
    assert!(release::refusals(&p, dir, &lock, cut(&p, "short")).is_empty());
}

#[test]
fn missing_takes_are_listed_up_to_five_and_counted() {
    let lines: String = (1..=8)
        .map(|n| format!("\n[[lines]]\nid = \"more-{n}\"\nsection = \"main\"\nshow = \"Line {n} of more.\"\nscene = \"card\"\n"))
        .collect();
    let text = PROJECT.replace(
        "lines = [\"main-1\", \"main-2\", \"main-3\"]",
        "lines = \"all\"",
    );
    // The new lines join the main section, before the hook's line.
    let text = text.replace(
        "\n[[lines]]\nid = \"hook-1\"",
        &format!("{lines}\n[[lines]]\nid = \"hook-1\""),
    );
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("card.html"), "x").unwrap();
    let p = project(dir.path(), &text);
    let refused = release::takes_of(&p, dir.path(), cut(&p, "long")).unwrap_err();
    assert_eq!(
        refused,
        [
            "cut long: 12 of its 12 lines have no take: main-1, main-2, main-3, more-1, more-2 and 7 more"
        ]
    );
}

/// The sources of the long cut, with its lines' takes as the audio.
fn sources(dir: &Path, p: &Project) -> Sources {
    let long = cut(p, "long");
    let clips = release::takes_of(p, dir, long).unwrap();
    release::sources(p, dir, long, &clips).unwrap()
}

#[test]
fn each_source_hash_changes_with_its_own_part_and_no_other() {
    let dir = drafted();
    let dir = dir.path();
    let p = project(dir, PROJECT);
    for line in ["main-1", "main-2", "main-3"] {
        record(dir, &p, line, line);
    }
    let was = sources(dir, &p);
    assert_eq!(was, sources(dir, &p));
    let differs = |now: &Sources| {
        [
            ("content", now.content != was.content),
            ("audio", now.audio != was.audio),
            ("look", now.look != was.look),
            ("captures", now.captures != was.captures),
        ]
        .into_iter()
        .filter_map(|(part, differs)| differs.then_some(part))
        .collect::<Vec<_>>()
    };

    // A gap is part of the look, which ratifying doesn't freeze.
    let gaps = project(
        dir,
        &PROJECT.replace(
            "[[cuts]]\nid = \"long\"",
            "[timing]\nline_gap = 0.5\n\n[[cuts]]\nid = \"long\"",
        ),
    );
    assert_eq!(differs(&sources(dir, &gaps)), ["look"]);
    // So are the title and the cut's own settings.
    let titled = project(
        dir,
        &PROJECT.replace("title = \"Release\"", "title = \"Another title\""),
    );
    assert_eq!(differs(&sources(dir, &titled)), ["look"]);
    let captioned = project(
        dir,
        &PROJECT.replace(
            "aspect = \"16:9\"",
            "aspect = \"16:9\"\ncaptions = \"file\"",
        ),
    );
    assert_eq!(differs(&sources(dir, &captioned)), ["look"]);

    // What only the listening check reads changes nothing a build is made from.
    let heard = project(
        dir,
        &PROJECT.replace(
            "[[cuts]]\nid = \"long\"",
            "[voice]\nheard_as = [{ heard = \"nibley\", means = \"nibli\" }]\n\n[[cuts]]\nid = \"long\"",
        ),
    );
    assert_eq!(differs(&sources(dir, &heard)), Vec::<&str>::new());
    // Nor does a line the cut doesn't use.
    let hook = edited(|f| {
        f.set_line(
            "hook-1",
            LineField::Show,
            Some("The short's line, reworded."),
        )
        .unwrap();
    });
    assert_eq!(
        differs(&sources(dir, &project(dir, &hook))),
        Vec::<&str>::new()
    );

    // A retake is new audio, and nothing else.
    let take = record(dir, &p, "main-2", "said again");
    assert_eq!(differs(&sources(dir, &p)), ["audio"]);
    fs::write(&take, "main-2").unwrap();
    assert_eq!(differs(&sources(dir, &p)), Vec::<&str>::new());
    // So is the same sound filed under another name, which the timeline would name.
    let clips = release::takes_of(&p, dir, cut(&p, "long")).unwrap();
    let mut moved = clips.clone();
    moved[1].1 = dir.join("elsewhere.wav");
    fs::copy(&take, &moved[1].1).unwrap();
    let elsewhere = release::sources(&p, dir, cut(&p, "long"), &moved).unwrap();
    assert_eq!(differs(&elsewhere), ["audio"]);

    // A still's picture and its frames are both the capture.
    let key = capture::line_keys(&p)["main-1"].clone();
    fs::write(capture::frames_path(dir, &key), b"{\"moved\": true}").unwrap();
    assert_eq!(differs(&sources(dir, &p)), ["captures"]);
    fs::write(capture::frames_path(dir, &key), br#"{"targets":{}}"#).unwrap();
    fs::write(capture::path(dir, &key), b"another screenshot").unwrap();
    assert_eq!(differs(&sources(dir, &p)), ["captures"]);
}

/// A final build's record of the long cut, as made from `sources`.
fn built(sources: Sources) -> Record {
    let hash = |s: &str| capture::sha256_hex(s.as_bytes());
    let tools = Tools {
        vidni: "0.1.0".into(),
        ffmpeg: "9.0.2".into(),
        fonts: BTreeMap::new(),
        engine: serde_json::json!({"fps": 30}),
    };
    Record {
        cut: "long".into(),
        voice: Voice::Takes,
        inputs: release::inputs(Voice::Takes, &sources, &tools, &hash("timeline")),
        sources,
        t1: T1 {
            timeline: hash("timeline"),
            srt: hash("srt"),
            vtt: hash("vtt"),
            pcm: hash("pcm"),
            pcm_normalized: hash("normalized"),
        },
        outputs: BTreeMap::from([(
            "long.mp4".to_string(),
            Output {
                captions: true,
                frames: hash("frames"),
                frame_count: 90,
                packets: hash("packets"),
                mp4: hash("mp4"),
                qa: String::new(),
                problems: Vec::new(),
            },
        )]),
        tools,
    }
}

#[test]
fn a_cut_stands_against_its_newest_release() {
    let dir = drafted();
    let dir = dir.path();
    let p = project(dir, PROJECT);
    for line in ["main-1", "main-2", "main-3"] {
        record(dir, &p, line, line);
    }
    let long = cut(&p, "long");
    let mut releases = Releases::default();
    assert_eq!(
        release::standing(&p, dir, &releases, long),
        Standing::Unreleased
    );

    release::approve(&mut releases, &built(sources(dir, &p)), "2026-11-14");
    let current = Standing::Current("long-r1".into());
    assert_eq!(release::standing(&p, dir, &releases, long), current);
    assert_eq!(
        release::standing(&p, dir, &releases, cut(&p, "short")),
        Standing::Unreleased
    );

    // A retake after the release: the release is of the earlier take.
    let take = record(dir, &p, "main-1", "said again");
    assert_eq!(
        release::standing(&p, dir, &releases, long),
        Standing::Outdated("long-r1".into(), vec!["its takes"])
    );
    // Approving the rebuilt cut makes the next release, which is current.
    release::approve(&mut releases, &built(sources(dir, &p)), "2026-11-15");
    assert_eq!(
        release::standing(&p, dir, &releases, long),
        Standing::Current("long-r2".into())
    );
    // New words and a new gap show as such.
    let text = edited(|f| {
        f.set_line(
            "main-3",
            LineField::Show,
            Some("Only the long cut has this card!"),
        )
        .unwrap();
    })
    .replace(
        "[[cuts]]\nid = \"long\"",
        "[timing]\ntail = 1.0\n\n[[cuts]]\nid = \"long\"",
    );
    let reworded = project(dir, &text);
    assert_eq!(
        release::standing(&reworded, dir, &releases, cut(&reworded, "long")),
        Standing::Outdated(
            "long-r2".into(),
            vec!["what it shows and says", "its gaps, look or settings"]
        )
    );
    // A take that is gone can't be the released one.
    fs::remove_file(take).unwrap();
    assert_eq!(
        release::standing(&p, dir, &releases, long),
        Standing::Outdated("long-r2".into(), vec!["its takes or captures"])
    );
}

#[test]
fn the_inputs_hash_covers_the_voice_the_sources_and_the_tools() {
    let dir = drafted();
    let dir = dir.path();
    let p = project(dir, PROJECT);
    for line in ["main-1", "main-2", "main-3"] {
        record(dir, &p, line, line);
    }
    let record = built(sources(dir, &p));
    let again =
        |voice, tools: &Tools| release::inputs(voice, &record.sources, tools, &record.t1.timeline);
    assert_eq!(record.inputs, again(Voice::Takes, &record.tools));
    assert_ne!(record.inputs, again(Voice::Draft, &record.tools));
    assert_ne!(
        record.inputs,
        release::inputs(
            Voice::Takes,
            &record.sources,
            &record.tools,
            "another timeline"
        )
    );
    let mut tools = record.tools.clone();
    tools.ffmpeg = "9.0.3".into();
    assert_ne!(record.inputs, again(Voice::Takes, &tools));
    let mut tools = record.tools.clone();
    tools
        .fonts
        .insert("IBMPlexSans-Bold.otf".into(), "00".into());
    assert_ne!(record.inputs, again(Voice::Takes, &tools));
    // And a record round-trips through its file.
    record.write(dir).unwrap();
    assert_eq!(
        Record::read(dir, Voice::Takes, "long").unwrap(),
        Some(record)
    );
    assert_eq!(Record::read(dir, Voice::Draft, "long").unwrap(), None);
}
