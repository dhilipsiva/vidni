//! The recording studio (TODO.md, P7-VIDNI-06): the line to read, a meter
//! and a waveform while it is recorded, the new take beside the kept one,
//! the line's raw takes, another trim, and room tone.
//!
//! It records with the recorder `vidni record` records with, into the
//! same folders, with the same records beside each take. A take made here
//! and one made in the terminal can't be told apart, by vidni or by a
//! person: either can keep, pick or trim the other's.

use crate::actions::Action;
use crate::jobs;
use eframe::egui;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use vidni_core::takes::{self, Pick, Trim};
use vidni_core::{validate, voices};
use vidni_media::sheet::{self, Take};
use vidni_record::input::{self, Input};
use vidni_record::mic::{self, Mic};
use vidni_record::play::{Playing, Speaker};
use vidni_record::take::{self, Candidate, Level, Recording};
use vidni_record::{clean, room};

/// One line to record.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub id: String,
    /// Its section's label.
    pub label: String,
    /// The words to read: the line's spoken form.
    pub text: String,
    /// How long the draft voice took over it, in seconds, when it has.
    pub pace: Option<f64>,
    /// Whether it has a take of its present words.
    pub take: Take,
    /// The hash of its spoken words, which its takes are named by.
    pub hash: String,
    /// Where its kept take is, or would be.
    pub kept: PathBuf,
    /// The cuts that use it.
    pub cuts: Vec<String>,
}

/// One of a line's raw takes.
#[derive(Debug, Clone, PartialEq)]
pub struct Raw {
    pub number: u32,
    pub path: PathBuf,
    /// How long it is, in seconds, and its loudest sample, in dBFS.
    pub seconds: f64,
    pub peak: f64,
    /// When it was recorded, and from what.
    pub recorded: String,
    /// Its samples a second.
    pub rate: u32,
    /// Its waveform.
    pub wave: Vec<f32>,
}

/// What is recorded.
enum Of {
    /// A take of the line with this ID.
    Line(String),
    /// The room, with nobody speaking.
    Room,
}

/// What the studio is doing.
enum Doing {
    Nothing,
    Recording {
        recording: Recording,
        of: Of,
    },
    /// A new take waits beside the line's kept take, to be kept or
    /// discarded.
    Review {
        candidate: Candidate,
        wave: Vec<f32>,
    },
}

/// The studio of one project.
pub struct Studio {
    folder: PathBuf,
    /// What takes are recorded from: found when the first is recorded.
    input: Option<Box<dyn Input>>,
    /// Where takes are played: found when one is first played.
    speaker: Option<Result<Speaker, String>>,
    /// The session's name: the moment it began.
    session: String,
    /// The lines to record, in script order, each once.
    pub lines: Vec<Line>,
    /// Which of them is in the prompter.
    pub at: usize,
    doing: Doing,
    /// What the last thing done came to.
    pub note: String,
    /// The raw takes of the line in the prompter, and which of them its
    /// kept take is cut from, and where.
    pub raws: Vec<Raw>,
    pub pick: Option<Pick>,
    /// The ends of another trim, as typed: seconds of the raw take.
    trim_from: String,
    trim_to: String,
    /// The take that was last asked to be played, and its sound while it
    /// plays.
    pub played: Option<PathBuf>,
    playing: Option<Playing>,
    /// The room tone's level, in dBFS, when the project has room tone.
    pub room: Option<f64>,
    /// Why the project can't be recorded, when it can't.
    pub problem: Option<String>,
    /// When what is shown was read: the times its files were written.
    read: Vec<Option<(SystemTime, u64)>>,
    /// Raw takes that were read already, by file and length.
    cache: HashMap<PathBuf, (u64, Raw)>,
}

/// A clause as a sentence.
fn sentence(clause: &str) -> String {
    let mut letters = clause.chars();
    match letters.next() {
        Some(first) => format!("{}{}.", first.to_uppercase(), letters.as_str()),
        None => String::new(),
    }
}

/// When the file or folder at `path` was last written, and how long it
/// is: two changes within one tick of the clock still differ in length,
/// mostly.
fn written(path: &Path) -> Option<(SystemTime, u64)> {
    let file = std::fs::metadata(path).ok()?;
    Some((file.modified().ok()?, file.len()))
}

/// What a take's level says about how it was recorded, when it says
/// anything: the words `vidni record` has for it.
pub fn advice(peak: f64) -> Option<&'static str> {
    if peak > -1.0 {
        Some("Too hot: lower the input gain or move back a little, then record again.")
    } else if peak < -20.0 {
        Some("Quiet: move closer or raise the input gain.")
    } else {
        None
    }
}

