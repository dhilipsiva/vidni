//! The listening check (TODO.md, P3-VIDNI-05): what whisper hears in a
//! clip, and how close that is to the script.
//!
//! It only advises. Nothing it says is ever written into an output or
//! decides one: what whisper hears isn't held to any of vidni's
//! determinism tiers, and a model's mishearing shouldn't change a video.
//! It is there to point a person at a line that may have been read, or
//! synthesized, differently from the script.
//!
//! The pinned `whisper-cli` does the listening, with the pinned model
//! (DECISIONS.md, T7). A clip with nothing to hear (empty, too short or
//! silent) is never given to whisper, which fails on an empty clip and
//! hears words in silence. And nothing here fails a build: when the check
//! can't run, it says why.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use vidni_core::project::{HeardAs, Project};
use vidni_core::voices::{self, Pack};
use vidni_core::{spoken, takes};

/// The model the check uses: whisper.cpp's base English model.
pub const MODEL: &str = "ggml-base.en.bin";
/// Where vidni's flake, and later `vidni setup`, fetch the model from.
pub const MODEL_URL: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/\
    5359861c739e955e79d9a303bcbc70fb988958b1/ggml-base.en.bin";
pub const MODEL_SIZE: u64 = 147_964_211;
pub const MODEL_SHA256: &str = "a03779c86df3323075f5e796cb2ce5029f00ec8869eee3fdfb897afe36c6d002";

/// A line heard less close to its script than this is worth a person's
/// listen.
pub const WORTH_A_LISTEN: f64 = 0.9;

/// A clip shorter than this, in seconds, holds no speech to check.
const SHORTEST: f64 = 0.3;
/// A clip that never rises above this share of full scale is silent
/// (about -60 dBFS).
const QUIETEST: f64 = 0.001;

/// What the check made of a clip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Heard {
    /// What whisper heard.
    Words(String),
    /// Nothing to hear: the clip is empty, too short or silent, or whisper
    /// heard no words in it.
    Nothing,
    /// The check couldn't run, and why.
    Unavailable(String),
}

/// Whether the check can listen to a script in `language`, a BCP 47 tag.
/// Its model knows English only, and its score compares English letters.
pub fn understands(language: &str) -> bool {
    let primary = language.split(['-', '_']).next().unwrap_or_default();
    primary.eq_ignore_ascii_case("en")
}

/// Where the model is: `$VIDNI_WHISPER_MODEL`, else `models/` beside the
/// voice store (`$XDG_DATA_HOME/vidni/models`, else
/// `~/.local/share/vidni/models`).
pub fn model_path() -> Option<PathBuf> {
    let var = |name: &str| std::env::var_os(name).filter(|v| !v.is_empty());
    if let Some(file) = var("VIDNI_WHISPER_MODEL") {
        return Some(PathBuf::from(file));
    }
    let data = var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| var("HOME").map(|home| Path::new(&home).join(".local/share")))?;
    Some(data.join("vidni/models").join(MODEL))
}

/// whisper-cli with the model.
#[derive(Debug, Clone)]
pub struct Listener {
    command: PathBuf,
    model: PathBuf,
}

impl Listener {
    /// The listener: `$VIDNI_WHISPER`, or `whisper-cli` on the path, with
    /// the model at [`model_path`], which must be the pinned file. When
    /// both are missing, the error names both.
    pub fn find() -> Result<Self, String> {
        let command = std::env::var_os("VIDNI_WHISPER")
            .filter(|v| !v.is_empty())
            .map_or_else(|| PathBuf::from("whisper-cli"), PathBuf::from);
        let runs = Command::new(&command)
            .arg("--help")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| {
                format!(
                    "{}: {e}; it is whisper.cpp's command, and VIDNI_WHISPER can name it",
                    command.display()
                )
            });
        match (runs, pinned_model()) {
            (Ok(_), Ok(model)) => Ok(Self { command, model }),
            (Err(command), Err(model)) => Err(format!("{command}; and {model}")),
            (Err(e), Ok(_)) | (Ok(_), Err(e)) => Err(e),
        }
    }

    /// What is heard in `clip`, a WAV file. Never fails: a clip with
    /// nothing to hear gives [`Heard::Nothing`], and anything that keeps
    /// the check from running gives [`Heard::Unavailable`].
    pub fn hear(&self, clip: &Path) -> Heard {
        match audible(clip) {
            Ok(true) => {}
            Ok(false) => return Heard::Nothing,
            Err(e) => return Heard::Unavailable(e),
        }
        match self.transcribe(clip) {
            Ok(transcript) => heard_in(&transcript),
            Err(e) => Heard::Unavailable(e),
        }
    }

    fn transcribe(&self, clip: &Path) -> Result<String, String> {
        let shown = |e: &dyn std::fmt::Display| format!("{}: {e}", self.command.display());
        let dir = tempfile::tempdir().map_err(|e| shown(&e))?;
        let out = dir.path().join("heard");
        let done = Command::new(&self.command)
            .arg("-m")
            .arg(&self.model)
            .arg("-f")
            .arg(clip)
            .args(["-nt", "-np", "-otxt", "-of"])
            .arg(&out)
            .output()
            .map_err(|e| shown(&e))?;
        if !done.status.success() {
            let said = String::from_utf8_lossy(&done.stderr);
            let last = said.lines().last().unwrap_or_default().trim().to_string();
            return Err(shown(&format!("{} ({last})", done.status)));
        }
        std::fs::read_to_string(out.with_extension("txt")).map_err(|e| shown(&e))
    }
}

