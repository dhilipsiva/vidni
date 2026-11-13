//! Edits through `ProjectFile` touch only what they name: every other byte,
//! and every comment, stays where it was.

use proptest::prelude::*;
use vidni_core::edit::{EditError, LineField, ProjectFile};
use vidni_core::project::Project;

const IDS: [&str; 6] = ["a-1", "a-2", "a-3", "b-1", "b-2", "b-3"];

/// A valid project with a comment on nearly everything.
fn doc() -> String {
    let mut s = String::from(
        "# header comment\nschema = 1\n\n# meta comment\n[meta]\ntitle = \"Edits\" # title comment\n\
         default_scene = \"page\"\n\n# cut comment\n[[cuts]]\nid = \"long\"\naspect = \"16:9\"\n\n\
         [sections.a]\nlabel = \"A\" # label a\n\n# section b comment\n[sections.b]\nlabel = \"B\"\n",
    );
    for id in IDS {
        let section = &id[..1];
        s += &format!(
            "\n# header {id}\n[[lines]]\n# before id {id}\nid = \"{id}\"\nsection = \"{section}\"\nshow = \"Show {id}\" # show {id}\n"
        );
    }
    s + "\n# scene comment\n[scenes.page]\nkind = \"web\" # kind comment\n# url comment\nurl = \"https://example.com\" # url trailing\n"
}

fn block(id: &str) -> String {
    let section = &id[..1];
    format!(
        "\n# header {id}\n[[lines]]\n# before id {id}\nid = \"{id}\"\nsection = \"{section}\"\nshow = \"Show {id}\" # show {id}\n"
    )
}

fn open() -> ProjectFile {
    ProjectFile::parse(&doc()).unwrap()
}

#[test]
fn changing_a_value_changes_only_that_value() {
    let mut f = open();
    f.set_line("a-2", LineField::Show, Some("New words"))
        .unwrap();
    assert_eq!(
        f.text(),
        doc().replace(
            "show = \"Show a-2\" # show a-2",
            "show = \"New words\" # show a-2"
        )
    );
}

#[test]
fn a_new_key_goes_where_fmt_would_put_it_and_leaves_the_rest() {
    let mut f = open();
    f.set_line("a-2", LineField::Say, Some("Spoken")).unwrap();
    f.set_line("a-2", LineField::Camera, Some("viewport"))
        .unwrap();
    let expected = doc().replace(
        "show = \"Show a-2\" # show a-2\n",
        "show = \"Show a-2\" # show a-2\nsay = \"Spoken\"\ncamera = \"viewport\"\n",
    );
    assert_eq!(f.text(), expected);
    f.set_line("a-2", LineField::Say, None).unwrap();
    f.set_line("a-2", LineField::Camera, None).unwrap();
    assert_eq!(f.text(), doc());
}

#[test]
fn inserting_a_line_adds_only_its_table() {
    let mut f = open();
    f.insert_line(Some("a-1"), "a-new", "a", "New").unwrap();
    let expected = doc().replace(
        "show = \"Show a-1\" # show a-1\n",
        "show = \"Show a-1\" # show a-1\n\n[[lines]]\nid = \"a-new\"\nsection = \"a\"\nshow = \"New\"\n",
    );
    assert_eq!(f.text(), expected);
    assert_eq!(
        f.line_ids(),
        ["a-1", "a-new", "a-2", "a-3", "b-1", "b-2", "b-3"]
    );
    f.insert_line(None, "a-0", "a", "First").unwrap();
    assert_eq!(f.line_ids()[0], "a-0");
    assert!(vidni_core::validate::check(&f.text(), None).is_ok());
}

#[test]
fn removing_a_line_removes_its_table_and_comments() {
    let mut f = open();
    f.remove_line("a-2").unwrap();
    assert_eq!(f.text(), doc().replace(&block("a-2"), ""));
}

#[test]
fn moving_a_line_takes_its_comments_along() {
    let mut f = open();
    f.move_line("a-3", None).unwrap();
    let without = doc().replace(&block("a-3"), "");
    let expected = without.replacen(
        &block("a-1"),
        &format!("{}{}", block("a-3"), block("a-1")),
        1,
    );
    assert_eq!(f.text(), expected);
    f.move_line("a-3", Some("a-2")).unwrap();
    assert_eq!(f.text(), doc());
}

#[test]
fn a_scene_value_keeps_its_comments() {
    let mut f = open();
    f.set_scene_value("page", "url", "https://example.org")
        .unwrap();
    assert_eq!(
        f.text(),
        doc().replace("https://example.com", "https://example.org")
    );
    f.set_title("Renamed");
    assert_eq!(
        f.text(),
        doc()
            .replace("https://example.com", "https://example.org")
            .replace("\"Edits\"", "\"Renamed\"")
    );
}

#[test]
fn refused_edits_change_nothing() {
    let mut f = open();
    assert_eq!(
        f.set_line("z-9", LineField::Show, Some("x")),
        Err(EditError::UnknownLine("z-9".into()))
    );
    assert_eq!(
        f.insert_line(None, "a-1", "a", "x"),
        Err(EditError::LineExists("a-1".into()))
    );
    assert_eq!(
        f.set_line("a-1", LineField::Show, None),
        Err(EditError::Required("show"))
    );
    assert_eq!(
        f.set_scene_value("nope", "url", "x"),
        Err(EditError::UnknownScene("nope".into()))
    );
    assert_eq!(
        f.move_line("a-1", Some("z-9")),
        Err(EditError::UnknownLine("z-9".into()))
    );
    assert_eq!(f.text(), doc());
}

