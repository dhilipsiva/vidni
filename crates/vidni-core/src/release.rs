//! Builds and releases: the record of what a build of a cut was made from
//! and what it made, the gates a final build must pass, and `release.json`,
//! where each approved cut is written down.
//!
//! A build's record names its inputs and outputs by hash, in the tiers of
//! `docs/determinism.md`: T1 for what vidni computes (the timeline, the
//! captions, the frames and the PCM), T2 for the encoded file. The resolved
//! project is T1 too, but it is the whole project's, not one cut's, so it
//! isn't in a cut's record: an edit to another cut must not touch it. `vidni repro` builds twice and compares two records, and
//! `vidni approve` copies the final build's record into `release.json`.
//!
//! The gates are here, in the core, so that the command line and the app
//! refuse the same things in the same words (TODO.md, rule 4).

use crate::capture::{self, canonical, sha256_hex};
use crate::project::{Cut, Project};
use crate::ratify::{self, Status};
use crate::takes;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// The releases, at the project folder's top (committed with the project).
pub const FILE: &str = "release.json";
/// Where the final cuts are delivered, relative to the project folder.
pub const OUT: &str = "out";

/// The voice a build speaks with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Voice {
    /// The synthetic draft voice, for a draft.
    Draft,
    /// The creator's takes, for the final build.
    Takes,
}

impl Voice {
    /// Where builds with this voice are made, relative to the project folder.
    pub fn dir(self) -> &'static str {
        match self {
            Voice::Draft => "build/draft",
            Voice::Takes => "build/final",
        }
    }
}

/// What a cut is made from, each part as one hash. Working them out needs
/// no tool, so a cut's standing can be told without rendering anything.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sources {
    /// What the cut shows and says: the hash that ratifying it freezes.
    pub content: String,
    /// Each line's clip, by the SHA-256 of its file.
    pub audio: String,
    /// The gaps, the look, the title and the cut's own settings: what only
    /// a release hashes.
    pub look: String,
    /// Each capture's picture and frames, by the SHA-256 of their files.
    pub captures: String,
}

/// The T1 outputs every picture of a cut shares.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct T1 {
    pub timeline: String,
    pub srt: String,
    pub vtt: String,
    /// The narration track as vidni assembles it.
    pub pcm: String,
    /// The narration track after loudness normalization, as it is encoded.
    pub pcm_normalized: String,
    /// The chapter list, when the cut has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chapters: Option<String>,
}

/// One encoded file of a cut.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Output {
    /// Whether captions are drawn into the picture.
    pub captions: bool,
    /// T1: the rgb24 frames before encoding, every frame in order.
    pub frames: String,
    pub frame_count: u64,
    /// T2: the packets (`ffmpeg -map 0 -c copy -f framemd5`).
    pub packets: String,
    /// T2: the file.
    pub mp4: String,
    /// What QA measured.
    pub qa: String,
    /// What QA found wrong; a final build with any is refused.
    pub problems: Vec<String>,
}

/// The tools a build ran with.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tools {
    pub vidni: String,
    pub ffmpeg: String,
    /// Each font file's SHA-256, by file name.
    pub fonts: BTreeMap<String, String>,
    /// The settings the frames and the file were made with.
    pub engine: Value,
}

/// What a build of one cut was made from and what it made, written beside
/// its outputs as `<cut>.build.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub cut: String,
    pub voice: Voice,
    /// Everything the build depends on, as one hash: its voice, its
    /// sources, its tools and its timeline.
    pub inputs: String,
    pub sources: Sources,
    pub t1: T1,
    /// Each encoded file, by name.
    pub outputs: BTreeMap<String, Output>,
    pub tools: Tools,
}

impl Record {
    /// Where the record of `cut`'s build with `voice` is.
    pub fn path(project_dir: &Path, voice: Voice, cut: &str) -> PathBuf {
        project_dir
            .join(voice.dir())
            .join(format!("{cut}.build.json"))
    }

