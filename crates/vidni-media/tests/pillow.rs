//! The Pillow ports give Pillow's own bytes (fixtures from tools/pillow_fixtures.py).
//! Text is the exception: its layout is Pillow's to the 1/64 pixel, but its
//! glyphs are rasterized by swash, not FreeType.

use std::path::{Path, PathBuf};
use vidni_media::compose::image::Image;
use vidni_media::compose::pillow;
use vidni_media::compose::text::{Font, Fonts, Pen};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pillow")
        .join(name)
}

fn load(name: &str) -> Image {
    Image::load_png(&fixture(name)).unwrap()
}

fn cases() -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(fixture("cases.json")).unwrap()).unwrap()
}

/// How many pixels differ, for a readable failure.
fn differing(a: &Image, b: &Image) -> usize {
    assert_eq!(
        (a.width, a.height, a.channels),
        (b.width, b.height, b.channels)
    );
    a.data
        .chunks(a.channels)
        .zip(b.data.chunks(b.channels))
        .filter(|(x, y)| x != y)
        .count()
}

#[test]
fn bicubic_resizes_with_a_box_match_pillow_exactly() {
    let src = load("src.png");
    for case in cases()["resize"].as_array().unwrap() {
        let size = (
            case["size"][0].as_u64().unwrap() as u32,
            case["size"][1].as_u64().unwrap() as u32,
        );
        let b: Vec<f32> = case["box"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap() as f32)
            .collect();
        let ours = pillow::resize_bicubic(&src, size, [b[0], b[1], b[2], b[3]]);
        let theirs = load(case["file"].as_str().unwrap());
        assert_eq!(differing(&ours, &theirs), 0, "{case}");
    }
}

#[test]
fn blends_match_pillow_exactly() {
    let (src, other) = (load("src.png"), load("other.png"));
    for case in cases()["blend"].as_array().unwrap() {
        let ours = pillow::blend(&src, &other, case["alpha"].as_f64().unwrap() as f32);
        assert_eq!(
            differing(&ours, &load(case["file"].as_str().unwrap())),
            0,
            "{case}"
        );
    }
}

#[test]
fn pastes_with_an_alpha_mask_match_pillow_exactly() {
    let piece = load("piece.png");
    assert_eq!(piece.channels, 4);
    for case in cases()["paste"].as_array().unwrap() {
        let mut frame = load("src.png");
        let (x, y) = (
            case["pos"][0].as_i64().unwrap() as i32,
            case["pos"][1].as_i64().unwrap() as i32,
        );
        pillow::paste_masked(&mut frame, &piece, x, y);
        assert_eq!(
            differing(&frame, &load(case["file"].as_str().unwrap())),
            0,
            "{case}"
        );
    }
}

#[test]
fn wide_lines_and_rectangles_match_pillow_exactly() {
    let mut frame = Image::new(60, 40, &[11, 11, 16]);
    pillow::line_h(&mut frame, 0, 60, 9, &[42, 42, 56], 2);
    pillow::line_h(&mut frame, 0, 60, 30, &[42, 42, 56], 2);
    pillow::rectangle(&mut frame, (0, 0, 60, 4), &[200, 100, 50]);
    assert_eq!(differing(&frame, &load("lines.png")), 0);
}

#[test]
fn rounded_rectangles_match_pillow_exactly() {
    for case in cases()["rounded"].as_array().unwrap() {
        let (w, h) = (
            case["size"][0].as_i64().unwrap() as i32,
            case["size"][1].as_i64().unwrap() as i32,
        );
        let fill: Vec<u8> = case["fill"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap() as u8)
            .collect();
        let mut img = Image::new(w as u32, h as u32, &[0, 0, 0, 0]);
        pillow::rounded_rectangle(
            &mut img,
            (0, 0, w - 1, h - 1),
            case["radius"].as_i64().unwrap() as i32,
            &fill,
            &[58, 58, 74, 255],
        );
        assert_eq!(
            differing(&img, &load(case["file"].as_str().unwrap())),
            0,
            "{case}"
        );
    }
}

#[test]
fn mask_fills_match_pillow_on_rgb_and_every_alpha() {
    let expected: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(fixture("bitmap.json")).unwrap()).unwrap();
    let mask = [0u8, 1, 64, 128, 254, 255];
    for (key, pixels) in expected.as_object().unwrap() {
        let mut img = match key.as_str() {
            "rgb" => Image::new(6, 1, &[7, 7, 10]),
            alpha => Image::new(6, 1, &[7, 7, 10, alpha.parse().unwrap()]),
        };
        pillow::draw_bitmap(&mut img, (0, 0), (6, 1), &mask, [244, 243, 239]);
        let want: Vec<u8> = pixels
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|p| {
                p.as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_u64().unwrap() as u8)
            })
            .collect();
        assert_eq!(img.data, want, "{key}");
    }
}

fn plex() -> (Fonts, Font) {
    let mut fonts = Fonts::new();
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/fonts")
        .join(cases()["text"]["font"].as_str().unwrap());
    let font = fonts.load(&path).unwrap();
    (fonts, font)
}

#[test]
fn text_widths_match_pillow_to_the_64th_of_a_pixel() {
    let (fonts, font) = plex();
    let cases = cases();
    let lengths = cases["text"]["lengths"].as_array().unwrap();
    assert!(lengths.len() > 100);
    for case in lengths {
        let size = case["size"].as_f64().unwrap() as f32;
        let text = case["text"].as_str().unwrap();
        let length26 = (f64::from(fonts.length(&font, size, text)) * 64.0).round() as i64;
        assert_eq!(
            length26,
            case["length26"].as_i64().unwrap(),
            "{text:?} at {size} px"
        );
    }
}

#[test]
fn text_lands_where_pillow_puts_it() {
    let (mut fonts, font) = plex();
    for case in cases()["text"]["drawn"].as_array().unwrap() {
        let at = |i: usize| case["xy"][i].as_f64().unwrap() as f32;
        let mut img = Image::new(1200, 120, &[11, 11, 16]);
        fonts.draw(
            &mut img,
            (at(0), at(1)),
            case["text"].as_str().unwrap(),
            Pen {
                font: &font,
                size: case["size"].as_f64().unwrap() as f32,
                ink: [244, 243, 239],
            },
            case["anchor"].as_str().unwrap(),
        );
        // swash's coverage may differ from FreeType's by a level at a glyph's
        // edge; a glyph a pixel out of place differs by far more.
        let want = load(case["file"].as_str().unwrap());
        let worst = img
            .data
            .iter()
            .zip(&want.data)
            .map(|(a, b)| a.abs_diff(*b))
            .max()
            .unwrap();
        assert!(worst <= 2, "{case}: worst difference {worst}");
    }
}
