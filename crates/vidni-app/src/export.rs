//! The export panel (TODO.md, P7-VIDNI-07): the platforms each cut is
//! made for, as boxes to tick; what the cut's final build and its last
//! export found; and a button for each upload folder.
//!
//! A box changes the cut's `platforms` in the project's file, as `vidni
//! exports --add` and `--remove` do. Nothing is built here: the next
//! final build makes the folders.

use crate::actions::Action;
use crate::jobs;
use eframe::egui;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use vidni_core::release::{Record, Voice};
use vidni_core::{platforms, validate};
use vidni_media::build::Exports;

/// One cut in the panel.
#[derive(Debug, Clone, PartialEq)]
pub struct Cut {
    pub id: String,
    /// Each platform that takes the cut's shape, and whether the cut is
    /// made for it.
    pub platforms: Vec<(String, bool)>,
    /// What `vidni exports` says of the cut.
    pub said: Vec<String>,
    /// The upload folders it has, by platform.
    pub folders: Vec<(String, PathBuf)>,
}

/// The export panel of one project.
pub struct Export {
    folder: PathBuf,
    pub cuts: Vec<Cut>,
    /// What the last box ticked came to.
    pub note: String,
    /// Why the project's cuts can't be listed, when they can't.
    pub problem: Option<String>,
    /// The program that opens a folder; none opens nothing.
    opener: Option<String>,
    /// The folder that was last asked to be opened.
    pub opened: Option<PathBuf>,
    /// When the files the panel shows were written, as of its last read.
    read: Vec<Option<(SystemTime, u64)>>,
}

fn written(path: &Path) -> Option<(SystemTime, u64)> {
    let file = std::fs::metadata(path).ok()?;
    Some((file.modified().ok()?, file.len()))
}

impl Export {
    /// The panel of the project in `folder`.
    pub fn open(folder: &Path) -> Self {
        let mut export = Self {
            folder: folder.to_path_buf(),
            cuts: Vec::new(),
            note: String::new(),
            problem: None,
            opener: Some("xdg-open".to_string()),
            opened: None,
            read: Vec::new(),
        };
        export.load();
        export
    }

    /// Opens folders with `program` from here on; with none, a folder
    /// that is asked for is noted and not opened.
    pub fn set_opener(&mut self, program: Option<String>) {
        self.opener = program;
    }

    /// The project's folder: where its file is.
    fn dir(&self) -> PathBuf {
        let file = validate::project_file(&self.folder);
        file.parent().unwrap_or(&self.folder).to_path_buf()
    }

    /// When each file the panel shows was last written.
    fn stamp(&self) -> Vec<Option<(SystemTime, u64)>> {
        let dir = self.dir();
        let mut stamp = vec![written(&validate::project_file(&self.folder))];
        for cut in &self.cuts {
            stamp.push(written(&Record::path(&dir, Voice::Takes, &cut.id)));
            stamp.push(written(&Exports::path(&dir, &cut.id)));
            for (platform, _) in &cut.platforms {
                let folder = vidni_cli::exports::folder(platform, &cut.id);
                stamp.push(written(&dir.join(folder)));
            }
        }
        stamp
    }

    /// Reads the cuts, their platforms and what `vidni exports` says.
    fn load(&mut self) {
        let file = validate::project_file(&self.folder);
        let dir = self.dir();
        let project = std::fs::read_to_string(&file)
            .map_err(|e| format!("{}: {e}", file.display()))
            .and_then(|text| {
                validate::check(&text, Some(&dir)).map_err(|_| {
                    format!(
                        "{} has problems, which the Script tab lists. Its cuts are listed once \
                         it is valid.",
                        file.display()
                    )
                })
            });
        let project = match project {
            Ok(project) => project,
            Err(why) => {
                self.problem = Some(why);
                self.cuts.clear();
                self.read = self.stamp();
                return;
            }
        };
        self.problem = None;
        self.cuts = project
            .cuts
            .iter()
            .map(|cut| {
                let wanted = cut.platforms();
                let platforms: Vec<(String, bool)> = platforms::all()
                    .into_iter()
                    .filter(|profile| profile.aspect == cut.aspect)
                    .map(|profile| {
                        let on = wanted.contains(&profile.id);
                        (profile.id, on)
                    })
                    .collect();
                let action = Action::Exports {
                    project: self.folder.clone(),
                    cut: cut.id.clone(),
                };
                let said = jobs::now(&action.args())
                    .1
                    .into_iter()
                    .map(|(_, line)| line)
                    .collect();
                let folders = platforms
                    .iter()
                    .map(|(platform, _)| {
                        let folder = vidni_cli::exports::folder(platform, &cut.id);
                        (platform.clone(), dir.join(folder))
                    })
                    .filter(|(_, folder)| folder.is_dir())
                    .collect();
                Cut {
                    id: cut.id.clone(),
                    platforms,
                    said,
                    folders,
                }
            })
            .collect();
        self.read = self.stamp();
    }

