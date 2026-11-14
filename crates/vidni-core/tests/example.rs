//! The nibli example validates, and it is a faithful conversion of the
//! reference pipeline's script (reference/segments.json).

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use vidni_core::project::{Project, SceneKind};
use vidni_core::validate::check;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn example() -> Project {
    let dir = root().join("examples/nibli-syllogism");
    let text = fs::read_to_string(dir.join("vidni.toml")).unwrap();
    check(&text, Some(&dir)).unwrap_or_else(|problems| panic!("{problems:?}"))
}

fn reference() -> Value {
    serde_json::from_str(&fs::read_to_string(root().join("reference/segments.json")).unwrap())
        .unwrap()
}

#[test]
fn the_example_validates() {
    let p = example();
    assert_eq!(
        (
            p.lines.len(),
            p.sections.len(),
            p.scenes.len(),
            p.cuts.len()
        ),
        (59, 10, 7, 3)
    );
}

#[test]
fn the_short_shares_lines_with_the_long_cut_and_adds_its_own() {
    let p = example();
    let cut = |id: &str| p.cuts.iter().find(|c| c.id == id).unwrap();
    let (long, short) = (cut("long").line_ids(&p), cut("short").lines_in(&p));
    assert_eq!(long.len(), 57);
    assert_eq!(long, cut("vertical").line_ids(&p));
    let own: Vec<&str> = short
        .iter()
        .filter(|l| !l.only.is_empty())
        .map(|l| l.id.as_str())
        .collect();
    assert_eq!(own, ["short-hook", "short-end"]);
    assert!(own.iter().all(|id| !long.iter().any(|l| l == id)));
    let shared = short.iter().filter(|l| long.contains(&l.id)).count();
    assert_eq!((short.len(), shared), (12, 10));
}

#[test]
fn the_example_keeps_the_reference_script_line_for_line() {
    let p = example();
    let spec = reference();
    // The reference has no shorts: the lines every cut may use are its script.
    let mut lines = p.lines.iter().filter(|l| l.only.is_empty());
    for segment in spec["segments"].as_array().unwrap() {
        let id = segment["id"].as_str().unwrap();
        assert_eq!(p.sections[id].label, segment["label"].as_str().unwrap());
        for (n, sentence) in segment["sentences"].as_array().unwrap().iter().enumerate() {
            let line = lines
                .next()
                .expect("the example has fewer lines than the reference");
            let field = |key: &str| sentence.get(key).and_then(Value::as_str);
            assert_eq!(line.id, format!("{id}-{}", n + 1));
            assert_eq!(line.section, id);
            assert_eq!(Some(line.show.as_str()), field("show"));
            assert_eq!(line.say.as_deref(), field("say"));
            assert_eq!(line.scene.as_deref(), field("card"), "{}", line.id);
            assert_eq!(line.overlay.as_deref(), field("overlay"), "{}", line.id);
            assert_eq!(
                line.camera_vertical.as_deref(),
                field("vcam"),
                "{}",
                line.id
            );
            let camera = field("cam").filter(|c| *c != "app");
            assert_eq!(line.camera.as_deref(), camera, "{}", line.id);
            let highlights: Vec<String> = sentence
                .get("hl")
                .and_then(Value::as_array)
                .map(|keys| {
                    keys.iter()
                        .map(|k| k.as_str().unwrap().replace(':', "-"))
                        .collect()
                })
                .unwrap_or_default();
            assert_eq!(line.highlight, highlights, "{}", line.id);
            let actions = sentence
                .get("do")
                .and_then(Value::as_array)
                .map_or(0, Vec::len);
            assert_eq!(line.actions.len(), actions, "{}", line.id);
        }
    }
    assert!(
        lines.next().is_none(),
        "the example has more lines than the reference"
    );
}

#[test]
fn the_example_keeps_the_reference_timing_and_voice() {
    let p = example();
    let voice = &reference()["voice"];
    let number = |key: &str| voice[key].as_f64().unwrap();
    assert_eq!(p.timing.lead_in, number("lead_in"));
    assert_eq!(p.timing.line_gap, number("sentence_gap"));
    assert_eq!(p.timing.section_gap, number("segment_gap"));
    assert_eq!(p.timing.tail, number("tail"));
    assert_eq!(p.voice.length_scale, number("length_scale"));
    assert_eq!(
        p.voice.draft,
        format!("piper:{}", voice["model"].as_str().unwrap())
    );
    let respell: Vec<(String, String)> = p
        .voice
        .respell
        .iter()
        .map(|r| (r.text.clone(), r.say.clone()))
        .collect();
    let reference: Vec<(String, String)> = reference()["respell"]
        .as_array()
        .unwrap()
        .iter()
        .map(|pair| {
            (
                pair[0].as_str().unwrap().into(),
                pair[1].as_str().unwrap().into(),
            )
        })
        .collect();
    assert_eq!(respell, reference);
    assert_eq!(p.scenes["playground"].kind, SceneKind::Web);
}

#[test]
fn the_committed_json_schema_is_current() {
    let committed = fs::read_to_string(root().join("docs/vidni.schema.json")).unwrap_or_default();
    assert!(
        committed == vidni_core::schema_json(),
        "docs/vidni.schema.json is stale; run `just schema`"
    );
}
