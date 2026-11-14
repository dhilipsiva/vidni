//! A take, from the input to the file a cut is built from.
//!
//! A recording goes straight into its raw file, which is written once and
//! never opened for writing again. When it stops, the raw take is checked
//! and trimmed ([`crate::clean`]) into a candidate beside the line's kept
//! take, to listen to. Keeping the candidate puts it in the kept take's
//! place; discarding it leaves the line as it was. Either way the raw take
//! stays.

use crate::clean::{self, Refused};
use crate::input::{Input, Stream};
use std::collections::VecDeque;
use std::fs::{self, File, OpenOptions};
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

/// How much of the newest sound a meter's level is taken over, in seconds.
pub const METER: f64 = 0.12;

/// What a meter reads.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Level {
    /// How long the recording is so far, in seconds.
    pub seconds: f64,
    /// The level of the newest sound, in dBFS: its RMS over [`METER`].
    pub now: f64,
    /// The loudest sample so far, in dBFS.
    pub peak: f64,
}

/// A level meter: it is fed the samples as they arrive.
#[derive(Debug)]
pub struct Meter {
    rate: u32,
    /// The newest samples, [`METER`] seconds of them at most.
    recent: VecDeque<i16>,
    span: usize,
    count: u64,
    peak: u16,
}

impl Meter {
    pub fn new(rate: u32) -> Self {
        let span = ((METER * f64::from(rate)).round() as usize).max(1);
        Self {
            rate,
            recent: VecDeque::with_capacity(span),
            span,
            count: 0,
            peak: 0,
        }
    }

    pub fn push(&mut self, samples: &[i16]) {
        self.count += samples.len() as u64;
        for sample in samples {
            self.peak = self.peak.max(sample.unsigned_abs());
            if self.recent.len() == self.span {
                self.recent.pop_front();
            }
            self.recent.push_back(*sample);
        }
    }

    pub fn level(&self) -> Level {
        let (older, newer) = self.recent.as_slices();
        let squares: f64 = older
            .iter()
            .chain(newer)
            .map(|sample| f64::from(*sample).powi(2))
            .sum();
        let now = match self.recent.len() {
            0 => clean::FLOOR,
            held => clean::db((squares / held as f64).sqrt() / 32768.0),
        };
        Level {
            seconds: self.count as f64 / f64::from(self.rate.max(1)),
            now,
            peak: clean::db(f64::from(self.peak) / 32768.0),
        }
    }
}

/// What the input's thread and the recorder share.
struct Shared {
    writer: Option<hound::WavWriter<BufWriter<File>>>,
    meter: Meter,
    /// Why the recording can't go on, once it can't.
    failed: Option<String>,
}

/// A recording in progress.
pub struct Recording {
    path: PathBuf,
    stream: Box<dyn Stream>,
    shared: Arc<Mutex<Shared>>,
}

fn locked(shared: &Mutex<Shared>) -> MutexGuard<'_, Shared> {
    // A panic on the input's thread leaves what was written as it is.
    shared
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Starts recording `input` into a new file at `path`: 16-bit PCM, one
/// channel, at the input's rate. A file that is there already is never
/// written over: a raw take is written once.
pub fn record(input: &dyn Input, path: &Path) -> Result<Recording, String> {
    let shown = |e: &dyn std::fmt::Display| format!("{}: {e}", path.display());
    if let Some(folder) = path.parent() {
        fs::create_dir_all(folder).map_err(|e| format!("{}: {e}", folder.display()))?;
    }
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| shown(&e))?;
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: input.rate(),
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let writer = hound::WavWriter::new(BufWriter::new(file), spec).map_err(|e| shown(&e))?;
    let shared = Arc::new(Mutex::new(Shared {
        writer: Some(writer),
        meter: Meter::new(input.rate()),
        failed: None,
    }));
    let (sink, failed) = (shared.clone(), shared.clone());
    let started = input.start(
        Box::new(move |samples| {
            let mut shared = locked(&sink);
            shared.meter.push(samples);
            let Some(writer) = shared.writer.as_mut() else {
                return;
            };
            let written = samples.iter().try_for_each(|s| writer.write_sample(*s));
            if let Err(e) = written {
                shared
                    .failed
                    .get_or_insert(format!("the take can't be written: {e}"));
            }
        }),
        Box::new(move |why| {
            let name = "the input stopped on its own";
            locked(&failed)
                .failed
                .get_or_insert(format!("{name}: {why}"));
        }),
    );
    match started {
        Ok(stream) => Ok(Recording {
            path: path.to_path_buf(),
            stream,
            shared,
        }),
        Err(e) => {
            // Nothing was recorded, so no file is left to look like a take.
            drop(locked(&shared).writer.take());
            let _ = fs::remove_file(path);
            Err(e)
        }
    }
}

impl Recording {
    /// What the meter reads now.
    pub fn level(&self) -> Level {
        locked(&self.shared).meter.level()
    }

