//! A cut's frames: the reference compositor (`reference/compose.py`, class
//! `Cut`), ported. A virtual camera eases between crops of each line's
//! capture, pictures crossfade between lines, and section labels, overlays
//! and captions are drawn on top.
//!
//! The geometry, colours and type are the reference's (the QUINE look of
//! the nibli example); P5-VIDNI-01 moves them into theme files.

use super::image::Image;
use super::pillow;
use super::text::{Font, Fonts, Pen};
use std::path::PathBuf;
use vidni_core::timeline::Timeline;

/// Frames per second.
pub const FPS: f64 = 30.0;
/// Seconds a camera move takes.
const CAM_EASE: f64 = 0.7;
/// Seconds a crossfade takes.
const XFADE: f64 = 0.35;
/// Where 16:9 overlays sit, below the section label.
const OVERLAY_Y: i32 = 100;

const VOID: [u8; 3] = [11, 11, 16];
const LINE: [u8; 3] = [42, 42, 56];
const INK: [u8; 3] = [244, 243, 239];
const MUTED: [u8; 3] = [144, 143, 154];
const EMBER: [u8; 3] = [242, 84, 45];

/// A cut's frame and where its screen view sits.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    /// `16x9` or `9x16`.
    pub name: &'static str,
    pub size: (u32, u32),
    /// The screen view: x, y, width, height.
    pub view: (u32, u32, u32, u32),
    /// The narrowest crop, in CSS pixels.
    pub min_w: f64,
    /// The share of the view kept clear for captions.
    pub reserve: f64,
}

/// The 16:9 cut: the view fills the frame.
pub const LANDSCAPE: Layout = Layout {
    name: "16x9",
    size: (1920, 1080),
    view: (0, 0, 1920, 1080),
    min_w: 640.0,
    reserve: 0.2,
};
/// The 9:16 cut: a title band, the view, and captions below.
pub const PORTRAIT: Layout = Layout {
    name: "9x16",
    size: (1080, 1920),
    view: (0, 330, 1080, 1170),
    min_w: 400.0,
    reserve: 0.0,
};

/// The faces the look uses.
pub struct Type {
    pub caption: Font,
    pub mono: Font,
    pub display: Font,
}

/// What one line shows in this cut.
#[derive(Debug, Clone)]
pub struct Shot {
    /// The section's label ("" for none).
    pub label: String,
    /// A web capture and the camera's rectangle in CSS pixels, or a full-frame card.
    pub picture: Picture,
    /// An RGBA overlay for this cut's aspect.
    pub overlay: Option<PathBuf>,
}

/// What a line shows: a crop of a web capture, or a full-frame card.
#[derive(Debug, Clone)]
pub enum Picture {
    /// `camera` is the framing in CSS pixels, on a page `viewport` CSS
    /// pixels big, captured at `scale` device pixels per CSS pixel.
    Still {
        path: PathBuf,
        camera: [f64; 4],
        viewport: [f64; 2],
        scale: f64,
    },
    Card(PathBuf),
}

/// A crop (CSS pixels) of the cut's aspect that frames `target` between
/// the top reserve (an overlay) and the bottom reserve (captions).
pub fn crop_rect(
    target: [f64; 4],
    aspect: f64,
    layout: &Layout,
    (vw, vh): (f64, f64),
    top_reserve: f64,
) -> [f64; 4] {
    let pad = 16.0;
    let (x, y, w, h) = (
        target[0] - pad,
        target[1] - pad,
        target[2] + 2.0 * pad,
        target[3] + 2.0 * pad,
    );
    let free = 1.0 - layout.reserve - top_reserve;
    let mut height = (h / free).max(w / aspect);
    let mut width = height * aspect;
    if width < layout.min_w {
        (width, height) = (layout.min_w, layout.min_w / aspect);
    }
    if width > vw {
        (width, height) = (vw, vw / aspect);
    }
    if height > vh {
        (height, width) = (vh, vh * aspect);
    }
    let cx = x + w / 2.0;
    let top = y - top_reserve * height - (height * free - h).max(0.0) / 2.0;
    let left = (cx - width / 2.0).max(0.0).min(vw - width);
    let top = top.max(0.0).min(vh - height);
    [left, top, width, height]
}