impl Studio {
    /// The studio of the project in `folder`.
    pub fn open(folder: &Path) -> Self {
        let mut studio = Self {
            folder: folder.to_path_buf(),
            input: None,
            speaker: None,
            session: vidni_core::release::now(),
            lines: Vec::new(),
            at: 0,
            doing: Doing::Nothing,
            note: String::new(),
            raws: Vec::new(),
            pick: None,
            trim_from: String::new(),
            trim_to: String::new(),
            played: None,
            playing: None,
            room: None,
            problem: None,
            read: Vec::new(),
            cache: HashMap::new(),
        };
        studio.load();
        // Where the last session stopped: the first line with no take of
        // its present words.
        studio.at = studio
            .lines
            .iter()
            .position(|line| line.take != Take::Recorded)
            .unwrap_or(0);
        studio.load_takes();
        studio
    }

    /// Records from `input` from here on, in the place of the microphone.
    pub fn set_input(&mut self, input: Box<dyn Input>) {
        self.input = Some(input);
    }

    /// Plays through `speaker` from here on, in the place of the system's
    /// output; or, with the reason why there is none, plays nothing.
    pub fn set_speaker(&mut self, speaker: Result<Speaker, String>) {
        self.speaker = Some(speaker);
    }

    /// The line in the prompter.
    pub fn line(&self) -> Option<&Line> {
        self.lines.get(self.at)
    }

    /// What the meter reads, while something is recorded.
    pub fn recording(&self) -> Option<Level> {
        match &self.doing {
            Doing::Recording { recording, .. } => Some(recording.level()),
            _ => None,
        }
    }

    /// The new take that waits to be kept or discarded.
    pub fn reviewing(&self) -> Option<&Candidate> {
        match &self.doing {
            Doing::Review { candidate, .. } => Some(candidate),
            _ => None,
        }
    }

    /// How many lines have a take of their present words.
    pub fn recorded(&self) -> usize {
        self.lines
            .iter()
            .filter(|line| line.take == Take::Recorded)
            .count()
    }

    /// When each file that the studio shows was last written.
    fn stamp(&self) -> Vec<Option<(SystemTime, u64)>> {
        let takes = self.folder.join(takes::DIR);
        let mut stamp = vec![
            written(&validate::project_file(&self.folder)),
            written(&self.folder.join(voices::LOCK)),
            written(&takes),
            written(&room::path(&self.folder)),
        ];
        for line in &self.lines {
            let folder = line.kept.parent().map(Path::to_path_buf);
            stamp.push(folder.as_deref().and_then(written));
        }
        if let Some(line) = self.line() {
            if let Some(folder) = line.kept.parent() {
                stamp.push(written(&folder.join("raw")));
            }
            // A pick's record is written over in place, which its folder's
            // time doesn't show.
            stamp.push(written(&line.kept));
            stamp.push(written(&takes::record_path(&line.kept)));
        }
        stamp
    }

    /// Reads the project, its lines and where their takes stand.
    fn load(&mut self) {
        // The times are taken before the files are read, so that a change
        // made while they are read is seen at the next look.
        self.read = self.stamp();
        let before = self.line().map(|line| line.id.clone());
        let file = validate::project_file(&self.folder);
        let dir = file.parent().unwrap_or(&self.folder).to_path_buf();
        let project = std::fs::read_to_string(&file)
            .map_err(|e| format!("{}: {e}", file.display()))
            .and_then(|text| {
                validate::check(&text, Some(&dir)).map_err(|problems| {
                    let first = problems
                        .first()
                        .map(|first| format!(" The first: {first}"))
                        .unwrap_or_default();
                    format!(
                        "{} has problems, which the Script tab lists. Its lines are recorded \
                         once it is valid.{first}",
                        file.display()
                    )
                })
            });
        let project = match project {
            Ok(project) => project,
            Err(why) => {
                self.problem = Some(why);
                self.lines.clear();
                self.raws.clear();
                self.pick = None;
                self.at = 0;
                return;
            }
        };
        let cuts: Vec<&vidni_core::project::Cut> = project.cuts.iter().collect();
        // The draft voice's pace, where its pack is at hand.
        let pack = voices::find(&project.voice.draft, voices::store().as_deref())
            .ok()
            .flatten();
        let rows = match sheet::sheet(&project, &dir, &cuts, pack.as_ref()) {
            Ok(rows) => rows,
            Err(why) => {
                self.problem = Some(why);
                self.lines.clear();
                self.raws.clear();
                self.pick = None;
                self.at = 0;
                return;
            }
        };
        let stored = takes::status(&project, &dir);
        self.problem = None;
        self.lines = rows
            .into_iter()
            .filter_map(|row| {
                let stored = stored.iter().find(|take| take.id == row.id)?;
                let cuts = cuts
                    .iter()
                    .filter(|cut| cut.line_ids(&project).contains(&row.id))
                    .map(|cut| cut.id.clone())
                    .collect();
                Some(Line {
                    hash: stored.hash.clone(),
                    kept: stored.path.clone(),
                    id: row.id,
                    label: row.label,
                    text: row.text,
                    pace: row.pace,
                    take: row.take,
                    cuts,
                })
            })
            .collect();
        // The prompter stays on its line, wherever that is now.
        self.at = before
            .and_then(|id| self.lines.iter().position(|line| line.id == id))
            .unwrap_or(self.at.min(self.lines.len().saturating_sub(1)));
        let tone = room::path(&self.folder);
        self.room = take::read(&tone)
            .ok()
            .map(|(samples, _)| clean::rms_db(&samples));
        self.load_takes();
    }

