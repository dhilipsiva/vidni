//! Measured geometry regressions. `just test-capture` runs these with pinned fonts.
use std::{fs, path::Path};
use vidni_core::{capture, project::Project, validate};
use vidni_media::capture::{Frames, Limits, capture_with};

fn project(targets: &str) -> Project {
    validate::check(
        &format!(
            r#"schema=1
[meta]
title="Measured targets"
[sections.main]
[[lines]]
id="parcel"
section="main"
show="A capital P and Packed Parcels."
scene="parcel"
emphasis=[{{id="capital",target="capital",phrase="capital P"}}]
[[cuts]]
id="long"
aspect="16:9"
[scenes.parcel]
kind="image"
url="parcel.svg"
{targets}
"#
        ),
        None,
    )
    .unwrap()
}

fn capture(p: &Project, dir: &Path) -> Result<(Frames, Vec<u8>), String> {
    let plan = capture::for_cut(p, &p.cuts[0]);
    capture_with(
        p,
        dir,
        &plan,
        Limits {
            offline: true,
            ..Default::default()
        },
    )?;
    let frame = fs::read(capture::frames_path(dir, &plan[0].key)).unwrap();
    Ok((
        serde_json::from_slice(&frame).unwrap(),
        fs::read(capture::path(dir, &plan[0].key)).unwrap(),
    ))
}

const SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="1600" height="760">
<rect width="1600" height="760" fill="#0b0b10"/>
<text id="parcel" x="100" y="535" fill="white" font-family="IBM Plex Mono" font-size="52" font-weight="700">ParcelA</text>
<text id="list" x="650" y="535" fill="white" font-family="IBM Plex Mono" font-size="52" font-weight="700">PackedParcels</text>
<text id="footer" x="80" y="710" fill="white" font-family="IBM Plex Sans" font-size="28">A footer below the statement</text>
</svg>"##;

const TARGETS: &str = r#"[scenes.parcel.targets.capital]
element="parcel"
span={text="P"}
[scenes.parcel.targets.list]
element="list"
[scenes.parcel.targets.names]
elements=["parcel","list"]
"#;

#[test]
#[ignore = "needs Chromium and pinned fonts; just test-capture"]
fn capital_and_long_name_follow_actual_layout_and_repeat_exactly() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("parcel.svg"), SVG).unwrap();
    let p = project(TARGETS);
    let (before, bytes) = capture(&p, dir.path()).unwrap();
    let capital = before.targets["capital"][0];
    let list = before.targets["list"][0];
    assert!(capital[2] > 25.0 && capital[2] < 35.0, "{capital:?}");
    // The previous handwritten rectangle ended at 1031, clipping the name.
    assert!(list[0] + list[2] > 1055.0, "{list:?}");
    assert!(list[1] < 535.0 && list[1] + list[3] < 600.0, "{list:?}");
    assert_eq!(before.targets["names"].len(), 2);
    let (again, repeated) = capture(&p, dir.path()).unwrap();
    assert_eq!(before, again);
    assert_eq!(bytes, repeated);
    fs::write(
        dir.path().join("parcel.svg"),
        SVG.replace("PackedParcels</text>", "PackedParcelsWithLongNames</text>")
            .replace("font-size=\"52\"", "font-size=\"38\""),
    )
    .unwrap();
    let (changed, _) = capture(&p, dir.path()).unwrap();
    assert!(changed.targets["capital"][0][2] < capital[2]);
    assert!(changed.targets["list"][0][2] > list[2]);
}

#[test]
#[ignore = "needs Chromium and pinned fonts; just test-capture"]
fn ambiguous_missing_and_clipped_targets_fail_instead_of_guessing() {
    let dir = tempfile::tempdir().unwrap();
    let p = project(TARGETS);
    for (bad, reason) in [
        (SVG.replace("ParcelA</text>", "ParcelP</text>"), "ambiguous"),
        (SVG.replace("id=\"parcel\"", "id=\"other\""), "missing"),
        (SVG.replace("x=\"650\"", "x=\"1500\""), "clipped"),
        (
            SVG.replace(
                "<text id=\"parcel\"",
                "<g opacity=\"0\"><text id=\"parcel\"",
            )
            .replace("ParcelA</text>", "ParcelA</text></g>"),
            "hidden",
        ),
    ] {
        fs::write(dir.path().join("parcel.svg"), bad).unwrap();
        let error = capture(&p, dir.path()).unwrap_err();
        assert!(error.contains(reason), "{error}");
    }
}

