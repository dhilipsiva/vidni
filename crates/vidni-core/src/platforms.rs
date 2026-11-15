//! Platform profiles, as data (`platforms.toml`).
//!
//! Each profile says what a platform accepts and recommends for an upload,
//! with the date its facts were last checked and where they come from.
//! Platforms change their limits without notice, so every value has a source
//! and a `verified_on` date, and the docs ask creators to confirm a limit in
//! the app before relying on it.

use crate::project::Aspect;
use serde::{Deserialize, Serialize};

/// The profiles' data, embedded in vidni.
pub const DATA: &str = include_str!("platforms.toml");

/// Every profile, in the order of `platforms.toml`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profiles {
    pub platform: Vec<Profile>,
}

/// What one platform accepts and recommends.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    /// The ID cuts name in `platforms`.
    pub id: String,
    pub name: String,
    /// Whether a cut of this profile's aspect uploads here unless it says otherwise (DECISIONS.md, D6).
    pub enabled_by_default: bool,
    /// The aspect vidni renders for this platform.
    pub aspect: Aspect,
    /// The resolution vidni renders, in pixels.
    pub resolution: [u32; 2],
    /// The longest upload the platform takes, in seconds, for the usual
    /// account; `notes` gives the other limits.
    pub max_upload: Option<u32>,
    /// The longest the platform recommends or distributes fully, in seconds,
    /// when it says so.
    #[serde(default)]
    pub max_recommended: Option<u32>,
    /// The largest file the usual upload path takes, in megabytes.
    #[serde(default)]
    pub max_size_mb: Option<u32>,
    pub loudness: Loudness,
    pub captions: CaptionStyle,
    /// Whether the platform shows chapters, read from a list of times
    /// and titles in the description.
    #[serde(default)]
    pub chapters: bool,
    pub safe_zone: SafeZone,
    /// The day these facts were last checked, YYYY-MM-DD.
    pub verified_on: String,
    /// Where they come from, most authoritative first.
    pub sources: Vec<String>,
    /// Caveats: other account types, upload paths, regions.
    #[serde(default)]
    pub notes: Vec<String>,
}

/// The loudness vidni masters a cut to for the platform.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Loudness {
    /// Integrated loudness, in LUFS.
    pub integrated: f64,
    /// The highest true peak, in dBTP.
    pub true_peak: f64,
    /// `official` (the platform publishes it), `measured` (independent
    /// measurements of the platform's normalization) or `default` (the
    /// platform says nothing; vidni's own default).
    pub basis: Basis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Basis {
    Official,
    Measured,
    Default,
}

/// How captions reach viewers on the platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CaptionStyle {
    /// A caption file (SRT or VTT) is uploaded with the video.
    File,
    /// The platform takes no caption file, so captions are drawn into the picture.
    Burned,
}

/// The parts of the frame the app's overlays cover, as fractions of its
/// height (top, bottom) and width (left, right).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SafeZone {
    pub top: f64,
    pub bottom: f64,
    pub left: f64,
    pub right: f64,
    /// Where the figures come from: `official` guidance, a `measured` estimate, or `none`.
    pub basis: String,
}

/// Every profile.
pub fn all() -> Vec<Profile> {
    toml::from_str::<Profiles>(DATA)
        .expect("platforms.toml is checked by the tests")
        .platform
}

/// The profile with `id`.
pub fn get(id: &str) -> Option<Profile> {
    all().into_iter().find(|p| p.id == id)
}

/// The IDs a cut of `aspect` uploads to unless it names its own.
pub fn defaults(aspect: Aspect) -> Vec<String> {
    all()
        .into_iter()
        .filter(|p| p.enabled_by_default && p.aspect == aspect)
        .map(|p| p.id)
        .collect()
}

/// The edges of a frame that apps cover, in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Insets {
    pub top: u32,
    pub bottom: u32,
    pub left: u32,
    pub right: u32,
}

impl Insets {
    /// `zone` on a frame of `width` by `height` pixels. A pixel that an
    /// app covers in part is covered.
    pub fn of(zone: &SafeZone, [width, height]: [u32; 2]) -> Self {
        // A share that is a whole number of pixels stays that number,
        // whatever its last binary digit.
        let px = |share: f64, size: u32| (share * f64::from(size) - 1e-6).ceil().max(0.0) as u32;
        Self {
            top: px(zone.top, height),
            bottom: px(zone.bottom, height),
            left: px(zone.left, width),
            right: px(zone.right, width),
        }
    }

