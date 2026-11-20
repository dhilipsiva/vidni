//! Build gates and rendered boundary frames, using known captures and tone audio.
use std::{
    fs,
    path::{Path, PathBuf},
};
use vidni_core::{
    capture, cues, project::Project, release::Voice, spoken, takes, theme::Theme, validate,
};
use vidni_media::{
    build::{self, Faces, Setup, Stop},
    capture::Frames,
    compose::image::Image,
};

fn fixture() -> (tempfile::TempDir, Project, Setup, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let project = validate::check(
        r##"schema=1
[meta]
title="Cue gates"
[emphasis]
min_duration=0
[sections.main]
[[lines]]
id="parcel"
section="main"
show="The capital P names the parcel."
scene="screen"
camera_vertical="word"
emphasis=[{id="capital",target="capital",phrase="capital P"}]
[[cuts]]
id="long"
aspect="16:9"
[[cuts]]
id="short"
aspect="9:16"
[scenes.screen]
kind="web"
url="https://example.com"
[scenes.screen.targets.capital]
css="#parcel"
span={text="P"}
[scenes.screen.cameras.word]
union=["capital"]
"##,
        None,
    )
    .unwrap();
    let font =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fonts/IBMPlexSans-Bold.otf");
    let setup = Setup {
        faces: Faces::all(&font),
        ffmpeg: "fixture".into(),
        again: false,
        theme: Theme::default_theme(),
    };
    let audio = takes::kept(
        dir.path(),
        "parcel",
        &spoken::spoken_hash(&project.lines[0], &project.voice),
    );
    fs::create_dir_all(audio.parent().unwrap()).unwrap();
    tone(&audio, 220.0);
    let c = capture::for_cut(&project, &project.cuts[0]);
    fs::create_dir_all(dir.path().join(capture::DIR)).unwrap();
    Image::new(1280, 720, &[30, 40, 50])
        .save_png(&capture::path(dir.path(), &c[0].key))
        .unwrap();
    let frames = Frames {
        viewport: [1280, 720],
        scale: 1,
        targets: [("capital".into(), vec![[540.0, 250.0, 30.0, 55.0]])].into(),
        highlights: Default::default(),
        browser: "fixture browser".into(),
        rows: None,
    };
    fs::write(
        capture::frames_path(dir.path(), &c[0].key),
        serde_json::to_vec(&frames).unwrap(),
    )
    .unwrap();
    (dir, project, setup, audio)
}

fn tone(path: &Path, hz: f64) {
    let mut out = hound::WavWriter::create(
        path,
        hound::WavSpec {
            channels: 1,
            sample_rate: 48000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        },
    )
    .unwrap();
    for n in 0..48000 {
        out.write_sample(
            (8000.0 * (f64::from(n) * hz * std::f64::consts::TAU / 48000.0).sin()) as i16,
        )
        .unwrap();
    }
    out.finalize().unwrap();
}

#[test]
fn proposals_never_pass_the_build_gate_and_review_plans_cannot_export() {
    let (dir, p, setup, _) = fixture();
    assert!(matches!(
        build::plan(&p, dir.path(), &p.cuts[0], Voice::Takes, None, &setup),
        Err(Stop::Refused(_))
    ));
    let review =
        build::plan_with_review(&p, dir.path(), &p.cuts[0], Voice::Takes, None, &setup, true)
            .unwrap();
    let cue = &review.emphasis[0];
    let mut proposed = cues::Records::default();
    proposed.entries.insert(
        cue.key.clone(),
        cues::Record {
            binding: cue.binding.clone(),
            interval: cues::Interval {
                start: 9600,
                end: 33600,
            },
        },
    );
    proposed.write(&dir.path().join(cues::PROPOSALS)).unwrap();
    assert!(matches!(
        build::plan(&p, dir.path(), &p.cuts[0], Voice::Takes, None, &setup),
        Err(Stop::Refused(_))
    ));
    assert!(
        build::build(&review, dir.path(), &mut |_| {})
            .unwrap_err()
            .contains("cannot be exported")
    );
}

