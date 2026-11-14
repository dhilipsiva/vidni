//! Text, drawn as Pillow's `ImageDraw.text` draws it.
//!
//! harfrust (a port of HarfBuzz) shapes each line as one run, as libraqm
//! shapes a line of one script for Pillow, and every position is rounded to
//! 26.6 as FreeType and HarfBuzz round it (see [`Scale`]). swash rasterizes
//! the glyphs, through cosmic-text's cache. The placement is Pillow's
//! (`bounding_box_and_anchors` and `font_render` in `_imagingft.c`), and
//! coverage is blended into the image as Pillow's `fill_mask_L` blends it.
//! Glyph edges can differ from FreeType's by a fraction of a pixel; the
//! compositor's parity is measured by SSIM, which allows for that.
//!
//! Fonts come only from the files given to [`Fonts::load`]: no system font
//! is ever consulted, so a frame doesn't depend on the host.

use super::image::Image;
use super::pillow;
use cosmic_text::{CacheKey, CacheKeyFlags, FontSystem, SwashCache, SwashContent, fontdb};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// A loaded font face.
#[derive(Debug, Clone)]
pub struct Font {
    id: fontdb::ID,
    weight: fontdb::Weight,
    units_per_em: f32,
    ascender: f32,
    descender: f32,
    /// The face that draws the letters this one hasn't: a script's font
    /// has its own letters, and seldom Latin ones.
    fallback: Option<Arc<Font>>,
}

/// How text is drawn: a face, a size in pixels and an ink colour.
#[derive(Debug, Clone, Copy)]
pub struct Pen<'f> {
    pub font: &'f Font,
    pub size: f32,
    pub ink: [u8; 3],
}

/// The font faces a cut uses, and the glyph cache.
pub struct Fonts {
    system: FontSystem,
    cache: SwashCache,
    faces: HashMap<fontdb::ID, Face>,
}

/// What shaping needs of a loaded face.
struct Face {
    bytes: Arc<Vec<u8>>,
    shaping: harfrust::ShaperData,
    /// Each glyph's advance in font units, from `hmtx`.
    advances: Vec<u16>,
}

/// A shaped glyph, its advance and offsets in 26.6.
struct Glyph {
    /// The face it is a glyph of, and that face's weight.
    face: fontdb::ID,
    weight: fontdb::Weight,
    id: u16,
    advance: i64,
    x_offset: i64,
    y_offset: i64,
}

impl Default for Fonts {
    fn default() -> Self {
        Self::new()
    }
}

impl Fonts {
    /// An empty set: no system fonts.
    pub fn new() -> Self {
        Self {
            system: FontSystem::new_with_locale_and_db("en-US".into(), fontdb::Database::new()),
            cache: SwashCache::new(),
            faces: HashMap::new(),
        }
    }

    /// Loads one font file (its first face).
    pub fn load(&mut self, path: &Path) -> Result<Font, String> {
        let bytes = Arc::new(std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?);
        let not_a_font = || format!("{}: not a font", path.display());
        let (units_per_em, ascender, descender, advances) =
            ttf_metrics(&bytes).ok_or_else(not_a_font)?;
        let shaping = harfrust::ShaperData::new(
            &harfrust::FontRef::from_index(&bytes, 0).map_err(|_| not_a_font())?,
        );
        let ids = self
            .system
            .db_mut()
            .load_font_source(fontdb::Source::Binary(bytes.clone()));
        let id = *ids
            .first()
            .ok_or_else(|| format!("{}: no face in the file", path.display()))?;
        let weight = self.system.db().face(id).ok_or("a loaded face")?.weight;
        self.faces.insert(
            id,
            Face {
                bytes,
                shaping,
                advances,
            },
        );
        Ok(Font {
            id,
            weight,
            units_per_em,
            ascender,
            descender,
            fallback: None,
        })
    }

    /// `font`, with the letters it hasn't drawn in `other`. A line is laid
    /// out by `font`'s own height.
    pub fn with_fallback(&self, font: &Font, other: &Font) -> Font {
        Font {
            fallback: Some(Arc::new(other.clone())),
            ..font.clone()
        }
    }

    /// Whether `font`'s face has a glyph for `c`.
    fn has(&self, font: &Font, c: char) -> bool {
        ttf_parser::Face::parse(&self.faces[&font.id].bytes, 0)
            .is_ok_and(|face| face.glyph_index(c).is_some())
    }

