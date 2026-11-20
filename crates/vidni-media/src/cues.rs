//! Emphasis review inputs, advisory alignment and diagnostic previews.
use crate::capture::Frames;
use crate::compose::cut::EmphasisEvent;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use vidni_core::cues::{self, Binding, Interval, Record, Records};
use vidni_core::project::{Cut, Project};
use vidni_core::release::{self, Voice};
use vidni_core::timeline::Timeline;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cue {
    pub key: String,
    pub line: usize,
    pub line_id: String,
    pub phrase: String,
    pub occurrence: Option<u32>,
    pub spoken: String,
    pub audio: PathBuf,
    pub line_start: f64,
    pub binding: Binding,
    pub boxes: Vec<[f64; 4]>,
    pub interval: Option<Interval>,
    /// Display time is separate from the phrase's exact audio samples.
    #[serde(default)]
    pub display: Option<Display>,
    pub verified: bool,
    pub problem: Option<String>,
}

impl Cue {
    pub fn events(&self) -> &[EmphasisEvent] {
        self.display.as_ref().map_or(&[], |d| &d.segments)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Display {
    pub start: u64,
    pub end: u64,
    pub requested_end: u64,
    pub cutoff: Option<String>,
    /// One event per active scene, using that scene's measured coordinates.
    pub segments: Vec<EmphasisEvent>,
}

struct Picture {
    hash: String,
    frames: Frames,
}

fn frame(t: f64) -> u64 {
    (t * 30.0 - 1e-9).ceil() as u64
}

fn display(
    cue: &Cue,
    spans: &[(usize, u64, u64, Vec<[f64; 4]>)],
    minimum: f64,
    total: u64,
) -> Option<Display> {
    let [start, spoken_end] = cue.interval?.frames(cue.line_start, cue.binding.rate, 30);
    let requested_end = spoken_end.max(start.saturating_add((minimum * 30.0).ceil() as u64));
    let mut end = requested_end.min(total);
    let available = spans.last().map_or(start, |s| s.2);
    let cutoff = if available < end {
        end = available;
        Some("scene change removes the target".into())
    } else if end < requested_end {
        Some("end of cut".into())
    } else {
        None
    };
    let segments = spans
        .iter()
        .filter_map(|(line, a, b, boxes)| {
            let a = start.max(*a);
            let b = end.min(*b);
            (a < b).then(|| EmphasisEvent {
                id: cue.key.clone(),
                line: *line,
                start: a,
                end: b,
                boxes: boxes.clone(),
            })
        })
        .collect();
    Some(Display {
        start,
        end,
        requested_end,
        cutoff,
        segments,
    })
}

fn visible_interval(
    binding: &Binding,
    line_start: f64,
    interval: Interval,
) -> Result<Interval, String> {
    interval.check(binding)?;
    let [start, end] = interval.frames(line_start, binding.rate, 30);
    if start >= end {
        return Err("cue interval contains no video frame; set its timing in review".into());
    }
    Ok(interval)
}

/// The spoken interval must be fully visible. Only the extra reading hold may
/// end early when the camera leaves its target; coverage/geometry errors fail.
pub(crate) fn resolve_visibility(
    items: &mut [Cue],
    composer: &crate::compose::cut::Composer,
) -> Result<(), String> {
    use crate::compose::cut::EmphasisError;
    for cue in items {
        let Some(display) = &mut cue.display else {
            continue;
        };
        let spoken_end = cue
            .interval
            .unwrap()
            .frames(cue.line_start, cue.binding.rate, 30)[1];
        let mut stop = None;
        'segments: for event in &display.segments {
            for f in event.start..event.end {
                match composer.emphasis_geometry(event, f) {
                    Ok(_) => {}
                    Err(EmphasisError::OutsideCamera(_)) if f >= spoken_end => {
                        stop = Some(f);
                        break 'segments;
                    }
                    Err(e) => return Err(e.to_string()),
                }
            }
        }
        if let Some(end) = stop {
            display.end = end;
            display.cutoff = Some("camera leaves the target".into());
            display.segments.retain(|e| e.start < end);
            for event in &mut display.segments {
                event.end = event.end.min(end);
            }
        }
    }
    Ok(())
}

pub struct Inputs<'a> {
    pub project: &'a Project,
    pub dir: &'a Path,
    pub cut: &'a Cut,
    pub voice: Voice,
    pub audio: &'a [(String, PathBuf)],
    pub timeline: &'a Timeline,
    pub theme: &'a vidni_core::theme::Theme,
}

