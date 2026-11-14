//! Schema 1 of `vidni.toml`, the project file.
//!
//! `docs/spec.md` describes every table and key, and `docs/vidni.schema.json`
//! is generated from these types (`vidni schema`). Most structs are flat on
//! purpose: a key that belongs to another kind of scene or action is caught
//! by [`crate::validate`] at its own line and column, which serde's tagged
//! enums can't do.

use schemars::JsonSchema;
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;
use std::fmt;

/// The schema version this vidni reads.
pub const SCHEMA: u32 = 1;

/// One video: its script, what each line shows, and the cuts made from it.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Project {
    /// The schema version: 1.
    pub schema: u32,
    /// What the video is.
    pub meta: Meta,
    /// The voices that speak the lines.
    #[serde(default)]
    pub voice: Voice,
    /// The pauses around the lines.
    #[serde(default)]
    pub timing: Timing,
    /// Wording checks.
    #[serde(default)]
    pub lint: Lint,
    /// The look of every cut.
    #[serde(default)]
    pub style: Style,
    /// The sections, by ID. They come in the order of their first line.
    #[serde(default)]
    pub sections: BTreeMap<String, Section>,
    /// The script: every narrated line, in order.
    pub lines: Vec<Line>,
    /// What the lines show, by ID.
    #[serde(default)]
    pub scenes: BTreeMap<String, Scene>,
    /// The videos made from the script.
    pub cuts: Vec<Cut>,
    /// Background music.
    #[serde(default)]
    pub music: Music,
    /// What goes with the video where it is uploaded: its description,
    /// tags, links, titles and thumbnails.
    #[serde(default, skip_serializing_if = "Publish::is_empty")]
    pub publish: Publish,
    /// Text that must be on screen when a scene is captured.
    #[serde(default)]
    pub expectations: Vec<Expectation>,
}

/// What goes with the video where it is uploaded (TODO.md, P6-VIDNI-04).
/// None of it is in a picture, so changing it renders nothing again.
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Publish {
    /// The description: what the video is about, in your own words.
    /// Chapters, links and credits are added after it.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Tags, most telling first.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Links to list under the description.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<Link>,
    /// The title on a platform, by profile ID, where it isn't `meta.title`.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub titles: BTreeMap<String, String>,
    /// Up to three thumbnails for a 16:9 cut, each a frame of a line.
    /// With none, there is one: the cut's first line, with the title.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub thumbnails: Vec<Thumbnail>,
}

impl Publish {
    /// Whether the project says nothing of it.
    pub fn is_empty(&self) -> bool {
        self == &Self::default()
    }
}

/// A link under a description.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Link {
    /// What the link is, such as `The code`.
    pub label: String,
    /// Where it goes: an `http` or `https` address.
    pub url: String,
}

/// A thumbnail: the picture of a line, with words over it.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Thumbnail {
    /// The line whose picture it is.
    pub line: String,
    /// The words drawn over the picture. Default: none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// What the video is.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Meta {
    /// The video's title.
    pub title: String,
    /// A short line shown with the title, such as a series name.
    #[serde(default)]
    pub subtitle: Option<String>,
    /// The script's language, as a BCP 47 tag.
    #[serde(default = "default_language")]
    pub language: String,
    /// Whether every factual line must name its source (P2-VIDNI-06).
    #[serde(default)]
    pub strict: bool,
    /// The scene a line shows when it names none.
    #[serde(default)]
    pub default_scene: Option<String>,
}

fn default_language() -> String {
    "en".into()
}

/// The voices that speak the lines.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Voice {
    /// The synthetic draft voice: a voice pack, as `<engine>:<model>`.
    pub draft: String,
    /// Where a line's first draft takes its seeds from. The seed it keeps
    /// goes into `voice.lock`, so changing this changes only the lines
    /// drafted afresh.
    pub seed: u64,
    /// The draft voice's pace: above 1 is slower.
    pub length_scale: f64,
    /// How a reader says symbols that the captions show. The prompter shows
    /// these, and takes are scored against them.
    pub read_as: Vec<Respelling>,
    /// Spellings the draft voice says better, applied in order to what it speaks.
    pub respell: Vec<Respelling>,
    /// Words the listening check hears spelled differently from the script
    /// for the same sound. They make its scores fairer and change nothing
    /// else: no output depends on what the check hears.
    pub heard_as: Vec<HeardAs>,
}

