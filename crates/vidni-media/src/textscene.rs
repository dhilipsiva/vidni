//! Terminal and code scenes (TODO.md, P5-VIDNI-02 and P5-VIDNI-03): a
//! transcript, or an excerpt of a source file, drawn as a window of text
//! that the virtual camera frames like any other still.
//!
//! - A **terminal** scene shows a transcript: a text file of what was
//!   typed and what was printed. A line that starts with the scene's
//!   prompt is a command. The colours a terminal's escape codes ask for
//!   are kept; every other escape code is left out.
//! - A **code** scene shows a file as it is in the project folder, or as
//!   it was at a commit of a git repository, with its syntax in colour.
//!
//! Nothing is run and nothing is browsed: the picture is drawn here, in
//! the pinned monospaced font, so it is the same on every machine.

use crate::capture::Frames;
use crate::compose::panel::{self, Panel, Row, Span};
use crate::compose::text::{Font, Fonts};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use vidni_core::capture::{self as keys, Files};
use vidni_core::project::{Scene, SceneKind};

/// The pinned fonts a window of text is drawn in.
const REGULAR: &str = "dejavu-sans-mono";
const BOLD: &str = "dejavu-sans-mono-bold";

/// The colours a terminal's escape codes name: 30 to 37, then 90 to 97.
const ANSI: [[u8; 3]; 16] = [
    [72, 78, 90],
    [237, 106, 94],
    [126, 231, 135],
    [245, 191, 79],
    [110, 168, 254],
    [210, 168, 255],
    [118, 227, 234],
    [214, 220, 230],
    [128, 138, 152],
    [255, 138, 128],
    [164, 244, 170],
    [255, 214, 110],
    [150, 196, 255],
    [228, 196, 255],
    [158, 240, 245],
    [255, 255, 255],
];

/// A line of a transcript as runs of colour: the colours its escape
/// codes ask for, and none of the codes.
fn coloured(line: &str) -> Vec<Span> {
    let mut spans = Vec::new();
    let (mut ink, mut bold) = (panel::INK, false);
    let mut run = String::new();
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\x1b' {
            run.push(c);
            continue;
        }
        // An escape: `ESC [`, its parameters, and the letter that ends it.
        if chars.next_if_eq(&'[').is_none() {
            continue;
        }
        let mut codes = String::new();
        let mut end = None;
        for c in chars.by_ref() {
            if ('\x40'..='\x7e').contains(&c) {
                end = Some(c);
                break;
            }
            codes.push(c);
        }
        if end != Some('m') {
            continue;
        }
        if !run.is_empty() {
            spans.push(Span {
                text: std::mem::take(&mut run),
                ink,
                bold,
            });
        }
        for code in codes.split(';') {
            match code.parse::<usize>().unwrap_or(0) {
                0 => (ink, bold) = (panel::INK, false),
                1 => bold = true,
                22 => bold = false,
                39 => ink = panel::INK,
                n @ 30..=37 => ink = ANSI[n - 30],
                n @ 90..=97 => ink = ANSI[n - 90 + 8],
                _ => {}
            }
        }
    }
    if !run.is_empty() || spans.is_empty() {
        spans.push(Span {
            text: run,
            ink,
            bold,
        });
    }
    spans
}

/// The lines `scene` shows of a file's `count` lines, as `first..=last`
/// counted from 1.
fn shown(scene: &Scene, count: usize, name: &str) -> Result<(u32, u32), String> {
    let count = count as u32;
    let (first, last) = match scene.lines {
        Some([first, last]) => (first, last),
        None => (1, count),
    };
    match count > 0 && last <= count {
        true => Ok((first, last)),
        false => Err(format!(
            "{name} has {count} lines, and the scene asks for {first} to {last}"
        )),
    }
}

/// The window's title: the scene's, or the file's name.
fn title(scene: &Scene) -> String {
    scene.title.clone().unwrap_or_else(|| {
        let name = scene.url.rsplit('/').next().unwrap_or(&scene.url);
        name.to_string()
    })
}

