//! Chapters (TODO.md, P6-VIDNI-03): a cut's sections as the chapter list
//! that YouTube reads from a video's description.
//!
//! YouTube shows chapters only when the list starts at 0:00, has at least
//! three chapters, and each lasts ten seconds or more; a list that breaks
//! a rule is ignored whole, without a word. So vidni makes a list only
//! from three chapters up, and QA holds it to the other rules.

use crate::platforms;
use crate::project::{Cut, Project, Section};
use crate::timeline::Timeline;
use serde::{Deserialize, Serialize};

/// The fewest chapters a list may have.
pub const FEWEST: usize = 3;
/// The shortest a chapter may be, in seconds.
pub const SHORTEST: u32 = 10;

/// One chapter: the second it starts at, and its title.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chapter {
    pub start: u32,
    pub title: String,
}

/// Whether `cut` is made for a platform that shows chapters.
pub fn wanted(cut: &Cut) -> bool {
    let ids = cut.platforms();
    platforms::all()
        .iter()
        .any(|p| p.chapters && ids.contains(&p.id))
}

/// The title a section gives its chapter: its `chapter`, or else its
/// label with a capital first letter. `None` when it has neither, or its
/// `chapter` is empty: then it stays in the chapter before it.
pub fn title(section: &Section) -> Option<String> {
    let title = match &section.chapter {
        Some(own) => own.trim().to_string(),
        None => {
            let mut chars = section.label.trim().chars();
            chars
                .next()
                .map(|c| c.to_uppercase().collect::<String>() + chars.as_str())
                .unwrap_or_default()
        }
    };
    (!title.is_empty()).then_some(title)
}

/// Each section of `project` with the title it gives its chapter, in the
/// file's order: what a cut's chapters are named from.
pub fn titles(project: &Project) -> Vec<(&str, Option<String>)> {
    project
        .sections
        .iter()
        .map(|(id, section)| (id.as_str(), title(section)))
        .collect()
}

/// The chapters of a cut with `timeline`: one for each run of sections
/// under one title. The first starts at 0, with the project's title when
/// its section has none; each later one starts at the whole second in
/// which the picture changes to its section, halfway through the pause
/// before the section's first line. Fewer than [`FEWEST`] is no list.
pub fn of(project: &Project, timeline: &Timeline) -> Vec<Chapter> {
    let mut chapters: Vec<Chapter> = Vec::new();
    let mut before: Option<(&str, f64)> = None;
    for entry in &timeline.sentences {
        if before.is_none_or(|(section, _)| section != entry.segment) {
            let own = project.sections.get(&entry.segment).and_then(title);
            let (start, title) = match (before, own) {
                (None, own) => (0, own.unwrap_or_else(|| project.meta.title.clone())),
                (Some((_, end)), Some(own)) => (((end + entry.start) / 2.0) as u32, own),
                (Some(_), None) => {
                    before = Some((&entry.segment, entry.end));
                    continue;
                }
            };
            if chapters.last().is_none_or(|last| last.title != title) {
                chapters.push(Chapter { start, title });
            }
        }
        before = Some((&entry.segment, entry.end));
    }
    match chapters.len() >= FEWEST {
        true => chapters,
        false => Vec::new(),
    }
}

/// How `chapters` break YouTube's rules in a video of `duration` seconds;
/// empty when they keep them all, and for an empty list.
pub fn problems(chapters: &[Chapter], duration: f64) -> Vec<String> {
    let mut found = Vec::new();
    let Some(first) = chapters.first() else {
        return found;
    };
    if first.start != 0 {
        found.push(format!(
            "the first chapter starts at {}, not at 0:00",
            platforms::clock(first.start)
        ));
    }
    if chapters.len() < FEWEST {
        found.push(format!(
            "the list has {} of the {FEWEST} chapters that YouTube shows a list from",
            chapters.len()
        ));
    }
    for (i, chapter) in chapters.iter().enumerate() {
        let end = chapters
            .get(i + 1)
            .map_or(duration, |next| f64::from(next.start));
        let lasts = end - f64::from(chapter.start);
        if lasts < f64::from(SHORTEST) {
            found.push(format!(
                "chapter {:?} lasts {} s; YouTube shows a video's chapters only when each \
                 lasts {SHORTEST} s or more",
                chapter.title,
                (lasts.max(0.0) * 10.0).floor() / 10.0
            ));
        }
    }
    found
}

/// The list as it is pasted into a description: `0:00 Title`, a line each.
pub fn text(chapters: &[Chapter]) -> String {
    chapters
        .iter()
        .map(|c| format!("{} {}\n", platforms::clock(c.start), c.title))
        .collect()
}

