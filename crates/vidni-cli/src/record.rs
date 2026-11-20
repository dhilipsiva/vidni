//! `vidni record`: the terminal recorder (TODO.md, P4-VIDNI-02). One line
//! at a time, read from the screen, with single keys and no Enter:
//!
//! ```text
//! ready      space record · p play the saved take · s play the section ·
//!            ← → move between lines · q quit
//! recording  space or enter stop · esc cancel
//! review     enter keep and go on · p play · space record again ·
//!            esc discard · q keep and quit
//! ```
//!
//! While a take is recorded, a meter shows its level and a timer runs
//! beside the pace of the draft voice. A take that is too short or has no
//! speech in it is discarded with a message, and the line stays. A take
//! that passes is trimmed, measured and, where the listening check is at
//! hand, heard, so that a misread line shows at once. Nothing takes a
//! line's kept take's place until the new one is kept.
//!
//! The session picks up where the last one stopped: at the first line with
//! no take of its present words. A line that several cuts use is recorded
//! once.

use crate::build::{cant, chosen};
use crate::{Loaded, count, load};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal;
use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use vidni_core::project::Cut;
use vidni_core::{takes, voices};
use vidni_media::hear::{self, Heard, Listener, WORTH_A_LISTEN};
use vidni_media::sheet::{self, Row, Take};
use vidni_record::input::{self, Input};
use vidni_record::mic::{self, Mic};
use vidni_record::play::Speaker;
use vidni_record::take::Candidate;
use vidni_record::{room, take};

/// After the screen changes, keys are ignored for this long: a double
/// press, or a key held down, isn't an answer to what has just appeared.
const SETTLE: Duration = Duration::from_millis(400);

const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const GREEN: &str = "\x1b[32m";
const AMBER: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
const RESET: &str = "\x1b[0m";

const READY: &str =
    "space record · p play the saved take · s play the section · ← → lines · q quit";
const REVIEW: &str =
    "enter keep and go on · p play · space record again · esc discard · q keep and quit";

/// Whether a recording session has the terminal, and whether it was asked
/// to stop.
static RUNNING: AtomicBool = AtomicBool::new(false);
static STOP: AtomicBool = AtomicBool::new(false);

/// Asks a recording session to stop, so that it can put the terminal back
/// as it was. False when none is running: the caller, a signal handler,
/// should then exit itself.
pub fn interrupt() -> bool {
    RUNNING.load(Ordering::SeqCst) && !STOP.swap(true, Ordering::SeqCst)
}

/// A key the recorder knows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Key {
    Space,
    Enter,
    Esc,
    Back,
    Forward,
    Play,
    Again,
    Section,
    Quit,
    /// Ctrl-C, or a signal that asks vidni to stop: nothing is kept that
    /// wasn't kept already.
    Interrupt,
    Other,
}

/// The terminal while the recorder has it: keys arrive one at a time, and
/// the cursor is hidden. Dropping it puts the terminal back.
struct Screen;

impl Screen {
    fn open() -> Result<Self, String> {
        terminal::enable_raw_mode().map_err(|e| format!("the terminal: {e}"))?;
        RUNNING.store(true, Ordering::SeqCst);
        show("\x1b[?25l");
        Ok(Self)
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        RUNNING.store(false, Ordering::SeqCst);
        show("\x1b[?25h\r\n");
    }
}

/// Writes to the terminal. A raw terminal wants a carriage return with
/// each line feed.
fn show(text: &str) {
    let mut out = std::io::stdout();
    let _ = out.write_all(
        text.replace('\n', "\r\n")
            .replace("\r\r\n", "\r\n")
            .as_bytes(),
    );
    let _ = out.flush();
}

/// Replaces the status line, the last line of the screen.
fn status(text: &str) {
    show(&format!("\r\x1b[2K{text}"));
}