/// Smoothstep, clamped to 0..1.
pub fn ease(x: f64) -> f64 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

struct Item {
    label: String,
    card: Option<PathBuf>,
    still: Option<(PathBuf, f64)>,
    overlay: Option<Image>,
    rect: Option<[f64; 4]>,
    from_rect: Option<[f64; 4]>,
}

struct CueAt {
    start: f64,
    end: f64,
}

/// Renders a cut's frames.
pub struct Composer {
    layout: Layout,
    items: Vec<Item>,
    switch: Vec<f64>,
    cues: Vec<CueAt>,
    cue_pieces: Vec<(Image, (i32, i32))>,
    label_pieces: Vec<(String, Image, (i32, i32))>,
    total: f64,
    images: Vec<(PathBuf, Image)>,
}

impl Composer {
    /// Prepares a cut: `shots` has one entry per timeline line, and
    /// `subtitle` is the kicker in the 9:16 title band.
    pub fn new(
        layout: Layout,
        timeline: &Timeline,
        shots: Vec<Shot>,
        fonts: &mut Fonts,
        faces: &Type,
        subtitle: &str,
    ) -> Result<Self, String> {
        if shots.len() != timeline.sentences.len() {
            return Err(format!(
                "{} shots for {} lines",
                shots.len(),
                timeline.sentences.len()
            ));
        }
        let (vw, vh) = (f64::from(layout.view.2), f64::from(layout.view.3));
        let aspect = vw / vh;
        let height = f64::from(layout.size.1);
        let mut items = Vec::with_capacity(shots.len());
        for shot in shots {
            let overlay = shot
                .overlay
                .as_deref()
                .map(Image::load_png)
                .transpose()?
                .map(|o| o.to_rgba());
            let item = match shot.picture {
                Picture::Card(path) => Item {
                    label: shot.label,
                    card: Some(path),
                    still: None,
                    overlay,
                    rect: None,
                    from_rect: None,
                },
                Picture::Still {
                    path,
                    camera,
                    viewport,
                    scale,
                } => {
                    let top = match (&overlay, layout.name) {
                        (Some(o), "16x9") => f64::from(OVERLAY_Y + o.height as i32 + 20) / height,
                        _ => 0.0,
                    };
                    let rect = crop_rect(camera, aspect, &layout, (viewport[0], viewport[1]), top);
                    Item {
                        label: shot.label,
                        card: None,
                        still: Some((path, scale)),
                        overlay,
                        rect: Some(rect),
                        from_rect: None,
                    }
                }
            };
            items.push(item);
        }
        // A page line after a card starts from the framing before the card.
        let mut previous: Option<[f64; 4]> = None;
        for item in items.iter_mut() {
            if let Some(rect) = item.rect {
                item.from_rect = Some(previous.unwrap_or(rect));
                previous = Some(rect);
            }
        }
        // Pictures change halfway through the pause before each line.
        let mut switch = vec![0.0];
        for pair in timeline.sentences.windows(2) {
            switch.push((pair[0].end + pair[1].start) / 2.0);
        }
        let mut cues = Vec::new();
        let mut cue_pieces = Vec::new();
        for entry in &timeline.sentences {
            for cue in &entry.cues {
                cues.push(CueAt {
                    start: cue.start,
                    end: cue.end,
                });
                cue_pieces.push(render_cue(&layout, fonts, faces, &cue.text));
            }
        }
        let mut label_pieces = Vec::new();
        for item in &items {
            if !label_pieces.iter().any(|(l, _, _)| *l == item.label) {
                let (img, pos) = render_label(&layout, fonts, faces, &item.label, subtitle);
                label_pieces.push((item.label.clone(), img, pos));
            }
        }
        Ok(Self {
            layout,
            items,
            switch,
            cues,
            cue_pieces,
            label_pieces,
            total: timeline.total,
            images: Vec::new(),
        })
    }

