//! The app, driven as a person drives it: by the names of its controls,
//! through AccessKit (TODO.md, P7-VIDNI-01). The harness also checks that
//! every control has a name for a screen reader to say.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use std::path::Path;
use vidni_app::actions;
use vidni_app::app::App;

fn harness<'a>(settings: &Path) -> Harness<'a, App> {
    let app = App::new(Some(settings.to_path_buf()));
    let mut harness = Harness::builder()
        .with_size([1000.0, 900.0])
        .build_ui_state(|ui, app: &mut App| app.show(ui), app);
    harness.run();
    harness
}

/// Types `text` into the field called `label`.
fn fill(harness: &mut Harness<'_, App>, label: &str, text: &str) {
    let field = harness.get_by_label(label);
    field.focus();
    harness.run();
    harness.key_combination_modifiers(egui::Modifiers::COMMAND, &[egui::Key::A]);
    harness.get_by_label(label).type_text(text);
    harness.run();
}

#[test]
fn the_help_lists_the_command_line_for_every_action() {
    let dir = tempfile::tempdir().unwrap();
    let mut harness = harness(dir.path());
    assert!(harness.query_by_label("On the command line").is_none());
    harness.get_by_label("Help").click();
    harness.run();
    for usage in actions::ALL {
        // A name may also be a heading or a button of the app itself.
        assert!(
            harness.query_all_by_label(usage.name).count() >= 1,
            "{}",
            usage.name
        );
        assert_eq!(
            harness.query_all_by_label(usage.command).count(),
            1,
            "{}",
            usage.command
        );
    }
    harness.get_by_label("Hide help").click();
    harness.run();
    assert!(harness.query_by_label("On the command line").is_none());
}

#[test]
fn a_project_is_created_and_opened_from_the_app() {
    let dir = tempfile::tempdir().unwrap();
    let (settings, videos) = (dir.path().join("settings"), dir.path().join("videos"));
    std::fs::create_dir(&videos).unwrap();
    let mut harness = harness(&settings);
    assert!(harness.query_by_label("None yet.").is_some());

    // A name that isn't one is refused in the command's own words.
    fill(&mut harness, "Name", "My Talk");
    fill(&mut harness, "In folder", videos.to_str().unwrap());
    harness.get_by_label("Create").click();
    harness.run();
    assert!(
        harness
            .query_by_label_contains("a project's name is lowercase letters and digits")
            .is_some()
    );
    assert!(!videos.join("My Talk").exists());

    // A good one is made, and opened: its title, and each cut with where it stands.
    fill(&mut harness, "Name", "my-talk");
    harness.get_by_label("Create").click();
    harness.run();
    let project = videos.join("my-talk");
    assert!(project.join("vidni.toml").is_file());
    assert!(harness.query_by_label("My talk").is_some());
    assert!(harness.query_by_label(project.to_str().unwrap()).is_some());
    for cut in ["long", "vertical"] {
        assert!(harness.query_by_label(cut).is_some(), "{cut}");
        assert!(
            harness.query_by_label(&format!("Draft {cut}")).is_some(),
            "{cut}"
        );
    }
    assert_eq!(harness.query_all_by_label("draft").count(), 2);
    assert_eq!(
        harness.state().project.as_ref().unwrap().cuts,
        [
            ("long".to_string(), "draft".to_string()),
            ("vertical".to_string(), "draft".to_string())
        ]
    );

    // Back among the projects it is the recent one, and the list is kept.
    harness.get_by_label("Projects").click();
    harness.run();
    assert!(harness.query_by_label("None yet.").is_none());
    assert_eq!(
        std::fs::read_to_string(settings.join("recent")).unwrap(),
        format!("{}\n", project.display())
    );
    harness.get_by_label(project.to_str().unwrap()).click();
    harness.run();
    assert!(harness.query_by_label("My talk").is_some());

    // Opened by its folder, in an app started afresh.
    drop(harness);
    let mut harness = self::harness(&settings);
    assert!(harness.query_by_label(project.to_str().unwrap()).is_some());
    fill(&mut harness, "Project folder", project.to_str().unwrap());
    harness.get_by_label("Open").click();
    harness.run();
    assert!(harness.query_by_label("Draft every cut").is_some());

    // A folder with no project in it says so, and isn't listed as recent.
    harness.get_by_label("Projects").click();
    harness.run();
    fill(&mut harness, "Project folder", videos.to_str().unwrap());
    harness.get_by_label("Open").click();
    harness.run();
    assert!(
        harness
            .query_by_label("A project that can't be read")
            .is_some()
    );
    assert_eq!(harness.state().recent, std::slice::from_ref(&project));

    // Check runs `vidni validate` as a job, and its log starts with the command.
    harness.get_by_label("Projects").click();
    harness.run();
    harness.get_by_label(project.to_str().unwrap()).click();
    harness.run();
    harness.get_by_label("Check").click();
    harness.step();
    harness.state_mut().job.as_mut().unwrap().wait();
    harness.run();
    let command = format!("$ vidni validate {}", project.display());
    assert!(harness.query_by_label(&command).is_some(), "{command}");
    assert!(harness.query_by_label("Finished.").is_some());
    assert!(
        harness
            .query_by_label_contains("ok: 2 lines in 1 section, 1 scene, 2 cuts")
            .is_some()
    );
}

#[test]
#[ignore = "needs Piper and its voice, Chromium and ffmpeg; run `just test-app`"]
fn a_project_is_drafted_from_the_app() {
    let dir = tempfile::tempdir().unwrap();
    let videos = dir.path().join("videos");
    std::fs::create_dir(&videos).unwrap();
    let mut harness = harness(&dir.path().join("settings"));
    fill(&mut harness, "Name", "demo");
    fill(&mut harness, "In folder", videos.to_str().unwrap());
    harness.get_by_label("Create").click();
    harness.run();
    harness.get_by_label("Draft every cut").click();
    // One frame, not until the picture is still: a running job spins.
    harness.step();
    // While it runs, nothing else can be started.
    assert!(harness.state().busy());
    let code = harness.state_mut().job.as_mut().unwrap().wait();
    harness.run();
    let project = videos.join("demo");
    let log: Vec<String> = harness
        .state()
        .job
        .as_ref()
        .unwrap()
        .lines
        .iter()
        .map(|(_, line)| line.clone())
        .collect();
    assert_eq!(code, 0, "{log:#?}");
    for cut in ["long.mp4", "vertical.mp4"] {
        assert!(project.join("build/draft").join(cut).is_file(), "{cut}");
    }
    // The log is the command's own: what it ran, and what it made.
    let command = format!("$ vidni draft {}", project.display());
    assert!(harness.query_by_label(&command).is_some(), "{command}");
    assert!(
        log.iter()
            .any(|line| line.contains("build/draft/long.mp4: ")),
        "{log:#?}"
    );
    assert!(harness.query_by_label("Finished.").is_some());
    // One cut alone, drafted again: nothing has changed, so nothing is rendered.
    harness.get_by_label("Draft vertical").click();
    harness.step();
    assert_eq!(harness.state_mut().job.as_mut().unwrap().wait(), 0);
    harness.run();
    let command = format!("$ vidni draft {} --cut vertical", project.display());
    assert!(harness.query_by_label(&command).is_some(), "{command}");
    assert!(
        harness
            .query_by_label_contains("nothing has changed since it was built")
            .is_some()
    );
}
