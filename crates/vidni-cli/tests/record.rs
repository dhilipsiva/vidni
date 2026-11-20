//! `vidni record` in a pseudo-terminal (TODO.md, P4-VIDNI-02): a silent
//! take, a double press, Esc, keep, and resuming on a project with two
//! cuts. The input is made up (`VIDNI_RECORD_INPUT`), and takes are played
//! through ALSA's null device, so nothing is heard and nobody is recorded.
//! They are ignored by `cargo test`; `just test-record` runs them.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[path = "shared/pty.rs"]
mod pty;
use pty::{PROJECT, READY, Recorder};

/// The `vidni` binary that the recorder's tests run.
fn vidni_binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_vidni"))
}

/// The WAV files in `dir`, by name, sorted.
fn wavs(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".wav") || name.ends_with(".new"))
        .collect();
    names.sort();
    names
}

fn project() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_path_buf();
    fs::write(path.join("vidni.toml"), PROJECT).unwrap();
    (dir, path)
}

#[test]
#[ignore = "needs a terminal and ALSA: just test-record"]
fn space_retries_without_replacing_a_saved_take_until_enter() {
    let (_keep, dir) = project();
    let mut recorder = Recorder::start(&dir, &[], "tone:220:0.3");
    recorder.wait_for(READY);
    recorder.take(1.3);
    recorder.review();
    recorder.press("q");
    assert_eq!(recorder.finish().0, 0);
    let folder = dir.join("takes/main-1");
    let file = wavs(&folder)
        .into_iter()
        .find(|n| n.ends_with(".wav"))
        .unwrap();
    let original = fs::read(folder.join(&file)).unwrap();
    let mut recorder = Recorder::start(&dir, &["--only", "main-1"], "tone:330:0.3");
    recorder.wait_for(READY);
    recorder.take(1.3);
    assert!(recorder.review().contains("space record again"));
    // The stop key may repeat while review settles: it must not start a retake.
    assert_eq!(fs::read(folder.join(&file)).unwrap(), original);
    recorder.press("  ");
    recorder.wait_for("● REC");
    std::thread::sleep(Duration::from_millis(900));
    assert_eq!(fs::read(folder.join(&file)).unwrap(), original);
    recorder.press(" ");
    recorder.review();
    assert_eq!(fs::read(folder.join(&file)).unwrap(), original);
    recorder.press("\x1b");
    recorder.wait_for("Discarded; the line is as it was.");
    assert_eq!(fs::read(folder.join(&file)).unwrap(), original);
    recorder.take(1.3);
    recorder.review();
    recorder.press(" ");
    recorder.wait_for("● REC");
    std::thread::sleep(Duration::from_millis(1400));
    recorder.press(" ");
    recorder.review();
    recorder.press("\r");
    assert_eq!(recorder.finish().0, 0);
    assert_ne!(fs::read(folder.join(&file)).unwrap(), original);
    assert!(wavs(&folder).iter().all(|n| !n.ends_with(".new")));
}

#[test]
#[ignore = "needs a terminal and ALSA: just test-record"]
fn a_silent_take_is_discarded_and_the_line_stays() {
    let (_keep, dir) = project();
    let mut recorder = Recorder::start(&dir, &[], "silence");
    let ready = recorder.wait_for(READY);
    assert!(ready.contains("A recording · 0 of 3 recorded"), "{ready}");
    assert!(
        ready.contains("Line 1 of 3  [main-1] · The main part · in long\r\n"),
        "{ready}"
    );
    assert!(
        ready.contains("The first line is only in the long cut."),
        "{ready}"
    );

    // A take with nobody speaking: discarded, and the line is still the first.
    recorder.take(1.3);
    let refused = recorder.wait_for("There was no speech in that take");
    assert!(refused.contains("so it was discarded"), "{refused}");
    assert!(
        refused.contains("Line 1 of 3") && refused.contains(READY),
        "{refused}"
    );
    // The raw take is there, as every raw take is, and nothing was kept.
    assert_eq!(wavs(&dir.join("takes/main-1")), Vec::<String>::new());
    assert_eq!(wavs(&dir.join("takes/main-1/raw")).len(), 1);

    // A take cut short at half a second is discarded too.
    recorder.take(0.5);
    let short = recorder.wait_for("That take was only 0.");
    assert!(short.contains("Line 1 of 3"), "{short}");
    assert_eq!(wavs(&dir.join("takes/main-1")), Vec::<String>::new());

    recorder.press("q");
    let (code, all) = recorder.finish();
    assert_eq!(code, 0, "{all}");
    assert!(all.contains("0 of 3 lines recorded, in "), "{all}");
    assert!(
        all.contains("still to record: main-1, main-2, end-1"),
        "{all}"
    );
}