    /// The frame's width and height.
    pub fn size(&self) -> (u32, u32) {
        self.layout.size
    }

    /// The number of frames: the timeline's length at [`FPS`], rounded up.
    pub fn frames(&self) -> u64 {
        (self.total * FPS).ceil() as u64
    }

    fn index_at(&self, t: f64) -> usize {
        let mut i = 0;
        while i + 1 < self.switch.len() && self.switch[i + 1] <= t {
            i += 1;
        }
        i
    }

    fn cue_at(&self, t: f64) -> Option<usize> {
        let mut best = None;
        for (j, cue) in self.cues.iter().enumerate() {
            if cue.start - 0.12 <= t {
                best = Some(j);
            } else {
                break;
            }
        }
        let best = best?;
        let next = self.cues.get(best + 1).map_or(f64::INFINITY, |c| c.start);
        (t < next.min(self.cues[best].end + 0.9)).then_some(best)
    }

    /// What frame `f` shows, as a key: frames with equal keys are equal.
    pub fn key(&self, f: u64) -> FrameKey {
        let t = f as f64 / FPS;
        let i = self.index_at(t);
        let since = t - self.switch[i];
        let cam = if i > 0 { ease(since / CAM_EASE) } else { 1.0 };
        let fade = if i > 0 { ease(since / XFADE) } else { 1.0 };
        let cue = self.cue_at(t);
        if cam >= 1.0 && fade >= 1.0 {
            FrameKey::Stable(i, cue)
        } else {
            FrameKey::Moving(f)
        }
    }

    /// Frame `f`, as rgb24.
    pub fn frame(&mut self, f: u64) -> Result<Image, String> {
        let t = f as f64 / FPS;
        let i = self.index_at(t);
        let since = t - self.switch[i];
        let cam = if i > 0 { ease(since / CAM_EASE) } else { 1.0 };
        let fade = if i > 0 { ease(since / XFADE) } else { 1.0 };
        let cue = self.cue_at(t);
        let rect = match (self.items[i].from_rect, self.items[i].rect) {
            (Some(a), Some(b)) => Some([0, 1, 2, 3].map(|j| a[j] + (b[j] - a[j]) * cam)),
            _ => None,
        };
        let mut picture = self.visual(i, rect)?;
        if fade < 1.0 && i > 0 {
            let prev_rect = match rect {
                Some(r) if self.items[i - 1].card.is_none() => Some(r),
                _ => self.items[i - 1].rect,
            };
            let old = self.visual(i - 1, prev_rect)?;
            picture = pillow::blend(&old, &picture, fade as f32);
        }
        self.decorate(&mut picture, i, cue);
        Ok(picture)
    }

    fn image(&mut self, path: &PathBuf) -> Result<Image, String> {
        if let Some((_, img)) = self.images.iter().find(|(p, _)| p == path) {
            return Ok(img.clone());
        }
        let img = Image::load_png(path)?.to_rgb();
        // Keep the few most recent pictures, as the reference does.
        if self.images.len() >= 4 {
            self.images.remove(0);
        }
        self.images.push((path.clone(), img.clone()));
        Ok(img)
    }

