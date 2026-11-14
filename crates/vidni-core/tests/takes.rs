//! Takes are filed by line ID and spoken words, so editing the script never
//! disturbs a take whose words didn't change.

use std::fs;
use std::path::Path;
use vidni_core::TEMPLATE;
use vidni_core::edit::{LineField, ProjectFile};
use vidni_core::takes::{self, Status};
use vidni_core::validate::check;

/// A dummy take for every line of the project text.
fn record_every_line(dir: &Path, text: &str) {
    let project = check(text, Some(dir)).unwrap();
    for line in takes::status(&project, dir) {
        fs::create_dir_all(line.path.parent().unwrap()).unwrap();
        fs::write(&line.path, line.id.as_bytes()).unwrap();
    }
}

fn states(dir: &Path, text: &str) -> Vec<(String, &'static str)> {
    let project = check(text, Some(dir)).unwrap();
    takes::status(&project, dir)
        .into_iter()
        .map(|l| {
            let state = match l.status {
                Status::Current(_) => "current",
                Status::Stale(_) => "stale",
                Status::Missing => "missing",
            };
            (l.id, state)
        })
        .collect()
}

fn recorded() -> (tempfile::TempDir, ProjectFile) {
    let dir = tempfile::tempdir().unwrap();
    // The template's scene is a page in the project's folder.
    fs::write(
        dir.path().join(vidni_core::starter::PAGE_FILE),
        "<!doctype html>",
    )
    .unwrap();
    record_every_line(dir.path(), TEMPLATE);
    (dir, ProjectFile::parse(TEMPLATE).unwrap())
}

fn owned(pairs: &[(&str, &'static str)]) -> Vec<(String, &'static str)> {
    pairs.iter().map(|(id, s)| (id.to_string(), *s)).collect()
}

#[test]
fn inserting_a_line_reuses_every_other_take() {
    let (dir, mut f) = recorded();
    f.insert_line(None, "intro-0", "intro", "A new first line.")
        .unwrap();
    f.insert_line(
        Some("intro-1"),
        "intro-1b",
        "intro",
        "A new line in the middle.",
    )
    .unwrap();
    assert_eq!(
        states(dir.path(), &f.text()),
        owned(&[
            ("intro-0", "missing"),
            ("intro-1", "current"),
            ("intro-1b", "missing"),
            ("intro-2", "current")
        ])
    );
}

#[test]
fn editing_one_lines_words_makes_only_its_take_stale() {
    let (dir, mut f) = recorded();
    f.set_line(
        "intro-2",
        LineField::Show,
        Some("This video was made by hand."),
    )
    .unwrap();
    assert_eq!(
        states(dir.path(), &f.text()),
        owned(&[("intro-1", "current"), ("intro-2", "stale")])
    );
}

#[test]
fn a_punctuation_or_capital_edit_keeps_the_take() {
    let (dir, mut f) = recorded();
    f.set_line("intro-1", LineField::Show, Some("hello and welcome!"))
        .unwrap();
    assert_eq!(
        states(dir.path(), &f.text()),
        owned(&[("intro-1", "current"), ("intro-2", "current")])
    );
}

#[test]
fn say_decides_what_is_said() {
    let (dir, mut f) = recorded();
    // Saying exactly what is shown changes nothing; saying other words does.
    f.set_line("intro-1", LineField::Say, Some("Hello and welcome"))
        .unwrap();
    assert_eq!(states(dir.path(), &f.text())[0].1, "current");
    f.set_line("intro-1", LineField::Say, Some("Hi there"))
        .unwrap();
    assert_eq!(states(dir.path(), &f.text())[0].1, "stale");
}

#[test]
fn undoing_an_edit_brings_the_take_back() {
    let (dir, mut f) = recorded();
    f.set_line(
        "intro-2",
        LineField::Show,
        Some("Different words entirely."),
    )
    .unwrap();
    f.set_line(
        "intro-2",
        LineField::Show,
        Some("This video was made with vidni."),
    )
    .unwrap();
    assert_eq!(
        states(dir.path(), &f.text()),
        owned(&[("intro-1", "current"), ("intro-2", "current")])
    );
}

#[test]
fn moving_and_removing_lines_keep_the_other_takes() {
    let (dir, mut f) = recorded();
    f.move_line("intro-2", None).unwrap();
    assert_eq!(
        states(dir.path(), &f.text()),
        owned(&[("intro-2", "current"), ("intro-1", "current")])
    );
    f.remove_line("intro-2").unwrap();
    assert_eq!(
        states(dir.path(), &f.text()),
        owned(&[("intro-1", "current")])
    );
}

#[test]
fn raw_takes_are_numbered_from_one_for_each_wording() {
    let dir = tempfile::tempdir().unwrap();
    let first = takes::next_raw(dir.path(), "intro-1", "00ff00ff00ff00ff");
    assert!(
        first.ends_with("takes/intro-1/raw/00ff00ff00ff00ff-1.wav"),
        "{}",
        first.display()
    );
    fs::create_dir_all(first.parent().unwrap()).unwrap();
    fs::write(&first, b"x").unwrap();
    assert!(
        takes::next_raw(dir.path(), "intro-1", "00ff00ff00ff00ff")
            .ends_with("00ff00ff00ff00ff-2.wav")
    );
    assert!(
        takes::next_raw(dir.path(), "intro-1", "1111111111111111")
            .ends_with("1111111111111111-1.wav")
    );
}