/// The next key, waiting for `wait` at most, or for as long as it takes.
/// A session that was asked to stop gets `Interrupt`.
fn key(wait: Option<Duration>) -> Result<Option<Key>, String> {
    let deadline = wait.map(|wait| Instant::now() + wait);
    loop {
        if STOP.load(Ordering::SeqCst) {
            return Ok(Some(Key::Interrupt));
        }
        let slice = match deadline {
            Some(deadline) => match deadline.checked_duration_since(Instant::now()) {
                Some(left) => left.min(Duration::from_millis(50)),
                None => return Ok(None),
            },
            None => Duration::from_millis(50),
        };
        if !event::poll(slice).map_err(|e| format!("the terminal: {e}"))? {
            continue;
        }
        let Event::Key(pressed) = event::read().map_err(|e| format!("the terminal: {e}"))? else {
            continue;
        };
        if pressed.kind != KeyEventKind::Press {
            continue;
        }
        let control = pressed.modifiers.contains(KeyModifiers::CONTROL);
        return Ok(Some(match pressed.code {
            KeyCode::Char('c') if control => {
                STOP.store(true, Ordering::SeqCst);
                Key::Interrupt
            }
            KeyCode::Char(' ') => Key::Space,
            KeyCode::Enter => Key::Enter,
            KeyCode::Esc => Key::Esc,
            KeyCode::Left | KeyCode::Up => Key::Back,
            KeyCode::Right | KeyCode::Down => Key::Forward,
            KeyCode::Char(c) => match c.to_ascii_lowercase() {
                'b' => Key::Back,
                'n' => Key::Forward,
                'p' => Key::Play,
                'r' => Key::Again,
                's' => Key::Section,
                'q' => Key::Quit,
                _ => Key::Other,
            },
            _ => Key::Other,
        }));
    }
}

/// Forgets the keys pressed so far.
fn forget_keys() -> Result<(), String> {
    while event::poll(Duration::ZERO).map_err(|e| format!("the terminal: {e}"))? {
        event::read().map_err(|e| format!("the terminal: {e}"))?;
    }
    Ok(())
}

/// Lets the screen settle: whatever is pressed in the next moment is
/// ignored.
fn settle() -> Result<(), String> {
    std::thread::sleep(SETTLE);
    forget_keys()
}

/// A level as a bar: green, amber near the top, red at it.
fn meter(db: f64) -> String {
    const WIDTH: usize = 24;
    let filled = (((db + 60.0) / 60.0 * WIDTH as f64).max(0.0) as usize).min(WIDTH);
    let colour = match db {
        db if db > -3.0 => RED,
        db if db > -12.0 => AMBER,
        _ => GREEN,
    };
    format!(
        "{colour}{}{RESET}{DIM}{}{RESET}",
        "█".repeat(filled),
        "·".repeat(WIDTH - filled)
    )
}