    /// The item's picture over the whole frame (screen view or card), before captions.
    fn visual(&mut self, i: usize, rect: Option<[f64; 4]>) -> Result<Image, String> {
        if let Some(card) = self.items[i].card.clone() {
            let img = self.image(&card)?;
            if (img.width, img.height) != self.layout.size {
                return Err(format!(
                    "{}: a card of {}x{}, but the {} cut is {}x{}",
                    card.display(),
                    img.width,
                    img.height,
                    self.layout.name,
                    self.layout.size.0,
                    self.layout.size.1
                ));
            }
            return Ok(img);
        }
        let (still, d) = self.items[i]
            .still
            .clone()
            .ok_or("a page line without a capture")?;
        let [x, y, w, h] = rect.ok_or("a page line without a framing")?;
        let source = self.image(&still)?;
        let (vx, vy, vw, vh) = self.layout.view;
        let bbox = [
            (x * d) as f32,
            (y * d) as f32,
            ((x + w) * d) as f32,
            ((y + h) * d) as f32,
        ];
        let view = pillow::resize_bicubic(&source, (vw, vh), bbox);
        if self.layout.name == "16x9" {
            return Ok(view);
        }
        let (fw, fh) = self.layout.size;
        let mut frame = Image::new(fw, fh, &VOID);
        pillow::paste(&mut frame, &view, vx as i32, vy as i32);
        pillow::line_h(&mut frame, 0, fw as i32, vy as i32 - 1, &LINE, 2);
        pillow::line_h(&mut frame, 0, fw as i32, (vy + vh) as i32, &LINE, 2);
        Ok(frame)
    }

    fn decorate(&self, frame: &mut Image, i: usize, cue: Option<usize>) {
        let item = &self.items[i];
        let portrait = self.layout.name == "9x16";
        if item.card.is_none()
            && (!item.label.is_empty() || portrait)
            && let Some((_, img, (x, y))) =
                self.label_pieces.iter().find(|(l, _, _)| *l == item.label)
        {
            pillow::paste_masked(frame, img, *x, *y);
        }
        if let (Some(overlay), None) = (&item.overlay, &item.card) {
            let x = (self.layout.size.0 as i32 - overlay.width as i32).div_euclid(2);
            if portrait {
                pillow::rectangle(frame, (0, 0, self.layout.size.0 as i32, 329), &VOID);
                pillow::paste_masked(frame, overlay, x, 0);
            } else {
                pillow::paste_masked(frame, overlay, x, OVERLAY_Y);
            }
        }
        if let Some(c) = cue {
            let (img, (x, y)) = &self.cue_pieces[c];
            pillow::paste_masked(frame, img, *x, *y);
        }
    }
}

/// Frames with the same key are the same picture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameKey {
    /// Between moves: the line and the caption shown.
    Stable(usize, Option<usize>),
    /// During a camera move or crossfade: a frame of its own.
    Moving(u64),
}

