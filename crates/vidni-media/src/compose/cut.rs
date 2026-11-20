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
/// The room a 16:9 cut's section label takes at the top of the frame, in pixels.
const LABEL: f64 = 84.0;

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
    /// An overlay scene to draw over the picture, in the place of a file.
    pub strip: Option<vidni_core::project::Scene>,
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
        /// Whether the framing may reach past the picture's edges, with
        /// the picture's ground around it: so for a drawn picture, which
        /// has no page of the cut's shape around it as a web page has.
        loose: bool,
        /// What is around a picture that the framing reaches past.
        ground: Ground,
    },
    Card(PathBuf),
    /// A card drawn from one of the theme's templates, at the cut's size.
    Drawn(super::card::Card),
}

/// What is around a picture where its framing reaches past it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ground {
    /// The colour of the picture's first pixel, all around.
    Corner,
    /// Each row goes on to the frame's edges in the colour it ends in, and
    /// the first and the last row go on above and below: so a window of
    /// text is a window as wide as the frame, its bar across the top.
    Rows,
}

/// A crop (CSS pixels) of the cut's aspect that frames `target` between
/// the top reserve (an overlay) and the bottom reserve (captions).
///
/// A crop stays on its page, and so shows less than `target` when the page
/// is too small for it. A `loose` crop keeps all of `target` and reaches
/// past the page instead, as far as it must.
pub fn crop_rect(
    target: [f64; 4],
    aspect: f64,
    layout: &Layout,
    (vw, vh): (f64, f64),
    top_reserve: f64,
    loose: bool,
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
    if !loose {
        if width > vw {
            (width, height) = (vw, vw / aspect);
        }
        if height > vh {
            (height, width) = (vh, vh * aspect);
        }
    }
    let mut top = y - top_reserve * height - (height * free - h).max(0.0) / 2.0;
    // A loose crop that starts in the picture's first rows would leave
    // them, a window's bar among them, under the label or the overlay. It
    // starts at the picture's top instead, below what covers it, and is
    // as large as it must be to hold the target still.
    if loose && top_reserve > 0.0 && top < top_reserve * height {
        height = height.max((y + h) / free);
        width = height * aspect;
        top = -top_reserve * height;
    }
    let cx = x + w / 2.0;
    // On the page where the crop is smaller than it, and around the page
    // where it is larger. A loose crop may also show `over` past an edge.
    let within = |at: f64, size: f64, page: f64, over: [f64; 2]| {
        let (low, high) = (-over[0], page - size + over[1]);
        at.max(low.min(high)).min(low.max(high))
    };
    // What an overlay or a label covers at the top of a loose crop, and
    // the captions at its bottom, needn't be the picture: so its first
    // rows can sit under the one and its last above the other.
    let over = match loose {
        true => [top_reserve * height, layout.reserve * height],
        false => [0.0, 0.0],
    };
    [
        within(cx - width / 2.0, width, vw, [0.0, 0.0]),
        within(top, height, vh, over),
        width,
        height,
    ]
}

/// `source` with room around it for `bbox` (left, top, right, bottom, in
/// its pixels), where `bbox` reaches past its edges, and `bbox` as the
/// larger picture has it. The room is the picture's `ground`.
fn around(source: Image, bbox: [f32; 4], ground: Ground) -> (Image, [f32; 4]) {
    let past = |v: f32| v.max(0.0).ceil() as u32;
    let (left, top) = (past(-bbox[0]), past(-bbox[1]));
    let right = past(bbox[2] - source.width as f32);
    let bottom = past(bbox[3] - source.height as f32);
    if (left, top, right, bottom) == (0, 0, 0, 0) {
        return (source, bbox);
    }
    let mut room = Image::new(
        source.width + left + right,
        source.height + top + bottom,
        source.pixel(0, 0),
    );
    if ground == Ground::Rows {
        let across = room.width as usize * 3;
        for (y, row) in room.data.chunks_exact_mut(across).enumerate() {
            let from = (y as u32).saturating_sub(top).min(source.height - 1);
            let ink = source.pixel(source.width - 1, from);
            for pixel in row.as_chunks_mut::<3>().0 {
                pixel.copy_from_slice(ink);
            }
        }
    }
    pillow::paste(&mut room, &source, left as i32, top as i32);
    let (dx, dy) = (left as f32, top as f32);
    (
        room,
        [bbox[0] + dx, bbox[1] + dy, bbox[2] + dx, bbox[3] + dy],
    )
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
    still: Option<(PathBuf, f64, Ground)>,
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

/// A reviewed emphasis interval, resolved to half-open output frames.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EmphasisEvent {
    pub id: String,
    pub line: usize,
    pub start: u64,
    pub end: u64,
    /// Unpadded measured rectangles in the capture's own coordinate system.
    pub boxes: Vec<[f64; 4]>,
}

