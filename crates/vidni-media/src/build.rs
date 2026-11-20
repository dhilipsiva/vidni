//! Building a cut: from its lines' audio and its captures to its encoded
//! files, and the record of what they were made from (TODO.md,
//! P3-VIDNI-06).
//!
//! A draft speaks with the draft voice and a final build with the creator's
//! takes. Otherwise the two are built alike:
//! 1. the timeline and the captions, from the clips' lengths;
//! 2. the narration track, and its loudness-normalized copy;
//! 3. each picture of the cut, composited and encoded;
//! 4. QA of each encoded file.
//!
//! The build's record (`<cut>.build.json`) names every input and output by
//! hash. The gates are here and in `vidni_core::release`, so the command
//! line and the app refuse the same things (TODO.md, rule 4): a final build
//! needs a ratified, unchanged cut and a current take for every line, and
//! it is delivered to `out/` only when QA passes. Approving a cut needs
//! that build, still current, and its files as they were delivered.

use crate::capture::Frames;
use crate::compose::card::Card;
use crate::compose::cut::{
    Composer, FPS, Ground, LANDSCAPE, Layout, Look, PORTRAIT, Picture, Shot, Type,
};
use crate::compose::text::Fonts;
use crate::encode::{self, Encoder, Settings, Target};
use crate::{compose, qa, review, run, voice};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use vidni_core::captions;
use vidni_core::capture::{self, sha256_hex};
use vidni_core::chapters::{self, Chapter};
use vidni_core::project::{Aspect, Captions, Cut, Line, Project, Scene, SceneKind};
use vidni_core::ratify;
use vidni_core::release::{self, Approval, Output, Record, Sources, T1, Tools, Voice};
use vidni_core::theme::{self, Theme};
use vidni_core::timeline::{self, Clip, Timeline};
use vidni_core::voices::{self, Pack};
use vidni_core::{VERSION, fonts};

/// How far a cut's loudness may be from its target, in LU.
const LOUDNESS_TOLERANCE: f64 = 1.0;

/// The highest true peak a cut may have, in dBTP.
const TRUE_PEAK: f64 = -1.0;

/// The room tone the mic check keeps, in the project folder.
const ROOM: &str = "takes/_room/tone.wav";

/// Counts the changes to what the compositor draws. A change makes every
/// earlier build's record out of date.
const COMPOSITOR: u32 = 2;

/// The font files a cut is drawn with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Faces {
    pub caption: PathBuf,
    pub mono: PathBuf,
    pub display: PathBuf,
    /// The font of the cut's script, when the theme's fonts haven't its
    /// letters: it draws them, and the theme's draw the rest.
    pub script: Option<PathBuf>,
}

impl Faces {
    /// The fonts `theme` names, found in `dirs` and checked by their
    /// SHA-256.
    pub fn pinned(theme: &Theme, dirs: &[PathBuf]) -> Result<Self, String> {
        let find = |id: &str| {
            let font = fonts::get(id).ok_or_else(|| format!("no pinned font {id}"))?;
            fonts::find(&font, dirs)
        };
        Ok(Self {
            caption: find(&theme.fonts.text)?,
            mono: find(&theme.fonts.mono)?,
            display: find(&theme.fonts.display)?,
            script: None,
        })
    }

    /// The same faces for a cut in `language`: with its script's pinned
    /// font, where vidni pins one for it.
    pub fn for_language(mut self, language: &str, dirs: &[PathBuf]) -> Result<Self, String> {
        if let Some(font) = fonts::for_language(language) {
            self.script = Some(fonts::find(&font, dirs)?);
        }
        Ok(self)
    }

    /// Loads the faces: each of the theme's, behind the script's font
    /// where there is one.
    pub fn load(&self, fonts: &mut Fonts) -> Result<Type, String> {
        let script = match &self.script {
            Some(file) => Some(fonts.load(file)?),
            None => None,
        };
        let mut face = |file: &Path| -> Result<_, String> {
            let font = fonts.load(file)?;
            Ok(match &script {
                Some(script) => fonts.with_fallback(script, &font),
                None => font,
            })
        };
        Ok(Type {
            caption: face(&self.caption)?,
            mono: face(&self.mono)?,
            display: face(&self.display)?,
        })
    }

    /// One file for all three faces.
    pub fn all(file: &Path) -> Self {
        Self {
            caption: file.to_path_buf(),
            mono: file.to_path_buf(),
            display: file.to_path_buf(),
            script: None,
        }
    }

    fn files(&self) -> Vec<&Path> {
        let mut files: Vec<&Path> = vec![&self.caption, &self.mono, &self.display];
        files.extend(self.script.as_deref());
        files
    }
}

/// What a build runs with, besides the project: the fonts, and the ffmpeg
/// on the path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Setup {
    pub faces: Faces,
    /// ffmpeg's version, as it prints it.
    pub ffmpeg: String,
    /// Build even when an earlier build of the same inputs is still whole.
    pub again: bool,
    /// The theme the cuts are drawn in.
    pub theme: Theme,
}

impl Setup {
    /// The theme of the project in `dir`, its fonts from the font
    /// folders, and the ffmpeg on the path.
    pub fn find(project: &Project, dir: &Path) -> Result<Self, String> {
        let theme = theme::load(dir, &project.style.theme)?;
        Ok(Self {
            faces: Faces::pinned(&theme, &fonts::dirs())?
                .for_language(&project.meta.language, &fonts::dirs())?,
            ffmpeg: ffmpeg_version()?,
            again: false,
            theme,
        })
    }

    /// `faces` and `theme`, with the ffmpeg on the path.
    pub fn with(faces: Faces, theme: Theme) -> Result<Self, String> {
        Ok(Self {
            faces,
            ffmpeg: ffmpeg_version()?,
            again: false,
            theme,
        })
    }

    fn tools(&self) -> Result<Tools, String> {
        let mut fonts = BTreeMap::new();
        for file in self.faces.files() {
            let name = file
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            fonts.insert(name, release::file_hash(file)?);
        }
        let settings = Settings::new(0, 0);
        let loudness = Target::default();
        Ok(Tools {
            vidni: VERSION.to_string(),
            ffmpeg: self.ffmpeg.clone(),
            fonts,
            engine: json!({
                "compositor": COMPOSITOR,
                "voice": voice::VERSION,
                "theme": self.theme,
                "fps": settings.fps,
                "crf": settings.crf,
                "preset": settings.preset,
                "threads": settings.threads,
                "x264": encode::X264_PARAMS,
                "aac": encode::AAC_PARAMS,
                "audio_kbps": settings.audio_kbps,
                "loudness": [loudness.integrated, loudness.true_peak, loudness.range],
            }),
        })
    }
}

/// The version of the ffmpeg on the path: `9.0.2` of `ffmpeg version 9.0.2 …`.
fn ffmpeg_version() -> Result<String, String> {
    let out = Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map_err(|e| format!("ffmpeg: {e}; vidni encodes with the ffmpeg on the path"))?;
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .next()
        .and_then(|line| line.strip_prefix("ffmpeg version "))
        .and_then(|rest| rest.split_whitespace().next())
        .map(str::to_string)
        .ok_or_else(|| "ffmpeg -version printed no version".to_string())
}

/// Why a build, or an approval, didn't happen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stop {
    /// A gate refused it: each reason is the creator's to fix.
    Refused(Vec<String>),
    /// Something broke: a tool, or a file.
    Failed(String),
}

