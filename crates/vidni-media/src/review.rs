//! Review frames (TODO.md, P6-VIDNI-02): one frame from the middle of each
//! section of a cut, as PNGs to look at. A cut made for platforms has a set
//! for each one, with what that platform's app covers shaded, so that
//! whatever sits under an app's buttons is plain to see.

use crate::compose::cut::{Composer, FPS};
use crate::compose::image::Image;
use crate::compose::pillow;
use crate::compose::text::{Font, Fonts, Pen};
use std::fs;
use std::path::{Path, PathBuf};
use vidni_core::platforms::{self, Insets};
use vidni_core::project::Cut;
use vidni_core::timeline::Timeline;

/// The folder a cut's review frames are in, inside its build's folder.
pub fn folder(build: &Path, cut: &str) -> PathBuf {
    build.join("review").join(cut)
}

/// The shade over what an app covers, and the line at its edge.
const SHADE: [u8; 4] = [226, 54, 66, 80];
const EDGE: [u8; 3] = [226, 54, 66];

/// A frame for each section: the section's ID, and the frame halfway
/// through its middle line, where the picture is still and the line's
/// caption shows.
fn middles(timeline: &Timeline) -> Vec<(String, u64)> {
    let mut sections: Vec<(String, Vec<f64>)> = Vec::new();
    for entry in &timeline.sentences {
        let halfway = (entry.start + entry.end) / 2.0;
        match sections.last_mut() {
            Some((id, lines)) if *id == entry.segment => lines.push(halfway),
            _ => sections.push((entry.segment.clone(), vec![halfway])),
        }
    }
    sections
        .into_iter()
        .map(|(id, lines)| (id, (lines[lines.len() / 2] * FPS) as u64))
        .collect()
}

/// Shades the edges of `frame` that the app of `platform` covers, and names it.
fn shade(frame: &mut Image, covered: Insets, platform: &str, fonts: &mut Fonts, mono: &Font) {
    let (w, h) = (frame.width as i32, frame.height as i32);
    let (top, bottom) = (covered.top as i32, covered.bottom as i32);
    let (left, right) = (covered.left as i32, covered.right as i32);
    // Each band: its corner and its size. The sides stop at the top and
    // the bottom band, so that no pixel is shaded twice.
    let between = h - top - bottom;
    let bands = [
        (0, 0, w, top),
        (0, h - bottom, w, bottom),
        (0, top, left, between),
        (w - right, top, right, between),
    ];
    for (x, y, bw, bh) in bands {
        if bw > 0 && bh > 0 {
            pillow::paste_masked(frame, &Image::new(bw as u32, bh as u32, &SHADE), x, y);
        }
    }
    // A line along the inside of each band.
    let (x0, x1, y0, y1) = (left, w - right - 1, top, h - bottom - 1);
    if top > 0 {
        pillow::rectangle(frame, (x0, y0, x1, y0 + 1), &EDGE);
    }
    if bottom > 0 {
        pillow::rectangle(frame, (x0, y1 - 1, x1, y1), &EDGE);
    }
    if left > 0 {
        pillow::rectangle(frame, (x0, y0, x0 + 1, y1), &EDGE);
    }
    if right > 0 {
        pillow::rectangle(frame, (x1 - 1, y0, x1, y1), &EDGE);
    }
    // The app's name, in the taller of the top and the bottom band.
    let (band, at) = match bottom >= top {
        true => (bottom, h - bottom),
        false => (top, 0),
    };
    if band >= 44 {
        let note =
            format!("{platform} covers: top {top}, bottom {bottom}, left {left}, right {right} px");
        let pen = Pen {
            font: mono,
            size: 24.0,
            ink: [255, 255, 255],
        };
        fonts.draw(frame, (24.0, (at + band - 26) as f32), &note, pen, "lm");
    }
}

