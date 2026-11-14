//! Each card template, in each theme and each shape, against its snapshot
//! (TODO.md, P5-VIDNI-01). The snapshots are the PNGs in
//! `tests/fixtures/cards/<theme>/`; `VIDNI_UPDATE_SNAPSHOTS=1 cargo test -p
//! vidni-media --test cards` draws them afresh. The themes' fonts are the
//! pinned ones, found in `$VIDNI_FONTS`, as the dev shell sets it.

use std::path::{Path, PathBuf};
use vidni_core::fonts;
use vidni_core::project::Template;
use vidni_core::theme::{self, Theme};
use vidni_media::build::Faces;
use vidni_media::compose::card::{self, Card};
use vidni_media::compose::cut::{LANDSCAPE, Look, PORTRAIT, Type};
use vidni_media::compose::image::Image;
use vidni_media::compose::text::Fonts;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// One card of each template, with words long enough to wrap.
fn cards() -> Vec<(&'static str, Card)> {
    let card =
        |template, title: &str, subtitle: &str, items: &[&str], quote: &str, by: &str| Card {
            template,
            title: title.into(),
            subtitle: subtitle.into(),
            items: items.iter().map(|item| item.to_string()).collect(),
            quote: quote.into(),
            by: by.into(),
        };
    vec![
        (
            "title",
            card(
                Template::Title,
                "The LLM drafts, the engine decides",
                "a five-minute tour",
                &[],
                "",
                "",
            ),
        ),
        (
            "section",
            card(
                Template::Section,
                "How a proof is traced",
                "part two",
                &[],
                "",
                "",
            ),
        ),
        (
            "bullets",
            card(
                Template::Bullets,
                "What a draft does",
                "",
                &[
                    "Speaks every line in the draft voice",
                    "Takes the captures the cuts need, and only those that are missing",
                    "Builds each cut",
                ],
                "",
                "",
            ),
        ),
        (
            "quote",
            card(
                Template::Quote,
                "",
                "",
                &[],
                "Premature optimization is the root of all evil.",
                "Donald Knuth",
            ),
        ),
        (
            "end",
            card(
                Template::End,
                "Thanks for watching",
                "github.com/dhilipsiva/vidni",
                &[],
                "",
                "",
            ),
        ),
    ]
}

fn themes() -> Vec<Theme> {
    vec![
        Theme::default_theme(),
        theme::load(&root().join("examples"), "themes/quine").unwrap(),
    ]
}

#[test]
fn every_template_matches_its_snapshot_in_both_themes_and_shapes() {
    let update = std::env::var_os("VIDNI_UPDATE_SNAPSHOTS").is_some();
    let mut differing = Vec::new();
    for theme in themes() {
        let files = Faces::pinned(&theme, &fonts::dirs()).unwrap();
        let mut fonts = Fonts::new();
        let faces = Type {
            caption: fonts.load(&files.caption).unwrap(),
            mono: fonts.load(&files.mono).unwrap(),
            display: fonts.load(&files.display).unwrap(),
        };
        let look = Look::of(&theme);
        let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cards")
            .join(&theme.name);
        for (name, card) in cards() {
            for layout in [LANDSCAPE, PORTRAIT] {
                let drawn = card::render(&card, &layout, &mut fonts, &faces, &look);
                assert_eq!((drawn.width, drawn.height), layout.size);
                // Drawn again, a card is the same card.
                assert_eq!(
                    card::render(&card, &layout, &mut fonts, &faces, &look),
                    drawn
                );
                let file = folder.join(format!("{name}-{}.png", layout.name));
                if update {
                    std::fs::create_dir_all(&folder).unwrap();
                    drawn.save_png(&file).unwrap();
                    continue;
                }
                let snapshot = Image::load_png(&file).map(|image| image.to_rgb());
                if snapshot.as_ref() != Ok(&drawn) {
                    // What was drawn is left beside the test's output, to look at.
                    let seen = std::env::temp_dir().join(format!(
                        "vidni-card-{}-{name}-{}.png",
                        theme.name, layout.name
                    ));
                    drawn.save_png(&seen).unwrap();
                    differing.push(format!("{} (drawn: {})", file.display(), seen.display()));
                }
            }
        }
    }
    assert!(
        differing.is_empty(),
        "cards differ from their snapshots:\n  {}",
        differing.join("\n  ")
    );
}

#[test]
fn a_theme_changes_a_cards_look_and_not_where_things_sit() {
    let [plain, quine] = [themes().remove(0), themes().remove(1)];
    // The two themes share their text face, so a card that is all running
    // text differs only in colour: the same pixels are drawn on.
    let draw = |theme: &Theme| {
        let files = Faces::pinned(theme, &fonts::dirs()).unwrap();
        let mut fonts = Fonts::new();
        let faces = Type {
            caption: fonts.load(&files.caption).unwrap(),
            mono: fonts.load(&files.mono).unwrap(),
            display: fonts.load(&files.caption).unwrap(),
        };
        let (_, bullets) = cards().remove(2);
        let look = Look::of(theme);
        (
            card::render(&bullets, &LANDSCAPE, &mut fonts, &faces, &look),
            look,
        )
    };
    let ((a, look_a), (b, look_b)) = (draw(&plain), draw(&quine));
    assert_ne!(a, b);
    let inked = |image: &Image, frame: [u8; 3]| -> Vec<bool> {
        image.data.chunks(3).map(|pixel| pixel != frame).collect()
    };
    assert_eq!(inked(&a, look_a.frame), inked(&b, look_b.frame));
}
