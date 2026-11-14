//! The export panel (TODO.md, P7-VIDNI-07): ticking a platform changes
//! the cut's `platforms` in the project's file, and the next `vidni
//! final` makes that platform's folder and changes nothing else.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use vidni_app::app::{App, Tab};
use vidni_app::export::Export;
use vidni_core::{takes, voices};

/// The app on the project in `dir`, at its exports. A folder that is
/// asked for is noted, and no file manager is started.
fn window<'a>(dir: &Path) -> Harness<'a, App> {
    let mut app = App::new(None);
    app.open(dir);
    app.tab = Tab::Export;
    let mut export = Export::open(dir);
    export.set_opener(None);
    app.export = Some(export);
    let mut harness = Harness::builder()
        .with_size([1200.0, 1200.0])
        .build_ui_state(|ui, app: &mut App| app.show(ui), app);
    harness.run();
    harness
}

fn export<'h>(harness: &'h Harness<'_, App>) -> &'h Export {
    harness.state().export.as_ref().expect("the panel")
}

fn press(harness: &mut Harness<'_, App>, label: &str) {
    harness.get_by_label(label).click();
    harness.run();
}

/// The platforms cut `id` has ticked.
fn ticked(harness: &Harness<'_, App>, id: &str) -> Vec<String> {
    let cut = export(harness).cuts.iter().find(|cut| cut.id == id);
    cut.unwrap()
        .platforms
        .iter()
        .filter(|(_, on)| *on)
        .map(|(platform, _)| platform.clone())
        .collect()
}

fn vidni(args: &[&str]) -> (u8, String) {
    let args: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
    let (code, said) = vidni_app::jobs::now(&args);
    (
        code,
        said.into_iter().map(|(_, line)| line + "\n").collect(),
    )
}

#[test]
fn ticking_a_platform_changes_the_cuts_platforms_in_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let folder = dir.path();
    fs::write(folder.join("vidni.toml"), vidni_core::TEMPLATE).unwrap();
    fs::write(folder.join("page.html"), "<!doctype html>").unwrap();
    let file = folder.join("vidni.toml");
    let mut app = window(folder);
    // Each cut has the platforms that take its shape, and its defaults
    // are ticked.
    let offered = |app: &Harness<'_, App>, id: &str| -> Vec<String> {
        let cut = export(app).cuts.iter().find(|cut| cut.id == id).unwrap();
        cut.platforms.iter().map(|(p, _)| p.clone()).collect()
    };
    assert_eq!(offered(&app, "long"), ["youtube", "linkedin", "x"]);
    assert_eq!(
        offered(&app, "vertical"),
        [
            "youtube-shorts",
            "instagram-reels",
            "tiktok",
            "whatsapp-status"
        ]
    );
    assert_eq!(ticked(&app, "long"), ["youtube"]);
    assert_eq!(
        ticked(&app, "vertical"),
        ["youtube-shorts", "instagram-reels", "whatsapp-status"]
    );
    assert!(
        app.query_by_label("  no final build yet; `vidni final --cut long` builds and exports it")
            .is_some()
    );

    // A tick: one more platform in the file, and every other byte as it was.
    press(&mut app, "long: x");
    assert_eq!(ticked(&app, "long"), ["youtube", "x"]);
    assert_eq!(
        fs::read_to_string(&file).unwrap(),
        vidni_core::TEMPLATE.replace(
            "id = \"long\"\naspect = \"16:9\"\n",
            "id = \"long\"\naspect = \"16:9\"\nplatforms = [\"youtube\", \"x\"]\n"
        )
    );
    assert!(
        app.query_by_label("long: now for youtube, x; the next `vidni final` exports it for x")
            .is_some()
    );
    // The command line has it so.
    let (code, said) = vidni(&["exports", folder.to_str().unwrap(), "--cut", "long"]);
    assert_eq!(code, 0);
    assert!(said.starts_with("long (16:9), for youtube, x\n"), "{said}");
    // The other cut is as it was.
    assert_eq!(
        ticked(&app, "vertical"),
        ["youtube-shorts", "instagram-reels", "whatsapp-status"]
    );

    // Unticked, and ticked on the command line: the panel follows the file.
    press(&mut app, "long: youtube");
    assert_eq!(ticked(&app, "long"), ["x"]);
    let path = folder.to_str().unwrap();
    assert_eq!(
        vidni(&["exports", path, "--cut", "vertical", "--add", "tiktok"]).0,
        0
    );
    app.run();
    assert_eq!(
        ticked(&app, "vertical"),
        [
            "youtube-shorts",
            "instagram-reels",
            "tiktok",
            "whatsapp-status"
        ]
    );
    // A platform of the other shape isn't offered, and is refused by name.
    assert!(app.query_by_label("long: tiktok").is_none());
    let (code, said) = vidni(&["exports", path, "--cut", "long", "--add", "tiktok"]);
    assert_eq!(
        (code, said.as_str()),
        (1, "vidni: tiktok takes 9:16 and cut long is 16:9\n")
    );

    // A folder to upload from has a button, which opens that folder.
    let upload = folder.join("out/x/long");
    fs::create_dir_all(&upload).unwrap();
    app.run();
    press(&mut app, "Open out/x/long");
    assert_eq!(export(&app).opened.as_ref(), Some(&upload));
    assert!(app.query_by_label("Open out/youtube/long").is_none());
}

