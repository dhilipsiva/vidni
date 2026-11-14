//! The script and scene editor (TODO.md, P7-VIDNI-02): forms over the
//! project's file, and the file's own text beside them.
//!
//! Every edit is made to the file as it is on disk at that moment, through
//! [`vidni_core::edit::ProjectFile`], which changes what the edit names and
//! leaves every other byte and comment where it was. So an edit made here
//! and an edit made by hand in another editor never undo each other. An
//! edit that would leave the file invalid isn't saved: what is wrong is
//! shown beside the form, at its line and column, and the file stays.

use eframe::egui;
use std::fs;
use std::path::{Path, PathBuf};
use vidni_core::edit::{EditError, LineField, ProjectFile};
use vidni_core::project::Project;
use vidni_core::validate;

/// Which face of the editor is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// The forms: the title, the lines by section, and the scenes.
    Forms,
    /// The file's text.
    Source,
}

/// The fields of the line that is selected.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct LineForm {
    show: String,
    say: String,
    section: String,
    scene: String,
    camera: String,
    camera_vertical: String,
    source: String,
}

impl LineForm {
    fn fields(&mut self) -> [(&'static str, LineField, &mut String); 7] {
        [
            ("Shown", LineField::Show, &mut self.show),
            ("Said, when it differs", LineField::Say, &mut self.say),
            ("Section", LineField::Section, &mut self.section),
            ("Scene", LineField::Scene, &mut self.scene),
            ("Camera", LineField::Camera, &mut self.camera),
            (
                "Camera, vertical",
                LineField::CameraVertical,
                &mut self.camera_vertical,
            ),
            ("Source of its facts", LineField::Source, &mut self.source),
        ]
    }
}

/// The string keys of a scene that the form edits.
const SCENE_KEYS: [(&str, &str); 5] = [
    ("url", "Address or file"),
    ("title", "Card's title"),
    ("subtitle", "Card's subtitle"),
    ("quote", "Quotation"),
    ("by", "Whose it is"),
];

/// A project's file, open in the editor.
pub struct Editor {
    folder: PathBuf,
    file: PathBuf,
    /// The file's text, as it was last read or written.
    pub text: String,
    /// What that text describes, when it is a valid project.
    pub project: Option<Project>,
    /// What is wrong with the file as it is on disk.
    pub problems: Vec<String>,
    /// Why the last edit wasn't saved; empty when it was.
    pub refused: Vec<String>,
    /// What changed under the forms, when a person should know.
    pub notice: String,
    /// Whether the file changed on disk while the source view held typing
    /// that isn't saved: that typing is of an older file.
    behind: bool,
    /// The line whose form is open.
    pub selected: Option<String>,
    pub view: View,
    form: LineForm,
    title: String,
    new_id: String,
    new_words: String,
    scene: Option<String>,
    scene_form: Vec<String>,
    source: String,
}

impl Editor {
    /// Opens the project file in `folder`.
    pub fn open(folder: &Path) -> Self {
        let mut editor = Self {
            folder: folder.to_path_buf(),
            file: validate::project_file(folder),
            text: String::new(),
            project: None,
            problems: Vec::new(),
            refused: Vec::new(),
            notice: String::new(),
            behind: false,
            selected: None,
            view: View::Forms,
            form: LineForm::default(),
            title: String::new(),
            new_id: String::new(),
            new_words: String::new(),
            scene: None,
            scene_form: Vec::new(),
            source: String::new(),
        };
        editor.read();
        editor
    }

    /// Takes in the file as it is on disk now.
    fn read(&mut self) {
        let text = match fs::read_to_string(&self.file) {
            Ok(text) => text,
            Err(e) => {
                self.project = None;
                self.problems = vec![format!("{}: {e}", self.file.display())];
                return;
            }
        };
        // Read on purpose, the file's text takes the place of any typing.
        self.source = self.text.clone();
        self.title = self
            .project
            .as_ref()
            .map_or(String::new(), |p| p.meta.title.clone());
        if let Some(form) = self.selected.as_deref().and_then(|id| self.line_form(id)) {
            self.form = form;
        }
        if let Some(form) = self.scene.as_deref().and_then(|id| self.scene_values(id)) {
            self.scene_form = form;
        }
        self.take(text);
    }