impl Default for Voice {
    fn default() -> Self {
        Self {
            draft: "piper:en_US-joe-medium".into(),
            seed: 0,
            length_scale: 1.0,
            read_as: Vec::new(),
            respell: Vec::new(),
            heard_as: Vec::new(),
        }
    }
}

/// One text and how it is said.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Respelling {
    /// The text as written.
    pub text: String,
    /// How it is said.
    pub say: String,
}

/// One word as the listening check hears it, and the script's word for
/// the same sound.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HeardAs {
    /// The word as whisper writes it. Case is ignored.
    pub heard: String,
    /// The script's word for it.
    pub means: String,
}

/// The pauses around the lines, in seconds.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Timing {
    /// Silence before the first line.
    pub lead_in: f64,
    /// The pause between two lines of one section.
    pub line_gap: f64,
    /// The pause between sections.
    pub section_gap: f64,
    /// Silence after the last line.
    pub tail: f64,
}

impl Default for Timing {
    fn default() -> Self {
        Self {
            lead_in: 0.6,
            line_gap: 0.28,
            section_gap: 0.75,
            tail: 2.4,
        }
    }
}

/// Wording checks.
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Lint {
    /// Regular expressions, matched without regard to case, that no line may
    /// show or say.
    pub banned: Vec<String>,
}

/// The look of every cut.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Style {
    /// `default`, or the path of a theme folder.
    pub theme: String,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            theme: "default".into(),
        }
    }
}

/// A run of lines under one label.
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Section {
    /// Shown on screen while the section plays, and used for chapters. Empty
    /// for none.
    pub label: String,
    /// The section's title in a chapter list, when it isn't the label.
    /// Empty to stay in the chapter before.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapter: Option<String>,
}

/// One narrated line.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Line {
    /// A stable ID: lowercase letters and digits in words joined by hyphens.
    pub id: String,
    /// The section the line belongs to.
    pub section: String,
    /// The caption: the line as it is shown.
    pub show: String,
    /// What is said, when it differs from what is shown.
    #[serde(default)]
    pub say: Option<String>,
    /// Where the line's facts come from.
    #[serde(default)]
    pub source: Option<String>,
    /// Whether the line states a fact. In strict projects, every line that
    /// does needs a `source`; set `claim = false` on a line that states none,
    /// such as a greeting or a transition.
    #[serde(default)]
    pub claim: Option<bool>,
    /// The scene the line shows; `meta.default_scene` when absent.
    #[serde(default)]
    pub scene: Option<String>,
    /// Actions on a web scene before the line is captured.
    #[serde(default, rename = "do")]
    pub actions: Vec<Action>,
    /// Targets of a web scene to box while the line plays.
    #[serde(default)]
    pub highlight: Vec<String>,
    /// The web scene's camera for landscape cuts; the scene's
    /// `default_camera` when absent.
    #[serde(default)]
    pub camera: Option<String>,
    /// The camera for vertical cuts, when it differs.
    #[serde(default)]
    pub camera_vertical: Option<String>,
    /// A page scene drawn over the line's picture.
    #[serde(default)]
    pub overlay: Option<String>,
    /// The cuts the line is only for, such as a short's hook. A cut that
    /// uses every line leaves out the lines that are only for others.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub only: Vec<String>,
}

impl Project {
    /// The files in the project folder that the scenes name, each once, in
    /// order: a page scene's file for both orientations, and a web scene's
    /// when its address is a file.
    pub fn scene_files(&self) -> Vec<String> {
        let mut files: Vec<String> = Vec::new();
        for scene in self.scenes.values() {
            let Some(file) = scene.file() else {
                continue;
            };
            for aspect in ["16x9", "9x16"] {
                let name = file.replace("{aspect}", aspect);
                if !files.contains(&name) {
                    files.push(name);
                }
            }
        }
        files
    }
}

impl Scene {
    /// Whether the scene's address is on the web, rather than a file in
    /// the project folder.
    pub fn on_the_web(&self) -> bool {
        self.url.starts_with("http://") || self.url.starts_with("https://")
    }

    /// The file in the project folder that the scene's address names,
    /// without its query and fragment, and with `{aspect}` as written.
    /// None for an address on the web.
    pub fn file(&self) -> Option<&str> {
        match self.on_the_web() || self.kind == SceneKind::Card {
            true => None,
            false => self.url.split(['?', '#']).next(),
        }
    }
}

