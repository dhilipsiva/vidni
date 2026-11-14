//! Terminal and code scenes (TODO.md, P5-VIDNI-02 and P5-VIDNI-03): the
//! window is the same picture each time it is drawn, its marked lines
//! are where its frames say, and code at a commit stays what it was.

use std::fs;
use std::path::Path;
use std::process::Command;
use vidni_core::project::Scene;
use vidni_media::compose::panel;
use vidni_media::textscene::still;

fn scene(keys: serde_json::Value) -> Scene {
    serde_json::from_value(keys).unwrap()
}

/// The colour at `(x, y)` of a still drawn at `scale`, in CSS pixels.
fn at(image: &vidni_media::compose::image::Image, scale: u32, x: f64, y: f64) -> [u8; 3] {
    let pixel = image.pixel((x * f64::from(scale)) as u32, (y * f64::from(scale)) as u32);
    [pixel[0], pixel[1], pixel[2]]
}

const SESSION: &str =
    "~> ask\nan answer \u{22a2} with a sign\n\n~> ask again\n\x1b[32mgreen\x1b[0m\n";

#[test]
fn a_terminal_is_the_same_picture_each_time_with_its_marked_lines_where_it_says() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("session.txt"), SESSION).unwrap();
    let terminal = scene(serde_json::json!({
        "kind": "terminal",
        "url": "session.txt",
        "prompt": "~> ",
        "scale": 2,
        "columns": 40,
        "targets": { "first": { "lines": [1, 2] }, "second": { "lines": [4, 5] } },
    }));
    let plain = still(dir.path(), &terminal, &[]).unwrap();
    let again = still(dir.path(), &terminal, &[]).unwrap();
    // Drawn twice: the same bytes, and the same frames.
    assert_eq!(plain.image, again.image);
    assert_eq!(plain.frames, again.frames);
    // As large as its text says, at twice its size.
    let [width, height] = plain.frames.viewport;
    assert_eq!(
        height,
        (panel::BAR + panel::PAD * 2.0 + 5.0 * panel::ROW) as u32
    );
    assert_eq!(
        (plain.image.width, plain.image.height),
        (width * 2, height * 2)
    );
    // Each part is a band across the window, at its lines' rows.
    let top = f64::from(panel::BAR + panel::PAD);
    let row = f64::from(panel::ROW);
    assert_eq!(
        plain.frames.targets["first"],
        [[0.0, top, f64::from(width), row * 2.0]]
    );
    assert_eq!(
        plain.frames.targets["second"],
        [[0.0, top + row * 3.0, f64::from(width), row * 2.0]]
    );
    assert!(plain.frames.highlights.is_empty());
    // The file it was read from is on record, by its hash.
    let sha = vidni_core::capture::sha256_hex(SESSION.as_bytes());
    assert_eq!(plain.files["session.txt"], Some(sha));

    // Marked: the band behind the second part is another colour, its
    // left edge is the mark's own, and the rest is as it was.
    let marked = still(dir.path(), &terminal, &["second".to_string()]).unwrap();
    assert_eq!(
        marked.frames.highlights["second"],
        marked.frames.targets["second"]
    );
    assert_eq!(marked.frames.targets, plain.frames.targets);
    let inside = (f64::from(width) - 4.0, top + row * 3.5);
    let outside = (f64::from(width) - 4.0, top + row * 0.5);
    assert_eq!(at(&plain.image, 2, inside.0, inside.1), panel::GROUND);
    assert_ne!(at(&marked.image, 2, inside.0, inside.1), panel::GROUND);
    assert_eq!(at(&marked.image, 2, 1.0, inside.1), panel::MARK);
    assert_eq!(at(&marked.image, 2, outside.0, outside.1), panel::GROUND);
    // There is ink where the first command is, and the sign of logic is
    // drawn: its cell isn't empty.
    let has_ink = |image: &vidni_media::compose::image::Image, x0: f64, y0: f64, w: f64| {
        (0..(w as u32 * 2)).any(|dx| {
            (0..(panel::ROW as u32 * 2)).any(|dy| {
                let p = image.pixel((x0 * 2.0) as u32 + dx, (y0 * 2.0) as u32 + dy);
                [p[0], p[1], p[2]] != panel::GROUND
            })
        })
    };
    let cell = f64::from(width - 48) / 40.0;
    assert!(has_ink(&plain.image, 24.0, top, cell * 6.0));
    assert!(has_ink(&plain.image, 24.0 + cell * 10.0, top + row, cell));
    assert!(!has_ink(&plain.image, 24.0, top + row * 2.0, cell * 40.0));

    // The transcript changes: the picture does, and so does its record.
    fs::write(
        dir.path().join("session.txt"),
        SESSION.replace("ask", "tell"),
    )
    .unwrap();
    let changed = still(dir.path(), &terminal, &[]).unwrap();
    assert_ne!(changed.image, plain.image);
    assert_ne!(changed.files, plain.files);
}