    /// Reads a cut's record; `None` when the cut hasn't been built.
    pub fn read(project_dir: &Path, voice: Voice, cut: &str) -> Result<Option<Record>, String> {
        let path = Self::path(project_dir, voice, cut);
        match fs::read_to_string(&path) {
            Ok(text) => serde_json::from_str(&text)
                .map(Some)
                .map_err(|e| format!("{}: {e}", path.display())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(format!("{}: {e}", path.display())),
        }
    }

    /// Writes the record beside the build's outputs.
    pub fn write(&self, project_dir: &Path) -> Result<PathBuf, String> {
        let path = Self::path(project_dir, self.voice, &self.cut);
        write_json(&path, self)?;
        Ok(path)
    }

    /// Everything QA found wrong, each with its file.
    pub fn problems(&self) -> Vec<String> {
        self.outputs
            .iter()
            .flat_map(|(name, o)| o.problems.iter().map(move |p| format!("{name}: {p}")))
            .collect()
    }

    /// Each hash that differs between this build and `other`, named as
    /// `t1 timeline` or `t2 long.mp4 packets`. Two builds of the same
    /// inputs must differ in none.
    pub fn differences(&self, other: &Record) -> Vec<String> {
        let mut found = Vec::new();
        // Two builds of one project should have been given the same things.
        if self.inputs != other.inputs {
            found.push("inputs".to_string());
        }
        let (a, b) = (&self.t1, &other.t1);
        for (name, mine, theirs) in [
            ("timeline", &a.timeline, &b.timeline),
            ("srt", &a.srt, &b.srt),
            ("vtt", &a.vtt, &b.vtt),
            ("pcm", &a.pcm, &b.pcm),
            ("pcm_normalized", &a.pcm_normalized, &b.pcm_normalized),
        ] {
            if mine != theirs {
                found.push(format!("t1 {name}"));
            }
        }
        if a.chapters != b.chapters {
            found.push("t1 chapters".to_string());
        }
        let names: std::collections::BTreeSet<&String> =
            self.outputs.keys().chain(other.outputs.keys()).collect();
        for name in names {
            match (self.outputs.get(name), other.outputs.get(name)) {
                (Some(mine), Some(theirs)) => {
                    if (&mine.frames, mine.frame_count) != (&theirs.frames, theirs.frame_count) {
                        found.push(format!("t1 {name} frames"));
                    }
                    if mine.packets != theirs.packets {
                        found.push(format!("t2 {name} packets"));
                    }
                    if mine.mp4 != theirs.mp4 {
                        found.push(format!("t2 {name} file"));
                    }
                }
                _ => found.push(format!("{name} is in one build only")),
            }
        }
        found
    }
}

/// One approved build of a cut. A release is never changed: approving a
/// cut again, after it has changed, adds the next one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Release {
    /// The release's name: the cut's ID and its number, such as `long-r1`.
    pub tag: String,
    /// The day it was approved, in UTC.
    pub approved: String,
    /// The final build it approves.
    pub build: Record,
}

/// The layout of `release.json` that this vidni reads and writes.
pub const SCHEMA: u32 = 1;

/// What `release.json` holds: each cut's releases, oldest first.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Releases {
    /// The file's layout. Releases are kept for good, so a later vidni
    /// must be able to tell an older file from its own.
    pub schema: u32,
    #[serde(default)]
    pub cuts: BTreeMap<String, Vec<Release>>,
}

impl Default for Releases {
    fn default() -> Self {
        Self {
            schema: SCHEMA,
            cuts: BTreeMap::new(),
        }
    }
}

impl Releases {
    /// The cut's newest release.
    pub fn latest(&self, cut: &str) -> Option<&Release> {
        self.cuts.get(cut).and_then(|releases| releases.last())
    }
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let text = serde_json::to_string_pretty(value).expect("JSON serializes") + "\n";
    let folder = path.parent().expect("a parent folder");
    fs::create_dir_all(folder)
        .and_then(|_| fs::write(path, text))
        .map_err(|e| format!("{}: {e}", path.display()))
}

