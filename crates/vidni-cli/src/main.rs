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
    /// Print the JSON Schema of vidni.toml, for editors.
    Schema,
}

fn main() -> ExitCode {
    match Cli::parse().command {
        Command::Validate { path } => validate(&path),
        Command::Schema => {
            print!("{}", vidni_core::schema_json());
            ExitCode::SUCCESS
        }
    }
}

fn validate(path: &Path) -> ExitCode {
    let file = validate::project_file(path);
    let text = match fs::read_to_string(&file) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("vidni: {}: {e}", file.display());
            return ExitCode::from(2);
        }
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
        Err(problems) => {
            for problem in &problems {
                eprintln!("{}:{problem}", file.display());
            }
            eprintln!("{}", count(problems.len(), "problem"));
            ExitCode::FAILURE
        }
    }
}

/// "1 line", "2 lines".
fn count(n: usize, noun: &str) -> String {
    format!("{n} {noun}{}", if n == 1 { "" } else { "s" })
}
