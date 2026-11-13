//! Every broken fixture in tests/fixtures/invalid fails with exactly the
//! problem its first line names (`# expect: <line>:<column>: <message>`), and
//! every fixture in tests/fixtures/valid passes.

use std::fs;
use std::path::{Path, PathBuf};
use vidni_core::validate::check;

fn fixtures(kind: &str) -> (PathBuf, Vec<PathBuf>) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(kind);
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "toml"))
        .collect();
    files.sort();
    (dir, files)
}

#[test]
fn every_valid_fixture_passes() {
    let (dir, files) = fixtures("valid");
    assert!(!files.is_empty());
    for file in files {
        let text = fs::read_to_string(&file).unwrap();
        if let Err(problems) = check(&text, Some(&dir)) {
            panic!("{}: {problems:?}", file.display());
        }
    }
}

#[test]
fn every_broken_fixture_fails_at_its_line_and_column() {
    let (dir, files) = fixtures("invalid");
    assert!(
        files.len() >= 10,
        "P2-VIDNI-01 asks for at least 10 broken fixtures"
    );
    for file in files {
        let text = fs::read_to_string(&file).unwrap();
        let expected = text
            .lines()
            .next()
            .and_then(|l| l.strip_prefix("# expect: "))
            .unwrap_or_else(|| {
                panic!(
                    "{}: the first line must be # expect: <line>:<column>: <message>",
                    file.display()
                )
            });
        let problems = check(&text, Some(&dir)).expect_err(&file.display().to_string());
        let got: Vec<String> = problems.iter().map(ToString::to_string).collect();
        assert_eq!(got, [expected], "{}", file.display());
    }
}
