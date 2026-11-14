//! A cut's frames: the reference compositor (`reference/compose.py`, class
//! `Cut`), ported. A virtual camera eases between crops of each line's
//! capture, pictures crossfade between lines, and section labels, overlays
//! and captions are drawn on top.
//!
//! The geometry is the reference's. The colours, the faces and the mark
//! before a label are the theme's ([`Look`]); with the QUINE theme of the
//! nibli example the frames are the reference's own.

use super::image::Image;
use super::pillow;
use super::text::{Font, Fonts, Pen};
use std::path::PathBuf;
use vidni_core::platforms::Insets;
use vidni_core::theme::{self, Theme};
use vidni_core::timeline::Timeline;

/// Frames per second.
pub const FPS: f64 = 30.0;
/// Seconds a camera move takes.
const CAM_EASE: f64 = 0.7;
/// Seconds a crossfade takes.
const XFADE: f64 = 0.35;
/// Where 16:9 overlays sit, below the section label.
const OVERLAY_Y: i32 = 100;

/// A theme's colours and marks, as the compositor draws with them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Look {
    pub frame: [u8; 3],
    pub rule: [u8; 3],
    pub ink: [u8; 3],
    pub muted: [u8; 3],
    pub accent: [u8; 3],
    /// The box behind captions, the box behind a label, and their edge.
    pub panel: [u8; 4],
    pub label: [u8; 4],
    pub edge: [u8; 4],
    /// Written before a label, in the accent colour.
    pub mark: String,
}

impl Look {
    /// The look of `theme`, whose tokens were checked when it was read.
    pub fn of(theme: &Theme) -> Self {
        let rgba = |color: &str| theme::rgba(color).unwrap_or([0, 0, 0, 255]);
        let rgb = |color: &str| {
            let [r, g, b, _] = rgba(color);
            [r, g, b]
        };
        let colors = &theme.colors;
        Self {
            frame: rgb(&colors.frame),
            rule: rgb(&colors.rule),
            ink: rgb(&colors.ink),
            muted: rgb(&colors.muted),
            accent: rgb(&colors.accent),
            panel: rgba(&colors.panel),
            label: rgba(&colors.label),
            edge: rgba(&colors.edge),
            mark: theme.label.mark.clone(),
        }
    }
}

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
    /// The edges of the frame that the cut's apps cover, when they cover any.
    pub covered: Option<Insets>,
}

/// The 16:9 cut: the view fills the frame.
pub const LANDSCAPE: Layout = Layout {
    name: "16x9",
    size: (1920, 1080),
    view: (0, 0, 1920, 1080),
    min_w: 640.0,
    reserve: 0.2,
    covered: None,
};
/// The 9:16 cut: a title band, the view, and captions below.
pub const PORTRAIT: Layout = Layout {
    name: "9x16",
    size: (1080, 1920),
    view: (0, 330, 1080, 1170),
    min_w: 400.0,
    reserve: 0.0,
    covered: None,
};

/// In a 9:16 cut laid out clear of its apps: the height of the title band
/// under the top edge they cover, and of the caption band above the bottom one.
const BAND: u32 = 176;
const CAPTIONS: u32 = 190;
/// The least such a cut's view may be, in pixels each way.
const LEAST_VIEW: u32 = 240;

impl Layout {
    /// The 9:16 cut for apps that cover `covered`: the title, the view and
    /// the captions all sit inside the part of the frame they leave free,
    /// and the rest of the frame is the frame's colour.
    pub fn clear_of(covered: Insets) -> Result<Self, String> {
        let (w, h) = PORTRAIT.size;
        let (top, bottom) = (covered.top + BAND, covered.bottom + CAPTIONS);
        if top + bottom + LEAST_VIEW > h || covered.left + covered.right + LEAST_VIEW > w {
            return Err(format!(
                "its platforms cover too much of the frame to draw between them: \
                 {} px at the top, {} at the bottom, {} at the left and {} at the right",
                covered.top, covered.bottom, covered.left, covered.right
            ));
        }
        Ok(Self {
            view: (
                covered.left,
                top,
                w - covered.left - covered.right,
                h - top - bottom,
            ),
            covered: Some(covered),
            ..PORTRAIT
        })
    }