/// Greedy word wrap at `width` pixels, as the reference wraps.
fn wrap(fonts: &mut Fonts, font: &Font, size: f32, text: &str, width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };
        if !current.is_empty() && fonts.length(font, size, &candidate) > width {
            lines.push(std::mem::replace(&mut current, word.to_string()));
        } else {
            current = candidate;
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

/// A caption cue: a rounded box at the bottom of the 16:9 frame, or lines
/// in the band under the 9:16 view.
fn render_cue(layout: &Layout, fonts: &mut Fonts, faces: &Type, text: &str) -> (Image, (i32, i32)) {
    let (w, h) = (layout.size.0 as i32, layout.size.1 as i32);
    if layout.name == "16x9" {
        let size = 42.0;
        let lines = wrap(fonts, &faces.caption, size, text, 1560.0);
        let lh = 56;
        let tw = lines
            .iter()
            .map(|l| fonts.length(&faces.caption, size, l))
            .fold(0.0, f32::max);
        let (bw, bh) = ((tw + 60.0) as i32, lines.len() as i32 * lh + 36);
        let mut img = Image::new(bw as u32, bh as u32, &[0, 0, 0, 0]);
        pillow::rounded_rectangle(
            &mut img,
            (0, 0, bw - 1, bh - 1),
            14,
            &[7, 7, 10, 222],
            &[58, 58, 74, 255],
        );
        for (i, line) in lines.iter().enumerate() {
            let y = 18.0 + (i as i32 * lh) as f32 + lh as f32 / 2.0;
            fonts.draw(
                &mut img,
                (bw as f32 / 2.0, y),
                line,
                Pen {
                    font: &faces.caption,
                    size,
                    ink: INK,
                },
                "mm",
            );
        }
        return (img, ((w - bw).div_euclid(2), h - bh - 44));
    }
    let size = 50.0;
    let lines = wrap(fonts, &faces.caption, size, text, 960.0);
    let lh = 66.0;
    let mut img = Image::new(w as u32, 420, &[0, 0, 0, 0]);
    let y0 = (420.0 - lines.len() as f32 * lh) / 2.0;
    for (i, line) in lines.iter().enumerate() {
        let y = y0 + i as f32 * lh + lh / 2.0;
        fonts.draw(
            &mut img,
            (w as f32 / 2.0, y),
            line,
            Pen {
                font: &faces.caption,
                size,
                ink: INK,
            },
            "mm",
        );
    }
    (img, (0, 1500))
}

/// A section label: a pill at the top left of the 16:9 frame, or the 9:16
/// title band with the kicker and the label as a title.
fn render_label(
    layout: &Layout,
    fonts: &mut Fonts,
    faces: &Type,
    label: &str,
    subtitle: &str,
) -> (Image, (i32, i32)) {
    if layout.name == "16x9" {
        let size = 26.0;
        let w = (fonts.length(&faces.mono, size, &format!("// {label}")) + 40.0) as i32;
        let mut img = Image::new(w as u32, 54, &[0, 0, 0, 0]);
        pillow::rounded_rectangle(
            &mut img,
            (0, 0, w - 1, 53),
            10,
            &[7, 7, 10, 210],
            &[58, 58, 74, 255],
        );
        let mono = |ink| Pen {
            font: &faces.mono,
            size,
            ink,
        };
        fonts.draw(&mut img, (20.0, 27.0), "//", mono(EMBER), "lm");
        let x = 20.0 + fonts.length(&faces.mono, size, "// ");
        fonts.draw(&mut img, (x, 27.0), label, mono(INK), "lm");
        return (img, (32, 30));
    }
    let mut img = Image::new(layout.size.0, 330, &[VOID[0], VOID[1], VOID[2], 255]);
    let size = 30.0;
    fonts.draw(
        &mut img,
        (80.0, 118.0),
        "//",
        Pen {
            font: &faces.mono,
            size,
            ink: EMBER,
        },
        "ls",
    );
    let x = 80.0 + fonts.length(&faces.mono, size, "// ");
    fonts.draw(
        &mut img,
        (x, 118.0),
        subtitle,
        Pen {
            font: &faces.mono,
            size,
            ink: MUTED,
        },
        "ls",
    );
    let mut chars = label.chars();
    let title = chars
        .next()
        .map(|c| c.to_uppercase().collect::<String>() + chars.as_str())
        .unwrap_or_default();
    let lines = wrap(fonts, &faces.display, 74.0, &title, 920.0);
    for (i, line) in lines.iter().take(2).enumerate() {
        let y = 212.0 + i as f32 * 80.0;
        fonts.draw(
            &mut img,
            (80.0, y),
            line,
            Pen {
                font: &faces.display,
                size: 74.0,
                ink: INK,
            },
            "ls",
        );
    }
    (img, (0, 0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crops_match_the_reference() {
        // The reference's own numbers (reference/tests/test_behaviour.py, known framings).
        let r = crop_rect(
            [646.0, 123.0, 622.0, 303.0],
            16.0 / 9.0,
            &LANDSCAPE,
            (1280.0, 720.0),
            0.0,
        );
        assert_eq!(
            r.map(|v| (v * 1000.0).round() / 1000.0),
            [535.556, 107.0, 744.444, 418.75]
        );
        let r = crop_rect(
            [676.0, 261.0, 554.0, 50.0],
            1080.0 / 1170.0,
            &PORTRAIT,
            (1280.0, 720.0),
            0.0,
        );
        assert_eq!(
            r.map(|v| (v * 1000.0).round() / 1000.0),
            [660.0, 0.0, 586.0, 634.833]
        );
    }

    #[test]
    fn ease_is_smoothstep() {
        assert_eq!((ease(-1.0), ease(0.5), ease(2.0)), (0.0, 0.5, 1.0));
    }

    use std::path::Path;
    use vidni_core::timeline::{Cue, Entry};

    const RED: [u8; 3] = [200, 40, 10];
    const BLUE: [u8; 3] = [10, 60, 220];

    fn line(k: usize, start: f64, end: f64) -> Entry {
        Entry {
            k,
            segment: "s".into(),
            label: String::new(),
            index: k,
            start,
            end,
            wav: String::new(),
            show: String::new(),
            say: String::new(),
            card: None,
            cues: vec![Cue {
                start,
                end,
                text: "A caption".into(),
            }],
        }
    }

    /// Two card lines, red from 0.4 to 2 s and blue from 4 to 5 s, so the
    /// picture switches at 3 s. Cards are `size`.
    fn two_cards(dir: &Path, size: (u32, u32)) -> Composer {
        let mut fonts = Fonts::new();
        let plex =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fonts/IBMPlexSans-Bold.otf");
        let font = fonts.load(&plex).unwrap();
        let faces = Type {
            caption: font.clone(),
            mono: font.clone(),
            display: font,
        };
        let card = |name: &str, colour: [u8; 3]| {
            let path = dir.join(name);
            Image::new(size.0, size.1, &colour).save_png(&path).unwrap();
            Shot {
                label: String::new(),
                picture: Picture::Card(path),
                overlay: None,
            }
        };
        let timeline = Timeline {
            total: 6.0,
            rate: 22050,
            sentences: vec![line(0, 0.4, 2.0), line(1, 4.0, 5.0)],
        };
        let shots = vec![card("red.png", RED), card("blue.png", BLUE)];
        Composer::new(LANDSCAPE, &timeline, shots, &mut fonts, &faces, "").unwrap()
    }

    #[test]
    fn captions_show_early_and_linger_until_the_next_one() {
        let dir = tempfile::tempdir().unwrap();
        let cut = two_cards(dir.path(), (1920, 1080));
        // 0.12 s before the cue, and 0.9 s after it unless the next one comes first.
        let at = |t: f64| cut.cue_at(t);
        assert_eq!((at(0.275), at(0.285)), (None, Some(0)));
        assert_eq!((at(2.895), at(2.905)), (Some(0), None));
        assert_eq!((at(3.875), at(3.885)), (None, Some(1)));
    }

    #[test]
    fn the_picture_moves_for_the_camera_ease_after_each_switch() {
        let dir = tempfile::tempdir().unwrap();
        let cut = two_cards(dir.path(), (1920, 1080));
        // The switch is halfway through the pause: 3 s, frame 90. The camera
        // eases for 0.7 s, 21 frames, and the crossfade inside it.
        assert_eq!(cut.key(89), FrameKey::Stable(0, None));
        assert_eq!(cut.key(90), FrameKey::Moving(90));
        assert_eq!(cut.key(110), FrameKey::Moving(110));
        assert_eq!(cut.key(111), FrameKey::Stable(1, None));
    }

    #[test]
    fn crossfades_blend_from_the_old_picture_with_an_eased_alpha() {
        let dir = tempfile::tempdir().unwrap();
        let mut cut = two_cards(dir.path(), (1920, 1080));
        let colour = |cut: &mut Composer, f: u64| cut.frame(f).unwrap().pixel(960, 300).to_vec();
        assert_eq!(colour(&mut cut, 89), RED);
        for f in [90, 93, 95, 100] {
            let alpha = ease((f as f64 / FPS - 3.0) / XFADE) as f32;
            let want = pillow::blend(&Image::new(1, 1, &RED), &Image::new(1, 1, &BLUE), alpha);
            assert_eq!(colour(&mut cut, f), want.data, "frame {f}");
        }
        assert_eq!(colour(&mut cut, 101), BLUE);
    }

    #[test]
    fn a_card_of_the_wrong_size_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let mut cut = two_cards(dir.path(), (1280, 720));
        let error = cut.frame(0).unwrap_err();
        assert!(error.contains("a card of 1280x720"), "{error}");
    }
}
