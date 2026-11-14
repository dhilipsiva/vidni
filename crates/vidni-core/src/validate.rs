//! Loading and checking a project file.
//!
//! The parser reports syntax errors, wrong types and unknown keys. Every rule
//! in `docs/spec.md` that reaches across tables (lines naming sections and
//! scenes, highlights naming targets, cuts naming lines) is checked here.
//! Each problem is reported at its line and column.

use crate::project::{
    Action, Aspect, Camera, Expectation, LineSelection, Project, SCHEMA, Scene, SceneKind, Target,
    TextMatch,
};
use regex::Regex;
use std::collections::{BTreeSet, HashMap};
use std::fmt;
use std::ops::Range;
use std::path::{Path, PathBuf};
use toml_edit::{Document, Item};

/// A problem in a project file, at a 1-based line and column.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Diagnostic {
    pub line: usize,
    pub column: usize,
    pub message: String,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}: {}", self.line, self.column, self.message)
    }
}

/// A project's file: `path` itself, or `path/vidni.toml` when `path` is a folder.
pub fn project_file(path: &Path) -> PathBuf {
    if path.is_dir() {
        path.join("vidni.toml")
    } else {
        path.to_path_buf()
    }
}

/// Parses and checks a project file's text. `dir` is the project folder,
/// for checking the files a project names; `None` skips those checks.
pub fn check(text: &str, dir: Option<&Path>) -> Result<Project, Vec<Diagnostic>> {
    let at = |span: Option<Range<usize>>, message: String| diagnostic(text, span, message);
    let doc = Document::parse(text)
        .map_err(|e| vec![at(e.span(), format!("TOML syntax: {}", e.message().trim()))])?;
    // A newer schema would fail below with confusing messages, so say so first.
    if let Some(item) = doc.as_item().get("schema")
        && let Some(version) = item.as_integer()
        && version != i64::from(SCHEMA)
    {
        let message =
            format!("schema {version} is not supported; this vidni reads schema {SCHEMA}");
        return Err(vec![at(item.span(), message)]);
    }
    let project: Project =
        toml::from_str(text).map_err(|e| vec![at(e.span(), plain(e.message()))])?;
    let mut rules = Rules::default();
    rules.project(&project, dir);
    if rules.found.is_empty() {
        return Ok(project);
    }
    let mut diagnostics: Vec<Diagnostic> = rules
        .found
        .into_iter()
        .map(|(path, message)| at(locate(doc.as_item(), &path), message))
        .collect();
    diagnostics.sort();
    diagnostics.dedup();
    Err(diagnostics)
}

/// A parser message in TOML's terms rather than Rust's.
fn plain(message: &str) -> String {
    let mut text = message.to_string();
    for (rust, toml) in [
        ("expected f64", "expected a number"),
        ("expected f32", "expected a number"),
        ("expected u64", "expected a whole number"),
        ("expected u32", "expected a whole number"),
        ("expected i64", "expected a whole number"),
        ("expected a sequence", "expected an array"),
        ("expected a map", "expected a table"),
        ("invalid type: sequence", "invalid type: array"),
        ("invalid type: map", "invalid type: table"),
    ] {
        text = text.replace(rust, toml);
    }
    match Regex::new(r"expected struct \w+") {
        Ok(re) => re.replace_all(&text, "expected a table").into_owned(),
        Err(_) => text,
    }
}

/// The 1-based line and column (in characters) of a byte offset.
fn position(text: &str, offset: usize) -> (usize, usize) {
    let mut offset = offset.min(text.len());
    while !text.is_char_boundary(offset) {
        offset -= 1;
    }
    let before = &text[..offset];
    let line = before.matches('\n').count() + 1;
    let column = before.rsplit('\n').next().map_or(0, |l| l.chars().count()) + 1;
    (line, column)
}

/// One step into the file.
#[derive(Debug, Clone)]
enum Seg {
    Key(String),
    Index(usize),
}

impl From<&str> for Seg {
    fn from(key: &str) -> Self {
        Seg::Key(key.into())
    }
}

impl From<usize> for Seg {
    fn from(index: usize) -> Self {
        Seg::Index(index)
    }
}

/// Builds a path into the file from keys and indexes.
macro_rules! path {
    ($($seg:expr),* $(,)?) => { vec![$(Seg::from($seg)),*] };
}

/// The span of the deepest item along `path` that the file spells out.
fn locate(root: &Item, path: &[Seg]) -> Option<Range<usize>> {
    let mut item = root;
    let mut span = root.span();
    for seg in path {
        let next = match seg {
            Seg::Key(k) => item.get(k.as_str()),
            Seg::Index(i) => item.get(*i),
        };
        let Some(next) = next else { break };
        item = next;
        span = next.span().or(span);
    }
    span
}

/// What `vidni validate` warns about without failing, in a project that
/// passed [`check`]: a draft voice whose licence doesn't allow commercial
/// use, or whose licence vidni can't find. `store` is the voice store, which
/// holds a creator's own packs.
pub fn warnings(text: &str, project: &Project, store: Option<&Path>) -> Vec<Diagnostic> {
    let Ok(doc) = Document::parse(text) else {
        return Vec::new();
    };
    let id = &project.voice.draft;
    let message = match crate::voices::find(id, store) {
        Ok(Some(pack)) if !pack.commercial => format!(
            "the draft voice {id} is licensed {}, which doesn't allow commercial use \
             (its model card: {})",
            pack.licence, pack.source
        ),
        Ok(Some(_)) => return Vec::new(),
        Ok(None) => format!(
            "the draft voice {id} isn't a voice pack vidni knows, and the voice store has no \
             pack.toml for it, so its licence is unknown"
        ),
        Err(e) => e,
    };
    let span = locate(doc.as_item(), &[Seg::from("voice"), Seg::from("draft")]);
    vec![diagnostic(text, span, message)]
}

fn diagnostic(text: &str, span: Option<Range<usize>>, message: String) -> Diagnostic {
    let (line, column) = position(text, span.map_or(0, |s| s.start));
    Diagnostic {
        line,
        column,
        message,
    }
}

/// Whether `id` is lowercase letters and digits in words joined by single hyphens.
pub fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.split('-').all(|word| {
            !word.is_empty()
                && word
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        })
}

fn id_rule(what: &str, id: &str) -> String {
    format!(
        "{what} ID {id:?} must be lowercase letters and digits, in words joined by single hyphens, like \"intro-1\""
    )
}

/// ", one of: a, b, c" for a short list of names; nothing for a long one.
fn one_of<'a>(names: impl IntoIterator<Item = &'a str>) -> String {
    let names: Vec<&str> = names.into_iter().collect();
    match names.len() {
        0 => " (there are none)".into(),
        1..=12 => format!(" (one of: {})", names.join(", ")),
        _ => String::new(),
    }
}

fn regex_problem(pattern: &str) -> Option<String> {
    Regex::new(pattern).err().map(|e| {
        let text = e.to_string();
        let last = text.lines().last().unwrap_or_default().trim();
        format!(
            "not a regular expression: {}",
            last.trim_start_matches("error: ")
        )
    })
}