pub fn collect(input: Inputs<'_>) -> Result<Vec<Cue>, String> {
    let Inputs {
        project,
        dir,
        cut,
        voice,
        audio,
        timeline,
        theme,
    } = input;
    if cut.lines_in(project).iter().all(|l| l.emphasis.is_empty()) {
        return Ok(Vec::new());
    }
    let verified = Records::read(&dir.join(cues::LOCK))?;
    let proposals = Records::read(&dir.join(cues::PROPOSALS))?;
    let keys = vidni_core::capture::keys_in(project, cut);
    let lines = cut.lines_in(project);
    // Measure every prospective continuation before resolving any cue. A still
    // without authored emphasis can nevertheless contain a held target.
    let pictures: Vec<Option<Picture>> = lines
        .iter()
        .map(|line| {
            let Some(key) = keys.get(&line.id) else {
                return Ok(None);
            };
            let file = vidni_core::capture::frames_path(dir, key);
            if !file.is_file() {
                return Ok(None);
            }
            let frames = serde_json::from_slice(&std::fs::read(file).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
            Ok(Some(Picture {
                hash: release::file_hash(&vidni_core::capture::path(dir, key))?,
                frames,
            }))
        })
        .collect::<Result<_, String>>()?;
    let audio_hashes = audio
        .iter()
        .map(|(_, path)| release::file_hash(path))
        .collect::<Result<Vec<_>, _>>()?;
    let switches: Vec<u64> = (0..lines.len())
        .map(|i| {
            if i == 0 {
                0
            } else {
                frame((timeline.sentences[i - 1].end + timeline.sentences[i].start) / 2.0)
            }
        })
        .chain(std::iter::once(frame(timeline.total)))
        .collect();
    let mut found = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if line.emphasis.is_empty() {
            continue;
        }
        let scene_id = cut.scene_of(project, line).ok_or("emphasis has no scene")?;
        let scene = &project.scenes[scene_id];
        let picture = pictures[i]
            .as_ref()
            .ok_or("emphasis has no measured capture")?;
        let frames = &picture.frames;
        let audio = &audio[i].1;
        let wav = hound::WavReader::open(audio).map_err(|e| e.to_string())?;
        let spoken = vidni_core::spoken::spoken(line, &project.voice);
        // Include the surrounding review audio, incoming holds, camera motion,
        // and every possible outgoing hold. Neighbour changes cannot reuse an
        // old verification or a preview missing newly overlapping emphasis.
        let from = timeline.sentences[i].start - project.emphasis.min_duration - 0.35;
        let until = timeline.sentences[i].end + project.emphasis.min_duration + 0.35;
        let context: Vec<_> = lines
            .iter()
            .enumerate()
            .filter(|(j, _)| {
                *j == i.saturating_sub(1)
                    || (timeline.sentences[*j].end >= from && timeline.sentences[*j].start <= until)
            })
            .map(|(j, l)| {
                json!([
                    l,
                    cut.scene_of(project, l)
                        .and_then(|id| project.scenes.get(id)),
                    pictures[j].as_ref().map(|p| (&p.hash, &p.frames)),
                    audio_hashes[j],
                    timeline.sentences[j]
                ])
            })
            .collect();
        for cue in &line.emphasis {
            let boxes = frames.boxes(&cue.target, scene);
            if boxes.is_empty() {
                return Err(format!(
                    "{}: target {:?} has no measured geometry",
                    line.id, cue.target
                ));
            }
            let binding = Binding {
                audio: audio_hashes[i].clone(),
                rate: wav.spec().sample_rate,
                samples: u64::from(wav.duration()),
                words: cues::hash(&json!([
                    spoken,
                    vidni_core::spoken::synthetic(line, &project.voice)
                ])),
                definition: cues::hash(cue),
                visual: cues::hash(
                    &json!({"renderer":2,"policy":project.emphasis,"picture":picture.hash,
                    "frames":frames,"target":scene.targets.get(&cue.target),"scene":scene,
                    "camera":line.camera,"vertical":line.camera_vertical,"cut":cut,"theme":theme,
                    "context":context,"total":timeline.total,"subtitle":project.meta.subtitle,
                    "caption":line.show,"timing":project.timing,
                    "label":project.sections.get(&line.section),"overlay":cut.overlay_of(line).and_then(|id|project.scenes.get(id)),
                    "fonts":vidni_core::fonts::DATA}),
                ),
            };
            let key = cues::key(&cut.id, voice, &line.id, &cue.id);
            let check =
                |interval| visible_interval(&binding, timeline.sentences[i].start, interval);
            let checked = verified.current(&key, &binding).and_then(check);
            let problem = checked.as_ref().err().cloned();
            let interval = checked
                .as_ref()
                .ok()
                .copied()
                .or_else(|| proposals.current(&key, &binding).and_then(check).ok());
            let mut item = Cue {
                key,
                line: i,
                line_id: line.id.clone(),
                phrase: cue.phrase.clone(),
                occurrence: cue.occurrence,
                spoken: spoken.clone(),
                audio: audio.clone(),
                line_start: timeline.sentences[i].start,
                binding,
                boxes,
                interval,
                display: None,
                verified: checked.is_ok(),
                problem,
            };
            let identity = scene
                .targets
                .get(&cue.target)
                .and_then(|t| t.continuity.as_ref());
            let mut spans = vec![(i, switches[i], switches[i + 1], item.boxes.clone())];
            let possible_end = frame(timeline.sentences[i].end + project.emphasis.min_duration);
            let mut previous = picture;
            let mut previous_boxes = item.boxes.clone();
            for j in i + 1..lines.len() {
                if switches[j] >= possible_end {
                    break;
                }
                let Some(next) = pictures[j].as_ref() else {
                    break;
                };
                let next_scene = cut
                    .scene_of(project, lines[j])
                    .and_then(|id| project.scenes.get(id));
                let explicit = identity.and_then(|identity| {
                    next_scene.and_then(|s| {
                        s.targets
                            .iter()
                            .find(|(_, t)| t.continuity.as_ref() == Some(identity))
                            .map(|(name, _)| next.frames.boxes(name, s))
                    })
                });
                let next_boxes = match explicit {
                    Some(boxes) if !boxes.is_empty() => boxes,
                    Some(_) => {
                        return Err(format!(
                            "{}: continuation has no measured geometry",
                            item.key
                        ));
                    }
                    None if next.hash == previous.hash
                        && next.frames.viewport == previous.frames.viewport
                        && next.frames.scale == previous.frames.scale =>
                    {
                        previous_boxes.clone()
                    }
                    _ => break,
                };
                spans.push((j, switches[j], switches[j + 1], next_boxes.clone()));
                previous = next;
                previous_boxes = next_boxes;
            }
            item.display = display(
                &item,
                &spans,
                project.emphasis.min_duration,
                frame(timeline.total),
            );
            found.push(item);
        }
    }
    Ok(found)
}

