//! Where each release was published: `published.json`, in the project
//! folder (TODO.md, P6-VIDNI-07).
//!
//! vidni uploads nothing, so the creator tells it where a cut went:
//! `vidni published --cut long --platform youtube --url …`. The list is
//! for people, and for `vidni where`, which says which published videos
//! say a line. No build reads it.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// The list's file, in the project folder.
pub const FILE: &str = "published.json";

/// The layout of `published.json` that this vidni reads and writes.
pub const SCHEMA: u32 = 1;

/// One upload: which release of which cut went where.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Upload {
    pub cut: String,
    pub platform: String,
    /// The release that was uploaded, such as `long-r2`.
    pub release: String,
    /// Where it is.
    pub url: String,
    /// The day it was recorded here, in UTC.
    pub on: String,
}

/// What `published.json` holds: every upload, oldest first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Published {
    pub schema: u32,
    #[serde(default)]
    pub uploads: Vec<Upload>,
}

impl Default for Published {
    fn default() -> Self {
        Self {
            schema: SCHEMA,
            uploads: Vec::new(),
        }
    }
}

impl Published {
    /// Adds `upload`. A second record for the same cut and platform at
    /// the same address replaces the first; another address is another
    /// upload, as a corrected video is on YouTube, which can't swap the
    /// file of a published one.
    pub fn add(&mut self, upload: Upload) {
        self.uploads.retain(|u| {
            (&u.cut, &u.platform, &u.url) != (&upload.cut, &upload.platform, &upload.url)
        });
        self.uploads.push(upload);
    }

    /// The uploads of `cut`, oldest first.
    pub fn of<'a>(&'a self, cut: &'a str) -> impl Iterator<Item = &'a Upload> {
        self.uploads.iter().filter(move |u| u.cut == cut)
    }
}

/// The project's list; empty when it has none.
pub fn read(project_dir: &Path) -> Result<Published, String> {
    let path = project_dir.join(FILE);
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Published::default()),
        Err(e) => return Err(format!("{}: {e}", path.display())),
    };
    let list: Published =
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    match list.schema {
        SCHEMA => Ok(list),
        other => Err(format!(
            "{}: its layout is schema {other}, and this vidni reads schema {SCHEMA}",
            path.display()
        )),
    }
}

/// Writes the project's list.
pub fn write(project_dir: &Path, list: &Published) -> Result<(), String> {
    let path = project_dir.join(FILE);
    let text = serde_json::to_string_pretty(list).map_err(|e| e.to_string())? + "\n";
    fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn upload(cut: &str, platform: &str, release: &str, url: &str) -> Upload {
        Upload {
            cut: cut.into(),
            platform: platform.into(),
            release: release.into(),
            url: url.into(),
            on: "2026-11-14".into(),
        }
    }

    #[test]
    fn the_list_is_kept_and_a_corrected_video_is_another_upload() {
        let dir = std::env::temp_dir().join(format!("vidni-published-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        assert_eq!(read(&dir).unwrap(), Published::default());
        let mut list = Published::default();
        list.add(upload("long", "youtube", "long-r1", "https://youtu.be/a"));
        list.add(upload(
            "short",
            "youtube-shorts",
            "short-r1",
            "https://youtube.com/shorts/b",
        ));
        // The same address again: the record is replaced, as for a typo in the release.
        list.add(upload("long", "youtube", "long-r2", "https://youtu.be/a"));
        // A corrected video has an address of its own, and both stay.
        list.add(upload("long", "youtube", "long-r3", "https://youtu.be/c"));
        write(&dir, &list).unwrap();
        let back = read(&dir).unwrap();
        assert_eq!(back, list);
        let long: Vec<(&str, &str)> = back
            .of("long")
            .map(|u| (u.release.as_str(), u.url.as_str()))
            .collect();
        assert_eq!(
            long,
            [
                ("long-r2", "https://youtu.be/a"),
                ("long-r3", "https://youtu.be/c")
            ]
        );
        fs::write(dir.join(FILE), "{\"schema\": 2}").unwrap();
        assert!(read(&dir).unwrap_err().contains("schema 2"));
        fs::remove_dir_all(&dir).unwrap();
    }
}
