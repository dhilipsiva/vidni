//! The `vidni` command line. Its commands arrive with the milestones of
//! `TODO.md`; each calls the same core functions as the desktop app.

use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use vidni_core::takes::Status;
use vidni_core::validate;

#[derive(Parser)]
#[command(
    name = "vidni",
    version,
    about = "An open, deterministic studio for narrated tutorial videos."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Check a project file, and report each problem at its line and column.
    Validate {
        /// A project folder, or its vidni.toml.
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Rewrite a project file in the canonical layout, keeping every comment.
    Fmt {
        /// A project folder, or its vidni.toml.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Change nothing; exit 1 if the file isn't in the canonical layout.
        #[arg(long)]
        check: bool,
    },
    /// List each line's take: current, stale (recorded for other words) or missing.
    Takes {
        /// A project folder, or its vidni.toml.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Instead, lay the current takes out in DIR as 000.wav, 001.wav … in
        /// script order, as symbolic links, for tools that read numbered files.
        #[arg(long, value_name = "DIR")]
        link: Option<PathBuf>,
    },
    /// Print the JSON Schema of vidni.toml, for editors.
    Schema,
}

fn main() -> ExitCode {
    match Cli::parse().command {
        Command::Validate { path } => validate(&path),
        Command::Fmt { path, check } => fmt(&path, check),
        Command::Takes { path, link } => takes(&path, link.as_deref()),
        Command::Schema => {
            print!("{}", vidni_core::schema_json());
            ExitCode::SUCCESS
        }
    }
}

/// Reads a project's file, or reports why it can't.
fn read(path: &Path) -> Result<(PathBuf, String), ExitCode> {
    let file = validate::project_file(path);
    match fs::read_to_string(&file) {
        Ok(text) => Ok((file, text)),
        Err(e) => {
            eprintln!("vidni: {}: {e}", file.display());
            Err(ExitCode::from(2))
        }
    }
}

fn report(file: &Path, problems: &[validate::Diagnostic]) -> ExitCode {
    for problem in problems {
        eprintln!("{}:{problem}", file.display());
    }
    eprintln!("{}", count(problems.len(), "problem"));
    ExitCode::FAILURE
}

fn fmt(path: &Path, check: bool) -> ExitCode {
    let (file, text) = match read(path) {
        Ok(read) => read,
        Err(code) => return code,
    };
    let formatted = match vidni_core::fmt::format(&text) {
        Ok(formatted) => formatted,
        Err(problems) => return report(&file, &problems),
    };
    if formatted == text {
        println!("{}: already in the canonical layout", file.display());
        ExitCode::SUCCESS
    } else if check {
        eprintln!(
            "{}: not in the canonical layout; run `vidni fmt`",
            file.display()
        );
        ExitCode::FAILURE
    } else if let Err(e) = fs::write(&file, formatted) {
        eprintln!("vidni: {}: {e}", file.display());
        ExitCode::from(2)
    } else {
        println!("{}: formatted", file.display());
        ExitCode::SUCCESS
    }
}

fn takes(path: &Path, link: Option<&Path>) -> ExitCode {
    let (file, text) = match read(path) {
        Ok(read) => read,
        Err(code) => return code,
    };
    let dir = file
        .parent()
        .filter(|d| !d.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let project = match validate::check(&text, Some(dir)) {
        Ok(project) => project,
        Err(problems) => return report(&file, &problems),
    };
    let lines = vidni_core::takes::status(&project, dir);
    let shown = |p: &Path| p.strip_prefix(dir).unwrap_or(p).display().to_string();
    if let Some(link) = link {
        return link_takes(&lines, link);
    }
    let width = lines.iter().map(|l| l.id.len()).max().unwrap_or(0);
    let (mut current, mut stale, mut missing) = (0, 0, 0);
    for line in &lines {
        let (state, note) = match &line.status {
            Status::Current(_) => {
                current += 1;
                ("current", String::new())
            }
            Status::Stale(others) => {
                stale += 1;
                (
                    "stale",
                    format!(
                        " (recorded for other words: {})",
                        count(others.len(), "take")
                    ),
                )
            }
            Status::Missing => {
                missing += 1;
                ("missing", String::new())
            }
        };
        println!("{:width$}  {state:7}  {}{note}", line.id, shown(&line.path));
    }
    println!("{current} current, {stale} stale, {missing} missing");
    ExitCode::SUCCESS
}

fn link_takes(lines: &[vidni_core::takes::LineTake], dir: &Path) -> ExitCode {
    let not_current: Vec<&str> = lines
        .iter()
        .filter(|l| !matches!(l.status, Status::Current(_)))
        .map(|l| l.id.as_str())
        .collect();
    if !not_current.is_empty() {
        eprintln!(
            "vidni: these lines have no current take: {}",
            not_current.join(", ")
        );
        return ExitCode::FAILURE;
    }
    if dir.exists()
        && fs::read_dir(dir)
            .map(|mut d| d.next().is_some())
            .unwrap_or(true)
    {
        eprintln!("vidni: {} must be a new or empty folder", dir.display());
        return ExitCode::from(2);
    }
    let linked = fs::create_dir_all(dir).and_then(|_| {
        lines.iter().enumerate().try_for_each(|(k, line)| {
            let target = std::path::absolute(&line.path)?;
            std::os::unix::fs::symlink(target, dir.join(format!("{k:03}.wav")))
        })
    });
    match linked {
        Ok(()) => {
            println!(
                "linked {} into {}",
                count(lines.len(), "take"),
                dir.display()
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("vidni: {}: {e}", dir.display());
            ExitCode::from(2)
        }
    }
}

fn validate(path: &Path) -> ExitCode {
    let (file, text) = match read(path) {
        Ok(read) => read,
        Err(code) => return code,
    };
    let dir = file
        .parent()
        .filter(|d| !d.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    match validate::check(&text, Some(dir)) {
        Ok(p) => {
            println!(
                "{}: ok: {} in {}, {}, {}",
                file.display(),
                count(p.lines.len(), "line"),
                count(p.sections.len(), "section"),
                count(p.scenes.len(), "scene"),
                count(p.cuts.len(), "cut")
            );
            ExitCode::SUCCESS
        }
        Err(problems) => report(&file, &problems),
    }
}

/// "1 line", "2 lines".
fn count(n: usize, noun: &str) -> String {
    format!("{n} {noun}{}", if n == 1 { "" } else { "s" })
}
