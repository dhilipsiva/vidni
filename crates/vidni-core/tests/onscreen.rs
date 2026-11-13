//! Checks of the text on screen: a capture's readings against the project's expectations.

use std::path::{Path, PathBuf};
use vidni_core::onscreen::{Reading, Readings, check, load};
use vidni_core::validate;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/onscreen")
        .join(name)
}

fn project() -> vidni_core::project::Project {
    let dir = fixture("");
    let text = std::fs::read_to_string(fixture("vidni.toml")).unwrap();
    validate::check(&text, Some(&dir)).unwrap()
}

#[test]
fn a_page_that_shows_what_the_script_expects_passes() {
    // Runs of whitespace don't count, as in the reference.
    assert_eq!(
        check(&project(), &load(&fixture("matching.json")).unwrap()),
        Vec::<String>::new()
    );
}

#[test]
fn each_difference_on_screen_has_a_clear_message() {
    assert_eq!(
        check(&project(), &load(&fixture("changed.json")).unwrap()),
        [
            "after scene \"page\" loads, target \"rows\" shows \"Adam is a mammal\" as text 2 of 3, not \"Adam is an animal\"",
            "at line \"main-1\", target \"result\" shows \"UNKNOWN\", not \"FALSE\"",
            "at line \"main-1\", target \"why\" wasn't read",
        ]
    );
}

#[test]
fn a_missing_or_extra_match_is_counted() {
    let mut readings = load(&fixture("matching.json")).unwrap();
    readings.readings[1].texts.pop();
    readings.readings[0].texts.push("TRUE".into());
    assert_eq!(
        check(&project(), &readings),
        [
            "after scene \"page\" loads, target \"result\" shows 2 texts where the script expects 1 text",
            "after scene \"page\" loads, target \"rows\" shows 2 texts where the script expects 3 texts",
        ]
    );
    let none = Readings {
        readings: vec![Reading {
            scene: "page".into(),
            at: "setup".into(),
            target: "result".into(),
            texts: vec![],
        }],
    };
    assert!(check(&project(), &none)[0].ends_with("shows 0 texts where the script expects 1 text"));
}
