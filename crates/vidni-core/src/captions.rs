//! Caption QA (TODO.md, P6-VIDNI-05): whether a cut's cues can be read.
//!
//! The rules are the ones subtitlers work to: at most two lines of 42
//! characters, at most 20 characters a second, at least 0.83 s on screen
//! (20 frames at 24 a second), and ink that stands out from what is
//! behind it by 4.5 to 1, as WCAG asks of text.

use crate::timeline::Timeline;

/// The most characters in a line of a cue, and the most lines.
pub const LINE: usize = 42;
pub const LINES: usize = 2;
/// The most characters a second a cue may ask a reader for.
pub const SPEED: f64 = 20.0;
/// The shortest a cue may be shown, in seconds.
pub const SHORTEST: f64 = 0.83;
/// The least contrast between a caption's ink and what is behind it.
pub const CONTRAST: f64 = 4.5;

/// How many letters `text` has, as a reader counts them: a mark that is
/// written onto the letter before it (an accent, or a vowel sign of a
/// script such as Tamil) isn't a letter of its own, and neither is a
/// joiner that isn't seen.
pub fn letters(text: &str) -> usize {
    let mark = |c: char| {
        matches!(u32::from(c),
            0x0300..=0x036F          // combining accents
            | 0x0900..=0x0903 | 0x093A..=0x094F  // Devanagari signs
            | 0x0B82 | 0x0BBE..=0x0BCD | 0x0BD7  // Tamil signs
            | 0x200B..=0x200D        // joiners
        )
    };
    text.chars().filter(|c| !mark(*c)).count()
}

/// `text` in lines of at most `width` characters, broken at spaces; a
/// word longer than that has a line to itself.
pub fn wrapped(text: &str, width: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        match lines.last_mut() {
            Some(line) if letters(line) + 1 + letters(word) <= width => {
                line.push(' ');
                line.push_str(word);
            }
            _ => lines.push(word.to_string()),
        }
    }
    lines
}

/// Whether `text` fits a cue: [`LINES`] lines of [`LINE`] characters.
pub fn fits(text: &str) -> bool {
    let lines = wrapped(text, LINE);
    lines.len() <= LINES && lines.iter().all(|l| letters(l) <= LINE)
}

/// One cue: when it shows, and its words.
#[derive(Debug, Clone, PartialEq)]
pub struct Shown {
    pub start: f64,
    pub end: f64,
    pub text: String,
}

/// The cues of `timeline`, in order.
pub fn of(timeline: &Timeline) -> Vec<Shown> {
    timeline
        .sentences
        .iter()
        .flat_map(|entry| &entry.cues)
        .map(|cue| Shown {
            start: cue.start,
            end: cue.end,
            text: cue.text.clone(),
        })
        .collect()
}

/// The cues of an SRT file.
pub fn srt(text: &str) -> Result<Vec<Shown>, String> {
    let seconds = |clock: &str| -> Option<f64> {
        let (hms, ms) = clock.trim().split_once(',')?;
        let mut parts = hms.split(':').map(|p| p.parse::<f64>().ok());
        let (h, m, s) = (parts.next()??, parts.next()??, parts.next()??);
        Some(h * 3600.0 + m * 60.0 + s + ms.parse::<f64>().ok()? / 1000.0)
    };
    let mut cues = Vec::new();
    for block in text.replace("\r\n", "\n").split("\n\n") {
        let mut lines = block.lines().skip_while(|l| !l.contains("-->"));
        let Some(times) = lines.next() else { continue };
        let (start, end) = times
            .split_once("-->")
            .and_then(|(a, b)| Some((seconds(a)?, seconds(b)?)))
            .ok_or_else(|| format!("no times in {times:?}"))?;
        cues.push(Shown {
            start,
            end,
            text: lines.collect::<Vec<_>>().join(" "),
        });
    }
    Ok(cues)
}

/// How `cues` break the rules of reading, a problem for each rule broken,
/// with the first cue that breaks it and how many do.
pub fn problems(cues: &[Shown]) -> Vec<String> {
    let mut found = Vec::new();
    let mut rule = |broken: Vec<(usize, String)>, what: &str| {
        if let Some((n, detail)) = broken.first() {
            let others = match broken.len() - 1 {
                0 => String::new(),
                1 => " (and 1 more)".to_string(),
                more => format!(" (and {more} more)"),
            };
            found.push(format!("caption {}{others} {what}: {detail}", n + 1));
        }
    };
    let numbered = || cues.iter().enumerate();
    rule(
        numbered()
            .filter(|(_, c)| !fits(&c.text))
            .map(|(n, c)| {
                let lines = wrapped(&c.text, LINE);
                (
                    n,
                    format!("{} characters, in {} lines", letters(&c.text), lines.len()),
                )
            })
            .collect(),
        "is more than 2 lines of 42 characters",
    );
    rule(
        numbered()
            // A twentieth is allowed for the times' rounding to milliseconds.
            .filter(|(_, c)| {
                c.end > c.start && letters(&c.text) as f64 / (c.end - c.start) > SPEED + 0.05
            })
            .map(|(n, c)| {
                let speed = letters(&c.text) as f64 / (c.end - c.start);
                (n, format!("{speed:.1} characters a second"))
            })
            .collect(),
        "is faster than 20 characters a second",
    );
    rule(
        numbered()
            // A hundredth is allowed for the times' rounding to milliseconds.
            .filter(|(_, c)| c.end - c.start < SHORTEST - 0.005)
            .map(|(n, c)| (n, format!("{:.2} s", c.end - c.start)))
            .collect(),
        "is shown for less than 0.83 s",
    );
    found
}

