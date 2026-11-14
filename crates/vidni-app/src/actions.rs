//! What the app can do, each as the `vidni` command it runs. The app
//! builds every command through [`Action::args`], and its help shows
//! [`Action::usage`], so the two can't drift apart.

use std::path::{Path, PathBuf};

/// One thing the app does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Start a project called `name` in the folder `parent`.
    New { name: String, parent: PathBuf },
    /// Read a project: each cut's standing.
    Status(PathBuf),
    /// Check a project's file.
    Validate(PathBuf),
    /// Draft every cut, or one.
    Draft {
        project: PathBuf,
        cut: Option<String>,
    },
    /// Write the review note on a line.
    Note {
        project: PathBuf,
        line: String,
        note: String,
    },
    /// Say what has changed since the last build and since ratification.
    Changes(PathBuf),
}

/// What the app's help says of a kind of action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Usage {
    /// What the app calls it.
    pub name: &'static str,
    /// The command line that does the same.
    pub command: &'static str,
    /// What it does.
    pub does: &'static str,
}

/// Every kind of action, in the order the help lists them.
pub const ALL: [Usage; 7] = [
    Usage {
        name: "New project",
        command: "vidni new <name> --in <folder>",
        does: "Starts a project that drafts as it is",
    },
    Usage {
        name: "Open project",
        command: "vidni status <project>",
        does: "Reads a project, and says where each cut stands",
    },
    Usage {
        name: "Check",
        command: "vidni validate <project>",
        does: "Checks the project's file, and lists each problem at its line",
    },
    Usage {
        name: "Draft every cut",
        command: "vidni draft <project>",
        does: "Speaks the lines in the draft voice, takes the captures, and renders every cut",
    },
    Usage {
        name: "Draft <cut>",
        command: "vidni draft <project> --cut <cut>",
        does: "Drafts one cut",
    },
    Usage {
        name: "Save note",
        command: "vidni notes <project> --line <line> --set <note>",
        does: "Writes what to change about a line into notes.md",
    },
    Usage {
        name: "What changed",
        command: "vidni changes <project>",
        does: "Lists the lines that changed since the last build, and the cuts that changed since they were ratified",
    },
];

impl Action {
    /// The arguments of the command that does it, as they follow `vidni`.
    pub fn args(&self) -> Vec<String> {
        let path = |p: &Path| p.display().to_string();
        match self {
            Self::New { name, parent } => {
                vec!["new".into(), name.clone(), "--in".into(), path(parent)]
            }
            Self::Status(project) => vec!["status".into(), path(project)],
            Self::Validate(project) => vec!["validate".into(), path(project)],
            Self::Draft { project, cut: None } => vec!["draft".into(), path(project)],
            Self::Draft {
                project,
                cut: Some(cut),
            } => vec!["draft".into(), path(project), "--cut".into(), cut.clone()],
            Self::Note {
                project,
                line,
                note,
            } => vec![
                "notes".into(),
                path(project),
                "--line".into(),
                line.clone(),
                "--set".into(),
                note.clone(),
            ],
            Self::Changes(project) => vec!["changes".into(), path(project)],
        }
    }

    /// What the help says of this kind of action.
    pub fn usage(&self) -> Usage {
        ALL[match self {
            Self::New { .. } => 0,
            Self::Status(_) => 1,
            Self::Validate(_) => 2,
            Self::Draft { cut: None, .. } => 3,
            Self::Draft { cut: Some(_), .. } => 4,
            Self::Note { .. } => 5,
            Self::Changes(_) => 6,
        }]
    }
}

/// `vidni` and `args` as one would type them: an argument with a space,
/// or anything else a shell reads, in single quotes.
pub fn shell(args: &[String]) -> String {
    let plain = |arg: &str| {
        !arg.is_empty()
            && arg
                .chars()
                .all(|c| c.is_alphanumeric() || "-_./:=+@%,".contains(c))
    };
    let mut line = String::from("vidni");
    for arg in args {
        line.push(' ');
        match plain(arg) {
            true => line.push_str(arg),
            false => line.push_str(&format!("'{}'", arg.replace('\'', "'\\''"))),
        }
    }
    line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_action_is_the_command_its_help_names() {
        let project = PathBuf::from("/videos/my talk");
        let actions = [
            Action::New {
                name: "my-talk".into(),
                parent: "/videos".into(),
            },
            Action::Status(project.clone()),
            Action::Validate(project.clone()),
            Action::Draft {
                project: project.clone(),
                cut: None,
            },
            Action::Draft {
                project: project.clone(),
                cut: Some("long".into()),
            },
            Action::Note {
                project: project.clone(),
                line: "intro-1".into(),
                note: "Say it slower.".into(),
            },
            Action::Changes(project),
        ];
        // Each kind of action has its own line in the help, and no line is spare.
        let listed: Vec<Usage> = actions.iter().map(Action::usage).collect();
        assert_eq!(listed, ALL);
        // The command typed is the help's command, with its blanks filled in.
        let typed: Vec<String> = actions.iter().map(|a| shell(&a.args())).collect();
        assert_eq!(
            typed,
            [
                "vidni new my-talk --in /videos",
                "vidni status '/videos/my talk'",
                "vidni validate '/videos/my talk'",
                "vidni draft '/videos/my talk'",
                "vidni draft '/videos/my talk' --cut long",
                "vidni notes '/videos/my talk' --line intro-1 --set 'Say it slower.'",
                "vidni changes '/videos/my talk'",
            ]
        );
        for (action, typed) in actions.iter().zip(&typed) {
            let usage = action.usage().command;
            let fixed: Vec<&str> = usage.split(' ').filter(|w| !w.starts_with('<')).collect();
            for word in fixed {
                assert!(
                    typed.split(' ').any(|w| w == word),
                    "{word} isn't in {typed}"
                );
            }
        }
        assert_eq!(shell(&["it's".to_string()]), "vidni 'it'\\''s'");
    }
}