    /// Why the recording can't go on, when it can't: the input stopped on
    /// its own, or the file can't be written.
    pub fn failed(&self) -> Option<String> {
        locked(&self.shared).failed.clone()
    }

    /// Stops the input and closes the file. Returns the writer's state.
    fn end(self) -> (PathBuf, Result<(), String>) {
        // The input is stopped first, so that nothing arrives after the
        // file is closed.
        drop(self.stream);
        let mut shared = locked(&self.shared);
        let closed = match shared.writer.take() {
            Some(writer) => writer.finalize().map_err(|e| e.to_string()),
            None => Ok(()),
        };
        let failed = shared.failed.take().map_or(closed, Err);
        (self.path.clone(), failed)
    }

    /// Stops recording. The file is the raw take.
    pub fn stop(self) -> Result<PathBuf, String> {
        let (path, ended) = self.end();
        ended
            .map(|()| path.clone())
            .map_err(|e| format!("{}: {e}", path.display()))
    }

    /// Stops recording, and removes the file: nothing was recorded.
    pub fn cancel(self) -> Result<(), String> {
        let (path, _) = self.end();
        fs::remove_file(&path).map_err(|e| format!("{}: {e}", path.display()))
    }
}

/// The samples of a take, and their rate.
pub fn read(path: &Path) -> Result<(Vec<i16>, u32), String> {
    let shown = |e: hound::Error| format!("{}: {e}", path.display());
    let mut reader = hound::WavReader::open(path).map_err(shown)?;
    let spec = reader.spec();
    if (spec.channels, spec.bits_per_sample, spec.sample_format)
        != (1, 16, hound::SampleFormat::Int)
    {
        return Err(format!(
            "{}: a take is 16-bit PCM with one channel",
            path.display()
        ));
    }
    let samples = reader
        .samples::<i16>()
        .collect::<Result<Vec<i16>, _>>()
        .map_err(shown)?;
    Ok((samples, spec.sample_rate))
}

/// Writes `samples` as a take at `path`.
pub(crate) fn write(path: &Path, samples: &[i16], rate: u32) -> Result<(), String> {
    let shown = |e: &dyn std::fmt::Display| format!("{}: {e}", path.display());
    if let Some(folder) = path.parent() {
        fs::create_dir_all(folder).map_err(|e| format!("{}: {e}", folder.display()))?;
    }
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec).map_err(|e| shown(&e))?;
    samples
        .iter()
        .try_for_each(|s| writer.write_sample(*s))
        .and_then(|()| writer.finalize())
        .map_err(|e| shown(&e))
}

/// A raw take, trimmed, waiting beside the line's kept take to be
/// listened to and kept or discarded.
#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    /// Where it waits.
    pub path: PathBuf,
    /// The kept take it would become.
    pub kept: PathBuf,
    /// How long it is, in seconds, and its loudest sample, in dBFS.
    pub seconds: f64,
    pub peak: f64,
    /// Where in the raw take it begins and ends, in samples.
    pub from: usize,
    pub to: usize,
}

/// Where the candidate for the kept take at `kept` waits. It doesn't end
/// in `.wav`, so that it is never taken for a take.
pub fn candidate_path(kept: &Path) -> PathBuf {
    let mut name = kept.file_name().unwrap_or_default().to_os_string();
    name.push(".new");
    kept.with_file_name(name)
}

/// Checks the raw take at `raw` and trims it into a candidate for the
/// kept take at `kept`. The raw take isn't changed, and neither is the
/// kept take. A take with nothing in it is refused, and leaves no candidate.
pub fn candidate(raw: &Path, kept: &Path) -> Result<Result<Candidate, Refused>, String> {
    let (samples, rate) = read(raw)?;
    let trimmed = match clean::clean(&samples, rate) {
        Ok(trimmed) => trimmed,
        Err(refused) => return Ok(Err(refused)),
    };
    let path = candidate_path(kept);
    write(&path, &trimmed.samples, rate)?;
    Ok(Ok(Candidate {
        path,
        kept: kept.to_path_buf(),
        seconds: trimmed.samples.len() as f64 / f64::from(rate.max(1)),
        peak: clean::peak_db(&trimmed.samples),
        from: trimmed.from,
        to: trimmed.to,
    }))
}

impl Candidate {
    /// Keeps it: it becomes the line's kept take, in the place of the one
    /// before it.
    pub fn keep(self) -> Result<PathBuf, String> {
        fs::rename(&self.path, &self.kept)
            .map(|()| self.kept.clone())
            .map_err(|e| format!("{}: {e}", self.kept.display()))
    }

    /// Discards it: the line is as it was.
    pub fn discard(self) -> Result<(), String> {
        fs::remove_file(&self.path).map_err(|e| format!("{}: {e}", self.path.display()))
    }
}