    /// The edges to lay a 9:16 cut out clear of.
    fn clear(&self) -> Option<Insets> {
        self.covered.filter(|_| self.name == "9x16")
    }

    /// The height of the band a cut laid out clear of its apps keeps for
    /// captions, at the bottom of the free part of the frame.
    pub(super) fn caption_band(&self) -> u32 {
        self.clear().map_or(0, |_| CAPTIONS)
    }

    /// The part of the frame no app covers, as its corners: left, top,
    /// right and bottom.
    pub fn safe(&self) -> Option<[i32; 4]> {
        let (w, h) = (self.size.0 as i32, self.size.1 as i32);
        self.covered.map(|c| {
            [
                c.left as i32,
                c.top as i32,
                w - c.right as i32,
                h - c.bottom as i32,
            ]
        })
    }
}

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
    /// A card drawn from one of the theme's templates, at the cut's size.
    Drawn(super::card::Card),
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
    /// A card drawn for this cut.
    drawn: Option<Image>,
    /// The size of the page or picture a still is of, in CSS pixels: what
    /// its framing is measured in.
    space: Option<[u64; 2]>,
    still: Option<(PathBuf, f64)>,
    overlay: Option<Image>,
    rect: Option<[f64; 4]>,
    from_rect: Option<[f64; 4]>,
}

impl Item {
    /// Whether the item fills the frame: a card, rendered or drawn.
    fn full(&self) -> bool {
        self.card.is_some() || self.drawn.is_some()
    }
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
    /// The line each cue belongs to.
    cue_line: Vec<usize>,
    label_pieces: Vec<(String, Image, (i32, i32))>,
    total: f64,
    images: Vec<(PathBuf, Image)>,
    /// Whether captions are drawn into the picture.
    captions: bool,
    look: Look,
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
        look: &Look,
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
                Picture::Drawn(card) => Item {
                    label: shot.label,
                    card: None,
                    drawn: Some(super::card::render(&card, &layout, fonts, faces, look)),
                    space: None,
                    still: None,
                    overlay,
                    rect: None,
                    from_rect: None,
                },
                Picture::Card(path) => Item {
                    label: shot.label,
                    card: Some(path),
                    drawn: None,
                    space: None,
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
                        drawn: None,
                        space: Some(viewport.map(f64::to_bits)),
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
        // A framing is measured on its own page or picture, so a line on a
        // picture of another size starts from its own framing.
        let mut previous: Option<([u64; 2], [f64; 4])> = None;
        for item in items.iter_mut() {
            if let (Some(rect), Some(space)) = (item.rect, item.space) {
                let from = previous
                    .filter(|(before, _)| *before == space)
                    .map_or(rect, |(_, rect)| rect);
                item.from_rect = Some(from);
                previous = Some((space, rect));
            }
        }
        // Pictures change halfway through the pause before each line.
        let mut switch = vec![0.0];
        for pair in timeline.sentences.windows(2) {
            switch.push((pair[0].end + pair[1].start) / 2.0);
        }
        let mut cues = Vec::new();
        let mut cue_pieces = Vec::new();
        let mut cue_line = Vec::new();
        for (i, entry) in timeline.sentences.iter().enumerate() {
            for cue in &entry.cues {
                cue_line.push(i);
                cues.push(CueAt {
                    start: cue.start,
                    end: cue.end,
                });
                cue_pieces.push(render_cue(&layout, fonts, faces, look, &cue.text));
            }
        }
        let mut label_pieces = Vec::new();
        for item in &items {
            if !label_pieces.iter().any(|(l, _, _)| *l == item.label) {
                let (img, pos) = render_label(&layout, fonts, faces, look, &item.label, subtitle);
                label_pieces.push((item.label.clone(), img, pos));
            }
        }
        Ok(Self {
            layout,
            items,
            switch,
            cues,
            cue_pieces,
            cue_line,
            label_pieces,
            total: timeline.total,
            images: Vec::new(),
            captions: true,
            look: look.clone(),
        })
    }