impl Line {
    /// Whether `cut` may use the line: it is for every cut, or for this one.
    pub fn is_for(&self, cut: &Cut) -> bool {
        self.only.is_empty() || self.only.contains(&cut.id)
    }

    /// The scene the line shows: its own, or the project's default.
    pub fn scene_in<'a>(&'a self, project: &'a Project) -> Option<&'a str> {
        self.scene
            .as_deref()
            .or(project.meta.default_scene.as_deref())
    }
}

/// What lines show.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Scene {
    /// `web`: a page captured at a fixed viewport and framed by the virtual
    /// camera. `page`: an HTML file rendered at each cut's own size, for
    /// cards and overlays. `card`: a card drawn from one of the theme's
    /// templates. `image`: a picture in the project folder, PNG or SVG,
    /// framed by the virtual camera.
    pub kind: SceneKind,
    /// web: the page's address. page: a file in the project folder;
    /// `{aspect}` becomes `16x9` or `9x16`. image: the picture's file.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub url: String,
    /// card: the template it is drawn from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<Template>,
    /// card: its title (`title`, `section`, `bullets`, `end`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// card: a second, smaller line (`title`, `section`, `end`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    /// card: the points of a `bullets` card.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<String>,
    /// card: the words of a `quote` card.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote: Option<String>,
    /// card: whose words a `quote` card's are.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub by: Option<String>,
    /// web: the viewport in CSS pixels. Default `[1280, 720]`.
    #[serde(default)]
    pub viewport: Option<[u32; 2]>,
    /// web: device pixels per CSS pixel. Default 1.
    #[serde(default)]
    pub scale: Option<u32>,
    /// web: the colour scheme the page is asked for.
    #[serde(default)]
    pub color_scheme: Option<ColorScheme>,
    /// web: actions run once, after the page loads.
    #[serde(default)]
    pub setup: Vec<Action>,
    /// web: named parts of the page, for highlights, cameras, actions and
    /// expectations.
    #[serde(default)]
    pub targets: BTreeMap<String, Target>,
    /// web: named framings. `viewport` and `highlights` are always defined.
    #[serde(default)]
    pub cameras: BTreeMap<String, Camera>,
    /// web: the camera of a line that names none. Default `viewport`.
    #[serde(default)]
    pub default_camera: Option<String>,
    /// web: in vertical cuts, a line whose highlights are at most this many
    /// CSS pixels wide is framed on its highlights.
    #[serde(default)]
    pub vertical_follows_highlights: Option<f64>,
    /// page: rendered on a transparent background, as an overlay.
    #[serde(default)]
    pub transparent: Option<bool>,
    /// page: the rendered size for each orientation. Default: the cut's size.
    #[serde(default)]
    pub size: Option<PageSize>,
}

/// The kinds of scene in schema 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SceneKind {
    Web,
    Page,
    Card,
    Image,
}

/// The templates a card scene is drawn from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Template {
    /// A title and a subtitle: a video's opening card.
    Title,
    /// A section's name, with a line above it.
    Section,
    /// A title and a list of points.
    Bullets,
    /// A quotation, and whose it is.
    Quote,
    /// A closing card: a title and a line under it, centred.
    End,
}

/// A colour scheme a web scene can ask for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ColorScheme {
    Light,
    Dark,
}

/// A page scene's size for each orientation, in pixels.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PageSize {
    pub landscape: [u32; 2],
    pub portrait: [u32; 2],
}

/// A named part of a web page: the visible elements that match.
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Target {
    /// A CSS selector.
    #[serde(default)]
    pub css: Option<String>,
    /// Keep the elements whose text matches.
    #[serde(default)]
    pub text: Option<TextMatch>,
    /// Match `text` against this child (a CSS selector) instead of the whole element.
    #[serde(default)]
    pub text_of: Option<String>,
    /// Drop the elements whose text matches.
    #[serde(default)]
    pub exclude: Option<TextMatch>,
    /// Keep the elements whose attribute matches.
    #[serde(default)]
    pub attr: Option<AttrMatch>,
    /// Which matches to keep. Default `all`.
    #[serde(default)]
    pub pick: Option<Pick>,
    /// Draw one box around all the matches.
    #[serde(default)]
    pub merge: bool,
    /// One line of the element's text, counted from 0.
    #[serde(default)]
    pub line: Option<u32>,
    /// Grow each match to its largest enclosing block that still ends with the same text.
    #[serde(default)]
    pub grow: Option<Grow>,
    /// When reading texts, keep each text once.
    #[serde(default)]
    pub unique: bool,
    /// The union of other matchers, instead of `css`.
    #[serde(default)]
    pub any: Vec<Target>,
    /// On an image scene: a part of the picture, as `[x, y, width,
    /// height]` in its own pixels (an SVG's, in its own units).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rect: Option<[f64; 4]>,
}

