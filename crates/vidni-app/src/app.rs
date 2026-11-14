//! The app's window (TODO.md, P7-VIDNI-01): the projects, one project's
//! cuts with where each stands, the job that is running with its log, and
//! the help.

use crate::actions::{self, Action};
use crate::editor::Editor;
use crate::export::Export;
use crate::jobs::{self, Job};
use crate::preview::{self, Player};
use crate::recent;
use crate::studio::Studio;
use eframe::egui;
use std::path::{Path, PathBuf};
use vidni_cli::console::Stream;
use vidni_core::release::Voice;

/// A project, as the app shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    /// Its folder.
    pub folder: PathBuf,
    /// Its title, when its file could be read.
    pub title: String,
    /// Each cut's ID and where it stands, in `vidni status`'s words.
    pub cuts: Vec<(String, String)>,
    /// What `vidni status` said instead, when the project can't be read.
    pub problems: Vec<String>,
}

impl Project {
    /// Reads the project in `folder`, as `vidni status` reads it.
    pub fn read(folder: &Path) -> Self {
        let (code, said) = jobs::now(&Action::Status(folder.to_path_buf()).args());
        let lines = |stream: Stream| -> Vec<String> {
            said.iter()
                .filter(|(s, _)| *s == stream)
                .map(|(_, line)| line.clone())
                .collect()
        };
        // A cut's line is its ID, spaces, and where it stands.
        let cuts = match code {
            0 => lines(Stream::Out)
                .iter()
                .filter_map(|line| {
                    let (id, standing) = line.split_once(' ')?;
                    Some((id.to_string(), standing.trim().to_string()))
                })
                .collect(),
            _ => Vec::new(),
        };
        let file = vidni_core::validate::project_file(folder);
        let title = std::fs::read_to_string(&file)
            .ok()
            .and_then(|text| vidni_core::validate::check(&text, file.parent()).ok())
            .map(|project| project.meta.title)
            .unwrap_or_default();
        Self {
            folder: folder.to_path_buf(),
            title,
            cuts,
            problems: match code {
                0 => Vec::new(),
                _ => lines(Stream::Err),
            },
        }
    }
}

/// What an open project shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    /// Its cuts, with where each stands.
    Cuts,
    /// Its script and scenes, in the editor.
    Script,
    /// A built cut, in the player.
    Preview,
    /// Its lines, to record.
    Record,
    /// Its cuts' platforms and upload folders.
    Export,
}

/// Which build of which cut the preview shows.
pub struct Preview {
    pub cut: String,
    pub voice: Voice,
    /// Whether the picture with captions drawn in is wanted.
    pub captions: bool,
    /// The player, once the picture is open; or why there is none.
    pub player: Option<Player>,
    why: String,
    /// What the player has open: the cut, the build, the picture, and
    /// when that file was written.
    open: Option<(String, Voice, PathBuf, Option<std::time::SystemTime>)>,
}

impl Default for Preview {
    fn default() -> Self {
        Self {
            cut: String::new(),
            voice: Voice::Draft,
            captions: true,
            player: None,
            why: String::new(),
            open: None,
        }
    }
}

/// The app.
pub struct App {
    /// The folder of settings, when there is one to keep the list in.
    settings: Option<PathBuf>,
    /// The projects opened lately, newest first.
    pub recent: Vec<PathBuf>,
    /// The project that is open.
    pub project: Option<Project>,
    /// The open project's file, in the editor.
    pub editor: Option<Editor>,
    /// Whether the open project shows its cuts or its script.
    pub tab: Tab,
    /// The preview of the open project's cuts.
    pub preview: Preview,
    /// The recording studio, once its tab has been opened.
    pub studio: Option<Studio>,
    /// The export panel, once its tab has been opened.
    pub export: Option<Export>,
    /// The job that is running, or the last one that ran.
    pub job: Option<Job>,
    /// Whether what the last job changed has been read.
    taken_in: bool,
    /// The cut whose frozen assets are listed, and the list, in `vidni
    /// frozen`'s words.
    pub frozen: Option<(String, Vec<String>)>,
    /// When the lock that list was read from was written; `None` until
    /// the list is read.
    frozen_read: Option<Option<std::time::SystemTime>>,
    /// What the forms hold.
    new_name: String,
    new_parent: String,
    open_folder: String,
    /// What the last thing done came to, when it is worth a line.
    note: String,
    help: bool,
}