/// A target's regular expression runs in the page, as JavaScript's in
/// Unicode mode, where a leading `(?i)`, `(?m)` or `(?s)` sets that flag.
/// So it is checked as JavaScript, not as Rust.
fn page_regex_problem(pattern: &str) -> Option<String> {
    let leading = pattern
        .strip_prefix("(?")
        .and_then(|rest| rest.split_once(')'))
        .filter(|(flags, _)| !flags.is_empty() && flags.chars().all(|c| "ims".contains(c)));
    let (mut flags, body): (Vec<char>, &str) = match leading {
        Some((flags, body)) => (flags.chars().collect(), body),
        None => (Vec::new(), pattern),
    };
    flags.sort_unstable();
    flags.dedup();
    flags.push('u');
    let flags: String = flags.into_iter().collect();
    regress::Regex::with_flags(body, flags.as_str())
        .err()
        .map(|e| format!("not a JavaScript regular expression: {e}"))
}

#[derive(Default)]
struct Rules {
    found: Vec<(Vec<Seg>, String)>,
}

impl Rules {
    fn add(&mut self, path: Vec<Seg>, message: impl Into<String>) {
        self.found.push((path, message.into()));
    }

    fn project(&mut self, p: &Project, dir: Option<&Path>) {
        if p.meta.title.trim().is_empty() {
            self.add(path!["meta", "title"], "the title is empty");
        }
        // The language goes into file names (`long.en.srt`), so it must
        // be a tag and nothing else: subtags of letters and digits, joined
        // by hyphens, the first of two to eight letters.
        let language = &p.meta.language;
        let mut subtags = language.split('-');
        let primary = subtags.next().unwrap_or_default();
        let tag = (2..=8).contains(&primary.len())
            && primary.chars().all(|c| c.is_ascii_alphabetic())
            && subtags.all(|s| {
                (1..=8).contains(&s.len()) && s.chars().all(|c| c.is_ascii_alphanumeric())
            });
        if !tag {
            self.add(
                path!["meta", "language"],
                format!("language must be a BCP 47 tag such as en or pt-BR, not {language:?}"),
            );
        }
        if let Some(scene) = &p.meta.default_scene
            && !p.scenes.contains_key(scene)
        {
            let names = one_of(p.scenes.keys().map(String::as_str));
            self.add(
                path!["meta", "default_scene"],
                format!("unknown scene {scene:?}{names}"),
            );
        }
        self.timing_and_voice(p);
        for (i, pattern) in p.lint.banned.iter().enumerate() {
            if let Some(problem) = regex_problem(&format!("(?i){pattern}")) {
                self.add(path!["lint", "banned", i], problem);
            }
        }
        self.lines(p);
        self.honesty(p);
        for (id, scene) in &p.scenes {
            self.scene(id, scene, dir);
        }
        self.cuts(p);
        self.music(p, dir);
        self.publish(p);
        for (i, x) in p.expectations.iter().enumerate() {
            self.expectation(p, i, x);
        }
    }

    fn timing_and_voice(&mut self, p: &Project) {
        let t = &p.timing;
        for (key, value) in [
            ("lead_in", t.lead_in),
            ("line_gap", t.line_gap),
            ("section_gap", t.section_gap),
            ("tail", t.tail),
        ] {
            if !value.is_finite() || value < 0.0 {
                self.add(
                    path!["timing", key],
                    format!("{key} must be zero or more seconds, not {value}"),
                );
            }
        }
        if !p.voice.length_scale.is_finite() || p.voice.length_scale <= 0.0 {
            self.add(
                path!["voice", "length_scale"],
                "length_scale must be above 0",
            );
        }
        for (list, entries) in [("read_as", &p.voice.read_as), ("respell", &p.voice.respell)] {
            for (i, r) in entries.iter().enumerate() {
                if r.text.is_empty() {
                    self.add(
                        path!["voice", list, i, "text"],
                        "the text to replace is empty",
                    );
                }
            }
        }
        let mut listed: BTreeSet<String> = BTreeSet::new();
        for (i, h) in p.voice.heard_as.iter().enumerate() {
            let at = |key: &str| path!["voice", "heard_as", i, key];
            // The listening check compares word by word, and its words are
            // runs of these characters.
            let word = |c: char| c.is_ascii_alphanumeric() || c == '\'';
            if h.heard.is_empty() || !h.heard.chars().all(word) {
                self.add(
                    at("heard"),
                    format!(
                        "heard must be one word of letters, digits and apostrophes, not {:?}",
                        h.heard
                    ),
                );
            } else if !listed.insert(h.heard.to_ascii_lowercase()) {
                self.add(
                    at("heard"),
                    format!("heard_as already says what {:?} means", h.heard),
                );
            }
            if !h.means.chars().any(|c| c.is_ascii_alphabetic()) {
                self.add(
                    at("means"),
                    format!("means must be a word, not {:?}", h.means),
                );
            }
        }
    }

    fn lines(&mut self, p: &Project) {
        if p.lines.is_empty() {
            self.add(path!["lines"], "the script has no lines");
        }
        for id in p.sections.keys() {
            if !valid_id(id) {
                self.add(path!["sections", id.as_str()], id_rule("section", id));
            }
        }
        let mut first_use: HashMap<&str, usize> = HashMap::new();
        let mut closed: BTreeSet<&str> = BTreeSet::new();
        let mut current: Option<&str> = None;
        for (i, line) in p.lines.iter().enumerate() {
            let at = |key: &str| path!["lines", i, key];
            if !valid_id(&line.id) {
                self.add(at("id"), id_rule("line", &line.id));
            } else if let Some(first) = first_use.insert(&line.id, i) {
                self.add(
                    at("id"),
                    format!(
                        "line ID {:?} is already used by line {} of the script",
                        line.id,
                        first + 1
                    ),
                );
            }
            if !p.sections.contains_key(&line.section) {
                let names = one_of(p.sections.keys().map(String::as_str));
                self.add(
                    at("section"),
                    format!("unknown section {:?}{names}", line.section),
                );
            } else if current != Some(line.section.as_str()) {
                if closed.contains(line.section.as_str()) {
                    self.add(
                        at("section"),
                        format!("section {:?} starts again after another section; keep its lines together", line.section),
                    );
                }
                closed.extend(current);
                current = Some(&line.section);
            }
            if line.show.trim().is_empty() {
                self.add(at("show"), "the line shows nothing");
            }
            for (j, cut) in line.only.iter().enumerate() {
                let at = path!["lines", i, "only", j];
                if line.only[..j].contains(cut) {
                    self.add(at, format!("cut {cut:?} is listed twice"));
                    continue;
                }
                match p.cuts.iter().find(|c| &c.id == cut) {
                    None => {
                        let names = one_of(p.cuts.iter().map(|c| c.id.as_str()));
                        self.add(at, format!("unknown cut {cut:?}{names}"));
                    }
                    // A line that is only for a cut that leaves it out is
                    // in no video: a hook that was written and then lost.
                    Some(c) => {
                        if let LineSelection::Ids(ids) = &c.lines
                            && !ids.contains(&line.id)
                        {
                            self.add(at, format!("cut {cut:?} doesn't list this line"));
                        }
                    }
                }
            }
            if line.say.as_deref().is_some_and(|s| s.trim().is_empty()) {
                self.add(
                    at("say"),
                    "say is empty; leave it out to say what the line shows",
                );
            }
            self.line_scene(p, i);
            if let Some(overlay) = &line.overlay {
                match p.scenes.get(overlay) {
                    None => {
                        let names = one_of(p.scenes.keys().map(String::as_str));
                        self.add(at("overlay"), format!("unknown scene {overlay:?}{names}"));
                    }
                    Some(s) if s.kind == SceneKind::Overlay => {}
                    Some(s) if s.kind != SceneKind::Page || s.transparent != Some(true) => {
                        self.add(
                            at("overlay"),
                            format!(
                                "overlay {overlay:?} must be an overlay scene, or a page scene with transparent = true"
                            ),
                        );
                    }
                    Some(_) => {}
                }
            }
        }
        let used: BTreeSet<&str> = p.lines.iter().map(|l| l.section.as_str()).collect();
        // A line naming an unknown section is most likely a typo for an empty
        // one; report the typo alone.
        let typo = used.iter().any(|s| !p.sections.contains_key(*s));
        for id in p.sections.keys().filter(|_| !typo) {
            if !used.contains(id.as_str()) {
                self.add(
                    path!["sections", id.as_str()],
                    format!("section {id:?} has no lines"),
                );
            }
        }
    }

