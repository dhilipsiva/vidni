//! The image operations the reference's compositor used, ported from
//! Pillow's C code (libImaging) so that they give the same bytes:
//! - `resize_bicubic`: `Image.resize(size, BICUBIC, box=...)`
//!   (Resample.c, with 22-bit fixed-point coefficients);
//! - `blend`: `Image.blend` (Blend.c, in single precision);
//! - `paste_masked` and `paste`: `Image.paste` with and without an RGBA
//!   mask (Paste.c);
//! - `rectangle`, `line_h` and `rounded_rectangle`: ImageDraw's shapes,
//!   which Pillow draws without anti-aliasing (Draw.c).
//!
//! `tests/pillow.rs` checks each against Pillow's own output.

use super::image::Image;

const PRECISION_BITS: u32 = 32 - 8 - 2;

fn bicubic(x: f64) -> f64 {
    const A: f64 = -0.5;
    let x = x.abs();
    if x < 1.0 {
        ((A + 2.0) * x - (A + 3.0)) * x * x + 1.0
    } else if x < 2.0 {
        (((x - 5.0) * x + 8.0) * x - 4.0) * A
    } else {
        0.0
    }
}

/// Each output pixel's first input pixel and count, and the fixed-point
/// weights, `ksize` per output pixel (Resample.c, `precompute_coeffs` and
/// `normalize_coeffs_8bpc`). `in0` and `in1` are single precision, as
/// Pillow parses the box.
fn coefficients(
    in_size: u32,
    in0: f32,
    in1: f32,
    out_size: u32,
) -> (Vec<(usize, usize)>, Vec<i32>, usize) {
    let scale = f64::from(in1 - in0) / f64::from(out_size);
    let filterscale = scale.max(1.0);
    let support = 2.0 * filterscale;
    let ksize = support.ceil() as usize * 2 + 1;
    let mut bounds = Vec::with_capacity(out_size as usize);
    let mut weights = vec![0i32; out_size as usize * ksize];
    let mut k = vec![0f64; ksize];
    for xx in 0..out_size as usize {
        let center = f64::from(in0) + (xx as f64 + 0.5) * scale;
        let ss = 1.0 / filterscale;
        let xmin = ((center - support + 0.5) as i32).max(0);
        let xmax = ((center + support + 0.5) as i32).min(in_size as i32) - xmin;
        let mut ww = 0.0;
        k.iter_mut().for_each(|w| *w = 0.0);
        for (x, slot) in k.iter_mut().enumerate().take(xmax.max(0) as usize) {
            let w = bicubic((f64::from(x as i32 + xmin) - center + 0.5) * ss);
            *slot = w;
            ww += w;
        }
        for (x, w) in k.iter().enumerate() {
            let w = if ww != 0.0 { w / ww } else { *w };
            weights[xx * ksize + x] = if w < 0.0 {
                (-0.5 + w * f64::from(1u32 << PRECISION_BITS)) as i32
            } else {
                (0.5 + w * f64::from(1u32 << PRECISION_BITS)) as i32
            };
        }
        bounds.push((xmin as usize, xmax.max(0) as usize));
    }
    (bounds, weights, ksize)
}

fn clip8(sum: i32) -> u8 {
    (sum >> PRECISION_BITS).clamp(0, 255) as u8
}

