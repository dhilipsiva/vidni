//! Loading and checking a project file.
//!
//! The parser reports syntax errors, wrong types and unknown keys. Every rule
//! in `docs/spec.md` that reaches across tables (lines naming sections and
//! scenes, highlights naming targets, cuts naming lines) is checked here.
//! Each problem is reported at its line and column.

use crate::project::{
    Action, Camera, Expectation, LineSelection, Project, SCHEMA, Scene, SceneKind, Target,
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
        if p.music.track != "none" {
            self.add(
                path!["music", "track"],
                "schema 1 has no music library yet (P6-VIDNI-06), so track must be \"none\"",
            );
        }
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
                    Some(s) if s.kind != SceneKind::Page || s.transparent != Some(true) => {
                        self.add(
                            at("overlay"),
                            format!(
                                "overlay {overlay:?} must be a page scene with transparent = true"
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
            SceneKind::Page => {
                for (key, set) in [
                    ("do", !line.actions.is_empty()),
                    ("highlight", !line.highlight.is_empty()),
                    ("camera", line.camera.is_some()),
                    ("camera_vertical", line.camera_vertical.is_some()),
                ] {
                    if set {
                        self.add(
                            at(key),
                            format!("{key} needs a web scene, and {id:?} is a page scene"),
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
        if s.url.trim().is_empty() {
            self.add(at("url"), "url is empty");
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
        }
    }

    /// A file the project names, relative to its folder, with `{aspect}` filled in.
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
