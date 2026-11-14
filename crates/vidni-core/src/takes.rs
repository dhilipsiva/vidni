//! The creator's takes, filed by line ID and spoken hash.
//!
//! A project keeps its takes in `takes/`, out of git (DECISIONS.md, D4):
//!
//! ```text
//! takes/<line-id>/<spoken-hash>.wav          the kept take for that wording
//! takes/<line-id>/raw/<spoken-hash>-<n>.wav  every raw take for it, from 1
//! takes/_room/                               room tone from the mic check
//! ```
//!
//! A take stays current while its line's spoken words stay the same.
//! Inserting, removing or moving other lines never touches it, and editing
//! the words leaves it on disk as a stale take, in case the edit is undone.

use crate::project::Project;
use crate::spoken::spoken_hash;
use std::fs;
use std::path::{Path, PathBuf};

/// Where a project's takes live, relative to the project folder.
pub const DIR: &str = "takes";

/// The state of a line's kept take.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    /// A take for the line's current words.
    Current(PathBuf),
    /// Takes for other words only, as after the line's words were edited.
    Stale(Vec<PathBuf>),
    /// No take at all.
    Missing,
}

/// One line's take, in script order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineTake {
    pub id: String,
    pub hash: String,
    /// Where the kept take for the current words is, or goes.
    pub path: PathBuf,
    pub status: Status,
}

/// The kept take for `id`'s words with `hash`.
pub fn kept(project_dir: &Path, id: &str, hash: &str) -> PathBuf {
    project_dir.join(DIR).join(id).join(format!("{hash}.wav"))
}

/// Raw take `n` (from 1) for `id`'s words with `hash`.
pub fn raw(project_dir: &Path, id: &str, hash: &str, n: u32) -> PathBuf {
    project_dir
        .join(DIR)
        .join(id)
        .join("raw")
        .join(format!("{hash}-{n}.wav"))
}

/// The first raw take number not used yet for `id`'s words with `hash`.
pub fn next_raw(project_dir: &Path, id: &str, hash: &str) -> PathBuf {
    (1..)
        .map(|n| raw(project_dir, id, hash, n))
        .find(|p| !p.exists())
        .expect("an unused take number")
}

/// Every line's take, in script order.
pub fn status(project: &Project, project_dir: &Path) -> Vec<LineTake> {
    project
        .lines
        .iter()
        .map(|line| {
            let hash = spoken_hash(line, &project.voice);
            let path = kept(project_dir, &line.id, &hash);
            let status = if path.is_file() {
                Status::Current(path.clone())
            } else {
                let others = other_takes(&project_dir.join(DIR).join(&line.id));
                if others.is_empty() {
                    Status::Missing
                } else {
                    Status::Stale(others)
                }
            };
            LineTake {
                id: line.id.clone(),
                hash,
                path,
                status,
            }
        })
        .collect()
}

/// The kept takes in a line's folder, for any wording, sorted.
fn other_takes(dir: &Path) -> Vec<PathBuf> {
    let mut takes: Vec<PathBuf> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file() && p.extension().is_some_and(|x| x == "wav"))
        .collect();
    takes.sort();
    takes
}

/// Which raw take a line's kept take is cut from, and where: the record
/// beside the kept take, `<spoken-hash>.json`. With it, the kept take can
/// be made again from the raw one, and another raw take picked in its
/// place (TODO.md, P4-VIDNI-04).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pick {
    /// The raw take's number, from 1.
    pub raw: u32,
    /// Where in the raw take the kept one begins and ends, in samples.
    pub from: usize,
    pub to: usize,
    /// `auto` when vidni found the ends, `set` when the creator gave them.
    pub trim: Trim,
}

/// Who chose where a kept take begins and ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Trim {
    Auto,
    Set,
}