    /// Reads the raw takes of the line in the prompter, and which of them
    /// is kept.
    fn load_takes(&mut self) {
        let read = self.stamp();
        let Some(line) = self.line().cloned() else {
            self.raws.clear();
            self.pick = None;
            return;
        };
        let dir = self.folder.clone();
        let mut raws = Vec::new();
        for (number, path) in takes::raws(&dir, &line.id, &line.hash) {
            let length = std::fs::metadata(&path).map(|file| file.len()).unwrap_or(0);
            if let Some((held, raw)) = self.cache.get(&path)
                && *held == length
            {
                raws.push(raw.clone());
                continue;
            }
            // A take that is being written, here or in a terminal, has no
            // length yet: it is listed once it is whole.
            let Ok((samples, rate)) = take::read(&path) else {
                continue;
            };
            if samples.is_empty() {
                continue;
            }
            let session = takes::session(&path).ok().flatten();
            // A take whose session isn't noted yet may be noted in a moment:
            // it is read again until it is.
            let whole = session.is_some();
            let recorded = match session {
                Some(s) => format!("recorded {} from {}", s.recorded, s.microphone),
                None => "recorded before sessions were noted".to_string(),
            };
            let raw = Raw {
                number,
                seconds: samples.len() as f64 / f64::from(rate.max(1)),
                peak: clean::peak_db(&samples),
                recorded,
                rate,
                wave: take::envelope(&samples, rate),
                path: path.clone(),
            };
            if whole {
                self.cache.insert(path, (length, raw.clone()));
            }
            raws.push(raw);
        }
        self.raws = raws;
        let pick = takes::pick(&line.kept)
            .ok()
            .flatten()
            .filter(|_| line.kept.is_file());
        if pick != self.pick {
            // The form holds the ends the kept take has.
            let rate = pick
                .as_ref()
                .and_then(|pick| self.raws.iter().find(|raw| raw.number == pick.raw))
                .map_or(mic::RATE, |raw| raw.rate);
            let at = |sample: usize| format!("{:.3}", sample as f64 / f64::from(rate.max(1)));
            (self.trim_from, self.trim_to) = match &pick {
                Some(pick) => (at(pick.from), at(pick.to)),
                None => (String::new(), String::new()),
            };
            self.pick = pick;
        }
        // The lines' own times are of when they were read; the rest is of
        // this read.
        if self.read.len() == read.len() {
            self.read = read;
        }
    }

    /// Takes in what has changed on disk: a take kept in a terminal, a
    /// line edited in the Script tab.
    pub fn refresh(&mut self) {
        if matches!(self.doing, Doing::Nothing) && self.stamp() != self.read {
            self.load();
        }
    }

    /// Ends whatever is under way, keeping nothing: a recording is
    /// cancelled, a new take is discarded, a take stops playing.
    pub fn settle(&mut self) {
        self.playing = None;
        match std::mem::replace(&mut self.doing, Doing::Nothing) {
            Doing::Nothing => {}
            Doing::Recording { recording, .. } => {
                let _ = recording.cancel();
            }
            Doing::Review { candidate, .. } => {
                let _ = candidate.discard();
            }
        }
    }