impl Stop {
    fn refused(reason: String) -> Self {
        Stop::Refused(vec![reason])
    }
}

/// What a build is doing, for whoever is watching.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step<'a> {
    /// A stage begins.
    Stage(&'a str),
    /// A picture's frames: so many of so many are encoded.
    Frames { file: &'a str, done: u64, of: u64 },
    /// Nothing is built: an earlier build of the same inputs is still whole.
    Kept,
}

/// The camera a web line is framed by in a cut (docs/spec.md, Cameras).
/// `highlights` is the width of the line's highlight boxes together, when
/// it has any.
pub fn camera_name(line: &Line, scene: &Scene, portrait: bool, highlights: Option<f64>) -> String {
    let landscape = line
        .camera
        .clone()
        .or_else(|| scene.default_camera.clone())
        .unwrap_or_else(|| "viewport".into());
    if !portrait {
        return landscape;
    }
    if let Some(vertical) = &line.camera_vertical {
        return vertical.clone();
    }
    if let (Some(limit), Some(width)) = (scene.vertical_follows_highlights, highlights)
        && width <= limit
    {
        return "highlights".into();
    }
    scene
        .cameras
        .get(&landscape)
        .and_then(|c| c.vertical.clone())
        .unwrap_or(landscape)
}

/// Everything one build of a cut is made from, worked out before anything
/// is rendered.
pub struct Plan<'p> {
    project: &'p Project,
    pub(crate) cut: &'p Cut,
    pub(crate) voice: Voice,
    layout: Layout,
    /// Each line's clip, in script order.
    clips: Vec<PathBuf>,
    timeline: Timeline,
    /// The timeline as its file holds it.
    timeline_json: String,
    shots: Vec<Shot>,
    /// Each picture to encode: its file name, and whether captions are
    /// drawn into it.
    pictures: Vec<(String, bool)>,
    faces: Faces,
    look: Look,
    /// The cut's chapters; none when it has no list.
    chapters: Vec<Chapter>,
    /// The room the takes were recorded in, to fill the pauses with; none
    /// for a draft, or when the mic check kept no room at the takes' rate.
    room: Option<Vec<i16>>,
    /// The music under the narration, when the project plays a track.
    music: Option<voice::Bed>,
    /// The loudness the cut is mastered to, in LUFS.
    loudness: f64,
    tools: Tools,
    pub sources: Sources,
    /// Everything the build depends on, as one hash.
    pub inputs: String,
    pub emphasis: Vec<crate::cues::Cue>,
    diagnostic: bool,
}

/// The lines of `cut` that the draft voice has spoken, each with its clip;
/// or the refusal that names the ones it hasn't.
fn drafted(
    project: &Project,
    dir: &Path,
    cut: &Cut,
    pack: &Pack,
) -> Result<Vec<(String, PathBuf)>, Stop> {
    let lock = voices::read_lock(dir).map_err(Stop::Failed)?;
    let lines = cut.lines_in(project);
    let (mut clips, mut unspoken) = (Vec::new(), Vec::new());
    for line in &lines {
        match voices::drafted(dir, &lock, pack, &project.voice, line) {
            Some(clip) => clips.push((line.id.clone(), clip)),
            None => unspoken.push(line.id.as_str()),
        }
    }
    match unspoken.first() {
        None => Ok(clips),
        Some(first) => Err(Stop::refused(format!(
            "cut {}: the draft voice hasn't spoken {} of its {} lines as they are now (the first: {first}); `vidni draft` speaks them",
            cut.id,
            unspoken.len(),
            lines.len()
        ))),
    }
}

/// Each line's shot in `cut`, from the project's captures.
fn shots(project: &Project, dir: &Path, cut: &Cut, lines: &[&Line]) -> Result<Vec<Shot>, String> {
    let keys = capture::keys_in(project, cut);
    let portrait = cut.aspect == Aspect::Portrait;
    let page =
        |id: &str| capture::page_capture(project, cut, id).map(|c| capture::path(dir, &c.key));
    lines
        .iter()
        .map(|line| {
            let label = project
                .sections
                .get(&line.section)
                .map(|s| s.label.clone())
                .unwrap_or_default();
            // The cut's own scenes, where it swaps one for the line's.
            let overlay = cut.overlay_of(line).and_then(page);
            // An overlay scene is data: the compositor draws it.
            let strip = cut
                .overlay_of(line)
                .and_then(|id| project.scenes.get(id))
                .filter(|scene| scene.kind == SceneKind::Overlay)
                .cloned();
            let scene_id = cut
                .scene_of(project, line)
                .ok_or_else(|| format!("line {} shows no scene", line.id))?;
            let drawn = project.scenes.get(scene_id).and_then(Card::of);
            let picture = match (drawn, page(scene_id)) {
                (Some(card), _) => Picture::Drawn(card),
                (None, Some(card)) => Picture::Card(card),
                (None, None) => {
                    let scene = project
                        .scenes
                        .get(scene_id)
                        .ok_or_else(|| format!("line {} shows an unknown scene", line.id))?;
                    let key = keys
                        .get(&line.id)
                        .ok_or_else(|| format!("line {} has no capture", line.id))?;
                    let file = capture::frames_path(dir, key);
                    let frames: Frames = fs::read_to_string(&file)
                        .map_err(|e| e.to_string())
                        .and_then(|text| serde_json::from_str(&text).map_err(|e| e.to_string()))
                        .map_err(|e| format!("{}: {e}", file.display()))?;
                    // Cameras are framed now, from the boxes the capture
                    // measured, so a camera edited since frames what it names today.
                    let highlights = frames.camera("highlights", scene).map(|b| b[2]);
                    let name = camera_name(line, scene, portrait, highlights);
                    let viewport = [f64::from(frames.viewport[0]), f64::from(frames.viewport[1])];
                    let camera = frames.camera(&name, scene).ok_or_else(|| {
                        format!(
                            "line {}: nothing that its camera {name} names shows in its capture",
                            line.id
                        )
                    })?;
                    Picture::Still {
                        path: capture::path(dir, key),
                        camera,
                        viewport,
                        scale: f64::from(frames.scale),
                        // A web page is captured at the cut's own shape.
                        // A picture, a session or code is whatever size
                        // it is, and is shown whole.
                        loose: scene.kind != SceneKind::Web,
                        ground: match scene.is_text() {
                            true => Ground::Rows,
                            false => Ground::Corner,
                        },
                    }
                }
            };
            Ok(Shot {
                label,
                picture,
                overlay,
                strip,
            })
        })
        .collect()
}

/// Works out what a build of `cut` with `voice` is made from. `pack` is
/// the draft voice's pack, which a draft needs to know its clips by.
pub fn plan<'p>(
    project: &'p Project,
    dir: &Path,
    cut: &'p Cut,
    voice: Voice,
    pack: Option<&Pack>,
    setup: &Setup,
) -> Result<Plan<'p>, Stop> {
    plan_with_review(project, dir, cut, voice, pack, setup, false)
}