#[test]
#[ignore = "needs a terminal and ALSA: just test-record"]
fn a_double_press_esc_keep_and_resume_on_a_project_with_two_cuts() {
    let (_keep, dir) = project();
    let kept = |line: &str| wavs(&dir.join("takes").join(line));
    let mut recorder = Recorder::start(&dir, &[], "tone:220:0.3");
    recorder.wait_for(READY);

    // A double press: the second one comes while the screen is still
    // settling, and doesn't stop the take it started.
    recorder.press("  ");
    recorder.wait_for("● REC");
    std::thread::sleep(Duration::from_millis(900));
    let recording = recorder.screen();
    assert!(!recording.contains("That take was only"), "{recording}");
    assert!(recording.contains("space stop · esc cancel"), "{recording}");
    std::thread::sleep(Duration::from_millis(400));
    recorder.press(" ");
    let review = recorder.review();
    assert!(
        review.contains("New take") && review.contains(" s · peak -"),
        "{review}"
    );
    // The new take waits beside where the kept one goes; nothing is kept yet.
    assert_eq!(kept("main-1").len(), 1);
    assert!(kept("main-1")[0].ends_with(".wav.new"));

    // Esc in the review discards it.
    recorder.press("\x1b");
    recorder.wait_for("Discarded; the line is as it was.");
    assert_eq!(kept("main-1"), Vec::<String>::new());
    // Esc while recording cancels the take, and leaves no raw take of it.
    recorder.press(" ");
    recorder.wait_for("● REC");
    std::thread::sleep(Duration::from_millis(700));
    recorder.press("\x1b");
    recorder.wait_for("Cancelled; nothing was saved.");
    assert_eq!(wavs(&dir.join("takes/main-1/raw")).len(), 1);

    // A take that is kept: played first, then enter. The session moves on
    // to the next line, which both cuts use.
    recorder.take(1.3);
    recorder.review();
    recorder.press("p");
    std::thread::sleep(Duration::from_millis(300));
    recorder.press("\r");
    let next = recorder.wait_for("Line 2 of 3");
    assert!(next.contains("A recording · 1 of 3 recorded"), "{next}");
    assert!(
        next.contains("[main-2] · The main part · in long and short"),
        "{next}"
    );
    assert!(
        next.contains("before  The first line is only in the long cut."),
        "{next}"
    );
    let takes = kept("main-1");
    assert_eq!(takes.len(), 1);
    assert!(
        takes[0].ends_with(".wav") && !takes[0].ends_with(".new"),
        "{takes:?}"
    );
    assert_eq!(wavs(&dir.join("takes/main-1/raw")).len(), 2);
    // P4-VIDNI-04: beside the kept take is the record of which raw take
    // it is cut from, and beside each raw take what it was recorded with.
    let records = |folder: &str| -> Vec<(String, String)> {
        let mut found: Vec<(String, String)> = fs::read_dir(dir.join(folder))
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|x| x == "json"))
            .map(|p| {
                (
                    p.file_name().unwrap().to_string_lossy().into_owned(),
                    fs::read_to_string(&p).unwrap(),
                )
            })
            .collect();
        found.sort();
        found
    };
    let picks = records("takes/main-1");
    assert_eq!(picks.len(), 1);
    assert_eq!(picks[0].0, takes[0].replace(".wav", ".json"));
    assert!(
        picks[0].1.contains("\"raw\": 2") && picks[0].1.contains("\"trim\": \"auto\""),
        "{picks:?}"
    );
    // Both raw takes that were stopped have a session; the cancelled one left nothing.
    let sessions = records("takes/main-1/raw");
    assert_eq!(sessions.len(), 2, "{sessions:?}");
    let named = |text: &str, key: &str| -> String {
        let at = text.find(&format!("\"{key}\": \"")).unwrap() + key.len() + 5;
        text[at..at + text[at..].find('"').unwrap()].to_string()
    };
    let first_session = named(&sessions[1].1, "session");
    assert_eq!(named(&sessions[0].1, "session"), first_session);
    assert!(
        first_session.ends_with('Z') && first_session.len() == 20,
        "{first_session}"
    );
    assert!(sessions[1].1.contains("\"rate\": 48000"), "{sessions:?}");
    assert!(!named(&sessions[1].1, "microphone").is_empty());

    // Back to the first line: its take is shown, and plays, alone and as
    // its section.
    recorder.press("\x1b[D");
    let first = recorder.wait_for("Line 1 of 3");
    assert!(first.contains("✓ saved take, 1."), "{first}");
    // (A key pressed while a take plays stops it, so each waits its turn.)
    for key in ["p", "s", "\x1b[C"] {
        recorder.press(key);
        std::thread::sleep(Duration::from_millis(300));
    }
    recorder.wait_for("Line 2 of 3");
    recorder.press("q");
    let (code, all) = recorder.finish();
    assert_eq!(code, 0, "{all}");
    assert!(all.contains("1 of 3 lines recorded, in "), "{all}");
    assert!(all.contains("still to record: main-2, end-1"), "{all}");

    // Started again, the session is where it stopped. A take kept with q
    // ends the session.
    let mut recorder = Recorder::start(&dir, &[], "tone:220:0.3");
    recorder.wait_for("Line 2 of 3");
    recorder.take(1.3);
    recorder.review();
    recorder.press("q");
    let (code, all) = recorder.finish();
    assert_eq!(code, 0, "{all}");
    assert!(all.contains("2 of 3 lines recorded, in "), "{all}");
    assert_eq!(kept("main-2").len(), 1);
    // That take is of another session, named by a later moment.
    let later = fs::read_dir(dir.join("takes/main-2/raw"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.extension().is_some_and(|x| x == "json"))
        .map(|p| fs::read_to_string(p).unwrap())
        .unwrap();
    assert!(
        later.contains("\"session\": \"") && !later.contains(&first_session),
        "{later}"
    );

    // The short cut alone: the line it shares with the long cut was
    // recorded once, for both, so only its own line is left.
    let mut recorder = Recorder::start(&dir, &["--cut", "short"], "tone:220:0.3");
    let short = recorder.wait_for("Line 2 of 2");
    assert!(short.contains("A recording · 1 of 2 recorded"), "{short}");
    assert!(short.contains("[end-1] · The end"), "{short}");
    recorder.press("q");
    let (code, all) = recorder.finish();
    assert_eq!(code, 0, "{all}");
    assert!(all.contains("1 of 2 lines recorded, in "), "{all}");
    assert!(all.contains("still to record: end-1"), "{all}");

    // A line whose words changed after it was recorded: its take is of
    // other words, and the session starts there.
    let reworded = PROJECT.replace("The first line is only", "The first line was only");
    fs::write(dir.join("vidni.toml"), reworded).unwrap();
    let mut recorder = Recorder::start(&dir, &[], "tone:220:0.3");
    let stale = recorder.wait_for("Line 1 of 3");
    assert!(
        stale.contains("A recording · 1 of 3 recorded, 1 of other words"),
        "{stale}"
    );
    assert!(stale.contains("! its take is of other words"), "{stale}");
    // One section's lines can be gone through alone.
    recorder.press("q");
    recorder.finish();
    let mut recorder = Recorder::start(&dir, &["--section", "end"], "tone:220:0.3");
    let end = recorder.wait_for("Line 1 of 1");
    assert!(
        end.contains("[end-1] · The end · in long and short"),
        "{end}"
    );
    // And the last line, kept, ends the session: every line of the
    // section has its take.
    recorder.take(1.3);
    recorder.review();
    recorder.press("\r");
    let (code, all) = recorder.finish();
    assert_eq!(code, 0, "{all}");
    assert!(all.contains("2 of 3 lines recorded, in "), "{all}");
    assert!(all.contains("still to record: main-1"), "{all}");
}