/// A transcript as a terminal's window.
pub fn terminal(text: &str, scene: &Scene, advance: f32) -> Result<Panel, String> {
    let lines: Vec<&str> = text.lines().collect();
    let (first, last) = shown(scene, lines.len(), &scene.url)?;
    let columns = scene.columns.unwrap_or(80);
    let prompt = scene.prompt.as_deref().unwrap_or("$ ");
    let mut rows = Vec::new();
    for number in first..=last {
        let line = lines[number as usize - 1].trim_end_matches('\r');
        let spans = match line.strip_prefix(prompt) {
            // What was typed: the prompt, and the command after it.
            Some(command) => vec![
                Span {
                    text: prompt.to_string(),
                    ink: panel::PROMPT,
                    bold: true,
                },
                Span {
                    text: command.to_string(),
                    ink: panel::BRIGHT,
                    bold: true,
                },
            ],
            None => coloured(line),
        };
        rows.extend(panel::wrap(spans, columns, number));
    }
    Ok(Panel {
        title: title(scene),
        rows,
        numbers: false,
        dots: true,
        columns,
        advance,
    })
}

/// A source file's excerpt as a window, with its syntax in colour.
pub fn code(text: &str, scene: &Scene, advance: f32) -> Result<Panel, String> {
    use syntect::easy::HighlightLines;
    use syntect::highlighting::{FontStyle, ThemeSet};
    use syntect::parsing::SyntaxSet;
    let name = scene.url.as_str();
    let lines: Vec<&str> = text.lines().collect();
    let (first, last) = shown(scene, lines.len(), name)?;
    let syntaxes = SyntaxSet::load_defaults_newlines();
    let syntax = match &scene.language {
        Some(language) => syntaxes.find_syntax_by_token(language).ok_or_else(|| {
            let mut known: Vec<&str> = syntaxes
                .syntaxes()
                .iter()
                .map(|syntax| syntax.name.as_str())
                .collect();
            known.sort_unstable();
            format!(
                "language {language:?} isn't one vidni colours; they are: {}",
                known.join(", ")
            )
        })?,
        // By the file's ending; a file of no known kind is plain text.
        None => name
            .rsplit('.')
            .next()
            .and_then(|ending| syntaxes.find_syntax_by_extension(ending))
            .unwrap_or_else(|| syntaxes.find_syntax_plain_text()),
    };
    let themes = ThemeSet::load_defaults();
    let theme = &themes.themes["base16-ocean.dark"];
    let mut highlighter = HighlightLines::new(syntax, theme);
    // The file is read from its top, so that a line knows it is inside a
    // comment or a string that began above the excerpt.
    let mut styled = Vec::new();
    for (n, line) in lines.iter().enumerate().take(last as usize) {
        let line = format!("{}\n", line.trim_end_matches('\r'));
        let parts = highlighter
            .highlight_line(&line, &syntaxes)
            .map_err(|e| format!("{name}, line {}: {e}", n + 1))?;
        if n as u32 + 1 < first {
            continue;
        }
        let spans: Vec<Span> = parts
            .into_iter()
            .map(|(style, text)| Span {
                text: text.trim_end_matches('\n').to_string(),
                ink: [style.foreground.r, style.foreground.g, style.foreground.b],
                bold: style.font_style.contains(FontStyle::BOLD),
            })
            .collect();
        styled.push((n as u32 + 1, spans));
    }
    let longest = styled
        .iter()
        .map(|(_, spans)| {
            let cells: usize = spans.iter().map(|span| span.text.chars().count()).sum();
            cells as u32
        })
        .max()
        .unwrap_or(0);
    let columns = scene.columns.unwrap_or(longest.clamp(40, 100));
    let rows: Vec<Row> = styled
        .into_iter()
        .flat_map(|(number, spans)| panel::wrap(spans, columns, number))
        .collect();
    Ok(Panel {
        title: title(scene),
        rows,
        numbers: true,
        dots: false,
        columns,
        advance,
    })
}

