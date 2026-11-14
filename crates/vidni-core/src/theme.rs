//! Themes (TODO.md, P5-VIDNI-01): the look of a cut as a file of tokens.
//!
//! A theme says which colours and which fonts the compositor draws with:
//! the frame behind a 9:16 cut's view, the caption box, the section label,
//! and the cards. Where things sit and how big they are is the
//! compositor's, the same in every theme. `[style] theme` names a theme:
//! `default`, which is built in, or a folder that holds a `theme.toml`.

use crate::fonts;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// The built-in theme's file.
pub const DEFAULT: &str = include_str!("themes/default.toml");
/// The name of a theme's file in its folder.
pub const FILE: &str = "theme.toml";

/// A theme's tokens.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Theme {
    pub name: String,
    pub colors: Colors,
    pub fonts: Faces,
    #[serde(default)]
    pub label: Label,
}

/// A theme's colours, each as `#rrggbb`, or `#rrggbbaa` for the two boxes.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Colors {
    /// Behind everything: cards, and the bands around a 9:16 cut's view.
    pub frame: String,
    /// The rules between the bands.
    pub rule: String,
    /// Text, and text that matters less.
    pub ink: String,
    pub muted: String,
    /// The mark before a label, and a card's rule.
    pub accent: String,
    /// The box behind captions, the box behind a label, and their edge.
    pub panel: String,
    pub label: String,
    pub edge: String,
}

/// The fonts a theme draws with, each by the ID vidni pins it under.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Faces {
    /// Captions, and a card's running text.
    pub text: String,
    /// Labels and other small type.
    pub mono: String,
    /// Titles.
    pub display: String,
}

/// How a section's label is written.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Label {
    /// Written before the label, in the accent colour. Empty for none.
    #[serde(default)]
    pub mark: String,
}

/// A colour's channels: red, green, blue, and how solid it is.
pub fn rgba(color: &str) -> Option<[u8; 4]> {
    let hex = color.strip_prefix('#')?;
    if !matches!(hex.len(), 6 | 8) || !hex.is_ascii() {
        return None;
    }
    let channel = |i: usize| u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).ok();
    Some([
        channel(0)?,
        channel(1)?,
        channel(2)?,
        if hex.len() == 8 { channel(3)? } else { 255 },
    ])
}

impl Theme {
    /// Reads a theme's file, and checks every token.
    pub fn parse(text: &str) -> Result<Self, String> {
        let theme: Theme = toml::from_str(text).map_err(|e| e.message().to_string())?;
        let colors = &theme.colors;
        for (name, color, solid) in [
            ("frame", &colors.frame, true),
            ("rule", &colors.rule, true),
            ("ink", &colors.ink, true),
            ("muted", &colors.muted, true),
            ("accent", &colors.accent, true),
            ("panel", &colors.panel, false),
            ("label", &colors.label, false),
            ("edge", &colors.edge, true),
        ] {
            match rgba(color) {
                None => {
                    return Err(format!(
                        "colors.{name}: {color:?} isn't a colour: #rrggbb, or #rrggbbaa"
                    ));
                }
                Some([_, _, _, alpha]) if solid && alpha != 255 => {
                    return Err(format!("colors.{name}: {color:?} must be solid: #rrggbb"));
                }
                Some(_) => {}
            }
        }
        for (name, id) in [
            ("text", &theme.fonts.text),
            ("mono", &theme.fonts.mono),
            ("display", &theme.fonts.display),
        ] {
            if fonts::get(id).is_none() {
                let known: Vec<String> = fonts::all().into_iter().map(|f| f.id).collect();
                return Err(format!(
                    "fonts.{name}: {id:?} isn't a font vidni pins: {}",
                    known.join(", ")
                ));
            }
        }
        Ok(theme)
    }

    /// The built-in theme.
    pub fn default_theme() -> Self {
        Self::parse(DEFAULT).expect("the built-in theme is checked by the tests")
    }
}

/// The theme `[style] theme` names: `default`, or a folder, from the
/// project folder, that holds a `theme.toml`.
pub fn load(project_dir: &Path, named: &str) -> Result<Theme, String> {
    if named == "default" {
        return Ok(Theme::default_theme());
    }
    let file = project_dir.join(named).join(FILE);
    let text = fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
    Theme::parse(&text).map_err(|e| format!("{}: {e}", file.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_built_in_theme_parses_and_its_colours_read() {
        let theme = Theme::default_theme();
        assert_eq!(theme.name, "default");
        assert_eq!(rgba(&theme.colors.frame), Some([16, 16, 20, 255]));
        assert_eq!(rgba(&theme.colors.panel), Some([8, 8, 12, 222]));
        assert_eq!(theme.label.mark, "");
        for odd in ["101014", "#10101", "#1010zz", "#fffffffff", "#ééé"] {
            assert_eq!(rgba(odd), None, "{odd}");
        }
    }

    #[test]
    fn a_theme_is_read_from_its_folder_and_checked() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(load(dir.path(), "default").unwrap(), Theme::default_theme());
        let folder = dir.path().join("themes/mine");
        fs::create_dir_all(&folder).unwrap();
        let mine = DEFAULT
            .replace("name = \"default\"", "name = \"mine\"")
            .replace("mark = \"\"", "mark = \"//\"");
        fs::write(folder.join(FILE), &mine).unwrap();
        let theme = load(dir.path(), "themes/mine").unwrap();
        assert_eq!(
            (theme.name.as_str(), theme.label.mark.as_str()),
            ("mine", "//")
        );
        // Each token is checked, and the file is named.
        for (old, new, problem) in [
            (
                "frame = \"#101014\"",
                "frame = \"black\"",
                "colors.frame: \"black\" isn't a colour",
            ),
            (
                "ink = \"#f5f4f0\"",
                "ink = \"#f5f4f080\"",
                "colors.ink: \"#f5f4f080\" must be solid",
            ),
            (
                "display = \"ibm-plex-sans-bold\"",
                "display = \"comic-sans\"",
                "fonts.display: \"comic-sans\" isn't a font vidni pins: ibm-plex-sans-bold, ",
            ),
            ("[label]", "[labels]", "unknown field `labels`"),
        ] {
            fs::write(folder.join(FILE), mine.replace(old, new)).unwrap();
            let error = load(dir.path(), "themes/mine").unwrap_err();
            assert!(
                error.contains("themes/mine/theme.toml: ") && error.contains(problem),
                "{error}"
            );
        }
        assert!(
            load(dir.path(), "themes/gone")
                .unwrap_err()
                .contains("themes/gone/theme.toml")
        );
    }
}