fn git(repo: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args([
            "-c",
            "user.name=vidni",
            "-c",
            "user.email=vidni@example.com",
        ])
        .args([
            "-c",
            "commit.gpgsign=false",
            "-c",
            "init.defaultBranch=main",
        ])
        .args(args)
        .env("GIT_AUTHOR_DATE", "2026-01-01T00:00:00Z")
        .env("GIT_COMMITTER_DATE", "2026-01-01T00:00:00Z")
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn code_at_a_commit_is_what_the_file_was_then() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo");
    let project = repo.join("videos/one");
    fs::create_dir_all(repo.join("src")).unwrap();
    fs::create_dir_all(&project).unwrap();
    let first = "// then\nfn main() {\n    println!(\"then\");\n}\n";
    fs::write(repo.join("src/main.rs"), first).unwrap();
    git(&repo, &["init", "-q"]);
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-q", "-m", "then"]);
    let then = git(&repo, &["rev-parse", "HEAD"]);
    let now = first.replace("then", "now") + "// one line more\n";
    fs::write(repo.join("src/main.rs"), &now).unwrap();
    git(&repo, &["commit", "-q", "-am", "now"]);

    let at_commit = |rev: &str| {
        scene(serde_json::json!({
            "kind": "code",
            "url": "src/main.rs",
            "repo": "../..",
            "rev": rev,
            "targets": { "body": { "lines": [3, 3] } },
        }))
    };
    let old = still(&project, &at_commit(&then), &["body".to_string()]).unwrap();
    // The file as it was: four lines, not five; read from no file of the
    // project's, and said to be of that commit.
    let rows = |still: &vidni_media::textscene::Still| {
        let text = f32::from(u16::try_from(still.frames.viewport[1]).unwrap());
        ((text - panel::BAR - panel::PAD * 2.0) / panel::ROW) as u32
    };
    assert_eq!(rows(&old), 4);
    assert!(old.files.is_empty());
    assert!(
        old.frames
            .browser
            .ends_with(&format!("src/main.rs at {then}")),
        "{}",
        old.frames.browser
    );
    assert_eq!(old.frames.highlights["body"].len(), 1);
    // Drawn again, after the file has changed: the same bytes. A short
    // hash names the same commit.
    assert_eq!(
        still(&project, &at_commit(&then), &["body".to_string()])
            .unwrap()
            .image,
        old.image
    );
    assert_eq!(
        still(&project, &at_commit(&then[..10]), &["body".to_string()])
            .unwrap()
            .image,
        old.image
    );
    // The commit after it is another picture, a line longer.
    let head = git(&repo, &["rev-parse", "HEAD"]);
    let new = still(&project, &at_commit(&head), &[]).unwrap();
    assert_eq!(rows(&new), 5);

    // A commit the repository hasn't, and a file the commit hasn't, are
    // said in git's words.
    let why = still(&project, &at_commit("0123456789abcdef"), &[])
        .err()
        .unwrap();
    assert!(
        why.starts_with("git rev-parse --verify 0123456789abcdef^{commit}: "),
        "{why}"
    );
    let mut elsewhere = at_commit(&then);
    elsewhere.url = "src/lib.rs".into();
    let why = still(&project, &elsewhere, &[]).err().unwrap();
    assert!(
        why.starts_with(&format!("git show {then}:src/lib.rs: ")),
        "{why}"
    );

    // Without rev, the file is the project's own, as it is now.
    fs::write(project.join("now.rs"), &now).unwrap();
    let own = scene(serde_json::json!({ "kind": "code", "url": "now.rs" }));
    let own = still(&project, &own, &[]).unwrap();
    assert_eq!(rows(&own), 5);
    assert_eq!(
        own.files["now.rs"],
        Some(vidni_core::capture::sha256_hex(now.as_bytes()))
    );
}