    /// Takes in what has changed on disk: a final build, a hand edit.
    pub fn refresh(&mut self) {
        if self.stamp() != self.read {
            self.load();
        }
    }

    /// Makes cut `cut` for `platform` as well (`on`), or no longer.
    pub fn target(&mut self, cut: &str, platform: &str, on: bool) {
        let action = Action::Target {
            project: self.folder.clone(),
            cut: cut.to_string(),
            platform: platform.to_string(),
            on,
        };
        let (_, said) = jobs::now(&action.args());
        self.note = said
            .into_iter()
            .map(|(_, line)| line)
            .collect::<Vec<_>>()
            .join("\n");
        self.load();
    }

    /// Opens `folder` in the system's file manager.
    pub fn open_folder(&mut self, folder: &Path) {
        self.opened = Some(folder.to_path_buf());
        let Some(program) = &self.opener else {
            return;
        };
        if let Err(e) = std::process::Command::new(program).arg(folder).spawn() {
            self.note = format!(
                "{program} couldn't open {}: {e}. The folder is there to open by hand.",
                folder.display()
            );
        }
    }

    /// Draws the panel. Returns the build that was asked for, for the
    /// app to run as a job; `busy` is whether one is running.
    pub fn ui(&mut self, ui: &mut egui::Ui, busy: bool) -> Option<Action> {
        self.refresh();
        ui.add_space(8.0);
        ui.heading("Exports");
        if let Some(problem) = &self.problem {
            ui.colored_label(ui.visuals().error_fg_color, problem);
            return None;
        }
        ui.label(
            "Tick the platforms a cut is made for. Its next final build makes a folder to \
             upload from for each, and removes the folder of one that is unticked.",
        );
        if !self.note.is_empty() {
            for line in self.note.clone().lines() {
                ui.label(line);
            }
        }
        let mut asked = None;
        let (mut ticked, mut open) = (None, None);
        for cut in &self.cuts {
            ui.separator();
            ui.strong(&cut.id);
            ui.horizontal_wrapped(|ui| {
                for (platform, on) in &cut.platforms {
                    let mut now = *on;
                    let name = format!("{}: {platform}", cut.id);
                    if ui.checkbox(&mut now, name).changed() {
                        ticked = Some((cut.id.clone(), platform.clone(), now));
                    }
                }
            });
            // The cut's own line is its heading here.
            for line in cut.said.iter().skip(1) {
                ui.monospace(line);
            }
            ui.horizontal_wrapped(|ui| {
                let build = egui::Button::new(format!("Final {}", cut.id));
                if ui.add_enabled(!busy, build).clicked() {
                    asked = Some(Action::Final {
                        project: self.folder.clone(),
                        cut: Some(cut.id.clone()),
                    });
                }
                for (platform, folder) in &cut.folders {
                    let name = format!("Open {}", vidni_cli::exports::folder(platform, &cut.id));
                    if ui.button(name).clicked() {
                        open = Some(folder.clone());
                    }
                }
            });
        }
        if let Some((cut, platform, on)) = ticked {
            self.target(&cut, &platform, on);
        }
        if let Some(folder) = open {
            self.open_folder(&folder);
        }
        asked
    }
}
