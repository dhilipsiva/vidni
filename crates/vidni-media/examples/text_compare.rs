//! Calibration: draws the samples tools render with Pillow, for comparison.
//! cargo run -p vidni-media --example text_compare -- <out dir> <IBM Plex Sans Bold> <IBM Plex Mono Bold> <Space Grotesk Bold>

use std::path::{Path, PathBuf};
use vidni_media::compose::image::Image;
use vidni_media::compose::text::{Fonts, Pen};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [out, plex, mono, grot] = args.as_slice() else {
        panic!("give an output folder and the three font files");
    };
    let out = PathBuf::from(out);
    let mut fonts = Fonts::new();
    let plex = fonts.load(Path::new(plex)).unwrap();
    let mono = fonts.load(Path::new(mono)).unwrap();
    let grot = fonts.load(Path::new(grot)).unwrap();
    let cases = [
        (
            &plex,
            42.0,
            "Backward chaining starts from the claim, and works back.",
            (600.5, 40.0),
            "mm",
        ),
        (&mono, 26.0, "// the syllogism", (20.0, 27.0), "lm"),
        (&grot, 74.0, "Building the proof", (80.0, 90.0), "ls"),
        (
            &plex,
            42.0,
            "A neuro-symbolic system pairs a neural front end",
            (600.0, 60.0),
            "mm",
        ),
        (
            &plex,
            50.0,
            "Half-pixel start: y is 60.5",
            (100.25, 80.5),
            "ls",
        ),
    ];
    for (i, (font, size, text, xy, anchor)) in cases.into_iter().enumerate() {
        let mut img = Image::new(1200, 120, &[11, 11, 16]);
        fonts.draw(
            &mut img,
            xy,
            text,
            Pen {
                font,
                size,
                ink: [244, 243, 239],
            },
            anchor,
        );
        img.save_png(&out.join(format!("ours-{i}.png"))).unwrap();
        println!("{i} length {}", fonts.length(font, size, text));
    }
}