/// `src.resize((width, height), Image.BICUBIC, box=bbox)`.
pub fn resize_bicubic(src: &Image, (width, height): (u32, u32), bbox: [f32; 4]) -> Image {
    let c = src.channels;
    let need_h = width != src.width || bbox[0] != 0.0 || bbox[2] != width as f32;
    let need_v = height != src.height || bbox[1] != 0.0 || bbox[3] != height as f32;
    let (hb, hk, hks) = coefficients(src.width, bbox[0], bbox[2], width);
    let (mut vb, vk, vks) = coefficients(src.height, bbox[1], bbox[3], height);
    let y_first = vb[0].0;
    let y_last = vb[height as usize - 1].0 + vb[height as usize - 1].1;
    let half = 1i32 << (PRECISION_BITS - 1);
    let temp = if need_h {
        for b in vb.iter_mut() {
            b.0 -= y_first;
        }
        let rows = (y_last - y_first) as u32;
        let mut out = Image {
            width,
            height: rows,
            channels: c,
            data: vec![0; (width * rows) as usize * c],
        };
        for yy in 0..rows {
            let row = src.row(yy + y_first as u32);
            for (xx, &(xmin, count)) in hb.iter().enumerate() {
                let k = &hk[xx * hks..xx * hks + count];
                for ch in 0..c {
                    let mut sum = half;
                    for (x, w) in k.iter().enumerate() {
                        sum += i32::from(row[(x + xmin) * c + ch]) * w;
                    }
                    out.data[(yy as usize * width as usize + xx) * c + ch] = clip8(sum);
                }
            }
        }
        out
    } else {
        // Without a horizontal pass, the vertical one reads the source directly.
        src.clone()
    };
    if !need_v {
        return temp;
    }
    let mut out = Image {
        width: temp.width,
        height,
        channels: c,
        data: vec![0; (temp.width * height) as usize * c],
    };
    let stride = temp.width as usize * c;
    for (yy, &(ymin, count)) in vb.iter().enumerate() {
        let k = &vk[yy * vks..yy * vks + count];
        for i in 0..stride {
            let mut sum = half;
            for (y, w) in k.iter().enumerate() {
                sum += i32::from(temp.data[(y + ymin) * stride + i]) * w;
            }
            out.data[yy * stride + i] = clip8(sum);
        }
    }
    out
}

/// `Image.blend(a, b, alpha)`: each byte is `a + alpha * (b - a)` in single
/// precision, truncated.
pub fn blend(a: &Image, b: &Image, alpha: f32) -> Image {
    let data = a
        .data
        .iter()
        .zip(&b.data)
        .map(|(&x, &y)| (f32::from(x) + alpha * (i32::from(y) - i32::from(x)) as f32) as u8)
        .collect();
    Image { data, ..a.clone() }
}

/// Pillow's DIV255: `(v + 128 + ((v + 128) >> 8)) >> 8`.
fn div255(v: u32) -> u8 {
    let t = v + 128;
    (((t >> 8) + t) >> 8) as u8
}

/// `dst.paste(src, (x, y), src)`: `src` is RGBA and its alpha is the mask.
/// Parts outside `dst` are clipped.
pub fn paste_masked(dst: &mut Image, src: &Image, x: i32, y: i32) {
    let dc = dst.channels;
    for sy in 0..src.height as i32 {
        let dy = y + sy;
        if dy < 0 || dy >= dst.height as i32 {
            continue;
        }
        for sx in 0..src.width as i32 {
            let dx = x + sx;
            if dx < 0 || dx >= dst.width as i32 {
                continue;
            }
            let s = src.pixel(sx as u32, sy as u32);
            let a = u32::from(s[3]);
            let at = (dy as usize * dst.width as usize + dx as usize) * dc;
            for ch in 0..dc {
                let source = if ch < 3 { s[ch] } else { s[3] };
                let out = u32::from(dst.data[at + ch]);
                dst.data[at + ch] = div255(out * (255 - a) + u32::from(source) * a);
            }
        }
    }
}

/// `dst.paste(src, (x, y))` with no mask: the pixels are copied (RGB onto RGB).
pub fn paste(dst: &mut Image, src: &Image, x: i32, y: i32) {
    for sy in 0..src.height as i32 {
        let dy = y + sy;
        if dy < 0 || dy >= dst.height as i32 {
            continue;
        }
        for sx in 0..src.width as i32 {
            let dx = x + sx;
            if dx < 0 || dx >= dst.width as i32 {
                continue;
            }
            let s = src.pixel(sx as u32, sy as u32);
            let at = (dy as usize * dst.width as usize + dx as usize) * dst.channels;
            dst.data[at..at + dst.channels].copy_from_slice(&s[..dst.channels]);
        }
    }
}

/// ImageDraw's `rectangle((x0, y0, x1, y1), fill=color)`: inclusive bounds, clipped.
pub fn rectangle(img: &mut Image, (x0, y0, x1, y1): (i32, i32, i32, i32), color: &[u8]) {
    for y in y0.max(0)..=y1.min(img.height as i32 - 1) {
        for x in x0.max(0)..=x1.min(img.width as i32 - 1) {
            let at = (y as usize * img.width as usize + x as usize) * img.channels;
            img.data[at..at + img.channels].copy_from_slice(&color[..img.channels]);
        }
    }
}