    /// The form of line `id`, as the file has it.
    fn line_form(&self, id: &str) -> Option<LineForm> {
        let line = self.project.as_ref()?.lines.iter().find(|l| l.id == id)?;
        let or_empty = |value: &Option<String>| value.clone().unwrap_or_default();
        Some(LineForm {
            show: line.show.clone(),
            say: or_empty(&line.say),
            section: line.section.clone(),
            scene: or_empty(&line.scene),
            camera: or_empty(&line.camera),
            camera_vertical: or_empty(&line.camera_vertical),
            source: or_empty(&line.source),
        })
    }

    /// The form of scene `id`, as the file has it.
    fn scene_values(&self, id: &str) -> Option<Vec<String>> {
        let scene = self.project.as_ref()?.scenes.get(id)?;
        let or_empty = |value: &Option<String>| value.clone().unwrap_or_default();
        Some(vec![
            scene.url.clone(),
            or_empty(&scene.title),
            or_empty(&scene.subtitle),
            or_empty(&scene.quote),
            or_empty(&scene.by),
        ])
    }

    /// Takes `text` as the file's text. A form that holds what the file
    /// had is filled from the new text; a form a person has typed in keeps
    /// their typing, which is applied to the file as it is then.
    fn take(&mut self, text: String) {
        // What was typed and not saved, before the file's text changes.
        let typed_line = self
            .selected
            .as_deref()
            .is_some_and(|id| self.line_form(id).is_some_and(|form| form != self.form));
        let typed_scene = self.scene.as_deref().is_some_and(|id| {
            self.scene_values(id)
                .is_some_and(|form| form != self.scene_form)
        });
        let typed_title = self
            .project
            .as_ref()
            .is_some_and(|p| p.meta.title != self.title);
        let typed_source = self.source != self.text;
        match validate::check(&text, Some(&self.folder)) {
            Ok(project) => {
                self.problems.clear();
                self.project = Some(project);
            }
            Err(problems) => {
                self.problems = problems.iter().map(ToString::to_string).collect();
                self.project = None;
            }
        }
        match typed_source {
            // The typing is of the file as it was: it can't be saved over
            // what the file is now.
            true => self.behind = text != self.text || self.behind,
            false => {
                self.source = text.clone();
                self.behind = false;
            }
        }
        self.text = text;
        // While the file isn't a valid project, the forms stay as they are.
        if self.project.is_none() {
            return;
        }
        if !typed_title {
            self.title = self
                .project
                .as_ref()
                .map_or(String::new(), |p| p.meta.title.clone());
        }
        self.notice.clear();
        if let Some(id) = self.selected.clone() {
            match self.line_form(&id) {
                Some(form) if !typed_line => self.form = form,
                Some(_) => {}
                None => {
                    self.selected = None;
                    self.notice =
                        format!("Line {id} is no longer in the file, so its form is closed.");
                }
            }
        }
        if let Some(id) = self.scene.clone() {
            match self.scene_values(&id) {
                Some(form) if !typed_scene => self.scene_form = form,
                Some(_) => {}
                None => self.scene = None,
            }
        }
    }

    /// Reads the file again when it has changed on disk since it was last
    /// read or written here: a hand edit. True when it had.
    pub fn refresh(&mut self) -> bool {
        match fs::read_to_string(&self.file) {
            Ok(text) if text != self.text => {
                self.take(text);
                true
            }
            Ok(_) => false,
            // The file is gone, or can't be read: the forms have nothing to edit.
            Err(e) => {
                let problem = format!("{}: {e}", self.file.display());
                let news = self.problems != [problem.clone()];
                self.problems = vec![problem];
                self.project = None;
                news
            }
        }
    }

    /// Opens the form of line `id`.
    pub fn select(&mut self, id: &str) {
        self.selected = self.line_form(id).map(|form| {
            self.form = form;
            id.to_string()
        });
    }

    /// Opens the form of scene `id`.
    pub fn select_scene(&mut self, id: &str) {
        self.scene = self.scene_values(id).map(|form| {
            self.scene_form = form;
            id.to_string()
        });
    }

