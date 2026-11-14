//! Draft voices: the voice packs vidni knows, the store they are fetched
//! into, and `voice.lock`, the seed each line's draft take was kept at.
//!
//! A voice pack is a model's files, pinned by size and SHA-256, plus what
//! its model card says about the licence. vidni ships a few packs
//! (`voices.toml`). A creator's own pack is a folder in the store holding its
//! files and a `pack.toml` with the same fields.
//!
//! A line's draft clip is filed under a key: a hash of the text the voice is
//! given, the pack's files and the pace. Piper samples, so the clip also
//! depends on a seed. The first draft of a line tries a few seeds and keeps
//! one; `voice.lock` records it, so that every redraft repeats that take
//! (docs/spec.md, The draft voice).

use crate::capture::sha256_hex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// The packs' data, embedded in vidni.
pub const DATA: &str = include_str!("voices.toml");

/// The lock, at the project folder's top (committed with the project).
pub const LOCK: &str = "voice.lock";

/// A pack's own description, in its folder in the store.
pub const PACK_FILE: &str = "pack.toml";

const LOCK_HEADER: &str = "# Written by `vidni speak` and `vidni draft`: the seed each line's draft take was kept at.\n\
                           # Commit it, so that a redraft gives the same audio. To hear another take of\n\
                           # a line, give it another seed (docs/spec.md, The draft voice).\n";

/// Every pack in `voices.toml`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Packs {
    pub voice: Vec<Pack>,
}

/// A voice pack: its files and its licence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pack {
    /// What `voice.draft` names: `<engine>:<model>`.
    pub id: String,
    pub language: String,
    /// The licence the model card gives, as an SPDX ID where there is one.
    pub licence: String,
    /// Whether that licence allows commercial use.
    pub commercial: bool,
    /// What the licence covers: the training data, and the model it was
    /// fine-tuned from.
    pub notes: String,
    /// The model card the licence was read from.
    pub source: String,
    /// When it was read.
    pub checked: String,
    pub files: Vec<PackFile>,
}

/// One file of a pack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackFile {
    pub name: String,
    /// Where vidni fetches it from; a creator's own pack has none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub size: u64,
    pub sha256: String,
}

impl Pack {
    /// The engine and the model, from the ID.
    pub fn engine_and_model(&self) -> (&str, &str) {
        self.id.split_once(':').unwrap_or(("", &self.id))
    }

    /// The pack's file whose name ends with `suffix`.
    pub fn file(&self, suffix: &str) -> Option<&PackFile> {
        self.files.iter().find(|f| f.name.ends_with(suffix))
    }
}

/// Every pack vidni ships, in the order of `voices.toml`.
pub fn all() -> Vec<Pack> {
    toml::from_str::<Packs>(DATA)
        .expect("voices.toml is checked by the tests")
        .voice
}

/// The voice store: `$VIDNI_VOICES`, else `$XDG_DATA_HOME/vidni/voices`,
/// else `~/.local/share/vidni/voices`.
pub fn store() -> Option<PathBuf> {
    let var = |name: &str| std::env::var_os(name).filter(|v| !v.is_empty());
    if let Some(dir) = var("VIDNI_VOICES") {
        return Some(PathBuf::from(dir));
    }
    let data = var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| var("HOME").map(|home| Path::new(&home).join(".local/share")))?;
    Some(data.join("vidni/voices"))
}

/// The folder of the pack named `id` in `store`: `<engine>/<model>`.
pub fn pack_dir(store: &Path, id: &str) -> PathBuf {
    let (engine, model) = id.split_once(':').unwrap_or(("", id));
    store.join(engine).join(model)
}

