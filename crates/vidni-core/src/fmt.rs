//! The canonical layout of `vidni.toml`, written by `vidni fmt`.
//!
//! Formatting keeps every comment, and every value as written: strings keep
//! their quotes and escapes, and numbers their spelling. It fixes the rest:
//! - Tables come in schema order: meta, voice, timing, lint, style, cuts,
//!   music, sections (in script order), lines, scenes, expectations.
//! - Keys come in schema order within each table.
//! - `key = value` has single spaces, and inline tables read `{ a = 1, b = 2 }`.
//! - Each table header has one blank line before it. Keys have none, except
//!   before a comment that starts a group of keys.
//! - An array stays on one line when its line fits in [`WIDTH`] columns and
//!   it holds no comments. Otherwise each element gets its own line, indented
//!   two spaces, with a trailing comma.
//! - A trailing comment sits one space after its value.
//! - Dotted tables become headed tables, and the file ends with one newline.

use crate::project::Project;
use crate::validate::{self, Diagnostic};
use std::collections::HashMap;
use toml_edit::{Array, Decor, DocumentMut, InlineTable, Item, RawString, Table, Value};

/// The widest line an array may share with its key.
pub const WIDTH: usize = 100;

/// The file in canonical layout. Only a valid project is formatted.
pub fn format(text: &str) -> Result<String, Vec<Diagnostic>> {
    let project = validate::check(text, None)?;
    let mut doc: DocumentMut = text.parse().map_err(|_| Vec::new())?;
    canonicalize(&mut doc, &project);
    Ok(doc.to_string())
}

/// Puts a parsed, valid project into canonical layout in place.
pub fn canonicalize(doc: &mut DocumentMut, project: &Project) {
    let mut first_line: HashMap<&str, usize> = HashMap::new();
    for (i, line) in project.lines.iter().enumerate() {
        first_line.entry(line.section.as_str()).or_insert(i);
    }
    let trailing = comment_lines(raw(doc.trailing()));
    doc.set_trailing(if trailing.is_empty() {
        String::new()
    } else {
        format!("\n{trailing}")
    });
    let mut walk = Walk {
        position: 0,
        first_line,
    };
    walk.table(doc.as_table_mut(), &mut Vec::new());
}

/// The key order of each kind of table, as in `project.rs`.
pub(crate) const ROOT: &[&str] = &[
    "schema",
    "meta",
    "voice",
    "timing",
    "lint",
    "style",
    "cuts",
    "music",
    "sections",
    "lines",
    "scenes",
    "expectations",
];
pub(crate) const META: &[&str] = &["title", "subtitle", "language", "strict", "default_scene"];
const VOICE: &[&str] = &[
    "draft",
    "seed",
    "length_scale",
    "read_as",
    "respell",
    "heard_as",
];
const RESPELLING: &[&str] = &["text", "say"];
const HEARD_AS: &[&str] = &["heard", "means"];
const TIMING: &[&str] = &["lead_in", "line_gap", "section_gap", "tail"];
const LINT: &[&str] = &["banned"];
const STYLE: &[&str] = &["theme"];
const MUSIC: &[&str] = &["track", "under", "library"];
const TRACK: &[&str] = &[
    "id",
    "file",
    "sha256",
    "licence",
    "attribution",
    "platforms",
    "content_id",
];
const SECTION: &[&str] = &["label", "chapter"];
pub(crate) const LINE: &[&str] = &[
    "id",
    "section",
    "show",
    "say",
    "source",
    "claim",
    "scene",
    "do",
    "highlight",
    "camera",
    "camera_vertical",
    "overlay",
    "only",
];
pub(crate) const SCENE: &[&str] = &[
    "kind",
    "url",
    "template",
    "title",
    "subtitle",
    "items",
    "quote",
    "by",
    "viewport",
    "scale",
    "color_scheme",
    "default_camera",
    "vertical_follows_highlights",
    "setup",
    "transparent",
    "size",
    "targets",
    "cameras",
];
const PAGE_SIZE: &[&str] = &["landscape", "portrait"];
const TARGET: &[&str] = &[
    "css", "rect", "any", "text_of", "text", "exclude", "attr", "pick", "merge", "line", "grow",
    "unique",
];
const TEXT_RULE: &[&str] = &["starts_with", "contains", "matches"];
const ATTR: &[&str] = &["name", "matches"];
const GROW: &[&str] = &["within", "max_height", "margin"];
const CAMERA: &[&str] = &["union", "viewport", "vertical"];
const ACTION: &[&str] = &[
    "click",
    "select",
    "value",
    "expand",
    "collapse",
    "control",
    "scroll",
    "top",
    "scroll_to",
    "within",
    "offset",
    "wait",
];
const CUT: &[&str] = &["id", "aspect", "lines", "captions", "platforms", "size"];
const EXPECTATION: &[&str] = &["scene", "at", "target", "text", "texts"];