/// Which of a target's matches to keep.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Pick {
    All,
    First,
    Last,
}

/// A text test: a string is an exact match after collapsing whitespace.
#[derive(Debug, Clone, PartialEq)]
pub enum TextMatch {
    Exact(String),
    Rule(TextRule),
}

/// A text test other than an exact match. Exactly one key is set.
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TextRule {
    #[serde(default)]
    pub starts_with: Option<String>,
    #[serde(default)]
    pub contains: Option<String>,
    /// A regular expression.
    #[serde(default)]
    pub matches: Option<String>,
}

impl TextRule {
    /// The keys that are set.
    pub fn keys(&self) -> Vec<&'static str> {
        [
            ("starts_with", self.starts_with.is_some()),
            ("contains", self.contains.is_some()),
            ("matches", self.matches.is_some()),
        ]
        .into_iter()
        .filter_map(|(k, set)| set.then_some(k))
        .collect()
    }
}

impl<'de> Deserialize<'de> for TextMatch {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct TextMatchVisitor;
        impl<'de> Visitor<'de> for TextMatchVisitor {
            type Value = TextMatch;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a string, or a table with one of starts_with, contains or matches")
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<TextMatch, E> {
                Ok(TextMatch::Exact(v.into()))
            }
            fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<TextMatch, A::Error> {
                TextRule::deserialize(de::value::MapAccessDeserializer::new(map))
                    .map(TextMatch::Rule)
            }
        }
        deserializer.deserialize_any(TextMatchVisitor)
    }
}

impl Serialize for TextMatch {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TextMatch::Exact(s) => serializer.serialize_str(s),
            TextMatch::Rule(r) => r.serialize(serializer),
        }
    }
}

impl JsonSchema for TextMatch {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "TextMatch".into()
    }
    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let rule = generator.subschema_for::<TextRule>();
        schemars::json_schema!({
            "description": "A string to match exactly, after collapsing whitespace, or a table with one of starts_with, contains or matches.",
            "oneOf": [{"type": "string"}, rule]
        })
    }
}

/// An attribute test.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AttrMatch {
    /// The attribute's name.
    pub name: String,
    /// A regular expression its value must match.
    pub matches: String,
}

/// How a match grows to its enclosing block.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Grow {
    /// The container the match grows within (a CSS selector).
    pub within: String,
    /// The tallest block it may grow to, in CSS pixels.
    pub max_height: f64,
    /// The block must stay this many CSS pixels narrower than the container.
    #[serde(default)]
    pub margin: f64,
}

/// A named framing of a web scene.
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Camera {
    /// Frame the union of these targets.
    #[serde(default)]
    pub union: Vec<String>,
    /// Frame the whole viewport.
    #[serde(default)]
    pub viewport: bool,
    /// The camera vertical cuts use instead, when this one is too wide.
    #[serde(default)]
    pub vertical: Option<String>,
}

/// One step on a web page. Exactly one verb is set: `click`, `select`,
/// `expand`, `collapse`, `scroll`, `scroll_to` or `wait`, each naming a target.
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Action {
    /// Click the target.
    #[serde(default)]
    pub click: Option<String>,
    /// Choose `value` in the target, a `<select>`.
    #[serde(default)]
    pub select: Option<String>,
    /// The option value for `select`.
    #[serde(default)]
    pub value: Option<String>,
    /// Open the target if it is closed (`aria-expanded`), by clicking `control`.
    #[serde(default)]
    pub expand: Option<String>,
    /// Close the target if it is open, by clicking `control`.
    #[serde(default)]
    pub collapse: Option<String>,
    /// For `expand` and `collapse`: the CSS selector, inside the target, to click.
    #[serde(default)]
    pub control: Option<String>,
    /// Scroll the target, a scroll container, to `top`.
    #[serde(default)]
    pub scroll: Option<String>,
    /// For `scroll`: the scroll position, in CSS pixels.
    #[serde(default)]
    pub top: Option<f64>,
    /// Scroll `within` until the target is at its top, less `offset`.
    #[serde(default)]
    pub scroll_to: Option<String>,
    /// For `scroll_to`: the scroll container, a target.
    #[serde(default)]
    pub within: Option<String>,
    /// For `scroll_to`: the gap left above the target, in CSS pixels.
    #[serde(default)]
    pub offset: Option<f64>,
    /// Wait until the target is visible.
    #[serde(default)]
    pub wait: Option<String>,
}

