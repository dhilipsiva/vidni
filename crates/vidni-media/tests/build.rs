//! A cut's build, its gates and its record (TODO.md, P3-VIDNI-06), on a
//! small project whose captures are stand-in pictures and whose audio is a
//! tone. It needs ffmpeg and ffprobe, as the encoder's tests do.

use std::fs;
use std::path::{Path, PathBuf};
use vidni_core::capture::{self, Source};
use vidni_core::project::{Cut, Project};
use vidni_core::ratify::{self, Lock};
use vidni_core::release::{self, Approval, Record, Voice};
use vidni_core::{spoken, takes, voices};
use vidni_media::build::{self, Faces, Setup, Step, Stop};
use vidni_media::capture::Frames;
use vidni_media::compose::image::Image;

/// A long cut of a web line and a card, and a short of the card alone.
/// The gaps are short, so that each build is a few seconds of video.
const PROJECT: &str = r#"schema = 1

[meta]
title = "A build"
subtitle = "a test"
default_scene = "page"

[timing]
lead_in = 0.1
line_gap = 0.2
section_gap = 0.2
tail = 0.2

[[cuts]]
id = "long"
aspect = "16:9"
captions = "burned"

[[cuts]]
id = "short"
aspect = "9:16"
lines = ["card-1"]

[sections.main]
label = "main"

[[lines]]
id = "page-1"
section = "main"
show = "Language models are good at reading prose."
camera = "left"

[[lines]]
id = "card-1"
section = "main"
show = "Language models are good at reading prose!"
scene = "card"

[scenes.page]
kind = "web"
url = "https://example.com"
viewport = [1280, 720]

[scenes.page.targets]
side = { css = "aside" }

[scenes.page.cameras]
left = { union = ["side"] }

[scenes.card]
kind = "page"
url = "card.html"
"#;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// The build's tools, with one pinned face standing for all three.
fn setup() -> Setup {
    Setup::with(
        Faces::all(&fixture("fonts/IBMPlexSans-Bold.otf")),
        vidni_core::theme::Theme::default_theme(),
    )
    .unwrap()
}

/// The stages of a build of the long cut.
const LONG_STAGES: [&str; 4] = [
    "timeline and captions",
    "narration",
    "long.mp4",
    "review frames",
];

fn project(dir: &Path, text: &str) -> Project {
    vidni_core::validate::check(text, Some(dir)).unwrap_or_else(|p| panic!("{p:?}"))
}

fn cut<'p>(p: &'p Project, id: &str) -> &'p Cut {
    p.cuts.iter().find(|c| c.id == id).unwrap()
}

