//! The draft voice (TODO.md, P3-VIDNI-03): every line spoken by a voice
//! pack, seeded, so that a redraft repeats it.
//!
//! The engine is the pinned Piper. Piper 1.8 draws its sampling noise
//! inside the model's ONNX graph, from generators that onnxruntime seeds
//! when a session starts. vidni's flake patches Piper to take `--seed` and
//! set that seed (`nix/piper-seed.patch`); the same seed then gives the same
//! PCM.
//!
//! A line's first draft tries [`ATTEMPTS`] seeds and keeps the take of
//! median length: a take much longer or shorter than the others has usually
//! stretched or dropped a word. The intelligibility check would judge
//! better, but it only advises and never decides an output (P3-VIDNI-05).
//! The kept seed goes into `voice.lock`, and every later draft speaks from
//! it, so clearing the cache and drafting again gives the same PCM.

use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use vidni_core::project::Project;
use vidni_core::spoken;
use vidni_core::voices::{self, Kept, Lock, Pack};

/// How many seeds a line's first draft tries.
pub const ATTEMPTS: usize = 3;

/// Something that speaks a text into a WAV file, seeded.
pub trait Engine {
    fn speak(&self, text: &str, seed: u32, length_scale: f64, out: &Path) -> Result<(), String>;
}

/// Piper with one pack's model: `$VIDNI_PIPER`, or `piper` on the path.
#[derive(Debug, Clone)]
pub struct Piper {
    command: PathBuf,
    model: PathBuf,
    config: PathBuf,
}

impl Piper {
    /// Piper for `pack`, whose files are in `pack_dir`. The files are checked
    /// against the pack's hashes, and Piper must take `--seed`: it ignores
    /// flags it doesn't know, so an unpatched Piper would draft a different
    /// take every time.
    pub fn new(pack: &Pack, pack_dir: &Path) -> Result<Self, String> {
        voices::check_files(pack, pack_dir)?;
        let file = |suffix: &str| {
            pack.file(suffix)
                .map(|f| pack_dir.join(&f.name))
                .ok_or_else(|| format!("{}: the pack has no {suffix} file", pack.id))
        };
        let piper = Self {
            command: std::env::var_os("VIDNI_PIPER")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("piper")),
            model: file(".onnx")?,
            config: file(".onnx.json")?,
        };
        let help = Command::new(&piper.command)
            .arg("--help")
            .output()
            .map_err(|e| format!("{}: {e}", piper.command.display()))?;
        if !String::from_utf8_lossy(&help.stdout).contains("--seed") {
            return Err(format!(
                "{} can't be seeded: vidni needs Piper with its --seed patch \
                 (nix/piper-seed.patch, in vidni's flake)",
                piper.command.display()
            ));
        }
        Ok(piper)
    }

    /// Piper's arguments for one take; the text goes to its stdin.
    pub fn args(&self, seed: u32, length_scale: f64, out: &Path) -> Vec<OsString> {
        let mut args: Vec<OsString> = vec!["--model".into(), self.model.clone().into()];
        args.extend(["--config".into(), self.config.clone().into()]);
        args.extend(["--seed".into(), seed.to_string().into()]);
        args.extend(["--length-scale".into(), length_scale.to_string().into()]);
        args.extend(["--output-file".into(), out.into()]);
        args
    }
}

impl Engine for Piper {
    fn speak(&self, text: &str, seed: u32, length_scale: f64, out: &Path) -> Result<(), String> {
        let shown = |e: &dyn std::fmt::Display| format!("{}: {e}", self.command.display());
        let mut child = Command::new(&self.command)
            .args(self.args(seed, length_scale, out))
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| shown(&e))?;
        child
            .stdin
            .take()
            .ok_or("no stdin")?
            .write_all(text.as_bytes())
            .map_err(|e| shown(&e))?;
        let done = child.wait_with_output().map_err(|e| shown(&e))?;
        if !done.status.success() {
            return Err(shown(&String::from_utf8_lossy(&done.stderr).trim()));
        }
        Ok(())
    }
}