/// Reads `release.json`; a project without one has approved nothing.
pub fn read(project_dir: &Path) -> Result<Releases, String> {
    let path = project_dir.join(FILE);
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Releases::default()),
        Err(e) => return Err(format!("{}: {e}", path.display())),
    };
    // The schema is looked at first, so that a file in another layout is
    // named as that, not as a file with a field out of place.
    let shown = |e: serde_json::Error| format!("{}: {e}", path.display());
    let value: Value = serde_json::from_str(&text).map_err(shown)?;
    match value["schema"].as_u64() {
        Some(schema) if schema == u64::from(SCHEMA) => serde_json::from_value(value).map_err(shown),
        Some(schema) => Err(format!(
            "{}: it is in release schema {schema}, and this vidni reads schema {SCHEMA}",
            path.display()
        )),
        None => Err(format!("{}: it names no schema", path.display())),
    }
}

/// Writes `release.json`.
pub fn write(project_dir: &Path, releases: &Releases) -> Result<(), String> {
    write_json(&project_dir.join(FILE), releases)
}

/// The SHA-256 of a file.
pub fn file_hash(path: &Path) -> Result<String, String> {
    fs::read(path)
        .map(|bytes| sha256_hex(&bytes))
        .map_err(|e| format!("{}: {e}", path.display()))
}

fn hash(value: &Value) -> String {
    sha256_hex(canonical(value).as_bytes())
}

/// What only a release hashes of `cut`: the gaps, the look, the title and
/// the cut's own settings. The voice's settings aren't here: what they
/// change is the audio, which is hashed as it is.
pub fn look(project: &Project, cut: &Cut) -> Value {
    let mut look = json!({
        "timing": project.timing,
        "style": project.style,
        "music": project.music,
        "meta": {
            "title": project.meta.title,
            "subtitle": project.meta.subtitle,
            "language": project.meta.language,
        },
        "cut": {
            "id": cut.id,
            "aspect": cut.aspect,
            "lines": cut.line_ids(project),
            "captions": cut.captions(),
            "platforms": cut.platforms(),
            "size": cut.size(),
        },
    });
    // The edges the cut's apps cover move what it draws, and they are
    // vidni's data, not the project's. A cut whose apps cover nothing has
    // the look it had before vidni knew of them.
    if let Some(covered) = cut.covered() {
        look["cut"]["covered"] = json!(covered);
    }
    if let Some(hold) = cut.end_hold {
        look["cut"]["end_hold"] = json!(hold);
    }
    // A cut for a platform that shows chapters is named by its sections'
    // chapter titles too; another cut's look doesn't know of them.
    if crate::chapters::wanted(cut) {
        look["chapters"] = json!(crate::chapters::titles(project));
    }
    look
}

/// The sources of `cut`, with `clips` as its lines' audio: each line's ID
/// and clip, in script order.
pub fn sources(
    project: &Project,
    project_dir: &Path,
    cut: &Cut,
    clips: &[(String, PathBuf)],
) -> Result<Sources, String> {
    // A clip is its line, its place in the project and its bytes: the
    // timeline names the place, so a clip moved is another input.
    let audio = clips
        .iter()
        .map(|(id, clip)| file_hash(clip).map(|sha| json!([id, relative(project_dir, clip), sha])))
        .collect::<Result<Vec<Value>, String>>()?;
    let captures = capture::for_cut(project, cut)
        .iter()
        .map(|c| {
            let picture = file_hash(&capture::path(project_dir, &c.key))?;
            let frames = match c.source {
                capture::Source::Line(_) => {
                    Some(file_hash(&capture::frames_path(project_dir, &c.key))?)
                }
                capture::Source::Page { .. } => None,
            };
            Ok(json!([c.key, picture, frames]))
        })
        .collect::<Result<Vec<Value>, String>>()?;
    Ok(Sources {
        content: ratify::content(project, cut).content,
        audio: hash(&Value::Array(audio)),
        look: hash(&look(project, cut)),
        captures: hash(&Value::Array(captures)),
    })
}

/// `path` as the project names it: from its folder, with `/` between parts.
pub fn relative(project_dir: &Path, path: &Path) -> String {
    let inside = path.strip_prefix(project_dir).unwrap_or(path);
    let parts: Vec<String> = inside
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    parts.join("/")
}