    /// The wording lint, and sources in strict projects.
    fn honesty(&mut self, p: &Project) {
        let banned: Vec<(&str, Regex)> = p
            .lint
            .banned
            .iter()
            .filter_map(|pattern| {
                Some((
                    pattern.as_str(),
                    Regex::new(&format!("(?i){pattern}")).ok()?,
                ))
            })
            .collect();
        for (i, line) in p.lines.iter().enumerate() {
            for (key, text) in [("show", Some(&line.show)), ("say", line.say.as_ref())] {
                let Some(text) = text else { continue };
                if let Some((pattern, found)) = banned
                    .iter()
                    .find_map(|(pattern, re)| Some((*pattern, re.find(text)?)))
                {
                    let verb = if key == "show" { "shows" } else { "says" };
                    self.add(
                        path!["lines", i, key],
                        format!(
                            "line {:?} {verb} banned wording: {:?} matches the lint pattern {pattern:?}",
                            line.id,
                            found.as_str()
                        ),
                    );
                }
            }
            if line.source.as_deref().is_some_and(|s| s.trim().is_empty()) {
                self.add(
                    path!["lines", i, "source"],
                    "source is empty; name where the line's facts come from",
                );
            } else if p.meta.strict && line.claim != Some(false) && line.source.is_none() {
                self.add(
                    path!["lines", i, "id"],
                    format!(
                        "line {:?} states a fact with no source, and the project is strict; add source = \"…\", or claim = false if the line states no fact",
                        line.id
                    ),
                );
            }
        }
    }

    fn line_scene(&mut self, p: &Project, i: usize) {
        let line = &p.lines[i];
        let at = |key: &str| path!["lines", i, key];
        let Some(id) = line.scene.as_deref().or(p.meta.default_scene.as_deref()) else {
            self.add(
                path!["lines", i],
                "the line names no scene, and [meta] sets no default_scene",
            );
            return;
        };
        let Some(scene) = p.scenes.get(id) else {
            // An unknown default_scene is reported once, at [meta].
            if line.scene.is_some() {
                let names = one_of(p.scenes.keys().map(String::as_str));
                self.add(at("scene"), format!("unknown scene {id:?}{names}"));
            }
            return;
        };
        match scene.kind {
            SceneKind::Overlay => {
                let key = if line.scene.is_some() { "scene" } else { "id" };
                self.add(
                    at(key),
                    format!("{id:?} is an overlay; show it with overlay = {id:?}"),
                );
            }
            SceneKind::Image => {
                for (key, set) in [
                    ("do", !line.actions.is_empty()),
                    ("highlight", !line.highlight.is_empty()),
                ] {
                    if set {
                        self.add(
                            at(key),
                            format!("{key} needs a web scene, and {id:?} is an image scene"),
                        );
                    }
                }
                for (key, camera) in [
                    ("camera", &line.camera),
                    ("camera_vertical", &line.camera_vertical),
                ] {
                    if let Some(camera) = camera {
                        self.camera_ref(at(key), camera, id, scene);
                    }
                }
            }
            SceneKind::Page | SceneKind::Card => {
                for (key, set) in [
                    ("do", !line.actions.is_empty()),
                    ("highlight", !line.highlight.is_empty()),
                    ("camera", line.camera.is_some()),
                    ("camera_vertical", line.camera_vertical.is_some()),
                ] {
                    if set {
                        self.add(
                            at(key),
                            format!(
                                "{key} needs a web scene, and {id:?} is a {} scene",
                                if scene.kind == SceneKind::Card {
                                    "card"
                                } else {
                                    "page"
                                }
                            ),
                        );
                    }
                }
                if scene.transparent == Some(true) {
                    let key = if line.scene.is_some() { "scene" } else { "id" };
                    self.add(
                        at(key),
                        format!("{id:?} is a transparent overlay; show it with overlay = {id:?}"),
                    );
                }
            }
            SceneKind::Terminal | SceneKind::Code => {
                if !line.actions.is_empty() {
                    self.add(
                        at("do"),
                        format!("do needs a web scene, and {id:?} is text that vidni draws"),
                    );
                }
                for (j, target) in line.highlight.iter().enumerate() {
                    if !scene.targets.contains_key(target) {
                        self.unknown_target(path!["lines", i, "highlight", j], target, id, scene);
                    }
                }
                for (key, camera) in [
                    ("camera", &line.camera),
                    ("camera_vertical", &line.camera_vertical),
                ] {
                    if let Some(camera) = camera {
                        self.camera_ref(at(key), camera, id, scene);
                    }
                }
            }
            SceneKind::Web => {
                for (j, action) in line.actions.iter().enumerate() {
                    self.action(path!["lines", i, "do", j], action, id, scene);
                }
                for (j, target) in line.highlight.iter().enumerate() {
                    if !scene.targets.contains_key(target) {
                        self.unknown_target(path!["lines", i, "highlight", j], target, id, scene);
                    }
                }
                for (key, camera) in [
                    ("camera", &line.camera),
                    ("camera_vertical", &line.camera_vertical),
                ] {
                    if let Some(camera) = camera {
                        self.camera_ref(at(key), camera, id, scene);
                    }
                }
            }
        }
    }

    fn unknown_target(&mut self, at: Vec<Seg>, target: &str, scene_id: &str, scene: &Scene) {
        let names = one_of(scene.targets.keys().map(String::as_str));
        self.add(
            at,
            format!("unknown target {target:?} in scene {scene_id:?}{names}"),
        );
    }

    fn camera_ref(&mut self, at: Vec<Seg>, camera: &str, scene_id: &str, scene: &Scene) {
        if !camera_exists(scene, camera) {
            let names = one_of(
                ["viewport", "highlights"]
                    .into_iter()
                    .chain(scene.cameras.keys().map(String::as_str)),
            );
            self.add(
                at,
                format!("unknown camera {camera:?} in scene {scene_id:?}{names}"),
            );
        }
    }