/// A diagnostic plan may show unverified proposals, but cannot be exported.
pub fn plan_with_review<'p>(
    project: &'p Project,
    dir: &Path,
    cut: &'p Cut,
    voice: Voice,
    pack: Option<&Pack>,
    setup: &Setup,
    diagnostic: bool,
) -> Result<Plan<'p>, Stop> {
    // A cut is laid out clear of what its platforms' apps cover.
    let covered = cut.covered();
    let layout = match (cut.aspect, cut.size(), covered) {
        (Aspect::Landscape, [1920, 1080], covered) => Layout {
            covered,
            ..LANDSCAPE
        },
        (Aspect::Portrait, [1080, 1920], None) => PORTRAIT,
        (Aspect::Portrait, [1080, 1920], Some(covered)) => Layout::clear_of(covered)
            .map_err(|why| Stop::refused(format!("cut {}: {why}", cut.id)))?,
        (_, [w, h], _) => {
            return Err(Stop::refused(format!(
                "cut {} is {w}x{h}; vidni draws 16:9 at 1920x1080 and 9:16 at 1080x1920 for now",
                cut.id
            )));
        }
    };
    let clips = match (voice, pack) {
        (Voice::Takes, _) => release::takes_of(project, dir, cut).map_err(Stop::Refused)?,
        (Voice::Draft, Some(pack)) => drafted(project, dir, cut, pack)?,
        (Voice::Draft, None) => {
            return Err(Stop::Failed("a draft needs its voice's pack".into()));
        }
    };
    let unready = capture::refusals(dir, &capture::for_cut(project, cut));
    if !unready.is_empty() {
        return Err(Stop::refused(format!(
            "cut {}: {}; `vidni draft` takes them",
            cut.id,
            unready.join("; ")
        )));
    }
    let lines = cut.lines_in(project);
    let infos = clips
        .iter()
        .map(|(_, path)| {
            let reader =
                hound::WavReader::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
            Ok(Clip {
                wav: release::relative(dir, path),
                frames: u64::from(reader.duration()),
                rate: reader.spec().sample_rate,
            })
        })
        .collect::<Result<Vec<Clip>, String>>()
        .map_err(Stop::Failed)?;
    let mut timeline = timeline::build_for(project, &lines, &infos, voice).map_err(Stop::Failed)?;
    if let Some(hold) = cut.end_hold {
        timeline.hold(hold);
    }
    let emphasis = crate::cues::collect(crate::cues::Inputs {
        project,
        dir,
        cut,
        voice,
        audio: &clips,
        timeline: &timeline,
        theme: &setup.theme,
    })
    .map_err(Stop::Failed)?;
    if !diagnostic {
        let problems: Vec<String> = emphasis
            .iter()
            .filter(|cue| !cue.verified)
            .filter_map(|cue| cue.problem.clone())
            .collect();
        if !problems.is_empty() {
            return Err(Stop::Refused(problems));
        }
    }
    // A line whose scene the cut swaps shows a card where the cut has one.
    for (entry, line) in timeline.sentences.iter_mut().zip(&lines) {
        if line.scene_in(project) != cut.scene_of(project, line) {
            let card = cut.scene_of(project, line).filter(|id| {
                let kind = project.scenes.get(*id).map(|scene| scene.kind);
                matches!(kind, Some(SceneKind::Page | SceneKind::Card))
            });
            entry.card = card.map(str::to_string);
        }
    }
    let timeline_json = serde_json::to_string_pretty(&timeline)
        .map(|json| json + "\n")
        .map_err(|e| Stop::Failed(e.to_string()))?;
    let shots = shots(project, dir, cut, &lines).map_err(Stop::Failed)?;
    let chapters = match chapters::wanted(cut) {
        true => chapters::of(project, &timeline),
        false => Vec::new(),
    };
    // A draft is for watching, so it is one picture, with the captions a
    // viewer would see. The final build makes every picture the cut asks for.
    let name = |suffix: &str| format!("{}{suffix}.mp4", cut.id);
    let pictures = match (cut.captions(), voice) {
        (Captions::File, _) => vec![(name(""), false)],
        (Captions::Burned, _) | (Captions::Both, Voice::Draft) => vec![(name(""), true)],
        (Captions::Both, Voice::Takes) => vec![(name(""), false), (name(".captioned"), true)],
    };
    // The pauses of a cut in the creator's voice hold the creator's room,
    // which is then one of the cut's sounds.
    let room_file = dir.join(ROOM);
    let room = match voice {
        Voice::Takes => crate::voice::room(&room_file, timeline.rate),
        Voice::Draft => None,
    };
    let mut sounds = clips.clone();
    if room.is_some() {
        sounds.push(("_room".to_string(), room_file));
    }
    // The track the project plays is one of the cut's sounds too.
    let music = match project.music.playing() {
        None => None,
        Some(track) => {
            // YouTube blocks a Short of over a minute that has a Content
            // ID claim, in every country.
            if track.content_id
                && timeline.total > 60.0
                && cut.platforms().iter().any(|p| p == "youtube-shorts")
            {
                return Err(Stop::refused(format!(
                    "cut {} lasts {:.0} s and track {} is registered with Content ID: YouTube \
                     blocks a Short of over a minute with a Content ID claim, worldwide",
                    cut.id, timeline.total, track.id
                )));
            }
            let file = dir.join(&track.file);
            let samples = crate::voice::track(&file, timeline.rate).map_err(Stop::Failed)?;
            sounds.push(("_music".to_string(), file));
            Some(crate::voice::Bed {
                samples,
                under: project.music.under(),
            })
        }
    };
    let sources = release::sources(project, dir, cut, &sounds).map_err(Stop::Failed)?;
    let tools = setup.tools().map_err(Stop::Failed)?;
    let inputs = release::inputs(
        voice,
        &sources,
        &tools,
        &sha256_hex(timeline_json.as_bytes()),
    );
    let mut plan = Plan {
        project,
        cut,
        voice,
        layout,
        clips: clips.into_iter().map(|(_, path)| path).collect(),
        timeline,
        timeline_json,
        shots,
        pictures,
        faces: setup.faces.clone(),
        look: Look::of(&setup.theme),
        chapters,
        room,
        music,
        // A cut is as loud as the quietest of its platforms plays it; a
        // cut for none is as loud as the reference's were.
        loudness: vidni_core::platforms::loudness(&cut.platforms())
            .unwrap_or(Target::default().integrated),
        tools,
        sources,
        inputs,
        emphasis,
        diagnostic,
    };
    if !plan.emphasis.is_empty() {
        let composer = plan.plain_composer(true).map_err(Stop::Failed)?;
        crate::cues::resolve_visibility(&mut plan.emphasis, &composer).map_err(Stop::Failed)?;
    }
    Ok(plan)
}