    /// Makes `change` to the file as it is on disk now, and saves the
    /// result when it is a valid project. Otherwise the file stays, and
    /// [`Self::refused`] says why. True when the file was saved.
    pub fn edit(&mut self, change: impl FnOnce(&mut ProjectFile) -> Result<(), EditError>) -> bool {
        self.refused.clear();
        // The file may have been edited by hand since it was read.
        let on_disk = match fs::read_to_string(&self.file) {
            Ok(text) => text,
            Err(e) => {
                self.refused = vec![format!("{}: {e}", self.file.display())];
                return false;
            }
        };
        let mut file = match ProjectFile::parse(&on_disk) {
            Ok(file) => file,
            Err(problems) => {
                self.take(on_disk);
                self.refused = vec![
                    "the file has problems of its own, which the source view shows; \
                     a form changes only a file that is valid"
                        .to_string(),
                ];
                self.refused
                    .extend(problems.iter().map(ToString::to_string));
                return false;
            }
        };
        if let Err(e) = change(&mut file) {
            self.refused = vec![e.to_string()];
            return false;
        }
        // The editing library writes its lines Unix's way; a file that
        // ends its lines Windows' way keeps doing so.
        let mut text = file.text();
        if on_disk.contains("\r\n") {
            text = text.replace("\r\n", "\n").replace('\n', "\r\n");
        }
        self.save(text, false)
    }

    /// Saves `text` as the file when it is a valid project. A problem is
    /// said with its line and column when `text` is what the person sees,
    /// and without them when it is what a form would have made.
    fn save(&mut self, text: String, seen: bool) -> bool {
        if let Err(problems) = validate::check(&text, Some(&self.folder)) {
            self.refused = problems
                .iter()
                .map(|problem| match seen {
                    true => problem.to_string(),
                    false => format!("the file would then have a problem: {}", problem.message),
                })
                .collect();
            return false;
        }
        // Written beside the file and moved over it, so that the file is
        // never half of itself.
        let part = self.file.with_extension("toml.part");
        let written = fs::write(&part, &text).and_then(|()| fs::rename(&part, &self.file));
        if let Err(e) = written {
            self.refused = vec![format!("{}: {e}", self.file.display())];
            return false;
        }
        // Saved from the source view, the text there is the file's again.
        // Saved from a form, what is typed in the source view stays, as
        // typing of an older file.
        if seen {
            self.source = self.text.clone();
        }
        self.take(text);
        true
    }

    /// Saves the source view's text as the file, when it is a valid project.
    pub fn save_source(&mut self) -> bool {
        self.refused.clear();
        self.refresh();
        if self.behind {
            self.refused = vec![
                "the file changed on disk after this text was read, and saving it would undo that \
                 change; Read from disk takes the file in again, in the place of what is typed here"
                    .to_string(),
            ];
            return false;
        }
        let text = self.source.clone();
        self.save(text, true)
    }

    /// Saves the open line's form: each field that differs from the file.
    fn apply_line(&mut self) -> bool {
        let Some(id) = self.selected.clone() else {
            return false;
        };
        let Some(line) = self
            .project
            .as_ref()
            .and_then(|p| p.lines.iter().find(|l| l.id == id))
            .cloned()
        else {
            self.refused = vec![format!("there is no line {id:?} in the file any more")];
            return false;
        };
        let was = [
            Some(line.show),
            line.say,
            Some(line.section),
            line.scene,
            line.camera,
            line.camera_vertical,
            line.source,
        ];
        let mut form = self.form.clone();
        let changes: Vec<(LineField, Option<String>)> = form
            .fields()
            .into_iter()
            .zip(was)
            .filter_map(|((_, field, now), was)| {
                // An empty field is a key that isn't there; `show` and
                // `section` are always there, and may not be empty.
                let wanted = match (now.trim().is_empty(), field) {
                    (true, LineField::Show | LineField::Section) => Some(String::new()),
                    (true, _) => None,
                    (false, _) => Some(now.trim().to_string()),
                };
                (wanted != was).then_some((field, wanted))
            })
            .collect();
        if changes.is_empty() {
            self.refused.clear();
            return true;
        }
        self.edit(|file| {
            changes
                .iter()
                .try_for_each(|(field, value)| file.set_line(&id, *field, value.as_deref()))
        })
    }

