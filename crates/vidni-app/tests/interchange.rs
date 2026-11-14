//! Takes recorded in the app and in `vidni record` are interchangeable
//! (TODO.md, P7-VIDNI-06). The terminal recorder runs in a
//! pseudo-terminal, the app's studio in a window with no screen, and both
//! record a made-up input into one project: each resumes where the other
//! stopped, lists and plays the other's takes, and picks and trims them.
//! The same sound recorded by each gives the same kept take, byte for
//! byte.
//!
//! Ignored by `cargo test`: it needs a terminal, ALSA's null device and
//! the `vidni` binary that `VIDNI_BIN` names. `just test-app` runs it.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use vidni_app::app::{App, Tab};
use vidni_app::studio::Studio;
use vidni_core::takes::{self, Trim};
use vidni_record::input::{self, Synthetic};
use vidni_record::play::Speaker;
use vidni_record::take;

#[path = "../../vidni-cli/tests/shared/pty.rs"]
mod pty;
use pty::{PROJECT, READY, Recorder};

const RATE: u32 = 48000;

/// The `vidni` binary that records in the terminal.
fn vidni_binary() -> PathBuf {
    PathBuf::from(std::env::var_os("VIDNI_BIN").expect("VIDNI_BIN names the vidni binary"))
}

/// What `vidni <args>` says, and its exit code.
fn vidni(args: &[&str]) -> (Option<i32>, String) {
    let out = std::process::Command::new(vidni_binary())
        .args(args)
        .output()
        .unwrap();
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned() + &String::from_utf8_lossy(&out.stderr),
    )
}

fn project(dir: &Path) -> PathBuf {
    fs::create_dir_all(dir).unwrap();
    fs::write(dir.join("vidni.toml"), PROJECT).unwrap();
    dir.to_path_buf()
}

/// The app at the studio of the project in `dir`. It plays through
/// ALSA's null device, so nothing is heard.
fn window<'a>(dir: &Path) -> Harness<'a, App> {
    let mut app = App::new(None);
    app.open(dir);
    app.tab = Tab::Record;
    let mut studio = Studio::open(dir);
    studio.set_speaker(Speaker::find(Some("Discard all samples")));
    studio.set_input(Box::new(Synthetic::silence(RATE).lasting(0.0)));
    app.studio = Some(studio);
    let mut harness = Harness::builder()
        .with_size([1200.0, 1600.0])
        .build_ui_state(|ui, app: &mut App| app.show(ui), app);
    harness.step();
    harness
}

fn studio<'h>(harness: &'h Harness<'_, App>) -> &'h Studio {
    harness.state().studio.as_ref().expect("the studio")
}

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

/// Records a take of the line in the prompter from the made-up input
/// `words`, at a real input's pace, for `seconds`; and keeps it.
fn take_in_the_app(harness: &mut Harness<'_, App>, words: &str, seconds: f64) {
    let input = input::made_up(words, RATE).unwrap();
    harness
        .state_mut()
        .studio
        .as_mut()
        .unwrap()
        .set_input(Box::new(input));
    press(harness, "Record a take");
    let began = Instant::now();
    while studio(harness).recording().expect("a recording").seconds < seconds {
        assert!(began.elapsed() < Duration::from_secs(20));
        harness.step();
        std::thread::sleep(Duration::from_millis(10));
    }
    press(harness, "Stop");
    assert!(
        studio(harness).reviewing().is_some(),
        "{}",
        studio(harness).note
    );
    press(harness, "Keep");
}

/// The kept take of `line`: its file, and the record beside it.
fn kept(dir: &Path, line: &str) -> (PathBuf, takes::Pick) {
    let folder = dir.join("takes").join(line);
    let file = fs::read_dir(&folder)
        .unwrap()
        .flatten()
        .map(|entry| entry.path())
        .find(|path| path.extension().is_some_and(|x| x == "wav"))
        .unwrap_or_else(|| panic!("no kept take in {}", folder.display()));
    let pick = takes::pick(&file).unwrap().expect("its record");
    (file, pick)
}

