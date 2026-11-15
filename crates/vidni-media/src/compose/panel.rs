//! A window of monospaced text, for terminal and code scenes (TODO.md,
//! P5-VIDNI-02 and P5-VIDNI-03): a bar with a title, rows of coloured
//! text, line numbers where they are wanted, and a band behind the rows
//! that are marked.
//!
//! Everything is laid out in whole and fractional CSS pixels and drawn
//! `scale` times as large, in the pinned monospaced font, so the picture
//! is the same wherever it is drawn.

use super::image::Image;
use super::text::{Font, Fonts, Pen};

/// The text's size and each row's height, in CSS pixels.
pub const FONT: f32 = 20.0;
pub const ROW: f32 = 30.0;
/// The space around the text, and the bar's height.
pub const PAD: f32 = 24.0;
pub const BAR: f32 = 44.0;

/// The window's colours.
pub const GROUND: [u8; 3] = [20, 22, 28];
pub const BAR_GROUND: [u8; 3] = [32, 35, 44];
pub const INK: [u8; 3] = [214, 220, 230];
pub const DIM: [u8; 3] = [128, 138, 152];
pub const BRIGHT: [u8; 3] = [255, 255, 255];
pub const PROMPT: [u8; 3] = [126, 231, 135];
pub const MARK: [u8; 3] = [255, 214, 10];

/// A run of text in one colour.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub text: String,
    pub ink: [u8; 3],
    pub bold: bool,
}

/// One row of the window: a line of the file, or a part of one that is
/// longer than the window is wide.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub spans: Vec<Span>,
    /// The file's line, counted from 1, and whether this row starts it.
    pub line: u32,
    pub first: bool,
}

/// A window of text.
#[derive(Debug, Clone, PartialEq)]
pub struct Panel {
    pub title: String,
    pub rows: Vec<Row>,
    /// Whether each line's number is shown.
    pub numbers: bool,
    /// Whether the bar has a terminal's three dots.
    pub dots: bool,
    /// How many characters wide the text is.
    pub columns: u32,
    /// One character's width at [`FONT`], in CSS pixels.
    pub advance: f32,
}

/// `line`, with each tab taken to the next multiple of four and whatever
/// isn't printable left out, cut into rows of `columns` characters.
pub fn wrap(spans: Vec<Span>, columns: u32, line: u32) -> Vec<Row> {
    let columns = columns.max(1) as usize;
    let mut rows = vec![Row {
        spans: Vec::new(),
        line,
        first: true,
    }];
    let mut column = 0;
    for span in spans {
        let mut run = String::new();
        let flush = |run: &mut String, rows: &mut Vec<Row>| {
            if !run.is_empty() {
                let text = std::mem::take(run);
                rows.last_mut().expect("a row").spans.push(Span {
                    text,
                    ..span.clone()
                });
            }
        };
        for c in span.text.chars() {
            let cells: Vec<char> = match c {
                '\t' => vec![' '; 4 - column % 4],
                c if c.is_control() => Vec::new(),
                c => vec![c],
            };
            for c in cells {
                if column == columns {
                    flush(&mut run, &mut rows);
                    rows.push(Row {
                        spans: Vec::new(),
                        line,
                        first: false,
                    });
                    column = 0;
                }
                run.push(c);
                column += 1;
            }
        }
        flush(&mut run, &mut rows);
    }
    rows
}

impl Panel {
    /// How wide the line numbers are, with the space after them.
    fn gutter(&self) -> f32 {
        if !self.numbers {
            return 0.0;
        }
        let last = self.rows.iter().map(|row| row.line).max().unwrap_or(1);
        (last.to_string().len() as f32 + 2.0) * self.advance
    }

    /// The window's size, in CSS pixels.
    pub fn size(&self) -> [u32; 2] {
        let width = PAD * 2.0 + self.gutter() + self.columns as f32 * self.advance;
        let height = BAR + PAD * 2.0 + self.rows.len() as f32 * ROW;
        [width.ceil() as u32, height.ceil() as u32]
    }

    /// The band across the window behind the file's lines `first` to
    /// `last`, as `[x, y, width, height]` in CSS pixels; none when none of
    /// them is shown. With `columns`, the band is those columns of the
    /// text, counted from 1, and no wider.
    pub fn lines(&self, [first, last]: [u32; 2], columns: Option<[u32; 2]>) -> Option<[f64; 4]> {
        let shown = |row: &&Row| (first..=last).contains(&row.line);
        let from = self.rows.iter().position(|row| shown(&row))?;
        let count = self.rows.iter().filter(shown).count();
        let [width, _] = self.size();
        let (x, across) = match columns {
            Some([a, b]) => (
                PAD + self.gutter() + a.saturating_sub(1) as f32 * self.advance,
                (b + 1).saturating_sub(a) as f32 * self.advance,
            ),
            None => (0.0, width as f32),
        };
        Some([
            f64::from(x),
            f64::from(BAR + PAD + from as f32 * ROW),
            f64::from(across),
            f64::from(count as f32 * ROW),
        ])
    }

