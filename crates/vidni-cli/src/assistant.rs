//! The assistant protocol's guard (TODO.md, P8-VIDNI-08;
//! `docs/assistant.md`). An assistant, of any make, works on a project
//! with `VIDNI_ASSISTANT` set. Then:
//!
//! - the commands that are a person's to run refuse: the gates, the
//!   recorder, and what moves or deletes assets;
//! - `vidni validate` also holds the project folder to what it was when
//!   the session started: only `vidni.toml` and `notes.md` may differ.
//!
//! `vidni assistant` starts a session: it notes every file of the project
//! folder by its hash, in `build/assistant.json`. A file that a command
//! writes, for the assistant or for a person meanwhile, is noted again by
//! that command.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use vidni_core::capture::sha256_hex;
use vidni_core::validate;

/// The variable an assistant's session is marked by.
pub const VAR: &str = "VIDNI_ASSISTANT";
/// Where a session's note of the project's files is.
pub const SESSION: &str = "build/assistant.json";
/// The folders an assistant's commands fill, and nobody's hand: they
/// aren't held to the session's start.
const BUILT: [&str; 4] = ["build", "out", "takes", ".git"];

/// Whether this command runs for an assistant.
pub fn on() -> bool {
    std::env::var_os(VAR).is_some_and(|v| !v.is_empty())
}

/// Every file under the project folder that a person keeps, with its
/// hash, by its path from the folder.
fn files(dir: &Path) -> Result<BTreeMap<String, String>, String> {
    let mut found = BTreeMap::new();
    let mut folders: Vec<PathBuf> = vec![dir.to_path_buf()];
    while let Some(folder) = folders.pop() {
        let entries = fs::read_dir(&folder).map_err(|e| format!("{}: {e}", folder.display()))?;
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if folder == dir && BUILT.contains(&name.as_str()) {
                continue;
            }
            // A file a command is writing at this moment.
            if name.ends_with(".part") {
                continue;
            }
            if path.is_dir() {
                folders.push(path);
                continue;
            }
            let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let inside = path.strip_prefix(dir).unwrap_or(&path);
            found.insert(inside.to_string_lossy().into_owned(), sha256_hex(&bytes));
        }
    }
    Ok(found)
}

fn read(dir: &Path) -> Option<BTreeMap<String, String>> {
    let text = fs::read_to_string(dir.join(SESSION)).ok()?;
    serde_json::from_str(&text).ok()
}

fn write(dir: &Path, files: &BTreeMap<String, String>) -> Result<(), String> {
    let path = dir.join(SESSION);
    let folder = path.parent().expect("a folder");
    fs::create_dir_all(folder).map_err(|e| format!("{}: {e}", folder.display()))?;
    let text = serde_json::to_string_pretty(files).expect("JSON serializes") + "\n";
    fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))
}

/// The project's folder and its file's name, from what a command is given.
fn project(path: &Path) -> (PathBuf, String) {
    let file = validate::project_file(path);
    let dir = file
        .parent()
        .filter(|d| !d.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
        .to_path_buf();
    let name = file.file_name().unwrap_or_default().to_string_lossy();
    (dir, name.into_owned())
}

/// Starts an assistant's session on the project at `path`.
pub fn start(path: &Path) -> ExitCode {
    let (dir, name) = project(path);
    // A session is started once: started again by the assistant, it would
    // take whatever was changed since as how things were.
    if on() && read(&dir).is_some() {
        tell!(
            "vidni: this project has an assistant's session already; only a person starts \
             another, by running `vidni assistant` without {VAR}"
        );
        return ExitCode::FAILURE;
    }
    let noted = files(&dir).and_then(|files| write(&dir, &files).map(|()| files.len()));
    match noted {
        Ok(count) => {
            say!(
                "an assistant's session on {}: {count} files noted as they are",
                dir.display()
            );
            say!("  it may change {name} and notes.md, and nothing else by hand");
            say!(
                "  it runs vidni's commands with {VAR}=1, and `vidni validate` after every change"
            );
            say!(
                "  ratify, record, approve and published are a person's; docs/assistant.md has the rest"
            );
            ExitCode::SUCCESS
        }
        Err(why) => {
            tell!("vidni: {why}");
            ExitCode::FAILURE
        }
    }
}

/// What differs from the session's start that an assistant may not
/// change, each as a sentence. `path` is what `vidni validate` was given.
pub fn broken(path: &Path) -> Vec<String> {
    let (dir, name) = project(path);
    let Some(then) = read(&dir) else {
        return vec![
            "there is no assistant's session on this project; `vidni assistant` starts one, \
             before anything is changed"
                .to_string(),
        ];
    };
    let now = match files(&dir) {
        Ok(now) => now,
        Err(why) => return vec![why],
    };
    let may = |file: &str| file == name || file == vidni_core::notes::FILE;
    let mut said = Vec::new();
    for (file, hash) in &now {
        match then.get(file) {
            _ if may(file) => {}
            None => said.push(format!("{file} is new, and an assistant adds no file")),
            Some(was) if was != hash => said.push(format!("{file} was changed{}", whose(file))),
            Some(_) => {}
        }
    }
    for file in then
        .keys()
        .filter(|file| !now.contains_key(*file) && !may(file))
    {
        said.push(format!("{file} was removed{}", whose(file)));
    }
    said
}

/// Whose a file is to change, for the sentence about it.
fn whose(file: &str) -> &'static str {
    match file {
        "ratified.lock" => ": only `vidni ratify`, run by a person, writes it",
        "release.json" => ": only `vidni approve`, run by a person, writes it",
        "published.json" => ": only `vidni published`, run by a person, writes it",
        "voice.lock" | "assets.lock" => ": vidni's commands write it, and no hand",
        _ => ": an assistant changes vidni.toml and notes.md only",
    }
}

/// Notes `names` again as they are now, in the session there is: a
/// command wrote them, for the assistant or for a person meanwhile.
pub fn wrote(path: &Path, names: &[&str]) {
    let (dir, _) = project(path);
    let Some(mut noted) = read(&dir) else { return };
    for name in names {
        match fs::read(dir.join(name)) {
            Ok(bytes) => noted.insert(name.to_string(), sha256_hex(&bytes)),
            Err(_) => noted.remove(*name),
        };
    }
    let _ = write(&dir, &noted);
}