/// What a raw take was recorded with: the record beside it,
/// `raw/<spoken-hash>-<n>.json`. It describes the take and decides
/// nothing: no output depends on it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    /// The recording session, named by the moment it began (UTC).
    pub session: String,
    /// When the take was stopped (UTC).
    pub recorded: String,
    /// The input it was recorded from.
    pub microphone: String,
    /// Its samples a second.
    pub rate: u32,
}

/// Where the record beside the take at `wav` is: the same name, as `.json`.
pub fn record_path(wav: &Path) -> PathBuf {
    wav.with_extension("json")
}

fn read_record<T: serde::de::DeserializeOwned>(wav: &Path) -> Result<Option<T>, String> {
    let path = record_path(wav);
    match fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str(&text)
            .map(Some)
            .map_err(|e| format!("{}: {e}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}

fn write_record<T: serde::Serialize>(wav: &Path, record: &T) -> Result<(), String> {
    let path = record_path(wav);
    let json = serde_json::to_string_pretty(record).map_err(|e| e.to_string())? + "\n";
    fs::write(&path, json).map_err(|e| format!("{}: {e}", path.display()))
}

/// The pick recorded beside the kept take at `kept`; none for a take kept
/// before picks were recorded.
pub fn pick(kept: &Path) -> Result<Option<Pick>, String> {
    read_record(kept)
}

/// Records `pick` beside the kept take at `kept`.
pub fn write_pick(kept: &Path, pick: &Pick) -> Result<(), String> {
    write_record(kept, pick)
}

/// The session recorded beside the raw take at `raw`, when there is one.
pub fn session(raw: &Path) -> Result<Option<Session>, String> {
    read_record(raw)
}

/// Records `session` beside the raw take at `raw`.
pub fn write_session(raw: &Path, session: &Session) -> Result<(), String> {
    write_record(raw, session)
}

/// The number of the raw take at `raw`, read off its name.
pub fn raw_number(raw: &Path) -> Option<u32> {
    let stem = raw.file_stem()?.to_str()?;
    stem.rsplit_once('-')?.1.parse().ok()
}

/// Every raw take for `id`'s words with `hash`, by number.
pub fn raws(project_dir: &Path, id: &str, hash: &str) -> Vec<(u32, PathBuf)> {
    let dir = project_dir.join(DIR).join(id).join("raw");
    let mut found: Vec<(u32, PathBuf)> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "wav"))
        .filter(|p| {
            p.file_stem()
                .and_then(|s| s.to_str())
                .and_then(|s| s.rsplit_once('-'))
                .is_some_and(|(words, _)| words == hash)
        })
        .filter_map(|p| raw_number(&p).map(|n| (n, p)))
        .collect();
    found.sort();
    found
}

/// How far apart two sessions' levels may be, in dB, before it is said.
pub const JUMP: f64 = 3.0;

/// A session whose takes are louder or quieter than the rest.
#[derive(Debug, Clone, PartialEq)]
pub struct Jump {
    /// The session that stands out, and its lines, in the order given.
    pub session: String,
    pub lines: Vec<String>,
    /// How much louder (positive) or quieter its takes are, in dB, than
    /// those of `than`: the session with the most lines.
    pub by: f64,
    pub than: String,
}

/// The sessions whose takes differ in level from the main session's by
/// more than [`JUMP`]. `takes` is each line with the session its kept
/// take was recorded in and that take's level in dBFS. A session's level
/// is the median of its takes'; the main session is the one with the most
/// takes, and the earliest of those with as many.
pub fn jumps(takes: &[(String, String, f64)]) -> Vec<Jump> {
    let mut sessions: std::collections::BTreeMap<&str, Vec<(&str, f64)>> = Default::default();
    for (line, session, level) in takes {
        sessions.entry(session).or_default().push((line, *level));
    }
    let median = |takes: &[(&str, f64)]| {
        let mut levels: Vec<f64> = takes.iter().map(|t| t.1).collect();
        levels.sort_by(f64::total_cmp);
        let mid = levels.len() / 2;
        match levels.len() % 2 {
            0 => (levels[mid - 1] + levels[mid]) / 2.0,
            _ => levels[mid],
        }
    };
    // Sessions are named by when they began, so the first of the largest is the earliest.
    let Some((main, level)) = sessions
        .iter()
        .max_by(|a, b| a.1.len().cmp(&b.1.len()).then(b.0.cmp(a.0)))
        .map(|(name, takes)| (*name, median(takes)))
    else {
        return Vec::new();
    };
    sessions
        .iter()
        .filter(|(name, _)| **name != main)
        .filter_map(|(name, takes)| {
            let by = median(takes) - level;
            (by.abs() > JUMP).then(|| Jump {
                session: name.to_string(),
                lines: takes.iter().map(|t| t.0.to_string()).collect(),
                by,
                than: main.to_string(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_sit_beside_their_takes_and_come_back_as_written() {
        let dir = std::env::temp_dir().join(format!("vidni-takes-{}", std::process::id()));
        let raw_dir = dir.join("takes/a-1/raw");
        fs::create_dir_all(&raw_dir).unwrap();
        let kept_take = kept(&dir, "a-1", "abcd");
        assert_eq!(pick(&kept_take), Ok(None));
        let picked = Pick {
            raw: 2,
            from: 9600,
            to: 96000,
            trim: Trim::Auto,
        };
        write_pick(&kept_take, &picked).unwrap();
        assert_eq!(record_path(&kept_take), dir.join("takes/a-1/abcd.json"));
        assert_eq!(pick(&kept_take), Ok(Some(picked)));
        let text = fs::read_to_string(record_path(&kept_take)).unwrap();
        assert!(text.contains("\"trim\": \"auto\""), "{text}");
        // Raw takes are found by their words' hash, in order of number.
        for name in [
            "abcd-2.wav",
            "abcd-10.wav",
            "abcd-1.wav",
            "ffff-1.wav",
            "abcd-2.json",
        ] {
            fs::write(raw_dir.join(name), b"").unwrap();
        }
        let numbers: Vec<u32> = raws(&dir, "a-1", "abcd").iter().map(|r| r.0).collect();
        assert_eq!(numbers, [1, 2, 10]);
        assert_eq!(raw_number(&raw(&dir, "a-1", "abcd", 10)), Some(10));
        let recorded = Session {
            session: "2026-11-14T10:20:11Z".into(),
            recorded: "2026-11-14T10:22:03Z".into(),
            microphone: "a microphone".into(),
            rate: 48000,
        };
        let take = raw(&dir, "a-1", "abcd", 2);
        write_session(&take, &recorded).unwrap();
        assert_eq!(session(&take), Ok(Some(recorded)));
        assert_eq!(session(&raw(&dir, "a-1", "abcd", 1)), Ok(None));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_session_that_is_louder_or_quieter_than_the_main_one_is_named() {
        let take =
            |line: &str, session: &str, level: f64| (line.to_string(), session.to_string(), level);
        let takes = [
            take("a-1", "monday", -24.0),
            take("a-2", "monday", -25.0),
            take("a-3", "monday", -23.0),
            take("b-1", "tuesday", -19.5),
            take("b-2", "tuesday", -20.5),
            take("c-1", "friday", -26.0),
        ];
        assert_eq!(
            jumps(&takes),
            [Jump {
                session: "tuesday".into(),
                lines: vec!["b-1".into(), "b-2".into()],
                by: 4.0,
                than: "monday".into(),
            }]
        );
        // One session, or none, has nothing to differ from.
        assert_eq!(jumps(&takes[..3]), []);
        assert_eq!(jumps(&[]), []);
        // Of two sessions as large, the earlier is the main one.
        let even = [take("a-1", "monday", -24.0), take("b-1", "tuesday", -30.0)];
        assert_eq!(jumps(&even)[0].session, "tuesday");
        assert_eq!(jumps(&even)[0].by, -6.0);
    }
}
