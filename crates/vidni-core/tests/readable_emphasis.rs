use vidni_core::validate;

const BASE: &str = r#"schema=1
[meta]
title="Readable emphasis"
[sections.main]
[[lines]]
id="one"
section="main"
show="A name."
scene="names"
[[cuts]]
id="long"
aspect="16:9"
[scenes.names]
kind="image"
url="names.svg"
[scenes.names.targets.name]
element="name"
continuity="parcel-name"
"#;

#[test]
fn default_three_seconds_can_be_disabled_explicitly() {
    assert_eq!(
        validate::check(BASE, None).unwrap().emphasis.min_duration,
        3.0
    );
    let disabled = format!("{BASE}\n[emphasis]\nmin_duration=0\n");
    assert_eq!(
        validate::check(&disabled, None)
            .unwrap()
            .emphasis
            .min_duration,
        0.0
    );
    for value in ["-1", "nan", "inf"] {
        assert!(
            validate::check(&format!("{BASE}\n[emphasis]\nmin_duration={value}\n"), None).is_err()
        );
    }
}

#[test]
fn continuity_is_explicit_and_unambiguous_within_a_scene() {
    let duplicate = format!(
        "{BASE}\n[scenes.names.targets.other]\nelement=\"other\"\ncontinuity=\"parcel-name\"\n"
    );
    assert!(
        format!("{:?}", validate::check(&duplicate, None).unwrap_err())
            .contains("unique within the scene")
    );
    assert!(validate::check(&BASE.replace("parcel-name", "not a valid ID"), None).is_err());
}
