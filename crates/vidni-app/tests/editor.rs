//! The script editor (TODO.md, P7-VIDNI-02): edits made in the app and
//! edits made by hand, in turn, to one file. Each changes what it names,
//! and every comment that was ever in the file is still in it at the end.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use std::fs;
use std::path::{Path, PathBuf};
use vidni_app::app::App;

/// The comments of a TOML text: from each `#` outside a string to the
/// end of its line.
fn comments(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    for line in text.lines() {
        let mut quoted = false;
        for (at, c) in line.char_indices() {
            match c {
                '"' => quoted = !quoted,
                '#' if !quoted => {
                    found.push(line[at..].trim_end().to_string());
                    break;
                }
                _ => {}
            }
        }
    }
    found
}

/// Every comment of `before` is in `after`, as many times as it was. A
/// line that moves takes its comments along, so their order may change.
fn kept(before: &[String], after: &str, when: &str) {
    let mut rest = comments(after);
    for comment in before {
        match rest.iter().position(|c| c == comment) {
            Some(at) => {
                rest.remove(at);
            }
            None => panic!("{when}: the comment {comment:?} is gone from:\n{after}"),
        }
    }
}

struct Session<'a> {
    harness: Harness<'a, App>,
    file: PathBuf,
    /// Every comment the file has ever had, in order.
    comments: Vec<String>,
}

impl Session<'_> {
    fn new(dir: &Path) -> Self {
        assert_eq!(
            vidni_cli::run(&["new", "demo", "--in", dir.to_str().unwrap()]),
            0
        );
        let folder = dir.join("demo");
        let mut app = App::new(None);
        app.open(&folder);
        let mut harness = Harness::builder()
            .with_size([1100.0, 2400.0])
            .build_ui_state(|ui, app: &mut App| app.show(ui), app);
        harness.run();
        harness.get_by_label("Script").click();
        harness.run();
        let file = folder.join("vidni.toml");
        let comments = comments(&fs::read_to_string(&file).unwrap());
        assert!(
            comments.len() > 10,
            "the starter has comments to keep: {comments:?}"
        );
        Self {
            harness,
            file,
            comments,
        }
    }

    fn text(&self) -> String {
        fs::read_to_string(&self.file).unwrap()
    }

    fn click(&mut self, label: &str) {
        self.harness.get_by_label(label).click();
        self.harness.run();
    }

    fn fill(&mut self, label: &str, text: &str) {
        self.harness.get_by_label(label).focus();
        self.harness.run();
        self.harness
            .key_combination_modifiers(egui::Modifiers::COMMAND, &[egui::Key::A]);
        self.harness.run();
        match text.is_empty() {
            true => self.harness.key_press(egui::Key::Backspace),
            false => self.harness.get_by_label(label).type_text(text),
        }
        self.harness.run();
    }

    /// Checks the file after an edit made in the app: every comment so
    /// far is kept. Returns the file's text.
    fn after_the_app(&mut self, when: &str) -> String {
        let text = self.text();
        kept(&self.comments, &text, when);
        self.comments = comments(&text);
        text
    }

    /// Edits the file by hand, as another editor would, and notes the
    /// comments it has now.
    fn by_hand(&mut self, edit: impl FnOnce(String) -> String) {
        let text = edit(self.text());
        fs::write(&self.file, &text).unwrap();
        kept(&self.comments, &text, "a hand edit of this test");
        self.comments = comments(&text);
    }
}