/// The model's file, checked to be the pinned one.
fn pinned_model() -> Result<PathBuf, String> {
    let model = model_path().ok_or("no place to look for the whisper model: set HOME")?;
    let meta = std::fs::metadata(&model).map_err(|e| {
        format!(
            "{}: {e}; the model is {MODEL_URL}, and VIDNI_WHISPER_MODEL can name a copy",
            model.display()
        )
    })?;
    let pinned = meta.len() == MODEL_SIZE && {
        let bytes = std::fs::read(&model).map_err(|e| format!("{}: {e}", model.display()))?;
        vidni_core::capture::sha256_hex(&bytes) == MODEL_SHA256
    };
    if pinned {
        Ok(model)
    } else {
        Err(format!(
            "{}: not the pinned model (sha256 {MODEL_SHA256})",
            model.display()
        ))
    }
}

/// Which of a line's audio a clip is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// The creator's take for the line's present words.
    Take,
    /// The draft voice's clip for them.
    Draft,
}

/// A line's audio, and the words it should hold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clip {
    pub kind: Kind,
    pub path: PathBuf,
    /// What the audio was read from: the line's spoken form for a take,
    /// and what the draft voice was given for its clip.
    pub script: String,
}

/// The audio to listen to for each line, by line ID in script order: its
/// current take, or else the clip the last draft kept for its present
/// wording in `pack`'s voice. With `draft_only`, the draft clip even where
/// there is a take. `None` for a line with neither.
pub fn clips(
    project: &Project,
    dir: &Path,
    pack: Option<&Pack>,
    draft_only: bool,
) -> Result<Vec<(String, Option<Clip>)>, String> {
    let lock = voices::read_lock(dir)?;
    let takes = takes::status(project, dir);
    let clips = project.lines.iter().zip(takes).map(|(line, take)| {
        let take = match take.status {
            takes::Status::Current(path) if !draft_only => Some(Clip {
                kind: Kind::Take,
                path,
                script: spoken::spoken(line, &project.voice),
            }),
            _ => None,
        };
        let drafted = || {
            let script = spoken::synthetic(line, &project.voice);
            let key = voices::clip_key(&script, pack?, project.voice.length_scale);
            // A clip kept for another key was spoken from an earlier wording.
            let kept = lock.lines.get(&line.id).filter(|kept| kept.key == key)?;
            let path = voices::clip_path(dir, &line.id, &key, kept.seed);
            path.is_file().then_some(Clip {
                kind: Kind::Draft,
                path,
                script,
            })
        };
        (line.id.clone(), take.or_else(drafted))
    });
    Ok(clips.collect())
}

/// Whether a clip has anything to hear: it is at least [`SHORTEST`] long
/// and rises above [`QUIETEST`].
fn audible(clip: &Path) -> Result<bool, String> {
    let shown = |e: hound::Error| format!("{}: {e}", clip.display());
    let mut reader = hound::WavReader::open(clip).map_err(shown)?;
    let spec = reader.spec();
    let seconds = f64::from(reader.duration()) / f64::from(spec.sample_rate.max(1));
    if seconds < SHORTEST {
        return Ok(false);
    }
    let peak = match spec.sample_format {
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .try_fold(0.0f64, |peak, s| s.map(|s| peak.max(f64::from(s.abs()))))
            .map_err(shown)?,
        hound::SampleFormat::Int => {
            let full = f64::from(1u32 << (spec.bits_per_sample.clamp(1, 32) - 1));
            reader
                .samples::<i32>()
                .try_fold(0.0f64, |peak, s| {
                    s.map(|s| peak.max(f64::from(s.unsigned_abs()) / full))
                })
                .map_err(shown)?
        }
    };
    Ok(peak > QUIETEST)
}