impl App {
    /// The app, with its list of projects kept in `settings`.
    pub fn new(settings: Option<PathBuf>) -> Self {
        let recent = settings.as_deref().map(recent::read).unwrap_or_default();
        let home = std::env::var("HOME").unwrap_or_default();
        Self {
            settings,
            recent,
            project: None,
            editor: None,
            tab: Tab::Cuts,
            preview: Preview::default(),
            studio: None,
            export: None,
            job: None,
            taken_in: true,
            frozen: None,
            frozen_read: None,
            new_name: String::new(),
            new_parent: home,
            open_folder: String::new(),
            note: String::new(),
            help: false,
        }
    }

    /// Opens the project in `folder`, and puts it first among the recent ones.
    pub fn open(&mut self, folder: &Path) {
        let project = Project::read(folder);
        if project.problems.is_empty() {
            self.recent = recent::with(&self.recent, folder);
            if let Some(settings) = &self.settings
                && let Err(e) = recent::write(settings, &self.recent)
            {
                self.note = e;
            }
        }
        self.editor = Some(Editor::open(folder));
        self.preview = Preview::default();
        self.studio = None;
        self.export = None;
        // The same project read again keeps its list open, read afresh.
        if self
            .project
            .as_ref()
            .is_none_or(|open| open.folder != folder)
        {
            self.frozen = None;
        }
        self.frozen_read = None;
        self.project = Some(project);
    }

    /// Whether a job is running: one runs at a time.
    pub fn busy(&self) -> bool {
        self.job.as_ref().is_some_and(Job::running)
    }

    fn start(&mut self, action: &Action, ctx: &egui::Context) {
        let ctx = ctx.clone();
        self.job = Some(Job::start(action.args(), move || ctx.request_repaint()));
        self.taken_in = false;
    }

    /// Takes in what the running job has said, and what it changed once
    /// it has finished.
    pub fn poll(&mut self) {
        let Some(job) = &mut self.job else { return };
        job.poll();
        if !job.running() && !self.taken_in {
            self.taken_in = true;
            // Whatever the job did, the project is read again. A picture
            // the job built again is another file, which the preview
            // sees for itself and opens where it was.
            if let Some(folder) = self.project.as_ref().map(|p| p.folder.clone()) {
                self.project = Some(Project::read(&folder));
            }
        }
    }

