//! The images that go with an upload (TODO.md, P6-VIDNI-04): thumbnails
//! for a 16:9 cut, and the cover of a reel. Each is drawn by the
//! compositor from what the cut itself shows, so it is the same picture
//! on every machine.

use crate::compose::card::{self, Card};
use crate::compose::cut::{Layout, Look, PORTRAIT, Type, wrap};
use crate::compose::image::Image;
use crate::compose::pillow;
use crate::compose::text::{Fonts, Pen};
use vidni_core::platforms::Insets;
use vidni_core::project::Template;

/// A thumbnail's size: 16:9, and well over the 640 px YouTube asks for.
pub const THUMBNAIL: (u32, u32) = (1280, 720);
/// The largest thumbnail file YouTube takes from a phone, in bytes.
pub const THUMBNAIL_BYTES: u64 = 2_000_000;

/// A thumbnail: `frame`, a 16:9 frame of the cut, at [`THUMBNAIL`], with
/// `title` over its lower left when there is one.
pub fn thumbnail(
    frame: &Image,
    title: Option<&str>,
    fonts: &mut Fonts,
    faces: &Type,
    look: &Look,
) -> Image {
    let whole = [0.0, 0.0, frame.width as f32, frame.height as f32];
    let mut picture = pillow::resize_bicubic(frame, THUMBNAIL, whole);
    let Some(title) = title.filter(|t| !t.trim().is_empty()) else {
        return picture;
    };
    // Large words in a panel: a thumbnail is seen small.
    let (size, leading, margin, pad) = (76.0, 90, 48, 28);
    let lines = wrap(fonts, &faces.display, size, title, 1000.0);
    let lines = &lines[..lines.len().min(3)];
    let width = lines
        .iter()
        .map(|l| fonts.length(&faces.display, size, l))
        .fold(0.0, f32::max) as i32;
    let (bw, bh) = (width + 2 * pad, lines.len() as i32 * leading + 2 * pad);
    let mut panel = Image::new(bw as u32, bh as u32, &[0, 0, 0, 0]);
    let [r, g, b] = look.frame;
    pillow::rounded_rectangle(
        &mut panel,
        (0, 0, bw - 1, bh - 1),
        18,
        &[r, g, b, 235],
        &look.edge,
    );
    for (i, line) in lines.iter().enumerate() {
        let y = (pad + i as i32 * leading + leading / 2) as f32;
        let pen = Pen {
            font: &faces.display,
            size,
            ink: look.ink,
        };
        fonts.draw(&mut panel, (pad as f32, y), line, pen, "lm");
    }
    pillow::paste_masked(
        &mut picture,
        &panel,
        margin,
        THUMBNAIL.1 as i32 - bh - margin,
    );
    picture
}

/// What a 3:4 crop of the middle of a 9:16 frame leaves out at the top
/// and at the bottom, in pixels: the profile grid shows that crop of a cover.
pub const GRID: u32 = 240;

/// A reel's cover: a title card of the cut's size, with its words inside
/// the 3:4 middle that the profile grid shows.
pub fn cover(title: &str, subtitle: &str, fonts: &mut Fonts, faces: &Type, look: &Look) -> Image {
    let layout = Layout {
        covered: Some(Insets {
            top: GRID,
            bottom: GRID,
            left: 0,
            right: 0,
        }),
        ..PORTRAIT
    };
    let card = Card {
        template: Template::Title,
        title: title.to_string(),
        subtitle: subtitle.to_string(),
        items: Vec::new(),
        quote: String::new(),
        by: String::new(),
    };
    card::render(&card, &layout, fonts, faces, look)
}