    /// Draws the window `scale` times its size, with a band behind each
    /// of `marked`'s boxes (as [`Self::lines`] gives them).
    pub fn render(
        &self,
        fonts: &mut Fonts,
        (regular, bold): (&Font, &Font),
        scale: u32,
        marked: &[[f64; 4]],
    ) -> Image {
        let k = scale.max(1) as f32;
        let [width, height] = self.size();
        let mut image = Image::new(width * scale.max(1), height * scale.max(1), &GROUND);
        let mut fill = |[x, y, w, h]: [f32; 4], ink: [u8; 3], part: f32| {
            let (x0, y0) = ((x * k).round() as u32, (y * k).round() as u32);
            let x1 = (((x + w) * k).round() as u32).min(image.width);
            let y1 = (((y + h) * k).round() as u32).min(image.height);
            for py in y0..y1 {
                for px in x0..x1 {
                    let at = ((py * image.width + px) * 3) as usize;
                    for (c, over) in ink.iter().enumerate() {
                        let under = f32::from(image.data[at + c]);
                        let over = f32::from(*over);
                        image.data[at + c] = (under + (over - under) * part).round() as u8;
                    }
                }
            }
        };
        fill([0.0, 0.0, width as f32, BAR], BAR_GROUND, 1.0);
        // The marked rows: a band behind them, and a bar at their left.
        for band in marked {
            let [x, y, w, h] = band.map(|v| v as f32);
            fill([x, y, w, h], MARK, 0.16);
            fill([x, y, 5.0, h], MARK, 1.0);
        }
        if self.dots {
            // Three dots, as a terminal's window has: drawn as squares
            // with their corners off, which is all they are at this size.
            for (n, ink) in [[237, 106, 94], [245, 191, 79], [98, 197, 84]]
                .into_iter()
                .enumerate()
            {
                let x = PAD + n as f32 * 22.0;
                let y = BAR / 2.0 - 6.0;
                fill([x + 2.0, y, 8.0, 12.0], ink, 1.0);
                fill([x, y + 2.0, 12.0, 8.0], ink, 1.0);
                fill([x + 1.0, y + 1.0, 10.0, 10.0], ink, 1.0);
            }
        }
        let size = FONT * k;
        let title_x = if self.dots { PAD + 84.0 } else { PAD };
        fonts.draw(
            &mut image,
            (title_x * k, BAR / 2.0 * k),
            &self.title,
            Pen {
                font: regular,
                size,
                ink: DIM,
            },
            "lm",
        );
        let gutter = self.gutter();
        for (n, row) in self.rows.iter().enumerate() {
            let y = (BAR + PAD + (n as f32 + 0.5) * ROW) * k;
            if self.numbers && row.first {
                fonts.draw(
                    &mut image,
                    ((PAD + gutter - 2.0 * self.advance) * k, y),
                    &row.line.to_string(),
                    Pen {
                        font: regular,
                        size,
                        ink: DIM,
                    },
                    "rm",
                );
            }
            let mut column = 0usize;
            for span in &row.spans {
                // Each run starts on its own column, so a glyph that is
                // missing or wide in one run doesn't move the next.
                let x = (PAD + gutter + column as f32 * self.advance) * k;
                let font = if span.bold { bold } else { regular };
                if !span.text.trim().is_empty() {
                    fonts.draw(
                        &mut image,
                        (x, y),
                        &span.text,
                        Pen {
                            font,
                            size,
                            ink: span.ink,
                        },
                        "lm",
                    );
                }
                column += span.text.chars().count();
            }
        }
        image
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(text: &str) -> Span {
        Span {
            text: text.to_string(),
            ink: INK,
            bold: false,
        }
    }

    #[test]
    fn a_long_line_goes_on_in_the_next_row_and_a_tab_goes_to_a_stop() {
        let rows = wrap(vec![span("abcdef"), span("ghij")], 4, 7);
        let texts: Vec<Vec<&str>> = rows
            .iter()
            .map(|row| row.spans.iter().map(|s| s.text.as_str()).collect())
            .collect();
        assert_eq!(texts, [vec!["abcd"], vec!["ef", "gh"], vec!["ij"]]);
        assert_eq!(
            rows.iter().map(|r| (r.line, r.first)).collect::<Vec<_>>(),
            [(7, true), (7, false), (7, false)]
        );
        // A tab reaches the next multiple of four; an escape is left out.
        let rows = wrap(vec![span("a\tb\u{7}c")], 80, 1);
        assert_eq!(rows[0].spans[0].text, "a   bc");
        // An empty line is a row all the same.
        assert_eq!(wrap(vec![span("")], 80, 3).len(), 1);
    }

    #[test]
    fn the_window_is_as_large_as_its_text_and_lines_are_found_by_number() {
        let rows: Vec<Row> = (10..=13)
            .flat_map(|line| wrap(vec![span("x".repeat(30).as_str())], 20, line))
            .collect();
        let panel = Panel {
            title: "a file".into(),
            rows,
            numbers: true,
            dots: false,
            columns: 20,
            advance: 12.0,
        };
        // Four lines of two rows each; numbers of two digits, and two spaces.
        assert_eq!(panel.size(), [48 + 4 * 12 + 240, 44 + 48 + 8 * 30]);
        assert_eq!(
            panel.lines([11, 12], None),
            Some([0.0, 44.0 + 24.0 + 60.0, 336.0, 120.0])
        );
        assert_eq!(panel.lines([14, 20], None), None);
        // Columns 3 to 10 of those lines: after the numbers, eight wide.
        assert_eq!(
            panel.lines([11, 12], Some([3, 10])),
            Some([24.0 + 48.0 + 24.0, 44.0 + 24.0 + 60.0, 96.0, 120.0])
        );
    }
}
