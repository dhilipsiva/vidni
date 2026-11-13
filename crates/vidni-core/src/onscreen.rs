//! Checks of the text on screen.
//!
//! A capture reads the texts of the targets that `[[expectations]]` name, at
//! the setup or at a line, and writes them to `build/onscreen.json`. A
//! capture whose page doesn't show the expected text stops, so a video never
//! narrates a page that changed underneath it. Texts are compared with
//! their runs of whitespace collapsed, as the reference did.

use crate::project::{Expectation, Project};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Where a capture writes what it read, relative to the project folder.
pub const FILE: &str = "build/onscreen.json";

/// What a capture read from the page: the texts of a target's matches.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reading {
    pub scene: String,
    /// `setup`, or a line ID.
    pub at: String,
    pub target: String,
    /// The texts of the target's matches, in page order.
    pub texts: Vec<String>,
}

/// `build/onscreen.json`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Readings {
    pub readings: Vec<Reading>,
}

/// Reads a readings file.
pub fn load(path: &Path) -> Result<Readings, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Whether `texts`, a target's texts as read from the page, are what the
/// expectation `x` says, with runs of whitespace collapsed. Capture waits
/// for this before it takes a line's still.
pub fn holds(x: &Expectation, texts: &[String]) -> bool {
    let shown: Vec<String> = texts.iter().map(|t| collapse(t)).collect();
    match (&x.text, &x.texts) {
        (Some(text), _) => shown == [collapse(text)],
        (None, Some(wanted)) => shown == wanted.iter().map(|t| collapse(t)).collect::<Vec<_>>(),
        (None, None) => true,
    }
}

/// Compares every expectation with what the page showed; one message for
/// each that doesn't hold. An empty list means the page shows what the
/// script says it shows.
pub fn check(project: &Project, readings: &Readings) -> Vec<String> {
    problems(project.expectations.iter(), readings)
}

/// [`check`] for one scene's expectations.
pub fn check_scene(project: &Project, scene: &str, readings: &Readings) -> Vec<String> {
    problems(
        project.expectations.iter().filter(|x| x.scene == scene),
        readings,
    )
}

fn problems<'a>(
    expectations: impl Iterator<Item = &'a Expectation>,
    readings: &Readings,
) -> Vec<String> {
    let mut problems = Vec::new();
    for x in expectations {
        let place = if x.at == "setup" {
            format!("after scene {:?} loads", x.scene)
        } else {
            format!("at line {:?}", x.at)
        };
        let read = readings
            .readings
            .iter()
            .find(|r| r.scene == x.scene && r.at == x.at && r.target == x.target);
        let Some(read) = read else {
            problems.push(format!("{place}, target {:?} wasn't read", x.target));
            continue;
        };
        let shown: Vec<String> = read.texts.iter().map(|t| collapse(t)).collect();
        let wanted: Vec<String> = match (&x.text, &x.texts) {
            (Some(text), _) => vec![collapse(text)],
            (None, Some(texts)) => texts.iter().map(|t| collapse(t)).collect(),
            (None, None) => continue,
        };
        if shown == wanted {
            continue;
        }
        let message = match shown.iter().zip(&wanted).position(|(a, b)| a != b) {
            Some(i) if x.text.is_some() => {
                format!(
                    "{place}, target {:?} shows {:?}, not {:?}",
                    x.target, shown[i], wanted[i]
                )
            }
            Some(i) => format!(
                "{place}, target {:?} shows {:?} as text {} of {}, not {:?}",
                x.target,
                shown[i],
                i + 1,
                wanted.len(),
                wanted[i]
            ),
            None => format!(
                "{place}, target {:?} shows {} where the script expects {}",
                x.target,
                count(shown.len()),
                count(wanted.len())
            ),
        };
        problems.push(message);
    }
    problems
}

fn count(n: usize) -> String {
    format!("{n} text{}", if n == 1 { "" } else { "s" })
}
