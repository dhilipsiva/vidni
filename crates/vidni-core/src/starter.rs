//! A new project's files (TODO.md, P2-VIDNI-08): what `vidni new` writes.
//!
//! A new project renders as it is, with no network: its one scene is a
//! page in its own folder. Given an outline, its script comes from that:
//! headings become sections, and paragraphs and list items become lines.

use crate::edit::ProjectFile;
use crate::validate::valid_id;
use crate::{TEMPLATE, fmt};
use std::fs;
use std::path::Path;

/// The project file's name.
pub const PROJECT_FILE: &str = "vidni.toml";
/// The page a new project's scene shows, and its name. `{title}` in it
/// becomes the project's title.
pub const PAGE_FILE: &str = "page.html";
const PAGE: &str = include_str!("starter-page.html");
/// What a new project keeps out of git: what vidni makes, and the takes,
/// which are large and are backed up another way (DECISIONS.md, D4).
pub const GITIGNORE: &str = "# What vidni makes, and your takes, stay out of git.\n\
                             build/\n\
                             out/\n\
                             takes/\n";

/// Where the template's script begins and ends: an outline's sections and
/// lines go between what comes before the first and from the second on.
const SCRIPT_BEGINS: &str =
    "# Sections group lines. Their labels are shown on screen and become chapters.\n";
const SCRIPT_ENDS: &str = "# What the lines show: here, a page under the virtual camera.";
const LINES_COMMENT: &str = "# The script, one narrated line at a time. A line's ID never changes: your\n\
                             # takes are filed under it.\n";

/// A title for a project named `slug`: `my-first-video` is `My first video`.
pub fn title_of(slug: &str) -> String {
    let words = slug.replace('-', " ");
    let mut chars = words.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// A script's outline: a title, and sections of lines.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Outline {
    /// The first heading, when it is a top-level one that comes before
    /// everything else.
    pub title: Option<String>,
    pub sections: Vec<Section>,
}

/// One heading of an outline, and the lines under it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    /// The heading; empty for the lines that come before any.
    pub heading: String,
    pub lines: Vec<String>,
}