    /// The edges that either covers.
    pub fn with(self, other: Self) -> Self {
        Self {
            top: self.top.max(other.top),
            bottom: self.bottom.max(other.bottom),
            left: self.left.max(other.left),
            right: self.right.max(other.right),
        }
    }

    /// Whether no edge is covered.
    pub fn is_none(&self) -> bool {
        (self.top, self.bottom, self.left, self.right) == (0, 0, 0, 0)
    }
}

/// What the apps of `platforms` cover together on a frame of `size`: the
/// widest of their safe zones at each edge. `None` when they cover nothing,
/// or none of them is a profile.
pub fn insets(platforms: &[String], size: [u32; 2]) -> Option<Insets> {
    let profiles = all();
    platforms
        .iter()
        .filter_map(|id| profiles.iter().find(|p| &p.id == id))
        .map(|p| Insets::of(&p.safe_zone, size))
        .reduce(Insets::with)
        .filter(|covered| !covered.is_none())
}

/// The loudness a cut for `platforms` is mastered to: the quietest of
/// their targets, in LUFS, so that none turns it down. `None` for a cut
/// made for no platform.
pub fn loudness(platforms: &[String]) -> Option<f64> {
    let profiles = all();
    platforms
        .iter()
        .filter_map(|id| profiles.iter().find(|p| &p.id == id))
        .map(|p| p.loudness.integrated)
        .min_by(f64::total_cmp)
}

/// A file to upload, as measured (TODO.md, P6-VIDNI-05).
#[derive(Debug, Clone, PartialEq)]
pub struct Upload {
    pub aspect: Aspect,
    /// The picture's width and height, in pixels.
    pub size: [u32; 2],
    /// How long it lasts, in seconds, and how large its file is, in bytes.
    pub duration: f64,
    pub bytes: u64,
    /// Its video codec, that codec's profile and its pixel format, as
    /// ffprobe names them; and its audio codec.
    pub video: [String; 3],
    pub audio: String,
    /// Its integrated loudness, in LUFS, and its true peak, in dBTP.
    pub loudness: Option<f64>,
    pub true_peak: Option<f64>,
}

/// How far a file's loudness may be from a platform's, in LU.
pub const LOUDNESS: f64 = 1.0;

impl Profile {
    /// Why the platform wouldn't take `upload`, or would play it wrong:
    /// each a reason not to export it. Empty when it would do.
    pub fn refusals(&self, upload: &Upload) -> Vec<String> {
        let mut found = Vec::new();
        let shape = |aspect: Aspect| match aspect {
            Aspect::Landscape => "16:9",
            Aspect::Portrait => "9:16",
        };
        if upload.aspect != self.aspect {
            found.push(format!(
                "it is {}, and {} takes {}",
                shape(upload.aspect),
                self.name,
                shape(self.aspect)
            ));
        }
        let ([w, h], [least_w, least_h]) = (upload.size, self.resolution);
        if upload.aspect == self.aspect && (w < least_w || h < least_h) {
            found.push(format!(
                "it is {w}x{h}, smaller than the {least_w}x{least_h} that {} is made for",
                self.name
            ));
        }
        if let Some(most) = self.max_upload
            && upload.duration > f64::from(most)
        {
            found.push(format!(
                "it lasts {}, and {} takes {}",
                clock(upload.duration.ceil() as u32),
                self.name,
                clock(most)
            ));
        }
        if let Some(most) = self.max_size_mb
            && upload.bytes > u64::from(most) * 1_000_000
        {
            found.push(format!(
                "its file is {:.0} MB, and {} takes {most} MB",
                upload.bytes as f64 / 1e6,
                self.name
            ));
        }
        let [codec, profile, pixels] = &upload.video;
        if codec != "h264" || !matches!(profile.as_str(), "High" | "Main") || pixels != "yuv420p" {
            found.push(format!(
                "its video is {codec} {profile} {pixels}, not H.264 High in yuv420p, which every platform plays"
            ));
        }
        if upload.audio != "aac" {
            found.push(format!("its audio is {}, not AAC", upload.audio));
        }
        match upload.loudness {
            Some(l) if (l - self.loudness.integrated).abs() <= LOUDNESS => {}
            Some(l) => found.push(format!(
                "its loudness is {l} LUFS, more than {LOUDNESS} LU from the {} LUFS that {} plays at",
                self.loudness.integrated, self.name
            )),
            None => found.push("its loudness couldn't be measured".to_string()),
        }
        match upload.true_peak {
            Some(peak) if peak <= self.loudness.true_peak => {}
            Some(peak) => found.push(format!(
                "its true peak is {peak} dBTP, above {} dBTP",
                self.loudness.true_peak
            )),
            None => found.push("its true peak couldn't be measured".to_string()),
        }
        found
    }