impl Plan<'_> {
    /// Review content independent of whether a person has already verified
    /// some of its cues. Approval bookkeeping must not change this identity.
    pub(crate) fn review_identity(&self) -> String {
        let sources = Sources {
            look: vidni_core::cues::hash(&release::look(self.project, self.cut)),
            ..self.sources.clone()
        };
        let inputs = release::inputs(
            self.voice,
            &sources,
            &self.tools,
            &sha256_hex(self.timeline_json.as_bytes()),
        );
        let cues: Vec<_> = self
            .emphasis
            .iter()
            .map(|c| (&c.key, &c.binding, &c.interval, &c.display))
            .collect();
        vidni_core::cues::hash(&serde_json::json!(["review-content-1", inputs, cues]))
    }

    /// The cut's timeline.
    pub fn timeline(&self) -> &Timeline {
        &self.timeline
    }

    /// Each line's clip, in script order.
    pub fn clips(&self) -> Vec<&Path> {
        self.clips.iter().map(PathBuf::as_path).collect()
    }

    /// A diagnostic uses the identical processed audio and timing as the cut.
    pub fn review_audio(&self, out: &Path) -> Result<(), String> {
        voice::narrate(
            &self.timeline,
            &self.clips(),
            self.room.as_deref(),
            self.music.as_ref(),
            self.loudness,
            out,
        )
    }

    /// Processed-audio identity, independent of visual cue verification.
    pub fn review_audio_key(&self) -> String {
        vidni_core::cues::hash(&serde_json::json!([
            "review-audio-1",
            self.sources.audio,
            self.timeline,
            self.room,
            self.music.as_ref().map(|m| (&m.samples, m.under)),
            self.loudness
        ]))
    }

    /// The compositor of the cut's picture, with its captions drawn in or
    /// left out.
    pub fn composer(&self, captions: bool) -> Result<Composer, String> {
        self.plain_composer(captions)?.with_emphasis(
            self.emphasis
                .iter()
                .flat_map(|c| c.events().iter().cloned())
                .collect(),
        )
    }

    fn plain_composer(&self, captions: bool) -> Result<Composer, String> {
        let mut fonts = Fonts::new();
        let faces = self.faces.load(&mut fonts)?;
        let subtitle = self.project.meta.subtitle.clone().unwrap_or_default();
        let composer = Composer::new(
            self.layout,
            &self.timeline,
            self.shots.clone(),
            &mut fonts,
            &faces,
            &self.look,
            &subtitle,
        )?;
        Ok(match captions {
            true => composer,
            false => composer.without_captions(),
        })
    }

    /// What the cut draws where one of its platforms' apps would cover it,
    /// each as a problem that names its line.
    fn covered(&self, composer: &mut Composer) -> Result<Vec<String>, String> {
        let ids = self.cut.line_ids(self.project);
        let spills = composer.spills()?;
        Ok(spills
            .iter()
            .map(|s| {
                format!(
                    "line {}: its {} reaches {} px into the {} edge that the cut's platforms cover",
                    ids[s.line], s.what, s.by, s.edge
                )
            })
            .collect())
    }

    /// Whether the captions drawn into the picture stand out from what is
    /// behind them: the panel in a 16:9 cut, over the lightest and the
    /// darkest picture, and the frame's colour under a 9:16 view.
    fn contrast(&self) -> Option<String> {
        let ink = self.look.ink;
        let least = match self.cut.aspect {
            Aspect::Landscape => [[255; 3], [0; 3]]
                .map(|behind| captions::contrast(ink, captions::over(self.look.panel, behind)))
                .into_iter()
                .fold(f64::INFINITY, f64::min),
            Aspect::Portrait => captions::contrast(ink, self.look.frame),
        };
        (least < captions::CONTRAST).then(|| {
            format!(
                "its captions stand out from what is behind them by {least:.1} to 1, and {} to 1 is the least that reads",
                captions::CONTRAST
            )
        })
    }

    /// The caption files' names: `<cut>.<language>.srt` and `.vtt`.
    fn caption_files(&self) -> [String; 2] {
        let stem = format!("{}.{}", self.cut.id, self.project.meta.language);
        [format!("{stem}.srt"), format!("{stem}.vtt")]
    }

    /// The names of the timeline's file, and of the narration's as it is
    /// assembled and as it is normalized.
    fn working_files(&self) -> [String; 3] {
        let cut = &self.cut.id;
        [
            format!("{cut}.timeline.json"),
            format!("{cut}.narration.wav"),
            format!("{cut}.narration-norm.wav"),
        ]
    }

    /// The chapter list's file, `<cut>.chapters.txt`, when the cut has one.
    fn chapters_file(&self) -> Option<String> {
        (!self.chapters.is_empty()).then(|| format!("{}.chapters.txt", self.cut.id))
    }

    /// The files a final build delivers: its pictures, its captions and
    /// its chapter list.
    fn deliverables(&self) -> Vec<String> {
        let pictures = self.pictures.iter().map(|(name, _)| name.clone());
        pictures
            .chain(self.caption_files())
            .chain(self.chapters_file())
            .collect()
    }
}