/// One line's draft clip.
#[derive(Debug, Clone, PartialEq)]
pub struct Drafted {
    pub line: String,
    pub clip: PathBuf,
    pub seed: u32,
    /// How the clip was found: in the cache, spoken from the lock's seed, or
    /// chosen afresh from [`ATTEMPTS`] takes.
    pub from: From,
}

/// Where a draft clip came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum From {
    Cache,
    Lock,
    Chosen,
}

/// Drafts every line's clip with `engine`, from `voice.lock` where it holds,
/// and writes the lock: each current line with the seed kept for it.
pub fn draft(
    project: &Project,
    dir: &Path,
    pack: &Pack,
    engine: &dyn Engine,
) -> Result<Vec<Drafted>, String> {
    let lock = voices::read_lock(dir)?;
    let length_scale = project.voice.length_scale;
    let mut kept = Lock::default();
    let mut drafted = Vec::new();
    for line in &project.lines {
        let text = spoken::synthetic(line, &project.voice);
        let key = voices::clip_key(&text, pack, length_scale);
        let say = |seed: u32| speak(engine, dir, &line.id, &key, seed, &text, length_scale);
        let locked = lock
            .lines
            .get(&line.id)
            .filter(|k| k.key == key)
            .map(|k| k.seed);
        let (seed, from) = match locked {
            Some(seed) => match say(seed)? {
                true => (seed, From::Lock),
                false => (seed, From::Cache),
            },
            None => {
                let seeds = voices::candidate_seeds(project.voice.seed, &line.id, &key, ATTEMPTS);
                let mut takes = Vec::with_capacity(seeds.len());
                for seed in seeds {
                    say(seed)?;
                    takes.push((frames(&voices::clip_path(dir, &line.id, &key, seed))?, seed));
                }
                let seed = median(&takes);
                for (_, other) in takes.iter().filter(|(_, s)| *s != seed) {
                    let loser = voices::clip_path(dir, &line.id, &key, *other);
                    fs::remove_file(&loser).map_err(|e| format!("{}: {e}", loser.display()))?;
                }
                (seed, From::Chosen)
            }
        };
        kept.lines.insert(
            line.id.clone(),
            Kept {
                key: key.clone(),
                seed,
            },
        );
        drafted.push(Drafted {
            line: line.id.clone(),
            clip: voices::clip_path(dir, &line.id, &key, seed),
            seed,
            from,
        });
    }
    voices::write_lock(dir, &kept)?;
    Ok(drafted)
}

/// Speaks one take into the cache unless it is there already; true when
/// it spoke.
fn speak(
    engine: &dyn Engine,
    dir: &Path,
    id: &str,
    key: &str,
    seed: u32,
    text: &str,
    length_scale: f64,
) -> Result<bool, String> {
    let clip = voices::clip_path(dir, id, key, seed);
    if clip.is_file() {
        return Ok(false);
    }
    let folder = clip.parent().ok_or("a clip path has a folder")?;
    fs::create_dir_all(folder).map_err(|e| format!("{}: {e}", folder.display()))?;
    // Written aside and moved in, so an interrupted take never looks cached.
    let part = clip.with_extension("wav.part");
    engine.speak(text, seed, length_scale, &part)?;
    fs::rename(&part, &clip).map_err(|e| format!("{}: {e}", clip.display()))?;
    Ok(true)
}

/// A WAV file's length in sample frames.
fn frames(path: &Path) -> Result<u32, String> {
    hound::WavReader::open(path)
        .map(|r| r.duration())
        .map_err(|e| format!("{}: {e}", path.display()))
}

/// The seed of the take of median length; between takes of equal length,
/// the one tried first.
fn median(takes: &[(u32, u32)]) -> u32 {
    let mut order: Vec<usize> = (0..takes.len()).collect();
    order.sort_by_key(|&i| (takes[i].0, i));
    takes[order[order.len() / 2]].1
}