#[test]
fn app_edits_and_hand_edits_in_turn_keep_every_comment() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Session::new(dir.path());
    let first = s.text();
    // Every line is listed with its ID.
    assert!(
        s.harness
            .query_by_label("[intro-1]  Hello, and welcome.")
            .is_some()
    );
    assert!(
        s.harness
            .query_by_label("[intro-2]  This video was made with vidni.")
            .is_some()
    );

    // In the app: other words for a line. Only that value changes.
    s.click("[intro-1]  Hello, and welcome.");
    s.fill("Shown", "Hello, everyone.");
    s.click("Apply to intro-1");
    let text = s.after_the_app("a line's words changed in the app");
    assert_eq!(
        text,
        first.replace(
            "show = \"Hello, and welcome.\"",
            "show = \"Hello, everyone.\""
        )
    );

    // By hand: a comment above the second line, and other words for it.
    s.by_hand(|text| {
        text.replace(
            "[[lines]]\nid = \"intro-2\"",
            "# a note by hand\n[[lines]]\nid = \"intro-2\"",
        )
        .replace(
            "This video was made with vidni.",
            "This video was made by hand, in part.",
        )
    });

    // In the app, which hasn't been told: a new line after the first.
    s.fill("New line's ID", "intro-1a");
    s.fill("Its words", "A line from the app.");
    s.click("Add after intro-1");
    let text = s.after_the_app("a line added in the app");
    assert!(
        text.contains("# a note by hand\n[[lines]]\nid = \"intro-2\""),
        "{text}"
    );
    assert!(
        text.contains("This video was made by hand, in part."),
        "{text}"
    );
    let at = |needle: &str| {
        text.find(needle)
            .unwrap_or_else(|| panic!("{needle} isn't in:\n{text}"))
    };
    assert!(
        at("id = \"intro-1\"") < at("id = \"intro-1a\"")
            && at("id = \"intro-1a\"") < at("# a note by hand")
    );
    // The app shows what was done by hand, too.
    assert!(
        s.harness
            .query_by_label("[intro-2]  This video was made by hand, in part.")
            .is_some()
    );

    // By hand: a comment after the new line's words, and one at the end.
    s.by_hand(|text| {
        text.replace(
            "show = \"A line from the app.\"",
            "show = \"A line from the app.\" # kept beside its value",
        ) + "\n# the end, by hand\n"
    });

    // In the app: the new line moves down past the second, with its comment.
    s.click("Move down");
    let text = s.after_the_app("a line moved in the app");
    let at = |needle: &str| text.find(needle).unwrap();
    assert!(at("id = \"intro-2\"") < at("id = \"intro-1a\""), "{text}");
    assert!(
        text.contains("show = \"A line from the app.\" # kept beside its value"),
        "{text}"
    );
    // The line it passed still has the comment that was written above it.
    assert!(
        text.contains("# a note by hand\n[[lines]]\nid = \"intro-2\""),
        "{text}"
    );
    // And its words change, with the comment beside them kept.
    s.fill("Shown", "A line that moved.");
    s.click("Apply to intro-1a");
    let text = s.after_the_app("a moved line's words changed in the app");
    assert!(
        text.contains("show = \"A line that moved.\" # kept beside its value"),
        "{text}"
    );

    // In the app: the title, and a scene's page.
    s.fill("Title", "  Edited in turns ");
    s.click("Apply title");
    let text = s.after_the_app("the title changed in the app");
    assert!(text.contains("title = \"Edited in turns\""), "{text}");
    fs::copy(
        s.file.with_file_name("page.html"),
        s.file.with_file_name("other.html"),
    )
    .unwrap();
    s.click("[page]  web");
    s.fill("Address or file", "other.html");
    s.click("Apply to scene page");
    let text = s.after_the_app("a scene's page changed in the app");
    assert!(text.contains("url = \"other.html\""), "{text}");

    // An edit that would leave the file invalid isn't saved, and says why
    // at its place in the file.
    s.fill("Shown", "");
    s.click("Apply to intro-1a");
    assert!(s.harness.query_by_label("Not saved:").is_some());
    assert!(
        s.harness
            .query_by_label_contains("the line shows nothing")
            .is_some()
    );
    assert_eq!(s.text(), text);
    s.fill("Address or file", "nowhere.html");
    s.click("Apply to scene page");
    assert!(
        s.harness
            .query_by_label_contains("nowhere.html is not in the project folder")
            .is_some()
    );
    assert_eq!(s.text(), text);

    // The source view: the file's own text, saved when it is valid.
    s.click("Source");
    let edited = text.replace(
        "# the end, by hand",
        "# the end, by hand\n# and a line from the source view",
    );
    s.fill("The project's file", &edited);
    s.click("Save source");
    let text = s.after_the_app("the source view saved");
    assert_eq!(text, edited);
    let broken = text.replace("schema = 1", "schema = 7");
    s.fill("The project's file", &broken);
    s.click("Save source");
    assert!(s.harness.query_by_label("Not saved:").is_some());
    assert_eq!(s.text(), text);
    s.click("Read from disk");
    assert!(s.harness.query_by_label("Not saved:").is_none());

    // Back in the forms: a line with a comment of its own isn't removed
    // from here, since the comment would go with it.
    s.click("Forms");
    s.click("[intro-1a]  A line that moved.");
    s.click("Remove intro-1a");
    assert!(
        s.harness
            .query_by_label_contains("has comments of its own in the file")
            .is_some()
    );
    assert_eq!(s.text(), text);
    // With the comment taken out by hand, it goes, and nothing else does.
    let without = s.text().replace(" # kept beside its value", "");
    fs::write(&s.file, &without).unwrap();
    s.comments = comments(&without);
    let before = s.text();
    s.click("Remove intro-1a");
    let text = s.after_the_app("a line removed in the app");
    assert!(!text.contains("intro-1a"), "{text}");
    assert_eq!(comments(&text).len(), comments(&before).len());
    // What is left is a valid project, in the layout `vidni fmt` keeps.
    assert_eq!(vidni_cli::run(&["validate", s.file.to_str().unwrap()]), 0);
}

