//! Edits to a project file that touch only what they name.
//!
//! The desktop app changes a line or a scene through [`ProjectFile`], and the
//! rest of the file stays byte for byte as it was: every other key, comment
//! and blank line. A changed value keeps the comment after it, a new key goes
//! where `vidni fmt` would put it, and a moved line takes its comments along.

use crate::fmt::{LINE, META, SCENE, rank};
use crate::validate::{self, Diagnostic};
use std::fmt;
use toml_edit::{Array, ArrayOfTables, DocumentMut, Item, Table, Value};

/// Why an edit was refused. The file is unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditError {
    UnknownLine(String),
    UnknownScene(String),
    LineExists(String),
    /// The key can't be removed: every line needs it.
    Required(&'static str),
}

impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EditError::UnknownLine(id) => write!(f, "there is no line {id:?}"),
            EditError::UnknownScene(id) => write!(f, "there is no scene {id:?} with its own table"),
            EditError::LineExists(id) => write!(f, "a line {id:?} already exists"),
            EditError::Required(key) => write!(f, "every line needs {key}"),
        }
    }
}

impl std::error::Error for EditError {}

/// The string keys of a line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineField {
    Section,
    Show,
    Say,
    Source,
    Scene,
    Camera,
    CameraVertical,
    Overlay,
}

impl LineField {
    pub fn key(self) -> &'static str {
        match self {
            LineField::Section => "section",
            LineField::Show => "show",
            LineField::Say => "say",
            LineField::Source => "source",
            LineField::Scene => "scene",
            LineField::Camera => "camera",
            LineField::CameraVertical => "camera_vertical",
            LineField::Overlay => "overlay",
        }
    }

    fn required(self) -> bool {
        matches!(self, LineField::Section | LineField::Show)
    }
}

/// A project file open for editing.
#[derive(Debug, Clone)]
pub struct ProjectFile {
    doc: DocumentMut,
}

impl ProjectFile {
    /// Opens a valid project file's text.
    pub fn parse(text: &str) -> Result<Self, Vec<Diagnostic>> {
        validate::check(text, None)?;
        let doc = text.parse().map_err(|_| Vec::new())?;
        Ok(Self { doc })
    }

    /// The file's text, with the edits made so far.
    pub fn text(&self) -> String {
        self.doc.to_string()
    }