#[derive(Debug)]
pub(crate) enum EmphasisError {
    OutsideCamera(String),
    Invalid(String),
}

impl From<String> for EmphasisError {
    fn from(value: String) -> Self {
        Self::Invalid(value)
    }
}

impl std::fmt::Display for EmphasisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutsideCamera(s) | Self::Invalid(s) => f.write_str(s),
        }
    }
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
    emphasis: Vec<EmphasisEvent>,
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
        // The face an overlay's signs are set in, loaded when one is drawn.
        let mut signs: Option<Font> = None;
        for shot in shots {
            let overlay = shot
                .overlay
                .as_deref()
                .map(Image::load_png)
                .transpose()?
                .map(|o| o.to_rgba());
            // An overlay scene is drawn here, in the theme's faces.
            let overlay = match (&shot.strip, overlay) {
                (Some(strip), _) => {
                    if signs.is_none() {
                        signs = Some(sign_face(fonts)?);
                    }
                    let face = signs.as_ref().expect("it was just loaded");
                    let portrait = layout.size.1 > layout.size.0;
                    // As wide as its shape has it, or as the part of the
                    // frame that the cut's apps leave free.
                    let (own, _) = super::overlay::size(strip, portrait);
                    let width = layout.clear().map_or(own, |covered| {
                        own.min(layout.size.0 - covered.left - covered.right)
                    });
                    Some(super::overlay::render_across(
                        strip, portrait, width, fonts, faces, face, look,
                    ))
                }
                (None, overlay) => overlay,
            };
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
                    loose,
                    ground,
                } => {
                    let top = match (&overlay, layout.name) {
                        (Some(o), "16x9") => f64::from(OVERLAY_Y + o.height as i32 + 20) / height,
                        // A 9:16 overlay stands in the title band's place,
                        // and one taller than the band reaches over the view.
                        (Some(o), _) => {
                            let from = layout.clear().map_or(0, |covered| covered.top);
                            f64::from((from + o.height).saturating_sub(layout.view.1)) / vh
                        }
                        // A drawn picture is shown whole, so its top would
                        // sit under the section's label.
                        (None, "16x9") if loose && !shot.label.is_empty() => LABEL / height,
                        _ => 0.0,
                    };
                    let rect = crop_rect(
                        camera,
                        aspect,
                        &layout,
                        (viewport[0], viewport[1]),
                        top,
                        loose,
                    );
                    Item {
                        label: shot.label,
                        card: None,
                        drawn: None,
                        space: Some(viewport.map(f64::to_bits)),
                        still: Some((path, scale, ground)),
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
            emphasis: Vec::new(),
        })
    }

    /// Attach only resolved events. Check every affected frame, including
    /// intermediate camera positions; checking the final camera alone is unsafe.
    pub fn with_emphasis(mut self, events: Vec<EmphasisEvent>) -> Result<Self, String> {
        for event in &events {
            if event.line >= self.items.len()
                || event.start >= event.end
                || event.end > self.frames()
                || event.boxes.is_empty()
            {
                return Err(format!(
                    "{}: emphasis has no valid visible interval",
                    event.id
                ));
            }
            for f in event.start..event.end {
                self.emphasis_boxes(event, f)?;
            }
        }
        self.emphasis = events;
        Ok(self)
    }

    fn rect_at(&self, i: usize, t: f64) -> Option<[f64; 4]> {
        let cam = if i > 0 {
            ease((t - self.switch[i]) / CAM_EASE)
        } else {
            1.0
        };
        match (self.items[i].from_rect, self.items[i].rect) {
            (Some(a), Some(b)) => Some([0, 1, 2, 3].map(|j| a[j] + (b[j] - a[j]) * cam)),
            _ => None,
        }
    }

    /// Exact same crop and view transform as `visual`, applied to geometry.
    pub fn emphasis_boxes(&self, event: &EmphasisEvent, f: u64) -> Result<Vec<[f64; 4]>, String> {
        self.emphasis_geometry(event, f).map_err(|e| e.to_string())
    }

    pub(crate) fn emphasis_geometry(
        &self,
        event: &EmphasisEvent,
        f: u64,
    ) -> Result<Vec<[f64; 4]>, EmphasisError> {
        let t = f as f64 / FPS;
        if self.index_at(t) != event.line {
            return Err(format!("{}: its scene is not active at frame {f}", event.id).into());
        }
        let [cx, cy, cw, ch] = self
            .rect_at(event.line, t)
            .ok_or_else(|| format!("{}: emphasis needs a measured still", event.id))?;
        let (vx, vy, vw, vh) = self.layout.view;
        let sx = f64::from(vw) / cw;
        let sy = f64::from(vh) / ch;
        let item = &self.items[event.line];
        let safe = self.layout.safe().unwrap_or([
            0,
            0,
            self.layout.size.0 as i32,
            self.layout.size.1 as i32,
        ]);
        let intersects = |b: [f64; 4], other: (i32, i32, u32, u32)| {
            b[0] < f64::from(other.0) + f64::from(other.2)
                && b[0] + b[2] > f64::from(other.0)
                && b[1] < f64::from(other.1) + f64::from(other.3)
                && b[1] + b[3] > f64::from(other.1)
        };
        event.boxes.iter().map(|b| {
            if b.iter().any(|x| !x.is_finite()) || b[2] <= 0.0 || b[3] <= 0.0 {
                return Err(format!("{}: invalid measured geometry", event.id).into());
            }
            let pad = 6.0;
            let r = [f64::from(vx)+(b[0]-pad-cx)*sx, f64::from(vy)+(b[1]-pad-cy)*sy,
                (b[2]+2.0*pad)*sx, (b[3]+2.0*pad)*sy];
            let in_view = r[0] >= f64::from(vx).max(f64::from(safe[0]))
                && r[1] >= f64::from(vy).max(f64::from(safe[1]))
                && r[0]+r[2] <= f64::from(vx+vw).min(f64::from(safe[2]))
                && r[1]+r[3] <= f64::from(vy+vh).min(f64::from(safe[3]));
            let caption = self.cue_at(t).is_some_and(|c| {
                let (img,(x,y)) = &self.cue_pieces[c]; intersects(r,(*x,*y,img.width,img.height))
            });
            let label = !item.label.is_empty() && self.label_pieces.iter().find(|(l,_,_)| *l == item.label)
                .is_some_and(|(_,img,(x,y))| intersects(r,(*x,*y,img.width,img.height)));
            let overlay = item.overlay.as_ref().is_some_and(|img| {
                let (x,y) = self.overlay_at(img); intersects(r,(x,y,img.width,img.height))
            });
            if !in_view {
                return Err(EmphasisError::OutsideCamera(format!("{}: highlight is clipped or covered at frame {f}; target is outside the camera",event.id)));
            }
            if caption || label || overlay {
                return Err(format!("{}: highlight is clipped or covered at frame {f}; adjust its camera/layout", event.id).into());
            }
            Ok(r)
        }).collect()
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
            if self.emphasis.iter().any(|e| e.start <= f && f < e.end) {
                let boundary = self
                    .emphasis
                    .iter()
                    .flat_map(|e| [e.start, e.end])
                    .filter(|at| *at <= f)
                    .max()
                    .unwrap_or(0);
                FrameKey::Emphasized(i, cue, boundary)
            } else {
                FrameKey::Stable(i, cue)
            }
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
            // The picture before fades out under this line's framing when
            // both are of one page. A framing is measured on its own page,
            // so a picture of another size keeps its own.
            let same = self.items[i - 1].space == self.items[i].space;
            let prev_rect = match rect {
                Some(r) if same && !self.items[i - 1].full() => Some(r),
                _ => self.items[i - 1].rect,
            };
            let old = self.visual(i - 1, prev_rect)?;
            picture = pillow::blend(&old, &picture, fade as f32);
        }
        // Base scenes crossfade; emphasis follows only the active event. An
        // expired highlight is never baked into the outgoing scene.
        let mut drawn = std::collections::BTreeSet::new();
        for event in &self.emphasis {
            if event.start <= f && f < event.end {
                for [x, y, w, h] in self.emphasis_boxes(event, f)? {
                    let (l, t, r, b) = (
                        x.floor() as i32,
                        y.floor() as i32,
                        (x + w).ceil() as i32,
                        (y + h).ceil() as i32,
                    );
                    if !drawn.insert((l, t, r, b)) {
                        continue;
                    }
                    pillow::line_h(&mut picture, l, r, t, &self.look.accent, 3);
                    pillow::line_h(&mut picture, l, r, b, &self.look.accent, 3);
                    pillow::rectangle(&mut picture, (l, t, l + 2, b), &self.look.accent);
                    pillow::rectangle(&mut picture, (r - 2, t, r, b), &self.look.accent);
                }
            }
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
        let (still, d, ground) = self.items[i]
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
        let (source, bbox) = around(source, bbox, ground);
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
    /// Stable geometry with the same active emphasis set since this boundary.
    Emphasized(usize, Option<usize>, u64),
    /// During a camera move or crossfade: a frame of its own.
    Moving(u64),
}

