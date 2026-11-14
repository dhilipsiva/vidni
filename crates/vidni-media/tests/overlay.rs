//! Overlays drawn from their parts (TODO.md, P5-VIDNI-05): each template,
//! in each shape, is the same picture each time, as large as it says,
//! and coloured by its parts' tones.

use vidni_core::fonts;
use vidni_core::project::Scene;
use vidni_core::theme::Theme;
use vidni_media::build::Faces;
use vidni_media::compose::cut::{Look, Type};
use vidni_media::compose::image::Image;
use vidni_media::compose::overlay::{self, NARROW, WIDE};
use vidni_media::compose::text::Fonts;

fn scene(keys: serde_json::Value) -> Scene {
    serde_json::from_value(keys).unwrap()
}

/// Draws `scene` for a wide cut and for a tall one.
fn drawn(scene: &Scene) -> [Image; 2] {
    let theme = Theme::default_theme();
    let mut fonts = Fonts::new();
    let files = Faces::pinned(&theme, &fonts::dirs()).unwrap();
    let faces = Type {
        caption: fonts.load(&files.caption).unwrap(),
        mono: fonts.load(&files.mono).unwrap(),
        display: fonts.load(&files.display).unwrap(),
    };
    let signs = fonts::get("dejavu-sans-mono-bold").unwrap();
    let signs = fonts
        .load(&fonts::find(&signs, &fonts::dirs()).unwrap())
        .unwrap();
    let look = Look::of(&theme);
    [false, true]
        .map(|portrait| overlay::render(scene, portrait, &mut fonts, &faces, &signs, &look))
}

/// Whether any pixel of `image` in the box is `ink`.
fn has(image: &Image, (x0, y0, x1, y1): (u32, u32, u32, u32), ink: [u8; 3]) -> bool {
    (y0..y1.min(image.height)).any(|y| {
        (x0..x1.min(image.width)).any(|x| {
            let p = image.pixel(x, y);
            [p[0], p[1], p[2]] == ink
        })
    })
}

const GOOD: [u8; 3] = [56, 227, 166];
const BAD: [u8; 3] = [255, 77, 94];

#[test]
fn chips_are_a_row_on_a_wide_cut_and_two_to_a_row_on_a_tall_one() {
    let chips = scene(serde_json::json!({
        "kind": "overlay",
        "template": "chips",
        "parts": [
            { "label": "TRUE", "text": "a proof exists", "sign": "\u{22a4}", "tone": "good" },
            { "label": "FALSE", "text": "not derivable", "sign": "\u{22a5}", "tone": "bad" },
            { "label": "UNKNOWN", "text": "with a reason", "sign": "?" },
        ],
    }));
    let [wide, tall] = drawn(&chips);
    // The same picture each time.
    assert_eq!(drawn(&chips), [wide.clone(), tall.clone()]);
    assert_eq!((wide.width, wide.height), (WIDE, 150));
    // Three chips are two rows of two on a tall cut.
    assert_eq!((tall.width, tall.height), (NARROW, 330));
    assert_eq!(overlay::size(&chips, false), (WIDE, 150));
    assert_eq!(overlay::size(&chips, true), (NARROW, 330));
    // Each chip's tone is in its own third of the row, and nowhere else.
    let third = WIDE / 3;
    assert!(has(&wide, (0, 0, third, 150), GOOD));
    assert!(!has(&wide, (third, 0, WIDE, 150), GOOD));
    assert!(has(&wide, (third, 0, third * 2, 150), BAD));
    assert!(!has(&wide, (0, 0, third, 150), BAD));
    // On the tall cut the second chip is at the right of the first row.
    assert!(has(&tall, (NARROW / 2, 0, NARROW, 165), BAD));
    assert!(!has(&tall, (0, 165, NARROW, 330), BAD));
    // It is drawn over the picture whole: nothing of it is see-through.
    assert_eq!(wide.channels, 4);
    assert!(wide.data.chunks(4).all(|pixel| pixel[3] == 255));
}

#[test]
fn a_flows_wall_parts_its_boxes_and_a_tone_edges_a_box() {
    let flow = scene(serde_json::json!({
        "kind": "overlay",
        "template": "flow",
        "parts": [
            { "label": "formalize", "text": "An LLM drafts" },
            { "wall": "a wall" },
            { "label": "gates", "text": "grammar and vocabulary" },
            { "label": "derive", "text": "a verdict and a proof", "tone": "good" },
        ],
    }));
    let [wide, tall] = drawn(&flow);
    assert_eq!(drawn(&flow), [wide.clone(), tall.clone()]);
    assert_eq!((wide.width, wide.height), (WIDE, 190));
    assert_eq!((tall.width, tall.height), (NARROW, 330));
    let accent = Look::of(&Theme::default_theme()).accent;
    // Wide: the wall's dashes are between the first box and the second,
    // and the toned box, the last, has its edge at the right.
    let wall: Vec<u32> = (0..WIDE)
        .filter(|x| has(&wide, (*x, 60, x + 1, 130), accent))
        .collect();
    assert!(!wall.is_empty());
    assert!(
        wall.iter().all(|x| (WIDE / 4..WIDE / 2).contains(x)),
        "{wall:?}"
    );
    assert!(has(&wide, (WIDE * 2 / 3, 0, WIDE, 40), GOOD));
    assert!(!has(&wide, (0, 0, WIDE * 2 / 3, 190), GOOD));
    // Tall: the wall is a line across, between two rows, and the toned
    // box is in the row under it.
    let across: Vec<u32> = (0..330)
        .filter(|y| has(&tall, (100, *y, 400, y + 1), accent))
        .collect();
    assert!(!across.is_empty());
    let (first, last) = (across[0], across[across.len() - 1]);
    assert!(first > 100 && last < 230, "{across:?}");
    assert!(has(&tall, (NARROW / 2, last, NARROW, 330), GOOD));
    assert!(!has(&tall, (0, 0, NARROW, first), GOOD));

    // A callout is one box of words.
    let callout = scene(serde_json::json!({
        "kind": "overlay",
        "template": "callout",
        "parts": [{ "label": "note", "text": "One thing to keep in mind.", "tone": "bad" }],
    }));
    let [wide, tall] = drawn(&callout);
    assert_eq!((wide.width, wide.height, tall.width), (WIDE, 150, NARROW));
    assert!(has(&wide, (24, 20, WIDE - 24, 24), BAD));
}