fn write(path: &Path, bytes: &[u8]) -> Result<String, String> {
    fs::write(path, bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(sha256_hex(bytes))
}

/// The record of an earlier build of exactly what `plan` describes, when
/// every file that build made is still as it made it. Then a build would
/// make the same files again, so there is nothing to build (TODO.md,
/// P2-VIDNI-09).
pub fn kept(plan: &Plan, dir: &Path) -> Option<Record> {
    let record = Record::read(dir, plan.voice, &plan.cut.id).ok()??;
    // The pictures are a part of the inputs; this is to be sure of it.
    let pictures: BTreeSet<&String> = plan.pictures.iter().map(|(name, _)| name).collect();
    if record.inputs != plan.inputs || !record.outputs.keys().eq(pictures) {
        return None;
    }
    let [timeline, narration, normalized] = plan.working_files();
    let [srt, vtt] = plan.caption_files();
    let t1 = &record.t1;
    // A cut with a chapter list has its file, and a cut without has none.
    if plan.chapters_file().is_some() != t1.chapters.is_some() {
        return None;
    }
    let mut files = vec![
        (timeline, &t1.timeline),
        (srt, &t1.srt),
        (vtt, &t1.vtt),
        (narration, &t1.pcm),
        (normalized, &t1.pcm_normalized),
    ];
    files.extend(plan.chapters_file().zip(t1.chapters.as_ref()));
    files.extend(
        record
            .outputs
            .iter()
            .map(|(name, o)| (name.clone(), &o.mp4)),
    );
    let folder = dir.join(plan.voice.dir());
    let whole = files
        .iter()
        .all(|(name, sha)| release::file_hash(&folder.join(name)).ok().as_ref() == Some(*sha));
    whole.then_some(record)
}

/// Builds what `plan` describes, unless an earlier build of the same
/// inputs is still whole and `again` doesn't ask for another.
fn make(
    plan: &Plan,
    dir: &Path,
    again: bool,
    progress: &mut dyn FnMut(Step),
) -> Result<Record, String> {
    if !again && let Some(record) = kept(plan, dir) {
        // The resolved project is the whole project's, and may have
        // changed where this cut hasn't.
        ratify::write_resolved(plan.project, dir)?;
        progress(Step::Kept);
        return Ok(record);
    }
    build(plan, dir, progress)
}

/// Builds what `plan` describes into its folder (`build/draft` or
/// `build/final`), and writes the build's record last, so that an
/// interrupted build never looks finished.
pub fn build(plan: &Plan, dir: &Path, progress: &mut dyn FnMut(Step)) -> Result<Record, String> {
    if plan.diagnostic {
        return Err("unverified cue previews cannot be exported as a draft or final build".into());
    }
    let cut = &plan.cut.id;
    let folder = dir.join(plan.voice.dir());
    fs::create_dir_all(&folder).map_err(|e| format!("{}: {e}", folder.display()))?;
    let at = |name: &str| folder.join(name);
    // A record left from an earlier build would describe files about to change.
    let record_path = Record::path(dir, plan.voice, cut);
    if record_path.is_file() {
        fs::remove_file(&record_path).map_err(|e| format!("{}: {e}", record_path.display()))?;
    }

    progress(Step::Stage("timeline and captions"));
    // The resolved project is the file the engine's inputs are read off;
    // it is the whole project's, so no cut's record holds its hash.
    ratify::write_resolved(plan.project, dir)?;
    let [srt, vtt] = plan.caption_files();
    let [timeline, narration, normalized] = plan.working_files();
    let srt_path = at(&srt);
    let mut t1 = T1 {
        timeline: write(&at(&timeline), plan.timeline_json.as_bytes())?,
        srt: write(&srt_path, timeline::srt(&plan.timeline).as_bytes())?,
        vtt: write(&at(&vtt), timeline::vtt(&plan.timeline).as_bytes())?,
        pcm: String::new(),
        pcm_normalized: String::new(),
        chapters: None,
    };
    // The chapter list, to paste into a description; and the same
    // chapters as ffmpeg reads them, to write into each picture.
    let metadata = at(&format!("{cut}.chapters.ffmeta"));
    if let Some(name) = plan.chapters_file() {
        t1.chapters = Some(write(
            &at(&name),
            chapters::text(&plan.chapters).as_bytes(),
        )?);
        let text = encode::chapter_metadata(&plan.chapters, plan.timeline.total);
        write(&metadata, text.as_bytes())?;
    }
    let metadata = t1.chapters.is_some().then_some(metadata);

    progress(Step::Stage("narration"));
    let narration_path = at(&narration);
    let normalized = at(&normalized);
    voice::narrate(
        &plan.timeline,
        &plan.clips(),
        plan.room.as_deref(),
        plan.music.as_ref(),
        plan.loudness,
        &narration_path,
    )?;
    encode::stereo(&narration_path, &normalized)?;
    t1.pcm = release::file_hash(&narration_path)?;
    t1.pcm_normalized = release::file_hash(&normalized)?;

    let (width, height) = plan.layout.size;
    let cues = plan.timeline.sentences.iter().map(|e| e.cues.len()).sum();
    let mut outputs = BTreeMap::new();
    for (name, captions_drawn) in &plan.pictures {
        progress(Step::Stage(name));
        let mut composer = plan.composer(*captions_drawn)?;
        // ffmpeg picks its container by the name's ending.
        let part = at(&format!("{name}.part.mp4"));
        let mp4 = at(name);
        let of = composer.frames();
        let mut encoder = Encoder::start_with(
            &Settings::new(width, height),
            &normalized,
            metadata.as_deref(),
            &part,
        )?;
        let rendered = compose::render(&mut composer, |f, bytes| {
            encoder.push(bytes)?;
            progress(Step::Frames {
                file: name,
                done: f + 1,
                of,
            });
            Ok(())
        });
        // When ffmpeg stops part of the way, what it said is the reason,
        // not the pipe it left broken.
        let (frames, frame_count) = match (rendered, encoder.finish()) {
            (Ok((frames, _)), Ok(count)) => (frames, count),
            (_, Err(ffmpeg)) => return Err(ffmpeg),
            (Err(e), Ok(_)) => return Err(e),
        };
        fs::rename(&part, &mp4).map_err(|e| format!("{}: {e}", mp4.display()))?;
        let report = qa::check(
            &mp4,
            &qa::Expect {
                width,
                height,
                fps: FPS as u32,
                duration: plan.timeline.total,
                loudness: plan.loudness,
                tolerance: LOUDNESS_TOLERANCE,
                true_peak: TRUE_PEAK,
                captions: Some((srt_path.clone(), cues)),
                chapters: plan.chapters.clone(),
            },
        )?;
        // What the picture draws where an app would cover it fails QA too,
        // and so do captions that can't be read.
        let mut problems = report.problems;
        problems.extend(plan.covered(&mut composer)?);
        problems.extend(captions::problems(&captions::of(&plan.timeline)));
        if *captions_drawn {
            problems.extend(plan.contrast());
        }
        outputs.insert(
            name.clone(),
            Output {
                captions: *captions_drawn,
                frames,
                frame_count,
                packets: sha256_hex(&crate::framemd5(&mp4)?),
                mp4: release::file_hash(&mp4)?,
                qa: report.summary,
                problems,
            },
        );
    }
    // Frames to look at, from the picture with captions when there is one.
    progress(Step::Stage("review frames"));
    let captioned = plan.pictures.iter().any(|(_, captions)| *captions);
    review::write(
        &mut plan.composer(captioned)?,
        &plan.timeline,
        &plan.cut.line_ids(plan.project),
        plan.cut,
        &plan.faces.mono,
        &review::folder(&folder, cut),
    )?;
    let record = Record {
        cut: cut.clone(),
        voice: plan.voice,
        inputs: plan.inputs.clone(),
        sources: plan.sources.clone(),
        t1,
        outputs,
        tools: plan.tools.clone(),
    };
    record.write(dir)?;
    Ok(record)
}

/// Drafts `cut`: builds it in the draft voice, into `build/draft/`. Its
/// lines must be spoken and its captures taken already. A draft of the
/// same inputs that is still whole is kept, unless `setup` says to build
/// again.
pub fn draft(
    project: &Project,
    dir: &Path,
    cut: &Cut,
    pack: &Pack,
    setup: &Setup,
    progress: &mut dyn FnMut(Step),
) -> Result<Record, Stop> {
    let plan = plan(project, dir, cut, Voice::Draft, Some(pack), setup)?;
    make(&plan, dir, setup.again, progress).map_err(Stop::Failed)
}

/// The final build of `cut`, from the creator's takes, into `build/final/`
/// and, when QA passes, `out/`. It is refused unless the cut is ratified
/// and unchanged and every line has a current take. A final build of the
/// same inputs that is still whole is kept, unless `setup` says to build
/// again; it is delivered all the same.
pub fn final_build(
    project: &Project,
    dir: &Path,
    lock: &ratify::Lock,
    cut: &Cut,
    setup: &Setup,
    progress: &mut dyn FnMut(Step),
) -> Result<Record, Stop> {
    let refusals = release::refusals(project, dir, lock, cut);
    if !refusals.is_empty() {
        return Err(Stop::Refused(refusals));
    }
    let plan = plan(project, dir, cut, Voice::Takes, None, setup)?;
    let record = make(&plan, dir, setup.again, progress).map_err(Stop::Failed)?;
    let problems = record.problems();
    if !problems.is_empty() {
        let mut refusals = vec![format!(
            "cut {} failed QA, so it isn't delivered; its files are in {}",
            cut.id,
            Voice::Takes.dir()
        )];
        refusals.extend(problems);
        return Err(Stop::Refused(refusals));
    }
    deliver(&plan, dir).map_err(Stop::Failed)?;
    Ok(record)
}

/// Writes what goes with `cut` where it is uploaded: for each platform it
/// is made for, the folder `out/<platform>/<cut>/` with the video, its
/// captions where the platform takes them, its images and `metadata.md`
/// (TODO.md, P6-VIDNI-04). The cut's final build must be delivered. The
/// folders are made afresh from it and from `[publish]`, and nothing is
/// rendered again but the images.
///
/// Each platform's folder is written only when the platform would take
/// the cut and play it right (TODO.md, P6-VIDNI-05): its length, size,
/// shape, codecs, loudness and captions are held to the platform's
/// profile first, and a cut that fails isn't exported for that platform.
pub fn exports(project: &Project, dir: &Path, cut: &Cut, setup: &Setup) -> Result<Exports, Stop> {
    let plan = plan(project, dir, cut, Voice::Takes, None, setup)?;
    let record = Record::read(dir, Voice::Takes, &cut.id)
        .map_err(Stop::Failed)?
        .ok_or_else(|| Stop::refused(format!("cut {} has no final build to export", cut.id)))?;
    export(&plan, dir, &record).map_err(Stop::Failed)
}

/// What exporting a cut found.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Exports {
    /// The platforms the cut was made for when it was exported.
    #[serde(default)]
    pub platforms: Vec<String>,
    /// Each platform the cut wasn't exported for, with every reason.
    pub refused: Vec<String>,
    /// What an upload would be cut short for, or isn't at its best with.
    pub warnings: Vec<String>,
}