/// The pinned face that has the signs an overlay's chips are given
/// (the signs of true and false, a clock) and its arrows.
fn sign_face(fonts: &mut Fonts) -> Result<Font, String> {
    let id = "dejavu-sans-mono-bold";
    let font = vidni_core::fonts::get(id).ok_or_else(|| format!("no pinned font {id}"))?;
    fonts.load(&vidni_core::fonts::find(&font, &vidni_core::fonts::dirs())?)
}

/// Greedy word wrap at `width` pixels, as the reference wraps.
pub(crate) fn wrap(
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
            false,
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
            false,
        );
        assert_eq!(
            r.map(|v| (v * 1000.0).round() / 1000.0),
            [660.0, 0.0, 586.0, 634.833]
        );
    }

    #[test]
    fn a_loose_crop_keeps_its_target_whole_and_reaches_past_the_page() {
        // A strip much wider than the cut: a crop that stays on it shows
        // its middle only; a loose one shows all of it, clear of the captions.
        let strip = [0.0, 0.0, 784.0, 136.0];
        let page = (784.0, 136.0);
        let on = crop_rect(strip, 16.0 / 9.0, &LANDSCAPE, page, 0.0, false);
        assert!(on[2] < 784.0 && on[3] <= 136.0, "{on:?}");
        let [x, y, w, h] = crop_rect(strip, 16.0 / 9.0, &LANDSCAPE, page, 0.0, true);
        assert!(x <= 0.0 && x + w >= 784.0, "all of its width: {x} {w}");
        assert!(
            y <= 0.0 && y + h * 0.8 >= 136.0,
            "above the captions: {y} {h}"
        );
        assert_eq!((w / h * 1000.0).round(), (16.0f64 / 9.0 * 1000.0).round());
        // The page is in the middle of what is left over, across.
        assert_eq!((-x).round(), (x + w - 784.0).round());
        // The last rows of a tall page are framed above the captions, with
        // the room below the page under them.
        let tall = (800.0, 1000.0);
        let [_, y, _, h] = crop_rect(
            [0.0, 900.0, 800.0, 100.0],
            16.0 / 9.0,
            &LANDSCAPE,
            tall,
            0.0,
            true,
        );
        assert!(y + h * 0.8 >= 1000.0 && y + h > 1000.0, "{y} {h}");
        // A target well inside a large page is framed the same either way.
        let part = [400.0, 300.0, 200.0, 100.0];
        let big = (1280.0, 2000.0);
        assert_eq!(
            crop_rect(part, 16.0 / 9.0, &LANDSCAPE, big, 0.0, true),
            crop_rect(part, 16.0 / 9.0, &LANDSCAPE, big, 0.0, false)
        );
    }

    #[test]
    fn a_loose_crop_at_a_pictures_top_starts_below_what_covers_it() {
        // A window's first rows, in a 16:9 cut whose label takes 84 of
        // its 1080 pixels: the window's top is just under the label, not
        // behind it, and the rows are whole above the captions.
        let page = (1200.0, 900.0);
        let label = LABEL / 1080.0;
        let rows = [0.0, 68.0, 1200.0, 180.0];
        let [_, y, _, h] = crop_rect(rows, 16.0 / 9.0, &LANDSCAPE, page, label, true);
        assert!(
            (y + label * h).abs() < 1e-9,
            "the top is at the label's foot: {y} {h}"
        );
        assert!(y + h * 0.8 >= 68.0 + 180.0, "above the captions: {y} {h}");
        // Rows far from the top are framed as they were: in the middle of
        // what the label and the captions leave.
        let low = [0.0, 600.0, 1200.0, 180.0];
        let [_, y, _, h] = crop_rect(low, 16.0 / 9.0, &LANDSCAPE, page, label, true);
        assert!(y > 0.0 && y + label * h < 600.0, "{y} {h}");
        // A page isn't moved: what is under a label there is the page.
        let [_, y, _, _] = crop_rect(rows, 16.0 / 9.0, &LANDSCAPE, (1280.0, 2000.0), label, false);
        assert_eq!(y, 0.0);
    }

    #[test]
    fn the_room_around_a_window_is_its_rows_gone_on() {
        // A bar of one colour over a body of another, and a framing that
        // reaches past every edge.
        let (bar, body) = ([32u8, 35, 44], [20u8, 22, 28]);
        let mut window = Image::new(40, 30, &body);
        pillow::rectangle(&mut window, (0, 0, 39, 9), &bar);
        let past = [-10.0, -6.0, 55.0, 42.0];
        let (room, bbox) = around(window.clone(), past, Ground::Rows);
        assert_eq!((room.width, room.height), (65, 48));
        assert_eq!(bbox, [0.0, 0.0, 65.0, 48.0]);
        // The bar goes on across and above; the body across and below.
        for (x, y) in [(0, 0), (64, 3), (2, 15), (62, 15)] {
            assert_eq!(room.pixel(x, y), bar, "({x}, {y})");
        }
        for (x, y) in [(0, 16), (64, 30), (30, 47), (0, 47)] {
            assert_eq!(room.pixel(x, y), body, "({x}, {y})");
        }
        // A picture's room is its first pixel's colour all around.
        let (room, _) = around(window, past, Ground::Corner);
        for (x, y) in [(0, 0), (64, 30), (30, 47)] {
            assert_eq!(room.pixel(x, y), bar, "({x}, {y})");
        }
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
                strip: None,
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
    fn expired_emphasis_is_absent_from_the_entire_outgoing_crossfade() {
        let dir = tempfile::tempdir().unwrap();
        let measured = |cut: &mut Composer| {
            for item in &mut cut.items {
                item.rect = Some([0.0, 0.0, 1920.0, 1080.0]);
                item.from_rect = item.rect;
            }
        };
        let mut base = two_cards(dir.path(), (1920, 1080)).without_captions();
        measured(&mut base);
        let mut highlighted = two_cards(dir.path(), (1920, 1080)).without_captions();
        measured(&mut highlighted);
        let mut highlighted = highlighted
            .with_emphasis(vec![EmphasisEvent {
                id: "last-word".into(),
                line: 0,
                start: 70,
                end: 90,
                boxes: vec![[600.0, 300.0, 200.0, 70.0]],
            }])
            .unwrap();
        assert_ne!(
            highlighted.frame(89).unwrap().data,
            base.frame(89).unwrap().data
        );
        assert_eq!(highlighted.key(75), highlighted.key(85));
        assert_ne!(highlighted.key(69), highlighted.key(70));
        for f in 90..112 {
            assert_eq!(
                highlighted.frame(f).unwrap().data,
                base.frame(f).unwrap().data,
                "frame {f}"
            );
        }
    }

    #[test]
    fn emphasis_uses_the_intermediate_camera_transform() {
        let dir = tempfile::tempdir().unwrap();
        let mut cut = two_cards(dir.path(), (1920, 1080)).without_captions();
        cut.items[1].from_rect = Some([0.0, 0.0, 1920.0, 1080.0]);
        cut.items[1].rect = Some([200.0, 100.0, 1400.0, 800.0]);
        let event = EmphasisEvent {
            id: "moving".into(),
            line: 1,
            start: 90,
            end: 115,
            boxes: vec![[700.0, 350.0, 90.0, 45.0]],
        };
        let cut = cut.with_emphasis(vec![event.clone()]).unwrap();
        let [x0, y0, w0, h0] = cut.emphasis_boxes(&event, 90).unwrap()[0];
        let [x1, y1, w1, h1] = cut.emphasis_boxes(&event, 111).unwrap()[0];
        let mid = cut.emphasis_boxes(&event, 100).unwrap()[0];
        assert!(mid[0] > x0.min(x1) && mid[0] < x0.max(x1));
        assert!(mid[1] > y0.min(y1) && mid[1] < y0.max(y1));
        assert!(mid[2] > w0.min(w1) && mid[2] < w0.max(w1));
        assert!(mid[3] > h0.min(h1) && mid[3] < h0.max(h1));
    }

    #[test]
    fn overlapping_holds_remain_independent_and_identical_outlines_do_not_flash() {
        let dir = tempfile::tempdir().unwrap();
        let make = || {
            let mut c = two_cards(dir.path(), (1920, 1080)).without_captions();
            for item in &mut c.items {
                item.rect = Some([0.0, 0.0, 1920.0, 1080.0]);
                item.from_rect = item.rect;
            }
            c
        };
        let a = EmphasisEvent {
            id: "name".into(),
            line: 0,
            start: 10,
            end: 50,
            boxes: vec![[600.0, 300.0, 200.0, 70.0]],
        };
        let b = EmphasisEvent {
            id: "same-name".into(),
            start: 30,
            end: 80,
            ..a.clone()
        };
        let mut together = make().with_emphasis(vec![a, b]).unwrap();
        let mut one = make()
            .with_emphasis(vec![EmphasisEvent {
                id: "union".into(),
                line: 0,
                start: 10,
                end: 80,
                boxes: vec![[600.0, 300.0, 200.0, 70.0]],
            }])
            .unwrap();
        for f in [9, 10, 29, 30, 49, 50, 79, 80] {
            assert_eq!(
                together.frame(f).unwrap().data,
                one.frame(f).unwrap().data,
                "frame {f}"
            );
        }
    }

    #[test]
    fn a_card_of_the_wrong_size_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let mut cut = two_cards(dir.path(), (1280, 720));
        let error = cut.frame(0).unwrap_err();
        assert!(error.contains("a card of 1280x720"), "{error}");
    }
}