#[test]
fn verified_cues_render_only_their_interval_in_both_aspects_and_retake_invalidates() {
    let (dir, p, setup, audio) = fixture();
    let mut locked = cues::Records::default();
    for cut in &p.cuts {
        let review =
            build::plan_with_review(&p, dir.path(), cut, Voice::Takes, None, &setup, true).unwrap();
        let cue = &review.emphasis[0];
        locked.entries.insert(
            cue.key.clone(),
            cues::Record {
                binding: cue.binding.clone(),
                interval: cues::Interval {
                    start: 9600,
                    end: 33600,
                },
            },
        );
    }
    locked.write(&dir.path().join(cues::LOCK)).unwrap();
    for cut in &p.cuts {
        let plan = build::plan(&p, dir.path(), cut, Voice::Takes, None, &setup).unwrap();
        let event = plan.emphasis[0].events()[0].clone();
        let mut with = plan.composer(true).unwrap();
        let mut base = build::plan(&p, dir.path(), cut, Voice::Takes, None, &setup).unwrap();
        base.emphasis.clear();
        let mut without = base.composer(true).unwrap();
        for f in [event.start - 1, event.end, event.end + 1] {
            assert_eq!(
                with.frame(f).unwrap().data,
                without.frame(f).unwrap().data,
                "{} frame {f}",
                cut.id
            );
        }
        for f in [event.start, event.end - 1] {
            assert_ne!(
                with.frame(f).unwrap().data,
                without.frame(f).unwrap().data,
                "{} frame {f}",
                cut.id
            );
        }
    }
    // Display text can change while `say` and its exact take stay the same.
    // It changes the caption layout and must invalidate visual verification.
    let mut changed = p.clone();
    changed.lines[0].say = Some(p.lines[0].show.clone());
    changed.lines[0].show = "The displayed caption has been changed.".into();
    assert!(matches!(
        build::plan(
            &changed,
            dir.path(),
            &changed.cuts[0],
            Voice::Takes,
            None,
            &setup
        ),
        Err(Stop::Refused(_))
    ));
    tone(&audio, 330.0);
    assert!(matches!(
        build::plan(&p, dir.path(), &p.cuts[0], Voice::Takes, None, &setup),
        Err(Stop::Refused(_))
    ));
}

#[test]
fn moving_a_target_into_the_caption_band_is_refused() {
    let (dir, p, setup, _) = fixture();
    let mut review =
        build::plan_with_review(&p, dir.path(), &p.cuts[0], Voice::Takes, None, &setup, true)
            .unwrap();
    review.emphasis[0].interval = Some(cues::Interval {
        start: 9600,
        end: 33600,
    });
    review.emphasis[0].boxes = vec![[500.0, 680.0, 120.0, 35.0]];
    review.emphasis[0].display = Some(vidni_media::cues::Display {
        start: 24,
        end: 39,
        requested_end: 39,
        cutoff: None,
        segments: vec![vidni_media::compose::cut::EmphasisEvent {
            id: "covered".into(),
            line: 0,
            start: 24,
            end: 39,
            boxes: review.emphasis[0].boxes.clone(),
        }],
    });
    let error = review.composer(true).err().unwrap();
    assert!(error.contains("clipped or covered"), "{error}");
}

fn proposed(dir: &Path, p: &Project, setup: &Setup) {
    let plan =
        build::plan_with_review(p, dir, &p.cuts[0], Voice::Takes, None, setup, true).unwrap();
    let mut records = cues::Records::default();
    for cue in &plan.emphasis {
        records.entries.insert(
            cue.key.clone(),
            cues::Record {
                binding: cue.binding.clone(),
                interval: cues::Interval {
                    start: 9600,
                    end: 33600,
                },
            },
        );
    }
    records.write(&dir.join(cues::PROPOSALS)).unwrap();
}

#[test]
fn readable_default_keeps_a_short_phrase_visible_for_ninety_frames() {
    let (dir, mut p, setup, _) = fixture();
    p.emphasis = Default::default();
    proposed(dir.path(), &p, &setup);
    let plan =
        build::plan_with_review(&p, dir.path(), &p.cuts[0], Voice::Takes, None, &setup, true)
            .unwrap();
    let cue = &plan.emphasis[0];
    let d = cue.display.as_ref().unwrap();
    assert_eq!(d.end - d.start, 90);
    assert!(d.cutoff.is_none());
    assert_eq!(
        cue.interval.unwrap().end,
        33600,
        "do not falsify the spoken timing"
    );
    let mut with = plan.composer(true).unwrap();
    let mut base =
        build::plan_with_review(&p, dir.path(), &p.cuts[0], Voice::Takes, None, &setup, true)
            .unwrap();
    base.emphasis.clear();
    let mut without = base.composer(true).unwrap();
    for f in [d.start, d.start + 45, d.end - 1] {
        assert_ne!(with.frame(f).unwrap().data, without.frame(f).unwrap().data);
    }
    for f in [d.start - 1, d.end] {
        assert_eq!(with.frame(f).unwrap().data, without.frame(f).unwrap().data);
    }
    assert!(!cue.verified);
}

