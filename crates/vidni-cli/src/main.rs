//! The `vidni` command line. Its commands arrive with the milestones of
//! `TODO.md`; each calls the same core functions as the desktop app.

use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
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
    /// Print the JSON Schema of vidni.toml, for editors.
    Schema,
}

fn main() -> ExitCode {
    match Cli::parse().command {
        Command::Validate { path } => validate(&path),
        Command::Fmt { path, check } => fmt(&path, check),
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