    fn action(&mut self, at: Vec<Seg>, action: &Action, scene_id: &str, scene: &Scene) {
        let verbs = action.verbs();
        let [(verb, target)] = verbs[..] else {
            let message = if verbs.is_empty() {
                "an action needs one verb: click, select, expand, collapse, scroll, scroll_to or wait".to_string()
            } else {
                let names: Vec<&str> = verbs.iter().map(|(v, _)| *v).collect();
                format!(
                    "an action has one verb, and this one has {}",
                    names.join(" and ")
                )
            };
            self.add(at, message);
            return;
        };
        let with = |key: &str| {
            let mut p = at.clone();
            p.push(Seg::from(key));
            p
        };
        if !scene.targets.contains_key(target) {
            self.unknown_target(with(verb), target, scene_id, scene);
        }
        for (modifier, verbs) in action.modifiers() {
            if !verbs.contains(&verb) {
                self.add(
                    with(modifier),
                    format!(
                        "{modifier} goes with {}, and this action is {verb}",
                        verbs.join(" or ")
                    ),
                );
            }
        }
        match verb {
            "select" if action.value.is_none() => self.add(with(verb), "select needs a value"),
            "scroll" if action.top.is_none() => self.add(with(verb), "scroll needs top"),
            "scroll" if action.top.is_some_and(|t| !t.is_finite()) => {
                self.add(with("top"), "top must be a finite number")
            }
            "scroll_to" if action.offset.is_some_and(|o| !o.is_finite()) => {
                self.add(with("offset"), "offset must be a finite number")
            }
            "scroll_to" => match &action.within {
                None => self.add(with(verb), "scroll_to needs within, the scroll container"),
                Some(w) if !scene.targets.contains_key(w) => {
                    self.unknown_target(with("within"), w, scene_id, scene)
                }
                Some(_) => {}
            },
            _ => {}
        }
    }

    fn scene(&mut self, id: &str, s: &Scene, dir: Option<&Path>) {
        let at = |key: &str| path!["scenes", id, key];
        if !valid_id(id) {
            self.add(path!["scenes", id], id_rule("scene", id));
        }
        if !matches!(s.kind, SceneKind::Card | SceneKind::Overlay) && s.url.trim().is_empty() {
            self.add(at("url"), "url is empty");
        }
        let card_only = [
            ("template", s.template.is_some()),
            ("title", s.title.is_some()),
            ("subtitle", s.subtitle.is_some()),
            ("items", !s.items.is_empty()),
            ("quote", s.quote.is_some()),
            ("by", s.by.is_some()),
        ];
        // A window of text has a title too, and an overlay has a template.
        let card_only: Vec<(&str, bool)> = card_only
            .into_iter()
            .filter(|(key, _)| !(s.is_text() && *key == "title"))
            .filter(|(key, _)| !(s.kind == SceneKind::Overlay && *key == "template"))
            .collect();
        if s.kind != SceneKind::Overlay && !s.parts.is_empty() {
            self.add(
                at("parts"),
                format!("parts is for overlay scenes, and {id:?} isn't one"),
            );
        }
        let text_only = [
            ("prompt", s.prompt.is_some()),
            ("repo", s.repo.is_some()),
            ("rev", s.rev.is_some()),
            ("language", s.language.is_some()),
            ("lines", s.lines.is_some()),
            ("columns", s.columns.is_some()),
        ];
        if !s.is_text() {
            for (key, set) in text_only {
                if set {
                    self.add(
                        at(key),
                        format!("{key} is for terminal and code scenes, and {id:?} isn't one"),
                    );
                }
            }
        }
        if s.kind != SceneKind::Card {
            for (key, set) in card_only {
                if set {
                    self.add(
                        at(key),
                        format!("{key} is for card scenes, and {id:?} isn't one"),
                    );
                }
            }
        }
        let web_only = [
            ("viewport", s.viewport.is_some()),
            ("scale", s.scale.is_some()),
            ("color_scheme", s.color_scheme.is_some()),
            ("setup", !s.setup.is_empty()),
            ("targets", !s.targets.is_empty()),
            ("cameras", !s.cameras.is_empty()),
            ("default_camera", s.default_camera.is_some()),
            (
                "vertical_follows_highlights",
                s.vertical_follows_highlights.is_some(),
            ),
        ];
        let page_only = [
            ("transparent", s.transparent.is_some()),
            ("size", s.size.is_some()),
        ];
        let remote = s.on_the_web();
        match s.kind {
            SceneKind::Web => {
                for (key, set) in page_only {
                    if set {
                        self.add(
                            at(key),
                            format!("{key} is for page scenes, and {id:?} is a web scene"),
                        );
                    }
                }
                if let Some([w, h]) = s.viewport
                    && (w == 0 || h == 0)
                {
                    self.add(
                        at("viewport"),
                        "the viewport needs a width and a height above 0",
                    );
                }
                if s.scale == Some(0) {
                    self.add(at("scale"), "scale must be 1 or more");
                }
                for (tid, t) in &s.targets {
                    if !valid_id(tid) {
                        self.add(
                            path!["scenes", id, "targets", tid.as_str()],
                            id_rule("target", tid),
                        );
                    }
                    self.target(path!["scenes", id, "targets", tid.as_str()], t);
                }
                for (cid, c) in &s.cameras {
                    self.camera(id, s, cid, c);
                }
                if let Some(camera) = &s.default_camera {
                    self.camera_ref(at("default_camera"), camera, id, s);
                }
                if let Some(width) = s.vertical_follows_highlights
                    && (!width.is_finite() || width <= 0.0)
                {
                    self.add(
                        at("vertical_follows_highlights"),
                        "the width must be above 0",
                    );
                }
                for (j, action) in s.setup.iter().enumerate() {
                    self.action(path!["scenes", id, "setup", j], action, id, s);
                }
                if !remote {
                    self.project_file(at("url"), &s.url, dir);
                }
            }
            SceneKind::Page => {
                for (key, set) in web_only {
                    if set {
                        self.add(
                            at(key),
                            format!("{key} is for web scenes, and {id:?} is a page scene"),
                        );
                    }
                }
                if remote {
                    self.add(
                        at("url"),
                        "a page scene renders a file in the project folder, not a web address",
                    );
                } else {
                    self.project_file(at("url"), &s.url, dir);
                }
                if let Some(size) = &s.size
                    && (size.landscape.contains(&0) || size.portrait.contains(&0))
                {
                    self.add(at("size"), "each size needs a width and a height above 0");
                }
            }
            SceneKind::Card => self.card(id, s, &web_only, &page_only),
            SceneKind::Image => self.image(id, s, dir, &page_only),
            SceneKind::Terminal | SceneKind::Code => self.text(id, s, dir, &page_only),
            SceneKind::Overlay => self.overlay(id, s, &web_only, &page_only),
        }
    }