/// ImageDraw's `line((x0, y, x1, y), fill=color, width=w)` for a horizontal
/// line: Pillow's wide-line polygon covers rows y to y + w / 2 rounded,
/// and up to (w - 1) / 2 rounded above.
pub fn line_h(img: &mut Image, x0: i32, x1: i32, y: i32, color: &[u8], width: i32) {
    let half = f64::from(width - 1) / 2.0;
    let below = (half + 0.5).floor() as i32;
    let above = (half - 0.5).ceil() as i32;
    rectangle(img, (x0, y - above, x1, y + below), color);
}

/// One quarter of an ellipse on Pillow's doubled grid (Draw.c, `quarter_state`).
struct Quarter {
    cx: i32,
    cy: i32,
    ex: i32,
    ey: i32,
    a2: i64,
    b2: i64,
    a2b2: i64,
    finished: bool,
}

impl Quarter {
    fn new(a: i32, b: i32) -> Self {
        let (a2, b2) = (i64::from(a) * i64::from(a), i64::from(b) * i64::from(b));
        Self {
            cx: a,
            cy: b % 2,
            ex: a % 2,
            ey: b,
            a2,
            b2,
            a2b2: a2 * b2,
            finished: a < 0 || b < 0,
        }
    }

    fn delta(&self, x: i32, y: i32) -> i64 {
        let (x, y) = (i64::from(x), i64::from(y));
        (self.a2 * y * y + self.b2 * x * x - self.a2b2).abs()
    }

    fn next(&mut self) -> Option<(i32, i32)> {
        if self.finished {
            return None;
        }
        let point = (self.cx, self.cy);
        if self.cx == self.ex && self.cy == self.ey {
            self.finished = true;
        } else {
            let (mut nx, mut ny) = (self.cx, self.cy + 2);
            let mut nd = self.delta(nx, ny);
            if nx > 1 {
                let d = self.delta(self.cx - 2, self.cy + 2);
                if nd > d {
                    (nx, ny, nd) = (self.cx - 2, self.cy + 2, d);
                }
                let d = self.delta(self.cx - 2, self.cy);
                if nd > d {
                    (nx, ny) = (self.cx - 2, self.cy);
                }
            }
            (self.cx, self.cy) = (nx, ny);
        }
        Some(point)
    }
}

/// The horizontal spans `(x0, y, x1)`, on the doubled grid, of an ellipse
/// with half-axes `a` and `b` (doubled) and a ring `w` pixels thick
/// (Draw.c, `ellipse_init` and `ellipse_next`).
fn ellipse_spans(a: i32, b: i32, w: i32) -> Vec<(i32, i32, i32)> {
    let mut spans = Vec::new();
    let leftmost = a % 2;
    let mut outer = Quarter::new(a, b);
    let Some((mut pr, mut py)) = outer.next().filter(|_| w >= 1) else {
        return spans;
    };
    let mut inner = Quarter::new(a - 2 * (w - 1), b - 2 * (w - 1));
    let mut pl = leftmost;
    let mut finished = false;
    while !finished {
        let (y, mut l, r) = (py, pl, pr);
        let mut next = outer.next();
        while let Some((_, cy)) = next.filter(|&(_, cy)| cy <= y) {
            let _ = cy;
            next = outer.next();
        }
        match next {
            None => finished = true,
            Some((cx, cy)) => (pr, py) = (cx, cy),
        }
        let mut next = inner.next();
        while let Some((cx, _)) = next.filter(|&(_, cy)| cy <= y) {
            l = cx;
            next = inner.next();
        }
        pl = next.map_or(leftmost, |(cx, _)| cx);
        if (l > 0 || l < r) && y > 0 {
            spans.push((if l == 0 { 2 } else { l }, y, r));
        }
        if y > 0 {
            spans.push((-r, y, -l));
        }
        if l > 0 || l < r {
            spans.push((if l == 0 { 2 } else { l }, -y, r));
        }
        spans.push((-r, -y, -l));
    }
    spans
}