    /// Leaves the captions out of the picture, for a cut whose captions
    /// are files only. The framing stays the same.
    pub fn without_captions(mut self) -> Self {
        self.captions = false;
        self
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
        if !self.captions {
            return None;
        }
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
                Some(r) if !self.items[i - 1].full() => Some(r),
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
        if let Some(drawn) = &self.items[i].drawn {
            return Ok(drawn.clone());
        }
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
        let mut frame = Image::new(fw, fh, &self.look.frame);
        pillow::paste(&mut frame, &view, vx as i32, vy as i32);
        // A rule above the view and one below it, as wide as it is.
        let (x0, x1) = (vx as i32, (vx + vw) as i32 - 1);
        pillow::line_h(&mut frame, x0, x1, vy as i32 - 1, &self.look.rule, 2);
        pillow::line_h(&mut frame, x0, x1, (vy + vh) as i32, &self.look.rule, 2);
        Ok(frame)
    }

    fn decorate(&self, frame: &mut Image, i: usize, cue: Option<usize>) {
        let item = &self.items[i];
        let portrait = self.layout.name == "9x16";
        if !item.full()
            && (!item.label.is_empty() || portrait)
            && let Some((_, img, (x, y))) =
                self.label_pieces.iter().find(|(l, _, _)| *l == item.label)
        {
            pillow::paste_masked(frame, img, *x, *y);
        }
        if let (Some(overlay), false) = (&item.overlay, item.full()) {
            let (x, y) = self.overlay_at(overlay);
            if portrait {
                // The overlay takes the title band's place.
                let band = self.layout.view.1 as i32 - 1;
                pillow::rectangle(
                    frame,
                    (0, 0, self.layout.size.0 as i32, band),
                    &self.look.frame,
                );
            }
            pillow::paste_masked(frame, overlay, x, y);
        }
        if let Some(c) = cue {
            let (img, (x, y)) = &self.cue_pieces[c];
            pillow::paste_masked(frame, img, *x, *y);
        }
    }
}

impl Composer {
    /// Where an overlay goes: under the section label of a 16:9 cut, and
    /// in place of the title band of a 9:16 one.
    fn overlay_at(&self, overlay: &Image) -> (i32, i32) {
        let (width, own) = (self.layout.size.0 as i32, overlay.width as i32);
        match (self.layout.name, self.layout.clear()) {
            ("16x9", _) => ((width - own).div_euclid(2), OVERLAY_Y),
            (_, None) => ((width - own).div_euclid(2), 0),
            (_, Some(covered)) => {
                let free = width - (covered.left + covered.right) as i32;
                (
                    covered.left as i32 + (free - own).div_euclid(2),
                    covered.top as i32,
                )
            }
        }
    }

