//! The fonts vidni draws with, each pinned by size and SHA-256
//! (`fonts.toml`), and where their files are found.
//!
//! A cut's text is part of its frames, so a font is an input like any
//! other: the same file everywhere, named by its hash. Fonts are looked for
//! only in the folders vidni is given, never in the system's.

use crate::capture::sha256_hex;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

/// The fonts' data, embedded in vidni.
pub const DATA: &str = include_str!("fonts.toml");

/// How deep below a font folder a file may be: packages keep fonts in
/// `opentype/` or `truetype/`, sometimes a level further down.
const DEPTH: usize = 4;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct Fonts {
    font: Vec<Font>,
}

/// One pinned font file.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Font {
    pub id: String,
    pub family: String,
    pub style: String,
    pub version: String,
    /// The file's name.
    pub file: String,
    pub size: u64,
    pub sha256: String,
    /// The licence, as an SPDX ID.
    pub licence: String,
    /// The font's home.
    pub source: String,
}

impl Font {
    /// The font as people name it: `IBM Plex Sans Bold 1.1.0`.
    pub fn name(&self) -> String {
        format!("{} {} {}", self.family, self.style, self.version)
    }
}

/// Every pinned font, in the order of `fonts.toml`.
pub fn all() -> Vec<Font> {
    toml::from_str::<Fonts>(DATA)
        .expect("fonts.toml is checked by the tests")
        .font
}

/// The pinned font with `id`.
pub fn get(id: &str) -> Option<Font> {
    all().into_iter().find(|f| f.id == id)
}

/// The folders fonts are looked for in: each folder `$VIDNI_FONTS` names
/// (separated by `:`), else `$XDG_DATA_HOME/vidni/fonts`, else
/// `~/.local/share/vidni/fonts`.
pub fn dirs() -> Vec<PathBuf> {
    let var = |name: &str| std::env::var_os(name).filter(|v| !v.is_empty());
    if let Some(list) = var("VIDNI_FONTS") {
        return std::env::split_paths(&list)
            .filter(|p| !p.as_os_str().is_empty())
            .collect();
    }
    var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| var("HOME").map(|home| Path::new(&home).join(".local/share")))
        .map(|data| vec![data.join("vidni/fonts")])
        .unwrap_or_default()
}

/// Every file named `name` in `dir` or up to [`DEPTH`] folders below it, in
/// name order.
fn named(dir: &Path, name: &str, depth: usize, found: &mut Vec<PathBuf>) {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            if depth > 0 {
                named(&path, name, depth - 1, found);
            }
        } else if path.file_name().is_some_and(|n| n == name) {
            found.push(path);
        }
    }
}

/// The file of `font` in `dirs`: a file of its name with the pinned size
/// and SHA-256. A file of that name with other bytes isn't the font.
pub fn find(font: &Font, dirs: &[PathBuf]) -> Result<PathBuf, String> {
    let mut candidates = Vec::new();
    for dir in dirs {
        named(dir, &font.file, DEPTH, &mut candidates);
    }
    for path in &candidates {
        let pinned = fs::metadata(path).is_ok_and(|m| m.len() == font.size)
            && fs::read(path).is_ok_and(|bytes| sha256_hex(&bytes) == font.sha256);
        if pinned {
            return Ok(path.clone());
        }
    }
    let looked = if dirs.is_empty() {
        "no font folder is set".to_string()
    } else {
        let dirs: Vec<String> = dirs.iter().map(|d| d.display().to_string()).collect();
        format!("looked in {}", dirs.join(", "))
    };
    Err(match candidates.first() {
        Some(other) => format!(
            "{}: not the pinned {} (sha256 {})",
            other.display(),
            font.name(),
            font.sha256
        ),
        None => format!(
            "the font {} ({}) wasn't found: {looked}; VIDNI_FONTS names the folders that hold vidni's fonts",
            font.file,
            font.name()
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pinned_fonts_parse_and_are_complete() {
        let fonts = all();
        assert!(get("ibm-plex-sans-bold").is_some());
        for f in &fonts {
            assert_eq!(f.sha256.len(), 64, "{}", f.id);
            assert!(f.size > 0 && !f.licence.is_empty(), "{}", f.id);
            assert!(f.source.starts_with("https://"), "{}", f.id);
            assert_eq!(fonts.iter().filter(|o| o.id == f.id).count(), 1, "{}", f.id);
        }
    }

    /// A font pinned to `bytes`.
    fn pinned(bytes: &[u8]) -> Font {
        Font {
            id: "test-regular".into(),
            family: "Test".into(),
            style: "Regular".into(),
            version: "1.0".into(),
            file: "Test-Regular.otf".into(),
            size: bytes.len() as u64,
            sha256: sha256_hex(bytes),
            licence: "OFL-1.1".into(),
            source: "https://example.com".into(),
        }
    }

    #[test]
    fn a_font_is_found_by_name_and_hash_below_its_folder() {
        let store = tempfile::tempdir().unwrap();
        let font = pinned(b"the font's bytes");
        let folder = store.path().join("share/fonts/opentype");
        fs::create_dir_all(&folder).unwrap();
        fs::write(folder.join("Test-Regular.otf"), b"the font's bytes").unwrap();
        let empty = store.path().join("empty");
        fs::create_dir_all(&empty).unwrap();
        let dirs = [empty.clone(), store.path().join("share")];
        assert_eq!(find(&font, &dirs).unwrap(), folder.join("Test-Regular.otf"));
        // Nowhere to be found: the error names the font and the folders.
        let error = find(&font, &[empty]).unwrap_err();
        assert!(
            error.starts_with(
                "the font Test-Regular.otf (Test Regular 1.0) wasn't found: looked in "
            ),
            "{error}"
        );
        assert!(
            find(&font, &[])
                .unwrap_err()
                .contains("no font folder is set")
        );
    }

    #[test]
    fn a_file_with_the_fonts_name_and_other_bytes_is_refused() {
        let store = tempfile::tempdir().unwrap();
        let font = pinned(b"the font's bytes");
        // The same length, so only the hash tells them apart.
        fs::write(store.path().join("Test-Regular.otf"), b"some other bytes").unwrap();
        let error = find(&font, &[store.path().to_path_buf()]).unwrap_err();
        assert!(error.contains("not the pinned Test Regular 1.0"), "{error}");
        // The pinned file further along is still found.
        let good = store.path().join("z");
        fs::create_dir_all(&good).unwrap();
        fs::write(good.join("Test-Regular.otf"), b"the font's bytes").unwrap();
        assert_eq!(
            find(&font, &[store.path().to_path_buf()]).unwrap(),
            good.join("Test-Regular.otf")
        );
    }
}