/// Draws the part of the ellipse in bounding box `(x0, y0, x1, y1)` that
/// lies in `clip` (inclusive pixel bounds): a filled ellipse, or a ring
/// `width` pixels thick.
fn ellipse_in(
    img: &mut Image,
    (x0, y0, x1, y1): (i32, i32, i32, i32),
    fill: bool,
    width: i32,
    clip: (i32, i32, i32, i32),
    color: &[u8],
) {
    let (a, b) = (x1 - x0, y1 - y0);
    let width = if fill { a + b } else { width };
    for (sx0, sy, sx1) in ellipse_spans(a, b, width) {
        let y = y0 + (sy + b) / 2;
        let (left, right) = (x0 + (sx0 + a) / 2, x0 + (sx1 + a) / 2);
        if y < clip.1 || y > clip.3 {
            continue;
        }
        rectangle(img, (left.max(clip.0), y, right.min(clip.2), y), color);
    }
}

/// ImageDraw's `rounded_rectangle((x0, y0, x1, y1), radius, fill, outline)`
/// with all four corners round and an outline 1 pixel wide. Each corner is a
/// quarter of Pillow's ellipse, filled and then outlined.
pub fn rounded_rectangle(
    img: &mut Image,
    (x0, y0, x1, y1): (i32, i32, i32, i32),
    radius: i32,
    fill: &[u8],
    outline: &[u8],
) {
    let d = radius * 2;
    assert!(
        d < x1 - x0 - 1 && d < y1 - y0 - 1,
        "a corner radius this large isn't supported"
    );
    let r = d / 2;
    let corners = [
        ((x1 - d, y0, x1, y0 + d), (x1 - r, y0, x1, y0 + r)),
        ((x1 - d, y1 - d, x1, y1), (x1 - r, y1 - r, x1, y1)),
        ((x0, y1 - d, x0 + d, y1), (x0, y1 - r, x0 + r, y1)),
        ((x0, y0, x0 + d, y0 + d), (x0, y0, x0 + r, y0 + r)),
    ];
    for (bbox, quadrant) in corners {
        ellipse_in(img, bbox, true, 0, quadrant, fill);
    }
    if x1 - r - 1 > x0 + r + 1 {
        rectangle(img, (x0 + r + 1, y0, x1 - r - 1, y1), fill);
    }
    rectangle(img, (x0, y0 + r + 1, x0 + r, y1 - r - 1), fill);
    rectangle(img, (x1 - r, y0 + r + 1, x1, y1 - r - 1), fill);
    for (bbox, quadrant) in corners {
        ellipse_in(img, bbox, false, 1, quadrant, outline);
    }
    rectangle(img, (x0 + r + 1, y0, x1 - r - 1, y0), outline);
    rectangle(img, (x0 + r + 1, y1, x1 - r - 1, y1), outline);
    rectangle(img, (x0, y0 + r + 1, x0, y1 - r - 1), outline);
    rectangle(img, (x1, y0 + r + 1, x1, y1 - r - 1), outline);
}

/// ImageDraw's `bitmap((x, y), mask, fill=ink)`, which `ImageDraw.text`
/// uses too (Paste.c, `fill_mask_L`): `ink` is blended in by each mask
/// value. On RGBA, a fully transparent pixel takes the ink's colour as is
/// and the mask as its alpha; any other pixel blends colour and alpha alike.
pub fn draw_bitmap(
    img: &mut Image,
    (x, y): (i32, i32),
    (mw, mh): (u32, u32),
    mask: &[u8],
    ink: [u8; 3],
) {
    let rgba = img.channels == 4;
    for my in 0..mh as i32 {
        let py = y + my;
        if py < 0 || py >= img.height as i32 {
            continue;
        }
        for mx in 0..mw as i32 {
            let px = x + mx;
            let m = mask[(my * mw as i32 + mx) as usize];
            if m == 0 || px < 0 || px >= img.width as i32 {
                continue;
            }
            let at = (py as usize * img.width as usize + px as usize) * img.channels;
            let m = u32::from(m);
            if rgba && img.data[at + 3] == 0 {
                img.data[at..at + 3].copy_from_slice(&ink);
                img.data[at + 3] = div255(255 * m);
                continue;
            }
            for (i, &c) in ink.iter().enumerate() {
                img.data[at + i] =
                    div255(u32::from(img.data[at + i]) * (255 - m) + u32::from(c) * m);
            }
            if rgba {
                img.data[at + 3] = div255(u32::from(img.data[at + 3]) * (255 - m) + 255 * m);
            }
        }
    }
}