/// A project folder with every capture taken: a still of the page with its
/// frames, and the card at each cut's size.
fn drafted(text: &str) -> (tempfile::TempDir, Project) {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("card.html"), "<!doctype html>").unwrap();
    let p = project(dir.path(), text);
    let cuts: Vec<&Cut> = p.cuts.iter().collect();
    for c in capture::plan(&p, dir.path(), &cuts) {
        let path = capture::path(dir.path(), &c.key);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        match &c.source {
            Source::Page { size, .. } => {
                Image::new(size[0], size[1], &[29, 53, 87])
                    .save_png(&path)
                    .unwrap();
            }
            Source::Line(_) => {
                Image::new(1280, 720, &[200, 60, 20])
                    .save_png(&path)
                    .unwrap();
                let frames = Frames {
                    viewport: [1280, 720],
                    scale: 1,
                    targets: [("side".to_string(), vec![[40.0, 60.0, 500.0, 400.0]])].into(),
                    highlights: Default::default(),
                    browser: "a stand-in".into(),
                    rows: None,
                };
                let json = serde_json::to_string(&frames).unwrap();
                fs::write(capture::frames_path(dir.path(), &c.key), json).unwrap();
            }
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
    (dir, p)
}

/// A clip of two and a half seconds: a 440 Hz tone at `level` of full
/// scale, which loudness normalization brings to its target; or, with
/// `click`, silence with one loud sample, which nothing can.
fn clip(dir: &Path, name: &str, level: f64, click: bool) -> PathBuf {
    let path = dir.join(name);
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 22050,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(&path, spec).unwrap();
    for n in 0..55125u32 {
        let t = f64::from(n) / 22050.0;
        let sample = match click {
            true => f64::from(u8::from(n == 22050)),
            false => (t * 440.0 * std::f64::consts::TAU).sin() * level,
        };
        writer.write_sample((sample * 32767.0) as i16).unwrap();
    }
    writer.finalize().unwrap();
    path
}

fn tone(dir: &Path) -> PathBuf {
    clip(dir, "tone.wav", 0.1, false)
}

/// Gives every line a take: the tone, or `sound` in its place.
fn record_takes(dir: &Path, p: &Project, sound: Option<&Path>) {
    let tone = tone(dir);
    for take in takes::status(p, dir) {
        fs::create_dir_all(take.path.parent().unwrap()).unwrap();
        fs::copy(sound.unwrap_or(&tone), &take.path).unwrap();
    }
}

/// Stands in for the draft voice: the tone as each line's draft clip, kept
/// in voice.lock.
fn speak(dir: &Path, p: &Project) -> voices::Pack {
    let pack = voices::all().remove(0);
    let tone = tone(dir);
    let mut lock = voices::Lock::default();
    for line in &p.lines {
        let given = spoken::synthetic(line, &p.voice);
        let key = voices::clip_key(&given, &pack, p.voice.length_scale);
        let clip = voices::clip_path(dir, &line.id, &key, 7);
        fs::create_dir_all(clip.parent().unwrap()).unwrap();
        fs::copy(&tone, clip).unwrap();
        lock.lines
            .insert(line.id.clone(), voices::Kept { key, seed: 7 });
    }
    voices::write_lock(dir, &lock).unwrap();
    pack
}

fn ratified(dir: &Path, p: &Project) -> Lock {
    let mut lock = Lock::default();
    ratify::ratify(p, dir, &mut lock, &[]).unwrap();
    lock
}

fn quiet() -> impl FnMut(Step) {
    |_| {}
}

fn refused(stop: Result<impl std::fmt::Debug, Stop>) -> Vec<String> {
    match stop {
        Err(Stop::Refused(reasons)) => reasons,
        other => panic!("expected a refusal, not {other:?}"),
    }
}

#[test]
fn a_final_build_records_what_it_made_and_repeats() {
    let (dir, p) = drafted(PROJECT);
    let dir = dir.path();
    record_takes(dir, &p, None);
    let lock = ratified(dir, &p);
    let long = cut(&p, "long");
    let mut stages = Vec::new();
    let mut watch = |step: Step| {
        if let Step::Stage(stage) = step {
            stages.push(stage.to_string());
        }
    };
    let record = build::final_build(&p, dir, &lock, long, &setup(), &mut watch).unwrap();
    assert_eq!(stages, LONG_STAGES);
    // A frame to look at for its one section, and a sheet of its lines,
    // for the platform it is made for.
    let review = dir.join("build/final/review/long/youtube");
    let mut frames: Vec<String> = fs::read_dir(&review)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    frames.sort();
    assert_eq!(frames, ["00-main.png", "lines-01.png"]);

    // One picture, with its captions burned in, measured by QA.
    assert_eq!(record.outputs.keys().collect::<Vec<_>>(), ["long.mp4"]);
    let output = &record.outputs["long.mp4"];
    assert!(output.captions && output.problems.is_empty(), "{output:?}");
    assert!(
        output
            .qa
            .starts_with("1920x1080 h264 30/1 fps; aac 48000 Hz 2 ch; "),
        "{}",
        output.qa
    );
    // Two clips of 2.5 s with a 0.2 s gap, a 0.1 s lead-in and a 0.2 s tail.
    assert_eq!(output.frame_count, 165);
    assert_eq!((record.cut.as_str(), record.voice), ("long", Voice::Takes));

    // The record names the files as they are, in build/final and in out.
    let made = dir.join("build/final");
    let hash = |path: PathBuf| release::file_hash(&path).unwrap();
    assert_eq!(hash(made.join("long.mp4")), output.mp4);
    assert_eq!(hash(made.join("long.timeline.json")), record.t1.timeline);
    assert_eq!(hash(made.join("long.narration.wav")), record.t1.pcm);
    assert!(dir.join("build/resolved.json").is_file());
    for (name, sha) in [
        ("long.mp4", &output.mp4),
        ("long.en.srt", &record.t1.srt),
        ("long.en.vtt", &record.t1.vtt),
    ] {
        assert_eq!(&hash(dir.join("out").join(name)), sha, "{name}");
    }
    assert_eq!(
        Record::read(dir, Voice::Takes, "long").unwrap().as_ref(),
        Some(&record)
    );
    // The timeline names each clip from the project folder, never by where
    // that is, and it is the timeline the build's inputs were worked out with.
    let timeline = fs::read_to_string(made.join("long.timeline.json")).unwrap();
    assert!(timeline.contains("\"wav\": \"takes/page-1/"), "{timeline}");
    assert!(!timeline.contains(&dir.display().to_string()));
    assert_eq!(
        record.inputs,
        release::inputs(
            Voice::Takes,
            &record.sources,
            &record.tools,
            &record.t1.timeline
        )
    );

    // Built again, every hash is the same: T1 and T2 repeat on one host.
    let afresh = Setup {
        again: true,
        ..setup()
    };
    let mut stages = Vec::new();
    let mut watch = |step: Step| {
        if let Step::Stage(stage) = step {
            stages.push(stage.to_string());
        }
    };
    let again = build::final_build(&p, dir, &lock, long, &afresh, &mut watch).unwrap();
    assert_eq!(stages, LONG_STAGES);
    assert_eq!(again, record);
    assert_eq!(record.differences(&again), Vec::<String>::new());
}

#[test]
fn a_build_is_kept_until_an_input_or_a_file_it_made_changes() {
    let (dir, p) = drafted(PROJECT);
    let dir = dir.path();
    record_takes(dir, &p, None);
    let lock = ratified(dir, &p);
    // A final build of the short cut, and what it did: the stages it went
    // through, or `kept` when it rendered nothing.
    let build = |p: &Project, setup: &Setup| -> (Record, Vec<String>) {
        let mut steps = Vec::new();
        let mut watch = |step: Step| match step {
            Step::Stage(stage) => steps.push(stage.to_string()),
            Step::Kept => steps.push("kept".to_string()),
            Step::Frames { .. } => {}
        };
        let record = build::final_build(p, dir, &lock, cut(p, "short"), setup, &mut watch).unwrap();
        (record, steps)
    };
    let stages = [
        "timeline and captions",
        "narration",
        "short.mp4",
        "review frames",
    ];
    let (first, steps) = build(&p, &setup());
    assert_eq!(steps, stages);
    let made = dir.join("build/final/short.mp4");
    let delivered = dir.join("out/short.mp4");
    let written = |path: &Path| fs::metadata(path).unwrap().modified().unwrap();
    let times = (written(&made), written(&delivered));

    // Nothing has changed: the build is kept, and no file is written again.
    let (kept, steps) = build(&p, &setup());
    assert_eq!(steps, ["kept"]);
    assert_eq!(kept, first);
    assert_eq!((written(&made), written(&delivered)), times);
    // A delivered file that is gone is delivered again, from the build kept.
    fs::remove_file(&delivered).unwrap();
    assert_eq!(build(&p, &setup()).1, ["kept"]);
    assert_eq!(
        release::file_hash(&delivered).unwrap(),
        first.outputs["short.mp4"].mp4
    );

    // A file of the build's that isn't what the build made: built again,
    // into the same files.
    fs::write(&made, b"cut short").unwrap();
    let (again, steps) = build(&p, &setup());
    assert_eq!(steps, stages);
    assert_eq!(again, first);
    assert_eq!(
        release::file_hash(&made).unwrap(),
        first.outputs["short.mp4"].mp4
    );
    for name in ["short.narration.wav", "short.en.srt"] {
        fs::remove_file(dir.join("build/final").join(name)).unwrap();
    }
    assert_eq!(build(&p, &setup()).1, stages);
    assert_eq!(build(&p, &setup()).1, ["kept"]);

    // An input changed: here a gap, which a cut's ratification doesn't cover.
    let slower = project(dir, &PROJECT.replace("tail = 0.2", "tail = 0.4"));
    let (longer, steps) = build(&slower, &setup());
    assert_eq!(steps, stages);
    assert_ne!(longer.inputs, first.inputs);
    assert_eq!(build(&slower, &setup()).1, ["kept"]);
    // Changed back, the build on disk is the other gap's.
    let (back, steps) = build(&p, &setup());
    assert_eq!(steps, stages);
    assert_eq!(back, first);

    // A draft is kept the same way, apart from the final build.
    let pack = speak(dir, &p);
    let draft = |setup: &Setup| -> Vec<String> {
        let mut steps = Vec::new();
        let mut watch = |step: Step| match step {
            Step::Stage(stage) => steps.push(stage.to_string()),
            Step::Kept => steps.push("kept".to_string()),
            Step::Frames { .. } => {}
        };
        build::draft(&p, dir, cut(&p, "short"), &pack, setup, &mut watch).unwrap();
        steps
    };
    assert_eq!(draft(&setup()), stages);
    assert_eq!(draft(&setup()), ["kept"]);
    // And either is built all the same when that is asked for.
    let afresh = Setup {
        again: true,
        ..setup()
    };
    assert_eq!(draft(&afresh), stages);
    assert_eq!(build(&p, &setup()).1, ["kept"]);
}

#[test]
fn captions_are_burned_in_kept_in_files_or_both_as_the_cut_asks() {
    let pictures = |captions: &str, voice: Voice| {
        let text = PROJECT.replace(
            "captions = \"burned\"",
            &format!("captions = \"{captions}\""),
        );
        let (dir, p) = drafted(&text);
        let dir = dir.path();
        let long = cut(&p, "long");
        let record = match voice {
            Voice::Takes => {
                record_takes(dir, &p, None);
                let lock = ratified(dir, &p);
                build::final_build(&p, dir, &lock, long, &setup(), &mut quiet()).unwrap()
            }
            Voice::Draft => {
                let pack = speak(dir, &p);
                build::draft(&p, dir, long, &pack, &setup(), &mut quiet()).unwrap()
            }
        };
        let delivered = fs::read_dir(dir.join("out")).map_or(0, |d| d.count());
        (record, delivered)
    };
    // Both: a clean master and a captioned variant, with the caption files.
    let (both, delivered) = pictures("both", Voice::Takes);
    let captions: Vec<(&str, bool)> = both
        .outputs
        .iter()
        .map(|(name, o)| (name.as_str(), o.captions))
        .collect();
    assert_eq!(
        captions,
        [("long.captioned.mp4", true), ("long.mp4", false)]
    );
    assert_ne!(
        both.outputs["long.mp4"].frames,
        both.outputs["long.captioned.mp4"].frames
    );
    assert_eq!(delivered, 4);
    // A draft is one picture to watch, with the captions in it.
    let (draft, delivered) = pictures("both", Voice::Draft);
    assert_eq!(draft.outputs.len(), 1);
    assert!(draft.outputs["long.mp4"].captions);
    assert_eq!(delivered, 0, "a draft delivers nothing");
    // The same frames as the captioned variant, whichever voice spoke: the clips are the same.
    assert_eq!(
        draft.outputs["long.mp4"].frames,
        both.outputs["long.captioned.mp4"].frames
    );
    // Files only: the picture is clean.
    let (file, _) = pictures("file", Voice::Takes);
    assert_eq!(file.outputs.len(), 1);
    assert!(!file.outputs["long.mp4"].captions);
    assert_eq!(
        file.outputs["long.mp4"].frames,
        both.outputs["long.mp4"].frames
    );
}

#[test]
fn a_build_is_refused_without_its_audio_its_captures_or_a_size_it_can_draw() {
    let (dir, p) = drafted(PROJECT);
    let dir = dir.path();
    let (long, short) = (cut(&p, "long"), cut(&p, "short"));
    let pack = voices::all().remove(0);
    let plan = |cut, voice| build::plan(&p, dir, cut, voice, Some(&pack), &setup()).map(|_| ());
    // Nothing spoken, nothing recorded.
    assert_eq!(
        refused(plan(long, Voice::Draft)),
        [
            "cut long: the draft voice hasn't spoken 2 of its 2 lines as they are now (the first: page-1); `vidni draft` speaks them"
        ]
    );
    assert_eq!(
        refused(plan(long, Voice::Takes)),
        ["cut long: 2 of its 2 lines have no take: page-1, card-1"]
    );
    // An unratified cut never reaches the build.
    record_takes(dir, &p, None);
    assert_eq!(
        refused(build::final_build(
            &p,
            dir,
            &Lock::default(),
            long,
            &setup(),
            &mut quiet()
        )),
        ["cut long isn't ratified; `vidni ratify --cut long` freezes it"]
    );
    assert!(!dir.join("build/final").exists());
    // A capture that is gone.
    let lock = ratified(dir, &p);
    let card = capture::page_capture(&p, short, "card").unwrap();
    fs::remove_file(capture::path(dir, &card.key)).unwrap();
    assert_eq!(
        refused(plan(short, Voice::Takes)),
        [
            "cut short: 1 of 1 captures isn't taken yet (the first: page card at 1080x1920 for 9x16); `vidni draft` takes them"
        ]
    );
    assert!(
        refused(build::final_build(
            &p,
            dir,
            &lock,
            short,
            &setup(),
            &mut quiet()
        ))[0]
            .starts_with("cut short has changed since it was ratified: capture ")
    );
    // A size the compositor doesn't draw yet.
    let small = project(
        dir,
        &PROJECT.replace("captions = \"burned\"", "size = [1280, 720]"),
    );
    assert_eq!(
        refused(
            build::plan(
                &small,
                dir,
                cut(&small, "long"),
                Voice::Takes,
                None,
                &setup()
            )
            .map(|_| ())
        ),
        ["cut long is 1280x720; vidni draws 16:9 at 1920x1080 and 9:16 at 1080x1920 for now"]
    );
}

#[test]
fn a_final_build_that_fails_qa_is_kept_back() {
    let (dir, p) = drafted(PROJECT);
    let dir = dir.path();
    // Takes of one click in silence: no gain brings them to the target
    // loudness without clipping.
    record_takes(dir, &p, Some(&clip(dir, "click.wav", 0.0, true)));
    let lock = ratified(dir, &p);
    let short = cut(&p, "short");
    let reasons = refused(build::final_build(
        &p,
        dir,
        &lock,
        short,
        &setup(),
        &mut quiet(),
    ));
    assert_eq!(
        reasons[0],
        "cut short failed QA, so it isn't delivered; its files are in build/final"
    );
    assert!(
        reasons.len() == 2 && reasons[1].starts_with("short.mp4: its loudness "),
        "{reasons:?}"
    );
    // The build is there to look at, with its problems on record, and nothing is in out.
    assert!(dir.join("build/final/short.mp4").is_file());
    let record = Record::read(dir, Voice::Takes, "short").unwrap().unwrap();
    assert_eq!(record.problems().len(), 1);
    assert!(!dir.join("out").exists());
    // And it can't be approved.
    assert_eq!(
        refused(build::approve(
            &p,
            dir,
            &lock,
            short,
            &setup(),
            "2026-11-14"
        )),
        ["cut short's final build failed QA; `vidni final --cut short` builds it again"]
    );
}

#[test]
fn approving_needs_the_final_build_as_it_was_delivered() {
    let (dir, p) = drafted(PROJECT);
    let dir = dir.path();
    record_takes(dir, &p, None);
    let lock = ratified(dir, &p);
    let short = cut(&p, "short");
    let approve = |p: &Project, lock: &Lock| {
        build::approve(p, dir, lock, cut(p, "short"), &setup(), "2026-11-14")
    };
    assert_eq!(
        refused(approve(&p, &lock)),
        ["cut short has no final build; `vidni final --cut short` builds it"]
    );
    let record = build::final_build(&p, dir, &lock, short, &setup(), &mut quiet()).unwrap();

    // A delivered file that was changed isn't what was built.
    let srt = dir.join("out/short.en.srt");
    let captions = fs::read(&srt).unwrap();
    fs::write(&srt, b"1\n00:00:00,000 --> 00:00:01,000\nSomething else.\n").unwrap();
    assert_eq!(
        refused(approve(&p, &lock)),
        [
            "out/short.en.srt isn't the file cut short's final build delivered; `vidni final --cut short` builds it again"
        ]
    );
    fs::write(&srt, captions).unwrap();

    // Approved, it is a release, written down once.
    let Approval::Approved(first) = approve(&p, &lock).unwrap() else {
        panic!("the first approval makes a release");
    };
    assert_eq!((first.tag.as_str(), &first.build), ("short-r1", &record));
    assert_eq!(
        release::read(dir).unwrap().cuts["short"],
        std::slice::from_ref(&first)
    );
    assert_eq!(
        approve(&p, &lock).unwrap(),
        Approval::Already(first.clone())
    );
    assert_eq!(release::read(dir).unwrap().cuts["short"].len(), 1);

    // The draft voice's settings are nothing to a final cut: a respelling
    // leaves its build, and so its release, as they are.
    let voice = |setting: &str| {
        let text = PROJECT.replace("[timing]", &format!("[voice]\n{setting}\n\n[timing]"));
        project(dir, &text)
    };
    let respelled = voice("respell = [{ text = \"prose\", say = \"proze\" }]");
    assert_eq!(
        approve(&respelled, &lock).unwrap(),
        Approval::Already(first.clone())
    );
    // Nor is what only the listening check reads.
    let heard = voice("heard_as = [{ heard = \"pros\", means = \"prose\" }]");
    assert_eq!(
        approve(&heard, &lock).unwrap(),
        Approval::Already(first.clone())
    );
    // A reading that changes how a word is written, and not the word,
    // keeps the take and changes the timeline, which is an input.
    let read = voice("read_as = [{ text = \"prose\", say = \"PROSE\" }]");
    assert_eq!(
        refused(approve(&read, &lock)),
        [
            "cut short has changed since its final build (its timeline); `vidni final --cut short` builds it again"
        ]
    );

    // A retake after the build: the build is of the old take.
    let take = takes::status(&p, dir).remove(1).path;
    fs::copy(clip(dir, "louder.wav", 0.2, false), &take).unwrap();
    assert_eq!(
        refused(approve(&p, &lock)),
        [
            "cut short has changed since its final build (its takes); `vidni final --cut short` builds it again"
        ]
    );
    fs::copy(tone(dir), &take).unwrap();
    // A gap changed: the cut is still ratified, and the build is out of date.
    let slower = project(dir, &PROJECT.replace("tail = 0.2", "tail = 0.4"));
    assert_eq!(
        refused(approve(&slower, &lock)),
        [
            "cut short has changed since its final build (its gaps, look or settings); `vidni final --cut short` builds it again"
        ]
    );
    // Other fonts: the same sources, other tools.
    let other = Setup {
        ffmpeg: "0.0.0".into(),
        ..setup()
    };
    assert_eq!(
        refused(build::approve(&p, dir, &lock, short, &other, "2026-11-14")),
        [
            "cut short has changed since its final build (vidni, ffmpeg or the fonts); `vidni final --cut short` builds it again"
        ]
    );
    // An unratified cut can't be approved, whatever was built.
    assert_eq!(
        refused(approve(&p, &Lock::default())),
        ["cut short isn't ratified; `vidni ratify --cut short` freezes it"]
    );
    assert_eq!(release::read(dir).unwrap().cuts["short"], [first]);
}

/// A project whose every scene is a card drawn from a template: it has
/// nothing to capture.
const CARDS: &str = r#"schema = 1

[meta]
title = "Cards"
default_scene = "open"

[timing]
lead_in = 0.1
line_gap = 0.2
section_gap = 0.2
tail = 0.2

[[cuts]]
id = "long"
aspect = "16:9"

[[cuts]]
id = "short"
aspect = "9:16"

[sections.main]

[[lines]]
id = "open-1"
section = "main"
show = "A title card opens the video."

[[lines]]
id = "points-1"
section = "main"
show = "Then a list makes two points."
scene = "points"

[scenes.open]
kind = "card"
template = "title"
title = "Cards from templates"
subtitle = "drawn, not captured"

[scenes.points]
kind = "card"
template = "bullets"
title = "Two points"
items = ["A card needs no browser", "A theme changes its look"]
"#;

#[test]
fn a_cut_of_cards_is_drawn_without_a_capture_and_changes_with_its_words() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    let p = project(dir, CARDS);
    let cuts: Vec<&Cut> = p.cuts.iter().collect();
    assert!(
        capture::plan(&p, dir, &cuts).is_empty(),
        "nothing to capture"
    );
    record_takes(dir, &p, None);
    let lock = ratified(dir, &p);
    let built = |p: &Project, id: &str| {
        build::final_build(p, dir, &lock, cut(p, id), &setup(), &mut quiet()).unwrap()
    };
    let (long, short) = (built(&p, "long"), built(&p, "short"));
    assert!(long.problems().is_empty() && short.problems().is_empty());
    assert!(!dir.join("build/captures").exists());
    // The two shapes are two pictures, and each repeats.
    assert_ne!(
        long.outputs["long.mp4"].frames,
        short.outputs["short.mp4"].frames
    );
    let afresh = Setup {
        again: true,
        ..setup()
    };
    let again = build::final_build(&p, dir, &lock, cut(&p, "long"), &afresh, &mut quiet()).unwrap();
    assert_eq!(again, long);
    // A card's words are a part of what its cut shows: changed, the cut is
    // no longer the ratified one.
    let reworded = project(dir, &CARDS.replace("Two points", "Two things"));
    let refusals = refused(build::final_build(
        &reworded,
        dir,
        &lock,
        cut(&reworded, "long"),
        &setup(),
        &mut quiet(),
    ));
    assert_eq!(
        refusals,
        ["cut long has changed since it was ratified: scene points changed"]
    );
    // Another theme is another look: other inputs, and other frames.
    let quine = vidni_core::theme::load(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples"),
        "themes/quine",
    )
    .unwrap();
    let themed = Setup {
        theme: quine,
        ..setup()
    };
    let other = build::final_build(&p, dir, &lock, cut(&p, "long"), &themed, &mut quiet()).unwrap();
    assert_ne!(other.inputs, long.inputs);
    assert_ne!(
        other.outputs["long.mp4"].frames,
        long.outputs["long.mp4"].frames
    );
}

