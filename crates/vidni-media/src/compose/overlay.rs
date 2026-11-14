//! Overlays (TODO.md, P5-VIDNI-05): a strip drawn over a line's picture
//! from one of three templates and the scene's parts, in the theme's
//! colours and faces.
//!
//! - `chips`: a sign, a word and a line under it, for each part; in one
//!   row on a 16:9 cut, and two to a row on a 9:16 one.
//! - `flow`: boxes joined by arrows, with walls between groups of them.
//!   On a 16:9 cut they stand in one row and a wall is two dashed lines
//!   between boxes; on a 9:16 cut each group is a row, and a wall is two
//!   dashed lines across with its words between them.
//! - `callout`: one box of words.
//!
//! Where things sit is the template's; a theme changes what they look
//! like. An overlay is drawn when a cut is composed, so it has no capture.

use super::cut::{Look, Type, wrap};
use super::image::Image;
use super::pillow;
use super::text::{Font, Fonts, Pen};
use vidni_core::project::{Part, Scene, Template, Tone};

/// How wide an overlay is, on a 16:9 cut and on a 9:16 one.
pub const WIDE: u32 = 1600;
pub const NARROW: u32 = 1000;

/// A tone's colour; the accent is the theme's.
fn tone(tone: Tone, look: &Look) -> [u8; 3] {
    match tone {
        Tone::Accent => look.accent,
        Tone::Good => [56, 227, 166],
        Tone::Bad => [255, 77, 94],
        Tone::Warn => [245, 181, 68],
        Tone::Info => [139, 124, 246],
    }
}

/// `a` with `part` of `b` mixed in.
fn mix(a: [u8; 3], b: [u8; 3], part: f32) -> [u8; 3] {
    [0, 1, 2].map(|c| (f32::from(a[c]) + (f32::from(b[c]) - f32::from(a[c])) * part).round() as u8)
}

/// What an overlay is drawn with.
struct Brush<'a> {
    fonts: &'a mut Fonts,
    faces: &'a Type,
    /// The face signs are set in: one that has them.
    signs: &'a Font,
    look: &'a Look,
    /// The strip's ground, a box's ground, and the lines between things.
    ground: [u8; 3],
    inner: [u8; 3],
    edge: [u8; 3],
}

impl Brush<'_> {
    fn text(
        &mut self,
        img: &mut Image,
        at: (f32, f32),
        text: &str,
        font: &Font,
        size: f32,
        ink: [u8; 3],
    ) {
        self.fonts
            .draw(img, at, text, Pen { font, size, ink }, "la");
    }

    /// A dashed line from `(x, y)`, `length` long, across or down.
    fn dashes(&self, img: &mut Image, (x, y): (i32, i32), length: i32, down: bool, ink: [u8; 3]) {
        let mut at = 0;
        while at < length {
            let end = (at + 7).min(length - 1);
            let dash = match down {
                true => (x, y + at, x + 1, y + end),
                false => (x + at, y, x + end, y + 1),
            };
            pillow::rectangle(img, dash, &ink);
            at += 13;
        }
    }

    /// A box with a small heading and words, at `(x, y)`, `w` by `h`.
    fn boxed(&mut self, img: &mut Image, part: &Part, (x, y, w, h): (i32, i32, i32, i32)) {
        pillow::rectangle(img, (x, y, x + w - 1, y + h - 1), &self.edge);
        pillow::rectangle(img, (x + 1, y + 1, x + w - 2, y + h - 2), &self.inner);
        if let Some(t) = part.tone {
            let ink = tone(t, self.look);
            pillow::rectangle(img, (x, y, x + w - 1, y + 2), &ink);
        }
        let pad = 18.0;
        let (mono, caption) = (self.faces.mono.clone(), self.faces.caption.clone());
        let label = part.label.as_deref().unwrap_or_default().to_uppercase();
        let words = part.text.as_deref().unwrap_or_default();
        let lines = wrap(self.fonts, &caption, 22.0, words, w as f32 - 2.0 * pad);
        // The heading and the words sit in the middle of the box's height.
        let heading = if label.is_empty() { 0.0 } else { 26.0 };
        let block = heading + lines.len() as f32 * 28.0;
        let mut top = y as f32 + ((h as f32 - block) / 2.0).max(8.0).round();
        if !label.is_empty() {
            let muted = self.look.muted;
            self.text(img, (x as f32 + pad, top), &label, &mono, 15.0, muted);
            top += heading;
        }
        for line in lines {
            let ink = self.look.ink;
            self.text(img, (x as f32 + pad, top), &line, &caption, 22.0, ink);
            top += 28.0;
        }
    }
}