/// What a transcript amounts to: its words, or nothing when it holds none.
fn heard_in(transcript: &str) -> Heard {
    let words = without_notes(transcript);
    if words.chars().any(char::is_alphanumeric) {
        Heard::Words(words)
    } else {
        Heard::Nothing
    }
}

/// A transcript on one line, without whisper's notes on sounds that aren't
/// speech, such as `[BLANK_AUDIO]` or `(machine whirring)`: a hum or a
/// click in a take is no word of it. A bracket that never closes is kept
/// as written.
fn without_notes(transcript: &str) -> String {
    let mut kept = String::with_capacity(transcript.len());
    // Where each note that is still open starts in `kept`.
    let mut open: Vec<usize> = Vec::new();
    for c in transcript.chars() {
        match c {
            '[' | '(' => {
                open.push(kept.len());
                kept.push(c);
            }
            ']' | ')' => match open.pop() {
                Some(start) => kept.truncate(start),
                None => kept.push(c),
            },
            _ => kept.push(c),
        }
    }
    kept.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// How close `heard` is to `script`, from 0 to 1, as the reference scores
/// it: both are reduced to their letters, and the score is the share of
/// letters in matching runs (Python's `difflib.SequenceMatcher.ratio`).
/// `heard_as` (`voice.heard_as`) names words whisper writes differently
/// from the script for the same sound, such as `nibley` for `nibli`; each
/// is replaced by what it means on both sides.
pub fn closeness(script: &str, heard: &str, heard_as: &[HeardAs]) -> f64 {
    let a = letters(script, heard_as);
    let b = letters(heard, heard_as);
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    2.0 * matching(&a, &b) as f64 / (a.len() + b.len()) as f64
}

/// The letters of a text: its words (runs of ASCII letters, digits and
/// apostrophes, in lowercase), each replaced by what it means, joined, with
/// everything but the letters dropped.
fn letters(text: &str, heard_as: &[HeardAs]) -> Vec<u8> {
    let lower = text.to_lowercase();
    let mut out = Vec::with_capacity(lower.len());
    let word = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '\'';
    for found in lower.split(|c: char| !word(c)).filter(|w| !w.is_empty()) {
        let means = heard_as
            .iter()
            .find(|h| h.heard.eq_ignore_ascii_case(found))
            .map_or(found, |h| h.means.as_str());
        out.extend(
            means
                .bytes()
                .map(|b| b.to_ascii_lowercase())
                .filter(u8::is_ascii_lowercase),
        );
    }
    out
}

/// How many elements of `a` and `b` are in matching runs, as difflib finds
/// them: the longest common run first (the earliest in `a`, then in `b`,
/// among equals), then the same to its left and to its right.
fn matching(a: &[u8], b: &[u8]) -> usize {
    // Where each byte occurs in b, in order.
    let mut places: [Vec<usize>; 256] = std::array::from_fn(|_| Vec::new());
    for (j, byte) in b.iter().enumerate() {
        places[usize::from(*byte)].push(j);
    }
    let mut total = 0;
    let mut todo = vec![(0, a.len(), 0, b.len())];
    while let Some((alo, ahi, blo, bhi)) = todo.pop() {
        let (mut best_i, mut best_j, mut best) = (alo, blo, 0);
        // run[j]: the length of the common run ending at a[i] and b[j].
        let mut run: Vec<usize> = vec![0; b.len() + 1];
        for i in alo..ahi {
            let mut next: Vec<usize> = vec![0; b.len() + 1];
            for &j in &places[usize::from(a[i])] {
                if j < blo {
                    continue;
                }
                if j >= bhi {
                    break;
                }
                let k = run[j] + 1;
                next[j + 1] = k;
                if k > best {
                    (best_i, best_j, best) = (i + 1 - k, j + 1 - k, k);
                }
            }
            run = next;
        }
        if best > 0 {
            total += best;
            todo.push((alo, best_i, blo, best_j));
            todo.push((best_i + best, ahi, best_j + best, bhi));
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A WAV of `seconds` at 16 kHz: a 440 Hz tone at `level` of full scale.
    fn clip(dir: &Path, name: &str, seconds: f64, level: f64) -> PathBuf {
        let path = dir.join(name);
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 16000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(&path, spec).unwrap();
        for n in 0..(seconds * 16000.0) as u32 {
            let t = f64::from(n) / 16000.0;
            let s = (t * 440.0 * std::f64::consts::TAU).sin() * level * 32767.0;
            writer.write_sample(s as i16).unwrap();
        }
        writer.finalize().unwrap();
        path
    }

    /// A listener whose whisper isn't there: enough for the clips whisper is never given.
    fn deaf() -> Listener {
        Listener {
            command: "/nonexistent/whisper-cli".into(),
            model: "/nonexistent/model.bin".into(),
        }
    }

    #[test]
    fn empty_short_and_silent_clips_have_nothing_to_hear() {
        let dir = tempfile::tempdir().unwrap();
        let listener = deaf();
        for (name, seconds, level) in [
            ("empty.wav", 0.0, 0.0),
            ("short.wav", 0.2, 0.5),
            ("silent.wav", 2.0, 0.0),
            ("hiss.wav", 2.0, 0.0005),
        ] {
            let heard = listener.hear(&clip(dir.path(), name, seconds, level));
            assert_eq!(heard, Heard::Nothing, "{name}");
        }
    }

    #[test]
    fn a_check_that_cant_run_says_why_and_fails_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let listener = deaf();
        let Heard::Unavailable(why) = listener.hear(&clip(dir.path(), "tone.wav", 1.0, 0.5)) else {
            panic!("a missing whisper-cli must make the check unavailable");
        };
        assert!(why.starts_with("/nonexistent/whisper-cli: "), "{why}");
        let Heard::Unavailable(why) = listener.hear(&dir.path().join("missing.wav")) else {
            panic!("a missing clip must make the check unavailable");
        };
        assert!(why.contains("missing.wav"), "{why}");
    }

    #[test]
    fn whispers_notes_on_what_isnt_speech_arent_words() {
        for (transcript, words) in [
            ("", ""),
            ("[BLANK_AUDIO]", ""),
            (" (wind blowing) ", ""),
            ("[ Silence ] ...", "..."),
            ("[BLANK_AUDIO] Hello.", "Hello."),
            (" Adam is a dog.\n [BLANK_AUDIO]\n", "Adam is a dog."),
            ("One (a [nested] note) two", "One two"),
            // A bracket that never closes, or never opened, is no note.
            ("f(x is odd", "f(x is odd"),
            ("so it ends) here", "so it ends) here"),
        ] {
            assert_eq!(without_notes(transcript), words, "{transcript:?}");
        }
    }

    #[test]
    fn what_is_heard_leaves_whispers_notes_out() {
        assert_eq!(
            heard_in(" Adam is a dog.\n [BLANK_AUDIO]\n"),
            Heard::Words("Adam is a dog.".into())
        );
        // Notes alone, or no words at all, are nothing to hear.
        for nothing in ["", "\n", "(eerie music)", " [BLANK_AUDIO]", " ..."] {
            assert_eq!(heard_in(nothing), Heard::Nothing, "{nothing:?}");
        }
    }

    #[test]
    fn closeness_is_the_references_score_for_every_line() {
        let fixture =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hear/closeness.json");
        let fixture: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(fixture).unwrap()).unwrap();
        let heard_as: Vec<HeardAs> = fixture["heard_as"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(heard, means)| HeardAs {
                heard: heard.clone(),
                means: means.as_str().unwrap().to_string(),
            })
            .collect();
        // The example carries the reference's table, so its lines score as
        // the reference scored them.
        let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/nibli-syllogism");
        let text = std::fs::read_to_string(example.join("vidni.toml")).unwrap();
        let project = vidni_core::validate::check(&text, Some(&example)).unwrap();
        let by_word = |list: &[HeardAs]| {
            let mut list = list.to_vec();
            list.sort_by(|a, b| a.heard.cmp(&b.heard));
            list
        };
        assert_eq!(by_word(&project.voice.heard_as), by_word(&heard_as));
        let lines = fixture["lines"].as_array().unwrap();
        assert_eq!(lines.len(), 57);
        for line in lines {
            let (script, heard) = (
                line["script"].as_str().unwrap(),
                line["heard"].as_str().unwrap(),
            );
            let score = (closeness(script, heard, &heard_as) * 10000.0).round() / 10000.0;
            assert_eq!(score, line["score"].as_f64().unwrap(), "{script:?}");
        }
    }

    #[test]
    fn the_check_understands_english_only() {
        for english in ["en", "EN", "en-GB", "en_US"] {
            assert!(understands(english), "{english}");
        }
        for other in ["ta", "ta-IN", "eng", "", "fr-en"] {
            assert!(!understands(other), "{other}");
        }
    }

    /// A project of three lines, in `dir`.
    fn project(dir: &Path) -> Project {
        let text = r#"
schema = 1

[meta]
title = "A project to listen to"
default_scene = "page"

[voice]
respell = [{ text = "vidni", say = "vid nee" }]

[sections.main]

[[lines]]
id = "main-1"
section = "main"
show = "Hello from vidni."

[[lines]]
id = "main-2"
section = "main"
show = "This line has #1 take."
say = "This line has number one take."

[[lines]]
id = "main-3"
section = "main"
show = "Nobody has said this yet."

[scenes.page]
kind = "web"
url = "https://example.com"

[[cuts]]
id = "long"
aspect = "16:9"
"#;
        vidni_core::validate::check(text, Some(dir)).unwrap()
    }

    #[test]
    fn a_line_is_heard_from_its_take_or_else_its_draft_clip() {
        let dir = tempfile::tempdir().unwrap();
        let dir = dir.path();
        let project = project(dir);
        let pack = &voices::all()[0];
        let touch = |path: &Path| {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, b"").unwrap();
        };
        // Nothing recorded or drafted: no line has audio.
        let none = clips(&project, dir, Some(pack), false).unwrap();
        assert_eq!(
            none,
            [("main-1", None), ("main-2", None), ("main-3", None)]
                .map(|(id, clip)| (id.to_string(), clip))
        );

        // A draft keeps a clip for the first two lines, and the second has a take.
        let mut lock = voices::Lock::default();
        let mut drafted = Vec::new();
        for line in &project.lines[..2] {
            let given = spoken::synthetic(line, &project.voice);
            let key = voices::clip_key(&given, pack, project.voice.length_scale);
            let path = voices::clip_path(dir, &line.id, &key, 7);
            touch(&path);
            lock.lines
                .insert(line.id.clone(), voices::Kept { key, seed: 7 });
            drafted.push(Clip {
                kind: Kind::Draft,
                path,
                script: given,
            });
        }
        voices::write_lock(dir, &lock).unwrap();
        let take = takes::status(&project, dir)[1].path.clone();
        touch(&take);
        assert_eq!(drafted[0].script, "Hello from vid nee.");

        let heard = clips(&project, dir, Some(pack), false).unwrap();
        assert_eq!(heard[0].1.as_ref(), Some(&drafted[0]));
        assert_eq!(
            heard[1].1,
            Some(Clip {
                kind: Kind::Take,
                path: take,
                script: "This line has number one take.".into(),
            })
        );
        assert_eq!(heard[2].1, None);
        // Asked for the draft voice, the take is passed over.
        let draft_only = clips(&project, dir, Some(pack), true).unwrap();
        assert_eq!(draft_only[1].1.as_ref(), Some(&drafted[1]));
        // Without the voice's pack, no draft clip can be told to be current.
        let packless = clips(&project, dir, None, false).unwrap();
        assert_eq!(packless[0].1, None);
        assert_eq!(packless[1].1.as_ref().map(|c| c.kind), Some(Kind::Take));

        // A clip kept for an earlier wording isn't the line's audio.
        let mut edited = project.clone();
        edited.lines[0].show = "Hello again from vidni.".into();
        assert_eq!(clips(&edited, dir, Some(pack), false).unwrap()[0].1, None);
        // Nor is a clip the lock names but the cache has lost.
        std::fs::remove_file(&drafted[0].path).unwrap();
        assert_eq!(clips(&project, dir, Some(pack), false).unwrap()[0].1, None);
    }

    #[test]
    fn closeness_reads_past_case_and_punctuation_but_not_words() {
        assert_eq!(closeness("Adam is a dog.", "adam is a dog", &[]), 1.0);
        assert_eq!(closeness("", "", &[]), 1.0);
        assert_eq!(closeness("Adam is a dog.", "", &[]), 0.0);
        let near = closeness("nibli is the engine", "Nibley is the engine", &[]);
        assert!(near < 1.0 && near > 0.8, "{near}");
        // A word is matched whatever its case, and so is what it means.
        let table = [HeardAs {
            heard: "Nibley".into(),
            means: "Nibli".into(),
        }];
        assert_eq!(
            closeness("nibli is the engine", "Nibley is the engine", &table),
            1.0
        );
        // Whole words only: "nibleys" isn't in the table.
        assert!(closeness("niblis", "nibleys", &table) < 1.0);
        // A number can be given its word, since digits aren't letters.
        let three = [HeardAs {
            heard: "3".into(),
            means: "three".into(),
        }];
        assert!(closeness("three lines", "3 lines", &[]) < 1.0);
        assert_eq!(closeness("three lines", "3 lines", &three), 1.0);
        // difflib's own example: 'abcd' against 'bcde' shares 'bcd'.
        assert_eq!(matching(b"abcd", b"bcde"), 3);
        assert_eq!(matching(b"abxcd", b"abcd"), 4);
    }
}