#[test]
fn a_hold_follows_explicit_identity_and_stops_at_an_unrelated_scene() {
    for (identity, changed_picture, camera_exit) in [
        (false, true, false),
        (true, true, false),
        (false, false, false),
        (true, true, true),
    ] {
        let continuing = identity || !changed_picture;
        let (dir, mut p, setup, _) = fixture();
        p.emphasis = Default::default();
        p.scenes
            .get_mut("screen")
            .unwrap()
            .targets
            .get_mut("capital")
            .unwrap()
            .continuity = Some("parcel-capital".into());
        let mut next = p.scenes["screen"].clone();
        next.url = "https://example.com/changed".into();
        if !identity {
            next.targets.get_mut("capital").unwrap().continuity = None;
        }
        if camera_exit {
            next.targets.insert(
                "far".into(),
                vidni_core::project::Target {
                    css: Some("#far".into()),
                    ..Default::default()
                },
            );
            next.cameras.insert(
                "far".into(),
                vidni_core::project::Camera {
                    union: vec!["far".into()],
                    viewport: false,
                    vertical: None,
                },
            );
        }
        p.scenes.insert("next".into(), next);
        let mut line = p.lines[0].clone();
        line.id = "next".into();
        line.scene = Some("next".into());
        line.emphasis.clear();
        if camera_exit {
            line.camera = Some("far".into());
        }
        let audio = takes::kept(dir.path(), &line.id, &spoken::spoken_hash(&line, &p.voice));
        fs::create_dir_all(audio.parent().unwrap()).unwrap();
        tone(&audio, 330.0);
        p.lines.push(line);
        // Recreate captures for the current declarations, with moved geometry.
        for (i, capture) in capture::for_cut(&p, &p.cuts[0]).iter().enumerate() {
            let shift = if changed_picture { i } else { 0 };
            Image::new(1280, 720, &[30 + shift as u8, 40, 50])
                .save_png(&capture::path(dir.path(), &capture.key))
                .unwrap();
            let mut frames = Frames {
                viewport: [1280, 720],
                scale: 1,
                targets: [(
                    "capital".into(),
                    vec![[540.0 + shift as f64 * 30.0, 250.0, 30.0, 55.0]],
                )]
                .into(),
                highlights: Default::default(),
                browser: "fixture".into(),
                rows: None,
            };
            if camera_exit && i == 1 {
                frames
                    .targets
                    .insert("far".into(), vec![[1150.0, 200.0, 30.0, 55.0]]);
            }
            fs::write(
                capture::frames_path(dir.path(), &capture.key),
                serde_json::to_vec(&frames).unwrap(),
            )
            .unwrap();
        }
        proposed(dir.path(), &p, &setup);
        let plan =
            build::plan_with_review(&p, dir.path(), &p.cuts[0], Voice::Takes, None, &setup, true)
                .unwrap();
        let d = plan.emphasis[0].display.as_ref().unwrap();
        if camera_exit {
            assert!(d.end - d.start < 90);
            assert_eq!(d.cutoff.as_deref(), Some("camera leaves the target"));
            assert!(d.end > d.segments[0].end);
        } else if continuing {
            assert_eq!(d.end - d.start, 90);
            assert_eq!(d.segments.len(), 2);
            assert_eq!(
                d.segments[1].boxes[0][0],
                if changed_picture { 570.0 } else { 540.0 }
            );
            assert_eq!(d.segments[0].end, d.segments[1].start);
        } else {
            assert!(d.end - d.start < 90);
            assert_eq!(d.cutoff.as_deref(), Some("scene change removes the target"));
        }
        plan.composer(true).unwrap();
        // A later take contributes to the held context and invalidates timing.
        tone(&audio, 440.0);
        let changed =
            build::plan_with_review(&p, dir.path(), &p.cuts[0], Voice::Takes, None, &setup, true)
                .unwrap();
        assert_ne!(plan.emphasis[0].binding, changed.emphasis[0].binding);
        assert!(changed.emphasis[0].interval.is_none());
    }
}