/// Advisory token alignment. Its output never verifies a cue.
pub fn propose(dir: &Path, items: &[Cue]) -> Result<Vec<String>, String> {
    let mut records = Records::read(&dir.join(cues::PROPOSALS))?;
    let mut heard: BTreeMap<PathBuf, Result<Vec<Word>, String>> = BTreeMap::new();
    let mut notes = Vec::new();
    for cue in items {
        if cue.verified
            || records
                .current(&cue.key, &cue.binding)
                .and_then(|i| visible_interval(&cue.binding, cue.line_start, i))
                .is_ok()
        {
            continue;
        }
        let words = heard
            .entry(cue.audio.clone())
            .or_insert_with(|| align(dir, &cue.audio));
        let interval = words
            .as_ref()
            .map_err(Clone::clone)
            .and_then(|words| suggestion(cue, words));
        match interval {
            Ok(interval) => {
                records.entries.insert(
                    cue.key.clone(),
                    Record {
                        binding: cue.binding.clone(),
                        interval,
                    },
                );
            }
            Err(why) => notes.push(format!("{}: set timing during review ({why})", cue.key)),
        }
    }
    records.write(&dir.join(cues::PROPOSALS))?;
    Ok(notes)
}

#[derive(Debug, Clone)]
struct Word {
    text: String,
    start: f64,
    end: f64,
}

