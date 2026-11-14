//! The recording studio (TODO.md, P7-VIDNI-06), with a made-up input: a
//! line is recorded, reviewed against its kept take and kept; another
//! take is picked and trimmed; a take is discarded, cancelled and refused;
//! the room is recorded. What the studio stores is what `vidni takes`
//! lists and what `vidni final` asks for.

use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use vidni_app::app::{App, Tab};
use vidni_app::jobs;
use vidni_app::studio::Studio;
use vidni_core::takes::Trim;
use vidni_record::input::Synthetic;
use vidni_record::take;

const RATE: u32 = 48000;

fn at(seconds: f64) -> usize {
    (seconds * f64::from(RATE)).round() as usize
}

/// A take as a person makes one: the start key's click, room noise,
/// `speech` seconds of a voice-like tone at `level`, room noise again and
/// the stop key's click; then silence, for as long as it is recorded.
fn spoken(speech: f64, level: f64) -> Synthetic {
    let room = |n: usize| ((n as f64 / 480.0 * std::f64::consts::TAU).sin() * 20.0) as i16;
    let voice =
        |n: usize| ((n as f64 / 218.0 * std::f64::consts::TAU).sin() * level * 32767.0) as i16;
    let mut samples: Vec<i16> = vec![30000, -30000, 20000, -20000, 9000, -9000];
    samples.extend((0..at(0.6)).map(room));
    samples.extend((0..at(speech)).map(voice));
    samples.extend((0..at(0.7)).map(room));
    samples.extend([30000, -30000, 20000, -20000, 9000, -9000]);
    let seconds = samples.len() as f64 / f64::from(RATE);
    Synthetic::samples(RATE, samples).lasting(seconds)
}

/// The app on a new project in `dir`, at its studio.
fn window<'a>(dir: &Path) -> (Harness<'a, App>, PathBuf) {
    fs::write(dir.join("vidni.toml"), vidni_core::TEMPLATE).unwrap();
    fs::write(dir.join("page.html"), "<!doctype html>").unwrap();
    let mut app = App::new(None);
    app.open(dir);
    app.tab = Tab::Record;
    // Nothing is heard and nobody is recorded: there is no output, and
    // the input is silence until a test sets one.
    let mut studio = Studio::open(dir);
    studio.set_speaker(Err("there is no output in a test".to_string()));
    studio.set_input(Box::new(Synthetic::silence(RATE).lasting(0.0)));
    app.studio = Some(studio);
    let mut harness = Harness::builder()
        .with_size([1200.0, 1600.0])
        .build_ui_state(|ui, app: &mut App| app.show(ui), app);
    harness.step();
    (harness, dir.to_path_buf())
}

fn studio<'h>(harness: &'h Harness<'_, App>) -> &'h Studio {
    harness.state().studio.as_ref().expect("the studio")
}

/// Presses the button `label`. One frame is drawn: a recording asks for
/// the next one for as long as it runs.
fn press(harness: &mut Harness<'_, App>, label: &str) {
    harness.get_by_label(label).click();
    harness.step();
    harness.step();
}

fn fill(harness: &mut Harness<'_, App>, label: &str, text: &str) {
    harness.get_by_label(label).focus();
    harness.step();
    harness.key_combination_modifiers(egui::Modifiers::COMMAND, &[egui::Key::A]);
    harness.step();
    harness.get_by_label(label).type_text(text);
    harness.step();
}