#[test]
fn a_picture_is_captured_without_a_browser_and_framed_by_its_parts() {
    use vidni_media::capture::{Limits, capture_with};
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="640" height="360">
  <rect width="640" height="360" fill="#101014"/>
  <rect x="400" y="100" width="160" height="120" fill="#f2542d"/>
  <text x="40" y="200" font-family="IBM Plex Sans" font-weight="700" font-size="40" fill="#fff">Words</text>
</svg>"##;
    fs::write(dir.join("diagram.svg"), svg).unwrap();
    let text = r#"schema = 1

[meta]
title = "Pictures"
default_scene = "diagram"

[timing]
lead_in = 0.1
line_gap = 0.2
section_gap = 0.2
tail = 0.2

[[cuts]]
id = "long"
aspect = "16:9"

[sections.main]

[[lines]]
id = "main-1"
section = "main"
show = "The whole picture."

[[lines]]
id = "main-2"
section = "main"
show = "And a part of it."
camera = "box"

[scenes.diagram]
kind = "image"
url = "diagram.svg"
scale = 2

[scenes.diagram.targets]
box = { rect = [400, 100, 160, 120] }

[scenes.diagram.cameras]
box = { union = ["box"] }
"#;
    let p = project(dir, text);
    let cuts: Vec<&Cut> = p.cuts.iter().collect();
    // Both lines show one picture: one capture, taken with no browser.
    let plan = capture::plan(&p, dir, &cuts);
    assert_eq!(plan.len(), 1);
    assert_eq!(plan[0].file.as_deref(), Some("diagram.svg"));
    let taken = capture_with(&p, dir, &plan, Limits::default()).unwrap();
    assert_eq!(taken, plan);
    assert!(capture::plan(&p, dir, &cuts).is_empty());
    let key = &plan[0].key;
    let picture = Image::load_png(&capture::path(dir, key)).unwrap();
    assert_eq!(
        (picture.width, picture.height),
        (1280, 720),
        "drawn at twice its size"
    );
    let frames: Frames =
        serde_json::from_str(&fs::read_to_string(capture::frames_path(dir, key)).unwrap()).unwrap();
    assert_eq!((frames.viewport, frames.scale), ([640, 360], 2));
    assert_eq!(frames.targets["box"], [[400.0, 100.0, 160.0, 120.0]]);
    // Taken again, it is the same picture, byte for byte.
    let first = fs::read(capture::path(dir, key)).unwrap();
    capture_with(&p, dir, &plan, Limits::default()).unwrap();
    assert_eq!(fs::read(capture::path(dir, key)).unwrap(), first);
    // The cut builds from it, and builds the same again.
    record_takes(dir, &p, None);
    let lock = ratified(dir, &p);
    let afresh = Setup {
        again: true,
        ..setup()
    };
    let long = cut(&p, "long");
    let built = build::final_build(&p, dir, &lock, long, &afresh, &mut quiet()).unwrap();
    let again = build::final_build(&p, dir, &lock, long, &afresh, &mut quiet()).unwrap();
    assert_eq!(again, built);
    // The picture's file changes: its capture is out of date, and the cut
    // that froze it is no longer the ratified one.
    fs::write(dir.join("diagram.svg"), svg.replace("#f2542d", "#2f5fd0")).unwrap();
    assert_eq!(capture::plan(&p, dir, &cuts).len(), 1);
    assert_eq!(
        ratify::status_of(&p, dir, &lock, long),
        ratify::Status::Changed(vec![
            "diagram.svg has changed since 1 of its captures was taken".into()
        ])
    );
    capture_with(&p, dir, &plan, Limits::default()).unwrap();
    assert_ne!(fs::read(capture::path(dir, key)).unwrap(), first);
}