#[test]
#[ignore = "needs a terminal, ALSA and VIDNI_BIN: just test-app"]
fn each_recorder_goes_on_from_the_others_takes() {
    let work = tempfile::tempdir().unwrap();
    let dir = project(&work.path().join("video"));
    let path = dir.to_str().unwrap();

    // In the terminal: the first line is recorded and kept.
    let mut terminal = Recorder::start(&dir, &[], "tone:220:0.3");
    let ready = terminal.wait_for(READY);
    assert!(ready.contains("0 of 3 recorded"), "{ready}");
    terminal.take(1.6);
    terminal.review();
    terminal.press("\r");
    terminal.wait_for("Line 2 of 3");
    terminal.press("q");
    let (code, all) = terminal.finish();
    assert_eq!(code, 0, "{all}");
    let (first_file, first_pick) = kept(&dir, "main-1");
    let first = fs::read(&first_file).unwrap();

    // The app goes on where the terminal stopped, and has its take.
    let mut app = window(&dir);
    assert!(app.query_by_label("1 of 3 lines recorded").is_some());
    assert_eq!(studio(&app).line().unwrap().id, "main-2");
    press(&mut app, "Previous line");
    let listed = studio(&app).raws.clone();
    assert_eq!(listed.len(), 1);
    assert!(
        listed[0].recorded.ends_with("from a made-up input"),
        "{listed:?}"
    );
    assert_eq!(studio(&app).pick.as_ref(), Some(&first_pick));
    // It plays it, kept and raw.
    press(&mut app, "Play the kept take");
    assert_eq!(studio(&app).played.as_ref(), Some(&first_file));
    assert_eq!(studio(&app).note, "");
    press(&mut app, "Play take 1");
    assert_eq!(studio(&app).played.as_ref(), Some(&listed[0].path));
    assert_eq!(studio(&app).note, "");
    // It trims it elsewhere, which the command line then reports.
    fill(&mut app, "From (s)", "0.30");
    fill(&mut app, "To (s)", "1.00");
    press(&mut app, "Trim");
    assert_eq!(kept(&dir, "main-1").1.trim, Trim::Set);
    let (code, said) = vidni(&["takes", path, "--line", "main-1"]);
    assert_eq!(code, Some(0));
    assert!(
        said.contains("raw take 1 from 0.30 s to 1.00 s (where --trim put its ends)"),
        "{said}"
    );
    // And puts it back as the terminal kept it.
    press(&mut app, "Trim where vidni finds the words");
    assert_eq!(fs::read(&first_file).unwrap(), first);
    assert_eq!(kept(&dir, "main-1").1, first_pick);

    // In the app: the second line is recorded and kept.
    press(&mut app, "Next line");
    take_in_the_app(&mut app, "tone:220:0.3", 1.6);
    let (second_file, second_pick) = kept(&dir, "main-2");
    let second = fs::read(&second_file).unwrap();
    assert_eq!((second_pick.raw, second_pick.trim), (1, Trim::Auto));
    assert!(app.query_by_label("2 of 3 lines recorded").is_some());
    // The command line lists it as it lists its own.
    let (code, said) = vidni(&["takes", path, "--line", "main-2"]);
    assert_eq!(code, Some(0), "{said}");
    assert!(said.contains("*  1  takes/main-2/raw/"), "{said}");
    assert!(said.contains("from a made-up input"), "{said}");
    // The command line cuts the app's take elsewhere, and puts it back.
    let (code, said) = vidni(&["takes", path, "--line", "main-2", "--trim", "0.4:1.2"]);
    assert_eq!(code, Some(0), "{said}");
    assert_eq!(kept(&dir, "main-2").1.trim, Trim::Set);
    assert_ne!(fs::read(&second_file).unwrap(), second);
    let (code, said) = vidni(&["takes", path, "--line", "main-2", "--pick", "1"]);
    assert_eq!(code, Some(0), "{said}");
    assert_eq!(fs::read(&second_file).unwrap(), second);
    assert_eq!(kept(&dir, "main-2").1, second_pick);
    // A final build no longer asks for either line's take.
    let (code, said) = vidni(&["final", path, "--cut", "long"]);
    assert_eq!(code, Some(1), "{said}");
    assert!(
        said.ends_with("vidni: cut long: 1 of its 3 lines has no take: end-1\n"),
        "{said}"
    );

    // The terminal goes on where the app stopped: at the third line.
    let mut terminal = Recorder::start(&dir, &[], "tone:220:0.3");
    let ready = terminal.wait_for(READY);
    assert!(ready.contains("2 of 3 recorded"), "{ready}");
    assert!(ready.contains("Line 3 of 3  [end-1]"), "{ready}");
    // It goes back to the app's line, plays its take, and records another.
    terminal.press("\x1b[D");
    let back = terminal.wait_for("Line 2 of 3  [main-2]");
    assert!(back.contains(READY), "{back}");
    terminal.press("p");
    terminal.wait_for(READY);
    std::thread::sleep(Duration::from_millis(600));
    terminal.take(2.0);
    terminal.review();
    terminal.press("\r");
    terminal.wait_for("Line 3 of 3");
    terminal.press("q");
    let (code, all) = terminal.finish();
    assert_eq!(code, 0, "{all}");
    assert!(all.contains("2 of 3 lines recorded"), "{all}");
    assert_ne!(fs::read(&second_file).unwrap(), second);

    // The open app has the terminal's take as the kept one, beside its own.
    app.step();
    press(
        &mut app,
        "[main-2]  recorded  The second line is in both cuts.",
    );
    assert_eq!(studio(&app).raws.len(), 2);
    assert_eq!(studio(&app).pick.as_ref().unwrap().raw, 2);
    // Its own take again, by its number: the bytes it kept before.
    press(&mut app, "Use take 1");
    assert_eq!(fs::read(&second_file).unwrap(), second);
    assert_eq!(kept(&dir, "main-2").1, second_pick);
    // And the terminal's, picked on the command line, shows in the app.
    let (code, said) = vidni(&["takes", path, "--line", "main-2", "--pick", "2"]);
    assert_eq!(code, Some(0), "{said}");
    app.step();
    assert_eq!(studio(&app).pick.as_ref().unwrap().raw, 2);
}