/// What `git <args>` prints, run in `repo`.
fn git(repo: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .map_err(|e| format!("git: {e}; a code scene with rev reads the file with git"))?;
    match out.status.success() {
        true => Ok(out.stdout),
        false => Err(format!(
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        )),
    }
}

/// The scene's text, the record of the project's files it was read from,
/// and what it was read at, for the capture's own record.
fn source(dir: &Path, scene: &Scene) -> Result<(String, Files, String), String> {
    let name = scene.url.as_str();
    let (bytes, files, at) = match (&scene.rev, scene.kind) {
        (Some(rev), SceneKind::Code) => {
            let repo = dir.join(scene.repo.as_deref().unwrap_or("."));
            // The commit the revision names, in full: a short hash that
            // names two things by now is refused by git itself.
            let commit = git(
                &repo,
                &["rev-parse", "--verify", &format!("{rev}^{{commit}}")],
            )?;
            let commit = String::from_utf8_lossy(&commit).trim().to_string();
            let bytes = git(&repo, &["show", &format!("{commit}:{name}")])?;
            (bytes, Files::new(), format!("{name} at {commit}"))
        }
        _ => {
            let bytes = fs::read(dir.join(name)).map_err(|e| format!("{name}: {e}"))?;
            let files: Files = [(name.to_string(), Some(keys::sha256_hex(&bytes)))].into();
            (bytes, files, name.to_string())
        }
    };
    let text = String::from_utf8(bytes).map_err(|_| format!("{name} isn't text in UTF-8"))?;
    Ok((text, files, at))
}

/// The pinned monospaced fonts, loaded.
fn faces(fonts: &mut Fonts) -> Result<(Font, Font), String> {
    let dirs = vidni_core::fonts::dirs();
    let mut load = |id: &str| {
        let font = vidni_core::fonts::get(id).ok_or_else(|| format!("no pinned font {id}"))?;
        fonts.load(&vidni_core::fonts::find(&font, &dirs)?)
    };
    Ok((load(REGULAR)?, load(BOLD)?))
}

/// What a terminal or code scene's capture is: its picture as a PNG's
/// image, the boxes of its targets and of the lines marked, and the
/// record of the files it was read from.
pub struct Still {
    pub image: crate::compose::image::Image,
    pub frames: Frames,
    pub files: Files,
}