/// Everything a build depends on, as one hash. `timeline` is the hash of
/// the timeline the build starts from: it is worked out from the sources,
/// and being in here it can't differ between two builds of the same inputs.
pub fn inputs(voice: Voice, sources: &Sources, tools: &Tools, timeline: &str) -> String {
    hash(&json!({"voice": voice, "sources": sources, "tools": tools, "timeline": timeline}))
}

/// Each line of `cut` with its current take, in script order; or the lines
/// that have none, as a refusal.
pub fn takes_of(
    project: &Project,
    project_dir: &Path,
    cut: &Cut,
) -> Result<Vec<(String, PathBuf)>, Vec<String>> {
    let ids = cut.line_ids(project);
    let (mut clips, mut missing, mut stale) = (Vec::new(), Vec::new(), Vec::new());
    for take in takes::status(project, project_dir)
        .into_iter()
        .filter(|t| ids.contains(&t.id))
    {
        match take.status {
            takes::Status::Current(path) => clips.push((take.id, path)),
            takes::Status::Stale(_) => stale.push(take.id),
            takes::Status::Missing => missing.push(take.id),
        }
    }
    let mut refusals = Vec::new();
    let listed = |ids: &[String]| {
        let shown: Vec<&str> = ids.iter().take(5).map(String::as_str).collect();
        let more = ids.len().saturating_sub(shown.len());
        match more {
            0 => shown.join(", "),
            _ => format!("{} and {more} more", shown.join(", ")),
        }
    };
    if !missing.is_empty() {
        refusals.push(format!(
            "cut {}: {} of its {} lines {} no take: {}",
            cut.id,
            missing.len(),
            ids.len(),
            if missing.len() == 1 { "has" } else { "have" },
            listed(&missing)
        ));
    }
    if !stale.is_empty() {
        refusals.push(format!(
            "cut {}: {} of its {} lines {} only for other words: {}",
            cut.id,
            stale.len(),
            ids.len(),
            if stale.len() == 1 {
                "has a take"
            } else {
                "have takes"
            },
            listed(&stale)
        ));
    }
    if refusals.is_empty() {
        Ok(clips)
    } else {
        Err(refusals)
    }
}

/// Why `cut` may not have a final build: it isn't ratified, something it
/// froze has changed, or a line's take is missing or stale. Empty when it
/// may. (The third gate, QA, is checked on the build itself.)
pub fn refusals(
    project: &Project,
    project_dir: &Path,
    lock: &ratify::Lock,
    cut: &Cut,
) -> Vec<String> {
    let mut refusals = Vec::new();
    match ratify::status_of(project, project_dir, lock, cut) {
        Status::Ratified => {}
        Status::Draft => refusals.push(format!(
            "cut {} isn't ratified; `vidni ratify --cut {}` freezes it",
            cut.id, cut.id
        )),
        Status::Changed(reasons) => refusals.push(format!(
            "cut {} has changed since it was ratified: {}",
            cut.id,
            reasons.join("; ")
        )),
    }
    if let Err(takes) = takes_of(project, project_dir, cut) {
        refusals.extend(takes);
    }
    refusals
}

/// What approving a build did.
#[derive(Debug, Clone, PartialEq)]
pub enum Approval {
    /// A new release.
    Approved(Release),
    /// The cut's newest release is of this very build.
    Already(Release),
}

/// Approves `record`, a cut's final build, on `day`: adds its release,
/// unless the cut's newest release is of the same build.
pub fn approve(releases: &mut Releases, record: &Record, day: &str) -> Approval {
    let of_cut = releases.cuts.entry(record.cut.clone()).or_default();
    if let Some(latest) = of_cut.last().filter(|r| r.build == *record) {
        return Approval::Already(latest.clone());
    }
    let release = Release {
        tag: format!("{}-r{}", record.cut, of_cut.len() + 1),
        approved: day.to_string(),
        build: record.clone(),
    };
    of_cut.push(release.clone());
    Approval::Approved(release)
}

/// How a cut stands against its newest release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Standing {
    /// Never approved.
    Unreleased,
    /// The newest release is of the cut as it is now.
    Current(String),
    /// The cut has changed since its newest release, in these parts.
    Outdated(String, Vec<&'static str>),
}