    /// An overlay scene: a template, and the parts that template draws.
    fn overlay(
        &mut self,
        id: &str,
        s: &Scene,
        web_only: &[(&str, bool)],
        page_only: &[(&str, bool)],
    ) {
        use crate::project::Template;
        let at = |key: &str| path!["scenes", id, key];
        let others = [("url", !s.url.is_empty())];
        for (key, set) in others.iter().chain(web_only).chain(page_only) {
            if *set {
                self.add(
                    at(key),
                    format!(
                        "{key} isn't for overlay scenes: an overlay is drawn from its template"
                    ),
                );
            }
        }
        let template = match s.template {
            Some(template @ (Template::Chips | Template::Flow | Template::Callout)) => template,
            _ => {
                self.add(
                    at("template"),
                    "an overlay's template is chips, flow or callout",
                );
                return;
            }
        };
        if s.parts.is_empty() {
            self.add(
                path!["scenes", id],
                "an overlay needs parts: [[scenes.<id>.parts]]",
            );
        }
        if template == Template::Callout && s.parts.len() > 1 {
            self.add(at("parts"), "a callout is one part");
        }
        let empty = |text: &Option<String>| text.as_deref().is_none_or(|t| t.trim().is_empty());
        for (n, part) in s.parts.iter().enumerate() {
            let here = path!["scenes", id, "parts", n];
            if part.wall.is_some() {
                let only_wall = crate::project::Part {
                    wall: part.wall.clone(),
                    ..Default::default()
                };
                if template != Template::Flow {
                    self.add(here, "a wall is a part of a flow");
                } else if *part != only_wall {
                    self.add(here, "a wall has no other key: it stands between boxes");
                } else if n == 0 || n + 1 == s.parts.len() || s.parts[n - 1].wall.is_some() {
                    self.add(here, "a wall stands between two boxes");
                }
                continue;
            }
            let problem = match template {
                Template::Chips if empty(&part.label) => Some("a chip needs a label"),
                Template::Chips if part.sign.as_deref().is_some_and(|s| s.chars().count() != 1) => {
                    Some("a chip's sign is one character")
                }
                Template::Flow | Template::Callout if empty(&part.text) => Some("a box needs text"),
                Template::Flow | Template::Callout if part.sign.is_some() => {
                    Some("sign is for chips")
                }
                _ => None,
            };
            if let Some(problem) = problem {
                self.add(here, problem);
            }
        }
    }

    /// A terminal or code scene: a text file, with lines of it named as
    /// its parts.
    fn text(&mut self, id: &str, s: &Scene, dir: Option<&Path>, page_only: &[(&str, bool)]) {
        let at = |key: &str| path!["scenes", id, key];
        let code = s.kind == SceneKind::Code;
        let kind = if code { "code" } else { "terminal" };
        let not_for_text = [
            ("viewport", s.viewport.is_some()),
            ("color_scheme", s.color_scheme.is_some()),
            ("setup", !s.setup.is_empty()),
            (
                "vertical_follows_highlights",
                s.vertical_follows_highlights.is_some(),
            ),
        ];
        for (key, set) in not_for_text.iter().chain(page_only) {
            if *set {
                self.add(at(key), format!("{key} isn't for {kind} scenes"));
            }
        }
        let for_one = [
            ("prompt", s.prompt.is_some(), false),
            ("repo", s.repo.is_some(), true),
            ("rev", s.rev.is_some(), true),
            ("language", s.language.is_some(), true),
        ];
        for (key, set, for_code) in for_one {
            if set && for_code != code {
                let other = if for_code { "code" } else { "terminal" };
                self.add(
                    at(key),
                    format!("{key} is for {other} scenes, and {id:?} is a {kind} scene"),
                );
            }
        }
        if s.prompt.as_deref() == Some("") {
            self.add(at("prompt"), "prompt is empty");
        }
        let file = s.url.split(['?', '#']).next().unwrap_or_default();
        if s.on_the_web() || file != s.url {
            self.add(
                at("url"),
                format!("a {kind} scene shows a file, not an address"),
            );
        } else if s.rev.is_none() {
            self.project_file(at("url"), &s.url, dir);
        } else if Path::new(&s.url).is_absolute() || s.url.split('/').any(|part| part == "..") {
            self.add(
                at("url"),
                "with rev, url is the file's path inside the repository",
            );
        }
        // A branch or a tag can move; a commit's hash can't.
        if let Some(rev) = &s.rev
            && !((7..=40).contains(&rev.len()) && rev.chars().all(|c| c.is_ascii_hexdigit()))
        {
            self.add(
                at("rev"),
                "rev is a commit's hash, 7 to 40 hexadecimal digits: a branch or a tag can move",
            );
        }
        if s.repo.is_some() && s.rev.is_none() {
            self.add(
                at("repo"),
                "repo goes with rev: the revision to read the file at",
            );
        }
        if s.scale == Some(0) {
            self.add(at("scale"), "scale must be 1 or more");
        }
        let range = |lines: [u32; 2]| lines[0] >= 1 && lines[1] >= lines[0];
        if s.lines.is_some_and(|lines| !range(lines)) {
            self.add(
                at("lines"),
                "lines = [first, last], counted from 1, with last not before first",
            );
        }
        if s.columns
            .is_some_and(|columns| !(20..=200).contains(&columns))
        {
            self.add(at("columns"), "columns is between 20 and 200");
        }
        for (tid, t) in &s.targets {
            let here = path!["scenes", id, "targets", tid.as_str()];
            if !valid_id(tid) {
                self.add(here.clone(), id_rule("target", tid));
            }
            let only_lines = Target {
                lines: t.lines,
                ..Target::default()
            };
            let inside = |lines: [u32; 2]| {
                s.lines
                    .is_none_or(|shown| lines[0] >= shown[0] && lines[1] <= shown[1])
            };
            match t.lines {
                Some(lines) if *t == only_lines && range(lines) && inside(lines) => {}
                Some(lines) if *t == only_lines && range(lines) => self.add(
                    here,
                    format!(
                        "lines {} to {} aren't all among the lines the scene shows",
                        lines[0], lines[1]
                    ),
                ),
                _ => self.add(
                    here,
                    format!(
                        "a part of a {kind} scene is lines = [first, last], counted from 1 in its file"
                    ),
                ),
            }
        }
        for (cid, c) in &s.cameras {
            self.camera(id, s, cid, c);
        }
        if let Some(camera) = &s.default_camera {
            self.camera_ref(at("default_camera"), camera, id, s);
        }
    }

    /// An image scene: a PNG or SVG file of the project's, with parts of it
    /// named by their rectangles, and cameras on them.
    fn image(&mut self, id: &str, s: &Scene, dir: Option<&Path>, page_only: &[(&str, bool)]) {
        let at = |key: &str| path!["scenes", id, key];
        // A Mermaid diagram is drawn dark or light; a picture is as it is.
        let mermaid = s.url.to_lowercase().ends_with(".mmd");
        let not_for_images = [
            ("viewport", s.viewport.is_some()),
            ("color_scheme", s.color_scheme.is_some() && !mermaid),
            ("setup", !s.setup.is_empty()),
        ];
        for (key, set) in not_for_images.iter().chain(page_only) {
            if *set {
                self.add(at(key), format!("{key} isn't for image scenes"));
            }
        }
        let file = s.url.split(['?', '#']).next().unwrap_or_default();
        let lower = file.to_lowercase();
        if s.on_the_web() || file != s.url {
            self.add(
                at("url"),
                "an image scene shows a file in the project folder",
            );
        } else if ![".png", ".svg", ".mmd"]
            .iter()
            .any(|end| lower.ends_with(end))
        {
            self.add(
                at("url"),
                "an image scene shows a PNG, an SVG or a Mermaid (.mmd) file",
            );
        } else {
            self.project_file(at("url"), &s.url, dir);
        }
        if s.scale == Some(0) {
            self.add(at("scale"), "scale must be 1 or more");
        }
        for (tid, t) in &s.targets {
            let here = path!["scenes", id, "targets", tid.as_str()];
            if !valid_id(tid) {
                self.add(here.clone(), id_rule("target", tid));
            }
            let only_rect = Target {
                rect: t.rect,
                ..Target::default()
            };
            match t.rect {
                Some([_, _, w, h]) if *t == only_rect && w > 0.0 && h > 0.0 => {}
                _ => self.add(
                    here,
                    "a part of an image is rect = [x, y, width, height], with a width and a height above 0",
                ),
            }
        }
        for (cid, c) in &s.cameras {
            self.camera(id, s, cid, c);
        }
        if let Some(camera) = &s.default_camera {
            self.camera_ref(at("default_camera"), camera, id, s);
        }
    }

