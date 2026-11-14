//! Cards (TODO.md, P5-VIDNI-01): a scene drawn from one of five templates,
//! in the theme's colours and faces. Where things sit is the template's;
//! a theme changes only what they look like. A card is drawn at the cut's
//! own size, so it needs no capture.

use super::cut::{Layout, Look, Type, wrap};
use super::image::Image;
use super::pillow;
use super::text::{Font, Fonts, Pen};
use vidni_core::project::{Scene, Template};

/// What a card shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    pub template: Template,
    pub title: String,
    pub subtitle: String,
    pub items: Vec<String>,
    pub quote: String,
    pub by: String,
}

impl Card {
    /// The card a card scene describes; none for a scene with no template.
    pub fn of(scene: &Scene) -> Option<Self> {
        Some(Self {
            template: scene.template?,
            title: scene.title.clone().unwrap_or_default(),
            subtitle: scene.subtitle.clone().unwrap_or_default(),
            items: scene.items.clone(),
            quote: scene.quote.clone().unwrap_or_default(),
            by: scene.by.clone().unwrap_or_default(),
        })
    }
}

/// A block of text: its lines, and how they are set.
struct Block<'f> {
    lines: Vec<String>,
    font: &'f Font,
    size: f32,
    ink: [u8; 3],
    /// The distance from one line to the next.
    leading: f32,
    /// The space above the block.
    above: f32,
}

impl Block<'_> {
    fn height(&self) -> f32 {
        self.above + self.lines.len() as f32 * self.leading
    }
}

/// Draws a card for `layout`.
pub fn render(card: &Card, layout: &Layout, fonts: &mut Fonts, faces: &Type, look: &Look) -> Image {
    let (w, h) = (layout.size.0 as f32, layout.size.1 as f32);
    let portrait = h > w;
    // The margin at each side, and how much smaller type is on a 9:16 card.
    let (margin, k) = if portrait { (90.0, 0.8) } else { (160.0, 1.0) };
    let width = w - 2.0 * margin;
    let mut img = Image::new(layout.size.0, layout.size.1, &look.frame);
    let block =
        |fonts: &mut Fonts, text: &str, font, size: f32, ink, above: f32, width: f32| Block {
            lines: match text.trim().is_empty() {
                true => Vec::new(),
                false => wrap(fonts, font, size * k, text, width),
            },
            font,
            size: size * k,
            ink,
            leading: (size * k * 1.18).round(),
            above: above * k,
        };
    let centred = card.template == Template::End;
    // The card's blocks, top to bottom, and the indent of them all.
    let (blocks, indent): (Vec<Block>, f32) = match card.template {
        Template::Title => (
            vec![
                block(
                    fonts,
                    &card.title,
                    &faces.display,
                    104.0,
                    look.ink,
                    44.0,
                    width,
                ),
                block(
                    fonts,
                    &card.subtitle,
                    &faces.mono,
                    36.0,
                    look.muted,
                    30.0,
                    width,
                ),
            ],
            0.0,
        ),
        Template::Section => (
            vec![
                block(
                    fonts,
                    &card.subtitle,
                    &faces.mono,
                    32.0,
                    look.accent,
                    0.0,
                    width,
                ),
                block(
                    fonts,
                    &card.title,
                    &faces.display,
                    88.0,
                    look.ink,
                    22.0,
                    width,
                ),
            ],
            0.0,
        ),
        Template::End => (
            vec![
                block(
                    fonts,
                    &card.title,
                    &faces.display,
                    96.0,
                    look.ink,
                    44.0,
                    width,
                ),
                block(
                    fonts,
                    &card.subtitle,
                    &faces.mono,
                    36.0,
                    look.muted,
                    30.0,
                    width,
                ),
            ],
            0.0,
        ),
        Template::Bullets => {
            let mut blocks = vec![block(
                fonts,
                &card.title,
                &faces.display,
                68.0,
                look.ink,
                0.0,
                width,
            )];
            for (i, item) in card.items.iter().enumerate() {
                let above = if i == 0 { 56.0 } else { 26.0 };
                blocks.push(block(
                    fonts,
                    item,
                    &faces.caption,
                    46.0,
                    look.ink,
                    above,
                    width - 56.0,
                ));
            }
            (blocks, 56.0)
        }
        Template::Quote => (
            vec![
                block(
                    fonts,
                    &card.quote,
                    &faces.display,
                    64.0,
                    look.ink,
                    0.0,
                    width - 60.0,
                ),
                block(
                    fonts,
                    &card.by,
                    &faces.mono,
                    34.0,
                    look.muted,
                    40.0,
                    width - 60.0,
                ),
            ],
            60.0,
        ),
    };
    // The blocks sit in the middle of the card, as one.
    let total: f32 = blocks.iter().map(Block::height).sum();
    let top = ((h - total) / 2.0).round();
    let accent = |img: &mut Image, (x0, y0, x1, y1): (f32, f32, f32, f32)| {
        pillow::rectangle(
            img,
            (x0 as i32, y0 as i32, x1 as i32, y1 as i32),
            &look.accent,
        );
    };
    match card.template {
        // A short rule above the title; in the middle on a closing card.
        Template::Title => accent(&mut img, (margin, top, margin + 96.0, top + 7.0)),
        Template::End => accent(&mut img, (w / 2.0 - 48.0, top, w / 2.0 + 48.0, top + 7.0)),
        // A rule down the side of the quotation.
        Template::Quote => accent(&mut img, (margin, top, margin + 9.0, top + total)),
        Template::Section | Template::Bullets => {}
    }
    let mut y = top;
    for (i, block) in blocks.iter().enumerate() {
        y += block.above;
        // The title of a list isn't indented; its points are, each behind
        // a small square.
        let listed = card.template == Template::Bullets && i > 0;
        let x = match (card.template, listed) {
            (Template::Bullets, false) => margin,
            _ => margin + indent,
        };
        if listed && !block.lines.is_empty() {
            let mid = y + block.leading / 2.0;
            accent(
                &mut img,
                (margin + 6.0, mid - 7.0, margin + 20.0, mid + 7.0),
            );
        }
        for line in &block.lines {
            let pen = Pen {
                font: block.font,
                size: block.size,
                ink: block.ink,
            };
            let mid = y + block.leading / 2.0;
            match centred {
                true => fonts.draw(&mut img, (w / 2.0, mid), line, pen, "mm"),
                false => fonts.draw(&mut img, (x, mid), line, pen, "lm"),
            }
            y += block.leading;
        }
    }
    img
}