/// Records from `input` until it has given `seconds`.
fn record(harness: &mut Harness<'_, App>, input: Synthetic, seconds: f64, button: &str) {
    harness
        .state_mut()
        .studio
        .as_mut()
        .unwrap()
        .set_input(Box::new(input));
    press(harness, button);
    let began = Instant::now();
    loop {
        harness.step();
        match studio(harness).recording() {
            Some(level) if level.seconds >= seconds - 1e-9 => break,
            // The room's recording stops on its own.
            None => break,
            Some(level) => assert!(
                began.elapsed() < Duration::from_secs(10),
                "{level:?}; {}",
                studio(harness).note
            ),
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// Records a spoken take and stops it: the new take is in review.
fn take_of(harness: &mut Harness<'_, App>, speech: f64, level: f64) {
    record(
        harness,
        spoken(speech, level),
        1.3 + speech,
        "Record a take",
    );
    // While it is recorded: the meter, the waveform and the time.
    assert!(harness.query_by_label_contains("Recording: ").is_some());
    assert!(harness.query_by_label_contains("Level: ").is_some());
    assert!(
        harness
            .query_by_label("Waveform of the recording")
            .is_some()
    );
    press(harness, "Stop");
}

/// What `vidni <args>` says, run in this process.
fn vidni(args: &[&str]) -> (u8, String) {
    let args: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
    let (code, said) = jobs::now(&args);
    (
        code,
        said.into_iter().map(|(_, line)| line + "\n").collect(),
    )
}

/// The files in `dir`, by name, sorted.
fn names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn a_line_is_recorded_reviewed_kept_picked_and_trimmed() {
    let dir = tempfile::tempdir().unwrap();
    let (mut app, folder) = window(dir.path());
    let path = folder.to_str().unwrap();
    // The prompter is at the first line with no take, and shows its words.
    assert!(app.query_by_label("0 of 2 lines recorded").is_some());
    assert!(app.query_by_label("Hello, and welcome.").is_some());
    assert!(
        app.query_by_label_contains("Line 1 of 2  [intro-1]")
            .is_some()
    );
    assert!(app.query_by_label("Room tone: none yet").is_some());
    let line = studio(&app).line().unwrap().clone();
    assert_eq!(line.cuts, ["long", "vertical"]);
    let raw_dir = folder.join("takes/intro-1/raw");

    // A first take. It waits beside where the kept take will be.
    take_of(&mut app, 1.0, 0.3);
    let candidate = studio(&app).reviewing().expect("a take to review").clone();
    assert!(candidate.path.is_file() && !line.kept.exists());
    // The words, and a little room before and after them.
    assert!((1.0..1.6).contains(&candidate.seconds), "{candidate:?}");
    assert!(app.query_by_label("New take").is_some());
    assert!(app.query_by_label("Waveform of the new take").is_some());
    // B is the new take. There is no kept take to play as A yet.
    press(&mut app, "Play B: the new take");
    assert_eq!(studio(&app).played.as_ref(), Some(&candidate.path));
    assert!(
        app.get_by_label("Play A: the kept take")
            .accesskit_node()
            .is_disabled()
    );

    // Kept: it is the line's take, with the record of where it was cut,
    // and the prompter goes on to the next line with none.
    press(&mut app, "Keep");
    assert!(line.kept.is_file() && !candidate.path.exists());
    let first = fs::read(&line.kept).unwrap();
    assert!(app.query_by_label("1 of 2 lines recorded").is_some());
    assert!(app.query_by_label("Kept the take of intro-1.").is_some());
    assert_eq!(studio(&app).line().unwrap().id, "intro-2");
    assert!(
        app.query_by_label("This video was made with vidni.")
            .is_some()
    );

    // The command line has it as the app has it.
    let (code, listed) = vidni(&["takes", path, "--line", "intro-1"]);
    assert_eq!(code, 0);
    let rows: Vec<&str> = listed.lines().collect();
    assert!(rows[1].starts_with("*  1  takes/intro-1/raw/"), "{listed}");
    assert!(rows[1].ends_with("from a made-up input"), "{listed}");
    assert!(
        rows[2].starts_with("kept: takes/intro-1/") && rows[2].contains("raw take 1 from "),
        "{listed}"
    );
    assert!(
        rows[2].ends_with("(where vidni found the words)"),
        "{listed}"
    );
    // A final build asks for the other line only. (It asks for the cut
    // to be ratified first, as ever.)
    let (code, refusal) = vidni(&["final", path, "--cut", "long"]);
    assert_eq!(code, 1);
    assert!(
        refusal.ends_with("vidni: cut long: 1 of its 2 lines has no take: intro-2\n"),
        "{refusal}"
    );

    // Back to the first line, for a second take: longer, and louder.
    press(&mut app, "[intro-1]  recorded  Hello, and welcome.");
    assert_eq!(studio(&app).line().unwrap().id, "intro-1");
    assert_eq!(studio(&app).raws.len(), 1);
    assert_eq!(studio(&app).pick.as_ref().unwrap().raw, 1);
    assert!(
        app.query_by_label("Waveform of the kept take's raw take")
            .is_some()
    );
    take_of(&mut app, 1.6, 0.5);
    let second = studio(&app).reviewing().unwrap().clone();
    // A, the kept take, is still the first one, and plays as such.
    assert_eq!(fs::read(&line.kept).unwrap(), first);
    press(&mut app, "Play A: the kept take");
    assert_eq!(studio(&app).played.as_ref(), Some(&line.kept));
    press(&mut app, "Play B: the new take");
    assert_eq!(studio(&app).played.as_ref(), Some(&second.path));
    press(&mut app, "Keep");
    assert_ne!(fs::read(&line.kept).unwrap(), first);
    // The prompter went on to the line with no take; back on the first,
    // its kept take is the second raw take.
    assert_eq!(studio(&app).line().unwrap().id, "intro-2");
    assert!(
        app.query_by_label("No raw take of these words yet.")
            .is_some()
    );
    press(&mut app, "[intro-1]  recorded  Hello, and welcome.");
    assert_eq!(studio(&app).pick.as_ref().unwrap().raw, 2);
    assert_eq!(studio(&app).raws.len(), 2);

    // The first take again, by its number: the very take that was kept.
    press(&mut app, "Play take 1");
    assert_eq!(studio(&app).played, Some(studio(&app).raws[0].path.clone()));
    press(&mut app, "Use take 1");
    assert_eq!(studio(&app).pick.as_ref().unwrap().raw, 1);
    assert_eq!(fs::read(&line.kept).unwrap(), first);
    assert!(
        studio(&app).note.contains("raw take 1"),
        "{}",
        studio(&app).note
    );

    // Another trim, by its ends: the kept take is that part of the raw take.
    fill(&mut app, "From (s)", "0.80");
    fill(&mut app, "To (s)", "1.40");
    press(&mut app, "Trim");
    let pick = studio(&app).pick.clone().unwrap();
    assert_eq!((pick.raw, pick.trim), (1, Trim::Set));
    assert_eq!((pick.from, pick.to), (at(0.8), at(1.4)));
    let seconds = take::seconds(&line.kept).unwrap();
    assert!((seconds - 0.6).abs() < 0.05, "{seconds}");
    assert!(
        app.query_by_label_contains("Kept: take 1 from 0.80 s to 1.40 s")
            .is_some()
    );
    // Ends that can't be are refused in `vidni takes`'s words, and the
    // take stays.
    fill(&mut app, "From (s)", "9");
    fill(&mut app, "To (s)", "1");
    press(&mut app, "Trim");
    assert!(
        studio(&app).note.starts_with("vidni: "),
        "{}",
        studio(&app).note
    );
    assert_eq!(studio(&app).pick, Some(pick));
    // And back to where vidni finds the words.
    press(&mut app, "Trim where vidni finds the words");
    assert_eq!(studio(&app).pick.as_ref().unwrap().trim, Trim::Auto);
    assert_eq!(fs::read(&line.kept).unwrap(), first);

    // A pick made on the command line shows in the open studio.
    assert_eq!(
        vidni(&["takes", path, "--line", "intro-1", "--pick", "2"]).0,
        0
    );
    app.step();
    assert_eq!(studio(&app).pick.as_ref().unwrap().raw, 2);
    let kept = fs::read(&line.kept).unwrap();

    // A take that is discarded leaves the line as it was. Its raw take
    // stays, as every raw take does.
    take_of(&mut app, 1.0, 0.3);
    let third = studio(&app).reviewing().unwrap().path.clone();
    press(&mut app, "Discard");
    assert!(!third.exists());
    assert_eq!(fs::read(&line.kept).unwrap(), kept);
    assert_eq!(studio(&app).raws.len(), 3);
    assert_eq!(studio(&app).pick.as_ref().unwrap().raw, 2);

    // A recording that is cancelled leaves no file at all.
    let before = names(&raw_dir);
    record(&mut app, spoken(1.0, 0.3), 1.0, "Record a take");
    assert_eq!(names(&raw_dir).len(), before.len() + 1);
    press(&mut app, "Cancel");
    assert_eq!(names(&raw_dir), before);
    assert!(
        app.query_by_label("Cancelled; nothing was saved.")
            .is_some()
    );

    // A take with nobody speaking is refused, in the recorder's words.
    record(
        &mut app,
        Synthetic::silence(RATE).lasting(1.5),
        1.5,
        "Record a take",
    );
    press(&mut app, "Stop");
    assert!(studio(&app).reviewing().is_none());
    assert!(
        studio(&app)
            .note
            .starts_with("There was no speech in that take"),
        "{}",
        studio(&app).note
    );
    assert_eq!(fs::read(&line.kept).unwrap(), kept);

    // A take that waits to be kept is discarded when the studio is left.
    take_of(&mut app, 1.0, 0.3);
    let left = studio(&app).reviewing().unwrap().path.clone();
    assert!(left.is_file());
    press(&mut app, "Cuts");
    assert!(!left.exists());
    assert_eq!(fs::read(&line.kept).unwrap(), kept);
    // Nothing but takes and their records is in the line's folder.
    let hash = &line.hash;
    assert_eq!(
        names(&folder.join("takes/intro-1")),
        [format!("{hash}.json"), format!("{hash}.wav"), "raw".into()]
    );
}

#[test]
fn the_room_is_recorded_and_measured() {
    let dir = tempfile::tempdir().unwrap();
    let (mut app, folder) = window(dir.path());
    let tone = folder.join("takes/_room/tone.wav");
    // A quiet hum, for as long as the room is recorded.
    let hum = Synthetic::tone(RATE, 120.0, 0.001).lasting(4.0);
    record(&mut app, hum, 99.0, "Record room tone");
    app.step();
    // It stopped on its own, and the middle of it is the room tone.
    assert!(studio(&app).recording().is_none());
    assert!(tone.is_file());
    assert_eq!(names(tone.parent().unwrap()), ["tone.wav"]);
    let seconds = take::seconds(&tone).unwrap();
    // At least the three seconds it is recorded for, less its ends.
    assert!((2.5..=3.5).contains(&seconds), "{seconds}");
    let level = studio(&app).room.expect("the room's level");
    assert!((level + 63.0).abs() < 1.0, "{level}");
    assert!(app.query_by_label("Room tone: -63 dB").is_some());
    assert!(app.query_by_label("Room tone kept: -63 dB.").is_some());

    // A noisy room is kept too, and said to be noisy.
    let noise = Synthetic::tone(RATE, 120.0, 0.02).lasting(4.0);
    record(&mut app, noise, 99.0, "Record room tone");
    app.step();
    assert!(
        studio(&app).note.contains("The room or the mic is noisy."),
        "{}",
        studio(&app).note
    );
    assert!((studio(&app).room.unwrap() + 37.0).abs() < 1.0);
}

#[test]
fn a_project_that_cant_be_read_isnt_recorded() {
    let dir = tempfile::tempdir().unwrap();
    let (mut app, folder) = window(dir.path());
    fs::write(folder.join("vidni.toml"), "schema = 1\n").unwrap();
    app.step();
    assert!(studio(&app).problem.is_some());
    assert!(app.query_by_label("Record a take").is_none());
    assert!(app.query_by_label_contains("has problems").is_some());
    // Mended, it is recorded again.
    fs::write(folder.join("vidni.toml"), vidni_core::TEMPLATE).unwrap();
    app.step();
    assert!(studio(&app).problem.is_none());
    assert_eq!(studio(&app).lines.len(), 2);
}

#[test]
fn what_goes_wrong_during_a_take_loses_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let (mut app, folder) = window(dir.path());
    let raw_dir = folder.join("takes/intro-1/raw");

    // An input that fails while a take is recorded: what it gave until
    // then is a raw take, as it is in the terminal.
    let unplugged = spoken(1.0, 0.3)
        .lasting(1.2)
        .failing("the device was unplugged");
    record(&mut app, unplugged, 99.0, "Record a take");
    app.step();
    assert!(studio(&app).recording().is_none());
    let note = studio(&app).note.clone();
    assert!(note.contains("the device was unplugged"), "{note}");
    assert!(note.contains("among the line's takes"), "{note}");
    assert_eq!(names(&raw_dir).len(), 1);
    assert_eq!(studio(&app).raws.len(), 1);
    let seconds = studio(&app).raws[0].seconds;
    assert!((seconds - 1.2).abs() < 0.01, "{seconds}");
    // Nothing plays, and no other line is gone to, while one is recorded.
    record(&mut app, spoken(1.0, 0.3), 2.3, "Record a take");
    for label in [
        "Play take 1",
        "[intro-2]  no take  This video was made with vidni.",
    ] {
        assert!(app.get_by_label(label).accesskit_node().is_disabled());
    }
    press(&mut app, "Stop");

    // The new take is taken away, as a terminal recorder started on the
    // project does: keeping it says what happened, and its raw take is
    // there to use.
    let candidate = studio(&app).reviewing().unwrap().clone();
    fs::remove_file(&candidate.path).unwrap();
    press(&mut app, "Keep");
    let note = studio(&app).note.clone();
    assert!(note.starts_with("The new take is gone"), "{note}");
    assert!(!candidate.kept.exists());
    assert_eq!(studio(&app).raws.len(), 2);
    press(&mut app, "Use take 2");
    assert!(candidate.kept.is_file());
    assert_eq!(studio(&app).pick.as_ref().unwrap().raw, 2);

    // The project's file is broken while a take of the second line waits:
    // the take is kept all the same, and the studio says what is wrong.
    press(&mut app, "Next line");
    assert_eq!(studio(&app).line().unwrap().id, "intro-2");
    take_of(&mut app, 1.0, 0.3);
    let second = studio(&app).reviewing().unwrap().clone();
    fs::write(folder.join("vidni.toml"), "schema = 1\n").unwrap();
    press(&mut app, "Keep");
    assert!(second.kept.is_file() && !second.path.exists());
    assert!(studio(&app).problem.is_some());
    assert!(app.query_by_label_contains("has problems").is_some());
    // Mended, both lines are recorded.
    fs::write(folder.join("vidni.toml"), vidni_core::TEMPLATE).unwrap();
    app.step();
    assert!(app.query_by_label("2 of 2 lines recorded").is_some());
}