#[test]
fn a_cut_goes_from_one_picture_to_another_of_another_size() {
    use vidni_media::capture::{Limits, capture_with};
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    let svg = |w: u32, h: u32, ink: &str| {
        format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}">
  <rect width="{w}" height="{h}" fill="#101014"/>
  <rect x="{}" y="{}" width="60" height="40" fill="{ink}"/>
</svg>"##,
            w - 100,
            h - 60
        )
    };
    // A tall picture, then a wide strip much smaller than it: a framing of
    // the first's lower part is past the second's edge.
    fs::write(dir.join("tall.svg"), svg(1200, 1000, "#f2542d")).unwrap();
    fs::write(dir.join("strip.svg"), svg(780, 130, "#2f5fd0")).unwrap();
    let text = r#"schema = 1

[meta]
title = "Two pictures"
default_scene = "tall"

[timing]
lead_in = 0.1
line_gap = 0.2
section_gap = 0.2
tail = 0.2

[[cuts]]
id = "long"
aspect = "16:9"

[[cuts]]
id = "vertical"
aspect = "9:16"
platforms = []

[sections.main]

[[lines]]
id = "main-1"
section = "main"
show = "The lower part of a tall picture."
camera = "low"

[[lines]]
id = "main-2"
section = "main"
show = "Then a strip."
scene = "strip"