impl Exports {
    /// Where what the last export of `cut` found is kept: beside its
    /// final build's record, out of what is delivered.
    pub fn path(project_dir: &Path, cut: &str) -> PathBuf {
        project_dir
            .join(Voice::Takes.dir())
            .join(format!("{cut}.exports.json"))
    }

    /// What the last export of `cut` found; `None` when it was never
    /// exported.
    pub fn read(project_dir: &Path, cut: &str) -> Result<Option<Self>, String> {
        let path = Self::path(project_dir, cut);
        match fs::read_to_string(&path) {
            Ok(text) => serde_json::from_str(&text)
                .map(Some)
                .map_err(|e| format!("{}: {e}", path.display())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(format!("{}: {e}", path.display())),
        }
    }

    fn write(&self, project_dir: &Path, cut: &str) -> Result<(), String> {
        let path = Self::path(project_dir, cut);
        let text = serde_json::to_string_pretty(self).expect("JSON serializes") + "\n";
        fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))
    }
}

fn export(plan: &Plan, dir: &Path, record: &Record) -> Result<Exports, String> {
    use vidni_core::platforms::{self, CaptionStyle};
    use vidni_core::publish::{self, Upload};
    let out = dir.join(release::OUT);
    let (cut, project) = (plan.cut, plan.project);
    fn made(path: &Path) -> impl Fn(std::io::Error) -> String + '_ {
        move |e| format!("{}: {e}", path.display())
    }
    let profiles = platforms::all();
    let wanted = cut.platforms();
    // A folder from when the cut was for another platform is no longer its.
    for profile in profiles.iter().filter(|p| !wanted.contains(&p.id)) {
        let stale = out.join(&profile.id).join(&cut.id);
        if stale.is_dir() {
            fs::remove_dir_all(&stale).map_err(made(&stale))?;
            // Its platform's folder goes with it, once that is empty.
            let _ = fs::remove_dir(out.join(&profile.id));
        }
    }
    if wanted.is_empty() {
        let found = Exports::default();
        found.write(dir, &cut.id)?;
        return Ok(found);
    }
    let mut fonts = Fonts::new();
    let faces = plan.faces.load(&mut fonts)?;
    // The picture with captions drawn in, and the one without.
    let picture = |captions: bool| {
        let named = plan.pictures.iter().find(|(_, drawn)| *drawn == captions);
        named
            .or(plan.pictures.first())
            .map(|(name, _)| name.clone())
    };
    let ids = cut.line_ids(project);
    // The corrections to lines this cut says, each with the second its
    // line starts at here.
    let corrections: Vec<(u32, String, String)> = project
        .errata
        .iter()
        .filter_map(|erratum| {
            let line = ids.iter().position(|id| *id == erratum.line)?;
            let at = plan.timeline.sentences[line].start as u32;
            Some((at, erratum.on.clone(), erratum.text.clone()))
        })
        .collect();
    let (mut warnings, mut refused) = (Vec::new(), Vec::new());
    // Each picture is measured once, however many platforms take it.
    let mut measured: BTreeMap<String, vidni_core::platforms::Upload> = BTreeMap::new();
    for profile in profiles.iter().filter(|p| wanted.contains(&p.id)) {
        let folder = out.join(&profile.id).join(&cut.id);
        if folder.is_dir() {
            fs::remove_dir_all(&folder).map_err(made(&folder))?;
        }
        // The video: the clean master where the platform takes a caption
        // file, and the picture with its captions in where it doesn't.
        let takes_file = profile.captions == CaptionStyle::File;
        let video = picture(!takes_file).ok_or("a cut has a picture")?;
        // Would the platform take it, and play it right?
        if !measured.contains_key(&video) {
            let upload = qa::upload(&out.join(&video), cut.aspect)?;
            measured.insert(video.clone(), upload);
        }
        let upload = &measured[&video];
        let mut reasons = profile.refusals(upload);
        if takes_file {
            let [srt, _] = plan.caption_files();
            let text = fs::read_to_string(out.join(&srt)).map_err(made(&out.join(&srt)))?;
            reasons.extend(qa::captions(&text, upload.duration));
        }
        if !reasons.is_empty() {
            let _ = fs::remove_dir(out.join(&profile.id));
            refused.push(format!(
                "cut {} isn't exported for {}: {}",
                cut.id,
                profile.id,
                reasons.join("; ")
            ));
            continue;
        }
        warnings.extend(
            profile
                .advice(upload)
                .into_iter()
                .map(|advice| format!("cut {} for {}: {advice}", cut.id, profile.id)),
        );
        fs::create_dir_all(&folder).map_err(made(&folder))?;
        let mut copied = vec![video.clone()];
        let captions: Vec<String> = match takes_file {
            true => plan.caption_files().to_vec(),
            false => Vec::new(),
        };
        copied.extend(captions.iter().cloned());
        for name in &copied {
            let (from, to) = (out.join(name), folder.join(name));
            // The same file under a second name, where the disk allows it.
            if fs::hard_link(&from, &to).is_err() {
                fs::copy(&from, &to).map_err(made(&to))?;
            }
        }
        let mut images = Vec::new();
        if profile.id == "youtube" && cut.aspect == Aspect::Landscape {
            // Each thumbnail the project asks for that is of a line of
            // this cut; or the cut's first line, under the title.
            let mut wanted: Vec<(usize, Option<String>)> = project
                .publish
                .thumbnails
                .iter()
                .filter_map(|t| Some((ids.iter().position(|id| *id == t.line)?, t.title.clone())))
                .collect();
            if wanted.is_empty() {
                let title = publish::title(project, &profile.id).to_string();
                wanted.push((0, Some(title)));
            }
            let mut composer = plan.composer(false)?;
            for (n, (line, title)) in wanted.iter().enumerate() {
                let entry = &plan.timeline.sentences[*line];
                let frame = (((entry.start + entry.end) / 2.0) * FPS) as u64;
                let picture = crate::export::thumbnail(
                    &composer.frame(frame.min(composer.frames().saturating_sub(1)))?,
                    title.as_deref(),
                    &mut fonts,
                    &faces,
                    &plan.look,
                );
                let name = format!("thumbnail-{}.png", n + 1);
                picture.save_png(&folder.join(&name))?;
                let bytes = fs::metadata(folder.join(&name)).map_or(0, |m| m.len());
                if bytes > crate::export::THUMBNAIL_BYTES {
                    warnings.push(format!(
                        "{}/{}/{name} is {:.1} MB, and YouTube takes 2 MB from a phone",
                        profile.id,
                        cut.id,
                        bytes as f64 / 1e6
                    ));
                }
                images.push(name);
            }
        }
        if profile.id == "instagram-reels" {
            let cover = crate::export::cover(
                publish::title(project, &profile.id),
                project.meta.subtitle.as_deref().unwrap_or_default(),
                &mut fonts,
                &faces,
                &plan.look,
            );
            cover.save_png(&folder.join("cover.png"))?;
            images.push("cover.png".to_string());
        }
        let upload = Upload {
            project,
            cut,
            platform: profile,
            chapters: &plan.chapters,
            synthetic: record.voice == Voice::Draft,
            corrections: &corrections,
            video: &video,
            captions: &captions,
            images: &images,
        };
        let metadata = folder.join("metadata.md");
        fs::write(&metadata, upload.metadata()).map_err(made(&metadata))?;
        // The same words for `vidni upload`, beside the build's record
        // and out of what is delivered.
        let words = json!({
            "title": publish::title(project, &profile.id),
            "description": upload.description(),
            "tags": project.publish.tags,
            "video": video,
        });
        let facts = dir
            .join(Voice::Takes.dir())
            .join(format!("{}.upload.{}.json", cut.id, profile.id));
        let text = serde_json::to_string_pretty(&words).expect("JSON serializes") + "\n";
        fs::write(&facts, text).map_err(made(&facts))?;
        warnings.extend(
            upload
                .problems(plan.timeline.total)
                .into_iter()
                .map(|problem| format!("cut {}: {problem}", cut.id)),
        );
    }
    // Each platform's UPLOAD.md: every cut of the project made for it,
    // with its release and where it is published.
    let releases = release::read(dir)?;
    let published = vidni_core::published::read(dir)?;
    for profile in &profiles {
        let cuts: Vec<&Cut> = project
            .cuts
            .iter()
            .filter(|c| c.platforms().contains(&profile.id))
            .collect();
        let folder = out.join(&profile.id);
        if !folder.is_dir() {
            continue;
        }
        // A platform's folder with no cut's folder left in it goes.
        let holds_a_cut = fs::read_dir(&folder)
            .map_err(made(&folder))?
            .flatten()
            .any(|entry| entry.path().is_dir());
        if cuts.is_empty() || !holds_a_cut {
            let _ = fs::remove_file(folder.join("UPLOAD.md"));
            let _ = fs::remove_dir(&folder);
            continue;
        }
        let listed: Vec<publish::Listed> = cuts
            .iter()
            .map(|c| publish::Listed {
                cut: &c.id,
                exported: folder.join(&c.id).is_dir(),
                release: releases.latest(&c.id).map(|r| r.tag.as_str()),
                published: published
                    .of(&c.id)
                    .filter(|u| u.platform == profile.id)
                    .map(|u| (u.url.as_str(), u.release.as_str()))
                    .collect(),
            })
            .collect();
        let file = folder.join("UPLOAD.md");
        fs::write(&file, publish::upload_md(profile, &listed)).map_err(made(&file))?;
    }
    let found = Exports {
        platforms: wanted,
        refused,
        warnings,
    };
    found.write(dir, &cut.id)?;
    Ok(found)
}