#[test]
#[ignore = "needs a terminal and ALSA: just test-record"]
fn an_interrupt_keeps_nothing_and_gives_the_terminal_back() {
    let (_keep, dir) = project();
    let mut recorder = Recorder::start(&dir, &[], "tone:220:0.3");
    recorder.wait_for(READY);
    // Ctrl-C in the review of a new take: the take isn't kept.
    recorder.take(1.3);
    recorder.review();
    recorder.press("\x03");
    let written = recorder.written.clone();
    let (code, all) = recorder.finish();
    assert_eq!(code, 130, "{all}");
    assert!(all.contains("0 of 3 lines recorded, in "), "{all}");
    assert_eq!(wavs(&dir.join("takes/main-1")), Vec::<String>::new());
    assert_eq!(wavs(&dir.join("takes/main-1/raw")).len(), 1);
    // The cursor is shown again before the last words are said.
    let raw = String::from_utf8_lossy(&written.lock().unwrap()).into_owned();
    let shown = raw.rfind("\x1b[?25h").expect("the cursor is shown again");
    assert!(shown < raw.rfind("0 of 3 lines recorded").unwrap());

    // Ctrl-C while a take is being recorded leaves no raw take of it.
    let mut recorder = Recorder::start(&dir, &[], "tone:220:0.3");
    recorder.wait_for(READY);
    recorder.press(" ");
    recorder.wait_for("● REC");
    std::thread::sleep(Duration::from_millis(700));
    recorder.press("\x03");
    let (code, all) = recorder.finish();
    assert_eq!(code, 130, "{all}");
    assert_eq!(wavs(&dir.join("takes/main-1/raw")).len(), 1);

    // A signal that asks vidni to stop ends the session the same way.
    let recorder = Recorder::start(&dir, &[], "tone:220:0.3");
    recorder.wait_for(READY);
    let pid = recorder.child.process_id().unwrap().to_string();
    let sent = std::process::Command::new("kill")
        .args(["-TERM", &pid])
        .status()
        .unwrap();
    assert!(sent.success());
    let (code, all) = recorder.finish();
    assert_eq!(code, 130, "{all}");
    assert!(all.contains("0 of 3 lines recorded, in "), "{all}");
}