/// The groups of boxes a flow's walls part, each with the wall after it.
fn groups(parts: &[Part]) -> Vec<(Vec<&Part>, Option<&str>)> {
    let mut groups: Vec<(Vec<&Part>, Option<&str>)> = vec![(Vec::new(), None)];
    for part in parts {
        match part.wall.as_deref() {
            Some(wall) => {
                groups.last_mut().expect("a group").1 = Some(wall);
                groups.push((Vec::new(), None));
            }
            None => groups.last_mut().expect("a group").0.push(part),
        }
    }
    groups
}

/// The size of `scene`'s overlay on a cut of that shape.
pub fn size(scene: &Scene, portrait: bool) -> (u32, u32) {
    let width = if portrait { NARROW } else { WIDE };
    let height = match (scene.template, portrait) {
        (Some(Template::Chips), false) => 150,
        (Some(Template::Chips), true) => 165 * scene.parts.len().div_ceil(2).max(1) as u32,
        (Some(Template::Flow), false) => 190,
        (Some(Template::Flow), true) => {
            let groups = groups(&scene.parts);
            let walls = groups.len().saturating_sub(1) as u32;
            (24 + groups.len() as u32 * 118 + walls * 44).max(330)
        }
        _ => 150,
    };
    (width, height)
}

/// Draws `scene`'s overlay for a cut of that shape. `signs` is a face
/// that has the signs chips are given.
pub fn render(
    scene: &Scene,
    portrait: bool,
    fonts: &mut Fonts,
    faces: &Type,
    signs: &Font,
    look: &Look,
) -> Image {
    let (width, height) = size(scene, portrait);
    let ground = mix(look.frame, look.ink, 0.08);
    let mut img = Image::new(width, height, &ground);
    let mut brush = Brush {
        fonts,
        faces,
        signs,
        look,
        ground,
        inner: look.frame,
        edge: mix(look.frame, look.ink, 0.18),
    };
    let (w, h) = (width as i32, height as i32);
    // The strip's own edge.
    let edge = brush.edge;
    pillow::rectangle(&mut img, (0, 0, w - 1, 0), &edge);
    pillow::rectangle(&mut img, (0, h - 1, w - 1, h - 1), &edge);
    pillow::rectangle(&mut img, (0, 0, 0, h - 1), &edge);
    pillow::rectangle(&mut img, (w - 1, 0, w - 1, h - 1), &edge);
    match scene.template {
        Some(Template::Chips) => {
            let across = if portrait {
                2
            } else {
                scene.parts.len().max(1) as i32
            };
            let rows = (scene.parts.len() as i32 + across - 1) / across;
            let (cell_w, cell_h) = (w / across, h / rows.max(1));
            for (n, part) in scene.parts.iter().enumerate() {
                let (col, row) = (n as i32 % across, n as i32 / across);
                let (x, y) = (col * cell_w, row * cell_h);
                // A line between a chip and the one before it, and above a row.
                if col > 0 {
                    pillow::rectangle(&mut img, (x, y + 1, x, y + cell_h - 2), &edge);
                }
                if row > 0 {
                    pillow::rectangle(&mut img, (x + 1, y, x + cell_w - 1, y), &edge);
                }
                let ink = part.tone.map_or(look.ink, |t| tone(t, look));
                let (left, top) = (x as f32 + 30.0, y as f32 + (cell_h as f32 - 74.0) / 2.0);
                let mut at = left;
                if let Some(sign) = part.sign.as_deref() {
                    let signs = brush.signs.clone();
                    brush.text(&mut img, (at, top), sign, &signs, 28.0, ink);
                    at += 34.0;
                }
                let mono = brush.faces.mono.clone();
                let label = part.label.as_deref().unwrap_or_default();
                brush.text(&mut img, (at, top), label, &mono, 28.0, ink);
                let caption = brush.faces.caption.clone();
                let under = part.text.as_deref().unwrap_or_default();
                let muted = look.muted;
                brush.text(&mut img, (left, top + 44.0), under, &caption, 22.0, muted);
            }
        }
        Some(Template::Flow) => {
            let groups = groups(&scene.parts);
            let pad = 24;
            let accent = look.accent;
            if portrait {
                // Each group is a row; a wall is two lines across, with
                // its words between them.
                let walls = groups.len() as i32 - 1;
                let row_h = (h - pad - walls * 44) / groups.len() as i32 - 12;
                let mut y = pad / 2 + 6;
                for (boxes, wall) in &groups {
                    row(&mut brush, &mut img, boxes, (pad, y, w - 2 * pad, row_h));
                    y += row_h + 6;
                    if let Some(words) = wall {
                        brush.dashes(&mut img, (pad, y + 4), w - 2 * pad, false, accent);
                        brush.dashes(&mut img, (pad, y + 36), w - 2 * pad, false, accent);
                        let mono = brush.faces.mono.clone();
                        let words = words.to_uppercase();
                        let length = brush.fonts.length(&mono, 15.0, &words);
                        let x = (w as f32 - length) / 2.0;
                        brush.text(&mut img, (x, y as f32 + 12.0), &words, &mono, 15.0, accent);
                        y += 44 + 6;
                    }
                }
            } else {
                // One row: every box as wide as the others, an arrow
                // between two of a group, and a wall between groups.
                let boxes: i32 = groups.iter().map(|(b, _)| b.len() as i32).sum();
                let arrows: i32 = groups
                    .iter()
                    .map(|(b, _)| (b.len() as i32 - 1).max(0))
                    .sum();
                let walls = groups.len() as i32 - 1;
                let (arrow_w, wall_w) = (40, 64);
                let box_w = (w - 2 * pad - arrows * arrow_w - walls * wall_w) / boxes.max(1);
                let (y, box_h) = (pad, h - 2 * pad);
                let mut x = pad;
                for (group, wall) in &groups {
                    row(
                        &mut brush,
                        &mut img,
                        group,
                        (
                            x,
                            y,
                            group.len() as i32 * box_w + (group.len() as i32 - 1).max(0) * arrow_w,
                            box_h,
                        ),
                    );
                    x += group.len() as i32 * box_w + (group.len() as i32 - 1).max(0) * arrow_w;
                    if wall.is_some() {
                        brush.dashes(&mut img, (x + 20, y), box_h, true, accent);
                        brush.dashes(&mut img, (x + 42, y), box_h, true, accent);
                        x += wall_w;
                    }
                }
            }
        }
        _ => {
            // A callout: one box, as wide as the strip.
            if let Some(part) = scene.parts.first() {
                brush.boxed(&mut img, part, (24, 20, w - 48, h - 40));
            }
        }
    }
    let _ = brush.ground;
    img.to_rgba()
}

/// A row of boxes in `(x, y, w, h)`, with an arrow between each two.
fn row(brush: &mut Brush, img: &mut Image, boxes: &[&Part], (x, y, w, h): (i32, i32, i32, i32)) {
    let arrow_w = 40;
    let count = boxes.len().max(1) as i32;
    let box_w = (w - (count - 1) * arrow_w) / count;
    for (n, part) in boxes.iter().enumerate() {
        let left = x + n as i32 * (box_w + arrow_w);
        brush.boxed(img, part, (left, y, box_w, h));
        if n + 1 < boxes.len() {
            let signs = brush.signs.clone();
            let muted = brush.look.muted;
            let at = ((left + box_w + 10) as f32, (y + h / 2 - 12) as f32);
            brush.text(img, at, "\u{2192}", &signs, 20.0, muted);
        }
    }
}