#[test]
#[ignore = "needs ffmpeg and pinned fonts"]
fn continuous_review_keeps_the_timeline_and_stereo_audio_without_verifying_cues() {
    let (dir, p, setup, _) = fixture();
    let plan =
        build::plan_with_review(&p, dir.path(), &p.cuts[0], Voice::Takes, None, &setup, true)
            .unwrap();
    let mut last = (0, 0);
    let video =
        vidni_media::cues::full_preview(&plan, dir.path(), &mut |n, total| last = (n, total))
            .unwrap();
    assert_eq!(last.0, last.1);
    let record: serde_json::Value =
        serde_json::from_slice(&fs::read(video.with_extension("json")).unwrap()).unwrap();
    assert_eq!(record["kind"], "diagnostic-review");
    assert_eq!(record["duration"].as_f64().unwrap(), plan.timeline().total);
    assert_eq!(record["emphasis"][0]["verified"], false);
    let output = std::process::Command::new("ffprobe")
        .args(["-v", "error", "-show_streams", "-of", "json"])
        .arg(&video)
        .output()
        .unwrap();
    assert!(output.status.success());
    let streams: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let audio = streams["streams"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["codec_type"] == "audio")
        .unwrap();
    assert_eq!(audio["sample_rate"], "48000");
    assert_eq!(audio["channels"], 2);
    assert!(!dir.path().join(cues::LOCK).exists());
    assert!(!dir.path().join("build/draft/long.mp4").exists());
    assert!(matches!(
        build::plan(&p, dir.path(), &p.cuts[0], Voice::Takes, None, &setup),
        Err(Stop::Refused(_))
    ));
    assert_eq!(
        vidni_media::cues::full_preview(&plan, dir.path(), &mut |_, _| panic!(
            "cached review rendered again"
        ))
        .unwrap(),
        video
    );
}

fn proposed_fixture() -> (tempfile::TempDir, Project, Setup, PathBuf) {
    let (dir, p, setup, audio) = fixture();
    let plan =
        build::plan_with_review(&p, dir.path(), &p.cuts[0], Voice::Takes, None, &setup, true)
            .unwrap();
    let cue = &plan.emphasis[0];
    let mut proposals = cues::Records::default();
    proposals.entries.insert(
        cue.key.clone(),
        cues::Record {
            binding: cue.binding.clone(),
            interval: cues::Interval {
                start: 9600,
                end: 33600,
            },
        },
    );
    proposals.write(&dir.path().join(cues::PROPOSALS)).unwrap();
    (dir, p, setup, audio)
}

// Receipt validation does not decode video; the encoded-media test above
// covers that path. Here distinct fixture bytes test exact artifact binding.
fn confirmation_receipt(dir: &Path, p: &Project, plan: &build::Plan<'_>) -> (PathBuf, String) {
    let movie = dir.join("build/cues/full/long-takes.review.mp4");
    fs::create_dir_all(movie.parent().unwrap()).unwrap();
    fs::write(&movie, b"exact reviewed video fixture").unwrap();
    let sha = vidni_core::release::file_hash(&movie).unwrap();
    let inputs = cues::hash(&serde_json::json!([
        "full-review-1",
        vidni_media::cues::REVIEW_SCHEMA,
        plan.inputs,
        plan.emphasis
    ]));
    let receipt = serde_json::json!({
        "schema": 1, "kind": "diagnostic-review", "inputs": inputs,
        "cut": "long", "voice": "takes", "fps": 30,
        "size": p.cuts[0].size(), "duration": plan.timeline().total,
        "sha256": sha, "emphasis": plan.emphasis,
    });
    fs::write(
        movie.with_extension("json"),
        serde_json::to_vec(&receipt).unwrap(),
    )
    .unwrap();
    (movie, sha)
}