/// A line of an outline without its inline marks: a link or an image is
/// its text, and bold, underline and code marks are dropped.
fn plain(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        // [text](address), with or without the ! of an image.
        let opens = if c == '!' && chars.get(i + 1) == Some(&'[') {
            Some(i + 1)
        } else if c == '[' {
            Some(i)
        } else {
            None
        };
        let link = opens.and_then(|open| {
            let close = (open + 1..chars.len()).find(|&j| chars[j] == ']')?;
            (chars.get(close + 1) == Some(&'(')).then_some(())?;
            let end = (close + 2..chars.len()).find(|&j| chars[j] == ')')?;
            Some((open, close, end))
        });
        if let Some((open, close, end)) = link {
            out.extend(&chars[open + 1..close]);
            i = end + 1;
            continue;
        }
        let pair = chars.get(i + 1) == Some(&c);
        match c {
            '`' => {}
            '*' | '_' if pair => i += 1,
            _ => out.push(c),
        }
        i += 1;
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The text of a list item, when `line` is one: `- `, `* `, `+ `, `1. ` or `1) `.
fn list_item(line: &str) -> Option<&str> {
    for bullet in ["- ", "* ", "+ "] {
        if let Some(item) = line.strip_prefix(bullet) {
            return Some(item);
        }
    }
    let digits = line.chars().take_while(char::is_ascii_digit).count();
    let rest = &line[digits..];
    (digits > 0)
        .then(|| rest.strip_prefix(". ").or_else(|| rest.strip_prefix(") ")))
        .flatten()
}

/// Reads an outline written in Markdown:
/// - a heading starts a section; the first one, when it is a `#` heading
///   before anything else, is the title;
/// - a paragraph is a line, and so is each list item;
/// - fenced code is left out, and a quote's `>` is dropped.
///
/// Sections with no lines are left out.
pub fn outline(markdown: &str) -> Outline {
    let mut outline = Outline::default();
    let mut paragraph: Vec<String> = Vec::new();
    let mut fenced = false;
    // Adds a line to the last section, or to one before any heading.
    fn add(outline: &mut Outline, line: String) {
        if line.is_empty() {
            return;
        }
        if outline.sections.is_empty() {
            outline.sections.push(Section {
                heading: String::new(),
                lines: Vec::new(),
            });
        }
        if let Some(section) = outline.sections.last_mut() {
            section.lines.push(line);
        }
    }
    let flush = |outline: &mut Outline, paragraph: &mut Vec<String>| {
        if !paragraph.is_empty() {
            add(outline, plain(&paragraph.join(" ")));
            paragraph.clear();
        }
    };
    for raw in markdown.lines() {
        let line = raw.trim();
        if line.starts_with("```") || line.starts_with("~~~") {
            flush(&mut outline, &mut paragraph);
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        let line = line.strip_prefix('>').map_or(line, str::trim);
        let level = line.chars().take_while(|c| *c == '#').count();
        let heading = (1..=6)
            .contains(&level)
            .then(|| line[level..].strip_prefix(' '))
            .flatten()
            .map(|text| plain(text.trim_end_matches('#')));
        if line.is_empty() {
            flush(&mut outline, &mut paragraph);
        } else if let Some(heading) = heading {
            flush(&mut outline, &mut paragraph);
            let first = outline.title.is_none() && outline.sections.is_empty();
            if level == 1 && first {
                outline.title = Some(heading);
            } else {
                outline.sections.push(Section {
                    heading,
                    lines: Vec::new(),
                });
            }
        } else if let Some(item) = list_item(line) {
            flush(&mut outline, &mut paragraph);
            add(&mut outline, plain(item));
        } else {
            paragraph.push(line.to_string());
        }
    }
    flush(&mut outline, &mut paragraph);
    outline.sections.retain(|section| !section.lines.is_empty());
    outline
}

/// An ID made of a heading's first words: `How it works, briefly` is
/// `how-it-works-briefly`. None when the heading has no letter or digit
/// that an ID can hold.
fn slug(heading: &str) -> Option<String> {
    let words: Vec<String> = heading
        .to_lowercase()
        .split(|c: char| !c.is_ascii_lowercase() && !c.is_ascii_digit())
        .filter(|word| !word.is_empty())
        .take(4)
        .map(str::to_string)
        .collect();
    let id = words.join("-");
    valid_id(&id).then_some(id)
}

/// A string as TOML writes it.
fn quoted(text: &str) -> String {
    toml_edit::Value::from(text).to_string().trim().to_string()
}

/// The project file for `outline`: the template, with the outline's
/// sections and lines as its script. Every line shows the template's page.
fn scripted(outline: &Outline) -> Result<String, String> {
    if outline.sections.is_empty() {
        return Err("the outline has no paragraph or list item to make a line of".into());
    }
    let (Some(begins), Some(ends)) = (TEMPLATE.find(SCRIPT_BEGINS), TEMPLATE.find(SCRIPT_ENDS))
    else {
        return Err("the template has lost the marks around its script".into());
    };
    let mut ids: Vec<String> = Vec::new();
    let mut sections = String::from(SCRIPT_BEGINS);
    let mut lines = String::from(LINES_COMMENT);
    for (n, section) in outline.sections.iter().enumerate() {
        // The lines before any heading are the introduction; a heading
        // with nothing an ID can hold is numbered.
        let base = match section.heading.is_empty() {
            true => "intro".to_string(),
            false => slug(&section.heading).unwrap_or_else(|| format!("section-{}", n + 1)),
        };
        let id = (1..)
            .map(|again| match again {
                1 => base.clone(),
                _ => format!("{base}-{again}"),
            })
            .find(|id| !ids.contains(id))
            .expect("some number is free");
        if n > 0 {
            sections.push('\n');
        }
        sections.push_str(&format!("[sections.{id}]\n"));
        if !section.heading.is_empty() {
            sections.push_str(&format!("label = {}\n", quoted(&section.heading)));
        }
        for (k, line) in section.lines.iter().enumerate() {
            if !(n == 0 && k == 0) {
                lines.push('\n');
            }
            lines.push_str(&format!(
                "[[lines]]\nid = \"{id}-{}\"\nsection = \"{id}\"\nshow = {}\n",
                k + 1,
                quoted(line)
            ));
        }
        ids.push(id);
    }
    Ok(format!(
        "{}{sections}\n{lines}\n{}",
        &TEMPLATE[..begins],
        &TEMPLATE[ends..]
    ))
}

/// `text` as HTML holds it.
fn escaped(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// A new project's files, by name: its `vidni.toml`, the page its scene
/// shows, and a `.gitignore`. `title` is used unless the outline has one.
pub fn files(
    title: &str,
    outline: Option<&Outline>,
) -> Result<Vec<(&'static str, String)>, String> {
    let title = outline
        .and_then(|o| o.title.as_deref())
        .unwrap_or(title)
        .trim();
    if title.is_empty() {
        return Err("a project needs a title".into());
    }
    let text = match outline {
        Some(outline) => scripted(outline)?,
        None => TEMPLATE.to_string(),
    };
    // The file is opened as any project is, so a script that isn't valid
    // stops here, with its problems.
    let listed = |problems: Vec<crate::validate::Diagnostic>| {
        let problems: Vec<String> = problems.iter().map(ToString::to_string).collect();
        format!("the new project isn't valid: {}", problems.join("; "))
    };
    let mut project = ProjectFile::parse(&text).map_err(listed)?;
    project.set_title(title).map_err(|e| e.to_string())?;
    let text = fmt::format(&project.text()).map_err(listed)?;
    Ok(vec![
        (PROJECT_FILE, text),
        (PAGE_FILE, PAGE.replace("{title}", &escaped(title))),
        (".gitignore", GITIGNORE.to_string()),
    ])
}

/// Writes a new project's `files` into `dir`, which must not exist yet.
pub fn write(dir: &Path, files: &[(&'static str, String)]) -> Result<(), String> {
    if dir.exists() {
        return Err(format!("{} already exists", dir.display()));
    }
    fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for (name, text) in files {
        let file = dir.join(name);
        fs::write(&file, text).map_err(|e| format!("{}: {e}", file.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate;

    #[test]
    fn a_new_project_is_the_template_with_its_title_and_page() {
        let files = files("Demo", None).unwrap();
        let names: Vec<&str> = files.iter().map(|(name, _)| *name).collect();
        assert_eq!(names, ["vidni.toml", "page.html", ".gitignore"]);
        assert_eq!(
            files[0].1,
            TEMPLATE.replace("title = \"My first video\"", "title = \"Demo\"")
        );
        assert!(files[1].1.contains("<h1>Demo</h1>") && files[1].1.contains("<title>Demo</title>"));
        assert_eq!(
            files[2].1.lines().skip(1).collect::<Vec<_>>(),
            ["build/", "out/", "takes/"]
        );
        // Written to a folder, it is a valid project, in the canonical layout.
        let dir = tempfile::tempdir().unwrap();
        let project = dir.path().join("demo");
        write(&project, &files).unwrap();
        let text = fs::read_to_string(project.join(PROJECT_FILE)).unwrap();
        assert!(validate::check(&text, Some(&project)).is_ok());
        assert_eq!(fmt::format(&text).unwrap(), text);
        // A folder that is there already is left alone.
        let error = write(&project, &files).unwrap_err();
        assert!(error.ends_with("demo already exists"), "{error}");
    }

    #[test]
    fn a_title_comes_from_the_projects_name_and_is_written_safely() {
        assert_eq!(title_of("demo"), "Demo");
        assert_eq!(title_of("my-first-video"), "My first video");
        let files = files("Q&A: \"why\" <now>?", None).unwrap();
        assert!(
            files[0].1.contains("title = 'Q&A: \"why\" <now>?'"),
            "{}",
            files[0].1
        );
        assert!(
            files[1]
                .1
                .contains("<h1>Q&amp;A: &quot;why&quot; &lt;now&gt;?</h1>"),
            "{}",
            files[1].1
        );
        assert!(super::files("  ", None).is_err());
    }

    const OUTLINE: &str = "# How vidni works

A short tour, in three parts.

## Write the script

Each line is one sentence to narrate.
It can run over
two lines of the outline.

- A list item is a line too.
- So is [a link](https://example.com), as its **text**.

```sh
vidni draft   # code isn't narrated
```

## Draft, then record!

1. Draft with the synthetic voice.
2) Record `your own` takes.

## An empty section

## Write the script

> A second section of the same name gets its own ID.
";

    #[test]
    fn an_outlines_headings_are_sections_and_its_paragraphs_lines() {
        let outline = outline(OUTLINE);
        assert_eq!(outline.title.as_deref(), Some("How vidni works"));
        let sections: Vec<(&str, Vec<&str>)> = outline
            .sections
            .iter()
            .map(|s| {
                (
                    s.heading.as_str(),
                    s.lines.iter().map(String::as_str).collect(),
                )
            })
            .collect();
        assert_eq!(
            sections,
            [
                ("", vec!["A short tour, in three parts."]),
                (
                    "Write the script",
                    vec![
                        "Each line is one sentence to narrate. It can run over two lines of the outline.",
                        "A list item is a line too.",
                        "So is a link, as its text."
                    ]
                ),
                (
                    "Draft, then record!",
                    vec!["Draft with the synthetic voice.", "Record your own takes."]
                ),
                (
                    "Write the script",
                    vec!["A second section of the same name gets its own ID."]
                ),
            ]
        );
    }

    #[test]
    fn a_project_from_an_outline_is_valid_and_canonical() {
        let files = files("ignored", Some(&outline(OUTLINE))).unwrap();
        let text = &files[0].1;
        assert_eq!(&fmt::format(text).unwrap(), text);
        let project = validate::check(text, None).unwrap_or_else(|p| panic!("{p:?}\n{text}"));
        assert_eq!(project.meta.title, "How vidni works");
        let sections: Vec<(&str, &str)> = project
            .lines
            .iter()
            .map(|l| (l.id.as_str(), l.section.as_str()))
            .collect();
        assert_eq!(
            sections,
            [
                ("intro-1", "intro"),
                ("write-the-script-1", "write-the-script"),
                ("write-the-script-2", "write-the-script"),
                ("write-the-script-3", "write-the-script"),
                ("draft-then-record-1", "draft-then-record"),
                ("draft-then-record-2", "draft-then-record"),
                ("write-the-script-2-1", "write-the-script-2"),
            ]
        );
        assert_eq!(
            project.sections["draft-then-record"].label,
            "Draft, then record!"
        );
        assert_eq!(project.sections["intro"].label, "");
        assert_eq!(project.lines[5].show, "Record your own takes.");
        // The template's page, cuts and voice are kept; its own lines are gone.
        assert!(text.contains("url = \"page.html\"") && text.contains("id = \"vertical\""));
        assert!(!text.contains("Hello, and welcome."));
        assert!(files[1].1.contains("<h1>How vidni works</h1>"));
    }

    #[test]
    fn an_outline_needs_a_line_and_copes_with_odd_headings() {
        assert_eq!(
            files(
                "Demo",
                Some(&outline("# Only a title\n\n## And a heading\n"))
            )
            .unwrap_err(),
            "the outline has no paragraph or list item to make a line of"
        );
        // A heading with nothing an ID can hold is numbered, and a quote
        // in a line is written as TOML needs it.
        let odd = outline("## தமிழ்\n\nShe said \"hello\".\n\n## 2026\n\nA year.\n");
        let files = files("Demo", Some(&odd)).unwrap();
        let project = validate::check(&files[0].1, None).unwrap();
        let ids: Vec<&str> = project.lines.iter().map(|l| l.id.as_str()).collect();
        assert_eq!(ids, ["section-1-1", "2026-1"]);
        assert_eq!(project.lines[0].show, "She said \"hello\".");
        assert_eq!(project.sections["section-1"].label, "தமிழ்");
    }
}