    /// The IDs of the lines, in script order.
    pub fn line_ids(&self) -> Vec<String> {
        self.lines()
            .map(|lines| {
                lines
                    .iter()
                    .filter_map(|t| t.get("id").and_then(Item::as_str).map(String::from))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Sets a line's key, or removes it with `None`.
    pub fn set_line(
        &mut self,
        id: &str,
        field: LineField,
        value: Option<&str>,
    ) -> Result<(), EditError> {
        let i = self.line_index(id)?;
        let line = self.lines_mut().get_mut(i).expect("index from line_index");
        match value {
            Some(v) => set_value(line, field.key(), Value::from(v), LINE),
            None if field.required() => return Err(EditError::Required(field.key())),
            None => {
                line.remove(field.key());
            }
        }
        Ok(())
    }

    /// Sets the targets a line highlights; an empty list removes the key.
    pub fn set_highlight(&mut self, id: &str, targets: &[&str]) -> Result<(), EditError> {
        let i = self.line_index(id)?;
        let line = self.lines_mut().get_mut(i).expect("index from line_index");
        if targets.is_empty() {
            line.remove("highlight");
        } else {
            let list: Array = targets.iter().copied().collect();
            set_value(line, "highlight", Value::Array(list), LINE);
        }
        Ok(())
    }

    /// Inserts a new line after the line `after`, or first when `after` is `None`.
    pub fn insert_line(
        &mut self,
        after: Option<&str>,
        id: &str,
        section: &str,
        show: &str,
    ) -> Result<(), EditError> {
        if self.line_ids().iter().any(|l| l == id) {
            return Err(EditError::LineExists(id.into()));
        }
        let at = match after {
            Some(a) => self.line_index(a)? + 1,
            None => 0,
        };
        let mut table = Table::new();
        table.insert("id", toml_edit::value(id));
        table.insert("section", toml_edit::value(section));
        table.insert("show", toml_edit::value(show));
        table.decor_mut().set_prefix("\n");
        self.place_line(at, table);
        Ok(())
    }

    /// Removes a line, with its comments.
    pub fn remove_line(&mut self, id: &str) -> Result<(), EditError> {
        let i = self.line_index(id)?;
        self.lines_mut().remove(i);
        Ok(())
    }

    /// Moves a line, with its comments, after the line `after`, or first
    /// when `after` is `None`.
    pub fn move_line(&mut self, id: &str, after: Option<&str>) -> Result<(), EditError> {
        let from = self.line_index(id)?;
        if let Some(a) = after {
            self.line_index(a)?;
        }
        if after == Some(id) {
            return Ok(());
        }
        let table = self.lines_mut().remove(from);
        let at = match after {
            Some(a) => self.line_index(a)? + 1,
            None => 0,
        };
        self.place_line(at, table);
        Ok(())
    }

    /// Sets a key of a scene that has its own `[scenes.<id>]` table.
    pub fn set_scene_value(
        &mut self,
        scene: &str,
        key: &str,
        value: impl Into<Value>,
    ) -> Result<(), EditError> {
        let table = self
            .doc
            .get_mut("scenes")
            .and_then(Item::as_table_mut)
            .and_then(|scenes| scenes.get_mut(scene))
            .and_then(Item::as_table_mut)
            .ok_or_else(|| EditError::UnknownScene(scene.into()))?;
        set_value(table, key, value.into(), SCENE);
        Ok(())
    }

    /// Removes a scene and everything under it.
    pub fn remove_scene(&mut self, scene: &str) -> Result<(), EditError> {
        self.doc
            .get_mut("scenes")
            .and_then(Item::as_table_mut)
            .and_then(|scenes| scenes.remove(scene))
            .map(|_| ())
            .ok_or_else(|| EditError::UnknownScene(scene.into()))
    }

    /// Sets the video's title.
    pub fn set_title(&mut self, title: &str) {
        if let Some(meta) = self.doc.get_mut("meta").and_then(Item::as_table_mut) {
            set_value(meta, "title", Value::from(title), META);
        }
    }

    fn lines(&self) -> Option<&ArrayOfTables> {
        self.doc.get("lines").and_then(Item::as_array_of_tables)
    }

    fn lines_mut(&mut self) -> &mut ArrayOfTables {
        self.doc
            .get_mut("lines")
            .and_then(Item::as_array_of_tables_mut)
            .expect("a valid project has [[lines]]")
    }

    fn line_index(&self, id: &str) -> Result<usize, EditError> {
        self.line_ids()
            .iter()
            .position(|l| l == id)
            .ok_or_else(|| EditError::UnknownLine(id.into()))
    }

    /// Puts a line table at index `at` of `[[lines]]`, and gives it the
    /// document position that prints it there.
    fn place_line(&mut self, at: usize, mut table: Table) {
        let lines = self.lines_mut();
        let position = if at == 0 {
            lines.get(0).and_then(Table::position).map_or(0, |p| p - 1)
        } else {
            lines.get(at - 1).and_then(Table::position).unwrap_or(0)
        };
        shift_after(self.doc.as_table_mut(), position);
        table.set_position(Some(position + 1));
        self.lines_mut().insert(at, table);
    }
}

/// Moves every table printed after `position` one place later.
fn shift_after(table: &mut Table, position: isize) {
    if let Some(p) = table.position()
        && p > position
    {
        table.set_position(Some(p + 1));
    }
    for (_, item) in table.iter_mut() {
        match item {
            Item::Table(sub) => shift_after(sub, position),
            Item::ArrayOfTables(tables) => {
                for sub in tables.iter_mut() {
                    shift_after(sub, position);
                }
            }
            _ => {}
        }
    }
}

/// Replaces a key's value and keeps the comments around it, or adds the key
/// where `vidni fmt` would put it among the keys already there.
fn set_value(table: &mut Table, key: &str, mut value: Value, order: &[&str]) {
    if let Some(Item::Value(old)) = table.get_mut(key) {
        *value.decor_mut() = old.decor().clone();
        *old = value;
        return;
    }
    let mut keys: Vec<String> = table.iter().map(|(k, _)| k.to_string()).collect();
    let at = keys
        .iter()
        .position(|k| rank(order, k) > rank(order, key))
        .unwrap_or(keys.len());
    keys.insert(at, key.to_string());
    table.insert(key, Item::Value(value));
    let place = |k: &str| keys.iter().position(|x| x == k).unwrap_or(usize::MAX);
    table.sort_values_by(|k1, _, k2, _| place(k1.get()).cmp(&place(k2.get())));
}