    /// A card scene: a template, and the words that template shows.
    fn card(&mut self, id: &str, s: &Scene, web_only: &[(&str, bool)], page_only: &[(&str, bool)]) {
        use crate::project::Template;
        let at = |key: &str| path!["scenes", id, key];
        let others = [("url", !s.url.is_empty())];
        for (key, set) in others.iter().chain(web_only).chain(page_only) {
            if *set {
                self.add(
                    at(key),
                    format!("{key} isn't for card scenes: a card is drawn from its template"),
                );
            }
        }
        let overlays = [Template::Chips, Template::Flow, Template::Callout];
        let Some(template) = s.template.filter(|t| !overlays.contains(t)) else {
            self.add(
                at("kind"),
                "a card scene needs a template: title, section, bullets, quote or end",
            );
            return;
        };
        // What each template shows: (key, whether it is set, needed, allowed).
        let (title, text, list, quoted) = match template {
            Template::Title | Template::Section | Template::End => (true, true, false, false),
            Template::Bullets => (true, false, true, false),
            Template::Quote => (false, false, false, true),
            // An overlay's templates were turned away above.
            Template::Chips | Template::Flow | Template::Callout => return,
        };
        let name = format!("{template:?}").to_lowercase();
        let fields = [
            (
                "title",
                s.title.as_deref().is_some_and(|t| !t.trim().is_empty()),
                title,
                title,
            ),
            ("subtitle", s.subtitle.is_some(), false, text),
            ("items", !s.items.is_empty(), list, list),
            (
                "quote",
                s.quote.as_deref().is_some_and(|q| !q.trim().is_empty()),
                quoted,
                quoted,
            ),
            ("by", s.by.is_some(), false, quoted),
        ];
        for (key, set, needed, allowed) in fields {
            if needed && !set {
                self.add(at("template"), format!("a {name} card needs {key}"));
            } else if set && !allowed {
                self.add(at(key), format!("a {name} card shows no {key}"));
            }
        }
        if s.items.iter().any(|item| item.trim().is_empty()) {
            self.add(at("items"), "an item is empty");
        }
    }

    /// A file the project names, relative to its folder, with `{aspect}` filled in.
    /// What goes with the video where it is uploaded fits where it goes
    /// (TODO.md, P6-VIDNI-04).
    fn publish(&mut self, p: &Project) {
        use crate::publish::{TAGS, THUMBNAILS, TITLE, tags_length, title};
        let publish = &p.publish;
        let profiles = crate::platforms::all();
        for (platform, own) in &publish.titles {
            let at = path!["publish", "titles", platform.as_str()];
            if !profiles.iter().any(|profile| &profile.id == platform) {
                let names = one_of(profiles.iter().map(|profile| profile.id.as_str()));
                self.add(at, format!("unknown platform {platform:?}{names}"));
            } else if own.trim().is_empty() {
                self.add(at, "a title can't be empty");
            }
        }
        // YouTube takes a title of 100 characters, on a video and on a Short.
        let mut seen: Vec<String> = Vec::new();
        for platform in p.cuts.iter().flat_map(|cut| cut.platforms()) {
            if !platform.starts_with("youtube") || seen.contains(&platform) {
                continue;
            }
            let length = title(p, &platform).chars().count();
            if length > TITLE {
                let at = match publish.titles.contains_key(&platform) {
                    true => path!["publish", "titles", platform.as_str()],
                    false => path!["meta", "title"],
                };
                self.add(
                    at,
                    format!("the title is {length} characters, and {platform} takes {TITLE}; [publish.titles] can give it a shorter one"),
                );
            }
            seen.push(platform);
        }
        for (i, tag) in publish.tags.iter().enumerate() {
            let at = path!["publish", "tags", i];
            if tag.trim().is_empty() || tag.contains(',') {
                self.add(at, "a tag is a word or a few, with no comma");
            } else if publish.tags[..i].contains(tag) {
                self.add(at, format!("tag {tag:?} is listed twice"));
            }
        }
        let length = tags_length(&publish.tags);
        if length > TAGS {
            self.add(
                path!["publish", "tags"],
                format!("the tags take {length} characters, and YouTube takes {TAGS}"),
            );
        }
        for (i, link) in publish.links.iter().enumerate() {
            if link.label.trim().is_empty() {
                self.add(
                    path!["publish", "links", i, "label"],
                    "a link needs a label",
                );
            }
            if !(link.url.starts_with("https://") || link.url.starts_with("http://")) {
                self.add(
                    path!["publish", "links", i, "url"],
                    "a link's url is an http or https address",
                );
            }
        }
        if publish.thumbnails.len() > THUMBNAILS {
            self.add(
                path!["publish", "thumbnails", THUMBNAILS],
                format!("YouTube compares {THUMBNAILS} thumbnails at most"),
            );
        }
        // A thumbnail is of a line that a 16:9 cut shows.
        let wide: Vec<String> = p
            .cuts
            .iter()
            .filter(|cut| cut.aspect == Aspect::Landscape)
            .flat_map(|cut| cut.line_ids(p))
            .collect();
        for (i, thumbnail) in publish.thumbnails.iter().enumerate() {
            let at = path!["publish", "thumbnails", i, "line"];
            if !p.lines.iter().any(|line| line.id == thumbnail.line) {
                self.add(at, format!("unknown line {:?}", thumbnail.line));
            } else if !wide.contains(&thumbnail.line) {
                self.add(
                    at,
                    format!("line {:?} is in no 16:9 cut, so there is no picture of it to make a thumbnail of", thumbnail.line),
                );
            }
        }
        for (i, erratum) in p.errata.iter().enumerate() {
            let at = |key: &str| path!["errata", i, key];
            if !p.lines.iter().any(|line| line.id == erratum.line) {
                self.add(at("line"), format!("unknown line {:?}", erratum.line));
            }
            let day = erratum.on.as_bytes();
            let dated = day.len() == 10
                && day.iter().enumerate().all(|(n, b)| match n {
                    4 | 7 => *b == b'-',
                    _ => b.is_ascii_digit(),
                });
            if !dated {
                self.add(at("on"), "a correction's day is written as 2026-11-20");
            }
            if erratum.text.trim().is_empty() {
                self.add(at("text"), "a correction says what is right");
            }
        }
        for (c, cut) in p.cuts.iter().enumerate() {
            let Some(hold) = cut.end_hold else { continue };
            let at = path!["cuts", c, "end_hold"];
            if cut.aspect != Aspect::Landscape {
                self.add(at, "end_hold is for a 16:9 cut: an end screen goes on a YouTube video, not on a Short");
            } else if !(5.0..=20.0).contains(&hold) {
                self.add(
                    at,
                    "an end screen covers the last 5 to 20 seconds of a video",
                );
            }
        }
    }