    /// What the platform takes of `upload` and doesn't do its best by.
    pub fn advice(&self, upload: &Upload) -> Vec<String> {
        let mut found = Vec::new();
        if let Some(best) = self.max_recommended
            && upload.duration > f64::from(best)
            && self
                .max_upload
                .is_none_or(|most| upload.duration <= f64::from(most))
        {
            found.push(format!(
                "it lasts {}, and {} recommends {} or less",
                clock(upload.duration.ceil() as u32),
                self.name,
                clock(best)
            ));
        }
        let ([w, h], [best_w, best_h]) = (upload.size, self.resolution);
        if upload.aspect == self.aspect && (w > best_w || h > best_h) {
            found.push(format!(
                "it is {w}x{h}, and {} recommends {best_w}x{best_h}",
                self.name
            ));
        }
        found
    }
}

/// `seconds` as `h:mm:ss` or `m:ss`.
pub fn clock(seconds: u32) -> String {
    let (h, m, s) = (seconds / 3600, seconds / 60 % 60, seconds % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_profile_is_complete_and_sourced() {
        let profiles = all();
        let ids: Vec<&str> = profiles.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "youtube",
                "youtube-shorts",
                "instagram-reels",
                "tiktok",
                "whatsapp-status",
                "linkedin",
                "x"
            ]
        );
        for p in &profiles {
            assert!(crate::validate::valid_id(&p.id), "{}", p.id);
            assert!(
                !p.sources.is_empty() && p.sources.iter().all(|s| s.starts_with("https://")),
                "{}",
                p.id
            );
            let date = p.verified_on.as_bytes();
            assert!(
                date.len() == 10 && date[4] == b'-' && date[7] == b'-',
                "{}: {}",
                p.id,
                p.verified_on
            );
            let [w, h] = p.resolution;
            assert_eq!(w > h, p.aspect == Aspect::Landscape, "{}", p.id);
            if let (Some(max), Some(recommended)) = (p.max_upload, p.max_recommended) {
                assert!(recommended <= max, "{}", p.id);
            }
            let z = &p.safe_zone;
            assert!(z.top + z.bottom < 1.0 && z.left + z.right < 1.0, "{}", p.id);
            assert!(
                p.loudness.integrated < 0.0 && p.loudness.true_peak <= 0.0,
                "{}",
                p.id
            );
        }
    }

    #[test]
    fn the_default_platforms_are_the_ruled_ones() {
        assert_eq!(defaults(Aspect::Landscape), ["youtube"]);
        assert_eq!(
            defaults(Aspect::Portrait),
            ["youtube-shorts", "instagram-reels", "whatsapp-status"]
        );
    }

    #[test]
    fn the_default_vertical_apps_cover_the_widest_zone_at_each_edge() {
        let ids = |ids: &[&str]| ids.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        // Shorts: 288 px top, 672 bottom, 48 left, 192 right. Reels: 14 %, 35 %, 6 %, 6 %.
        assert_eq!(
            insets(&defaults(Aspect::Portrait), [1080, 1920]),
            Some(Insets {
                top: 288,
                bottom: 672,
                left: 65,
                right: 193
            })
        );
        assert_eq!(
            insets(&ids(&["instagram-reels"]), [1080, 1920]),
            Some(Insets {
                top: 269,
                bottom: 672,
                left: 65,
                right: 119
            })
        );
        // WhatsApp's are measured: its name row and its button for more.
        assert_eq!(
            insets(&ids(&["whatsapp-status"]), [1080, 1920]),
            Some(Insets {
                top: 58,
                bottom: 135,
                left: 0,
                right: 0
            })
        );
        // No platform, or one that covers nothing, leaves the frame free.
        assert_eq!(insets(&[], [1080, 1920]), None);
        assert_eq!(insets(&ids(&["youtube"]), [1920, 1080]), None);
    }

    #[test]
    fn a_broken_upload_fails_each_check_and_a_good_one_none() {
        let shorts = get("youtube-shorts").unwrap();
        let good = Upload {
            aspect: Aspect::Portrait,
            size: [1080, 1920],
            duration: 54.0,
            bytes: 8_000_000,
            video: ["h264".into(), "High".into(), "yuv420p".into()],
            audio: "aac".into(),
            loudness: Some(-14.3),
            true_peak: Some(-1.2),
        };
        assert_eq!(shorts.refusals(&good), Vec::<String>::new());
        assert_eq!(shorts.advice(&good), Vec::<String>::new());
        // One thing wrong at a time, and the reason it gives.
        let broken: [(Upload, &str); 9] = [
            (
                Upload {
                    aspect: Aspect::Landscape,
                    size: [1920, 1080],
                    ..good.clone()
                },
                "it is 16:9, and YouTube Shorts takes 9:16",
            ),
            (
                Upload {
                    size: [720, 1280],
                    ..good.clone()
                },
                "it is 720x1280, smaller than the 1080x1920 that YouTube Shorts is made for",
            ),
            (
                Upload {
                    duration: 200.5,
                    ..good.clone()
                },
                "it lasts 3:21, and YouTube Shorts takes 3:00",
            ),
            (
                Upload {
                    bytes: 300_000_000_000,
                    ..good.clone()
                },
                "its file is 300000 MB, and YouTube Shorts takes 256000 MB",
            ),
            (
                Upload {
                    video: ["hevc".into(), "Main".into(), "yuv420p".into()],
                    ..good.clone()
                },
                "its video is hevc Main yuv420p, not H.264 High in yuv420p, which every platform plays",
            ),
            (
                Upload {
                    video: [
                        "h264".into(),
                        "High 4:4:4 Predictive".into(),
                        "yuv444p".into(),
                    ],
                    ..good.clone()
                },
                "its video is h264 High 4:4:4 Predictive yuv444p, not H.264 High in yuv420p, which every platform plays",
            ),
            (
                Upload {
                    audio: "mp3".into(),
                    ..good.clone()
                },
                "its audio is mp3, not AAC",
            ),
            (
                Upload {
                    loudness: Some(-16.5),
                    ..good.clone()
                },
                "its loudness is -16.5 LUFS, more than 1 LU from the -14 LUFS that YouTube Shorts plays at",
            ),
            (
                Upload {
                    true_peak: Some(-0.3),
                    ..good.clone()
                },
                "its true peak is -0.3 dBTP, above -1 dBTP",
            ),
        ];
        for (upload, reason) in broken {
            assert_eq!(shorts.refusals(&upload), [reason]);
        }
        // What a platform takes but doesn't recommend is advice, not a refusal.
        let reels = get("instagram-reels").unwrap();
        let long = Upload {
            duration: 200.0,
            ..good.clone()
        };
        assert_eq!(reels.refusals(&long), Vec::<String>::new());
        assert_eq!(
            reels.advice(&long),
            ["it lasts 3:20, and Instagram Reels recommends 3:00 or less"]
        );
        let x = get("x").unwrap();
        let wide = Upload {
            aspect: Aspect::Landscape,
            size: [1920, 1080],
            duration: 100.0,
            ..good.clone()
        };
        assert_eq!(x.refusals(&wide), Vec::<String>::new());
        assert_eq!(
            x.advice(&wide),
            ["it is 1920x1080, and X recommends 1280x720"]
        );
        // The quietest platform sets a cut's loudness.
        assert_eq!(loudness(&defaults(Aspect::Portrait)), Some(-14.0));
        assert_eq!(loudness(&[]), None);
    }

    #[test]
    fn clocks() {
        assert_eq!(clock(43200), "12:00:00");
        assert_eq!(clock(180), "3:00");
        assert_eq!(clock(140), "2:20");
    }
}