[[lines]]
id = "main-3"
section = "main"
show = "And the tall one again."
camera = "low"

[scenes.tall]
kind = "image"
url = "tall.svg"

[scenes.tall.targets]
low = { rect = [1100, 940, 60, 40] }

[scenes.tall.cameras]
low = { union = ["low"] }

[scenes.strip]
kind = "image"
url = "strip.svg"
"#;
    let p = project(dir, text);
    let cuts: Vec<&Cut> = p.cuts.iter().collect();
    let plan = capture::plan(&p, dir, &cuts);
    capture_with(&p, dir, &plan, Limits::default()).unwrap();
    record_takes(dir, &p, None);
    let lock = ratified(dir, &p);
    for id in ["long", "vertical"] {
        let built = build::final_build(&p, dir, &lock, cut(&p, id), &setup(), &mut quiet());
        assert!(built.is_ok(), "{id}: {:?}", built.err());
    }
}

/// A 16:9 cut for YouTube: `sections` sections of card lines, the last
/// with `last` lines and the others with four. A line's take is 2.5 s
/// and the gaps 0.2 s, so four lines last some 11 s.
fn chaptered(sections: usize, last: usize) -> String {
    let mut text = String::from(
        "schema = 1\n\n[meta]\ntitle = \"Chapters\"\ndefault_scene = \"card\"\n\n\
         [timing]\nlead_in = 0.1\nline_gap = 0.2\nsection_gap = 0.2\ntail = 0.2\n\n\
         [[cuts]]\nid = \"long\"\naspect = \"16:9\"\n\n",
    );
    for s in 0..sections {
        let label = ["first things", "the middle", "last words", "more"][s];
        text.push_str(&format!("[sections.s{s}]\nlabel = \"{label}\"\n\n"));
    }
    for s in 0..sections {
        let lines = if s + 1 == sections { last } else { 4 };
        for n in 0..lines {
            text.push_str(&format!(
                "[[lines]]\nid = \"s{s}-{n}\"\nsection = \"s{s}\"\nshow = \"Line {n} of section {s}.\"\n\n"
            ));
        }
    }
    text.push_str("[scenes.card]\nkind = \"page\"\nurl = \"card.html\"\n");
    text
}