fn align(dir: &Path, audio: &Path) -> Result<Vec<Word>, String> {
    let model = crate::hear::model_path().ok_or("no local alignment model")?;
    if release::file_hash(&model)? != crate::hear::MODEL_SHA256 {
        return Err("alignment model differs from its pin".into());
    }
    let folder = dir.join("build/cues/alignment");
    std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    let digest = cues::hash(&json!([
        release::file_hash(audio)?,
        crate::hear::MODEL_SHA256,
        "word-segments-v1"
    ]));
    let out = folder.join(digest);
    let command = std::env::var_os("VIDNI_WHISPER").unwrap_or_else(|| "whisper-cli".into());
    if !out.with_extension("json").exists() {
        crate::run(
            std::process::Command::new(command)
                .arg("-m")
                .arg(model)
                .arg("-f")
                .arg(audio)
                .args(["-ojf", "-sow", "-ml", "1", "-ng", "-t", "4", "-of"])
                .arg(&out),
        )?;
    }
    let data: serde_json::Value = serde_json::from_slice(
        &std::fs::read(out.with_extension("json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let mut words = Vec::new();
    for segment in data["transcription"]
        .as_array()
        .ok_or("alignment returned no transcription")?
    {
        let text = vidni_core::spoken::words(segment["text"].as_str().unwrap_or(""));
        let start = segment["offsets"]["from"]
            .as_f64()
            .ok_or("alignment has no start")?
            / 1000.0;
        let end = segment["offsets"]["to"]
            .as_f64()
            .ok_or("alignment has no end")?
            / 1000.0;
        if !text.is_empty() && start < end {
            words.push(Word { text, start, end });
        }
    }
    Ok(words)
}

fn suggestion(cue: &Cue, words: &[Word]) -> Result<Interval, String> {
    // Whisper can split a name into pieces or join its words. This only
    // proposes a cue; no recognized spelling ever becomes verified timing.
    let phrase = vidni_core::spoken::words(&cue.phrase).replace(' ', "");
    let mut matches = Vec::new();
    for start in 0..words.len() {
        let mut text = String::new();
        for (end, word) in words.iter().enumerate().skip(start) {
            text.push_str(&word.text.replace(' ', ""));
            if text == phrase {
                matches.push((start, end));
                break;
            }
            if text.len() > phrase.len() {
                break;
            }
        }
    }
    let at = match (cue.occurrence, matches.as_slice()) {
        (None, [one]) => *one,
        (Some(n), _) => *n
            .checked_sub(1)
            .and_then(|n| matches.get(n as usize))
            .ok_or("alignment did not find that occurrence")?,
        _ => return Err("alignment did not find one unambiguous phrase".into()),
    };
    let rate = f64::from(cue.binding.rate);
    visible_interval(
        &cue.binding,
        cue.line_start,
        Interval {
            start: (words[at.0].start * rate).round() as u64,
            end: ((words[at.1].end * rate).round() as u64).min(cue.binding.samples),
        },
    )
}

/// Person-only verification, with a fresh binding supplied by the caller.
pub fn verify(dir: &Path, expected: &Cue, current: &Cue, interval: Interval) -> Result<(), String> {
    if std::env::var_os("VIDNI_ASSISTANT").is_some_and(|s| !s.is_empty()) {
        return Err(
            "cue verification belongs to the person watching and hearing the preview".into(),
        );
    }
    if expected.key != current.key || expected.binding != current.binding {
        return Err("review inputs changed; regenerate the preview before verifying".into());
    }
    visible_interval(&current.binding, current.line_start, interval)?;
    let mut records = Records::read(&dir.join(cues::LOCK))?;
    records.entries.insert(
        current.key.clone(),
        Record {
            binding: current.binding.clone(),
            interval,
        },
    );
    records.write(&dir.join(cues::LOCK))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preview {
    pub line_id: String,
    pub path: PathBuf,
    pub sha256: String,
    /// The cut frame represented by the preview's first frame.
    pub first_frame: u64,
    pub end_frame: u64,
    pub inputs: String,
    pub cues: Vec<Cue>,
    pub context: Vec<Cue>,
}

impl Preview {
    pub fn current(&self, cues: &[Cue]) -> bool {
        let cut: Vec<_> = cues
            .iter()
            .filter(|c| {
                self.cues
                    .first()
                    .is_some_and(|own| c.key.split('/').take(2).eq(own.key.split('/').take(2)))
            })
            .cloned()
            .collect();
        let context = preview_context(&cut, self.first_frame, self.end_frame);
        context.len() == self.context.len()
            && context
                .iter()
                .zip(&self.context)
                .all(|(a, b)| same_timing(a, b))
            && self
                .cues
                .iter()
                .all(|old| cues.iter().any(|c| same_timing(c, old)))
    }
}

fn same_timing(a: &Cue, b: &Cue) -> bool {
    a.key == b.key && a.binding == b.binding && a.interval == b.interval && a.display == b.display
}

fn preview_context(cues: &[Cue], start: u64, end: u64) -> Vec<Cue> {
    cues.iter()
        .filter(|c| c.events().iter().any(|e| e.start < end && e.end > start))
        .cloned()
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub schema: u32,
    pub cut: String,
    pub voice: Voice,
    pub cues: Vec<Cue>,
    pub previews: BTreeMap<String, Preview>,
}

pub const REVIEW_SCHEMA: u32 = 3;

pub fn review_path(dir: &Path, cut: &str, voice: Voice) -> PathBuf {
    let voice = if voice == Voice::Draft {
        "draft"
    } else {
        "takes"
    };
    dir.join("build/cues")
        .join(format!("{cut}-{voice}.review.json"))
}

fn review_master(plan: &crate::build::Plan<'_>, folder: &Path) -> Result<PathBuf, String> {
    std::fs::create_dir_all(folder).map_err(|e| e.to_string())?;
    let key = plan.review_audio_key();
    let narration = folder.join(format!("audio-{key}.wav"));
    if !narration.exists() {
        plan.review_audio(&narration)?;
    }
    let stereo = folder.join(format!("audio-{key}-stereo.wav"));
    if !stereo.exists() {
        crate::encode::stereo(&narration, &stereo)?;
    }
    Ok(stereo)
}

/// Continuous diagnostic: the cut's unchanged timeline and master audio, with
/// a permanent review label. It never writes a production build or cue lock.
pub fn full_preview(
    plan: &crate::build::Plan<'_>,
    dir: &Path,
    progress: &mut dyn FnMut(u64, u64),
) -> Result<PathBuf, String> {
    let folder = dir.join("build/cues/full");
    std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    let voice = if plan.voice == Voice::Draft {
        "draft"
    } else {
        "takes"
    };
    let path = folder.join(format!("{}-{voice}.review.mp4", plan.cut.id));
    let receipt = path.with_extension("json");
    let inputs = cues::hash(&json!([
        "full-review-1",
        REVIEW_SCHEMA,
        plan.inputs,
        plan.emphasis
    ]));
    if let Ok(bytes) = std::fs::read(&receipt)
        && let Ok(previous) = serde_json::from_slice::<serde_json::Value>(&bytes)
        && previous["inputs"].as_str() == Some(&inputs)
        && let Ok(hash) = release::file_hash(&path)
        && previous["sha256"].as_str() == Some(&hash)
    {
        return Ok(path);
    }
    let mut composer = plan.composer(true)?;
    let audio = review_master(plan, &dir.join("build/cues"))?;
    let (w, h) = composer.size();
    let temporary = path.with_extension("new.mp4");
    let settings = crate::encode::Settings {
        preset: "fast",
        ..crate::encode::Settings::new(w, h)
    };
    let mut encoder = crate::encode::Encoder::start(&settings, &audio, &temporary)?;
    let mut fonts = crate::compose::text::Fonts::new();
    let face = vidni_core::fonts::get("ibm-plex-sans-bold").ok_or("review font is missing")?;
    let font = fonts.load(&vidni_core::fonts::find(&face, &vidni_core::fonts::dirs())?)?;
    let label = if plan.emphasis.iter().any(|cue| !cue.verified) {
        "REVIEW COPY — HIGHLIGHT TIMING UNVERIFIED"
    } else {
        "REVIEW COPY — NOT A FINAL EXPORT"
    };
    let frames = composer.frames();
    let mut last_key = None;
    let mut picture = None;
    for frame in 0..frames {
        let key = composer.key(frame);
        if last_key != Some(key) {
            let mut image = composer.frame(frame)?;
            crate::compose::pillow::rectangle(&mut image, (0, 0, w as i32, 38), &[11, 11, 16]);
            fonts.draw(
                &mut image,
                (12.0, 7.0),
                label,
                crate::compose::text::Pen {
                    font: &font,
                    size: 22.0,
                    ink: [242, 84, 45],
                },
                "la",
            );
            picture = Some(image);
            last_key = Some(key);
        }
        encoder.push(&picture.as_ref().ok_or("review has no frame")?.data)?;
        progress(frame + 1, frames);
    }
    encoder.finish()?;
    std::fs::rename(temporary, &path).map_err(|e| e.to_string())?;
    let record = json!({"schema":1,"kind":"diagnostic-review","inputs":inputs,
        "review_identity":plan.review_identity(),
        "path":path,"sha256":release::file_hash(&path)?,"cut":plan.cut.id,"voice":voice,
        "size":[w,h],"fps":30,"frames":frames,"duration":plan.timeline().total,
        "audio_sha256":release::file_hash(&audio)?,"emphasis":plan.emphasis});
    std::fs::write(
        receipt,
        serde_json::to_vec_pretty(&record).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(path)
}

/// Check an owner's confirmation of an exact continuous review. This does not
/// write a lock: callers validate every selected cut before one atomic save.
/// The statement is the owner's decision, not an assistant's self-verification.
pub fn confirmed_full_review(
    plan: &crate::build::Plan<'_>,
    dir: &Path,
    expected_sha256: &str,
    owner_statement: &str,
    relayed_by_assistant: bool,
) -> Result<(BTreeMap<String, Record>, cues::Confirmation), String> {
    if owner_statement.trim().is_empty() {
        return Err("an explicit owner review statement is required".into());
    }
    if expected_sha256.len() != 64
        || !expected_sha256
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err("the reviewed video's lowercase SHA-256 is required".into());
    }
    let voice = if plan.voice == Voice::Draft {
        "draft"
    } else {
        "takes"
    };
    let movie = dir.join(format!(
        "build/cues/full/{}-{voice}.review.mp4",
        plan.cut.id
    ));
    let receipt = movie.with_extension("json");
    let bytes = std::fs::read(&receipt).map_err(|e| format!("{}: {e}", receipt.display()))?;
    let data: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if data["schema"] != 1
        || data["kind"] != "diagnostic-review"
        || data["cut"] != plan.cut.id
        || data["voice"] != voice
        || data["fps"] != 30
        || data["size"] != json!(plan.cut.size())
        || data["duration"].as_f64() != Some(plan.timeline().total)
    {
        return Err("the continuous review receipt does not describe this cut and voice".into());
    }
    if data["sha256"] != expected_sha256 || release::file_hash(&movie)? != expected_sha256 {
        return Err("the review video differs from the exact video the owner confirmed".into());
    }
    let reviewed: Vec<Cue> =
        serde_json::from_value(data["emphasis"].clone()).map_err(|e| e.to_string())?;
    if plan.emphasis.is_empty()
        || reviewed.len() != plan.emphasis.len()
        || !reviewed
            .iter()
            .zip(&plan.emphasis)
            .all(|(a, b)| same_timing(a, b))
    {
        return Err(
            "reviewed cues differ from the current audio, wording, layout or timing".into(),
        );
    }
    let identity = plan.review_identity();
    let receipt_sha256 = vidni_core::capture::sha256_hex(&bytes);
    let legacy_inputs = cues::hash(&json!([
        "full-review-1",
        REVIEW_SCHEMA,
        plan.inputs,
        reviewed
    ]));
    // Old receipts lack the status-independent identity. Their exact legacy
    // input hash is checked on first confirmation; its saved audit allows an
    // unchanged confirmation to be repeated after the lock itself changes.
    let previous = Records::read(&dir.join(cues::LOCK))?;
    let already_current = previous.confirmations.iter().any(|c| {
        c.review_sha256 == expected_sha256
            && c.receipt_sha256 == receipt_sha256
            && c.review_identity == identity
            && c.cut == plan.cut.id
            && c.voice == plan.voice
    });
    let current = match data.get("review_identity") {
        Some(held) => held.as_str() == Some(&identity),
        None => data["inputs"] == legacy_inputs || already_current,
    };
    if !current {
        return Err(
            "the review video is stale; render and review the changed cut before confirming".into(),
        );
    }
    let mut records = BTreeMap::new();
    for cue in &plan.emphasis {
        let interval = cue
            .interval
            .ok_or("the reviewed video has an unset highlight interval")?;
        visible_interval(&cue.binding, cue.line_start, interval)?;
        if cue.display.is_none() {
            return Err("the reviewed video has no visible display interval for a cue".into());
        }
        records.insert(
            cue.key.clone(),
            Record {
                binding: cue.binding.clone(),
                interval,
            },
        );
    }
    let confirmation = cues::Confirmation {
        cut: plan.cut.id.clone(),
        voice: plan.voice,
        owner_statement: owner_statement.to_string(),
        recorded_by: if relayed_by_assistant {
            "assistant relaying owner confirmation"
        } else {
            "owner"
        }
        .into(),
        review_sha256: expected_sha256.into(),
        receipt_sha256,
        review_identity: identity,
        cue_keys: records.keys().cloned().collect(),
    };
    Ok((records, confirmation))
}

/// Small, full-resolution clips, visibly marked as diagnostic. Only requested
/// lines are rebuilt. Transition boundary frames are also saved losslessly.
pub fn previews(
    plan: &crate::build::Plan<'_>,
    dir: &Path,
    only: &[String],
) -> Result<Review, String> {
    let file = review_path(dir, &plan.cut.id, plan.voice);
    let mut review: Review = std::fs::read(&file)
        .ok()
        .and_then(|v| serde_json::from_slice::<Review>(&v).ok())
        .filter(|r| r.schema == REVIEW_SCHEMA)
        .unwrap_or_else(|| Review {
            schema: REVIEW_SCHEMA,
            cut: plan.cut.id.clone(),
            voice: plan.voice,
            cues: Vec::new(),
            previews: BTreeMap::new(),
        });
    review.cues = plan.emphasis.clone();
    let folder = file.parent().ok_or("review needs a folder")?;
    std::fs::create_dir_all(folder).map_err(|e| e.to_string())?;
    let mut composer = plan.composer(true)?;
    // Includes room tone, music and loudness, not only the dry narration clips.
    // Match the production master before slicing, including resampling phase
    // and channel levels. Resampling each small clip separately would differ.
    let stereo = review_master(plan, folder)?;
    let mut reader = hound::WavReader::open(&stereo).map_err(|e| e.to_string())?;
    let spec = reader.spec();
    if spec.channels != 2 || spec.sample_rate != 48000 || spec.bits_per_sample != 16 {
        return Err("review audio must match the 48 kHz stereo production master".into());
    }
    let rate = spec.sample_rate;
    let samples = reader
        .samples::<i16>()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let lines: BTreeMap<usize, String> = plan
        .emphasis
        .iter()
        .filter(|c| only.is_empty() || only.contains(&c.line_id))
        .map(|c| (c.line, c.line_id.clone()))
        .collect();
    for (i, id) in lines {
        let line_cues: Vec<Cue> = plan
            .emphasis
            .iter()
            .filter(|c| c.line == i)
            .cloned()
            .collect();
        let entry = &plan.timeline().sentences[i];
        let first = ((entry.start - 0.35).max(0.0) * 30.0).floor() as u64;
        let last_hold = line_cues
            .iter()
            .filter_map(|c| c.display.as_ref().map(|d| d.end as f64 / 30.0))
            .fold(entry.end, f64::max);
        let end = ((last_hold + 0.35).min(plan.timeline().total) * 30.0).ceil() as u64;
        let context = preview_context(&plan.emphasis, first, end);
        let digest = cues::hash(&json!([
            REVIEW_SCHEMA,
            line_cues,
            context,
            plan.inputs,
            first,
            end
        ]));
        if review.previews.get(&id).is_some_and(|p| {
            p.inputs == digest
                && p.current(&plan.emphasis)
                && release::file_hash(&p.path).ok().as_deref() == Some(&p.sha256)
        }) {
            continue;
        }
        let path = folder.join(format!("{}-{id}-{}.review.mp4", plan.cut.id, &digest[..12]));
        let audio = folder.join(format!("{id}.preview.wav"));
        let mut writer = hound::WavWriter::create(
            &audio,
            hound::WavSpec {
                channels: spec.channels,
                sample_rate: rate,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            },
        )
        .map_err(|e| e.to_string())?;
        let from =
            (first as f64 / 30.0 * f64::from(rate)).round() as usize * usize::from(spec.channels);
        let to =
            (end as f64 / 30.0 * f64::from(rate)).round() as usize * usize::from(spec.channels);
        for j in from..to {
            writer
                .write_sample(samples.get(j).copied().unwrap_or(0))
                .map_err(|e| e.to_string())?;
        }
        writer.finalize().map_err(|e| e.to_string())?;
        let (w, h) = composer.size();
        let settings = crate::encode::Settings {
            preset: "ultrafast",
            ..crate::encode::Settings::new(w, h)
        };
        let mut encoder = crate::encode::Encoder::start(&settings, &audio, &path)?;
        let mut fonts = crate::compose::text::Fonts::new();
        let face = vidni_core::fonts::get("ibm-plex-sans-bold").ok_or("review font is missing")?;
        let font = fonts.load(&vidni_core::fonts::find(&face, &vidni_core::fonts::dirs())?)?;
        let mut boundaries = std::collections::BTreeSet::new();
        for cue in &line_cues {
            for event in cue.events() {
                boundaries.extend([
                    event.start.saturating_sub(1),
                    event.start,
                    (event.start + event.end) / 2,
                    event.end.saturating_sub(1),
                    event.end,
                ]);
            }
        }
        let mut last_key = None;
        let mut base = None;
        let mut stamped = None;
        for f in first..end {
            let key = composer.key(f);
            if last_key != Some(key) {
                let frame = composer.frame(f)?;
                let mut marked = frame.clone();
                crate::compose::pillow::rectangle(&mut marked, (0, 0, w as i32, 38), &[11, 11, 16]);
                fonts.draw(
                    &mut marked,
                    (12.0, 7.0),
                    "CUE REVIEW — NOT AN EXPORT",
                    crate::compose::text::Pen {
                        font: &font,
                        size: 22.0,
                        ink: [242, 84, 45],
                    },
                    "la",
                );
                base = Some(frame);
                stamped = Some(marked);
                last_key = Some(key);
            }
            if boundaries.contains(&f) {
                base.as_ref()
                    .ok_or("no preview frame")?
                    .save_png(&folder.join(format!("{}-{id}-frame-{f}.png", plan.cut.id)))?;
            }
            encoder.push(&stamped.as_ref().ok_or("no preview frame")?.data)?;
        }
        encoder.finish()?;
        let _ = std::fs::remove_file(audio);
        review.previews.insert(
            id.clone(),
            Preview {
                line_id: id,
                path: path.clone(),
                sha256: release::file_hash(&path)?,
                first_frame: first,
                end_frame: end,
                inputs: digest,
                cues: line_cues,
                context,
            },
        );
        // Keep completed work reusable if a later line fails or is interrupted.
        std::fs::write(
            &file,
            serde_json::to_vec_pretty(&review).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
    }
    std::fs::write(
        file,
        serde_json::to_vec_pretty(&review).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(review)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cue(start: u64, end: u64) -> Cue {
        Cue {
            key: "long/draft/line/cue".into(),
            line: 0,
            line_id: "line".into(),
            phrase: "name".into(),
            occurrence: None,
            spoken: "name".into(),
            audio: PathBuf::new(),
            line_start: 0.0,
            binding: Binding {
                audio: "audio".into(),
                rate: 30,
                samples: 300,
                words: "words".into(),
                definition: "cue".into(),
                visual: "visual".into(),
            },
            boxes: vec![[100.0, 100.0, 50.0, 30.0]],
            interval: Some(Interval { start, end }),
            display: None,
            verified: false,
            problem: None,
        }
    }

    #[test]
    fn display_hold_preserves_long_phrases_and_reports_early_boundaries() {
        let cue = test_cue(20, 35);
        let spans = vec![
            (0, 0, 60, cue.boxes.clone()),
            (1, 60, 200, vec![[200.0, 100.0, 50.0, 30.0]]),
        ];
        let d = display(&cue, &spans, 3.0, 200).unwrap();
        assert_eq!((d.start, d.end), (20, 110));
        assert_eq!(d.segments.len(), 2);
        assert_eq!(d.segments[0].end, d.segments[1].start);
        let d = display(&cue, &spans[..1], 3.0, 200).unwrap();
        assert_eq!(d.end, 60);
        assert_eq!(d.cutoff.as_deref(), Some("scene change removes the target"));
        let d = display(&cue, &spans, 3.0, 80).unwrap();
        assert_eq!(d.end, 80);
        assert_eq!(d.cutoff.as_deref(), Some("end of cut"));
        let d = display(&test_cue(20, 150), &spans, 3.0, 200).unwrap();
        assert_eq!(d.end, 150);
        let d = display(&cue, &spans, 0.0, 200).unwrap();
        assert_eq!(d.end, 35);
    }

    #[test]
    fn preview_detects_changed_or_new_overlapping_cues_but_ignores_other_cuts() {
        let mut own = test_cue(20, 35);
        let spans = [(0, 0, 300, own.boxes.clone())];
        own.display = display(&own, &spans, 3.0, 300);
        let preview = Preview {
            line_id: "line".into(),
            path: PathBuf::new(),
            sha256: "hash".into(),
            first_frame: 10,
            end_frame: 121,
            inputs: "inputs".into(),
            cues: vec![own.clone()],
            context: vec![own.clone()],
        };
        assert!(preview.current(&[own.clone()]));
        let mut other = own.clone();
        other.key = "short/draft/line/cue".into();
        assert!(preview.current(&[own.clone(), other]));
        let mut neighbour = own.clone();
        neighbour.key = "long/draft/next/other".into();
        assert!(!preview.current(&[own.clone(), neighbour]));
        let mut changed = own.clone();
        changed.display.as_mut().unwrap().end += 1;
        assert!(!preview.current(&[changed]));
        own.verified = true;
        assert!(preview.current(&[own]));
    }

    #[test]
    fn a_proposal_that_rounds_to_no_frame_is_not_reviewable() {
        let binding = Binding {
            audio: String::new(),
            rate: 22050,
            samples: 40000,
            words: String::new(),
            definition: String::new(),
            visual: String::new(),
        };
        assert!(
            visible_interval(
                &binding,
                0.0,
                Interval {
                    start: 37265,
                    end: 37376
                }
            )
            .unwrap_err()
            .contains("no video frame")
        );
        assert!(
            visible_interval(
                &binding,
                0.0,
                Interval {
                    start: 36000,
                    end: 37376
                }
            )
            .is_ok()
        );
    }
}