    /// `text` shaped, positioned as Pillow's layout positions it: as one
    /// run in `font`, or, where `font` has a fallback, as runs of the
    /// letters each face has.
    fn shape(&self, font: &Font, size: f32, text: &str) -> Vec<Glyph> {
        let Some(other) = font.fallback.as_deref() else {
            return self.shape_run(font, size, text);
        };
        // A letter goes to the fallback only when the font hasn't it and
        // the fallback has; what neither has stays with the font.
        let mut runs: Vec<(bool, String)> = Vec::new();
        for c in text.chars() {
            let fallen = !self.has(font, c) && self.has(other, c);
            match runs.last_mut() {
                Some((kind, run)) if *kind == fallen => run.push(c),
                _ => runs.push((fallen, c.to_string())),
            }
        }
        runs.iter()
            .flat_map(|(fallen, run)| self.shape_run(if *fallen { other } else { font }, size, run))
            .collect()
    }

    /// `text` shaped as one run in one face.
    fn shape_run(&self, font: &Font, size: f32, text: &str) -> Vec<Glyph> {
        let face = &self.faces[&font.id];
        let Ok(font_ref) = harfrust::FontRef::from_index(&face.bytes, 0) else {
            return Vec::new();
        };
        let shaper = face.shaping.shaper(&font_ref).build();
        let mut buffer = harfrust::UnicodeBuffer::new();
        buffer.push_str(text);
        buffer.guess_segment_properties();
        let shaped = shaper.shape(buffer, &[]);
        let scale = Scale::new(size, font.units_per_em);
        shaped
            .glyph_infos()
            .iter()
            .zip(shaped.glyph_positions())
            .map(|(info, position)| {
                // Shaping ran at units-per-em scale, so the advance in font
                // units is exact: the `hmtx` advance plus the GPOS
                // adjustment (kerning), which is scaled on its own. HarfBuzz
                // scales each GPOS value it applies; a glyph moved by more
                // than one lookup could land 1/64 pixel away here. Latin
                // kerning moves a glyph once.
                let id = info.glyph_id as u16;
                let base = i64::from(face.advances.get(usize::from(id)).copied().unwrap_or(0));
                Glyph {
                    face: font.id,
                    weight: font.weight,
                    id,
                    advance: scale.advance(base)
                        + scale.value(i64::from(position.x_advance) - base),
                    x_offset: scale.value(i64::from(position.x_offset)),
                    y_offset: scale.value(i64::from(position.y_offset)),
                }
            })
            .collect()
    }

    /// The advance width of `text`, as Pillow's `ImageFont.getlength`.
    pub fn length(&self, font: &Font, size: f32, text: &str) -> f32 {
        let advance26: i64 = self.shape(font, size, text).iter().map(|g| g.advance).sum();
        advance26 as f32 / 64.0
    }