#[test]
#[ignore = "needs a terminal and ALSA: just test-record"]
fn the_mic_check_measures_the_room_and_a_line() {
    let (_keep, dir) = project();
    // The made-up input hums at -34 dBFS: a noisy room, and a quiet voice.
    let mut recorder = Recorder::start(&dir, &["--test"], "tone:120:0.02");
    let room = recorder.wait_for("Step 1 of 2: stay quiet for 3 seconds");
    assert!(room.contains("Mic check · a made-up input"), "{room}");
    // The room is recorded for three seconds without a key; then the
    // project's first line is read.
    let line = recorder.wait_for("Step 2 of 2: press space, read this line, press space.");
    assert!(
        line.contains("The first line is only in the long cut."),
        "{line}"
    );
    std::thread::sleep(Duration::from_millis(600));
    recorder.take(1.3);
    let report = recorder.wait_for("p play it back · space run the check again · q done");
    assert!(report.contains("room noise -37 dB"), "{report}");
    assert!(report.contains("speech peak -3"), "{report}");
    assert!(report.contains("The room or the mic is noisy."), "{report}");
    assert!(report.contains("Speech is quiet."), "{report}");
    std::thread::sleep(Duration::from_millis(600));
    recorder.press("p");
    std::thread::sleep(Duration::from_millis(300));
    recorder.press(" ");
    // The previous mic-check screen also contains both step labels. Wait for
    // the retry to clear that screen before waiting for its new speech step.
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while recorder.screen().contains("Speech is quiet.") {
        assert!(
            std::time::Instant::now() < deadline,
            "the mic check did not restart"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    recorder.wait_for("Step 1 of 2: stay quiet for 3 seconds");
    recorder.wait_for("Step 2 of 2: press space, read this line, press space.");
    std::thread::sleep(Duration::from_millis(600));
    recorder.take(1.3);
    recorder.wait_for("p play it back · space run the check again · q done");
    std::thread::sleep(Duration::from_millis(600));
    recorder.press("q");
    let (code, all) = recorder.finish();
    assert_eq!(code, 0, "{all}");
    // The room tone is kept, and nothing else of the check.
    assert_eq!(wavs(&dir.join("takes/_room")), ["tone.wav"]);
    assert_eq!(
        fs::read_dir(dir.join("takes/_room")).unwrap().count(),
        1,
        "the check's recordings are gone"
    );
    assert!(!dir.join("takes/main-1").exists());
}

#[test]
#[ignore = "needs a terminal, ALSA and whisper with its model: just test-hear"]
fn the_review_says_how_a_take_matches_its_script() {
    let (_keep, dir) = project();
    // The made-up input says "Language models are good at reading prose."
    let spoken =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../vidni-media/tests/fixtures/hear/spoken.wav");
    let input = format!("wav:{}", spoken.display());
    let scripted = PROJECT.replace(
        "The first line is only in the long cut.",
        "Language models are good at reading prose.",
    );
    fs::write(dir.join("vidni.toml"), scripted).unwrap();
    let mut recorder = Recorder::start_with(&dir, &[], &input, true);
    recorder.wait_for(READY);
    // The first line is what the input says: the take matches its script.
    recorder.take(3.6);
    let matched = recorder.review();
    assert!(
        matched.contains(" dB · matches the script 1.00"),
        "{matched}"
    );
    assert!(!matched.contains("Heard:"), "{matched}");
    recorder.press("\r");
    // The second line is other words: the review shows what was heard.
    recorder.wait_for("Line 2 of 3");
    recorder.take(3.6);
    let misread = recorder.review();
    assert!(misread.contains(" dB · matches the script 0."), "{misread}");
    assert!(
        misread.contains("Heard: Language models are good at reading prose."),
        "{misread}"
    );
    // It only advises: the take can be kept all the same.
    recorder.press("\x1b");
    recorder.wait_for("Discarded; the line is as it was.");
    recorder.press("q");
    let (code, all) = recorder.finish();
    assert_eq!(code, 0, "{all}");
    assert!(all.contains("1 of 3 lines recorded, in "), "{all}");
}