#[test]
fn what_the_app_cant_do_without_losing_something_it_refuses_and_says() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Session::new(dir.path());

    // A key with a comment isn't cleared from a form; one without is.
    s.by_hand(|text| {
        text.replace(
            "show = \"Hello, and welcome.\"\n",
            "show = \"Hello, and welcome.\"\n# why it is said this way\nsay = \"Hullo.\" # beside say\nsource = \"a book\"\n",
        )
    });
    s.click("[intro-1]  Hello, and welcome.");
    let text = s.text();
    s.fill("Said, when it differs", "");
    s.click("Apply to intro-1");
    assert!(
        s.harness
            .query_by_label_contains("say has a comment, which would go with it")
            .is_some()
    );
    assert_eq!(s.text(), text);
    s.fill("Said, when it differs", "Hullo.");
    s.fill("Source of its facts", "");
    s.click("Apply to intro-1");
    let text = s.after_the_app("a key with no comment cleared in the app");
    assert!(!text.contains("source = "), "{text}");
    assert!(
        text.contains("# why it is said this way\nsay = \"Hullo.\" # beside say\n"),
        "{text}"
    );

    // Typing that isn't applied yet survives a hand edit elsewhere, and
    // is then applied to the file as the hand edit left it.
    s.fill("Shown", "Hello again.");
    s.fill("Title", "A title being typed");
    s.by_hand(|text| text + "# written by hand while the app held typing\n");
    s.harness.run();
    s.click("Apply to intro-1");
    let text = s.after_the_app("typing applied after a hand edit");
    assert!(text.contains("show = \"Hello again.\""), "{text}");
    assert!(
        text.ends_with("# written by hand while the app held typing\n"),
        "{text}"
    );
    s.click("Apply title");
    let text = s.after_the_app("a title applied after a hand edit");
    assert!(text.contains("title = \"A title being typed\""), "{text}");

    // A new line is in the section of the line it follows, whatever is
    // typed in that line's form.
    s.fill("Section", "nowhere");
    s.fill("New line's ID", "intro-1b");
    s.fill("Its words", "In its neighbour's section.");
    s.click("Add after intro-1");
    let text = s.after_the_app("a line added beside typing");
    assert!(
        text.contains(
            "id = \"intro-1b\"\nsection = \"intro\"\nshow = \"In its neighbour's section.\""
        ),
        "{text}"
    );

    // A line renamed by hand: its form closes, and the app says so.
    s.click("[intro-2]  This video was made with vidni.");
    s.by_hand(|text| text.replace("id = \"intro-2\"", "id = \"intro-9\""));
    s.harness.run();
    assert!(
        s.harness
            .query_by_label("Line intro-2 is no longer in the file, so its form is closed.")
            .is_some()
    );
    assert!(
        s.harness
            .query_by_label_contains("Apply to intro-2")
            .is_none()
    );

    // The source view: typing of an older file isn't saved over a newer one.
    s.click("Source");
    let typed = s
        .text()
        .replace("Hello again.", "Typed in the source view.");
    s.fill("The project's file", &typed);
    s.by_hand(|text| text + "# and another, while the source view held typing\n");
    s.harness.run();
    let text = s.text();
    s.click("Save source");
    assert!(
        s.harness
            .query_by_label_contains("the file changed on disk after this text was read")
            .is_some()
    );
    assert_eq!(s.text(), text);
    s.click("Read from disk");
    s.click("Save source");
    assert!(s.harness.query_by_label("Not saved:").is_none());
    assert_eq!(s.text(), text);

    // A title that is written on one line can't be set from the form.
    s.by_hand(|text| {
        let start = text.find("# What the video is.").unwrap();
        let end = text.find("# The draft voice").unwrap();
        format!(
            "{}meta = {{ title = \"On one line\", default_scene = \"page\" }}\n\n{}",
            text[..start].replace("# What the video is.", ""),
            &text[end..]
        ) + "# What the video is.\n# The scene a line shows when it names none.\n"
    });
    s.click("Forms");
    let text = s.text();
    s.fill("Title", "From the form");
    s.click("Apply title");
    assert!(
        s.harness
            .query_by_label_contains("meta is written on one line in the file")
            .is_some()
    );
    assert_eq!(s.text(), text);
}

#[test]
fn a_file_with_windows_line_endings_keeps_them() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Session::new(dir.path());
    s.by_hand(|text| text.replace('\n', "\r\n"));
    let lines = s.text().matches("\r\n").count();
    s.harness.run();
    s.click("[intro-1]  Hello, and welcome.");
    s.fill("Shown", "Hello, Windows.");
    s.click("Apply to intro-1");
    let text = s.after_the_app("a line changed in a file with CRLF endings");
    assert!(text.contains("show = \"Hello, Windows.\""), "{text}");
    assert_eq!(text.matches("\r\n").count(), lines);
    assert_eq!(text.matches('\n').count(), lines);
    // Moving the first line carries the comments written above it.
    s.click("Move down");
    let text = s.after_the_app("the first line moved down");
    assert_eq!(text.matches("\r\n").count(), lines);
    let at = |needle: &str| text.find(needle).unwrap();
    assert!(at("id = \"intro-2\"") < at("id = \"intro-1\""), "{text}");
}