/// How `cut` stands against its newest release: whether what it is made
/// from now is what was approved.
pub fn standing(project: &Project, project_dir: &Path, releases: &Releases, cut: &Cut) -> Standing {
    let Some(release) = releases.latest(&cut.id) else {
        return Standing::Unreleased;
    };
    let was = &release.build.sources;
    let now = takes_of(project, project_dir, cut)
        .ok()
        .and_then(|clips| sources(project, project_dir, cut, &clips).ok());
    let changed: Vec<&'static str> = match &now {
        // A take or a capture is gone or unreadable.
        None => vec!["its takes or captures"],
        Some(now) => [
            ("what it shows and says", now.content != was.content),
            ("its takes", now.audio != was.audio),
            ("its captures", now.captures != was.captures),
            ("its gaps, look or settings", now.look != was.look),
        ]
        .into_iter()
        .filter_map(|(what, differs)| differs.then_some(what))
        .collect(),
    };
    if changed.is_empty() {
        Standing::Current(release.tag.clone())
    } else {
        Standing::Outdated(release.tag.clone(), changed)
    }
}

/// Today's date in UTC, as `2026-11-14`, for a release's record. With
/// [`now`] it is where vidni reads the clock, and no build output depends
/// on either. `SOURCE_DATE_EPOCH` isn't consulted: Nix shells set it to
/// 1980, which no release was approved in.
pub fn today() -> String {
    date(clock() / 86_400)
}

/// This moment in UTC, as `2026-11-14T10:22:03Z`, for a take's record.
pub fn now() -> String {
    let seconds = clock();
    let of_day = seconds % 86_400;
    format!(
        "{}T{:02}:{:02}:{:02}Z",
        date(seconds / 86_400),
        of_day / 3600,
        of_day / 60 % 60,
        of_day % 60
    )
}