#[test]
fn a_long_cut_has_a_clean_master_a_captioned_variant_and_its_chapters() {
    let (dir, p) = drafted(&chaptered(3, 4));
    let dir = dir.path();
    record_takes(dir, &p, None);
    let lock = ratified(dir, &p);
    let long = cut(&p, "long");
    let record = build::final_build(&p, dir, &lock, long, &setup(), &mut quiet()).unwrap();
    assert_eq!(record.problems(), Vec::<String>::new());
    // The master is clean, and the variant has the captions drawn in (D7).
    let captions: Vec<(&str, bool)> = record
        .outputs
        .iter()
        .map(|(name, o)| (name.as_str(), o.captions))
        .collect();
    assert_eq!(
        captions,
        [("long.captioned.mp4", true), ("long.mp4", false)]
    );
    // Each section is a chapter, from the second its picture comes in.
    let list = fs::read_to_string(dir.join("out/long.chapters.txt")).unwrap();
    assert_eq!(
        list,
        "0:00 First things\n0:10 The middle\n0:21 Last words\n"
    );
    assert_eq!(
        record.t1.chapters.as_deref(),
        Some(vidni_core::release::file_hash(&dir.join("out/long.chapters.txt")).unwrap())
            .as_deref()
    );
    let mut delivered: Vec<String> = fs::read_dir(dir.join("out"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    delivered.sort();
    assert_eq!(
        delivered,
        [
            "long.captioned.mp4",
            "long.chapters.txt",
            "long.en.srt",
            "long.en.vtt",
            "long.mp4"
        ]
    );
    // Both pictures hold the chapters.
    for name in ["long.mp4", "long.captioned.mp4"] {
        let probe = std::process::Command::new("ffprobe")
            .args(["-v", "error", "-show_entries", "chapter_tags=title"])
            .args(["-of", "csv=p=0"])
            .arg(dir.join("out").join(name))
            .output()
            .unwrap();
        assert_eq!(
            String::from_utf8_lossy(&probe.stdout),
            "First things\nThe middle\nLast words\n",
            "{name}"
        );
    }
    // The build is kept as any other, and approved with its list.
    let mut steps = Vec::new();
    let mut watch = |step: Step| {
        if step == Step::Kept {
            steps.push("kept");
        }
    };
    build::final_build(&p, dir, &lock, long, &setup(), &mut watch).unwrap();
    assert_eq!(steps, ["kept"]);
    let release = match build::approve(&p, dir, &lock, long, &setup(), "2026-11-14").unwrap() {
        vidni_core::release::Approval::Approved(release) => release,
        other => panic!("{other:?}"),
    };
    let files: Vec<String> = vidni_core::assets::delivered(&p, &release)
        .into_iter()
        .map(|a| a.path)
        .collect();
    assert!(
        files.contains(&"long.chapters.txt".to_string()),
        "{files:?}"
    );
    // A list that is gone from out is missed.
    fs::remove_file(dir.join("out/long.chapters.txt")).unwrap();
    let reasons = refused(build::approve(&p, dir, &lock, long, &setup(), "2026-11-14"));
    assert!(
        reasons[0].starts_with("out/long.chapters.txt isn't the file"),
        "{reasons:?}"
    );
}

#[test]
fn chapters_that_youtube_would_ignore_fail_qa_and_two_sections_make_no_list() {
    // The last of three sections has one line: a chapter of some 3 s.
    let (dir, p) = drafted(&chaptered(3, 1));
    let dir = dir.path();
    record_takes(dir, &p, None);
    let lock = ratified(dir, &p);
    let reasons = refused(build::final_build(
        &p,
        dir,
        &lock,
        cut(&p, "long"),
        &setup(),
        &mut quiet(),
    ));
    assert_eq!(reasons.len(), 3, "{reasons:?}");
    for (reason, file) in reasons[1..].iter().zip(["long.captioned.mp4", "long.mp4"]) {
        assert!(
            reason.starts_with(&format!("{file}: chapter \"Last words\" lasts 3."))
                && reason.ends_with("only when each lasts 10 s or more"),
            "{reason}"
        );
    }
    assert!(!dir.join("out").exists());

    // Two sections are no list: no file, no chapters in the picture, no problem.
    let (dir, p) = drafted(&chaptered(2, 4));
    let dir = dir.path();
    record_takes(dir, &p, None);
    let lock = ratified(dir, &p);
    let record =
        build::final_build(&p, dir, &lock, cut(&p, "long"), &setup(), &mut quiet()).unwrap();
    assert_eq!(record.problems(), Vec::<String>::new());
    assert_eq!(record.t1.chapters, None);
    assert!(!dir.join("out/long.chapters.txt").exists());
    assert!(!dir.join("build/final/long.chapters.txt").exists());
}

/// A track for a project's library: a second of a 330 Hz tone, with its
/// SHA-256. A draft's clips are at 22.05 kHz, so it is resampled.
fn music(dir: &Path) -> String {
    let path = dir.join("calm.wav");
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(&path, spec).unwrap();
    for n in 0..44100u32 {
        let t = f64::from(n) / 44100.0;
        let s = ((t * 330.0 * std::f64::consts::TAU).sin() * 9000.0) as i16;
        writer.write_sample(s).unwrap();
        writer.write_sample(s).unwrap();
    }
    writer.finalize().unwrap();
    capture::file_sha256(dir, "calm.wav").unwrap()
}

/// `text` with a track playing, licensed for every platform the cuts are for.
fn with_music(text: &str, sha256: &str, content_id: bool) -> String {
    format!(
        "{text}\n[music]\ntrack = \"calm\"\n\n[[music.library]]\nid = \"calm\"\nfile = \"calm.wav\"\n\
         sha256 = \"{sha256}\"\nlicence = \"CC-BY-4.0\"\nattribution = \"Calm, by A. Composer\"\n\
         platforms = [\"youtube\", \"youtube-shorts\", \"instagram-reels\", \"whatsapp-status\"]\n\
         content_id = {content_id}\n"
    )
}

#[test]
fn a_track_from_the_library_plays_under_the_narration_and_is_one_of_its_sounds() {
    let (dir, plain) = drafted(PROJECT);
    let dir = dir.path();
    let sha = music(dir);
    let p = project(dir, &with_music(PROJECT, &sha, false));
    let pack = speak(dir, &p);
    let long = cut(&p, "long");
    let bare = build::draft(
        &plain,
        dir,
        cut(&plain, "long"),
        &pack,
        &setup(),
        &mut quiet(),
    )
    .unwrap();
    let narration = |dir: &Path| fs::read(dir.join("build/draft/long.narration.wav")).unwrap();
    let without = narration(dir);
    let with = build::draft(&p, dir, long, &pack, &setup(), &mut quiet()).unwrap();
    assert_eq!(with.problems(), Vec::<String>::new());
    // Another narration, from another set of sounds; the pictures are the same.
    assert_ne!(narration(dir), without);
    assert_ne!(with.sources.audio, bare.sources.audio);
    assert_ne!(with.t1.pcm, bare.t1.pcm);
    assert_eq!(
        with.outputs["long.mp4"].frames,
        bare.outputs["long.mp4"].frames
    );
    // The track's bytes are an input: another file under the same name
    // is another build.
    let before = with.inputs.clone();
    fs::write(
        dir.join("calm.wav"),
        fs::read(dir.join("tone.wav")).unwrap(),
    )
    .unwrap();
    let other = project(
        dir,
        &with_music(
            PROJECT,
            &capture::file_sha256(dir, "calm.wav").unwrap(),
            false,
        ),
    );
    let again = build::draft(
        &other,
        dir,
        cut(&other, "long"),
        &pack,
        &setup(),
        &mut quiet(),
    )
    .unwrap();
    assert_ne!(again.inputs, before);
}

#[test]
fn a_short_over_a_minute_with_a_content_id_track_is_refused() {
    // 26 lines of 2.5 s each, in a 9:16 cut for Shorts: 70 s.
    let mut text = String::from(
        "schema = 1\n\n[meta]\ntitle = \"A long short\"\ndefault_scene = \"card\"\n\n\
         [timing]\nlead_in = 0.1\nline_gap = 0.2\nsection_gap = 0.2\ntail = 0.2\n\n\
         [[cuts]]\nid = \"short\"\naspect = \"9:16\"\n\n[sections.main]\n\n",
    );
    for n in 0..26 {
        text.push_str(&format!(
            "[[lines]]\nid = \"main-{n}\"\nsection = \"main\"\nshow = \"Line {n}.\"\n\n"
        ));
    }
    text.push_str("[scenes.card]\nkind = \"page\"\nurl = \"card.html\"\n");
    let (dir, _) = drafted(&text);
    let dir = dir.path();
    let sha = music(dir);
    let claimed = project(dir, &with_music(&text, &sha, true));
    let pack = speak(dir, &claimed);
    let reasons = refused(build::draft(
        &claimed,
        dir,
        cut(&claimed, "short"),
        &pack,
        &setup(),
        &mut quiet(),
    ));
    assert_eq!(
        reasons,
        [
            "cut short lasts 70 s and track calm is registered with Content ID: YouTube blocks \
          a Short of over a minute with a Content ID claim, worldwide"
        ]
    );
    // Nothing was rendered for it.
    assert!(!dir.join("build/draft/short.mp4").exists());
    // The same track with no claim on it is planned as any other.
    let free = project(dir, &with_music(&text, &sha, false));
    assert!(
        build::plan(
            &free,
            dir,
            cut(&free, "short"),
            Voice::Draft,
            Some(&pack),
            &setup()
        )
        .is_ok()
    );
}

/// The names in a folder, sorted.
fn names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .map(|entries| {
            entries
                .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

#[test]
fn each_platform_gets_a_folder_with_the_video_its_metadata_and_its_images() {
    // A long cut for YouTube with chapters, and a short for three apps.
    let publish = "\n[publish]\ndescription = \"What this is about.\"\ntags = [\"one\", \"two words\"]\n\
                   links = [{ label = \"Home\", url = \"https://example.com\" }]\n\
                   thumbnails = [{ line = \"s0-1\", title = \"A thumbnail\" }, { line = \"s2-0\" }]\n\n\
                   [publish.titles]\nyoutube-shorts = \"The short of it\"\n";
    let text = chaptered(3, 4).replace(
        "[[cuts]]\nid = \"long\"\naspect = \"16:9\"\n",
        "[[cuts]]\nid = \"long\"\naspect = \"16:9\"\nend_hold = 6\n\n\
         [[cuts]]\nid = \"short\"\naspect = \"9:16\"\nlines = [\"s0-0\", \"s0-1\"]\n",
    ) + publish;
    let (dir, p) = drafted(&text);
    let dir = dir.path();
    record_takes(dir, &p, None);
    let lock = ratified(dir, &p);
    let build_and_export = |p: &Project, id: &str| {
        let record = build::final_build(p, dir, &lock, cut(p, id), &setup(), &mut quiet()).unwrap();
        (
            record,
            build::exports(p, dir, cut(p, id), &setup()).unwrap(),
        )
    };
    let (long, warnings) = build_and_export(&p, "long");
    assert_eq!((&warnings.refused, &warnings.warnings), (&vec![], &vec![]));
    // The long cut holds its last picture 6 s, where its tail was 0.2 s.
    let lasts = long.outputs["long.mp4"].frame_count as f64 / 30.0;
    assert!((lasts - 37.5).abs() < 1.0, "{lasts} s");

    let youtube = dir.join("out/youtube/long");
    assert_eq!(
        names(&youtube),
        [
            "long.en.srt",
            "long.en.vtt",
            "long.mp4",
            "metadata.md",
            "thumbnail-1.png",
            "thumbnail-2.png"
        ]
    );
    // The video is the clean master, the very file that was delivered.
    assert_eq!(
        fs::read(youtube.join("long.mp4")).unwrap(),
        fs::read(dir.join("out/long.mp4")).unwrap()
    );
    assert!(!long.outputs["long.mp4"].captions);
    let metadata = fs::read_to_string(youtube.join("metadata.md")).unwrap();
    for part in [
        "# long on YouTube\n",
        "## Title\n\nChapters\n",
        "## Description\n\nWhat this is about.\n\n0:00 First things\n0:10 The middle\n0:21 Last words\n\nHome: https://example.com\n",
        "## Tags\n\none, two words\n",
        "choose Test & compare and upload the 2 thumbnails",
        "Add an end screen over the last 6 seconds",
    ] {
        assert!(metadata.contains(part), "{part:?} isn't in:\n{metadata}");
    }
    // Thumbnails are 1280 by 720, under 2 MB, and the one with words
    // differs from the frame alone.
    let thumbnail = |n: u32| Image::load_png(&youtube.join(format!("thumbnail-{n}.png"))).unwrap();
    let (first, second) = (thumbnail(1), thumbnail(2));
    assert_eq!((first.width, first.height), (1280, 720));
    assert_eq!((second.width, second.height), (1280, 720));
    assert!(fs::metadata(youtube.join("thumbnail-1.png")).unwrap().len() < 2_000_000);
    // The cards are one colour, so the second is plain and the first has its words.
    assert!(second.to_rgb().ink(&[29, 53, 87]).is_none());
    let words = first.to_rgb().ink(&[29, 53, 87]).unwrap();
    assert!(
        words[0] >= 48 && words[3] <= 720 - 48 && words[1] > 360,
        "{words:?}"
    );

    // The short: a folder for each of its three apps.
    let (_, warnings) = build_and_export(&p, "short");
    assert_eq!((&warnings.refused, &warnings.warnings), (&vec![], &vec![]));
    assert_eq!(
        names(&dir.join("out/youtube-shorts/short")),
        ["metadata.md", "short.mp4"]
    );
    assert_eq!(
        names(&dir.join("out/whatsapp-status/short")),
        ["metadata.md", "short.mp4"]
    );
    let reels = dir.join("out/instagram-reels/short");
    assert_eq!(names(&reels), ["cover.png", "metadata.md", "short.mp4"]);
    let shorts = fs::read_to_string(dir.join("out/youtube-shorts/short/metadata.md")).unwrap();
    assert!(shorts.contains("## Title\n\nThe short of it\n"), "{shorts}");
    assert!(
        shorts.contains("set this Short's related video to the `long` cut"),
        "{shorts}"
    );
    // The cover's words are inside the 3:4 middle of the frame.
    let cover = Image::load_png(&reels.join("cover.png")).unwrap().to_rgb();
    assert_eq!((cover.width, cover.height), (1080, 1920));
    let background = cover.pixel(0, 0).to_vec();
    let ink = cover.ink(&background).unwrap();
    assert!(ink[1] >= 240 && ink[3] <= 1680, "{ink:?}");

    // Other words, and one app fewer: the folders are made again, the
    // dropped app's goes, and nothing is rendered.
    let built = fs::metadata(dir.join("build/final/short.mp4"))
        .unwrap()
        .modified()
        .unwrap();
    let edited = text
        .replace("What this is about.", "What this is really about.")
        .replace(
            "lines = [\"s0-0\", \"s0-1\"]\n",
            "lines = [\"s0-0\", \"s0-1\"]\nplatforms = [\"youtube-shorts\", \"instagram-reels\"]\n",
        );
    let p = project(dir, &edited);
    build::exports(&p, dir, cut(&p, "long"), &setup()).unwrap();
    let metadata = fs::read_to_string(youtube.join("metadata.md")).unwrap();
    assert!(
        metadata.contains("What this is really about."),
        "{metadata}"
    );
    // The short's platforms are part of its look, so it is built again
    // before it is exported; the long cut's files were not touched.
    let before = fs::metadata(dir.join("build/final/long.mp4"))
        .unwrap()
        .modified()
        .unwrap();
    let lock = ratified(dir, &p);
    build::final_build(&p, dir, &lock, cut(&p, "short"), &setup(), &mut quiet()).unwrap();
    build::exports(&p, dir, cut(&p, "short"), &setup()).unwrap();
    assert!(!dir.join("out/whatsapp-status").exists());
    assert_eq!(names(&dir.join("out/instagram-reels/short")).len(), 3);
    assert_eq!(
        fs::metadata(dir.join("build/final/long.mp4"))
            .unwrap()
            .modified()
            .unwrap(),
        before
    );
    let _ = built;
}

#[test]
fn a_cut_a_platform_wouldnt_take_isnt_exported_for_it() {
    // 36 lines of 2.5 s each: 97 s, which Shorts takes and a Status doesn't.
    let mut text = String::from(
        "schema = 1\n\n[meta]\ntitle = \"A long short\"\ndefault_scene = \"card\"\n\n\
         [timing]\nlead_in = 0.1\nline_gap = 0.2\nsection_gap = 0.2\ntail = 0.2\n\n\
         [[cuts]]\nid = \"short\"\naspect = \"9:16\"\nplatforms = [\"youtube-shorts\", \"whatsapp-status\"]\n\n\
         [sections.main]\n\n",
    );
    for n in 0..36 {
        text.push_str(&format!(
            "[[lines]]\nid = \"main-{n}\"\nsection = \"main\"\nshow = \"Line {n}.\"\n\n"
        ));
    }
    text.push_str("[scenes.card]\nkind = \"page\"\nurl = \"card.html\"\n");
    let (dir, p) = drafted(&text);
    let dir = dir.path();
    record_takes(dir, &p, None);
    let lock = ratified(dir, &p);
    let short = cut(&p, "short");
    let record = build::final_build(&p, dir, &lock, short, &setup(), &mut quiet()).unwrap();
    assert_eq!(record.problems(), Vec::<String>::new());
    let exports = build::exports(&p, dir, short, &setup()).unwrap();
    assert_eq!(
        exports.refused,
        [
            "cut short isn't exported for whatsapp-status: it lasts 1:38, and WhatsApp Status takes 1:30"
        ]
    );
    assert_eq!(exports.warnings, Vec::<String>::new());
    // It is delivered, and exported where it is taken.
    assert!(dir.join("out/short.mp4").is_file());
    assert_eq!(
        names(&dir.join("out/youtube-shorts/short")),
        ["metadata.md", "short.mp4"]
    );
    assert!(!dir.join("out/whatsapp-status").exists());
}