    /// Puts line number `at` in the prompter.
    pub fn select(&mut self, at: usize) {
        if at >= self.lines.len() || at == self.at {
            return;
        }
        self.settle();
        self.at = at;
        self.note.clear();
        self.pick = None;
        self.load_takes();
    }

    /// The input to record from: the one that was set, or the microphone
    /// `$VIDNI_MIC` names, or the default one. `$VIDNI_RECORD_INPUT` puts
    /// a made-up input in its place, as it does for `vidni record`.
    fn input(&mut self) -> Result<&dyn Input, String> {
        if self.input.is_none() {
            let named = |name: &str| std::env::var(name).ok().filter(|value| !value.is_empty());
            let input: Box<dyn Input> = match named("VIDNI_RECORD_INPUT") {
                Some(words) => Box::new(input::made_up(&words, mic::RATE)?),
                None => Box::new(Mic::find(named("VIDNI_MIC").as_deref())?),
            };
            self.input = Some(input);
        }
        Ok(self.input.as_deref().expect("it was just found"))
    }

    /// Starts recording into `raw`.
    fn start(&mut self, raw: &Path, of: Of) {
        self.settle();
        let recording = self.input().and_then(|input| take::record(input, raw));
        match recording {
            Ok(recording) => {
                self.note.clear();
                self.doing = Doing::Recording { recording, of };
            }
            Err(why) => {
                self.note = format!(
                    "{} Check the mic with `vidni record --test`.",
                    sentence(&why)
                );
            }
        }
    }

    /// Starts a take of the line in the prompter.
    pub fn record(&mut self) {
        let Some(line) = self.line().cloned() else {
            return;
        };
        let raw = takes::next_raw(&self.folder, &line.id, &line.hash);
        self.start(&raw, Of::Line(line.id));
    }

    /// Starts recording the room, which stops on its own.
    pub fn record_room(&mut self) {
        let raw = room::path(&self.folder).with_file_name("room.raw");
        // What an interrupted recording of the room left behind.
        let _ = std::fs::remove_file(&raw);
        self.start(&raw, Of::Room);
    }

    /// Cancels the recording: nothing is saved.
    pub fn cancel(&mut self) {
        if matches!(self.doing, Doing::Recording { .. }) {
            self.settle();
            self.note = "Cancelled; nothing was saved.".to_string();
        }
    }

    /// Stops the recording. A take is checked and trimmed, and waits to
    /// be kept or discarded; the room is measured and kept.
    pub fn stop(&mut self) {
        let Doing::Recording { recording, of } = std::mem::replace(&mut self.doing, Doing::Nothing)
        else {
            return;
        };
        let (name, rate) = match &self.input {
            Some(input) => (input.name(), input.rate()),
            None => (String::new(), mic::RATE),
        };
        let raw = match recording.stop() {
            Ok(raw) => raw,
            Err(why) => {
                self.note = format!(
                    "{} Check the mic with `vidni record --test`.",
                    sentence(&why)
                );
                return;
            }
        };
        let id = match of {
            Of::Line(id) => id,
            Of::Room => {
                self.note = match room::keep(&raw, &self.folder) {
                    Ok(tone) if tone.noisy() => format!(
                        "Room tone kept: {:.0} dB. The room or the mic is noisy. A quieter \
                         room, soft furnishings or a closer mic help.",
                        tone.level
                    ),
                    Ok(tone) => format!("Room tone kept: {:.0} dB.", tone.level),
                    Err(why) => sentence(&why),
                };
                self.load();
                return;
            }
        };
        let Some(kept) = self
            .lines
            .iter()
            .find(|line| line.id == id)
            .map(|line| line.kept.clone())
        else {
            return;
        };
        // What the take was recorded with goes beside it.
        let noted = takes::write_session(
            &raw,
            &takes::Session {
                session: self.session.clone(),
                recorded: vidni_core::release::now(),
                microphone: name,
                rate,
            },
        );
        let candidate = noted.and_then(|()| take::candidate(&raw, &kept));
        match candidate {
            Ok(Ok(candidate)) => {
                let wave = take::read(&candidate.path)
                    .map(|(samples, rate)| take::envelope(&samples, rate))
                    .unwrap_or_default();
                self.note.clear();
                self.doing = Doing::Review { candidate, wave };
            }
            Ok(Err(refused)) => self.note = sentence(&refused.to_string()),
            Err(why) => self.note = sentence(&why),
        }
        // The raw take is listed, kept or not.
        self.load_takes();
    }

