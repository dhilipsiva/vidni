//! Review notes (TODO.md, P7-VIDNI-04): what to change about a line,
//! written while watching a draft. They are kept in `notes.md` in the
//! project folder, a heading for each line, so that they can be read and
//! written by hand as well, and go into git with the script.
//!
//! ```text
//! # Notes
//!
//! ## intro-2
//!
//! Slower, and say the name once.
//! ```

use std::fs;
use std::path::Path;

/// The notes' file, in the project folder.
pub const FILE: &str = "notes.md";

/// What a new `notes.md` starts with.
const HEAD: &str = "# Notes\n\nWhat to change, line by line. Each heading is a line's ID.\n";

/// A project's notes: the file's text, with each line's note under a
/// `## <line-id>` heading.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Notes {
    text: String,
}

/// The line a heading is for: `## intro-1` names `intro-1`.
fn heading(line: &str) -> Option<&str> {
    line.strip_prefix("## ")
        .map(str::trim)
        .filter(|id| !id.is_empty())
}

impl Notes {
    /// The notes in `text`.
    pub fn parse(text: &str) -> Self {
        Self {
            text: text.replace("\r\n", "\n"),
        }
    }

    /// The file's text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Every line that has a note, with it, in the file's order.
    pub fn all(&self) -> Vec<(String, String)> {
        let mut found: Vec<(String, Vec<&str>)> = Vec::new();
        for line in self.text.lines() {
            match (heading(line), found.last_mut()) {
                (Some(id), _) => found.push((id.to_string(), Vec::new())),
                // What is above the first heading is the file's own words.
                (None, Some((_, body))) => body.push(line),
                (None, None) => {}
            }
        }
        found
            .into_iter()
            .map(|(id, body)| (id, body.join("\n").trim().to_string()))
            .filter(|(_, note)| !note.is_empty())
            .collect()
    }

    /// The note on `line`; empty when it has none.
    pub fn of(&self, line: &str) -> String {
        self.all()
            .into_iter()
            .find(|(id, _)| id == line)
            .map(|(_, note)| note)
            .unwrap_or_default()
    }

    /// Sets the note on `line`, in the place of the one it had; an empty
    /// note takes the line's heading out. Everything else in the file
    /// stays as it is.
    pub fn set(&mut self, line: &str, note: &str) {
        // A line of the note that would read as a heading is set in by a
        // space, so that it stays part of the note.
        let note: Vec<String> = note
            .replace("\r\n", "\n")
            .trim()
            .lines()
            .map(|l| match heading(l) {
                Some(_) => format!(" {l}"),
                None => l.to_string(),
            })
            .collect();
        let note = note.join("\n");
        let note = note.as_str();
        let lines: Vec<&str> = self.text.lines().collect();
        // The line's section: from its heading to the next one.
        let start = lines.iter().position(|l| heading(l) == Some(line));
        let mut kept: Vec<String> = Vec::new();
        match start {
            Some(start) => {
                let end = lines[start + 1..]
                    .iter()
                    .position(|l| heading(l).is_some())
                    .map_or(lines.len(), |n| start + 1 + n);
                kept.extend(lines[..start].iter().map(|l| l.to_string()));
                if !note.is_empty() {
                    kept.extend([
                        format!("## {line}"),
                        String::new(),
                        note.to_string(),
                        String::new(),
                    ]);
                }
                kept.extend(lines[end..].iter().map(|l| l.to_string()));
            }
            None if note.is_empty() => return,
            None => {
                let head = match self.text.trim().is_empty() {
                    true => HEAD.trim_end(),
                    false => self.text.trim_end(),
                };
                kept.extend(head.lines().map(str::to_string));
                kept.extend([
                    String::new(),
                    format!("## {line}"),
                    String::new(),
                    note.to_string(),
                ]);
            }
        }
        self.text = kept.join("\n").trim_end().to_string() + "\n";
    }
}

/// The project's notes; none when it has no `notes.md`.
pub fn read(project_dir: &Path) -> Result<Notes, String> {
    let path = project_dir.join(FILE);
    match fs::read_to_string(&path) {
        Ok(text) => Ok(Notes::parse(&text)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Notes::default()),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}

/// Writes the project's notes.
pub fn write(project_dir: &Path, notes: &Notes) -> Result<(), String> {
    let path = project_dir.join(FILE);
    fs::write(&path, notes.text()).map_err(|e| format!("{}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_note_is_kept_under_its_lines_heading() {
        let mut notes = Notes::default();
        assert_eq!(notes.all(), []);
        notes.set("intro-2", "Slower, and say the name once.\n");
        assert_eq!(
            notes.text(),
            "# Notes\n\nWhat to change, line by line. Each heading is a line's ID.\n\n## intro-2\n\nSlower, and say the name once.\n"
        );
        notes.set("intro-1", "Two paragraphs.\n\nThe second.");
        assert_eq!(notes.of("intro-1"), "Two paragraphs.\n\nThe second.");
        assert_eq!(notes.of("intro-2"), "Slower, and say the name once.");
        assert_eq!(notes.of("intro-9"), "");
        // Another note takes the first one's place, where it was.
        notes.set("intro-2", "Faster.");
        let all: Vec<(String, String)> = notes.all();
        assert_eq!(all[0], ("intro-2".to_string(), "Faster.".to_string()));
        assert_eq!(all[1].0, "intro-1");
        // An empty note takes the heading out, and nothing else.
        notes.set("intro-2", "  ");
        assert_eq!(notes.all().len(), 1);
        assert!(!notes.text().contains("intro-2"));
        assert!(notes.text().starts_with("# Notes\n"));
        notes.set("intro-7", "");
        assert_eq!(notes.all().len(), 1);
    }

    #[test]
    fn what_was_written_by_hand_stays() {
        let by_hand = "My notes on the draft of 14 November.\n\n## intro-1\nToo fast.\n\n### a smaller heading stays in the note\n\n## end-1\n\n- one\n- two\n";
        let mut notes = Notes::parse(by_hand);
        assert_eq!(
            notes.all(),
            [
                (
                    "intro-1".to_string(),
                    "Too fast.\n\n### a smaller heading stays in the note".to_string()
                ),
                ("end-1".to_string(), "- one\n- two".to_string())
            ]
        );
        notes.set("end-1", "- one");
        notes.set("intro-5", "New.");
        assert_eq!(
            notes.text(),
            "My notes on the draft of 14 November.\n\n## intro-1\nToo fast.\n\n### a smaller heading stays in the note\n\n## end-1\n\n- one\n\n## intro-5\n\nNew.\n"
        );
    }
}