#[test]
#[ignore = "needs a terminal, ALSA and VIDNI_BIN: just test-app"]
fn the_same_sound_gives_the_same_take_in_both() {
    let work = tempfile::tempdir().unwrap();
    // A recording of somebody speaking: room noise, a second and a half
    // of a voice-like tone, room noise.
    let at = |seconds: f64| (seconds * f64::from(RATE)).round() as usize;
    let room = |n: usize| ((n as f64 / 480.0 * std::f64::consts::TAU).sin() * 20.0) as i16;
    let voice = |n: usize| ((n as f64 / 218.0 * std::f64::consts::TAU).sin() * 9000.0) as i16;
    let mut samples: Vec<i16> = (0..at(0.4)).map(room).collect();
    samples.extend((0..at(1.5)).map(voice));
    samples.extend((0..at(0.6)).map(room));
    let spoken = work.path().join("spoken.wav");
    let seconds = samples.len() as f64 / f64::from(RATE);
    let recording =
        take::record(&Synthetic::samples(RATE, samples).lasting(seconds), &spoken).unwrap();
    while recording.level().seconds < seconds - 1e-9 {
        std::thread::sleep(Duration::from_millis(1));
    }
    recording.stop().unwrap();
    let words = format!("wav:{}", spoken.display());

    // In the terminal, in one copy of the project.
    let (a, b) = (
        project(&work.path().join("a")),
        project(&work.path().join("b")),
    );
    let mut terminal = Recorder::start(&a, &[], &words);
    terminal.wait_for(READY);
    terminal.take(3.4);
    terminal.review();
    terminal.press("\r");
    terminal.wait_for("Line 2 of 3");
    terminal.press("q");
    let (code, all) = terminal.finish();
    assert_eq!(code, 0, "{all}");

    // In the app, in the other; stopped at another moment.
    let mut app = window(&b);
    assert_eq!(studio(&app).line().unwrap().id, "main-1");
    take_in_the_app(&mut app, &words, 3.1);

    // The kept takes are the same bytes, cut at the same samples of raw
    // takes that differ only in where they were stopped.
    let ((in_a, pick_a), (in_b, pick_b)) = (kept(&a, "main-1"), kept(&b, "main-1"));
    assert_eq!(
        in_a.strip_prefix(&a).unwrap(),
        in_b.strip_prefix(&b).unwrap()
    );
    assert_eq!(pick_a, pick_b);
    assert_eq!(fs::read(&in_a).unwrap(), fs::read(&in_b).unwrap());
    let raw = |dir: &Path| {
        let (id, hash) = ("main-1", in_a.file_stem().unwrap().to_str().unwrap());
        let path = takes::raw(dir, id, hash, 1);
        (
            take::read(&path).unwrap(),
            takes::session(&path).unwrap().expect("its session"),
        )
    };
    let (((raw_a, rate_a), session_a), ((raw_b, rate_b), session_b)) = (raw(&a), raw(&b));
    assert_eq!(rate_a, rate_b);
    let shared = raw_a.len().min(raw_b.len());
    assert!(shared >= pick_a.to && raw_a.len() != raw_b.len());
    assert_eq!(raw_a[..shared], raw_b[..shared]);
    // Each is noted as recorded from the same input, at the same rate.
    assert_eq!(
        (session_a.microphone, session_a.rate),
        (session_b.microphone, session_b.rate)
    );
}