#[test]
fn explicit_owner_confirmation_is_audited_and_retake_still_invalidates() {
    let (dir, p, setup, audio) = proposed_fixture();
    let plan =
        build::plan_with_review(&p, dir.path(), &p.cuts[0], Voice::Takes, None, &setup, true)
            .unwrap();
    let (_, sha) = confirmation_receipt(dir.path(), &p, &plan);
    let statement =
        "I watched this video and confirm its highlight timing and targets are correct.";
    let (entries, confirmation) =
        vidni_media::cues::confirmed_full_review(&plan, dir.path(), &sha, statement, true).unwrap();
    assert!(
        !dir.path().join(cues::LOCK).exists(),
        "checking a confirmation does not save it"
    );
    assert_eq!(confirmation.owner_statement, statement);
    assert_eq!(
        confirmation.recorded_by,
        "assistant relaying owner confirmation"
    );
    assert_eq!(confirmation.review_sha256, sha);
    assert_eq!(confirmation.cue_keys, vec![plan.emphasis[0].key.clone()]);
    let mut saved = cues::Records::default();
    saved.entries.extend(entries);
    saved.confirmations.push(confirmation.clone());
    saved.write(&dir.path().join(cues::LOCK)).unwrap();
    let current = build::plan(&p, dir.path(), &p.cuts[0], Voice::Takes, None, &setup).unwrap();
    assert!(current.emphasis.iter().all(|c| c.verified));
    let (_, repeated) =
        vidni_media::cues::confirmed_full_review(&current, dir.path(), &sha, statement, true)
            .unwrap();
    assert_eq!(
        repeated, confirmation,
        "lock bookkeeping is not a content change"
    );
    tone(&audio, 440.0);
    assert!(matches!(
        build::plan(&p, dir.path(), &p.cuts[0], Voice::Takes, None, &setup),
        Err(Stop::Refused(_))
    ));
    let changed =
        build::plan_with_review(&p, dir.path(), &p.cuts[0], Voice::Takes, None, &setup, true)
            .unwrap();
    assert!(
        vidni_media::cues::confirmed_full_review(&changed, dir.path(), &sha, statement, true)
            .is_err()
    );
}

#[test]
fn confirmation_refuses_wrong_changed_incomplete_and_stale_receipts() {
    let (dir, p, setup, _) = proposed_fixture();
    let plan =
        build::plan_with_review(&p, dir.path(), &p.cuts[0], Voice::Takes, None, &setup, true)
            .unwrap();
    let (movie, sha) = confirmation_receipt(dir.path(), &p, &plan);
    let receipt = movie.with_extension("json");
    let original: serde_json::Value = serde_json::from_slice(&fs::read(&receipt).unwrap()).unwrap();
    for case in [
        "cut",
        "voice",
        "hash",
        "inputs",
        "missing cue",
        "duplicate cue",
        "timing",
        "identity",
        "duration",
        "size",
    ] {
        let mut changed = original.clone();
        match case {
            "cut" => changed["cut"] = "short".into(),
            "voice" => changed["voice"] = "draft".into(),
            "hash" => changed["sha256"] = "0".repeat(64).into(),
            "inputs" => changed["inputs"] = "stale".into(),
            "missing cue" => changed["emphasis"] = serde_json::json!([]),
            "duplicate cue" => {
                let cue = changed["emphasis"][0].clone();
                changed["emphasis"].as_array_mut().unwrap().push(cue);
            }
            "timing" => changed["emphasis"][0]["interval"]["start"] = 10000.into(),
            "identity" => changed["review_identity"] = "stale".into(),
            "duration" => changed["duration"] = 999.into(),
            "size" => changed["size"] = serde_json::json!([1080, 1920]),
            _ => unreachable!(),
        }
        fs::write(&receipt, serde_json::to_vec(&changed).unwrap()).unwrap();
        assert!(
            vidni_media::cues::confirmed_full_review(
                &plan,
                dir.path(),
                &sha,
                "Owner confirms the reviewed video.",
                true
            )
            .is_err(),
            "{case}"
        );
        assert!(!dir.path().join(cues::LOCK).exists());
    }
    fs::write(&receipt, serde_json::to_vec(&original).unwrap()).unwrap();
    for (expected, statement) in [
        (sha.as_str(), "  "),
        ("", "Confirmed"),
        ("wrong video", "Confirmed"),
    ] {
        assert!(
            vidni_media::cues::confirmed_full_review(&plan, dir.path(), expected, statement, true)
                .is_err()
        );
    }
    fs::write(&movie, b"a different video").unwrap();
    assert!(
        vidni_media::cues::confirmed_full_review(&plan, dir.path(), &sha, "Confirmed", true)
            .is_err()
    );
}

#[test]
fn confirmation_cannot_accept_a_video_with_an_unset_highlight() {
    let (dir, p, setup, _) = fixture();
    let plan =
        build::plan_with_review(&p, dir.path(), &p.cuts[0], Voice::Takes, None, &setup, true)
            .unwrap();
    let (_, sha) = confirmation_receipt(dir.path(), &p, &plan);
    assert!(
        vidni_media::cues::confirmed_full_review(&plan, dir.path(), &sha, "Confirmed", false)
            .unwrap_err()
            .contains("unset highlight")
    );
    assert!(!dir.path().join(cues::LOCK).exists());
}
