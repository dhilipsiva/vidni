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