#[test]
fn an_edit_to_the_nibli_example_changes_one_line_of_the_file() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/nibli-syllogism/vidni.toml");
    let before = std::fs::read_to_string(path).unwrap();
    let mut f = ProjectFile::parse(&before).unwrap();
    f.set_line(
        "false-1",
        LineField::Show,
        Some("Now the claim that Adam is a cat."),
    )
    .unwrap();
    let after = f.text();
    let changed: Vec<(&str, &str)> = before
        .lines()
        .zip(after.lines())
        .filter(|(a, b)| a != b)
        .collect();
    assert_eq!(before.lines().count(), after.lines().count());
    assert_eq!(
        changed,
        [(
            "show = \"Now the claim that Adam is a bird.\"",
            "show = \"Now the claim that Adam is a cat.\""
        )]
    );
}

#[derive(Debug, Clone)]
enum Edit {
    Show(usize, String),
    Say(usize, Option<String>),
    Insert(Option<usize>),
    Remove(usize),
    Move(usize, Option<usize>),
    SceneUrl(String),
    Title(String),
}

fn edit() -> impl Strategy<Value = Edit> {
    let text = "[A-Za-z ]{1,12}";
    prop_oneof![
        (any::<usize>(), text).prop_map(|(i, t)| Edit::Show(i, t)),
        (any::<usize>(), proptest::option::of(text)).prop_map(|(i, t)| Edit::Say(i, t)),
        proptest::option::of(any::<usize>()).prop_map(Edit::Insert),
        any::<usize>().prop_map(Edit::Remove),
        (any::<usize>(), proptest::option::of(any::<usize>())).prop_map(|(i, a)| Edit::Move(i, a)),
        "[a-z]{1,8}".prop_map(|h| Edit::SceneUrl(format!("https://{h}.example"))),
        text.prop_map(Edit::Title),
    ]
}

#[derive(Debug, Clone)]
struct ModelLine {
    id: String,
    show: String,
    say: Option<String>,
    original: bool,
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    #[test]
    fn random_edit_sequences_keep_every_untouched_comment(edits in prop::collection::vec(edit(), 0..30)) {
        let mut f = open();
        let mut model: Vec<ModelLine> = IDS
            .iter()
            .map(|id| ModelLine { id: id.to_string(), show: format!("Show {id}"), say: None, original: true })
            .collect();
        let mut fresh = 0;
        for e in edits {
            match e {
                Edit::Show(i, t) => {
                    let i = i % model.len();
                    f.set_line(&model[i].id, LineField::Show, Some(&t)).unwrap();
                    model[i].show = t;
                }
                Edit::Say(i, t) => {
                    let i = i % model.len();
                    f.set_line(&model[i].id, LineField::Say, t.as_deref()).unwrap();
                    model[i].say = t;
                }
                Edit::Insert(after) => {
                    fresh += 1;
                    let id = format!("n-{fresh}");
                    let after = after.map(|a| a % model.len());
                    let after_id = after.map(|a| model[a].id.clone());
                    f.insert_line(after_id.as_deref(), &id, "a", "New").unwrap();
                    model.insert(after.map_or(0, |a| a + 1), ModelLine { id, show: "New".into(), say: None, original: false });
                }
                Edit::Remove(i) => {
                    if model.len() > 1 {
                        let i = i % model.len();
                        f.remove_line(&model[i].id).unwrap();
                        model.remove(i);
                    }
                }
                Edit::Move(i, after) => {
                    let i = i % model.len();
                    let id = model[i].id.clone();
                    let after_id = after.map(|a| model[a % model.len()].id.clone());
                    f.move_line(&id, after_id.as_deref()).unwrap();
                    if after_id.as_deref() != Some(id.as_str()) {
                        let line = model.remove(i);
                        let at = after_id.map_or(0, |a| model.iter().position(|l| l.id == a).unwrap() + 1);
                        model.insert(at, line);
                    }
                }
                Edit::SceneUrl(u) => f.set_scene_value("page", "url", u).unwrap(),
                Edit::Title(t) => f.set_title(&t),
            }
        }
        let text = f.text();
        let ids: Vec<String> = model.iter().map(|l| l.id.clone()).collect();
        prop_assert_eq!(f.line_ids(), ids);
        // Every comment of a line that is still there stays attached to it.
        for line in model.iter().filter(|l| l.original) {
            let id = &line.id;
            let attached = format!("# header {id}\n[[lines]]\n# before id {id}\nid = \"{id}\"\n");
            prop_assert_eq!(text.matches(&attached).count(), 1, "{}", id);
            let show_comment = format!(" # show {id}\n");
            prop_assert_eq!(text.matches(&show_comment).count(), 1, "{}", id);
        }
        // The comments of removed lines go with them.
        for id in IDS.iter().filter(|id| !model.iter().any(|l| l.id == **id)) {
            let header = format!("# header {id}\n");
            prop_assert_eq!(text.matches(&header).count(), 0);
        }
        // Every comment outside the lines stays, once.
        for c in ["# header comment\n", "# meta comment\n", " # title comment\n", "# cut comment\n", " # label a\n",
                  "# section b comment\n", "# scene comment\n", " # kind comment\n", "# url comment\n", " # url trailing\n"] {
            prop_assert_eq!(text.matches(c).count(), 1, "{:?}", c);
        }
        // And the values are the model's.
        let project: Project = toml::from_str(&text).unwrap();
        for (line, want) in project.lines.iter().zip(&model) {
            prop_assert_eq!(&line.show, &want.show);
            prop_assert_eq!(&line.say, &want.say);
        }
    }
}