/// Every file under `dir`, by its path there, with its bytes' hash.
fn files(dir: &Path) -> BTreeMap<PathBuf, String> {
    let mut found = BTreeMap::new();
    let mut folders = vec![dir.to_path_buf()];
    while let Some(folder) = folders.pop() {
        for entry in fs::read_dir(&folder).unwrap().flatten() {
            let path = entry.path();
            match path.is_dir() {
                true => folders.push(path),
                false => {
                    let hash = vidni_core::capture::sha256_hex(&fs::read(&path).unwrap());
                    found.insert(path.strip_prefix(dir).unwrap().to_path_buf(), hash);
                }
            }
        }
    }
    found
}

/// Presses a button that starts a job, and waits for the job.
fn run(harness: &mut Harness<'_, App>, label: &str) -> (u8, Vec<String>) {
    harness.get_by_label(label).click();
    harness.step();
    let job = harness.state_mut().job.as_mut().expect("a job");
    let code = job.wait();
    let said = job.lines.iter().map(|(_, line)| line.clone()).collect();
    harness.run();
    (code, said)
}

#[test]
#[ignore = "needs Piper and its voice, Chromium and ffmpeg; run `just test-app`"]
fn a_ticked_platform_changes_the_next_final_build_and_only_its_exports() {
    let work = tempfile::tempdir().unwrap();
    let parent = work.path().to_str().unwrap();
    assert_eq!(vidni(&["new", "demo", "--in", parent]).0, 0);
    let folder = work.path().join("demo");
    let path = folder.to_str().unwrap();
    let (code, said) = vidni(&["draft", path]);
    assert_eq!(code, 0, "{said}");
    assert_eq!(vidni(&["ratify", path]).0, 0);
    // Each line's draft clip stands in for the creator's take.
    let text = fs::read_to_string(folder.join("vidni.toml")).unwrap();
    let project = vidni_core::validate::check(&text, Some(&folder)).unwrap();
    let lock = voices::read_lock(&folder).unwrap();
    let pack = voices::find(&project.voice.draft, None).unwrap().unwrap();
    for (line, take) in project.lines.iter().zip(takes::status(&project, &folder)) {
        let clip = voices::drafted(&folder, &lock, &pack, &project.voice, line).unwrap();
        fs::create_dir_all(take.path.parent().unwrap()).unwrap();
        fs::copy(clip, take.path).unwrap();
    }

    // A final build of both cuts, from the panel.
    let mut app = window(&folder);
    for cut in ["long", "vertical"] {
        let (code, said) = run(&mut app, &format!("Final {cut}"));
        assert_eq!(code, 0, "{said:#?}");
    }
    let out = folder.join("out");
    let first = files(&out);
    let of = |all: &BTreeMap<PathBuf, String>, folder: &str| -> BTreeMap<PathBuf, String> {
        all.iter()
            .filter(|(path, _)| path.starts_with(folder))
            .map(|(path, hash)| (path.clone(), hash.clone()))
            .collect()
    };
    assert!(!of(&first, "youtube/long").is_empty());
    assert!(of(&first, "x").is_empty());
    // What QA measured and each folder are listed, with a button for each.
    assert_eq!(app.query_all_by_label("  QA: nothing wrong").count(), 2);
    assert!(
        app.query_by_label_contains("  youtube: out/youtube/long (")
            .is_some()
    );
    assert!(app.query_by_label("Open out/youtube/long").is_some());
    assert!(
        app.query_by_label("Open out/youtube-shorts/vertical")
            .is_some()
    );
    let master = fs::metadata(out.join("long.mp4"))
        .unwrap()
        .modified()
        .unwrap();
    let other = folder.join("build/final/vertical.build.json");
    let other_built = fs::metadata(&other).unwrap().modified().unwrap();

    // A platform is ticked. Nothing is built yet: the folder is promised.
    press(&mut app, "long: x");
    assert_eq!(files(&out), first);
    assert!(
        app.query_by_label("  x: not exported yet; the next `vidni final` exports it")
            .is_some()
    );
    // The cut is still ratified: its platforms are no part of what it says.
    let (_, status) = vidni(&["status", path]);
    assert!(status.starts_with("long      ratified\n"), "{status}");

    // The next final build: that platform's folder, and nothing else changed.
    let (code, said) = run(&mut app, "Final long");
    assert_eq!(code, 0, "{said:#?}");
    let second = files(&out);
    let added = of(&second, "x/long");
    assert!(
        added.contains_key(Path::new("x/long/long.captioned.mp4")),
        "{added:?}"
    );
    assert!(
        added.contains_key(Path::new("x/long/metadata.md")),
        "{added:?}"
    );
    let rest: BTreeMap<PathBuf, String> = second
        .iter()
        .filter(|(path, _)| !path.starts_with("x"))
        .map(|(path, hash)| (path.clone(), hash.clone()))
        .collect();
    assert_eq!(rest, first);
    // The cut is built again, since its platforms decide how loud it is
    // and what its layout keeps clear of; here they decide the same, so
    // the build is the same bytes and the delivered file isn't touched.
    assert_eq!(
        fs::metadata(out.join("long.mp4"))
            .unwrap()
            .modified()
            .unwrap(),
        master
    );
    // The other cut wasn't built: its record is the one from before.
    assert_eq!(
        fs::metadata(&other).unwrap().modified().unwrap(),
        other_built
    );
    assert!(app.query_by_label("Open out/x/long").is_some());
    press(&mut app, "Open out/x/long");
    assert_eq!(export(&app).opened, Some(out.join("x/long")));

    // Unticked: the next final build takes the folder away, and the rest
    // is what the first build made.
    press(&mut app, "long: x");
    assert_eq!(files(&out), second);
    let (code, said) = run(&mut app, "Final long");
    assert_eq!(code, 0, "{said:#?}");
    assert_eq!(files(&out), first);
    assert!(app.query_by_label("Open out/x/long").is_none());
}
