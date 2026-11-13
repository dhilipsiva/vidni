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
    fn clocks() {
        assert_eq!(clock(43200), "12:00:00");
        assert_eq!(clock(180), "3:00");
        assert_eq!(clock(140), "2:20");
    }
}
