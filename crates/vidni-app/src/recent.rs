//! The projects opened lately, newest first: `recent` in the app's
//! folder of settings, a path to a line.

use std::fs;
use std::path::{Path, PathBuf};

/// How many projects are kept.
pub const KEPT: usize = 12;

/// The app's folder of settings: `$VIDNI_CONFIG`, or `vidni` under
/// `$XDG_CONFIG_HOME`, or under `~/.config`.
pub fn folder() -> Option<PathBuf> {
    let set = |name: &str| {
        std::env::var_os(name)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    set("VIDNI_CONFIG")
        .or_else(|| set("XDG_CONFIG_HOME").map(|dir| dir.join("vidni")))
        .or_else(|| set("HOME").map(|home| home.join(".config/vidni")))
}

/// The projects listed in `folder`; none when there is no list.
pub fn read(folder: &Path) -> Vec<PathBuf> {
    fs::read_to_string(folder.join("recent"))
        .unwrap_or_default()
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(PathBuf::from)
        .collect()
}

/// `list` with `project` first, once, and no more than [`KEPT`].
pub fn with(list: &[PathBuf], project: &Path) -> Vec<PathBuf> {
    let mut all = vec![project.to_path_buf()];
    all.extend(list.iter().filter(|p| p.as_path() != project).cloned());
    all.truncate(KEPT);
    all
}

/// Writes `list` into `folder`.
pub fn write(folder: &Path, list: &[PathBuf]) -> Result<(), String> {
    let text: String = list.iter().map(|p| format!("{}\n", p.display())).collect();
    fs::create_dir_all(folder)
        .and_then(|()| fs::write(folder.join("recent"), text))
        .map_err(|e| format!("{}: {e}", folder.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_newest_project_is_first_and_listed_once() {
        let paths = |names: &[&str]| names.iter().map(PathBuf::from).collect::<Vec<_>>();
        let list = with(&paths(&["/a", "/b", "/c"]), Path::new("/b"));
        assert_eq!(list, paths(&["/b", "/a", "/c"]));
        let many: Vec<PathBuf> = (0..20).map(|n| PathBuf::from(format!("/p{n}"))).collect();
        assert_eq!(with(&many, Path::new("/new")).len(), KEPT);
        let dir = std::env::temp_dir().join(format!("vidni-recent-{}", std::process::id()));
        assert_eq!(read(&dir), Vec::<PathBuf>::new());
        write(&dir, &list).unwrap();
        assert_eq!(read(&dir), list);
        fs::remove_dir_all(&dir).unwrap();
    }
}