impl Action {
    /// The verbs that are set, with the target each names.
    pub fn verbs(&self) -> Vec<(&'static str, &str)> {
        [
            ("click", &self.click),
            ("select", &self.select),
            ("expand", &self.expand),
            ("collapse", &self.collapse),
            ("scroll", &self.scroll),
            ("scroll_to", &self.scroll_to),
            ("wait", &self.wait),
        ]
        .into_iter()
        .filter_map(|(verb, target)| target.as_deref().map(|t| (verb, t)))
        .collect()
    }

    /// The modifiers that are set, with the verbs that take each.
    pub fn modifiers(&self) -> Vec<(&'static str, &'static [&'static str])> {
        [
            ("value", self.value.is_some(), &["select"][..]),
            (
                "control",
                self.control.is_some(),
                &["expand", "collapse"][..],
            ),
            ("top", self.top.is_some(), &["scroll"][..]),
            ("within", self.within.is_some(), &["scroll_to"][..]),
            ("offset", self.offset.is_some(), &["scroll_to"][..]),
        ]
        .into_iter()
        .filter_map(|(key, set, verbs)| set.then_some((key, verbs)))
        .collect()
    }
}

/// A video made from the script.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Cut {
    /// A stable ID, like a line's.
    pub id: String,
    /// `16:9` or `9:16`.
    pub aspect: Aspect,
    /// `all`, or the IDs of the lines the cut uses, in script order.
    #[serde(default)]
    pub lines: LineSelection,
    /// `burned`, `file` or `both`. Default: `both` for 16:9, `burned` for 9:16.
    #[serde(default)]
    pub captions: Option<Captions>,
    /// The platform profiles the cut is made for. Default: `youtube` for
    /// 16:9; `youtube-shorts`, `instagram-reels` and `whatsapp-status` for 9:16.
    #[serde(default)]
    pub platforms: Option<Vec<String>>,
    /// The frame size in pixels. Default `[1920, 1080]` or `[1080, 1920]`.
    #[serde(default)]
    pub size: Option<[u32; 2]>,
    /// How long a 16:9 cut holds its last picture after the narration
    /// ends, in seconds from 5 to 20, for an end screen to go over. It
    /// takes the place of `timing.tail` in this cut.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_hold: Option<f64>,
}

/// A cut's aspect ratio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
pub enum Aspect {
    #[serde(rename = "16:9")]
    Landscape,
    #[serde(rename = "9:16")]
    Portrait,
}

/// How a cut carries its captions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Captions {
    /// Drawn into the picture.
    Burned,
    /// A clean picture, with captions as SRT and VTT files.
    File,
    /// A clean master with caption files, and a burned-in variant.
    Both,
}

/// The lines a cut uses.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum LineSelection {
    #[default]
    All,
    Ids(Vec<String>),
}

impl<'de> Deserialize<'de> for LineSelection {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct LinesVisitor;
        impl<'de> Visitor<'de> for LinesVisitor {
            type Value = LineSelection;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("\"all\" or an array of line IDs")
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<LineSelection, E> {
                if v == "all" {
                    Ok(LineSelection::All)
                } else {
                    Err(E::invalid_value(de::Unexpected::Str(v), &self))
                }
            }
            fn visit_seq<A: SeqAccess<'de>>(self, seq: A) -> Result<LineSelection, A::Error> {
                Vec::<String>::deserialize(de::value::SeqAccessDeserializer::new(seq))
                    .map(LineSelection::Ids)
            }
        }
        deserializer.deserialize_any(LinesVisitor)
    }
}