    /// Keeps the new take as the line's take, and goes on to the next
    /// line that has none.
    pub fn keep(&mut self) {
        let Doing::Review { candidate, .. } = std::mem::replace(&mut self.doing, Doing::Nothing)
        else {
            return;
        };
        self.playing = None;
        if !candidate.path.is_file() {
            // Only another recorder on this project takes it away.
            self.note = "The new take is gone: `vidni record` was started on this project \
                         and cleared it. Its raw take is kept, and is in the list below: Use \
                         take makes it the line's take. Record in one place at a time."
                .to_string();
            self.load();
            return;
        }
        match candidate.keep() {
            Ok(_) => {
                let kept = self.line().map(|line| line.id.clone()).unwrap_or_default();
                self.load();
                if self.lines.is_empty() {
                    // The project stopped being readable meanwhile. The take
                    // is kept all the same.
                    return;
                }
                let next = (self.at + 1..self.lines.len())
                    .chain(0..self.at)
                    .find(|at| self.lines[*at].take != Take::Recorded);
                match next {
                    Some(next) => {
                        self.select(next);
                        self.note = format!("Kept the take of {kept}.");
                    }
                    None => {
                        self.note = format!(
                            "Kept the take of {kept}. Every line has its take; `vidni final` \
                             builds the cuts from them."
                        );
                    }
                }
            }
            Err(why) => self.note = sentence(&why),
        }
    }

    /// Discards the new take: the line is as it was.
    pub fn discard(&mut self) {
        if matches!(self.doing, Doing::Review { .. }) {
            self.settle();
            self.note = "Discarded; the line is as it was.".to_string();
            self.load_takes();
        }
    }

    /// Plays the take at `path` from its start, in the place of whatever
    /// plays.
    pub fn play(&mut self, path: &Path) {
        self.playing = None;
        self.played = Some(path.to_path_buf());
        let wanted = std::env::var("VIDNI_SPEAKER")
            .ok()
            .filter(|v| !v.is_empty());
        let speaker = self
            .speaker
            .get_or_insert_with(|| Speaker::find(wanted.as_deref()));
        let playing = match speaker {
            Ok(speaker) => speaker.play(path),
            Err(why) => Err(why.clone()),
        };
        match playing {
            Ok(playing) => self.playing = Some(playing),
            Err(why) => self.note = sentence(&why),
        }
    }

    /// Runs `vidni takes` on the line in the prompter, and says what it
    /// said.
    fn retake(&mut self, pick: Option<u32>, trim: Option<String>) {
        let Some(line) = self.line().map(|line| line.id.clone()) else {
            return;
        };
        self.settle();
        let action = Action::Retake {
            project: self.folder.clone(),
            line,
            pick,
            trim,
        };
        let (code, said) = jobs::now(&action.args());
        self.note = said
            .into_iter()
            .map(|(_, line)| line)
            .collect::<Vec<_>>()
            .join("\n");
        // What was done shows in the form; what was refused leaves the
        // typing as it is.
        if code == 0 {
            self.pick = None;
        }
        self.load();
    }

    /// Makes raw take `number` the line's kept take, trimmed where vidni
    /// finds the words.
    pub fn use_take(&mut self, number: u32) {
        self.retake(Some(number), None);
    }

    /// Cuts the kept take from `from` to `to` seconds of its raw take.
    pub fn trim(&mut self, from: &str, to: &str) {
        self.retake(None, Some(format!("{}:{}", from.trim(), to.trim())));
    }