    /// Draws `text` at `(x, y)` with Pillow's `anchor` (`lm`, `mm`, `ls` …):
    /// the first letter picks left, middle or right, the second ascender,
    /// middle, baseline or descender. Pillow's `t` and `b`, which depend on
    /// the glyphs' bounds, aren't supported.
    pub fn draw(
        &mut self,
        img: &mut Image,
        (x, y): (f32, f32),
        text: &str,
        pen: Pen,
        anchor: &str,
    ) {
        let Pen { font, size, ink } = pen;
        debug_assert!(
            matches!(
                anchor.as_bytes(),
                [b'l' | b'm' | b'r', b'a' | b'm' | b's' | b'd']
            ),
            "unsupported anchor {anchor:?}"
        );
        let glyphs = self.shape(font, size, text);
        let pixel = |v26: i64| (v26 + 32) >> 6;
        let advance26: i64 = glyphs.iter().map(|g| g.advance).sum();
        let x_anchor = match anchor.as_bytes().first() {
            Some(b'm') => pixel(advance26 / 2),
            Some(b'r') => pixel(advance26),
            _ => 0,
        };
        // FreeType rounds the scaled ascender up and the descender down to whole pixels.
        let scale = f64::from(size) / f64::from(font.units_per_em);
        let ascender26 = ((f64::from(font.ascender) * scale).ceil() * 64.0) as i64;
        let descender26 = ((f64::from(font.descender) * scale).floor() * 64.0) as i64;
        let y_anchor = match anchor.as_bytes().get(1) {
            Some(b'a') => pixel(ascender26),
            Some(b'm') => pixel((ascender26 + descender26) / 2),
            Some(b'd') => pixel(descender26),
            _ => 0,
        };
        // Pillow never draws at a fraction of a pixel. It splits the start
        // into a whole part and a fraction, adds the fraction (in 26.6) to
        // each glyph's pen position, and rounds that to a whole pixel
        // (`PIXEL`); every glyph is then rasterized on the pixel grid. Its
        // vertical pen points up, as HarfBuzz's offsets do.
        let to26 = |v: f32| (f64::from(v) * 64.0).round() as i64;
        let (start_x26, start_y26) = (to26(x.fract()), to26(y.fract()));
        let base = (x.trunc() as i64 - x_anchor, y.trunc() as i64 + y_anchor);
        // Like Pillow, rasterize every glyph into one coverage mask (the
        // larger value wins where glyphs overlap), then blend it in once.
        let mut placed: Vec<(i32, i32, u32, u32, Vec<u8>)> = Vec::new();
        let mut pen26 = 0;
        for glyph in &glyphs {
            let at = (
                base.0 + pixel(start_x26 + pen26 + glyph.x_offset),
                base.1 - pixel(glyph.y_offset - start_y26),
            );
            pen26 += glyph.advance;
            let (key, gx, gy) = CacheKey::new(
                glyph.face,
                glyph.id,
                size,
                (at.0 as f32, at.1 as f32),
                glyph.weight,
                CacheKeyFlags::empty(),
            );
            let Some(image) = self.cache.get_image(&mut self.system, key).as_ref() else {
                continue;
            };
            if !matches!(image.content, SwashContent::Mask) || image.placement.width == 0 {
                continue;
            }
            let p = image.placement;
            placed.push((
                gx + p.left,
                gy - p.top,
                p.width,
                p.height,
                image.data.clone(),
            ));
        }
        let (Some(x0), Some(y0)) = (
            placed.iter().map(|g| g.0).min(),
            placed.iter().map(|g| g.1).min(),
        ) else {
            return;
        };
        let x1 = placed.iter().map(|g| g.0 + g.2 as i32).max().unwrap_or(x0);
        let y1 = placed.iter().map(|g| g.1 + g.3 as i32).max().unwrap_or(y0);
        let (mw, mh) = ((x1 - x0) as u32, (y1 - y0) as u32);
        let mut mask = vec![0u8; (mw * mh) as usize];
        for (gx, gy, gw, gh, data) in &placed {
            for row in 0..*gh as i32 {
                for col in 0..*gw as i32 {
                    let at = ((gy - y0 + row) * mw as i32 + (gx - x0 + col)) as usize;
                    mask[at] = mask[at].max(data[(row * *gw as i32 + col) as usize]);
                }
            }
        }
        pillow::draw_bitmap(img, (x0, y0), (mw, mh), &mask, ink);
    }
}

/// Units per em, ascender and descender (hhea, or OS/2 typo metrics when
/// the font asks for them), as FreeType reads them, and every glyph's
/// advance.
fn ttf_metrics(bytes: &[u8]) -> Option<(f32, f32, f32, Vec<u16>)> {
    let face = ttf_parser::Face::parse(bytes, 0).ok()?;
    let advances = (0..face.number_of_glyphs())
        .map(|g| face.glyph_hor_advance(ttf_parser::GlyphId(g)).unwrap_or(0))
        .collect();
    Some((
        f32::from(face.units_per_em()),
        f32::from(face.ascender()),
        f32::from(face.descender()),
        advances,
    ))
}

/// The fixed-point scaling Pillow's layout uses at one size. Pillow gets
/// positions from HarfBuzz over FreeType (`hb-ft`), and both work in whole
/// 26.6 units: a glyph's advance is FreeType's scaled `hmtx` advance,
/// rounded to 1/64 pixel, plus its GPOS adjustment (kerning), rounded on
/// its own. Summing float advances instead can land 1/64 pixel away, and a
/// caption box sized from that is a pixel off.
struct Scale {
    /// FreeType's `x_scale` (16.16): font units to 26.6.
    freetype: i64,
    /// HarfBuzz's `x_mult` for GPOS values.
    harfbuzz: i64,
}

impl Scale {
    fn new(size: f32, units_per_em: f32) -> Self {
        let upem = units_per_em as i64;
        let size26 = (f64::from(size) * 64.0) as i64;
        // FT_DivFix(size26, upem), as FT_Request_Size sets it.
        let freetype = ((size26 << 16) + upem / 2) / upem;
        // hb_ft_font_create's scale, then hb_font_t::mults_changed.
        let font_scale = (freetype * upem + (1 << 15)) >> 16;
        let harfbuzz = (font_scale << 16) / upem;
        Self { freetype, harfbuzz }
    }

    /// An `hmtx` advance in 26.6: FT_Get_Advance's 16.16
    /// (`FT_MulDiv(units, x_scale, 64)`), which hb-ft rounds to 26.6.
    fn advance(&self, units: i64) -> i64 {
        let fixed = (units * self.freetype + 32) / 64;
        (fixed + 512) >> 10
    }

    /// A GPOS value in 26.6 (HarfBuzz's `em_mult`).
    fn value(&self, units: i64) -> i64 {
        (units * self.harfbuzz + 32768) >> 16
    }
}