    /// Removes line `id`, unless comments of its own would go with it.
    fn remove(&mut self, id: &str) -> bool {
        let commented = fs::read_to_string(&self.file)
            .ok()
            .and_then(|text| ProjectFile::parse(&text).ok())
            .is_some_and(|file| file.line_has_comments(id));
        if commented {
            self.refused = vec![format!(
                "line {id:?} has comments of its own in the file, which would go with it; remove it in the source view"
            )];
            return false;
        }
        let removed = self.edit(|file| file.remove_line(id));
        if removed {
            self.selected = None;
            self.notice.clear();
        }
        removed
    }

    /// The line before `id` in the script, and the one before that.
    fn before(&self, id: &str) -> (Option<String>, Option<String>) {
        let ids: Vec<&str> = self
            .project
            .iter()
            .flat_map(|p| p.lines.iter().map(|l| l.id.as_str()))
            .collect();
        let at = ids.iter().position(|l| *l == id).unwrap_or(0);
        let named = |i: Option<usize>| i.and_then(|i| ids.get(i)).map(|s| s.to_string());
        (named(at.checked_sub(1)), named(at.checked_sub(2)))
    }

    /// Draws the editor.
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        self.refresh();
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.view, View::Forms, "Forms");
            ui.selectable_value(&mut self.view, View::Source, "Source");
        });
        for problem in &self.problems {
            ui.colored_label(ui.visuals().error_fg_color, problem);
        }
        match self.view {
            View::Forms => self.forms(ui),
            View::Source => self.source_view(ui),
        }
        if !self.notice.is_empty() {
            ui.label(&self.notice);
        }
        if !self.refused.is_empty() {
            ui.add_space(6.0);
            ui.colored_label(ui.visuals().error_fg_color, "Not saved:");
            for reason in &self.refused {
                ui.colored_label(ui.visuals().error_fg_color, reason);
            }
        }
    }

    fn source_view(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Save source").clicked() {
                self.save_source();
            }
            if ui.button("Read from disk").clicked() {
                self.refused.clear();
                self.behind = false;
                self.read();
            }
        });
        if self.behind {
            ui.label(
                "The file changed on disk while this text was being typed. Read from disk takes \
                 the file in again; what is typed here goes.",
            );
        }
        let label = ui.label("The project's file");
        ui.add(
            egui::TextEdit::multiline(&mut self.source)
                .code_editor()
                .desired_width(f32::INFINITY)
                .desired_rows(24),
        )
        .labelled_by(label.id);
    }

    fn forms(&mut self, ui: &mut egui::Ui) {
        let Some(project) = self.project.clone() else {
            ui.label("The forms open once the file is a valid project; the source view shows it as it is.");
            return;
        };
        ui.horizontal(|ui| {
            let label = ui.label("Title");
            ui.text_edit_singleline(&mut self.title)
                .labelled_by(label.id);
            if ui.button("Apply title").clicked() {
                let title = self.title.clone();
                self.edit(|file| file.set_title(title.trim()));
            }
        });
        ui.add_space(8.0);
        ui.heading("Script");
        // The lines by section, each with its ID; one is picked to edit.
        let mut section = None;
        for line in &project.lines {
            if section != Some(&line.section) {
                section = Some(&line.section);
                let label = &project.sections.get(&line.section).map_or("", |s| &s.label);
                ui.strong(match label.is_empty() {
                    true => line.section.clone(),
                    false => format!("{}: {label}", line.section),
                });
            }
            let picked = self.selected.as_deref() == Some(&line.id);
            let row = format!("[{}]  {}", line.id, line.show);
            if ui.selectable_label(picked, row).clicked() {
                self.select(&line.id);
                self.refused.clear();
            }
        }
        if let Some(id) = self.selected.clone() {
            ui.add_space(8.0);
            ui.heading(format!("Line {id}"));
            egui::Grid::new("line").num_columns(2).show(ui, |ui| {
                for (name, _, value) in self.form.fields() {
                    let label = ui.label(name);
                    ui.add(egui::TextEdit::singleline(value).desired_width(560.0))
                        .labelled_by(label.id);
                    ui.end_row();
                }
            });
            let (before, two_before) = self.before(&id);
            let after = project
                .lines
                .iter()
                .skip_while(|l| l.id != id)
                .nth(1)
                .map(|l| l.id.clone());
            ui.horizontal(|ui| {
                if ui.button(format!("Apply to {id}")).clicked() {
                    self.apply_line();
                }
                if ui
                    .add_enabled(before.is_some(), egui::Button::new("Move up"))
                    .clicked()
                {
                    self.edit(|file| file.move_line(&id, two_before.as_deref()));
                }
                if ui
                    .add_enabled(after.is_some(), egui::Button::new("Move down"))
                    .clicked()
                {
                    self.edit(|file| file.move_line(&id, after.as_deref()));
                }
                if ui.button(format!("Remove {id}")).clicked() {
                    self.remove(&id);
                }
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let label = ui.label("New line's ID");
                ui.add(egui::TextEdit::singleline(&mut self.new_id).desired_width(140.0))
                    .labelled_by(label.id);
                let label = ui.label("Its words");
                ui.add(egui::TextEdit::singleline(&mut self.new_words).desired_width(320.0))
                    .labelled_by(label.id);
                if ui.button(format!("Add after {id}")).clicked() {
                    let (new, words) = (self.new_id.trim().to_string(), self.new_words.clone());
                    // The new line is in the section of the line it follows.
                    let section = project
                        .lines
                        .iter()
                        .find(|l| l.id == id)
                        .map_or(String::new(), |l| l.section.clone());
                    if self.edit(|file| file.insert_line(Some(&id), &new, &section, &words)) {
                        self.new_id.clear();
                        self.new_words.clear();
                        self.select(&new);
                    }
                }
            });
        }
        ui.add_space(8.0);
        ui.heading("Scenes");
        for (id, scene) in &project.scenes {
            let picked = self.scene.as_deref() == Some(id);
            let kind = format!("{:?}", scene.kind).to_lowercase();
            if ui
                .selectable_label(picked, format!("[{id}]  {kind}"))
                .clicked()
            {
                self.select_scene(id);
                self.refused.clear();
            }
        }
        if let Some(id) = self.scene.clone() {
            ui.heading(format!("Scene {id}"));
            egui::Grid::new("scene").num_columns(2).show(ui, |ui| {
                for ((_, name), value) in SCENE_KEYS.iter().zip(self.scene_form.iter_mut()) {
                    let label = ui.label(*name);
                    ui.add(egui::TextEdit::singleline(value).desired_width(560.0))
                        .labelled_by(label.id);
                    ui.end_row();
                }
            });
            if ui.button(format!("Apply to scene {id}")).clicked() {
                self.apply_scene(&id);
            }
        }
    }

    /// Saves the open scene's form: each field that differs from the file.
    /// A field left empty is left as it is: a scene's key is taken out in
    /// the source view.
    fn apply_scene(&mut self, id: &str) -> bool {
        let Some(scene) = self
            .project
            .as_ref()
            .and_then(|p| p.scenes.get(id))
            .cloned()
        else {
            return false;
        };
        let was = [
            Some(scene.url),
            scene.title,
            scene.subtitle,
            scene.quote,
            scene.by,
        ];
        // A field emptied of what the file has would take a key out.
        let cleared: Vec<&str> = SCENE_KEYS
            .iter()
            .zip(&self.scene_form)
            .zip(&was)
            .filter(|((_, now), was)| now.trim().is_empty() && was.is_some())
            .map(|(((_, name), _), _)| *name)
            .collect();
        if let Some(name) = cleared.first() {
            self.refused = vec![format!(
                "{name} is empty here and set in the file; a scene's key is taken out in the source view"
            )];
            return false;
        }
        let changes: Vec<(&str, String)> = SCENE_KEYS
            .iter()
            .zip(&self.scene_form)
            .zip(was)
            .filter(|((_, now), was)| !now.trim().is_empty() && Some(now.trim()) != was.as_deref())
            .map(|(((key, _), now), _)| (*key, now.trim().to_string()))
            .collect();
        if changes.is_empty() {
            self.refused.clear();
            return true;
        }
        self.edit(|file| {
            changes
                .iter()
                .try_for_each(|(key, value)| file.set_scene_value(id, key, value.as_str()))
        })
    }
}