    /// What the cut draws where an app would cover it: for each line, its
    /// card, its section label, its overlay and its captions, wherever one
    /// reaches past the part of the frame the apps leave free. Empty for a
    /// cut whose apps cover nothing. A line's view is never listed: a cut
    /// laid out clear of its apps keeps the view inside that part.
    pub fn spills(&mut self) -> Result<Vec<Spill>, String> {
        let Some(safe) = self.layout.safe() else {
            return Ok(Vec::new());
        };
        let portrait = self.layout.name == "9x16";
        let frame = self.look.frame;
        let mut spills = Vec::new();
        for i in 0..self.items.len() {
            // What the line draws, and where each thing's ink is.
            let mut drawn: Vec<(&'static str, [i32; 4])> = Vec::new();
            if self.items[i].full() {
                // A card's ink is whatever differs from its own background.
                let card = self.visual(i, None)?;
                let background = card.pixel(0, 0).to_vec();
                drawn.extend(card.ink(&background).map(|ink| ("card", ink)));
            } else {
                let item = &self.items[i];
                if (!item.label.is_empty() || portrait)
                    && let Some((_, img, at)) =
                        self.label_pieces.iter().find(|(l, _, _)| *l == item.label)
                {
                    drawn.extend(
                        img.ink(&frame)
                            .map(|ink| ("section label", moved(ink, *at))),
                    );
                }
                if let Some(overlay) = &item.overlay {
                    let at = self.overlay_at(overlay);
                    drawn.extend(overlay.ink(&frame).map(|ink| ("overlay", moved(ink, at))));
                }
            }
            if self.captions {
                for (c, (img, at)) in self.cue_pieces.iter().enumerate() {
                    if self.cue_line[c] == i {
                        drawn.extend(img.ink(&frame).map(|ink| ("caption", moved(ink, *at))));
                    }
                }
            }
            for (what, [x0, y0, x1, y1]) in drawn {
                let past = [
                    ("top", safe[1] - y0),
                    ("bottom", y1 - safe[3]),
                    ("left", safe[0] - x0),
                    ("right", x1 - safe[2]),
                ];
                for (edge, by) in past {
                    if by > 0
                        && !spills
                            .iter()
                            .any(|s: &Spill| (s.line, s.what, s.edge) == (i, what, edge))
                    {
                        spills.push(Spill {
                            line: i,
                            what,
                            edge,
                            by: by as u32,
                        });
                    }
                }
            }
        }
        Ok(spills)
    }
}

/// A box of ink, moved to where its image is pasted.
fn moved([x0, y0, x1, y1]: [i32; 4], (x, y): (i32, i32)) -> [i32; 4] {
    [x0 + x, y0 + y, x1 + x, y1 + y]
}

/// Something a line draws that an app would cover.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spill {
    /// The line, by its place in the cut.
    pub line: usize,
    /// `card`, `section label`, `overlay` or `caption`.
    pub what: &'static str,
    /// The edge the app covers: `top`, `bottom`, `left` or `right`.
    pub edge: &'static str,
    /// How far past the free part of the frame it reaches, in pixels.
    pub by: u32,
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
pub(super) fn wrap(
    fonts: &mut Fonts,
    font: &Font,
    size: f32,
    text: &str,
    width: f32,
) -> Vec<String> {
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
fn render_cue(
    layout: &Layout,
    fonts: &mut Fonts,
    faces: &Type,
    look: &Look,
    text: &str,
) -> (Image, (i32, i32)) {
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
            &look.panel,
            &look.edge,
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
                    ink: look.ink,
                },
                "mm",
            );
        }
        return (img, ((w - bw).div_euclid(2), h - bh - 44));
    }
    // Clear of the apps: smaller lines, in the band above the bottom edge
    // they cover. A cue of more lines than the band holds grows upwards.
    if let Some(covered) = layout.clear() {
        let (size, lh) = (44.0, 56.0);
        let width = w - (covered.left + covered.right) as i32;
        let lines = wrap(fonts, &faces.caption, size, text, width as f32 - 40.0);
        let height = (CAPTIONS as i32).max(lines.len() as i32 * lh as i32 + 16);
        let mut img = Image::new(width as u32, height as u32, &[0, 0, 0, 0]);
        let y0 = (height as f32 - lines.len() as f32 * lh) / 2.0;
        for (i, line) in lines.iter().enumerate() {
            let y = y0 + i as f32 * lh + lh / 2.0;
            fonts.draw(
                &mut img,
                (width as f32 / 2.0, y),
                line,
                Pen {
                    font: &faces.caption,
                    size,
                    ink: look.ink,
                },
                "mm",
            );
        }
        let at = (covered.left as i32, h - covered.bottom as i32 - height);
        return (img, at);
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
                ink: look.ink,
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
    look: &Look,
    label: &str,
    subtitle: &str,
) -> (Image, (i32, i32)) {
    // The mark and the space after it, when the theme has one.
    let mark = match look.mark.is_empty() {
        true => String::new(),
        false => format!("{} ", look.mark),
    };
    if layout.name == "16x9" {
        let size = 26.0;
        let w = (fonts.length(&faces.mono, size, &format!("{mark}{label}")) + 40.0) as i32;
        let mut img = Image::new(w as u32, 54, &[0, 0, 0, 0]);
        pillow::rounded_rectangle(&mut img, (0, 0, w - 1, 53), 10, &look.label, &look.edge);
        let mono = |ink| Pen {
            font: &faces.mono,
            size,
            ink,
        };
        if !mark.is_empty() {
            fonts.draw(&mut img, (20.0, 27.0), &look.mark, mono(look.accent), "lm");
        }
        let x = 20.0 + fonts.length(&faces.mono, size, &mark);
        fonts.draw(&mut img, (x, 27.0), label, mono(look.ink), "lm");
        return (img, (32, 30));
    }
    // The band, where it sits, and how its kicker and its title are set:
    // the size of each, the left edge, the kicker's baseline, the first
    // title line's, and the step to the second.
    let [r, g, b] = look.frame;
    let (img, at, kicker, big, x0, y0, y1, step) = match layout.clear() {
        None => (
            Image::new(layout.size.0, 330, &[r, g, b, 255]),
            (0, 0),
            30.0,
            74.0,
            80.0,
            118.0,
            212.0,
            80.0,
        ),
        // Clear of the apps: a lower band under the top edge they cover.
        Some(covered) => (
            Image::new(
                layout.size.0 - covered.left - covered.right,
                BAND,
                &[r, g, b, 255],
            ),
            (covered.left as i32, covered.top as i32),
            24.0,
            50.0,
            24.0,
            38.0,
            98.0,
            58.0,
        ),
    };
    let mut img = img;
    if !mark.is_empty() {
        fonts.draw(
            &mut img,
            (x0, y0),
            &look.mark,
            Pen {
                font: &faces.mono,
                size: kicker,
                ink: look.accent,
            },
            "ls",
        );
    }
    let x = x0 + fonts.length(&faces.mono, kicker, &mark);
    fonts.draw(
        &mut img,
        (x, y0),
        subtitle,
        Pen {
            font: &faces.mono,
            size: kicker,
            ink: look.muted,
        },
        "ls",
    );
    let mut chars = label.chars();
    let title = chars
        .next()
        .map(|c| c.to_uppercase().collect::<String>() + chars.as_str())
        .unwrap_or_default();
    let width = img.width as f32 - 2.0 * x0;
    let lines = wrap(fonts, &faces.display, big, &title, width);
    for (i, line) in lines.iter().take(2).enumerate() {
        let y = y1 + i as f32 * step;
        fonts.draw(
            &mut img,
            (x0, y),
            line,
            Pen {
                font: &faces.display,
                size: big,
                ink: look.ink,
            },
            "ls",
        );
    }
    (img, at)
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
        two_cards_in(dir, size, LANDSCAPE)
    }

    fn two_cards_in(dir: &Path, size: (u32, u32), layout: Layout) -> Composer {
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
        let look = Look::of(&Theme::default_theme());
        Composer::new(layout, &timeline, shots, &mut fonts, &faces, &look, "").unwrap()
    }

    /// What Shorts and Reels cover together.
    const APPS: Insets = Insets {
        top: 288,
        bottom: 672,
        left: 65,
        right: 193,
    };

    #[test]
    fn a_cut_clear_of_its_apps_keeps_its_view_between_the_bands() {
        let layout = Layout::clear_of(APPS).unwrap();
        // Under the top edge and the title band, above the caption band
        // and the bottom edge, and between the sides.
        assert_eq!(layout.view, (65, 288 + 176, 822, 1920 - 464 - 190 - 672));
        assert_eq!(layout.safe(), Some([65, 288, 887, 1248]));
        assert_eq!((layout.name, layout.size), ("9x16", (1080, 1920)));
        // The cut that no app covers is the reference's.
        assert_eq!(PORTRAIT.safe(), None);
        assert_eq!(PORTRAIT.view, (0, 330, 1080, 1170));
        // Apps that leave no room for a view are refused.
        let tall = Insets {
            top: 600,
            bottom: 800,
            ..APPS
        };
        let error = Layout::clear_of(tall).unwrap_err();
        assert!(
            error.contains("600 px at the top, 800 at the bottom"),
            "{error}"
        );
    }

    #[test]
    fn a_cut_clear_of_its_apps_draws_its_captions_inside_the_free_part() {
        let dir = tempfile::tempdir().unwrap();
        let layout = Layout::clear_of(APPS).unwrap();
        let mut cut = two_cards_in(dir.path(), (1080, 1920), layout);
        assert_eq!(cut.spills().unwrap(), []);
        // Frame 30 is inside the first cue: its ink is the only thing
        // that isn't the card, and all of it is in the caption band.
        let frame = cut.frame(30).unwrap();
        let ink = frame.ink(&RED).unwrap();
        assert!(ink[0] >= 65 && ink[2] <= 887, "{ink:?}");
        assert!(ink[1] >= 1248 - 190 && ink[3] <= 1248, "{ink:?}");
        // The cut that no app covers draws the same cue lower, as the reference does.
        let mut old = two_cards_in(dir.path(), (1080, 1920), PORTRAIT);
        assert_eq!(old.spills().unwrap(), []);
        let ink = old.frame(30).unwrap().ink(&RED).unwrap();
        assert!(ink[1] >= 1500, "{ink:?}");
    }

    #[test]
    fn what_an_app_would_cover_is_listed_by_line_and_edge() {
        let dir = tempfile::tempdir().unwrap();
        // A 16:9 cut keeps its layout, so its caption box, which ends 44
        // pixels above the bottom, is under an app that covers 100.
        let covered = Insets {
            top: 0,
            bottom: 100,
            left: 0,
            right: 0,
        };
        let layout = Layout {
            covered: Some(covered),
            ..LANDSCAPE
        };
        let mut cut = two_cards_in(dir.path(), (1920, 1080), layout);
        let spill = |line| Spill {
            line,
            what: "caption",
            edge: "bottom",
            by: 56,
        };
        assert_eq!(cut.spills().unwrap(), [spill(0), spill(1)]);
        // Without captions in the picture, nothing is covered.
        let mut clean = two_cards_in(dir.path(), (1920, 1080), layout).without_captions();
        assert_eq!(clean.spills().unwrap(), []);
        // A card's ink is what differs from its own background: here a
        // white square near the right edge of the second card.
        let mut card = Image::new(1920, 1080, &BLUE);
        pillow::rectangle(&mut card, (1800, 500, 1899, 599), &[255, 255, 255]);
        card.save_png(&dir.path().join("blue.png")).unwrap();
        let sides = Insets {
            top: 0,
            bottom: 0,
            left: 0,
            right: 60,
        };
        let layout = Layout {
            covered: Some(sides),
            ..LANDSCAPE
        };
        let mut cut = two_cards_in(dir.path(), (1920, 1080), layout);
        // two_cards_in writes the cards again, so the square is drawn after it.
        card.save_png(&dir.path().join("blue.png")).unwrap();
        assert_eq!(
            cut.spills().unwrap(),
            [Spill {
                line: 1,
                what: "card",
                edge: "right",
                by: 40,
            }]
        );
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
    fn a_cut_without_captions_draws_none_and_changes_nothing_else() {
        let dir = tempfile::tempdir().unwrap();
        let mut burned = two_cards(dir.path(), (1920, 1080));
        let mut clean = two_cards(dir.path(), (1920, 1080)).without_captions();
        assert_eq!(clean.frames(), burned.frames());
        // Frame 30 is inside the first cue: the caption's box is the only difference.
        assert_eq!(burned.key(30), FrameKey::Stable(0, Some(0)));
        assert_eq!(clean.key(30), FrameKey::Stable(0, None));
        let (with, without) = (burned.frame(30).unwrap(), clean.frame(30).unwrap());
        assert_ne!(with.data, without.data);
        assert_eq!(without.data, Image::new(1920, 1080, &RED).data);
        // Before the cue, the two are the same picture.
        assert_eq!(burned.frame(0).unwrap().data, clean.frame(0).unwrap().data);
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