/// Writes the review frames of `cut` into `dir`, which is emptied first:
/// for each of its platforms a folder of frames, shaded where that
/// platform's app covers the picture; or, for a cut made for no platform,
/// the frames alone. Returns how many files it wrote.
pub fn write(
    composer: &mut Composer,
    timeline: &Timeline,
    cut: &Cut,
    mono: &Path,
    dir: &Path,
) -> Result<usize, String> {
    if dir.exists() {
        fs::remove_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let mut fonts = Fonts::new();
    let mono = fonts.load(mono)?;
    let size = [composer.size().0, composer.size().1];
    // Each set: its folder, and what its platform's app covers.
    let profiles = platforms::all();
    let mut sets: Vec<(PathBuf, Option<(String, Insets)>)> = cut
        .platforms()
        .into_iter()
        .map(|id| {
            let covered = profiles
                .iter()
                .find(|p| p.id == id)
                .map(|p| Insets::of(&p.safe_zone, size))
                .filter(|covered| !covered.is_none());
            (dir.join(&id), covered.map(|covered| (id, covered)))
        })
        .collect();
    if sets.is_empty() {
        sets.push((dir.to_path_buf(), None));
    }
    for (folder, _) in &sets {
        fs::create_dir_all(folder).map_err(|e| format!("{}: {e}", folder.display()))?;
    }
    let mut written = 0;
    for (n, (section, f)) in middles(timeline).into_iter().enumerate() {
        let frame = composer.frame(f.min(composer.frames().saturating_sub(1)))?;
        for (folder, covered) in &sets {
            let mut frame = frame.clone();
            if let Some((platform, covered)) = covered {
                shade(&mut frame, *covered, platform, &mut fonts, &mono);
            }
            frame.save_png(&folder.join(format!("{n:02}-{section}.png")))?;
            written += 1;
        }
    }
    Ok(written)
}

/// The side of a square of the grid, in pixels.
const CELL: u32 = 120;

/// A picture to measure an app's overlays with: squares of 120 pixels,
/// each with the place of its top left corner written in it, and a fainter
/// line halfway across each. Whatever an app draws over an upload of it,
/// the squares still showing say where the app's own things begin.
pub fn grid((width, height): (u32, u32), fonts: &mut Fonts, mono: &Font) -> Image {
    let (dark, light) = ([18u8, 20, 26], [34u8, 38, 48]);
    let (faint, line, ink) = ([70u8, 76, 92], [150u8, 158, 178], [236u8, 238, 244]);
    let mut img = Image::new(width, height, &dark);
    let (w, h) = (width as i32, height as i32);
    let cell = CELL as i32;
    for (row, y) in (0..h).step_by(CELL as usize).enumerate() {
        for (column, x) in (0..w).step_by(CELL as usize).enumerate() {
            if (row + column) % 2 == 1 {
                pillow::rectangle(&mut img, (x, y, x + cell - 1, y + cell - 1), &light);
            }
        }
    }
    // Lines every 60 pixels: the squares' own edges, and fainter ones between.
    for (at, colour) in (0..w.max(h)).step_by(60).map(|at| match at % cell {
        0 => (at, line),
        _ => (at, faint),
    }) {
        if at < w {
            pillow::rectangle(&mut img, (at, 0, at, h - 1), &colour);
        }
        if at < h {
            pillow::rectangle(&mut img, (0, at, w - 1, at), &colour);
        }
    }
    for y in (0..h).step_by(CELL as usize) {
        for x in (0..w).step_by(CELL as usize) {
            let pen = |size| Pen {
                font: mono,
                size,
                ink,
            };
            let (cx, cy) = ((x + cell / 2) as f32, (y + cell / 2) as f32);
            fonts.draw(
                &mut img,
                (cx, cy - 16.0),
                &format!("x {x}"),
                pen(22.0),
                "mm",
            );
            fonts.draw(
                &mut img,
                (cx, cy + 16.0),
                &format!("y {y}"),
                pen(22.0),
                "mm",
            );
        }
    }
    img
}

/// Writes the grid for a frame of `size` into `dir`, as `grid-<w>x<h>.png`
/// and as a silent clip of `seconds` of it, `grid-<w>x<h>.mp4`, encoded as
/// a cut is. Returns the two files.
pub fn grid_clip(
    size: (u32, u32),
    seconds: u32,
    mono: &Path,
    dir: &Path,
) -> Result<[PathBuf; 2], String> {
    use crate::encode::{Encoder, Settings};
    fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut fonts = Fonts::new();
    let mono = fonts.load(mono)?;
    let picture = grid(size, &mut fonts, &mono);
    let name = format!("grid-{}x{}", size.0, size.1);
    let (png, mp4) = (
        dir.join(format!("{name}.png")),
        dir.join(format!("{name}.mp4")),
    );
    picture.save_png(&png)?;
    // The clip's sound is silence, as long as its picture.
    let silence = dir.join(format!("{name}.silence.wav"));
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 48000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let wav = |e: hound::Error| format!("{}: {e}", silence.display());
    let mut writer = hound::WavWriter::create(&silence, spec).map_err(wav)?;
    for _ in 0..2 * 48000 * seconds {
        writer.write_sample(0i16).map_err(wav)?;
    }
    writer.finalize().map_err(wav)?;
    let settings = Settings::new(size.0, size.1);
    let part = dir.join(format!("{name}.part.mp4"));
    let mut encoder = Encoder::start(&settings, &silence, &part)?;
    let mut pushed = Ok(());
    for _ in 0..settings.fps * seconds {
        pushed = encoder.push(&picture.data);
        if pushed.is_err() {
            break;
        }
    }
    // What ffmpeg said is the reason, not the pipe it left broken.
    encoder.finish()?;
    pushed?;
    fs::rename(&part, &mp4).map_err(|e| format!("{}: {e}", mp4.display()))?;
    fs::remove_file(&silence).map_err(|e| format!("{}: {e}", silence.display()))?;
    Ok([png, mp4])
}

#[cfg(test)]
mod tests {
    use super::*;
    use vidni_core::timeline::{Cue, Entry};

    fn plex() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fonts/IBMPlexSans-Bold.otf")
    }

    fn line(section: &str, start: f64, end: f64) -> Entry {
        Entry {
            k: 0,
            segment: section.into(),
            label: String::new(),
            index: 0,
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

    #[test]
    fn a_review_frame_is_halfway_through_each_sections_middle_line() {
        let timeline = Timeline {
            total: 20.0,
            rate: 22050,
            sentences: vec![
                line("a", 1.0, 2.0),
                line("a", 3.0, 5.0),
                line("a", 6.0, 7.0),
                line("b", 8.0, 9.0),
                line("b", 10.0, 12.0),
                line("a", 13.0, 14.0),
            ],
        };
        let frames = middles(&timeline);
        let named: Vec<(&str, u64)> = frames.iter().map(|(id, f)| (id.as_str(), *f)).collect();
        // Of two lines the second is the middle one. A section that comes
        // back has a frame of its own.
        assert_eq!(named, [("a", 120), ("b", 330), ("a", 405)]);
    }

    #[test]
    fn shading_tints_what_an_app_covers_and_nothing_else() {
        let mut fonts = Fonts::new();
        let mono = fonts.load(&plex()).unwrap();
        let grey = [100u8, 100, 100];
        let mut frame = Image::new(1080, 1920, &grey);
        let covered = Insets {
            top: 288,
            bottom: 672,
            left: 48,
            right: 193,
        };
        shade(&mut frame, covered, "youtube-shorts", &mut fonts, &mono);
        // Inside the free part, away from its edge, nothing changes.
        for (x, y) in [(60, 300), (500, 800), (880, 1240)] {
            assert_eq!(frame.pixel(x, y), grey, "({x}, {y})");
        }
        // Each band is tinted, and so are the corners where two meet, once.
        let tinted = frame.pixel(500, 100).to_vec();
        assert_ne!(tinted, grey);
        assert!(tinted[0] > tinted[1], "the tint is red: {tinted:?}");
        for (x, y) in [(500, 1500), (20, 800), (1000, 800), (20, 100), (1000, 1700)] {
            assert_eq!(frame.pixel(x, y), tinted, "({x}, {y})");
        }
        // The line along the inside of each band.
        for (x, y) in [(500, 288), (500, 1247), (48, 800), (886, 800)] {
            assert_eq!(frame.pixel(x, y), EDGE, "({x}, {y})");
        }
        // The app is named in the taller band.
        let band = (1248..1920).any(|y| (0..1080).any(|x| frame.pixel(x, y) == [255, 255, 255]));
        assert!(band, "no name in the bottom band");
    }

    #[test]
    fn the_grid_marks_every_square_with_its_corner() {
        let mut fonts = Fonts::new();
        let mono = fonts.load(&plex()).unwrap();
        let img = grid((1080, 1920), &mut fonts, &mono);
        assert_eq!((img.width, img.height), (1080, 1920));
        // Squares alternate, their edges are lines, and fainter lines run between.
        let (dark, light) = (img.pixel(10, 10).to_vec(), img.pixel(130, 10).to_vec());
        assert_ne!(dark, light);
        assert_eq!(img.pixel(130, 130), dark);
        assert_eq!(img.pixel(250, 130), light);
        let (line, faint) = (img.pixel(120, 10).to_vec(), img.pixel(60, 10).to_vec());
        assert_eq!(img.pixel(10, 240), line);
        assert_eq!(img.pixel(10, 180), faint);
        assert!(line != faint && line != dark && line != light);
        // Each square holds ink: its corner's place, written in it.
        let ink = [236u8, 238, 244];
        for (x0, y0) in [(0, 0), (960, 1800), (480, 960)] {
            let written =
                (y0 + 1..y0 + 120).any(|y| (x0 + 1..x0 + 120).any(|x| img.pixel(x, y) == ink));
            assert!(written, "the square at ({x0}, {y0}) is empty");
        }
    }
}