/// The key order of the table or inline table at `path` ("#" stands for an
/// array index), or `None` for a map whose order is the author's.
fn key_order(path: &[String]) -> Option<&'static [&'static str]> {
    let p: Vec<&str> = path.iter().map(String::as_str).collect();
    match p.as_slice() {
        [] => Some(ROOT),
        ["meta"] => Some(META),
        ["voice"] => Some(VOICE),
        ["voice", "read_as" | "respell", "#"] => Some(RESPELLING),
        ["voice", "heard_as", "#"] => Some(HEARD_AS),
        ["timing"] => Some(TIMING),
        ["lint"] => Some(LINT),
        ["style"] => Some(STYLE),
        ["music"] => Some(MUSIC),
        ["music", "library", "#"] => Some(TRACK),
        ["sections", _] => Some(SECTION),
        ["lines", "#"] => Some(LINE),
        ["lines", "#", "do", "#"] | ["scenes", _, "setup", "#"] => Some(ACTION),
        ["scenes", _] => Some(SCENE),
        ["scenes", _, "size"] => Some(PAGE_SIZE),
        ["scenes", _, "cameras", _] => Some(CAMERA),
        ["scenes", _, "targets", _, rest @ ..] => target_part(rest),
        ["cuts", "#"] => Some(CUT),
        ["expectations", "#"] => Some(EXPECTATION),
        _ => None,
    }
}

fn target_part(rest: &[&str]) -> Option<&'static [&'static str]> {
    match rest {
        [] | ["any", "#"] => Some(TARGET),
        ["any", "#", more @ ..] => target_part(more),
        ["text" | "exclude"] => Some(TEXT_RULE),
        ["attr"] => Some(ATTR),
        ["grow"] => Some(GROW),
        _ => None,
    }
}

/// A key's place in `order`; unknown keys go last, in their own order.
pub(crate) fn rank(order: &[&str], key: &str) -> usize {
    order.iter().position(|k| *k == key).unwrap_or(order.len())
}

struct Walk<'p> {
    position: isize,
    first_line: HashMap<&'p str, usize>,
}

impl Walk<'_> {
    fn table(&mut self, t: &mut Table, path: &mut Vec<String>) {
        if let Some(order) = key_order(path) {
            t.sort_values_by(|k1, _, k2, _| rank(order, k1.get()).cmp(&rank(order, k2.get())));
        } else if path.len() == 1 && path[0] == "sections" {
            let first = &self.first_line;
            t.sort_values_by(|k1, _, k2, _| {
                let at = |k: &str| first.get(k).copied().unwrap_or(usize::MAX);
                at(k1.get()).cmp(&at(k2.get()))
            });
        }
        if !path.is_empty() && !t.is_implicit() {
            self.position += 1;
            t.set_position(Some(self.position));
            let prefix = format!("\n{}", comment_lines(prefix_of(t.decor())));
            let suffix = trailing_comment(t.decor().suffix());
            t.decor_mut().set_prefix(prefix);
            t.decor_mut().set_suffix(suffix);
        }
        let mut first = true;
        for (mut key, item) in t.iter_mut() {
            path.push(key.get().to_string());
            match item {
                Item::Value(v) => {
                    let prefix = key_prefix(key.leaf_decor(), first, path.len() == 1);
                    key.leaf_decor_mut().set_prefix(prefix);
                    key.leaf_decor_mut().set_suffix(" ");
                    let width = key.display_repr().chars().count() + " = ".len();
                    value(v, path, width);
                    first = false;
                }
                Item::Table(sub) => {
                    sub.set_dotted(false);
                    self.table(sub, path);
                }
                Item::ArrayOfTables(tables) => {
                    for sub in tables.iter_mut() {
                        path.push("#".into());
                        self.table(sub, path);
                        path.pop();
                    }
                }
                Item::None => {}
            }
            path.pop();
        }
    }
}