/// Seconds since 1970-01-01, by the system's clock.
fn clock() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// The date `days` after 1970-01-01, in the Gregorian calendar.
fn date(days: u64) -> String {
    // Days are counted from 0000-03-01, so that a leap day ends its year.
    let z = days + 719_468;
    let (era, day_of_era) = (z / 146_097, z % 146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let m = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * m + 2) / 5 + 1;
    let month = if m < 10 { m + 3 } else { m - 9 };
    let year = year_of_era + era * 400 + u64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_follow_the_gregorian_calendar() {
        assert_eq!(date(0), "1970-01-01");
        assert_eq!(date(59), "1970-03-01");
        assert_eq!(date(11_016), "2000-02-29");
        assert_eq!(date(11_017), "2000-03-01");
        assert_eq!(date(19_782), "2024-02-29");
        assert_eq!(date(20_771), "2026-11-14");
        assert_eq!(date(47_540), "2100-02-28");
        assert_eq!(date(47_541), "2100-03-01");
    }

    #[test]
    fn what_the_apps_cover_is_in_a_cuts_look_only_when_they_cover_something() {
        let project: Project = toml::from_str(crate::TEMPLATE).unwrap();
        let of = |id: &str| {
            let cut = project.cuts.iter().find(|c| c.id == id).unwrap();
            look(&project, cut)["cut"].clone()
        };
        // YouTube covers nothing of a 16:9 cut: its look has no such key.
        assert!(of("long").get("covered").is_none());
        assert_eq!(
            of("vertical")["covered"],
            json!({"top": 288, "bottom": 672, "left": 65, "right": 193})
        );
        // Nor has a 9:16 cut made for no platform.
        let mut bare = project.clone();
        bare.cuts[1].platforms = Some(Vec::new());
        assert!(look(&bare, &bare.cuts[1])["cut"].get("covered").is_none());
    }

    fn record(cut: &str, mp4: &str) -> Record {
        let hash = |s: &str| sha256_hex(s.as_bytes());
        Record {
            cut: cut.into(),
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
                pcm_normalized: hash("norm"),
                chapters: None,
            },
            outputs: BTreeMap::from([(
                format!("{cut}.mp4"),
                Output {
                    captions: true,
                    frames: hash("frames"),
                    frame_count: 90,
                    packets: hash("packets"),
                    mp4: hash(mp4),
                    qa: "1920x1080 h264 30/1 fps".into(),
                    problems: Vec::new(),
                },
            )]),
            tools: Tools {
                vidni: "0.1.0".into(),
                ffmpeg: "9.0.2".into(),
                fonts: BTreeMap::new(),
                engine: json!({"fps": 30}),
            },
        }
    }

    #[test]
    fn releases_are_numbered_per_cut_and_never_repeated() {
        let mut releases = Releases::default();
        let first = record("long", "a");
        let Approval::Approved(r1) = approve(&mut releases, &first, "2026-11-14") else {
            panic!("the first approval makes a release");
        };
        assert_eq!(
            (r1.tag.as_str(), r1.approved.as_str()),
            ("long-r1", "2026-11-14")
        );
        // The same build again is the same release, whatever the day.
        assert_eq!(
            approve(&mut releases, &first, "2026-12-01"),
            Approval::Already(r1.clone())
        );
        // Another cut counts on its own, and a changed build is the next release.
        let Approval::Approved(short) = approve(&mut releases, &record("short", "a"), "2026-11-15")
        else {
            panic!("another cut makes its own release");
        };
        assert_eq!(short.tag, "short-r1");
        let Approval::Approved(r2) = approve(&mut releases, &record("long", "b"), "2026-11-16")
        else {
            panic!("a changed build makes the next release");
        };
        assert_eq!(r2.tag, "long-r2");
        assert_eq!(releases.cuts["long"], [r1, r2.clone()]);
        assert_eq!(releases.latest("long"), Some(&r2));
        assert_eq!(releases.latest("nope"), None);
    }

    #[test]
    fn releases_round_trip_through_their_file() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(read(dir.path()).unwrap(), Releases::default());
        let mut releases = Releases::default();
        approve(&mut releases, &record("long", "a"), "2026-11-14");
        write(dir.path(), &releases).unwrap();
        assert_eq!(read(dir.path()).unwrap(), releases);
        let text = fs::read_to_string(dir.path().join(FILE)).unwrap();
        assert!(text.starts_with("{\n  \"schema\": 1,\n"), "{text}");
        assert!(text.ends_with("}\n") && text.contains("\"tag\": \"long-r1\""));
        // A file in a later layout is named as that.
        fs::write(
            dir.path().join(FILE),
            text.replace("\"schema\": 1", "\"schema\": 2"),
        )
        .unwrap();
        let error = read(dir.path()).unwrap_err();
        assert!(
            error.ends_with("it is in release schema 2, and this vidni reads schema 1"),
            "{error}"
        );
    }

    #[test]
    fn two_builds_differ_in_exactly_the_hashes_that_changed() {
        let a = record("long", "a");
        assert_eq!(a.differences(&a), Vec::<String>::new());
        let mut b = record("long", "b");
        b.t1.timeline = sha256_hex(b"another timeline");
        b.inputs = sha256_hex(b"other inputs");
        b.outputs.get_mut("long.mp4").unwrap().frame_count = 91;
        assert_eq!(
            a.differences(&b),
            [
                "inputs",
                "t1 timeline",
                "t1 long.mp4 frames",
                "t2 long.mp4 file"
            ]
        );
        let mut c = record("long", "a");
        let extra = c.outputs["long.mp4"].clone();
        c.outputs.insert("long.captioned.mp4".into(), extra);
        assert_eq!(
            a.differences(&c),
            ["long.captioned.mp4 is in one build only"]
        );
    }

    #[test]
    fn a_file_is_named_from_the_project_folder() {
        let dir = Path::new("/somewhere/a project");
        assert_eq!(
            relative(dir, &dir.join("takes/intro-1/0123.wav")),
            "takes/intro-1/0123.wav"
        );
        assert_eq!(
            relative(Path::new("."), Path::new("./build/voice/a/k-1.wav")),
            "build/voice/a/k-1.wav"
        );
    }

    #[test]
    fn a_builds_qa_problems_name_their_file() {
        let mut r = record("long", "a");
        assert!(r.problems().is_empty());
        r.outputs.get_mut("long.mp4").unwrap().problems = vec!["there is no audio stream".into()];
        assert_eq!(r.problems(), ["long.mp4: there is no audio stream"]);
    }
}
