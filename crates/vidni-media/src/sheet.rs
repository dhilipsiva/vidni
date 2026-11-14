//! The recording sheet (TODO.md, P2-VIDNI-08): every line to read, in
//! script order, with the words to say, whether it has a take yet, and how
//! long the draft voice took over it, as a pace to read at.

use std::path::Path;
use vidni_core::project::{Cut, Project};
use vidni_core::spoken;
use vidni_core::takes::{self, Status};
use vidni_core::voices::{self, Pack};

/// Whether a line has been recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Take {
    /// There is a take of the line's present words.
    Recorded,
    /// There are takes of other words only: the line was edited since.
    Stale,
    /// There is no take.
    Missing,
}

/// One line of the sheet.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub id: String,
    /// The line's section, and the section's label.
    pub section: String,
    pub label: String,
    /// The words to read: the line's spoken form.
    pub text: String,
    pub take: Take,
    /// How long the draft voice took over the line, in seconds, when it
    /// has spoken the line as it is now.
    pub pace: Option<f64>,
}

/// The sheet for `cuts`' lines, each once, in script order. `pack` is the
/// draft voice's pack; without it no line has a pace.
pub fn sheet(
    project: &Project,
    dir: &Path,
    cuts: &[&Cut],
    pack: Option<&Pack>,
) -> Result<Vec<Row>, String> {
    let lock = voices::read_lock(dir)?;
    let wanted: Vec<String> = cuts.iter().flat_map(|cut| cut.line_ids(project)).collect();
    let rows = project
        .lines
        .iter()
        .zip(takes::status(project, dir))
        .filter(|(line, _)| wanted.contains(&line.id))
        .map(|(line, take)| {
            let pace = pack
                .and_then(|pack| voices::drafted(dir, &lock, pack, &project.voice, line))
                .and_then(|clip| hound::WavReader::open(clip).ok())
                .map(|clip| f64::from(clip.duration()) / f64::from(clip.spec().sample_rate.max(1)));
            Row {
                id: line.id.clone(),
                section: line.section.clone(),
                label: project
                    .sections
                    .get(&line.section)
                    .map(|s| s.label.clone())
                    .unwrap_or_default(),
                text: spoken::spoken(line, &project.voice),
                take: match take.status {
                    Status::Current(_) => Take::Recorded,
                    Status::Stale(_) => Take::Stale,
                    Status::Missing => Take::Missing,
                },
                pace,
            }
        })
        .collect();
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    const PROJECT: &str = r##"schema = 1

[meta]
title = "A sheet"
default_scene = "page"

[voice]
read_as = [{ text = "#1", say = "number one" }]

[[cuts]]
id = "long"
aspect = "16:9"

[[cuts]]
id = "short"
aspect = "9:16"
lines = ["main-2", "end-1"]

[sections.main]
label = "The main part"

[sections.end]

[[lines]]
id = "main-1"
section = "main"
show = "Source #1 says so."

[[lines]]
id = "main-2"
section = "main"
show = "Shown one way."
say = "Said another way."

[[lines]]
id = "end-1"
section = "end"
show = "The end."

[scenes.page]
kind = "web"
url = "https://example.com"
"##;

    #[test]
    fn the_sheet_lists_what_to_read_what_is_recorded_and_the_drafts_pace() {
        let dir = tempfile::tempdir().unwrap();
        let dir = dir.path();
        let project = vidni_core::validate::check(PROJECT, Some(dir)).unwrap();
        let pack = voices::all().remove(0);
        let cuts: Vec<&Cut> = project.cuts.iter().collect();
        // The first line is drafted: a clip of a second and a half.
        let line = &project.lines[0];
        let given = spoken::synthetic(line, &project.voice);
        let key = voices::clip_key(&given, &pack, project.voice.length_scale);
        let clip = voices::clip_path(dir, &line.id, &key, 3);
        fs::create_dir_all(clip.parent().unwrap()).unwrap();
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 22050,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(&clip, spec).unwrap();
        for _ in 0..33075 {
            writer.write_sample(0i16).unwrap();
        }
        writer.finalize().unwrap();
        let mut lock = voices::Lock::default();
        lock.lines
            .insert(line.id.clone(), voices::Kept { key, seed: 3 });
        voices::write_lock(dir, &lock).unwrap();
        // The second has a take; the third has one of other words.
        let states = takes::status(&project, dir);
        fs::create_dir_all(states[1].path.parent().unwrap()).unwrap();
        fs::write(&states[1].path, b"a take").unwrap();
        let other = states[2].path.with_file_name("0000000000000000.wav");
        fs::create_dir_all(other.parent().unwrap()).unwrap();
        fs::write(other, b"a take of other words").unwrap();

        let rows = sheet(&project, dir, &cuts, Some(&pack)).unwrap();
        let shown: Vec<(&str, &str, &str, Take, Option<f64>)> = rows
            .iter()
            .map(|r| {
                (
                    r.id.as_str(),
                    r.label.as_str(),
                    r.text.as_str(),
                    r.take,
                    r.pace,
                )
            })
            .collect();
        assert_eq!(
            shown,
            [
                // What a reader says, not what the captions show.
                (
                    "main-1",
                    "The main part",
                    "Source number one says so.",
                    Take::Missing,
                    Some(1.5)
                ),
                (
                    "main-2",
                    "The main part",
                    "Said another way.",
                    Take::Recorded,
                    None
                ),
                ("end-1", "", "The end.", Take::Stale, None),
            ]
        );
        // One cut's sheet has its lines only, and no pace without the voice.
        let short = sheet(&project, dir, &cuts[1..], None).unwrap();
        let ids: Vec<&str> = short.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(ids, ["main-2", "end-1"]);
        assert!(short.iter().all(|r| r.pace.is_none()));
    }
}