    /// Draws the studio.
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        self.refresh();
        self.tick(ui.ctx());
        ui.add_space(8.0);
        ui.heading("Recording");
        if let Some(problem) = &self.problem {
            ui.colored_label(ui.visuals().error_fg_color, problem);
            return;
        }
        if self.lines.is_empty() {
            ui.label("There is no line to record.");
            return;
        }
        ui.label(format!(
            "{} of {} lines recorded",
            self.recorded(),
            self.lines.len()
        ));
        let busy = !matches!(self.doing, Doing::Nothing);
        self.room_ui(ui, busy);
        ui.separator();
        self.prompter(ui);
        if !self.note.is_empty() {
            ui.add_space(4.0);
            for line in self.note.clone().lines() {
                ui.label(line);
            }
        }
        ui.separator();
        self.takes_ui(ui, busy);
        ui.separator();
        self.lines_ui(ui);
    }

    /// What goes on by itself: a recording is watched, the room's
    /// recording stops when it is long enough, a take that has played to
    /// its end is let go.
    fn tick(&mut self, ctx: &egui::Context) {
        if self.playing.as_ref().is_some_and(Playing::finished) {
            self.playing = None;
        }
        if self.playing.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
        let Doing::Recording { recording, of } = &self.doing else {
            return;
        };
        // The meter and the waveform move while the take is recorded.
        ctx.request_repaint();
        if recording.failed().is_some() {
            // What was recorded until then is closed and stays as a raw
            // take, as in the terminal: the line's words may be in it.
            self.playing = None;
            let Doing::Recording { recording, of } =
                std::mem::replace(&mut self.doing, Doing::Nothing)
            else {
                return;
            };
            let why = match recording.stop() {
                Err(why) => why,
                Ok(raw) => format!("{}: the input stopped", raw.display()),
            };
            let room = matches!(of, Of::Room);
            if room {
                let _ = std::fs::remove_file(room::path(&self.folder).with_file_name("room.raw"));
            }
            self.note = format!(
                "{} {}Check the mic with `vidni record --test`.",
                sentence(&why),
                match room {
                    true => "",
                    false => "What was recorded until then is among the line's takes. ",
                }
            );
            self.load_takes();
            return;
        }
        if matches!(of, Of::Room) && recording.level().seconds >= room::SECONDS {
            self.stop();
        }
    }

    /// Room tone: what the project has, and a button to record it.
    fn room_ui(&mut self, ui: &mut egui::Ui, busy: bool) {
        ui.horizontal(|ui| {
            match self.room {
                Some(level) => ui.label(format!("Room tone: {level:.0} dB")),
                None => ui.label("Room tone: none yet"),
            };
            let button = egui::Button::new("Record room tone");
            if ui.add_enabled(!busy, button).clicked() {
                self.record_room();
            }
            ui.weak(format!(
                "Stay quiet for {:.0} seconds while the room is recorded.",
                room::SECONDS
            ));
        });
    }

    /// The prompter: the line to read, and its recording.
    fn prompter(&mut self, ui: &mut egui::Ui) {
        let Some(line) = self.line().cloned() else {
            return;
        };
        let state = match line.take {
            Take::Recorded => "recorded",
            Take::Stale => "recorded with other words",
            Take::Missing => "no take yet",
        };
        // A section may have no label, and a line may be in no cut.
        let mut about = vec![format!(
            "Line {} of {}  [{}]",
            self.at + 1,
            self.lines.len(),
            line.id
        )];
        if !line.label.is_empty() {
            about.push(line.label.clone());
        }
        if !line.cuts.is_empty() {
            about.push(format!("in {}", line.cuts.join(", ")));
        }
        about.push(state.to_string());
        ui.label(about.join(" · "));
        ui.add_space(4.0);
        ui.label(egui::RichText::new(&line.text).size(24.0).strong());
        ui.add_space(4.0);
        if let Some(pace) = line.pace {
            ui.weak(format!("The draft voice takes {pace:.1} s over it."));
        }
        match &self.doing {
            Doing::Nothing => {
                ui.horizontal(|ui| {
                    if ui.button("Record a take").clicked() {
                        self.record();
                    }
                    let (first, last) = (self.at == 0, self.at + 1 == self.lines.len());
                    if ui
                        .add_enabled(!first, egui::Button::new("Previous line"))
                        .clicked()
                    {
                        self.select(self.at - 1);
                    }
                    if ui
                        .add_enabled(!last, egui::Button::new("Next line"))
                        .clicked()
                    {
                        self.select(self.at + 1);
                    }
                    let open = (self.at + 1..self.lines.len())
                        .chain(0..self.at)
                        .find(|at| self.lines[*at].take != Take::Recorded);
                    if ui
                        .add_enabled(open.is_some(), egui::Button::new("Next line to record"))
                        .clicked()
                        && let Some(open) = open
                    {
                        self.select(open);
                    }
                });
            }
            Doing::Recording { recording, of } => {
                let level = recording.level();
                let wave = recording.envelope();
                let room = matches!(of, Of::Room);
                let what = match (room, line.pace) {
                    (true, _) => format!(
                        "Recording the room: {:.1} s of {:.0} s",
                        level.seconds,
                        room::SECONDS
                    ),
                    (false, Some(pace)) => {
                        format!("Recording: {:.1} s of ~{pace:.0} s", level.seconds)
                    }
                    (false, None) => format!("Recording: {:.1} s", level.seconds),
                };
                ui.colored_label(ui.visuals().error_fg_color, what);
                meter(ui, level.now);
                waveform(ui, "Waveform of the recording", &wave, None);
                let (mut stop, mut cancel) = (false, false);
                ui.horizontal(|ui| {
                    stop = ui.add_enabled(!room, egui::Button::new("Stop")).clicked();
                    cancel = ui.button("Cancel").clicked();
                });
                if stop {
                    self.stop();
                } else if cancel {
                    self.cancel();
                }
            }
            Doing::Review { candidate, wave } => {
                ui.strong("New take");
                ui.label(format!(
                    "{:.1} s · peak {:.1} dB",
                    candidate.seconds, candidate.peak
                ));
                if let Some(advice) = advice(candidate.peak) {
                    ui.colored_label(ui.visuals().warn_fg_color, advice);
                }
                waveform(ui, "Waveform of the new take", wave, None);
                let (new, kept) = (candidate.path.clone(), candidate.kept.clone());
                let has_kept = kept.is_file();
                let mut pressed = None;
                ui.horizontal(|ui| {
                    for (name, enabled) in [
                        ("Keep", true),
                        ("Play A: the kept take", has_kept),
                        ("Play B: the new take", true),
                        ("Record again", true),
                        ("Discard", true),
                    ] {
                        if ui.add_enabled(enabled, egui::Button::new(name)).clicked() {
                            pressed = Some(name);
                        }
                    }
                });
                match pressed {
                    Some("Keep") => self.keep(),
                    Some("Play A: the kept take") => self.play(&kept),
                    Some("Play B: the new take") => self.play(&new),
                    Some("Record again") => {
                        self.settle();
                        self.record();
                    }
                    Some("Discard") => self.discard(),
                    _ => {}
                }
            }
        }
    }

    /// The line's raw takes, the kept one among them, and another trim.
    fn takes_ui(&mut self, ui: &mut egui::Ui, busy: bool) {
        let Some(line) = self.line().cloned() else {
            return;
        };
        // Nothing is played while a take is recorded: it would be in the take.
        let recording = matches!(self.doing, Doing::Recording { .. });
        ui.strong(format!("Takes of {}", line.id));
        if self.raws.is_empty() {
            ui.label("No raw take of these words yet.");
            return;
        }
        let current = self.pick.as_ref().map(|pick| pick.raw);
        let mut pressed = None;
        for raw in &self.raws {
            ui.horizontal(|ui| {
                let mark = match current == Some(raw.number) {
                    true => "kept",
                    false => "",
                };
                ui.monospace(format!(
                    "take {}  {:.2} s, peak {:.1} dB; {}  {mark}",
                    raw.number, raw.seconds, raw.peak, raw.recorded
                ));
                let play = egui::Button::new(format!("Play take {}", raw.number));
                if ui.add_enabled(!recording, play).clicked() {
                    pressed = Some((raw.number, false));
                }
                let button = egui::Button::new(format!("Use take {}", raw.number));
                if ui.add_enabled(!busy, button).clicked() {
                    pressed = Some((raw.number, true));
                }
            });
        }
        match pressed {
            Some((number, true)) => self.use_take(number),
            Some((number, false)) => {
                if let Some(path) = self
                    .raws
                    .iter()
                    .find(|raw| raw.number == number)
                    .map(|raw| raw.path.clone())
                {
                    self.play(&path);
                }
            }
            None => {}
        }
        // The kept take: which raw take it is, and where it was cut.
        let Some(pick) = self.pick.clone() else {
            if line.kept.is_file() {
                ui.label("The kept take has no record of the raw take it was cut from.");
                let play = egui::Button::new("Play the kept take");
                if ui.add_enabled(!recording, play).clicked() {
                    self.play(&line.kept);
                }
            }
            return;
        };
        let Some(raw) = self.raws.iter().find(|raw| raw.number == pick.raw).cloned() else {
            return;
        };
        ui.add_space(6.0);
        // The kept part, in seconds of the raw take and as parts of it.
        let rate = f64::from(raw.rate.max(1));
        let (from, to) = (pick.from as f64 / rate, pick.to as f64 / rate);
        let span = (from / raw.seconds.max(1e-9), to / raw.seconds.max(1e-9));
        ui.label(format!(
            "Kept: take {} from {from:.2} s to {to:.2} s ({})",
            pick.raw,
            match pick.trim {
                Trim::Auto => "where vidni found the words",
                Trim::Set => "where its ends were put",
            }
        ));
        waveform(
            ui,
            "Waveform of the kept take's raw take",
            &raw.wave,
            Some((span.0 as f32, span.1 as f32)),
        );
        let mut pressed = None;
        ui.horizontal(|ui| {
            let play = egui::Button::new("Play the kept take");
            if ui.add_enabled(!recording, play).clicked() {
                pressed = Some("play");
            }
            let label = ui.label("From (s)");
            ui.add(egui::TextEdit::singleline(&mut self.trim_from).desired_width(60.0))
                .labelled_by(label.id);
            let label = ui.label("To (s)");
            ui.add(egui::TextEdit::singleline(&mut self.trim_to).desired_width(60.0))
                .labelled_by(label.id);
            if ui.add_enabled(!busy, egui::Button::new("Trim")).clicked() {
                pressed = Some("trim");
            }
            let auto = egui::Button::new("Trim where vidni finds the words");
            if ui.add_enabled(!busy, auto).clicked() {
                pressed = Some("auto");
            }
        });
        match pressed {
            Some("play") => self.play(&line.kept),
            Some("trim") => {
                let (from, to) = (self.trim_from.clone(), self.trim_to.clone());
                self.trim(&from, &to);
            }
            Some("auto") => self.use_take(pick.raw),
            _ => {}
        }
    }

    /// Every line, to go to by a click.
    fn lines_ui(&mut self, ui: &mut egui::Ui) {
        ui.strong("Lines");
        let recording = matches!(self.doing, Doing::Recording { .. });
        let mut go = None;
        for (at, line) in self.lines.iter().enumerate() {
            let state = match line.take {
                Take::Recorded => "recorded",
                Take::Stale => "other words",
                Take::Missing => "no take",
            };
            let row = format!("[{}]  {state}  {}", line.id, line.text);
            // Another line is gone to once the recording is stopped or
            // cancelled, not by a stray click during it.
            let row = egui::Button::selectable(at == self.at, row);
            if ui.add_enabled(!recording, row).clicked() {
                go = Some(at);
            }
        }
        if let Some(at) = go {
            self.select(at);
        }
    }
}