/// The pack named `id`: one vidni ships, else a creator's own `pack.toml`
/// in `store`.
pub fn find(id: &str, store: Option<&Path>) -> Result<Option<Pack>, String> {
    if let Some(pack) = all().into_iter().find(|p| p.id == id) {
        return Ok(Some(pack));
    }
    let Some(store) = store else {
        return Ok(None);
    };
    let path = pack_dir(store, id).join(PACK_FILE);
    match fs::read_to_string(&path) {
        Ok(text) => toml::from_str(&text)
            .map(Some)
            .map_err(|e| format!("{}: {}", path.display(), e.message())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}

/// Whether `path` holds `file`: the same size and SHA-256.
pub fn holds(path: &Path, file: &PackFile) -> Result<bool, String> {
    match fs::metadata(path) {
        Ok(meta) if meta.len() == file.size => {
            let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
            Ok(sha256_hex(&bytes) == file.sha256)
        }
        Ok(_) => Ok(false),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}

/// Checks that `dir` holds every file of `pack`, as pinned.
pub fn check_files(pack: &Pack, dir: &Path) -> Result<(), String> {
    for file in &pack.files {
        let path = dir.join(&file.name);
        if !holds(&path, file)? {
            let fix = match file.url {
                Some(_) => format!("run `vidni voices fetch {}`", pack.id),
                None => format!("its {PACK_FILE} pins another file"),
            };
            return Err(format!("{}: missing or changed; {fix}", path.display()));
        }
    }
    Ok(())
}

/// Writes the pack's `pack.toml` into `dir`, so that a fetched pack carries
/// its licence with it.
pub fn write_pack(dir: &Path, pack: &Pack) -> Result<(), String> {
    let path = dir.join(PACK_FILE);
    let body = toml::to_string(pack).map_err(|e| e.to_string())?;
    fs::write(&path, body).map_err(|e| format!("{}: {e}", path.display()))
}

/// What `voice.lock` holds: each line's kept take, by line ID.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Lock {
    #[serde(default)]
    pub lines: BTreeMap<String, Kept>,
}

/// The take kept for one line.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Kept {
    /// The clip key it was kept for ([`clip_key`]). Another key, from an
    /// edited line or another voice, means the line is drafted afresh.
    pub key: String,
    pub seed: u32,
}

/// Reads `voice.lock`; a project without one has kept no takes yet.
pub fn read_lock(project_dir: &Path) -> Result<Lock, String> {
    let path = project_dir.join(LOCK);
    match fs::read_to_string(&path) {
        Ok(text) => {
            toml::from_str(&text).map_err(|e| format!("{}: {}", path.display(), e.message()))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Lock::default()),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}

/// Writes `voice.lock`.
pub fn write_lock(project_dir: &Path, lock: &Lock) -> Result<(), String> {
    let path = project_dir.join(LOCK);
    let body = toml::to_string(lock).map_err(|e| e.to_string())?;
    fs::write(&path, format!("{LOCK_HEADER}\n{body}"))
        .map_err(|e| format!("{}: {e}", path.display()))
}

/// The key a line's draft clip is filed under, before its seed: the first
/// 16 hex digits of a SHA-256 over the text the voice is given, the pack's
/// ID and file hashes, and the pace.
pub fn clip_key(text: &str, pack: &Pack, length_scale: f64) -> String {
    let files: Vec<&str> = pack.files.iter().map(|f| f.sha256.as_str()).collect();
    let input = serde_json::json!({
        "text": text,
        "voice": pack.id,
        "files": files,
        "length_scale": length_scale,
    });
    sha256_hex(input.to_string().as_bytes())[..16].to_string()
}

/// The seeds a line's first draft tries, `n` of them. They come from the
/// project's `voice.seed`, the line's ID and its clip key, so each line has
/// its own and changing `voice.seed` gives every line new ones. Seeds stay
/// below 2^31, which every engine takes.
pub fn candidate_seeds(voice_seed: u64, line_id: &str, key: &str, n: usize) -> Vec<u32> {
    (0..n)
        .map(|i| {
            let digest = sha256_hex(format!("{voice_seed}:{line_id}:{key}:{i}").as_bytes());
            u32::from_str_radix(&digest[..8], 16).expect("hex digits") >> 1
        })
        .collect()
}

/// Where a line's draft clip is cached: `build/voice/<line-id>/<key>-<seed>.wav`.
pub fn clip_path(project_dir: &Path, line_id: &str, key: &str, seed: u32) -> PathBuf {
    project_dir
        .join("build/voice")
        .join(line_id)
        .join(format!("{key}-{seed}.wav"))
}

/// The clip the last draft kept for `line`'s present wording in `pack`'s
/// voice, when the cache still holds it. A clip kept for another key was
/// spoken from an earlier wording, or by another voice.
pub fn drafted(
    project_dir: &Path,
    lock: &Lock,
    pack: &Pack,
    voice: &crate::project::Voice,
    line: &crate::project::Line,
) -> Option<PathBuf> {
    let given = crate::spoken::synthetic(line, voice);
    let key = clip_key(&given, pack, voice.length_scale);
    let kept = lock.lines.get(&line.id).filter(|kept| kept.key == key)?;
    let clip = clip_path(project_dir, &line.id, &key, kept.seed);
    clip.is_file().then_some(clip)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shipped_packs_parse_and_are_complete() {
        let packs = all();
        assert!(packs.iter().any(|p| p.id == "piper:en_US-joe-medium"));
        for p in &packs {
            assert!(p.id.split_once(':').is_some(), "{}", p.id);
            assert!(
                p.file(".onnx").is_some() && p.file(".onnx.json").is_some(),
                "{}",
                p.id
            );
            for f in &p.files {
                assert_eq!(f.sha256.len(), 64, "{}", f.name);
                assert!(f.url.as_deref().is_some_and(|u| u.starts_with("https://")));
            }
        }
    }

    #[test]
    fn the_default_voice_allows_commercial_use_and_one_shipped_voice_does_not() {
        let joe = all().into_iter().find(|p| p.id == "piper:en_US-joe-medium");
        assert!(joe.is_some_and(|p| p.commercial));
        assert!(all().iter().any(|p| !p.commercial));
    }

    #[test]
    fn a_creators_own_pack_is_read_from_the_store() {
        let store = tempfile::tempdir().unwrap();
        let dir = pack_dir(store.path(), "piper:my-voice");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join(PACK_FILE),
            r#"
            id = "piper:my-voice"
            language = "en_GB"
            licence = "proprietary"
            commercial = true
            notes = "Recorded and trained by the creator."
            source = "notes.md"
            checked = "2026-11-13"
            files = [{ name = "my-voice.onnx", size = 1, sha256 = "00" }]
            "#,
        )
        .unwrap();
        let pack = find("piper:my-voice", Some(store.path())).unwrap().unwrap();
        assert_eq!(pack.engine_and_model(), ("piper", "my-voice"));
        assert!(find("piper:nobody", Some(store.path())).unwrap().is_none());
    }

    #[test]
    fn keys_change_with_the_text_the_voice_and_the_pace() {
        let joe = all().remove(0);
        let key = clip_key("Hello there.", &joe, 1.0);
        assert_eq!(key.len(), 16);
        assert_eq!(key, clip_key("Hello there.", &joe, 1.0));
        assert_ne!(key, clip_key("Hello, there.", &joe, 1.0));
        assert_ne!(key, clip_key("Hello there.", &joe, 1.1));
        assert_ne!(key, clip_key("Hello there.", &all().remove(1), 1.0));
    }

    #[test]
    fn seeds_are_per_line_and_follow_the_projects_seed() {
        let a = candidate_seeds(0, "intro-1", "k", 3);
        assert_eq!(a.len(), 3);
        assert_eq!(a, candidate_seeds(0, "intro-1", "k", 3));
        assert_ne!(a, candidate_seeds(1, "intro-1", "k", 3));
        assert_ne!(a, candidate_seeds(0, "intro-2", "k", 3));
        assert!(a.iter().all(|s| *s < 1 << 31));
    }

    #[test]
    fn the_lock_round_trips_with_its_header() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(read_lock(dir.path()).unwrap(), Lock::default());
        let mut lock = Lock::default();
        lock.lines.insert(
            "intro-1".into(),
            Kept {
                key: "0123456789abcdef".into(),
                seed: 42,
            },
        );
        write_lock(dir.path(), &lock).unwrap();
        let text = fs::read_to_string(dir.path().join(LOCK)).unwrap();
        assert!(text.starts_with("# Written by `vidni speak` and `vidni draft`"));
        assert_eq!(read_lock(dir.path()).unwrap(), lock);
    }
}