/// Copies a final build's pictures and captions to `out/`. A file that
/// is there already, byte for byte, is left as it is.
fn deliver(plan: &Plan, dir: &Path) -> Result<(), String> {
    let (from, to) = (dir.join(Voice::Takes.dir()), dir.join(release::OUT));
    fs::create_dir_all(&to).map_err(|e| format!("{}: {e}", to.display()))?;
    let delivered = plan.deliverables();
    for name in &delivered {
        let built = release::file_hash(&from.join(name))?;
        if release::file_hash(&to.join(name)).ok().as_ref() == Some(&built) {
            continue;
        }
        fs::copy(from.join(name), to.join(name))
            .map_err(|e| format!("{}: {e}", to.join(name).display()))?;
    }
    // A captioned variant or a chapter list from when the cut had one is
    // no longer the cut's.
    for ending in ["captioned.mp4", "chapters.txt"] {
        let name = format!("{}.{ending}", plan.cut.id);
        let stale = to.join(&name);
        if !delivered.contains(&name) && stale.is_file() {
            fs::remove_file(&stale).map_err(|e| format!("{}: {e}", stale.display()))?;
        }
    }
    Ok(())
}

/// Approves `cut` on `day`: its final build becomes a release in
/// `release.json`. It is refused unless a final build would be allowed
/// now, the build on disk is of the cut as it is now and passed QA, and
/// the files in `out/` are the ones it delivered.
pub fn approve(
    project: &Project,
    dir: &Path,
    lock: &ratify::Lock,
    cut: &Cut,
    setup: &Setup,
    day: &str,
) -> Result<Approval, Stop> {
    let refusals = release::refusals(project, dir, lock, cut);
    if !refusals.is_empty() {
        return Err(Stop::Refused(refusals));
    }
    let plan = plan(project, dir, cut, Voice::Takes, None, setup)?;
    let again = format!("`vidni final --cut {}` builds it", cut.id);
    let record = Record::read(dir, Voice::Takes, &cut.id)
        .map_err(Stop::Failed)?
        .ok_or_else(|| Stop::refused(format!("cut {} has no final build; {again}", cut.id)))?;
    if record.inputs != plan.inputs {
        let (was, now) = (&record.sources, &plan.sources);
        let mut changed: Vec<&str> = [
            ("what it shows and says", was.content != now.content),
            ("its takes", was.audio != now.audio),
            ("its captures", was.captures != now.captures),
            ("its gaps, look or settings", was.look != now.look),
            ("vidni, ffmpeg or the fonts", record.tools != plan.tools),
        ]
        .into_iter()
        .filter_map(|(what, differs)| differs.then_some(what))
        .collect();
        // The one input left: what a line's words are read as can change
        // the timeline without changing a take.
        if changed.is_empty() {
            changed.push("its timeline");
        }
        return Err(Stop::refused(format!(
            "cut {} has changed since its final build ({}); {again} again",
            cut.id,
            changed.join(", ")
        )));
    }
    if !record.problems().is_empty() {
        return Err(Stop::refused(format!(
            "cut {}'s final build failed QA; {again} again",
            cut.id
        )));
    }
    let [srt, vtt] = plan.caption_files();
    let mut delivered: Vec<(&str, &str)> = vec![(&srt, &record.t1.srt), (&vtt, &record.t1.vtt)];
    let chapters = plan.chapters_file();
    if let (Some(name), Some(sha)) = (&chapters, &record.t1.chapters) {
        delivered.push((name, sha));
    }
    delivered.extend(
        record
            .outputs
            .iter()
            .map(|(name, o)| (name.as_str(), o.mp4.as_str())),
    );
    for (name, sha) in delivered {
        let file = dir.join(release::OUT).join(name);
        if release::file_hash(&file).ok().as_deref() != Some(sha) {
            return Err(Stop::refused(format!(
                "{} isn't the file cut {}'s final build delivered; {again} again",
                Path::new(release::OUT).join(name).display(),
                cut.id
            )));
        }
    }
    let mut releases = release::read(dir).map_err(Stop::Failed)?;
    let approval = release::approve(&mut releases, &record, day);
    if matches!(approval, Approval::Approved(_)) {
        release::write(dir, &releases).map_err(Stop::Failed)?;
    }
    Ok(approval)
}

/// Runs ffmpeg and ffprobe once, so that a build fails before it starts
/// when either is missing.
pub fn check_tools() -> Result<(), String> {
    for tool in ["ffmpeg", "ffprobe"] {
        run(Command::new(tool).arg("-version"))
            .map_err(|e| format!("{e}; vidni builds with the {tool} on the path"))?;
    }
    Ok(())
}