impl Drop for Studio {
    fn drop(&mut self) {
        // A recording that nobody stopped, and a take nobody kept, leave
        // nothing behind.
        self.settle();
    }
}

/// The level meter: from -60 dB to full scale.
fn meter(ui: &mut egui::Ui, db: f64) {
    let part = ((db + 60.0) / 60.0).clamp(0.0, 1.0) as f32;
    let text = match db <= clean::FLOOR {
        true => "silence".to_string(),
        false => format!("{db:.0} dB"),
    };
    ui.add(
        egui::ProgressBar::new(part)
            .desired_width(360.0)
            .text(format!("Level: {text}")),
    );
}

/// A waveform: a bar for each 20 ms, as tall as its loudest sample. The
/// part between `span`'s ends, given as parts of the whole, is marked as
/// the part that is kept.
fn waveform(ui: &mut egui::Ui, name: &str, bars: &[f32], span: Option<(f32, f32)>) {
    let width = ui.available_width().clamp(120.0, 720.0);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 56.0), egui::Sense::hover());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Image, true, name));
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 2.0, ui.visuals().extreme_bg_color);
    if let Some((from, to)) = span {
        let kept = egui::Rect::from_min_max(
            egui::pos2(
                rect.left() + rect.width() * from.clamp(0.0, 1.0),
                rect.top(),
            ),
            egui::pos2(
                rect.left() + rect.width() * to.clamp(0.0, 1.0),
                rect.bottom(),
            ),
        );
        painter.rect_filled(
            kept,
            0.0,
            ui.visuals().selection.bg_fill.gamma_multiply(0.5),
        );
    }
    if bars.is_empty() {
        return;
    }
    // One stroke a pixel at most: the loudest of the bars that share it.
    let columns = (rect.width() as usize).clamp(1, bars.len());
    let stroke = egui::Stroke::new(1.0, ui.visuals().text_color());
    for column in 0..columns {
        let from = column * bars.len() / columns;
        let to = ((column + 1) * bars.len() / columns).max(from + 1);
        let peak = bars[from..to.min(bars.len())]
            .iter()
            .fold(0.0f32, |a, b| a.max(*b));
        let x = rect.left() + (column as f32 + 0.5) * rect.width() / columns as f32;
        let half = (peak * rect.height() / 2.0).max(0.5);
        painter.line_segment(
            [
                egui::pos2(x, rect.center().y - half),
                egui::pos2(x, rect.center().y + half),
            ],
            stroke,
        );
    }
}