    /// The music library's entries each carry what allows the track, the
    /// track that plays is one of them, and its licence covers every
    /// platform a cut is made for (TODO.md, P6-VIDNI-06).
    fn music(&mut self, p: &Project, dir: Option<&Path>) {
        let music = &p.music;
        let profiles = crate::platforms::all();
        for (i, track) in music.library.iter().enumerate() {
            let at = |key: &str| path!["music", "library", i, key];
            if !valid_id(&track.id) {
                self.add(at("id"), id_rule("track", &track.id));
            } else if music.library[..i].iter().any(|t| t.id == track.id) {
                self.add(
                    at("id"),
                    format!("track {:?} is in the library twice", track.id),
                );
            }
            for (key, value, what) in [
                ("licence", &track.licence, "the licence it is used under"),
                (
                    "attribution",
                    &track.attribution,
                    "the credit its licence asks for",
                ),
            ] {
                if value.trim().is_empty() {
                    self.add(at(key), format!("a track needs {what}"));
                }
            }
            if track.platforms.is_empty() {
                self.add(
                    at("platforms"),
                    "a track needs the platforms its licence covers",
                );
            }
            for (j, id) in track.platforms.iter().enumerate() {
                if !profiles.iter().any(|profile| &profile.id == id) {
                    let names = one_of(profiles.iter().map(|profile| profile.id.as_str()));
                    self.add(
                        path!["music", "library", i, "platforms", j],
                        format!("unknown platform {id:?}{names}"),
                    );
                }
            }
            let hex = track.sha256.len() == 64
                && track
                    .sha256
                    .chars()
                    .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c));
            if !hex {
                self.add(
                    at("sha256"),
                    "a track's sha256 is 64 hex digits, in lowercase",
                );
            }
            // The licence is of one file: the one in the project folder
            // must be it.
            let inside = !track.file.is_empty()
                && Path::new(&track.file)
                    .components()
                    .all(|part| matches!(part, std::path::Component::Normal(_)));
            if !inside {
                self.add(
                    at("file"),
                    "a track's file is a path inside the project folder",
                );
            } else if let Some(dir) = dir {
                match crate::capture::file_sha256(dir, &track.file) {
                    None => self.add(at("file"), format!("{} isn't there", track.file)),
                    Some(found) if hex && found != track.sha256 => self.add(
                        at("sha256"),
                        format!(
                            "{} isn't the file this entry is for: its SHA-256 is {found}",
                            track.file
                        ),
                    ),
                    Some(_) => {}
                }
            }
        }
        if let Some(under) = music.under
            && !(6.0..=40.0).contains(&under)
        {
            self.add(
                path!["music", "under"],
                "music plays from 6 to 40 LU under the narration",
            );
        }
        if music.track == "none" {
            return;
        }
        let Some(track) = music.playing() else {
            let names = one_of(music.library.iter().map(|t| t.id.as_str()));
            self.add(
                path!["music", "track"],
                format!(
                    "unknown track {:?}{names}; a track comes from [[music.library]], with its licence",
                    music.track
                ),
            );
            return;
        };
        for cut in &p.cuts {
            for platform in cut.platforms() {
                if !track.platforms.contains(&platform) {
                    self.add(
                        path!["music", "track"],
                        format!(
                            "track {:?} isn't licensed for {platform}, which cut {:?} is made for",
                            track.id, cut.id
                        ),
                    );
                }
            }
        }
    }

    fn project_file(&mut self, at: Vec<Seg>, url: &str, dir: Option<&Path>) {
        let Some(dir) = dir else { return };
        let file = url.split(['?', '#']).next().unwrap_or_default();
        if Path::new(file).is_absolute() || file.split('/').any(|part| part == "..") {
            self.add(
                at,
                format!("{file:?} must be a path inside the project folder"),
            );
            return;
        }
        for aspect in ["16x9", "9x16"] {
            let name = file.replace("{aspect}", aspect);
            if !dir.join(&name).is_file() {
                self.add(at, format!("{name} is not in the project folder"));
                return;
            }
        }
    }

    fn target(&mut self, at: Vec<Seg>, t: &Target) {
        let with = |key: &str| {
            let mut p = at.clone();
            p.push(Seg::from(key));
            p
        };
        if t.rect.is_some() {
            self.add(
                with("rect"),
                "rect names a part of an image scene's picture",
            );
        }
        if t.lines.is_some() {
            self.add(
                with("lines"),
                "lines names a part of a terminal or code scene's text",
            );
        }
        match (&t.css, t.any.is_empty()) {
            (None, true) => self.add(at.clone(), "a target needs css, or any = [matchers]"),
            (Some(_), false) => self.add(with("any"), "set css or any, not both"),
            _ => {}
        }
        if !t.any.is_empty() {
            for (key, set) in [
                ("text", t.text.is_some()),
                ("text_of", t.text_of.is_some()),
                ("exclude", t.exclude.is_some()),
                ("attr", t.attr.is_some()),
                ("line", t.line.is_some()),
                ("grow", t.grow.is_some()),
            ] {
                if set {
                    self.add(
                        with(key),
                        format!("with any, put {key} inside each matcher"),
                    );
                }
            }
            for (j, sub) in t.any.iter().enumerate() {
                let mut p = with("any");
                p.push(Seg::from(j));
                self.target(p, sub);
            }
        }
        if t.text_of.is_some() && t.text.is_none() && t.exclude.is_none() {
            self.add(
                with("text_of"),
                "text_of says where text and exclude look; set one of them",
            );
        }
        for (key, rule) in [("text", &t.text), ("exclude", &t.exclude)] {
            if let Some(TextMatch::Rule(rule)) = rule {
                let keys = rule.keys();
                if keys.len() != 1 {
                    self.add(
                        with(key),
                        "a text rule has exactly one of starts_with, contains or matches",
                    );
                } else if let Some(problem) = rule.matches.as_deref().and_then(page_regex_problem) {
                    self.add(with(key), problem);
                }
            }
        }
        if let Some(problem) = t.attr.as_ref().and_then(|a| page_regex_problem(&a.matches)) {
            self.add(with("attr"), problem);
        }
        if t.merge
            && matches!(
                t.pick,
                Some(crate::project::Pick::First | crate::project::Pick::Last)
            )
        {
            self.add(
                with("merge"),
                "merge joins every match, so pick must be all",
            );
        }
    }

    fn camera(&mut self, scene_id: &str, scene: &Scene, id: &str, c: &Camera) {
        let at = path!["scenes", scene_id, "cameras", id];
        let with = |key: &str| {
            let mut p = at.clone();
            p.push(Seg::from(key));
            p
        };
        if id == "viewport" || id == "highlights" {
            self.add(
                at.clone(),
                format!("{id:?} is a built-in camera; pick another name"),
            );
        } else if !valid_id(id) {
            self.add(at.clone(), id_rule("camera", id));
        }
        match (c.union.is_empty(), c.viewport) {
            (true, false) => self.add(
                at.clone(),
                "a camera frames union = [targets], or viewport = true",
            ),
            (false, true) => self.add(with("viewport"), "set union or viewport, not both"),
            _ => {}
        }
        for (j, target) in c.union.iter().enumerate() {
            if !scene.targets.contains_key(target) {
                let mut p = with("union");
                p.push(Seg::from(j));
                self.unknown_target(p, target, scene_id, scene);
            }
        }
        if let Some(vertical) = &c.vertical {
            self.camera_ref(with("vertical"), vertical, scene_id, scene);
        }
    }

    fn cuts(&mut self, p: &Project) {
        if p.cuts.is_empty() {
            self.add(path!["cuts"], "there are no cuts; add a [[cuts]] table");
        }
        let order: HashMap<&str, usize> = p
            .lines
            .iter()
            .enumerate()
            .map(|(i, l)| (l.id.as_str(), i))
            .collect();
        let mut first_use: HashMap<&str, usize> = HashMap::new();
        for (c, cut) in p.cuts.iter().enumerate() {
            let at = |key: &str| path!["cuts", c, key];
            if !valid_id(&cut.id) {
                self.add(at("id"), id_rule("cut", &cut.id));
            } else if let Some(first) = first_use.insert(&cut.id, c) {
                self.add(
                    at("id"),
                    format!("cut ID {:?} is already used by cut {}", cut.id, first + 1),
                );
            }
            // A dubbed cut: its language, and its lines' words in it.
            if let Some(language) = &cut.language {
                let tag = language.split('-').all(|part| {
                    (2..=8).contains(&part.len()) && part.chars().all(|c| c.is_ascii_alphanumeric())
                });
                if !tag {
                    self.add(
                        at("language"),
                        format!("{language:?} isn't a language tag, such as \"ta\" or \"pt-BR\""),
                    );
                } else if *language == p.meta.language {
                    self.add(
                        at("language"),
                        format!("{language:?} is the script's own language: leave language out"),
                    );
                } else {
                    let untranslated: Vec<&str> = cut
                        .lines_in(p)
                        .into_iter()
                        .filter(|line| {
                            line.translations
                                .get(language)
                                .is_none_or(|words| words.show.trim().is_empty())
                        })
                        .map(|line| line.id.as_str())
                        .collect();
                    if let Some(first) = untranslated.first() {
                        self.add(
                            at("language"),
                            format!(
                                "cut {} speaks {language}, and {} of its lines {} no in.{language}.show (the first: {first})",
                                cut.id,
                                untranslated.len(),
                                if untranslated.len() == 1 { "has" } else { "have" }
                            ),
                        );
                    }
                }
            }
            if let LineSelection::Ids(ids) = &cut.lines {
                if ids.is_empty() {
                    self.add(at("lines"), "the cut uses no lines");
                }
                let mut previous: Option<usize> = None;
                for (j, id) in ids.iter().enumerate() {
                    match order.get(id.as_str()) {
                        None => {
                            self.add(path!["cuts", c, "lines", j], format!("unknown line {id:?}"))
                        }
                        Some(&pos) => {
                            if !p.lines[pos].is_for(cut) {
                                self.add(
                                    path!["cuts", c, "lines", j],
                                    format!(
                                        "line {id:?} is only for {}",
                                        p.lines[pos]
                                            .only
                                            .iter()
                                            .map(|c| format!("{c:?}"))
                                            .collect::<Vec<_>>()
                                            .join(" and ")
                                    ),
                                );
                            }
                            if previous.is_some_and(|prev| pos <= prev) {
                                self.add(
                                    path!["cuts", c, "lines", j],
                                    format!("line {id:?} is out of script order; a cut keeps the script's order"),
                                );
                            }
                            previous = Some(pos);
                        }
                    }
                }
            }
            let known: Vec<String> = crate::platforms::all().into_iter().map(|p| p.id).collect();
            for (j, platform) in cut.platforms.iter().flatten().enumerate() {
                if !known.contains(platform) {
                    self.add(
                        path!["cuts", c, "platforms", j],
                        format!(
                            "unknown platform {platform:?}{}",
                            one_of(known.iter().map(String::as_str))
                        ),
                    );
                }
            }
            if let Some([w, h]) = cut.size {
                let landscape = cut.aspect == crate::project::Aspect::Landscape;
                let ratio_ok = if landscape {
                    w * 9 == h * 16
                } else {
                    w * 16 == h * 9
                };
                if w == 0 || h == 0 || w % 2 == 1 || h % 2 == 1 || !ratio_ok {
                    let aspect = if landscape { "16:9" } else { "9:16" };
                    self.add(
                        at("size"),
                        format!(
                            "the size must be even numbers in exactly {aspect}, like {}",
                            if landscape {
                                "[1920, 1080]"
                            } else {
                                "[1080, 1920]"
                            }
                        ),
                    );
                }
            }
        }
    }

    fn expectation(&mut self, p: &Project, i: usize, x: &Expectation) {
        let at = |key: &str| path!["expectations", i, key];
        match p.scenes.get(&x.scene) {
            None => {
                let names = one_of(p.scenes.keys().map(String::as_str));
                self.add(at("scene"), format!("unknown scene {:?}{names}", x.scene));
            }
            Some(s) if s.kind != SceneKind::Web => {
                self.add(
                    at("scene"),
                    format!(
                        "expectations check web scenes, and {:?} is a page scene",
                        x.scene
                    ),
                );
            }
            Some(s) => {
                if !s.targets.contains_key(&x.target) {
                    self.unknown_target(at("target"), &x.target, &x.scene, s);
                }
                if x.at != "setup" {
                    match p.lines.iter().find(|l| l.id == x.at) {
                        None => self.add(
                            at("at"),
                            format!("{:?} is neither \"setup\" nor a line ID", x.at),
                        ),
                        Some(line) => {
                            let shown = line.scene.as_deref().or(p.meta.default_scene.as_deref());
                            if shown != Some(x.scene.as_str()) {
                                self.add(
                                    at("at"),
                                    format!("line {:?} doesn't show scene {:?}", x.at, x.scene),
                                );
                            }
                        }
                    }
                }
            }
        }
        match (x.text.is_some(), x.texts.is_some()) {
            (true, true) => self.add(at("texts"), "set text or texts, not both"),
            (false, false) => self.add(
                path!["expectations", i],
                "an expectation needs text or texts",
            ),
            _ => {}
        }
    }
}

fn camera_exists(scene: &Scene, camera: &str) -> bool {
    camera == "viewport" || camera == "highlights" || scene.cameras.contains_key(camera)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positions_count_lines_and_characters_from_one() {
        let text = "a = 1\nb = \"é\" x\n";
        assert_eq!(position(text, 0), (1, 1));
        assert_eq!(position(text, 6), (2, 1));
        let x = text.find('x').unwrap();
        assert_eq!(position(text, x), (2, 9));
    }

    #[test]
    fn parser_messages_speak_toml() {
        assert_eq!(
            plain("invalid type: string \"fast\", expected f64"),
            "invalid type: string \"fast\", expected a number"
        );
        assert_eq!(
            plain("invalid type: sequence, expected struct Meta"),
            "invalid type: array, expected a table"
        );
    }

    #[test]
    fn ids_are_lowercase_words_joined_by_hyphens() {
        for good in ["intro", "intro-1", "a1-b2-c3"] {
            assert!(valid_id(good), "{good}");
        }
        for bad in [
            "", "Intro", "intro_1", "intro--1", "-intro", "intro-", "intro 1", "ïntro",
        ] {
            assert!(!valid_id(bad), "{bad}");
        }
    }
}
