//! What goes with a cut where it is uploaded (TODO.md, P6-VIDNI-04): its
//! title, its description, its tags and the steps of the upload, as the
//! `metadata.md` of each platform's folder in `out/`.
//!
//! The words are the project's (`[publish]`); what vidni adds is what it
//! knows: the chapters, the credit a track's licence asks for, the limits
//! of each platform, and what each app asks to be told. The limits were
//! read off the platforms' own pages on 2026-11-14 (DECISIONS.md, T19).

use crate::chapters::{self, Chapter};
use crate::platforms::Profile;
use crate::project::{Aspect, Cut, Project};

/// The longest title YouTube takes, in characters.
pub const TITLE: usize = 100;
/// The longest description YouTube takes, in characters.
pub const DESCRIPTION: usize = 5000;
/// The most characters YouTube takes of tags: each tag, the commas
/// between them, and two quotation marks for a tag with a space in it.
pub const TAGS: usize = 500;
/// The longest caption Instagram takes, and the most tags of a post.
pub const CAPTION: usize = 2200;
pub const HASHTAGS: usize = 5;
/// The most thumbnails YouTube compares against each other.
pub const THUMBNAILS: usize = 3;
/// The shortest video YouTube puts an end screen on, in seconds.
pub const END_SCREEN: f64 = 25.0;

/// Whether `platform` is one of YouTube's.
fn youtube(platform: &str) -> bool {
    platform == "youtube" || platform == "youtube-shorts"
}

/// The cut's title on `platform`: the one `[publish.titles]` gives it, or
/// the project's.
pub fn title<'p>(project: &'p Project, platform: &str) -> &'p str {
    project
        .publish
        .titles
        .get(platform)
        .unwrap_or(&project.meta.title)
}

/// How many of YouTube's characters `tags` take.
pub fn tags_length(tags: &[String]) -> usize {
    let quotes = tags.iter().filter(|t| t.contains(' ')).count() * 2;
    tags.iter().map(|t| t.chars().count()).sum::<usize>() + tags.len().saturating_sub(1) + quotes
}

/// A tag as Instagram writes one: `#`, and its letters and digits run together.
fn hashtag(tag: &str) -> String {
    let word: String = tag.chars().filter(|c| c.is_alphanumeric()).collect();
    format!("#{word}")
}

/// What one upload is made of.
pub struct Upload<'a> {
    pub project: &'a Project,
    pub cut: &'a Cut,
    pub platform: &'a Profile,
    /// The cut's chapters; none when it has no list.
    pub chapters: &'a [Chapter],
    /// Whether any of the narration is a synthetic voice.
    pub synthetic: bool,
    /// The video's file, its caption files and its images, as they are
    /// named in the upload's folder.
    pub video: &'a str,
    pub captions: &'a [String],
    pub images: &'a [String],
}

