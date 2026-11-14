//! A project's assets, their backups, and what can be made again (TODO.md,
//! P4-VIDNI-05), on a project whose files are stand-ins.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use vidni_core::assets::{self, Asset, Rebuildable, State};
use vidni_core::capture;
use vidni_core::project::Project;
use vidni_core::ratify::{self, Lock};
use vidni_core::release::{self, Output, Record, Releases, Sources, T1, Tools, Voice};
use vidni_core::takes;
use vidni_core::validate::check;

/// A long cut with a still and a card, and a short with a still of its own
/// that is never ratified.
const PROJECT: &str = r#"schema = 1

[meta]
title = "Assets"
default_scene = "page"

[[cuts]]
id = "long"
aspect = "16:9"
lines = ["main-1", "main-2"]

[[cuts]]
id = "short"
aspect = "9:16"
lines = ["hook-1"]

[sections.main]

[[lines]]
id = "main-1"
section = "main"
show = "A still of the page."

[[lines]]
id = "main-2"
section = "main"
show = "A card."
scene = "card"

[[lines]]
id = "hook-1"
section = "main"
show = "Only the short has this still."
highlight = ["title"]

[scenes.page]
kind = "web"
url = "https://example.com"

[scenes.page.targets]
title = { css = "h1" }

[scenes.card]
kind = "page"
url = "card.html"
"#;

fn write(path: &Path, bytes: &[u8]) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