/// A key's comments: none of its own blank lines, except one before a
/// comment that starts a group (not at the top of a table), and the file's
/// leading comment, which keeps one blank line after it.
fn key_prefix(decor: &Decor, first_in_table: bool, at_root: bool) -> String {
    let block = Block::of(prefix_of(decor));
    if block.comments.is_empty() {
        return String::new();
    }
    let lines = block.render();
    if at_root && first_in_table {
        // The file's leading comment.
        return if block.blank_after {
            format!("{lines}\n")
        } else {
            lines
        };
    }
    if block.blank_before && !first_in_table {
        format!("\n{lines}")
    } else {
        lines
    }
}

fn value(v: &mut Value, path: &mut Vec<String>, width: usize) {
    let comment = trailing_comment(v.decor().suffix());
    match v {
        Value::Array(a) => array(a, path, width + comment.chars().count()),
        Value::InlineTable(t) => inline_table(t, path),
        _ => {}
    }
    v.decor_mut().set_prefix(" ");
    v.decor_mut().set_suffix(comment);
}

/// An array that is a key's value: on one line when it fits, else one
/// element per line.
fn array(a: &mut Array, path: &mut Vec<String>, used: usize) {
    for el in a.iter_mut() {
        path.push("#".into());
        element(el, path);
        path.pop();
    }
    let commented = a.iter().any(|v| has_comment(v.decor())) || raw(a.trailing()).contains('#');
    let mut single = a.clone();
    single.fmt();
    single.decor_mut().clear();
    let text = single.to_string();
    if !commented && !text.contains('\n') && used + text.chars().count() <= WIDTH {
        let decor = a.decor().clone();
        *a = single;
        *a.decor_mut() = decor;
        return;
    }
    // One element per line. A comment that follows an element on its line
    // reaches the parser as the next element's prefix (or the trailing).
    let mut carry: Option<String> = None;
    for el in a.iter_mut() {
        let (same_line, lines) = split_prefix(prefix_of(el.decor()));
        let after_previous = join_comments(carry.take(), same_line);
        let mut prefix = after_previous.map(|c| format!(" {c}")).unwrap_or_default();
        prefix.push('\n');
        for line in lines {
            prefix.push_str(&format!("  {line}\n"));
        }
        prefix.push_str("  ");
        carry = comment_text(el.decor().suffix().map(raw).unwrap_or_default());
        el.decor_mut().set_prefix(prefix);
        el.decor_mut().set_suffix("");
    }
    let (same_line, lines) = split_prefix(raw(a.trailing()));
    let mut trailing = join_comments(carry, same_line)
        .map(|c| format!(" {c}"))
        .unwrap_or_default();
    trailing.push('\n');
    for line in lines {
        trailing.push_str(&format!("  {line}\n"));
    }
    a.set_trailing(trailing);
    a.set_trailing_comma(!a.is_empty());
}

/// An element of an array, or a value inside an inline table: always on one line.
fn element(v: &mut Value, path: &mut Vec<String>) {
    match v {
        Value::InlineTable(t) => inline_table(t, path),
        Value::Array(a) => {
            for el in a.iter_mut() {
                path.push("#".into());
                element(el, path);
                path.pop();
            }
            if !raw(a.trailing()).contains('#') && !a.iter().any(|el| has_comment(el.decor())) {
                let decor = a.decor().clone();
                a.fmt();
                *a.decor_mut() = decor;
            }
        }
        _ => {}
    }
}