/// The contrast between two colours, from 1 to 21, as WCAG 2 defines it.
pub fn contrast(a: [u8; 3], b: [u8; 3]) -> f64 {
    let luminance = |[r, g, b]: [u8; 3]| {
        let linear = |c: u8| {
            let c = f64::from(c) / 255.0;
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b)
    };
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

/// The colour of `over` (with its alpha) laid over `under`.
pub fn over(over: [u8; 4], under: [u8; 3]) -> [u8; 3] {
    let alpha = f64::from(over[3]) / 255.0;
    [0, 1, 2]
        .map(|i| (f64::from(over[i]) * alpha + f64::from(under[i]) * (1.0 - alpha)).round() as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cue(start: f64, end: f64, text: &str) -> Shown {
        Shown {
            start,
            end,
            text: text.into(),
        }
    }

    #[test]
    fn a_cue_is_two_lines_of_42_characters_at_most() {
        assert_eq!(
            wrapped(
                "The quick brown fox jumps over the lazy dog, and then it sleeps.",
                42
            ),
            [
                "The quick brown fox jumps over the lazy",
                "dog, and then it sleeps."
            ]
        );
        assert!(fits("Short."));
        assert!(fits(&"word ".repeat(16)));
        // 85 characters can't be two lines of 42.
        assert!(!fits(&"a".repeat(85)));
        // Nor can 80 that break badly.
        assert!(!fits(
            "Forty-one characters make this one line!! and-a-word-of-thirty-five-characters-here x"
        ));
    }

    #[test]
    fn each_rule_names_the_first_cue_that_breaks_it() {
        let cues = [
            cue(0.0, 3.0, "A fine cue."),
            // 60 characters in 2 seconds: 30 a second.
            cue(3.0, 5.0, &"quick ".repeat(10)),
            cue(5.0, 5.5, "Gone."),
            cue(6.0, 16.0, &"a long cue that goes on ".repeat(5)),
            cue(16.0, 16.4, "Too."),
        ];
        assert_eq!(
            problems(&cues),
            [
                "caption 4 is more than 2 lines of 42 characters: 120 characters, in 3 lines",
                "caption 2 is faster than 20 characters a second: 30.0 characters a second",
                "caption 3 (and 1 more) is shown for less than 0.83 s: 0.50 s",
            ]
        );
        assert_eq!(problems(&cues[..1]), Vec::<String>::new());
        assert_eq!(
            problems(&[cue(0.0, 0.83, "Just right.")]),
            Vec::<String>::new()
        );
    }

    #[test]
    fn contrast_is_wcags() {
        assert!((contrast([255, 255, 255], [0, 0, 0]) - 21.0).abs() < 1e-9);
        assert!((contrast([0, 0, 0], [0, 0, 0]) - 1.0).abs() < 1e-9);
        // WCAG's own example: #777 on white is 4.48, just under 4.5.
        assert!((contrast([0x77, 0x77, 0x77], [255, 255, 255]) - 4.48).abs() < 0.01);
        assert_eq!(over([0, 0, 0, 128], [255, 255, 255]), [127, 127, 127]);
        assert_eq!(over([10, 20, 30, 255], [255, 255, 255]), [10, 20, 30]);
    }

    #[test]
    fn an_srt_files_cues_are_read_back() {
        let text = "1\n00:00:00,600 --> 00:00:03,240\nOne line,\nand a second.\n\n2\n00:01:03,000 --> 00:01:04,500\nTwo.\n";
        assert_eq!(
            srt(text).unwrap(),
            [
                cue(0.6, 3.24, "One line, and a second."),
                cue(63.0, 64.5, "Two.")
            ]
        );
        assert!(srt("1\nsoon --> later\nWhat?\n").is_err());
    }

    #[test]
    fn the_references_captions_fail() {
        // golden/ holds the captions of the reference pipeline's cuts.
        let golden = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../golden/nibli-syllogism-voice.en.srt");
        let cues = srt(&std::fs::read_to_string(golden).unwrap()).unwrap();
        let found = problems(&cues);
        assert!(
            found
                .iter()
                .any(|p| p.contains("is more than 2 lines of 42 characters")),
            "{found:?}"
        );
    }
}
