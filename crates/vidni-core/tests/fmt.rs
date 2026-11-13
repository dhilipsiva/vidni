//! `vidni fmt`: one canonical layout, and every comment kept.

use std::fs;
use std::path::Path;
use vidni_core::TEMPLATE;
use vidni_core::fmt::format;

fn read(path: &str) -> String {
    fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path),
    )
    .unwrap()
}

fn comments(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|l| l.find('#').map(|i| l[i..].trim_end().to_string()))
        .filter(|c| !c.starts_with("#1") && !c.starts_with("#0") && !c.starts_with("#kb"))
        .collect()
}

#[test]
fn the_commented_template_round_trips_unchanged() {
    assert_eq!(format(TEMPLATE).unwrap(), TEMPLATE);
}

#[test]
fn the_nibli_example_is_in_the_canonical_layout() {
    let text = read("examples/nibli-syllogism/vidni.toml");
    assert!(
        format(&text).unwrap() == text,
        "run `vidni fmt examples/nibli-syllogism`"
    );
}

/// The template with its tables and keys shuffled, odd spacing, stray blank
/// lines, a wrapped array, and the same comments.
const SCRAMBLED: &str = r#"#:schema https://raw.githubusercontent.com/dhilipsiva/vidni/main/docs/vidni.schema.json
# A vidni project: one video, in one file. docs/spec.md in the vidni
# repository describes every key, and `vidni validate` checks this file.



schema=1
# What the lines show: here, a web page under the virtual camera.
[scenes.page]
url   =   "https://example.com"
kind = "web"
# The script, one narrated line at a time. A line's ID never changes: your
# takes are filed under it.
[[lines]]
show = "Hello, and welcome."
section = "intro"
id = "intro-1"


[[lines]]
  id = "intro-2"
  show = "This video was made with vidni."    # Edit the words; the ID stays.
  section = "intro"
# What the video is.
[meta]
# The scene a line shows when it names none.
default_scene = "page"
title = "My first video"

language = "en"
# A long video for YouTube.
[[cuts]]
aspect = "16:9"
id = "long"
# Pauses, in seconds.
[timing]
tail = 2.4
lead_in = 0.6
section_gap = 0.75
line_gap = 0.28
# The draft voice speaks every line until you record your own takes.
[voice]
# Spellings the draft voice says better, such as names it mispronounces.
respell = [
    {say="vid nee",text="vidni"},
]
draft = "piper:en_US-joe-medium"
# A vertical cut for YouTube Shorts, Instagram Reels and WhatsApp Status.
[[cuts]]
id = "vertical"
aspect = "9:16"
# Sections group lines. Their labels are shown on screen and become chapters.
[sections.intro]
label="Introduction"
"#;

#[test]
fn any_layout_of_the_template_formats_to_the_template() {
    assert_eq!(format(SCRAMBLED).unwrap(), TEMPLATE);
}

#[test]
fn formatting_keeps_every_comment() {
    let mut before = comments(SCRAMBLED);
    let mut after = comments(&format(SCRAMBLED).unwrap());
    before.sort();
    after.sort();
    assert_eq!(before, after);
    let example = read("examples/nibli-syllogism/vidni.toml");
    assert_eq!(comments(&example), comments(&format(&example).unwrap()));
}

#[test]
fn dotted_tables_become_headed_tables() {
    let dotted = TEMPLATE.replace(
        "# Pauses, in seconds.\n[timing]\nlead_in = 0.6\nline_gap = 0.28\nsection_gap = 0.75\ntail = 2.4\n\n",
        "",
    );
    let dotted = dotted.replace(
        "schema = 1\n",
        "schema = 1\ntiming.lead_in = 0.6\ntiming.line_gap = 0.28\ntiming.section_gap = 0.75\ntiming.tail = 2.4\n",
    );
    assert_ne!(dotted, TEMPLATE);
    assert_eq!(
        format(&dotted).unwrap(),
        TEMPLATE.replace("# Pauses, in seconds.\n", "")
    );
}

#[test]
fn formatting_twice_changes_nothing() {
    for text in [
        TEMPLATE.to_string(),
        SCRAMBLED.to_string(),
        read("examples/nibli-syllogism/vidni.toml"),
        read("crates/vidni-core/tests/fixtures/valid/minimal.toml"),
    ] {
        let once = format(&text).unwrap();
        assert_eq!(format(&once).unwrap(), once);
    }
}

#[test]
fn an_invalid_file_is_not_formatted() {
    let problems = format("schema = 1\n").unwrap_err();
    assert_eq!(problems[0].to_string(), "1:1: missing field `meta`");
}
