//! Plain 8-bit images, RGB or RGBA, laid out as Pillow keeps them: rows
//! top to bottom, channels interleaved, alpha straight (not premultiplied).

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

/// An 8-bit image with 3 (RGB) or 4 (RGBA) channels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub channels: usize,
    pub data: Vec<u8>,
}

impl Image {
    /// An image filled with one colour (`fill` has `channels` bytes).
    pub fn new(width: u32, height: u32, fill: &[u8]) -> Self {
        let channels = fill.len();
        assert!(
            channels == 3 || channels == 4,
            "an image has 3 or 4 channels"
        );
        let data = fill.repeat((width * height) as usize);
        Self {
            width,
            height,
            channels,
            data,
        }
    }

    /// Reads a PNG as RGB, or RGBA when it has alpha. Grey images become RGB,
    /// and 16-bit samples keep their high byte. (Pillow differs on 16-bit
    /// grey: it clips the value at 255. The reference's pictures are 8-bit.)
    pub fn load_png(path: &Path) -> Result<Self, String> {
        let shown = |e: &dyn std::fmt::Display| format!("{}: {e}", path.display());
        let mut decoder = png::Decoder::new(std::io::BufReader::new(
            File::open(path).map_err(|e| shown(&e))?,
        ));
        decoder.set_transformations(png::Transformations::normalize_to_color8());
        let mut reader = decoder.read_info().map_err(|e| shown(&e))?;
        let mut buf = vec![
            0;
            reader
                .output_buffer_size()
                .ok_or_else(|| shown(&"image too large"))?
        ];
        let info = reader.next_frame(&mut buf).map_err(|e| shown(&e))?;
        buf.truncate(info.buffer_size());
        let (channels, data) = match info.color_type {
            png::ColorType::Rgb => (3, buf),
            png::ColorType::Rgba => (4, buf),
            png::ColorType::Grayscale => (3, buf.iter().flat_map(|&g| [g, g, g]).collect()),
            png::ColorType::GrayscaleAlpha => (
                4,
                buf.chunks(2)
                    .flat_map(|p| [p[0], p[0], p[0], p[1]])
                    .collect(),
            ),
            other => return Err(shown(&format!("unexpected colour type {other:?}"))),
        };
        Ok(Self {
            width: info.width,
            height: info.height,
            channels,
            data,
        })
    }

    /// Writes the image as a PNG.
    pub fn save_png(&self, path: &Path) -> Result<(), String> {
        let shown = |e: &dyn std::fmt::Display| format!("{}: {e}", path.display());
        let file = File::create(path).map_err(|e| shown(&e))?;
        let mut encoder = png::Encoder::new(BufWriter::new(file), self.width, self.height);
        encoder.set_color(if self.channels == 4 {
            png::ColorType::Rgba
        } else {
            png::ColorType::Rgb
        });
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|e| shown(&e))?;
        writer.write_image_data(&self.data).map_err(|e| shown(&e))
    }

    /// The image without its alpha, as Pillow's `convert("RGB")` makes it.
    pub fn to_rgb(&self) -> Self {
        if self.channels == 3 {
            return self.clone();
        }
        let data = self
            .data
            .chunks(4)
            .flat_map(|p| [p[0], p[1], p[2]])
            .collect();
        Self {
            width: self.width,
            height: self.height,
            channels: 3,
            data,
        }
    }

    /// The image with an opaque alpha, as Pillow's `convert("RGBA")` makes it from RGB.
    pub fn to_rgba(&self) -> Self {
        if self.channels == 4 {
            return self.clone();
        }
        let data = self
            .data
            .chunks(3)
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect();
        Self {
            width: self.width,
            height: self.height,
            channels: 4,
            data,
        }
    }

    /// The bytes of the pixel at (x, y).
    pub fn pixel(&self, x: u32, y: u32) -> &[u8] {
        let at = (y as usize * self.width as usize + x as usize) * self.channels;
        &self.data[at..at + self.channels]
    }

    /// The box around the image's ink, as its corners (left, top, right,
    /// bottom; the last two just past the ink): every pixel that isn't
    /// `background`, and that isn't fully transparent. `None` when there
    /// is no ink.
    pub fn ink(&self, background: &[u8]) -> Option<[i32; 4]> {
        let mut found: Option<[i32; 4]> = None;
        for y in 0..self.height {
            let row = self.row(y);
            let inked = |x: usize| {
                let pixel = &row[x * self.channels..(x + 1) * self.channels];
                pixel[..3] != background[..3] && pixel.get(3) != Some(&0)
            };
            let width = self.width as usize;
            let Some(first) = (0..width).find(|x| inked(*x)) else {
                continue;
            };
            let last = (0..width).rev().find(|x| inked(*x)).unwrap_or(first);
            let (first, last, y) = (first as i32, last as i32 + 1, y as i32);
            found = Some(match found {
                None => [first, y, last, y + 1],
                Some([x0, y0, x1, _]) => [x0.min(first), y0, x1.max(last), y + 1],
            });
        }
        found
    }

    pub(crate) fn row(&self, y: u32) -> &[u8] {
        let len = self.width as usize * self.channels;
        &self.data[y as usize * len..(y as usize + 1) * len]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ink_is_what_differs_from_the_background_and_shows() {
        let grey = [9u8, 9, 9];
        let mut img = Image::new(8, 6, &[9, 9, 9, 255]);
        assert_eq!(img.ink(&grey), None);
        let mut set = |x: usize, y: usize, pixel: [u8; 4]| {
            let at = (y * 8 + x) * 4;
            img.data[at..at + 4].copy_from_slice(&pixel);
        };
        set(2, 1, [200, 0, 0, 255]);
        set(5, 4, [0, 0, 200, 128]);
        // A pixel of another colour that is fully transparent doesn't show.
        set(7, 5, [0, 200, 0, 0]);
        assert_eq!(img.ink(&grey), Some([2, 1, 6, 5]));
        // Without alpha, every pixel of another colour is ink.
        assert_eq!(img.to_rgb().ink(&grey), Some([2, 1, 8, 6]));
    }
}