impl Serialize for LineSelection {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            LineSelection::All => serializer.serialize_str("all"),
            LineSelection::Ids(ids) => ids.serialize(serializer),
        }
    }
}

impl JsonSchema for LineSelection {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "LineSelection".into()
    }
    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "description": "\"all\", or the IDs of the lines a cut uses.",
            "oneOf": [{"const": "all"}, {"type": "array", "items": {"type": "string"}}]
        })
    }
}

/// Background music.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Music {
    /// `none`, or the ID of a track in `library`.
    pub track: String,
    /// How far under the narration the track plays, in LU. Default 20.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub under: Option<f64>,
    /// The tracks the project may use, each with its licence.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub library: Vec<Track>,
}

impl Default for Music {
    fn default() -> Self {
        Self {
            track: "none".into(),
            under: None,
            library: Vec::new(),
        }
    }
}

impl Music {
    /// How far under the narration a track plays when the project doesn't say, in LU.
    pub const UNDER: f64 = 20.0;

    /// The track the project plays, when it plays one that its library has.
    pub fn playing(&self) -> Option<&Track> {
        self.library.iter().find(|t| t.id == self.track)
    }

    /// How far under the narration the track plays, in LU.
    pub fn under(&self) -> f64 {
        self.under.unwrap_or(Self::UNDER)
    }
}

/// A piece of music the project may use, with what allows it.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Track {
    /// A stable ID, like a line's.
    pub id: String,
    /// The track's file in the project folder: a 16-bit WAV.
    pub file: String,
    /// The file's SHA-256, so that the licence is of this very file.
    pub sha256: String,
    /// The licence it is used under, such as `CC-BY-4.0`, or the name of
    /// a paid licence.
    pub licence: String,
    /// The credit the licence asks for, as it goes into a description.
    pub attribution: String,
    /// The platforms the licence covers.
    pub platforms: Vec<String>,
    /// Whether the track is registered with YouTube's Content ID. A Short
    /// over a minute with a Content ID claim is blocked worldwide.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub content_id: bool,
}

/// Text a web scene must show, checked when it is captured.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Expectation {
    /// The web scene.
    pub scene: String,
    /// `setup`, or the ID of a line on that scene: checked after its actions.
    pub at: String,
    /// The target whose text is read.
    pub target: String,
    /// The text of the target's single match.
    #[serde(default)]
    pub text: Option<String>,
    /// The texts of all its matches, in page order.
    #[serde(default)]
    pub texts: Option<Vec<String>>,
}

impl Cut {
    /// The IDs of the lines the cut uses, in script order.
    pub fn line_ids(&self, project: &Project) -> Vec<String> {
        match &self.lines {
            LineSelection::All => project
                .lines
                .iter()
                .filter(|l| l.is_for(self))
                .map(|l| l.id.clone())
                .collect(),
            LineSelection::Ids(ids) => ids.clone(),
        }
    }

    /// The cut's lines, in script order.
    pub fn lines_in<'p>(&self, project: &'p Project) -> Vec<&'p Line> {
        match &self.lines {
            LineSelection::All => project.lines.iter().filter(|l| l.is_for(self)).collect(),
            LineSelection::Ids(ids) => project
                .lines
                .iter()
                .filter(|l| ids.contains(&l.id))
                .collect(),
        }
    }

    /// How the cut carries its captions, with the default for its aspect.
    pub fn captions(&self) -> Captions {
        self.captions.unwrap_or(match self.aspect {
            Aspect::Landscape => Captions::Both,
            Aspect::Portrait => Captions::Burned,
        })
    }

    /// The frame size in pixels, with the default for its aspect.
    pub fn size(&self) -> [u32; 2] {
        self.size.unwrap_or(match self.aspect {
            Aspect::Landscape => [1920, 1080],
            Aspect::Portrait => [1080, 1920],
        })
    }

    /// The platforms the cut is made for, with the default for its aspect:
    /// the profiles on by default for that aspect (`platforms.toml`).
    pub fn platforms(&self) -> Vec<String> {
        self.platforms
            .clone()
            .unwrap_or_else(|| crate::platforms::defaults(self.aspect))
    }

    /// The edges of the frame that the cut's platforms cover, when they
    /// cover any (`platforms.toml`, each profile's safe zone).
    pub fn covered(&self) -> Option<crate::platforms::Insets> {
        crate::platforms::insets(&self.platforms(), self.size())
    }
}