/// Fetches a pack's files into its folder in `store` with curl, checks each
/// one's size and SHA-256, and writes the pack's `pack.toml` beside them.
/// Files already there and matching are kept.
pub fn fetch(pack: &Pack, store: &Path) -> Result<PathBuf, String> {
    let dir = voices::pack_dir(store, &pack.id);
    fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for file in &pack.files {
        let path = dir.join(&file.name);
        if voices::holds(&path, file)? {
            continue;
        }
        let url = file
            .url
            .as_deref()
            .ok_or_else(|| format!("{}: no URL to fetch it from", file.name))?;
        let part = dir.join(format!("{}.part", file.name));
        let done = Command::new("curl")
            .args(["--fail", "--location", "--silent", "--show-error"])
            .args(["--retry", "3", "--output"])
            .arg(&part)
            .arg(url)
            .output()
            .map_err(|e| format!("curl: {e}"))?;
        if !done.status.success() {
            return Err(format!(
                "{url}: {}",
                String::from_utf8_lossy(&done.stderr).trim()
            ));
        }
        if !voices::holds(&part, file)? {
            let _ = fs::remove_file(&part);
            return Err(format!(
                "{url}: not the file the pack pins (its size or SHA-256 differs)"
            ));
        }
        fs::rename(&part, &path).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    voices::write_pack(&dir, pack)?;
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    /// Speaks a tone whose length comes from the text and the seed, and
    /// counts what it was asked for.
    #[derive(Default)]
    struct Tone {
        spoken: RefCell<Vec<(String, u32)>>,
    }

    impl Engine for Tone {
        fn speak(
            &self,
            text: &str,
            seed: u32,
            length_scale: f64,
            out: &Path,
        ) -> Result<(), String> {
            self.spoken.borrow_mut().push((text.to_string(), seed));
            let spec = hound::WavSpec {
                channels: 1,
                sample_rate: 22050,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            };
            let mut w = hound::WavWriter::create(out, spec).map_err(|e| e.to_string())?;
            let n = ((text.len() as f64 * 100.0 + f64::from(seed % 1000)) * length_scale) as u32;
            for i in 0..n {
                w.write_sample(((i * 37 + seed) % 2000) as i16 - 1000)
                    .map_err(|e| e.to_string())?;
            }
            w.finalize().map_err(|e| e.to_string())
        }
    }

    fn project() -> Project {
        let text = r#"
            schema = 1
            [meta]
            title = "T"
            default_scene = "page"
            [sections.a]
            [[lines]]
            id = "one"
            section = "a"
            show = "The first line."
            [[lines]]
            id = "two"
            section = "a"
            show = "The second line, which is longer."
            [scenes.page]
            kind = "web"
            url = "https://example.com"
            [[cuts]]
            id = "long"
            aspect = "16:9"
        "#;
        vidni_core::validate::check(text, None).unwrap()
    }

    fn joe() -> Pack {
        voices::all().remove(0)
    }

    fn pcm(clips: &[Drafted]) -> Vec<Vec<u8>> {
        clips.iter().map(|d| fs::read(&d.clip).unwrap()).collect()
    }

    #[test]
    fn a_first_draft_keeps_the_median_take_and_locks_its_seed() {
        let dir = tempfile::tempdir().unwrap();
        let tone = Tone::default();
        let drafted = draft(&project(), dir.path(), &joe(), &tone).unwrap();
        assert_eq!(tone.spoken.borrow().len(), 2 * ATTEMPTS);
        assert!(
            drafted
                .iter()
                .all(|d| d.from == From::Chosen && d.clip.is_file())
        );
        // The kept take is the middle one by length.
        let text = "The first line.";
        let key = voices::clip_key(text, &joe(), 1.0);
        let seeds = voices::candidate_seeds(0, "one", &key, ATTEMPTS);
        let mut by_length: Vec<u32> = seeds.clone();
        by_length.sort_by_key(|s| s % 1000);
        assert_eq!(drafted[0].seed, by_length[1]);
        // Only the kept takes stay in the cache.
        let left = fs::read_dir(dir.path().join("build/voice/one"))
            .unwrap()
            .count();
        assert_eq!(left, 1);
        let lock = voices::read_lock(dir.path()).unwrap();
        assert_eq!(lock.lines["one"].seed, drafted[0].seed);
        assert_eq!(lock.lines["two"].seed, drafted[1].seed);
    }

    #[test]
    fn clearing_the_cache_and_redrafting_gives_identical_pcm() {
        let dir = tempfile::tempdir().unwrap();
        let first = draft(&project(), dir.path(), &joe(), &Tone::default()).unwrap();
        let before = pcm(&first);
        let again = Tone::default();
        let cached = draft(&project(), dir.path(), &joe(), &again).unwrap();
        assert!(again.spoken.borrow().is_empty());
        assert!(cached.iter().all(|d| d.from == From::Cache));
        fs::remove_dir_all(dir.path().join("build")).unwrap();
        let fresh = Tone::default();
        let redrafted = draft(&project(), dir.path(), &joe(), &fresh).unwrap();
        // One take per line, from the locked seed, and the same PCM.
        assert_eq!(fresh.spoken.borrow().len(), 2);
        assert!(redrafted.iter().all(|d| d.from == From::Lock));
        assert_eq!(pcm(&redrafted), before);
    }

    #[test]
    fn an_edited_line_is_drafted_afresh_and_the_others_keep_their_takes() {
        let dir = tempfile::tempdir().unwrap();
        draft(&project(), dir.path(), &joe(), &Tone::default()).unwrap();
        let mut edited = project();
        edited.lines[1].show = "The second line, said another way.".into();
        let tone = Tone::default();
        let drafted = draft(&edited, dir.path(), &joe(), &tone).unwrap();
        assert_eq!(drafted[0].from, From::Cache);
        assert_eq!(drafted[1].from, From::Chosen);
        assert_eq!(tone.spoken.borrow().len(), ATTEMPTS);
    }

    #[test]
    fn a_seed_given_in_the_lock_is_the_take_drafted() {
        let dir = tempfile::tempdir().unwrap();
        draft(&project(), dir.path(), &joe(), &Tone::default()).unwrap();
        let mut lock = voices::read_lock(dir.path()).unwrap();
        lock.lines.get_mut("one").unwrap().seed = 12345;
        voices::write_lock(dir.path(), &lock).unwrap();
        let tone = Tone::default();
        let drafted = draft(&project(), dir.path(), &joe(), &tone).unwrap();
        assert_eq!((drafted[0].seed, drafted[0].from), (12345, From::Lock));
        assert_eq!(tone.spoken.borrow()[0].1, 12345);
    }

    #[test]
    fn piper_is_given_the_model_the_seed_and_the_pace() {
        let piper = Piper {
            command: "piper".into(),
            model: "/v/m.onnx".into(),
            config: "/v/m.onnx.json".into(),
        };
        let args: Vec<String> = piper
            .args(7, 1.1, Path::new("/o.wav"))
            .into_iter()
            .map(|a| a.into_string().unwrap())
            .collect();
        assert_eq!(
            args,
            [
                "--model",
                "/v/m.onnx",
                "--config",
                "/v/m.onnx.json",
                "--seed",
                "7",
                "--length-scale",
                "1.1",
                "--output-file",
                "/o.wav"
            ]
        );
    }

    #[test]
    fn piper_refuses_a_pack_whose_files_are_missing() {
        let store = tempfile::tempdir().unwrap();
        let error = Piper::new(&joe(), store.path()).unwrap_err();
        assert!(
            error.contains("vidni voices fetch piper:en_US-joe-medium"),
            "{error}"
        );
    }
}