#[test]
fn handwritten_rectangles_cannot_drive_timed_text_emphasis() {
    let invalid = r#"schema=1
[meta]
title="No rectangles"
[sections.main]
[[lines]]
id="one"
section="main"
show="A target."
scene="image"
emphasis=[{id="one",target="word",phrase="target"}]
[[cuts]]
id="long"
aspect="16:9"
[scenes.image]
kind="image"
url="image.png"
[scenes.image.targets.word]
rect=[1,2,30,40]
"#;
    assert!(
        validate::check(invalid, None)
            .unwrap_err()
            .iter()
            .any(|e| e.message.contains("measured geometry"))
    );
}

#[test]
#[ignore = "needs Chromium and pinned fonts; just test-capture"]
fn screenshot_receipts_follow_transforms_and_refuse_changed_pixels_or_crops() {
    use base64::Engine;
    use vidni_media::compose::image::Image;
    let dir = tempfile::tempdir().unwrap();
    let png = dir.path().join("screen.png");
    Image::new(200, 100, &[40, 60, 80]).save_png(&png).unwrap();
    let bytes = fs::read(&png).unwrap();
    let receipt = serde_json::json!({
        "schema":1, "image":"screen.png", "sha256":capture::sha256_hex(&bytes),
        "width":200, "height":100, "browser":"fixture measured DOM",
        "targets":{"word":[[20,20,40,10]]}
    });
    fs::write(
        dir.path().join("receipt.json"),
        serde_json::to_vec(&receipt).unwrap(),
    )
    .unwrap();
    let svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="1600" height="760">
      <svg x="100" y="100" width="400" height="200" viewBox="0 0 200 100" preserveAspectRatio="none" overflow="hidden">
        <image id="screen" width="200" height="100" preserveAspectRatio="none" href="data:image/png;base64,{}"/>
      </svg></svg>"#,
        base64::engine::general_purpose::STANDARD.encode(bytes)
    );
    fs::write(dir.path().join("parcel.svg"), &svg).unwrap();
    let p = project(
        r#"[scenes.parcel.targets.capital]
element="screen"
capture={receipt="receipt.json",target="word"}
"#,
    );
    let (frames, _) = capture(&p, dir.path()).unwrap();
    assert_eq!(frames.targets["capital"], [[140.0, 140.0, 80.0, 20.0]]);
    fs::write(
        dir.path().join("parcel.svg"),
        svg.replace("0 0 200 100", "0 0 30 100"),
    )
    .unwrap();
    assert!(capture(&p, dir.path()).unwrap_err().contains("clipped"));
    fs::write(dir.path().join("parcel.svg"), svg).unwrap();
    Image::new(200, 100, &[80, 60, 40]).save_png(&png).unwrap();
    assert!(
        capture(&p, dir.path())
            .unwrap_err()
            .contains("differs from its measured receipt")
    );
}

#[test]
#[ignore = "needs Chromium and pinned fonts; just test-capture"]
fn web_spans_wrap_and_repeated_words_require_an_occurrence() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("page.html"),
        r#"<!doctype html><style>
      body{margin:30px} #words{font:24px monospace;width:110px;line-height:32px}
      </style><div id="words">ParcelA PackedParcels ParcelA</div>"#,
    )
    .unwrap();
    let mut p = project(TARGETS);
    let scene = p.scenes.get_mut("parcel").unwrap();
    scene.kind = vidni_core::project::SceneKind::Web;
    scene.url = "page.html".into();
    scene.viewport = Some([400, 300]);
    scene.targets.clear();
    scene.targets.insert(
        "capital".into(),
        vidni_core::project::Target {
            css: Some("#words".into()),
            span: Some(vidni_core::project::TextSpan {
                text: "ParcelA PackedParcels".into(),
                occurrence: None,
            }),
            ..Default::default()
        },
    );
    let (frames, _) = capture(&p, dir.path()).unwrap();
    assert_eq!(frames.targets["capital"].len(), 2);
    assert!(frames.targets["capital"][1][1] > frames.targets["capital"][0][1]);
    p.scenes
        .get_mut("parcel")
        .unwrap()
        .targets
        .get_mut("capital")
        .unwrap()
        .span
        .as_mut()
        .unwrap()
        .text = "ParcelA".into();
    assert!(capture(&p, dir.path()).unwrap_err().contains("ambiguous"));
    p.scenes
        .get_mut("parcel")
        .unwrap()
        .targets
        .get_mut("capital")
        .unwrap()
        .span
        .as_mut()
        .unwrap()
        .occurrence = Some(2);
    let (frames, _) = capture(&p, dir.path()).unwrap();
    assert_eq!(frames.targets["capital"].len(), 1);
    assert!(frames.targets["capital"][0][1] > 85.0);
}