fn inline_table(t: &mut InlineTable, path: &mut Vec<String>) {
    // TOML 1.1 lets an inline table span lines and hold comments; leave such a table as written.
    if raw(t.trailing()).contains('#') || t.iter().any(|(_, v)| has_comment(v.decor())) {
        return;
    }
    if let Some(order) = key_order(path) {
        t.sort_values_by(|k1, _, k2, _| rank(order, k1.get()).cmp(&rank(order, k2.get())));
    }
    for (key, v) in t.iter_mut() {
        path.push(key.get().to_string());
        element(v, path);
        path.pop();
    }
    let decor = t.decor().clone();
    t.fmt();
    *t.decor_mut() = decor;
}

fn raw(s: &RawString) -> &str {
    s.as_str().unwrap_or_default()
}

fn prefix_of(decor: &Decor) -> &str {
    decor.prefix().map(raw).unwrap_or_default()
}

fn has_comment(decor: &Decor) -> bool {
    prefix_of(decor).contains('#') || decor.suffix().map(raw).unwrap_or_default().contains('#')
}

/// " # comment" from a suffix that holds one, else "".
fn trailing_comment(suffix: Option<&RawString>) -> String {
    comment_text(suffix.map(raw).unwrap_or_default())
        .map(|c| format!(" {c}"))
        .unwrap_or_default()
}

/// The comment in `s`, from its `#`, without trailing spaces.
fn comment_text(s: &str) -> Option<String> {
    s.find('#').map(|i| s[i..].trim_end().to_string())
}

fn join_comments(a: Option<String>, b: Option<String>) -> Option<String> {
    match (a, b) {
        (Some(a), Some(b)) => Some(format!("{a} {}", b.trim_start_matches('#').trim_start())),
        (a, b) => a.or(b),
    }
}

/// An array element's prefix: the comment that ends the previous line, then
/// the comment lines before the element.
fn split_prefix(s: &str) -> (Option<String>, Vec<String>) {
    match s.split_once('\n') {
        None => (None, Vec::new()),
        Some((same_line, rest)) => {
            let lines = rest
                .lines()
                .map(str::trim)
                .filter(|l| l.starts_with('#'))
                .map(|l| l.trim_end().to_string())
                .collect();
            (comment_text(same_line), lines)
        }
    }
}

/// The comment lines of `s`, each ending in a newline, with runs of blank
/// lines between them kept as one, and none before or after.
fn comment_lines(s: impl AsRef<str>) -> String {
    Block::of(s.as_ref()).render()
}

/// The comments of a prefix, and the blank lines around and between them.
struct Block {
    /// `None` marks a blank line between comments.
    comments: Vec<Option<String>>,
    blank_before: bool,
    blank_after: bool,
}

impl Block {
    fn of(s: &str) -> Block {
        // The text after the last newline is the indentation of what follows.
        let lines: Vec<&str> = match s.rfind('\n') {
            Some(i) => s[..i].split('\n').collect(),
            None => Vec::new(),
        };
        let mut comments: Vec<Option<String>> = Vec::new();
        let (mut blank_before, mut blank_after) = (false, false);
        for line in lines {
            let line = line.trim();
            if line.starts_with('#') {
                if blank_after && !comments.is_empty() {
                    comments.push(None);
                }
                blank_after = false;
                comments.push(Some(line.to_string()));
            } else if comments.is_empty() {
                blank_before = true;
            } else {
                blank_after = true;
            }
        }
        Block {
            comments,
            blank_before,
            blank_after,
        }
    }

    fn render(&self) -> String {
        self.comments
            .iter()
            .map(|c| format!("{}\n", c.as_deref().unwrap_or_default()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_keep_comments_and_one_blank_between_them() {
        let b = Block::of("\n\n# one\n\n\n# two\n\n  ");
        assert!(b.blank_before && b.blank_after);
        assert_eq!(b.render(), "# one\n\n# two\n");
        assert_eq!(Block::of("  ").render(), "");
    }

    #[test]
    fn array_prefixes_split_the_previous_lines_comment_from_their_own() {
        assert_eq!(
            split_prefix(" # after a\n  # before b\n  "),
            (Some("# after a".into()), vec!["# before b".into()])
        );
        assert_eq!(split_prefix(" "), (None, vec![]));
    }
}
