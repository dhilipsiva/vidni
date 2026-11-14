//! The app's window (TODO.md, P7-VIDNI-01): the projects, one project's
//! cuts with where each stands, the job that is running with its log, and
//! the help.

use crate::actions::{self, Action};
use crate::jobs::{self, Job};
use crate::recent;
use eframe::egui;
use std::path::{Path, PathBuf};
use vidni_cli::console::Stream;

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

/// The app.
pub struct App {
    /// The folder of settings, when there is one to keep the list in.
    settings: Option<PathBuf>,
    /// The projects opened lately, newest first.
    pub recent: Vec<PathBuf>,
    /// The project that is open.
    pub project: Option<Project>,
    /// The job that is running, or the last one that ran.
    pub job: Option<Job>,
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
            job: None,
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
        self.project = Some(project);
    }

    /// Whether a job is running: one runs at a time.
    pub fn busy(&self) -> bool {
        self.job.as_ref().is_some_and(Job::running)
    }

    fn start(&mut self, action: &Action, ctx: &egui::Context) {
        let ctx = ctx.clone();
        self.job = Some(Job::start(action.args(), move || ctx.request_repaint()));
    }

    /// Takes in what the running job has said, and what it changed once
    /// it has finished.
    pub fn poll(&mut self) {
        let Some(job) = &mut self.job else { return };
        let was_running = job.running();
        job.poll();
        if was_running && !job.running() {
            // Whatever the job did, the project is read again.
            if let Some(folder) = self.project.as_ref().map(|p| p.folder.clone()) {
                self.project = Some(Project::read(&folder));
            }
        }
    }

    /// Draws the app.
    pub fn show(&mut self, ui: &mut egui::Ui) {
        self.poll();
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
                    Some(project) => self.cuts(ui, &project),
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
        ui.add_space(8.0);
        egui::Grid::new("cuts")
            .num_columns(3)
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
                    ui.end_row();
                }
            });
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