/// `text` in lines of `width` characters at most, each after `indent`.
fn wrapped(text: &str, width: usize, indent: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        match lines.last_mut() {
            Some(line) if line.chars().count() + 1 + word.chars().count() <= width => {
                line.push(' ');
                line.push_str(word);
            }
            _ => lines.push(word.to_string()),
        }
    }
    lines
        .iter()
        .map(|line| format!("{indent}{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// A sentence from a clause: its first letter raised, and a full stop.
fn sentence(clause: &str) -> String {
    let mut letters = clause.chars();
    match letters.next() {
        Some(first) => format!("{}{}.", first.to_uppercase(), letters.as_str()),
        None => String::new(),
    }
}

/// One line of the session: its row of the recording sheet, and where its
/// takes are.
struct Line {
    row: Row,
    hash: String,
    kept: PathBuf,
    /// The cuts that use it.
    cuts: Vec<String>,
}

/// What the session records and plays with.
struct Studio {
    input: Box<dyn Input>,
    /// Where takes are played: found when one is first played.
    speaker: Option<Result<Speaker, String>>,
    /// The listening check, where it is at hand and knows the language.
    listener: Option<Listener>,
    /// The session's name: the moment it began.
    session: String,
}

impl Studio {
    /// The input is the microphone `$VIDNI_MIC` names, or the default one.
    /// `$VIDNI_RECORD_INPUT` puts a made-up input in its place, for tests
    /// and demonstrations.
    fn open(language: &str) -> Result<Self, String> {
        let named = |name: &str| std::env::var(name).ok().filter(|value| !value.is_empty());
        let input: Box<dyn Input> = match named("VIDNI_RECORD_INPUT") {
            Some(words) => Box::new(input::made_up(&words, mic::RATE)?),
            None => Box::new(Mic::find(named("VIDNI_MIC").as_deref())?),
        };
        let listener = hear::understands(language)
            .then(|| Listener::find().ok())
            .flatten();
        Ok(Self {
            input,
            speaker: None,
            listener,
            session: vidni_core::release::now(),
        })
    }

    /// Plays the takes at `paths`, one after another. Any key stops them.
    fn play(&mut self, paths: &[PathBuf]) -> Result<Option<String>, String> {
        let wanted = std::env::var("VIDNI_SPEAKER")
            .ok()
            .filter(|v| !v.is_empty());
        let speaker = self
            .speaker
            .get_or_insert_with(|| Speaker::find(wanted.as_deref()));
        let speaker = match speaker {
            Ok(speaker) => speaker,
            Err(e) => return Ok(Some(format!("{AMBER}{}{RESET}", sentence(e)))),
        };
        status(&format!("  {DIM}▶ playing · any key stops{RESET}"));
        forget_keys()?;
        for path in paths {
            let playing = match speaker.play(path) {
                Ok(playing) => playing,
                Err(e) => return Ok(Some(format!("{AMBER}{}{RESET}", sentence(&e)))),
            };
            while !playing.finished() {
                if key(Some(Duration::from_millis(50)))?.is_some() {
                    status("");
                    return Ok(None);
                }
            }
        }
        status("");
        Ok(None)
    }
}

/// How a recording ended.
enum Captured {
    /// It was stopped: its raw take.
    Stopped(PathBuf),
    Cancelled,
    /// It couldn't go on, and why.
    Failed(String),
}

/// Records into `raw` until space or enter, or for `lasting` seconds. Esc
/// cancels, and leaves no file.
fn capture(
    studio: &Studio,
    raw: &Path,
    pace: Option<f64>,
    lasting: Option<f64>,
    label: &str,
) -> Result<Captured, String> {
    let recording = match take::record(studio.input.as_ref(), raw) {
        Ok(recording) => recording,
        Err(e) => return Ok(Captured::Failed(e)),
    };
    let started = Instant::now();
    forget_keys()?;
    let cancelled = loop {
        if recording.failed().is_some() {
            break false;
        }
        let level = recording.level();
        if lasting.is_some_and(|lasting| level.seconds >= lasting) {
            break false;
        }
        let pace = pace.map_or(String::new(), |pace| format!(" of ~{pace:.0} s"));
        let hint = match lasting {
            Some(_) => "esc cancel",
            None => "space stop · esc cancel",
        };
        status(&format!(
            "  {RED}● {label}{RESET} {:4.1} s{pace}  {}  {DIM}{hint}{RESET}",
            level.seconds,
            meter(level.now)
        ));
        let pressed = key(Some(Duration::from_millis(100)))?;
        if pressed == Some(Key::Interrupt) {
            break true;
        }
        // A double press of the key that started the take doesn't stop it.
        if started.elapsed() < SETTLE {
            continue;
        }
        match pressed {
            Some(Key::Esc | Key::Quit) => break true,
            Some(Key::Space | Key::Enter) if lasting.is_none() => break false,
            _ => {}
        }
    };
    let cancelled = cancelled || STOP.load(Ordering::SeqCst);
    status("");
    if cancelled {
        recording.cancel()?;
        return Ok(Captured::Cancelled);
    }
    Ok(match recording.stop() {
        Ok(raw) => Captured::Stopped(raw),
        Err(e) => Captured::Failed(e),
    })
}

/// The session: the lines it goes through, and where it is among them.
struct Session<'a> {
    loaded: &'a Loaded,
    cuts: Vec<&'a Cut>,
    pack: Option<voices::Pack>,
    /// The lines of the session, by ID, in script order.
    ids: Vec<String>,
    position: usize,
}

impl Session<'_> {
    /// The session's lines as they stand now.
    fn lines(&self) -> Result<Vec<Line>, String> {
        let Loaded { dir, project, .. } = self.loaded;
        let rows = sheet::sheet(project, dir, &self.cuts, self.pack.as_ref())?;
        let stored = takes::status(project, dir);
        Ok(rows
            .into_iter()
            .filter(|row| self.ids.contains(&row.id))
            .filter_map(|row| {
                let stored = stored.iter().find(|take| take.id == row.id)?;
                let cuts = self
                    .cuts
                    .iter()
                    .filter(|cut| cut.line_ids(project).contains(&row.id))
                    .map(|cut| cut.id.clone())
                    .collect();
                Some(Line {
                    hash: stored.hash.clone(),
                    kept: stored.path.clone(),
                    row,
                    cuts,
                })
            })
            .collect())
    }

    /// Draws the line the session is at, with `panel` under it and
    /// `legend` at the foot.
    fn draw(&self, lines: &[Line], panel: &[String], legend: &str) {
        let columns = terminal::size().map_or(90, |(columns, _)| usize::from(columns));
        let width = columns.saturating_sub(4).clamp(40, 80);
        let line = &lines[self.position];
        let n = |take: Take| lines.iter().filter(|l| l.row.take == take).count();
        let mut out = vec!["\x1b[2J\x1b[H".to_string()];
        let stale = match n(Take::Stale) {
            0 => String::new(),
            stale => format!(", {stale} of other words"),
        };
        out.push(format!(
            "{DIM}{} · {} of {} recorded{stale}{RESET}",
            self.loaded.project.meta.title,
            n(Take::Recorded),
            lines.len()
        ));
        let section = match line.row.label.is_empty() {
            true => &line.row.section,
            false => &line.row.label,
        };
        let pace = line.row.pace.map_or(String::new(), |pace| {
            format!(" · the draft voice took {pace:.0} s")
        });
        // Which cuts use the line is worth saying where the cuts differ.
        let project = &self.loaded.project;
        let mut theirs = self.cuts.iter().map(|cut| cut.line_ids(project));
        let first = theirs.next().unwrap_or_default();
        let shared = match theirs.any(|ids| ids != first) {
            true => format!(" · in {}", line.cuts.join(" and ")),
            false => String::new(),
        };
        out.push(format!(
            "{BOLD}Line {} of {}{RESET}  {DIM}[{}] · {section}{pace}{shared}{RESET}",
            self.position + 1,
            lines.len(),
            line.row.id
        ));
        out.push(format!("{DIM}{}{RESET}", "─".repeat(width)));
        if let Some(before) = self.position.checked_sub(1).map(|i| &lines[i]) {
            let text = wrapped(&before.row.text, width - 8, "        ");
            out.push(format!("{DIM}before  {}{RESET}", text.trim_start()));
        }
        out.push(String::new());
        out.push(format!(
            "{BOLD}{}{RESET}",
            wrapped(&line.row.text, width - 2, "  ")
        ));
        out.push(String::new());
        match line.row.take {
            Take::Recorded => {
                let seconds = take::seconds(&line.kept).unwrap_or_default();
                out.push(format!("  {GREEN}✓ saved take, {seconds:.1} s{RESET}"));
            }
            Take::Stale => out.push(format!(
                "  {AMBER}! its take is of other words: the line was changed after it was recorded{RESET}"
            )),
            Take::Missing => {}
        }
        out.extend(panel.iter().map(|line| format!("  {line}")));
        out.push(format!("{DIM}{}{RESET}", "─".repeat(width)));
        out.push(format!("  {legend}"));
        show(&(out.join("\n") + "\n"));
    }

    /// What a review says of a new take: its length and level, and what
    /// the listening check heard where it isn't the script.
    fn review(&self, studio: &Studio, line: &Line, candidate: &Candidate) -> Vec<String> {
        let mut lines = vec![format!(
            "{:.1} s · peak {:.1} dB",
            candidate.seconds, candidate.peak
        )];
        if candidate.peak > -1.0 {
            lines.push(format!(
                "{RED}Too hot: lower the input gain or move back a little, then record again.{RESET}"
            ));
        } else if candidate.peak < -20.0 {
            lines.push(format!(
                "{AMBER}Quiet: move closer or raise the input gain.{RESET}"
            ));
        }
        let heard = studio
            .listener
            .as_ref()
            .map(|listener| listener.hear(&candidate.path));
        let heard_as = &self.loaded.project.voice.heard_as;
        match heard {
            Some(Heard::Words(words)) => {
                let score = hear::closeness(&line.row.text, &words, heard_as);
                let colour = if score >= WORTH_A_LISTEN {
                    GREEN
                } else {
                    AMBER
                };
                lines[0] += &format!(" · matches the script {colour}{score:.2}{RESET}");
                if score < WORTH_A_LISTEN {
                    lines.push(format!("{AMBER}Heard: {words}{RESET}"));
                }
            }
            Some(Heard::Nothing) => lines.push(format!("{AMBER}Heard: (nothing){RESET}")),
            Some(Heard::Unavailable(_)) | None => {}
        }
        lines
    }

    /// Records takes of the line the session is at until one is kept or
    /// the creator backs out.
    fn record_line(&self, studio: &mut Studio) -> Result<Recorded, String> {
        let dir = &self.loaded.dir;
        loop {
            let lines = self.lines()?;
            let line = &lines[self.position];
            let hint = format!(
                "{DIM}Read when the meter starts moving; press space when you finish.{RESET}"
            );
            self.draw(&lines, &[hint], "");
            let raw = takes::next_raw(dir, &line.row.id, &line.hash);
            let raw = match capture(studio, &raw, line.row.pace, None, "REC")? {
                Captured::Stopped(raw) => raw,
                Captured::Cancelled => {
                    return Ok(Recorded::Back(format!(
                        "{DIM}Cancelled; nothing was saved.{RESET}"
                    )));
                }
                Captured::Failed(why) => {
                    return Ok(Recorded::Back(format!(
                        "{RED}{} Check the mic with `vidni record --test`.{RESET}",
                        sentence(&why)
                    )));
                }
            };
            // What the take was recorded with goes beside it.
            takes::write_session(
                &raw,
                &takes::Session {
                    session: studio.session.clone(),
                    recorded: vidni_core::release::now(),
                    microphone: studio.input.name(),
                    rate: studio.input.rate(),
                },
            )?;
            status(&format!("  {DIM}checking the take…{RESET}"));
            let candidate = match take::candidate(&raw, &line.kept)? {
                Ok(candidate) => candidate,
                Err(refused) => {
                    status("");
                    let why = sentence(&refused.to_string());
                    return Ok(Recorded::Back(format!("{AMBER}{why}{RESET}")));
                }
            };
            let mut panel = vec![format!("{BOLD}New take{RESET}")];
            panel.extend(self.review(studio, line, &candidate));
            let mut note = None;
            // The key that stopped the take, pressed twice, isn't an answer
            // to the review.
            self.draw(&lines, &panel, REVIEW);
            settle()?;
            loop {
                let mut shown = panel.clone();
                shown.extend(note.take());
                self.draw(&lines, &shown, REVIEW);
                match key(None)? {
                    Some(Key::Play) => note = studio.play(std::slice::from_ref(&candidate.path))?,
                    Some(Key::Enter) => {
                        candidate.keep()?;
                        return Ok(Recorded::Kept);
                    }
                    Some(Key::Quit) => {
                        candidate.keep()?;
                        return Ok(Recorded::KeptAndQuit);
                    }
                    Some(Key::Esc | Key::Interrupt) => {
                        candidate.discard()?;
                        return Ok(Recorded::Back(format!(
                            "{DIM}Discarded; the line is as it was.{RESET}"
                        )));
                    }
                    Some(Key::Space | Key::Again) => {
                        candidate.discard()?;
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
}

/// How recording a line ended.
enum Recorded {
    Kept,
    KeptAndQuit,
    /// Back at the ready screen, with a note for it.
    Back(String),
}

/// Removes what an interrupted session left behind: trimmed takes that
/// were never kept or discarded.
fn clear_candidates(lines: &[Line]) {
    for line in lines {
        let _ = std::fs::remove_file(take::candidate_path(&line.kept));
    }
}

/// The mic check: the room is recorded, then a line is read, and both are
/// measured.
fn mic_check(session: &Session, studio: &mut Studio) -> Result<(), String> {
    let dir = &session.loaded.dir;
    let folder = room::path(dir)
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default();
    let (room_raw, speech_raw) = (folder.join("room.raw"), folder.join("speech.raw"));
    let speech = folder.join("speech.wav");
    let words = session
        .lines()?
        .first()
        .map_or("This is a test of the microphone.".to_string(), |line| {
            line.row.text.clone()
        });
    let clear = || {
        for left in [&room_raw, &speech_raw, &take::candidate_path(&speech)] {
            let _ = std::fs::remove_file(left);
        }
    };
    loop {
        clear();
        show(&format!(
            "\x1b[2J\x1b[H{BOLD}Mic check{RESET} {DIM}· {}{RESET}\n\n  \
             Step 1 of 2: stay quiet for {:.0} seconds while the room is measured.\n\n",
            studio.input.name(),
            room::SECONDS
        ));
        settle()?;
        let recorded = match capture(studio, &room_raw, None, Some(room::SECONDS), "ROOM")? {
            Captured::Stopped(raw) => raw,
            Captured::Cancelled => break,
            Captured::Failed(why) => return Err(why),
        };
        let tone = room::keep(&recorded, dir)?;
        show(&format!(
            "  Step 2 of 2: press {BOLD}space{RESET}, read this line, press {BOLD}space{RESET}.\n\n{BOLD}{}{RESET}\n\n",
            wrapped(&words, 76, "  ")
        ));
        settle()?;
        loop {
            match key(None)? {
                Some(Key::Space | Key::Enter) => break,
                Some(Key::Esc | Key::Quit | Key::Interrupt) => {
                    clear();
                    return Ok(());
                }
                _ => {}
            }
        }
        let recorded = match capture(studio, &speech_raw, None, None, "REC")? {
            Captured::Stopped(raw) => raw,
            Captured::Cancelled => break,
            Captured::Failed(why) => return Err(why),
        };
        let candidate = take::candidate(&recorded, &speech)?;
        let mut report = vec![format!("  room noise {:.0} dB", tone.level)];
        match &candidate {
            Err(refused) => report.push(format!(
                "  {AMBER}{}{RESET}",
                sentence(&refused.to_string())
            )),
            Ok(candidate) => {
                report.push(format!("  speech peak {:.1} dB", candidate.peak));
                if tone.noisy() {
                    report.push(format!(
                        "  {AMBER}The room or the mic is noisy. A quieter room, soft furnishings or a closer mic help.{RESET}"
                    ));
                }
                if candidate.peak > -1.0 {
                    report.push(format!(
                        "  {RED}Speech is clipping. Lower the input gain.{RESET}"
                    ));
                } else if candidate.peak < -20.0 {
                    report.push(format!(
                        "  {AMBER}Speech is quiet. Move closer or raise the input gain.{RESET}"
                    ));
                } else if !tone.noisy() {
                    report.push(format!("  {GREEN}Levels look good.{RESET}"));
                }
            }
        }
        report.push(format!(
            "\n  {BOLD}p{RESET} play it back · {BOLD}space{RESET} run the check again · {BOLD}q{RESET} done\n"
        ));
        show(&report.join("\n"));
        settle()?;
        let again = loop {
            match (key(None)?, &candidate) {
                (Some(Key::Play), Ok(candidate)) => {
                    if let Some(note) = studio.play(std::slice::from_ref(&candidate.path))? {
                        show(&format!("  {note}\n"));
                    }
                }
                (Some(Key::Space | Key::Again), _) => break true,
                (Some(Key::Quit | Key::Esc | Key::Enter | Key::Interrupt), _) => break false,
                _ => {}
            }
        };
        if !again {
            break;
        }
    }
    clear();
    Ok(())
}

/// Goes through the session's lines until the creator quits or every one
/// is recorded.
fn run(session: &mut Session, studio: &mut Studio, redo: bool) -> Result<(), String> {
    let mut note = String::new();
    loop {
        let lines = session.lines()?;
        let panel: Vec<String> = Some(std::mem::take(&mut note))
            .filter(|note| !note.is_empty())
            .into_iter()
            .collect();
        session.draw(&lines, &panel, READY);
        let line = &lines[session.position];
        match key(None)? {
            Some(Key::Quit | Key::Esc | Key::Interrupt) => return Ok(()),
            Some(Key::Back) => session.position = session.position.saturating_sub(1),
            Some(Key::Forward) => {
                session.position = (session.position + 1).min(lines.len() - 1);
            }
            Some(Key::Play) if line.row.take == Take::Recorded => {
                note = studio
                    .play(std::slice::from_ref(&line.kept))?
                    .unwrap_or_default();
            }
            Some(Key::Section) => {
                let section: Vec<PathBuf> = lines
                    .iter()
                    .filter(|l| l.row.section == line.row.section && l.row.take == Take::Recorded)
                    .map(|l| l.kept.clone())
                    .collect();
                note = match section.is_empty() {
                    true => format!("{DIM}No line of this section is recorded yet.{RESET}"),
                    false => studio.play(&section)?.unwrap_or_default(),
                };
            }
            Some(Key::Space | Key::Enter) => match session.record_line(studio)? {
                Recorded::KeptAndQuit => return Ok(()),
                Recorded::Back(why) => note = why,
                Recorded::Kept => {
                    let lines = session.lines()?;
                    // On to the next line still to record; when redoing
                    // chosen lines, simply to the next one.
                    let open = |i: &usize| redo || lines[*i].row.take != Take::Recorded;
                    let later = (session.position + 1..lines.len()).find(open);
                    let earlier = (0..session.position)
                        .find(|i| lines[*i].row.take != Take::Recorded)
                        .filter(|_| !redo);
                    match (later, earlier) {
                        (Some(next), _) => session.position = next,
                        (None, Some(first)) => {
                            session.position = first;
                            note = format!("{AMBER}Back to the first line still to record.{RESET}");
                        }
                        (None, None) => return Ok(()),
                    }
                }
            },
            _ => {}
        }
    }
}

pub fn record(
    path: &Path,
    cuts: &[String],
    section: Option<&str>,
    from: Option<&str>,
    only: &[String],
    test: bool,
) -> ExitCode {
    let loaded = match load(path) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let Loaded { dir, project, .. } = &loaded;
    let cuts = match chosen(project, cuts) {
        Ok(cuts) => cuts,
        Err(code) => return code,
    };
    // What can't be done is said before the terminal is taken.
    let refused = |why: String| {
        tell!("vidni: {why}");
        ExitCode::FAILURE
    };
    if let Some(section) = section.filter(|id| !project.sections.contains_key(*id)) {
        return refused(format!("there is no section {section:?}"));
    }
    let in_cuts: Vec<String> = cuts.iter().flat_map(|cut| cut.line_ids(project)).collect();
    for id in only.iter().map(String::as_str).chain(from) {
        if !in_cuts.iter().any(|line| line == id) {
            return refused(format!("there is no line {id:?} in the cuts to record"));
        }
    }
    let ids: Vec<String> = project
        .lines
        .iter()
        .filter(|line| in_cuts.contains(&line.id))
        .filter(|line| section.is_none_or(|section| line.section == section))
        .filter(|line| only.is_empty() || only.contains(&line.id))
        .map(|line| line.id.clone())
        .collect();
    if ids.is_empty() {
        return refused("there is no line to record".to_string());
    }
    if let Some(from) = from.filter(|from| !ids.iter().any(|id| id == from)) {
        return refused(format!("line {from:?} isn't among the lines to record"));
    }
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        return cant("vidni record reads single keys: run it in a terminal".to_string());
    }
    let pack = match voices::find(&project.voice.draft, voices::store().as_deref()) {
        Ok(pack) => pack,
        Err(e) => return cant(e),
    };
    let mut studio = match Studio::open(&project.meta.language) {
        Ok(studio) => studio,
        Err(e) => return cant(e),
    };
    let mut session = Session {
        loaded: &loaded,
        cuts,
        pack,
        ids,
        position: 0,
    };
    let lines = match session.lines() {
        Ok(lines) => lines,
        Err(e) => return cant(e),
    };
    clear_candidates(&lines);
    // Where the last session stopped: the first line with no take of its
    // present words. Chosen lines are gone through from the first.
    session.position = match from {
        Some(from) => lines.iter().position(|l| l.row.id == from).unwrap_or(0),
        None if !only.is_empty() => 0,
        None => lines
            .iter()
            .position(|l| l.row.take != Take::Recorded)
            .unwrap_or(0),
    };
    let ended = {
        let screen = match Screen::open() {
            Ok(screen) => screen,
            Err(e) => return cant(e),
        };
        let ended = match test {
            true => mic_check(&session, &mut studio),
            false => run(&mut session, &mut studio, !only.is_empty()),
        };
        drop(screen);
        ended
    };
    if let Ok(lines) = session.lines() {
        clear_candidates(&lines);
    }
    if let Err(e) = ended {
        tell!("vidni: {e}");
        tell!("vidni: every take you kept is saved; `vidni record` goes on from there");
        return ExitCode::FAILURE;
    }
    if test {
        return ExitCode::SUCCESS;
    }
    // Every line of the cuts, not only the session's.
    let all = match sheet::sheet(project, dir, &session.cuts, None) {
        Ok(all) => all,
        Err(e) => return cant(e),
    };
    let open: Vec<&str> = all
        .iter()
        .filter(|row| row.take != Take::Recorded)
        .map(|row| row.id.as_str())
        .collect();
    say!(
        "{} of {} recorded, in {}",
        all.len() - open.len(),
        count(all.len(), "line"),
        dir.join(takes::DIR).display()
    );
    match open.len() {
        0 => say!("every line has its take; `vidni final` builds the cuts from them"),
        more => say!(
            "still to record: {}{}",
            open[..more.min(12)].join(", "),
            if more > 12 { " …" } else { "" }
        ),
    }
    if STOP.load(Ordering::SeqCst) {
        return ExitCode::from(130);
    }
    ExitCode::SUCCESS
}