impl Upload<'_> {
    /// The description, as it is pasted: the project's words, then the
    /// chapters where the platform shows them, the links, the credit for
    /// the music, and a note on a synthetic voice.
    pub fn description(&self) -> String {
        let publish = &self.project.publish;
        let mut parts: Vec<String> = Vec::new();
        if !publish.description.trim().is_empty() {
            parts.push(publish.description.trim().to_string());
        }
        if self.platform.chapters && !self.chapters.is_empty() {
            parts.push(chapters::text(self.chapters).trim_end().to_string());
        }
        if !publish.links.is_empty() {
            let links: Vec<String> = publish
                .links
                .iter()
                .map(|l| format!("{}: {}", l.label, l.url))
                .collect();
            parts.push(links.join("\n"));
        }
        if let Some(track) = self.project.music.playing() {
            parts.push(format!("Music: {}", track.attribution));
        }
        if self.synthetic {
            parts.push("The narration is a synthetic voice.".to_string());
        }
        // Instagram's tags go at the end of its caption.
        if self.platform.id == "instagram-reels" && !publish.tags.is_empty() {
            let tags: Vec<String> = publish
                .tags
                .iter()
                .take(HASHTAGS)
                .map(|t| hashtag(t))
                .collect();
            parts.push(tags.join(" "));
        }
        parts.join("\n\n")
    }

    /// The other cuts of the project that this one, a Short, can name as
    /// its related video: the 16:9 cuts made for YouTube.
    fn related(&self) -> Vec<&str> {
        self.project
            .cuts
            .iter()
            .filter(|c| {
                c.aspect == Aspect::Landscape && c.platforms().iter().any(|p| p == "youtube")
            })
            .map(|c| c.id.as_str())
            .collect()
    }

    /// The steps of the upload, each a sentence.
    fn steps(&self) -> Vec<String> {
        let (video, name) = (self.video, self.platform.name.as_str());
        let mut steps = Vec::new();
        match self.platform.id.as_str() {
            "youtube" => {
                steps.push(format!(
                    "Upload `{video}` in YouTube Studio, and paste the title and the description."
                ));
                if let Some(first) = self.captions.first() {
                    steps.push(format!("Under Subtitles, upload `{first}`, with timing."));
                }
                match self.images {
                    [] => {}
                    [one] => steps.push(format!("Under Thumbnail, upload `{one}`.")),
                    [first, ..] => steps.push(format!(
                        "Under Thumbnail, choose Test & compare and upload the {} thumbnails, `{first}` first. \
                         It is on a computer only, and needs advanced features; YouTube keeps the one that is watched longest.",
                        self.images.len()
                    )),
                }
                if let Some(hold) = self.cut.end_hold {
                    steps.push(format!(
                        "Add an end screen over the last {hold} seconds, where the last picture is held for it."
                    ));
                }
                if !self.project.publish.tags.is_empty() {
                    steps.push("Under Show more, paste the tags.".to_string());
                }
            }
            "youtube-shorts" => {
                steps.push(format!("Upload `{video}` in YouTube Studio or the app, and paste the title and the description."));
                let related = self.related();
                if !related.is_empty() {
                    steps.push(format!(
                        "Once both are up, set this Short's related video to {}: in Studio, Content, this Short, \
                         Related Video. It needs advanced features, and the video must be public or unlisted.",
                        related
                            .iter()
                            .map(|id| format!("the `{id}` cut"))
                            .collect::<Vec<_>>()
                            .join(" or ")
                    ));
                }
            }
            "instagram-reels" => {
                steps.push(format!(
                    "Share `{video}` as a reel, and paste the description as its caption."
                ));
                if let Some(cover) = self.images.first() {
                    steps.push(format!(
                        "Under Edit cover, add `{cover}` from your camera roll. Its title sits inside the 3:4 middle that the profile grid shows."
                    ));
                }
            }
            "whatsapp-status" => {
                steps.push(format!(
                    "Share `{video}` as a status, with the description as its caption."
                ));
            }
            _ => {
                steps.push(format!(
                    "Upload `{video}` to {name}, and paste the title and the description."
                ));
                if let Some(first) = self.captions.first() {
                    steps.push(format!("Attach `{first}` as its captions."));
                }
            }
        }
        if self.synthetic {
            steps.push(match self.platform.id.as_str() {
                "instagram-reels" => "Turn on Add AI label: Meta asks for it on a reel narrated with a realistic synthetic voice.".to_string(),
                "youtube" | "youtube-shorts" => "Under Attributes, answer AI use as it applies. YouTube asks for it where a real person appears to say what they didn't; its page doesn't name a synthetic voice reading your own script.".to_string(),
                _ => format!("Check whether {name} asks you to label a synthetic voice."),
            });
        }
        steps
    }

    /// What the upload would be refused or cut short for.
    pub fn problems(&self, duration: f64) -> Vec<String> {
        let mut found = Vec::new();
        let (id, description) = (self.platform.id.as_str(), self.description());
        let length = description.chars().count();
        let limit = match id {
            "youtube" | "youtube-shorts" => Some(DESCRIPTION),
            "instagram-reels" => Some(CAPTION),
            _ => None,
        };
        if let Some(limit) = limit.filter(|limit| length > *limit) {
            found.push(format!(
                "its description for {id} is {length} characters, and {} takes {limit}",
                self.platform.name
            ));
        }
        if let Some(hold) = self.cut.end_hold
            && id == "youtube"
            && duration < END_SCREEN
        {
            found.push(format!(
                "it holds its last picture {hold} s for an end screen, and YouTube puts one only on a video of {END_SCREEN} s or more"
            ));
        }
        found
    }

    /// The upload's `metadata.md`.
    pub fn metadata(&self) -> String {
        let publish = &self.project.publish;
        let id = self.platform.id.as_str();
        let mut text = format!(
            "# {} on {}\n\nMade by vidni from `vidni.toml`. Paste each part where the app asks for it; the steps are at the end.\n",
            self.cut.id, self.platform.name
        );
        if id != "whatsapp-status" && id != "instagram-reels" {
            text.push_str(&format!("\n## Title\n\n{}\n", title(self.project, id)));
        }
        let heading = match id {
            "whatsapp-status" | "instagram-reels" => "Caption",
            _ => "Description",
        };
        let description = self.description();
        text.push_str(&format!(
            "\n## {heading}\n\n{}\n",
            match description.is_empty() {
                true => "(none: `[publish] description` in `vidni.toml` is where it is written)",
                false => description.as_str(),
            }
        ));
        if youtube(id) && !publish.tags.is_empty() {
            text.push_str(&format!("\n## Tags\n\n{}\n", publish.tags.join(", ")));
        }
        text.push_str("\n## Files\n\n");
        text.push_str(&format!("- `{}`: the video\n", self.video));
        for file in self.captions {
            text.push_str(&format!("- `{file}`: its captions\n"));
        }
        for file in self.images {
            text.push_str(&format!("- `{file}`\n"));
        }
        text.push_str("\n## Steps\n\n");
        for (n, step) in self.steps().iter().enumerate() {
            text.push_str(&format!("{}. {step}\n", n + 1));
        }
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platforms;

    fn project() -> Project {
        let text = crate::TEMPLATE.to_string()
            + r#"
[publish]
description = "A short tour of vidni.\n\nMade from one text file."
tags = ["vidni", "tutorial videos", "rust", "open source", "video tools", "more"]
links = [{ label = "The code", url = "https://github.com/dhilipsiva/vidni" }]

[publish.titles]
youtube-shorts = "vidni in a minute"
"#;
        toml::from_str(&text).unwrap()
    }

    fn upload<'a>(
        project: &'a Project,
        cut: usize,
        platform: &'a Profile,
        chapters: &'a [Chapter],
        files: &'a (String, Vec<String>, Vec<String>),
        synthetic: bool,
    ) -> Upload<'a> {
        Upload {
            project,
            cut: &project.cuts[cut],
            platform,
            chapters,
            synthetic,
            video: &files.0,
            captions: &files.1,
            images: &files.2,
        }
    }

    #[test]
    fn a_youtube_upload_has_its_title_description_chapters_tags_and_steps() {
        let mut p = project();
        p.cuts[0].end_hold = Some(8.0);
        let youtube = platforms::get("youtube").unwrap();
        let chapters = [
            Chapter {
                start: 0,
                title: "Intro".into(),
            },
            Chapter {
                start: 14,
                title: "How".into(),
            },
            Chapter {
                start: 40,
                title: "End".into(),
            },
        ];
        let files = (
            "long.mp4".to_string(),
            vec!["long.en.srt".to_string(), "long.en.vtt".to_string()],
            vec!["thumbnail-1.png".to_string(), "thumbnail-2.png".to_string()],
        );
        let up = upload(&p, 0, &youtube, &chapters, &files, false);
        assert_eq!(
            up.metadata(),
            "# long on YouTube\n\n\
             Made by vidni from `vidni.toml`. Paste each part where the app asks for it; the steps are at the end.\n\n\
             ## Title\n\nMy first video\n\n\
             ## Description\n\nA short tour of vidni.\n\nMade from one text file.\n\n\
             0:00 Intro\n0:14 How\n0:40 End\n\n\
             The code: https://github.com/dhilipsiva/vidni\n\n\
             ## Tags\n\nvidni, tutorial videos, rust, open source, video tools, more\n\n\
             ## Files\n\n- `long.mp4`: the video\n- `long.en.srt`: its captions\n- `long.en.vtt`: its captions\n\
             - `thumbnail-1.png`\n- `thumbnail-2.png`\n\n\
             ## Steps\n\n\
             1. Upload `long.mp4` in YouTube Studio, and paste the title and the description.\n\
             2. Under Subtitles, upload `long.en.srt`, with timing.\n\
             3. Under Thumbnail, choose Test & compare and upload the 2 thumbnails, `thumbnail-1.png` first. It is on a computer only, and needs advanced features; YouTube keeps the one that is watched longest.\n\
             4. Add an end screen over the last 8 seconds, where the last picture is held for it.\n\
             5. Under Show more, paste the tags.\n"
        );
        assert_eq!(up.problems(60.0), Vec::<String>::new());
        // An end screen needs a video of 25 seconds.
        assert_eq!(
            up.problems(20.0),
            [
                "it holds its last picture 8 s for an end screen, and YouTube puts one only on a video of 25 s or more"
            ]
        );
    }

    #[test]
    fn each_app_gets_what_it_asks_for() {
        let p = project();
        let none: [Chapter; 0] = [];
        let files = (
            "vertical.mp4".to_string(),
            Vec::new(),
            vec!["cover.png".to_string()],
        );
        let profile = |id: &str| platforms::get(id).unwrap();
        // A Short has its own title, and names the long cut as its related video.
        let shorts = profile("youtube-shorts");
        let text = upload(&p, 1, &shorts, &none, &files, false).metadata();
        assert!(text.contains("## Title\n\nvidni in a minute\n"), "{text}");
        assert!(text.contains("set this Short's related video to the `long` cut: in Studio, Content, this Short, Related Video."), "{text}");
        // A reel has a caption with five tags at most, and a cover.
        let reels = profile("instagram-reels");
        let reel = upload(&p, 1, &reels, &none, &files, true);
        let text = reel.metadata();
        assert!(
            !text.contains("## Title") && !text.contains("## Tags"),
            "{text}"
        );
        assert!(
            text.contains("## Caption\n\nA short tour of vidni."),
            "{text}"
        );
        assert!(
            text.contains("\n\n#vidni #tutorialvideos #rust #opensource #videotools\n"),
            "{text}"
        );
        assert!(text.contains("Under Edit cover, add `cover.png`"), "{text}");
        // A synthetic voice is said in the caption, and Meta's label asked for.
        assert!(
            text.contains("The narration is a synthetic voice.\n\n#vidni"),
            "{text}"
        );
        assert!(text.contains("Turn on Add AI label"), "{text}");
        assert!(
            !upload(&p, 1, &reels, &none, &files, false)
                .metadata()
                .contains("synthetic")
        );
        // A status is the video and a caption.
        let status = profile("whatsapp-status");
        let text = upload(&p, 1, &status, &none, &files, false).metadata();
        assert!(
            text.contains(
                "1. Share `vertical.mp4` as a status, with the description as its caption.\n"
            ),
            "{text}"
        );
        // A caption over Instagram's limit is a problem.
        let mut long = p.clone();
        long.publish.description = "word ".repeat(500);
        let wordy = upload(&long, 1, &reels, &none, &files, false);
        assert_eq!(
            wordy.problems(30.0),
            [
                "its description for instagram-reels is 2600 characters, and Instagram Reels takes 2200"
            ]
        );
        // With nothing written, the file says where to write it.
        let mut bare = p.clone();
        bare.publish = Default::default();
        let text = upload(&bare, 0, &profile("youtube"), &none, &files, false).metadata();
        assert!(
            text.contains("(none: `[publish] description` in `vidni.toml` is where it is written)"),
            "{text}"
        );
        assert_eq!(title(&bare, "youtube-shorts"), "My first video");
    }

    #[test]
    fn tags_are_counted_as_youtube_counts_them() {
        let tags = |tags: &[&str]| tags.iter().map(|t| t.to_string()).collect::<Vec<_>>();
        // Five letters, a comma, and nine letters in quotation marks.
        assert_eq!(tags_length(&tags(&["vidni", "two words"])), 5 + 1 + 9 + 2);
        assert_eq!(tags_length(&[]), 0);
        assert_eq!(hashtag("open source!"), "#opensource");
    }
}