/// A project folder as a creator's is after a first release: every
/// capture taken, the long cut ratified, each line recorded, and around
/// them what vidni has made and what a session left behind.
fn project() -> (tempfile::TempDir, PathBuf, Project, Lock) {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("my-video");
    write(&dir.join("card.html"), b"<!doctype html>");
    let p = check(PROJECT, Some(&dir)).unwrap_or_else(|p| panic!("{p:?}"));
    let cuts: Vec<_> = p.cuts.iter().collect();
    for c in capture::plan(&p, &dir, &cuts) {
        write(&capture::path(&dir, &c.key), c.key.as_bytes());
        if matches!(c.source, capture::Source::Line(_)) {
            write(&capture::frames_path(&dir, &c.key), br#"{"targets":{}}"#);
        }
        if let Some(file) = &c.file {
            let made_from = [(file.clone(), capture::file_sha256(&dir, file))].into();
            write(
                &capture::files_path(&dir, &c.key),
                capture::files_json(&made_from).as_bytes(),
            );
        }
    }
    let mut lock = Lock::default();
    ratify::ratify(&p, &dir, &mut lock, &["long".into()]).unwrap();
    for take in takes::status(&p, &dir) {
        write(&take.path, format!("a take of {}", take.id).as_bytes());
        write(
            &takes::raw(&dir, &take.id, &take.hash, 1),
            format!("a raw take of {}", take.id).as_bytes(),
        );
    }
    for (path, bytes) in [
        ("takes/_room/tone.wav", "room tone"),
        // What a session left behind.
        ("takes/main-1/abcd.wav.new", "a take never kept"),
        ("takes/_room/room.raw", "a mic check"),
        // What vidni made, and can make again.
        ("build/voice/main-1/key-7.wav", "a draft clip"),
        ("build/draft/long.mp4", "a draft"),
        ("build/draft/long.build.json", "{}"),
        ("build/final/long.mp4", "mp4"),
        ("build/captures/0123.png", "a capture of a scene as it was"),
        ("build/captures/4567.part", "a capture cut short"),
        ("build/resolved.json", "{}"),
        // What was delivered.
        ("out/long.mp4", "mp4"),
        ("out/long.en.srt", "srt"),
        ("out/long.en.vtt", "vtt"),
        // And the creator's own files.
        ("notes.md", "notes"),
    ] {
        write(&dir.join(path), bytes.as_bytes());
    }
    (root, dir, p, lock)
}

/// The long cut's release, whose record names the files in `out/`.
fn released() -> Releases {
    let hash = |s: &str| capture::sha256_hex(s.as_bytes());
    let record = Record {
        cut: "long".into(),
        voice: Voice::Takes,
        inputs: hash("inputs"),
        sources: Sources {
            content: hash("content"),
            audio: hash("audio"),
            look: hash("look"),
            captures: hash("captures"),
        },
        t1: T1 {
            timeline: hash("timeline"),
            srt: hash("srt"),
            vtt: hash("vtt"),
            pcm: hash("pcm"),
            pcm_normalized: hash("normalized"),
            chapters: None,
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
        tools: Tools {
            vidni: "0.1.0".into(),
            ffmpeg: "9.0.2".into(),
            fonts: BTreeMap::new(),
            engine: serde_json::json!({}),
        },
    };
    let mut releases = Releases::default();
    release::approve(&mut releases, &record, "2026-11-14");
    releases
}

/// Every file under `dir`, by its path from it, sorted.
fn files(dir: &Path) -> Vec<String> {
    let mut found = Vec::new();
    let mut folders = vec![dir.to_path_buf()];
    while let Some(folder) = folders.pop() {
        for entry in fs::read_dir(folder).into_iter().flatten().flatten() {
            let path = entry.path();
            match path.is_dir() {
                true => folders.push(path),
                false => found.push(release::relative(dir, &path)),
            }
        }
    }
    found.sort();
    found
}

#[test]
fn the_assets_are_the_takes_and_what_a_ratified_cut_froze() {
    let (_root, dir, p, lock) = project();
    let paths = assets::paths(&dir, &lock).unwrap();
    // The long cut froze its still, with its frames, and its card, with
    // the record of its files. The short cut isn't ratified, so its still
    // isn't frozen.
    let still = &capture::line_keys(&p)["main-1"];
    let card = capture::page_capture(&p, &p.cuts[0], "card").unwrap().key;
    let mut captures = [
        format!("build/captures/{card}.files.json"),
        format!("build/captures/{card}.png"),
        format!("build/captures/{still}.json"),
        format!("build/captures/{still}.png"),
    ];
    captures.sort();
    let frozen: Vec<&String> = paths.iter().filter(|p| p.starts_with("build/")).collect();
    assert_eq!(frozen, captures.iter().collect::<Vec<_>>());
    // Every take: kept, raw, and the room tone; and nothing a session
    // left behind.
    let taken: Vec<&str> = paths
        .iter()
        .filter(|p| p.starts_with("takes/"))
        .map(|p| p.rsplit('/').next().unwrap())
        .collect();
    assert_eq!(taken.len(), 7, "{taken:?}");
    assert!(taken.contains(&"tone.wav") && !taken.contains(&"abcd.wav.new"));
    assert_eq!(paths.len(), 11);

    // The list is each of them with its hash, sorted, as sha256sum reads it.
    let snapshot = assets::snapshot(&dir, &lock).unwrap();
    assert!(assets::write_lock(&dir, &snapshot).unwrap());
    assert_eq!(assets::read_lock(&dir).unwrap(), snapshot);
    assert_eq!(
        snapshot[0],
        Asset {
            path: captures[0].clone(),
            sha256: assets::sha256_of(&dir.join(&captures[0])).unwrap()
        }
    );
    assert_eq!(assets::faults(&dir, &snapshot), []);
    // A frozen capture that is gone can't be listed.
    fs::remove_file(dir.join(&captures[1])).unwrap();
    let error = assets::snapshot(&dir, &lock).unwrap_err();
    assert!(error.contains(&captures[1]), "{error}");
    assert_eq!(
        assets::faults(&dir, &snapshot),
        [(captures[1].clone(), State::Missing)]
    );
}

#[test]
fn a_backup_holds_the_assets_and_the_delivered_files_and_restores_them() {
    let (root, dir, p, lock) = project();
    let releases = released();
    let snapshot = assets::snapshot(&dir, &lock).unwrap();
    assets::write_lock(&dir, &snapshot).unwrap();
    // A backup's folder is named as the project's is, under each place.
    let places = [root.path().join("disk"), root.path().join("elsewhere")];
    let folders: Vec<PathBuf> = places
        .iter()
        .map(|place| assets::folder(place, &dir).unwrap())
        .collect();
    assert_eq!(folders[0], places[0].join("my-video"));
    for folder in &folders {
        let backed = assets::backup(&p, &dir, &snapshot, &releases, folder).unwrap();
        assert_eq!((backed.assets, backed.copied), (11, 11));
        assert_eq!((backed.delivered, backed.delivered_copied), (3, 3));
        assert_eq!(assets::faults(folder, &snapshot), []);
    }
    // It holds the list, the assets at their paths, and the release's
    // delivered files under its tag; and nothing else of the project.
    let held = files(&folders[0]);
    assert_eq!(held.len(), 1 + 11 + 3, "{held:?}");
    assert!(held.contains(&"assets.lock".to_string()));
    for delivered in ["long.mp4", "long.en.srt", "long.en.vtt"] {
        assert!(
            held.contains(&format!("releases/long-r1/{delivered}")),
            "{held:?}"
        );
    }
    assert!(
        !held
            .iter()
            .any(|f| f.contains("notes") || f.contains("draft"))
    );
    // Backed up again, nothing is copied.
    let again = assets::backup(&p, &dir, &snapshot, &releases, &folders[0]).unwrap();
    assert_eq!((again.copied, again.delivered_copied), (0, 0));

    // The assets are wiped, and the delivered files with them.
    fs::remove_dir_all(dir.join("takes")).unwrap();
    fs::remove_dir_all(dir.join("build")).unwrap();
    fs::remove_dir_all(dir.join("out")).unwrap();
    assert_eq!(assets::faults(&dir, &snapshot).len(), 11);
    // One backup has lost a take, and holds another that isn't the take.
    let lost = &snapshot
        .iter()
        .find(|a| a.path.ends_with("tone.wav"))
        .unwrap()
        .path;
    let spoiled = &snapshot
        .iter()
        .rev()
        .find(|a| a.path.contains("/raw/"))
        .unwrap()
        .path;
    fs::remove_file(folders[0].join(lost)).unwrap();
    fs::write(folders[0].join(spoiled), b"not the take").unwrap();
    assert_eq!(
        assets::faults(&folders[0], &snapshot),
        [
            (lost.clone(), State::Missing),
            (spoiled.clone(), State::Changed)
        ]
    );
    // Restored: every file is back as listed, the two from the other backup.
    let restored = assets::restore(&p, &dir, &snapshot, &releases, &folders).unwrap();
    assert_eq!((restored.restored, restored.already), (14, 0));
    assert_eq!((restored.differing, restored.missing), (vec![], vec![]));
    assert_eq!(assets::faults(&dir, &snapshot), []);
    assert_eq!(fs::read(dir.join("out/long.mp4")).unwrap(), b"mp4");
    // Restored again, everything is there already.
    let again = assets::restore(&p, &dir, &snapshot, &releases, &folders).unwrap();
    assert_eq!((again.restored, again.already), (0, 14));

    // A file that is there and isn't the listed one is left as it is: it
    // may be a take recorded since the backup. A file no backup holds is
    // said to be missing.
    let take = takes::status(&p, &dir).remove(0).path;
    fs::write(&take, b"a newer take").unwrap();
    fs::remove_file(dir.join(lost)).unwrap();
    fs::remove_file(folders[1].join(lost)).unwrap();
    let partly = assets::restore(&p, &dir, &snapshot, &releases, &folders).unwrap();
    assert_eq!(partly.differing, [release::relative(&dir, &take)]);
    assert_eq!(partly.missing, std::slice::from_ref(lost));
    assert_eq!(fs::read(&take).unwrap(), b"a newer take");
    // A delivered file that isn't the release's isn't archived.
    fs::write(dir.join("out/long.mp4"), b"another build").unwrap();
    let other = root.path().join("third/my-video");
    let backed = assets::backup(&p, &dir, &[], &releases, &other).unwrap();
    assert_eq!((backed.delivered, backed.delivered_copied), (2, 2));
    assert!(!other.join("releases/long-r1/long.mp4").exists());
}

#[test]
fn what_can_be_made_again_is_never_a_take_or_a_frozen_capture() {
    let (_root, dir, p, lock) = project();
    let found = assets::rebuildable(&p, &dir, &lock).unwrap();
    let listed: Vec<(Rebuildable, &str)> = found
        .iter()
        .map(|(kind, path, _)| (*kind, path.as_str()))
        .collect();
    assert_eq!(
        listed,
        [
            (Rebuildable::Voice, "build/voice/main-1/key-7.wav"),
            (Rebuildable::Draft, "build/draft/long.build.json"),
            (Rebuildable::Draft, "build/draft/long.mp4"),
            (Rebuildable::Final, "build/final/long.mp4"),
            // A capture of a scene as it was: no cut shows it now.
            (Rebuildable::Capture, "build/captures/0123.png"),
            (Rebuildable::Leftover, "build/captures/4567.part"),
            (Rebuildable::Leftover, "takes/_room/room.raw"),
            (Rebuildable::Leftover, "takes/main-1/abcd.wav.new"),
        ]
    );
    assert_eq!(found[0].2, "a draft clip".len() as u64);
    // The short cut's still isn't frozen, but the cut shows it, so it
    // stays; so do the takes, the frozen captures, and what was delivered.
    let assets = assets::paths(&dir, &lock).unwrap();
    assert!(
        !listed
            .iter()
            .any(|(_, path)| assets.iter().any(|a| a == path))
    );
    let hook = &capture::line_keys(&p)["hook-1"];
    assert!(!listed.iter().any(|(_, path)| path.contains(hook.as_str())));
    assert!(!listed.iter().any(|(_, path)| path.starts_with("out/")));
    // With the short cut gone from the project, its still is no cut's.
    let without = PROJECT.replace(
        "[[cuts]]\nid = \"short\"\naspect = \"9:16\"\nlines = [\"hook-1\"]\n",
        "",
    );
    let p = check(&without, Some(&dir)).unwrap();
    let found = assets::rebuildable(&p, &dir, &lock).unwrap();
    let captures = found
        .iter()
        .filter(|(kind, _, _)| *kind == Rebuildable::Capture)
        .count();
    assert_eq!(captures, 3, "its picture and its frames, and the old one");
}