    /// Draws the app.
    pub fn show(&mut self, ui: &mut egui::Ui) {
        self.poll();
        // A cut plays only while its preview is in front.
        if (self.project.is_none() || self.tab != Tab::Preview)
            && let Some(player) = &mut self.preview.player
        {
            player.pause();
        }
        // A take is recorded, and waits to be kept, only while the studio
        // is in front.
        if (self.project.is_none() || self.tab != Tab::Record)
            && let Some(studio) = &mut self.studio
        {
            studio.settle();
        }
        egui::Frame::central_panel(ui.style()).show(ui, |ui| {
            // The panel is the whole window, however little is in it.
            ui.set_min_size(ui.available_size());
            ui.horizontal(|ui| {
                ui.heading("vidni");
                if ui.button("Projects").clicked() {
                    self.project = None;
                }
                let help = if self.help { "Hide help" } else { "Help" };
                if ui.button(help).clicked() {
                    self.help = !self.help;
                }
            });
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                if self.help {
                    help(ui);
                    ui.separator();
                }
                match self.project.clone() {
                    None => self.browser(ui),
                    Some(project) => {
                        ui.horizontal(|ui| {
                            ui.selectable_value(&mut self.tab, Tab::Cuts, "Cuts");
                            ui.selectable_value(&mut self.tab, Tab::Script, "Script");
                            ui.selectable_value(&mut self.tab, Tab::Preview, "Preview");
                            ui.selectable_value(&mut self.tab, Tab::Record, "Record");
                            ui.selectable_value(&mut self.tab, Tab::Export, "Export");
                        });
                        match self.tab {
                            Tab::Cuts => self.cuts(ui, &project),
                            Tab::Script => self.script(ui, &project),
                            Tab::Preview => self.show_preview(ui, &project),
                            Tab::Record => self
                                .studio
                                .get_or_insert_with(|| Studio::open(&project.folder))
                                .ui(ui),
                            Tab::Export => {
                                let busy = self.busy();
                                let asked = self
                                    .export
                                    .get_or_insert_with(|| Export::open(&project.folder))
                                    .ui(ui, busy);
                                if let Some(action) = asked {
                                    let ctx = ui.ctx().clone();
                                    self.start(&action, &ctx);
                                }
                            }
                        }
                    }
                }
                if !self.note.is_empty() {
                    ui.separator();
                    ui.label(&self.note);
                }
                self.log(ui);
            });
        });
    }

    /// The projects: start one, open one, or pick a recent one.
    fn browser(&mut self, ui: &mut egui::Ui) {
        ui.heading("New project");
        ui.horizontal(|ui| {
            let label = ui.label("Name");
            ui.text_edit_singleline(&mut self.new_name)
                .labelled_by(label.id);
            let label = ui.label("In folder");
            ui.text_edit_singleline(&mut self.new_parent)
                .labelled_by(label.id);
            if ui.button("Create").clicked() {
                self.create();
            }
        });
        ui.label("A name is lowercase words joined by hyphens, such as my-first-video.");
        ui.add_space(12.0);
        ui.heading("Open project");
        ui.horizontal(|ui| {
            let label = ui.label("Project folder");
            ui.text_edit_singleline(&mut self.open_folder)
                .labelled_by(label.id);
            if ui.button("Open").clicked() {
                let folder = PathBuf::from(self.open_folder.trim());
                self.open(&folder);
            }
        });
        ui.add_space(12.0);
        ui.heading("Recent projects");
        if self.recent.is_empty() {
            ui.label("None yet.");
        }
        for folder in self.recent.clone() {
            if ui.button(folder.display().to_string()).clicked() {
                self.open(&folder);
            }
        }
    }

    /// Starts the project the form names, and opens it.
    fn create(&mut self) {
        let action = Action::New {
            name: self.new_name.trim().to_string(),
            parent: PathBuf::from(self.new_parent.trim()),
        };
        // A new project is made at once: there is nothing to wait for.
        let (code, said) = jobs::now(&action.args());
        self.note = said
            .iter()
            .map(|(_, line)| line.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        if code == 0 {
            let folder = PathBuf::from(self.new_parent.trim()).join(self.new_name.trim());
            self.new_name.clear();
            self.open(&folder);
        }
    }

    /// One project: its cuts, where each stands, and what can be done.
    fn cuts(&mut self, ui: &mut egui::Ui, project: &Project) {
        let title = match project.title.is_empty() {
            true => "A project that can't be read",
            false => project.title.as_str(),
        };
        ui.heading(title);
        ui.label(project.folder.display().to_string());
        for problem in &project.problems {
            ui.colored_label(ui.visuals().error_fg_color, problem);
        }
        let busy = self.busy();
        let ctx = ui.ctx().clone();
        ui.horizontal(|ui| {
            if ui.button("Read again").clicked() {
                self.open(&project.folder);
            }
            if ui.add_enabled(!busy, egui::Button::new("Check")).clicked() {
                self.start(&Action::Validate(project.folder.clone()), &ctx);
            }
            let all = egui::Button::new("Draft every cut");
            if ui
                .add_enabled(!busy && !project.cuts.is_empty(), all)
                .clicked()
            {
                let action = Action::Draft {
                    project: project.folder.clone(),
                    cut: None,
                };
                self.start(&action, &ctx);
            }
        });
        // The gates, for every cut at once. Each is the command, with its
        // own preconditions: what it refuses, it says in the job's log.
        ui.horizontal(|ui| {
            let folder = &project.folder;
            let gates = [
                (
                    "Ratify every cut",
                    Action::Ratify {
                        project: folder.clone(),
                        cut: None,
                    },
                ),
                (
                    "Final build of every cut",
                    Action::Final {
                        project: folder.clone(),
                        cut: None,
                    },
                ),
                (
                    "Approve every cut",
                    Action::Approve {
                        project: folder.clone(),
                        cut: None,
                    },
                ),
            ];
            for (name, action) in gates {
                let button = egui::Button::new(name);
                if ui
                    .add_enabled(!busy && !project.cuts.is_empty(), button)
                    .clicked()
                {
                    self.start(&action, &ctx);
                }
            }
        });
        ui.add_space(8.0);
        egui::Grid::new("cuts")
            .num_columns(8)
            .striped(true)
            .show(ui, |ui| {
                ui.strong("Cut");
                ui.strong("Where it stands");
                ui.end_row();
                for (id, standing) in &project.cuts {
                    ui.label(id);
                    ui.label(standing);
                    let draft = egui::Button::new(format!("Draft {id}"));
                    if ui.add_enabled(!busy, draft).clicked() {
                        let action = Action::Draft {
                            project: project.folder.clone(),
                            cut: Some(id.clone()),
                        };
                        self.start(&action, &ctx);
                    }
                    let (folder, cut) = (project.folder.clone(), id.clone());
                    let gates = [
                        (
                            format!("Ratify {id}"),
                            Action::Ratify {
                                project: folder.clone(),
                                cut: Some(cut.clone()),
                            },
                        ),
                        (
                            format!("Unratify {id}"),
                            Action::Unratify {
                                project: folder.clone(),
                                cut: cut.clone(),
                            },
                        ),
                        (
                            format!("Final {id}"),
                            Action::Final {
                                project: folder.clone(),
                                cut: Some(cut.clone()),
                            },
                        ),
                        (
                            format!("Approve {id}"),
                            Action::Approve {
                                project: folder.clone(),
                                cut: Some(cut.clone()),
                            },
                        ),
                    ];
                    for (name, action) in gates {
                        if ui.add_enabled(!busy, egui::Button::new(name)).clicked() {
                            self.start(&action, &ctx);
                        }
                    }
                    // What the cut froze, shown under the table while asked for.
                    let shown = self.frozen.as_ref().is_some_and(|(of, _)| of == id);
                    let name = match shown {
                        true => format!("Hide what {id} froze"),
                        false => format!("Frozen {id}"),
                    };
                    if ui.button(name).clicked() {
                        self.frozen_read = None;
                        self.frozen = match shown {
                            true => None,
                            false => Some((id.clone(), Vec::new())),
                        };
                    }
                    ui.end_row();
                }
            });
        // Read again whenever the lock has been written since, so that it
        // is of the lock as it is now: a cut ratified again, here or on
        // the command line, shows its new hashes.
        if let Some((cut, lines)) = &mut self.frozen {
            let written = std::fs::metadata(project.folder.join(vidni_core::ratify::LOCK))
                .and_then(|file| file.modified())
                .ok();
            if !busy && self.frozen_read != Some(written) {
                self.frozen_read = Some(written);
                let action = Action::Frozen {
                    project: project.folder.clone(),
                    cut: cut.clone(),
                };
                *lines = jobs::now(&action.args())
                    .1
                    .into_iter()
                    .map(|(_, line)| line)
                    .collect();
            }
            ui.add_space(8.0);
            ui.strong(format!("What {cut} froze"));
            for line in lines.iter() {
                ui.monospace(line);
            }
        }
    }

    /// The project's file, in the editor. A cut's standing follows from
    /// the file, so it is read again whenever the file is saved.
    fn script(&mut self, ui: &mut egui::Ui, project: &Project) {
        let Some(editor) = &mut self.editor else {
            return;
        };
        let before = editor.text.clone();
        editor.ui(ui);
        if editor.text != before {
            self.project = Some(Project::read(&project.folder));
        }
    }

    /// A built cut in the player: which cut, which build of it, and with
    /// or without its captions where the build made both pictures.
    fn show_preview(&mut self, ui: &mut egui::Ui, project: &Project) {
        let folder = &project.folder;
        let built = |cut: &str, voice: Voice| !preview::pictures(folder, voice, cut).is_empty();
        let preview = &mut self.preview;
        // The first cut that has a build, until one is picked.
        if !project.cuts.iter().any(|(id, _)| *id == preview.cut) {
            preview.cut = project
                .cuts
                .iter()
                .map(|(id, _)| id.clone())
                .find(|id| built(id, Voice::Draft) || built(id, Voice::Takes))
                .unwrap_or_default();
        }
        ui.horizontal(|ui| {
            ui.label("Cut");
            for (id, _) in &project.cuts {
                ui.selectable_value(&mut preview.cut, id.clone(), id);
            }
        });
        let (draft, last) = (
            built(&preview.cut, Voice::Draft),
            built(&preview.cut, Voice::Takes),
        );
        if !draft && !last {
            ui.label("This cut isn't built yet. Draft it from the Cuts tab, and it plays here.");
            preview.player = None;
            preview.open = None;
            return;
        }
        if !built(&preview.cut, preview.voice) {
            preview.voice = if draft { Voice::Draft } else { Voice::Takes };
        }
        let pictures = preview::pictures(folder, preview.voice, &preview.cut);
        let both =
            pictures.iter().any(|(_, drawn)| *drawn) && pictures.iter().any(|(_, drawn)| !*drawn);
        ui.horizontal(|ui| {
            ui.label("Build");
            ui.add_enabled_ui(draft, |ui| {
                ui.selectable_value(&mut preview.voice, Voice::Draft, "Draft");
            });
            ui.add_enabled_ui(last, |ui| {
                ui.selectable_value(&mut preview.voice, Voice::Takes, "Final");
            });
            // A build with one picture has its captions in it, or it hasn't.
            let captions = ui.add_enabled(both, egui::Checkbox::new(&mut preview.captions, "Captions"));
            if !both {
                captions.on_disabled_hover_text(
                    "This build made one picture. A final build of a cut with captions = \"both\" makes one with and one without.",
                );
            }
        });
        let picture = pictures
            .iter()
            .find(|(_, drawn)| *drawn == preview.captions)
            .or(pictures.first())
            .map(|(path, _)| path.clone());
        let Some(picture) = picture else { return };
        // The file's own time is part of what is open: a cut built
        // again, here or on the command line, is opened afresh.
        let written = std::fs::metadata(&picture).and_then(|m| m.modified()).ok();
        let wanted = (preview.cut.clone(), preview.voice, picture.clone(), written);
        if preview.open.as_ref() != Some(&wanted) {
            // Another picture of the same cut opens where this one was,
            // with the contact sheet up if it was.
            let same_cut = preview.open.as_ref().is_some_and(|open| open.0 == wanted.0);
            let at = preview
                .player
                .as_ref()
                .filter(|_| same_cut)
                .map(|player| (player.position, player.sheet));
            let timeline = folder
                .join(preview.voice.dir())
                .join(format!("{}.timeline.json", preview.cut));
            let ids = self
                .editor
                .as_ref()
                .and_then(|editor| editor.project.as_ref())
                .and_then(|p| {
                    p.cuts
                        .iter()
                        .find(|c| c.id == preview.cut)
                        .map(|c| c.line_ids(p))
                })
                .unwrap_or_default();
            let lines = vidni_core::timeline::Timeline::read(&timeline)
                .map(|timeline| preview::lines(&timeline, &ids))
                .unwrap_or_default();
            let ctx = ui.ctx().clone();
            match Player::open(&picture, lines, move || ctx.request_repaint()) {
                Ok(mut player) => {
                    if let Some((at, sheet)) = at {
                        player.seek(at);
                        player.sheet = sheet;
                    }
                    preview.player = Some(player);
                    preview.why.clear();
                }
                Err(why) => {
                    preview.player = None;
                    preview.why = why;
                }
            }
            preview.open = Some(wanted);
        }
        if !preview.why.is_empty() {
            ui.colored_label(ui.visuals().error_fg_color, &preview.why);
        }
        if let Some(player) = &mut preview.player {
            player.ui(ui);
        }
    }

    /// The job: its command, how far it is, and what it has said.
    fn log(&mut self, ui: &mut egui::Ui) {
        let Some(job) = &self.job else { return };
        ui.separator();
        ui.heading("Job");
        ui.monospace(format!("$ {}", job.line()));
        match (job.code, job.progress) {
            (Some(0), _) => {
                ui.label("Finished.");
            }
            (Some(code), _) => {
                ui.colored_label(
                    ui.visuals().error_fg_color,
                    format!("Stopped, with exit code {code}."),
                );
            }
            (None, Some((done, all))) => {
                let part = done as f32 / all.max(1) as f32;
                ui.add(egui::ProgressBar::new(part).text(format!("{done} of {all} frames")));
            }
            (None, None) => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Running…");
                });
            }
        }
        for (stream, line) in &job.lines {
            match stream {
                Stream::Out => ui.monospace(line),
                Stream::Err => ui.weak(egui::RichText::new(line).monospace()),
            };
        }
    }
}

/// The help: each thing the app does, and the command that does the same.
fn help(ui: &mut egui::Ui) {
    ui.heading("Help");
    ui.label(
        "Everything the app does is a vidni command, run for you. Each is listed here with the \
         command line that does the same, and a job's log starts with the command it ran.",
    );
    egui::Grid::new("help")
        .num_columns(3)
        .striped(true)
        .show(ui, |ui| {
            ui.strong("In the app");
            ui.strong("On the command line");
            ui.strong("What it does");
            ui.end_row();
            for usage in actions::ALL {
                ui.label(usage.name);
                ui.monospace(usage.command);
                ui.label(usage.does);
                ui.end_row();
            }
        });
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }
}