/// A build of the project in `dir`, for people, a line for each file:
/// where it is, its frames, and what QA measured.
pub fn summary(record: &Record, dir: &Path) -> Vec<String> {
    let folder = dir.join(record.voice.dir());
    record
        .outputs
        .iter()
        .map(|(name, o)| {
            format!(
                "{}: {} frames; {}",
                folder.join(name).display(),
                o.frame_count,
                o.qa
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A web scene with a wide camera whose 9:16 stand-in is a narrow one,
    /// and four lines framed in the four ways a 9:16 cut chooses between.
    const PROJECT: &str = r#"schema = 1

[meta]
title = "Framing"
default_scene = "page"

[[cuts]]
id = "long"
aspect = "16:9"

[[cuts]]
id = "short"
aspect = "9:16"

[sections.main]

[[lines]]
id = "plain"
section = "main"
show = "The scene's default camera."

[[lines]]
id = "wide"
section = "main"
show = "A camera that names its 9:16 stand-in."
camera = "table"

[[lines]]
id = "boxed"
section = "main"
show = "Highlights narrow enough to follow."
camera = "table"
highlight = ["cell"]

[[lines]]
id = "named"
section = "main"
show = "A 9:16 camera of the line's own."
camera = "table"
camera_vertical = "column"
highlight = ["cell"]

[scenes.page]
kind = "web"
url = "https://example.com"
default_camera = "table"
vertical_follows_highlights = 400

[scenes.page.targets]
grid = { css = "table" }
cell = { css = "td" }
side = { css = "aside" }

[scenes.page.cameras]
table = { union = ["grid"], vertical = "column" }
column = { union = ["side"] }
"#;

    fn project() -> Project {
        vidni_core::validate::check(PROJECT, None).unwrap_or_else(|p| panic!("{p:?}"))
    }

    #[test]
    fn a_cut_chooses_each_lines_camera_in_the_specs_order() {
        let p = project();
        let scene = &p.scenes["page"];
        let name = |line: usize, portrait: bool, highlights: Option<f64>| {
            camera_name(&p.lines[line], scene, portrait, highlights)
        };
        // 16:9: the line's camera, or else the scene's default.
        assert_eq!(name(0, false, None), "table");
        assert_eq!(name(3, false, Some(300.0)), "table");
        // 9:16, in order: the line's own vertical camera;
        assert_eq!(name(3, true, Some(300.0)), "column");
        // its highlights, when they are narrow enough;
        assert_eq!(name(2, true, Some(400.0)), "highlights");
        // else the vertical stand-in of its camera;
        assert_eq!(name(2, true, Some(401.0)), "column");
        assert_eq!(name(1, true, None), "column");
        // else the camera itself, and the whole viewport when nothing names one.
        let mut bare = scene.clone();
        bare.cameras.get_mut("table").unwrap().vertical = None;
        assert_eq!(camera_name(&p.lines[1], &bare, true, None), "table");
        bare.default_camera = None;
        assert_eq!(camera_name(&p.lines[0], &bare, true, None), "viewport");
    }

    /// Writes each web line's capture into `dir`: a still's file, and its
    /// frames with `targets` as the boxes measured. A line's highlights
    /// are measured too, as in a real capture.
    fn capture_lines(dir: &Path, p: &Project, targets: &[(&str, &[[f64; 4]])]) {
        let keys = capture::line_keys(p);
        let boxes = |name: &str| -> Vec<[f64; 4]> {
            targets
                .iter()
                .find(|(target, _)| *target == name)
                .map(|(_, boxes)| boxes.to_vec())
                .unwrap_or_default()
        };
        for line in &p.lines {
            let key = &keys[&line.id];
            let still = capture::path(dir, key);
            fs::create_dir_all(still.parent().unwrap()).unwrap();
            fs::write(&still, b"a still").unwrap();
            let frames = Frames {
                viewport: [1280, 720],
                scale: 3,
                targets: targets
                    .iter()
                    .map(|(name, boxes)| (name.to_string(), boxes.to_vec()))
                    .collect(),
                highlights: line
                    .highlight
                    .iter()
                    .map(|name| (name.clone(), boxes(name)))
                    .collect(),
                browser: "a stand-in".into(),
                rows: None,
            };
            let json = serde_json::to_string(&frames).unwrap();
            fs::write(capture::frames_path(dir, key), json).unwrap();
        }
    }

    /// The camera rectangle of each of a cut's lines.
    fn framing(p: &Project, dir: &Path, cut: usize) -> Vec<[f64; 4]> {
        let lines = p.cuts[cut].lines_in(p);
        shots(p, dir, &p.cuts[cut], &lines)
            .unwrap()
            .into_iter()
            .map(|shot| match shot.picture {
                Picture::Still {
                    camera,
                    viewport,
                    scale,
                    ..
                } => {
                    assert_eq!((viewport, scale), ([1280.0, 720.0], 3.0));
                    camera
                }
                Picture::Card(card) => panic!("{} is no still", card.display()),
                Picture::Drawn(card) => panic!("{card:?} is no still"),
            })
            .collect()
    }

    const TABLE: [f64; 4] = [100.0, 80.0, 900.0, 500.0];
    const COLUMN: [f64; 4] = [1000.0, 80.0, 240.0, 600.0];
    const CELL: [f64; 4] = [300.0, 200.0, 180.0, 40.0];

    #[test]
    fn a_shot_is_framed_by_its_cameras_targets_in_the_capture() {
        let dir = tempfile::tempdir().unwrap();
        let dir = dir.path();
        let p = project();
        capture_lines(
            dir,
            &p,
            &[("grid", &[TABLE]), ("side", &[COLUMN]), ("cell", &[CELL])],
        );
        assert_eq!(framing(&p, dir, 0), [TABLE, TABLE, TABLE, TABLE]);
        assert_eq!(framing(&p, dir, 1), [COLUMN, COLUMN, CELL, COLUMN]);
    }

    #[test]
    fn a_camera_edited_after_the_capture_frames_what_it_names_now() {
        let dir = tempfile::tempdir().unwrap();
        let dir = dir.path();
        let p = project();
        capture_lines(
            dir,
            &p,
            &[("grid", &[TABLE]), ("side", &[COLUMN]), ("cell", &[CELL])],
        );
        // The 9:16 stand-in now frames the cell, and a new camera frames
        // the table and the column together. Cameras aren't part of a
        // capture's key, so the stills and their frames are the same files.
        let text = PROJECT
            .replace(
                "column = { union = [\"side\"] }",
                "column = { union = [\"cell\"] }\nboth = { union = [\"grid\", \"side\"] }",
            )
            .replace(
                "show = \"The scene's default camera.\"",
                "show = \"The scene's default camera.\"\ncamera = \"both\"",
            );
        let edited = vidni_core::validate::check(&text, None).unwrap_or_else(|p| panic!("{p:?}"));
        assert_eq!(capture::line_keys(&edited), capture::line_keys(&p));
        let both = [100.0, 80.0, 1140.0, 600.0];
        assert_eq!(framing(&edited, dir, 0), [both, TABLE, TABLE, TABLE]);
        assert_eq!(framing(&edited, dir, 1), [both, CELL, CELL, CELL]);
    }

    #[test]
    fn a_camera_whose_targets_dont_show_stops_the_build_by_name() {
        let dir = tempfile::tempdir().unwrap();
        let dir = dir.path();
        let p = project();
        // The column's target matched nothing on the page.
        capture_lines(
            dir,
            &p,
            &[("grid", &[TABLE]), ("side", &[]), ("cell", &[CELL])],
        );
        let lines = p.cuts[1].lines_in(&p);
        assert_eq!(
            shots(&p, dir, &p.cuts[1], &lines).unwrap_err(),
            "line plain: nothing that its camera column names shows in its capture"
        );
        // The 16:9 cut doesn't use it.
        assert_eq!(framing(&p, dir, 0).len(), 4);
    }
}
