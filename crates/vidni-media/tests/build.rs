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
    Setup::with(Faces::all(&fixture("fonts/IBMPlexSans-Bold.otf"))).unwrap()
}

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
    assert_eq!(stages, ["timeline and captions", "narration", "long.mp4"]);

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
    assert_eq!(stages, ["timeline and captions", "narration", "long.mp4"]);
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
    let stages = ["timeline and captions", "narration", "short.mp4"];
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