/// Draws `scene`, with `highlight`'s targets marked.
pub fn still(dir: &Path, scene: &Scene, highlight: &[String]) -> Result<Still, String> {
    let (text, files, at) = source(dir, scene)?;
    let mut fonts = Fonts::new();
    let (regular, bold) = faces(&mut fonts)?;
    let advance = fonts.length(&regular, panel::FONT, "0");
    let panel = match scene.kind {
        SceneKind::Code => code(&text, scene, advance)?,
        _ => terminal(&text, scene, advance)?,
    };
    let boxes = |names: &mut dyn Iterator<Item = &String>| -> BTreeMap<String, Vec<[f64; 4]>> {
        names
            .filter_map(|name| {
                let target = scene.targets.get(name)?;
                Some((
                    name.clone(),
                    vec![panel.lines(target.lines?, target.columns)?],
                ))
            })
            .collect()
    };
    let targets = boxes(&mut scene.targets.keys());
    let highlights = boxes(&mut highlight.iter());
    let marked: Vec<[f64; 4]> = highlights.values().flatten().copied().collect();
    let scale = scene.scale.unwrap_or(1).max(1);
    let image = panel.render(&mut fonts, (&regular, &bold), scale, &marked);
    Ok(Still {
        image,
        frames: Frames {
            viewport: panel.size(),
            scale,
            targets,
            highlights,
            browser: format!("vidni {}; {at}", vidni_core::VERSION),
        },
        files,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn scene(kind: &str, more: serde_json::Value) -> Scene {
        let mut keys = serde_json::json!({ "kind": kind, "url": "a.rs" });
        for (key, value) in more.as_object().into_iter().flatten() {
            keys[key] = value.clone();
        }
        serde_json::from_value(keys).unwrap()
    }

    #[test]
    fn a_transcripts_commands_and_colours_are_told_apart() {
        let text = "$ ls\nplain \x1b[31mred\x1b[0m \x1b[1;92mbold green\x1b[K\x1b[0m end\n";
        let panel = terminal(text, &scene("terminal", json!({})), 12.0).unwrap();
        let spans = |row: usize| -> Vec<(&str, [u8; 3], bool)> {
            panel.rows[row]
                .spans
                .iter()
                .map(|s| (s.text.as_str(), s.ink, s.bold))
                .collect()
        };
        assert_eq!(
            spans(0),
            [("$ ", panel::PROMPT, true), ("ls", panel::BRIGHT, true)]
        );
        assert_eq!(
            spans(1),
            [
                ("plain ", panel::INK, false),
                ("red", ANSI[1], false),
                (" ", panel::INK, false),
                ("bold green", ANSI[10], true),
                (" end", panel::INK, false),
            ]
        );
        assert!(panel.dots && !panel.numbers);
        // Another prompt, and a part of the file.
        let nibli = scene(
            "terminal",
            json!({ "prompt": "~/nibli> ", "lines": [2, 2] }),
        );
        let panel = terminal("out\n~/nibli> dog(Adam).\nmore\n", &nibli, 12.0).unwrap();
        assert_eq!(panel.rows.len(), 1);
        assert_eq!(panel.rows[0].line, 2);
        assert_eq!(panel.rows[0].spans[1].text, "dog(Adam).");
        // Lines the file hasn't are said.
        let past = scene("terminal", json!({ "lines": [2, 9] }));
        let why = terminal("one\ntwo\n", &past, 12.0).unwrap_err();
        assert_eq!(why, "a.rs has 2 lines, and the scene asks for 2 to 9");
    }

    #[test]
    fn code_is_coloured_by_its_syntax_and_numbered_by_its_file() {
        let text = "// a comment\nfn main() {\n    let x = \"words\";\n}\n";
        let panel = code(text, &scene("code", json!({ "lines": [2, 3] })), 12.0).unwrap();
        assert!(panel.numbers && !panel.dots);
        assert_eq!(
            panel.rows.iter().map(|r| r.line).collect::<Vec<_>>(),
            [2, 3]
        );
        let line = |row: usize| -> String {
            panel.rows[row]
                .spans
                .iter()
                .map(|s| s.text.as_str())
                .collect()
        };
        assert_eq!(line(0), "fn main() {");
        assert_eq!(line(1), "    let x = \"words\";");
        // A keyword, a name and a string aren't one colour.
        let ink_of = |row: usize, word: &str| {
            let span = panel.rows[row].spans.iter().find(|s| s.text.contains(word));
            span.unwrap_or_else(|| panic!("{word}")).ink
        };
        let (keyword, name, string) = (ink_of(0, "fn"), ink_of(0, "main"), ink_of(1, "words"));
        assert!(keyword != name && name != string && keyword != string);
        // No shorter than 40 characters, however short the lines.
        assert_eq!(panel.columns, 40);
        // A language vidni doesn't know is said, with the ones it knows.
        let odd = scene("code", json!({ "language": "klingon" }));
        let why = code(text, &odd, 12.0).unwrap_err();
        assert!(why.starts_with("language \"klingon\" isn't one vidni colours; they are: "));
        assert!(why.contains("Rust"), "{why}");
        // A file with no known ending is plain text.
        let mut plain = scene("code", json!({}));
        plain.url = "notes".into();
        assert_eq!(code("a\nb\n", &plain, 12.0).unwrap().rows.len(), 2);
    }
}