/// Reads a list back from [`text`]'s layout.
pub fn parse(text: &str) -> Result<Vec<Chapter>, String> {
    text.lines()
        .map(|line| {
            let (clock, title) = line
                .split_once(' ')
                .ok_or_else(|| format!("no title in {line:?}"))?;
            let seconds = clock.split(':').try_fold(0u32, |total, part| {
                part.parse::<u32>()
                    .map(|n| total * 60 + n)
                    .map_err(|_| format!("no time in {line:?}"))
            })?;
            Ok(Chapter {
                start: seconds,
                title: title.to_string(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timeline::Entry;

    /// A project with sections `a` to `e`: `b` has a label, `c` a chapter
    /// title of its own, `d` an empty one, and `a` and `e` nothing.
    fn project() -> Project {
        let text = crate::TEMPLATE.replace(
            "[sections.intro]\nlabel = \"Introduction\"\n",
            "[sections.intro]\nlabel = \"Introduction\"\n\n[sections.a]\n\n[sections.b]\n\
             label = \"how it works\"\n\n[sections.c]\nlabel = \"shown\"\nchapter = \"A closer look\"\n\n\
             [sections.d]\nlabel = \"an aside\"\nchapter = \"\"\n\n[sections.e]\n",
        );
        assert_ne!(text, crate::TEMPLATE);
        toml::from_str(&text).unwrap()
    }

    fn line(section: &str, start: f64, end: f64) -> Entry {
        Entry {
            k: 0,
            segment: section.into(),
            label: String::new(),
            index: 0,
            start,
            end,
            wav: String::new(),
            show: String::new(),
            say: String::new(),
            card: None,
            cues: Vec::new(),
        }
    }

    fn timeline(lines: &[(&str, f64, f64)]) -> Timeline {
        Timeline {
            total: lines.last().map_or(0.0, |l| l.2 + 2.0),
            rate: 22050,
            sentences: lines.iter().map(|(s, a, b)| line(s, *a, *b)).collect(),
        }
    }

    fn listed(chapters: &[Chapter]) -> Vec<(u32, &str)> {
        chapters
            .iter()
            .map(|c| (c.start, c.title.as_str()))
            .collect()
    }

    #[test]
    fn a_section_names_its_chapter_or_stays_in_the_one_before() {
        let p = project();
        let named: Vec<(&str, Option<String>)> = titles(&p);
        let named: Vec<(&str, Option<&str>)> =
            named.iter().map(|(id, t)| (*id, t.as_deref())).collect();
        assert_eq!(
            named,
            [
                ("a", None),
                ("b", Some("How it works")),
                ("c", Some("A closer look")),
                ("d", None),
                ("e", None),
                ("intro", Some("Introduction")),
            ]
        );
    }

    #[test]
    fn chapters_start_where_the_picture_changes_and_the_first_at_zero() {
        let p = project();
        let t = timeline(&[
            ("a", 0.6, 9.0),
            ("a", 9.3, 14.0),
            // The picture changes at 14.5 s, in second 14.
            ("b", 15.0, 30.0),
            ("c", 31.5, 50.0),
            // No title: `d` and `e` stay in "A closer look".
            ("d", 51.0, 60.0),
            ("e", 61.0, 70.0),
            ("intro", 71.4, 90.0),
        ]);
        // The first section has no title, so its chapter has the project's.
        assert_eq!(
            listed(&of(&p, &t)),
            [
                (0, "My first video"),
                (14, "How it works"),
                (30, "A closer look"),
                (70, "Introduction"),
            ]
        );
        assert_eq!(problems(&of(&p, &t), t.total), Vec::<String>::new());
        assert_eq!(
            text(&of(&p, &t)),
            "0:00 My first video\n0:14 How it works\n0:30 A closer look\n1:10 Introduction\n"
        );
        assert_eq!(parse(&text(&of(&p, &t))).unwrap(), of(&p, &t));
        // Two chapters are no list, and neither is a cut of one section.
        let two = timeline(&[("b", 0.6, 20.0), ("c", 21.0, 40.0), ("d", 41.0, 60.0)]);
        assert_eq!(of(&p, &two), []);
        assert_eq!(of(&p, &timeline(&[("b", 0.6, 20.0)])), []);
        // A section that comes back under the same title makes no new chapter
        // while it follows itself, and one when another came between.
        let back = timeline(&[
            ("b", 0.6, 20.0),
            ("d", 21.0, 30.0),
            ("b", 31.0, 45.0),
            ("c", 46.0, 60.0),
            ("b", 61.0, 80.0),
        ]);
        assert_eq!(
            listed(&of(&p, &back)),
            [
                (0, "How it works"),
                (45, "A closer look"),
                (60, "How it works")
            ]
        );
    }

    #[test]
    fn a_list_that_youtube_would_ignore_is_a_problem() {
        let chapter = |start, title: &str| Chapter {
            start,
            title: title.into(),
        };
        let list = [chapter(0, "One"), chapter(12, "Two"), chapter(18, "Three")];
        assert_eq!(
            problems(&list, 25.5),
            [
                "chapter \"Two\" lasts 6 s; YouTube shows a video's chapters only when each lasts 10 s or more",
                "chapter \"Three\" lasts 7.5 s; YouTube shows a video's chapters only when each lasts 10 s or more",
            ]
        );
        assert_eq!(problems(&list[..2], 40.0).len(), 1);
        let late = [chapter(3, "One"), chapter(20, "Two"), chapter(40, "Three")];
        assert_eq!(
            problems(&late, 60.0),
            ["the first chapter starts at 0:03, not at 0:00"]
        );
        assert_eq!(problems(&[], 60.0), Vec::<String>::new());
        assert!(parse("12 no colon\nnothing").is_err());
        assert_eq!(parse("1:02:03 Long").unwrap(), [chapter(3723, "Long")]);
    }

    #[test]
    fn only_a_cut_for_a_platform_with_chapters_wants_them() {
        let p = project();
        // The long cut is for YouTube; the vertical one for Shorts, Reels and WhatsApp.
        assert!(wanted(&p.cuts[0]));
        assert!(!wanted(&p.cuts[1]));
        let mut bare = p.cuts[0].clone();
        bare.platforms = Some(Vec::new());
        assert!(!wanted(&bare));
    }
}
