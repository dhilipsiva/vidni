//! The `vidni` command line. Its commands arrive with the milestones of
//! `TODO.md`; each calls the same core functions as the desktop app.

use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use vidni_core::ratify::{self, Status as RatifyStatus};
use vidni_core::takes::Status;
use vidni_core::validate;

/// Prints a line to stdout. A reader that has gone away, as in
/// `vidni takes | head`, isn't an error, so a failed write is ignored
/// instead of panicking as `println!` does.
macro_rules! say {
    ($($arg:tt)*) => {{
        use std::io::Write as _;
        let _ = writeln!(std::io::stdout(), $($arg)*);
    }};
}

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
    /// Show each cut's status: draft, ratified, or changed since it was ratified.
    Status {
        /// A project folder, or its vidni.toml.
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Freeze cuts: record the hashes of their lines, scenes and captures in ratified.lock.
    Ratify {
        /// A project folder, or its vidni.toml.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// The cut to ratify; repeat for more. Default: every cut.
        #[arg(long = "cut", value_name = "ID")]
        cuts: Vec<String>,
    },
    /// Unfreeze cuts. Their captures stay, so the next draft retakes none.
    Unratify {
        /// A project folder, or its vidni.toml.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// The cut to unratify; repeat for more. Default: every cut.
        #[arg(long = "cut", value_name = "ID")]
        cuts: Vec<String>,
        /// Also delete the captures no ratified cut shares, so they are taken afresh.
        #[arg(long)]
        refresh: bool,
    },
    /// List the platform profiles, with the date each was checked and its sources.
    Platforms,
    /// Print the JSON Schema of vidni.toml, for editors.
    Schema,
}

fn main() -> ExitCode {
    match Cli::parse().command {
        Command::Validate { path } => validate(&path),
        Command::Fmt { path, check } => fmt(&path, check),
        Command::Takes { path, link } => takes(&path, link.as_deref()),
        Command::Status { path } => status(&path),
        Command::Ratify { path, cuts } => ratify(&path, &cuts),
        Command::Unratify {
            path,
            cuts,
            refresh,
        } => unratify(&path, &cuts, refresh),
        Command::Platforms => platforms(),
        Command::Schema => {
            say!("{}", vidni_core::schema_json().trim_end());
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
        say!("{}: already in the canonical layout", file.display());
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
        say!("{}: formatted", file.display());
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
        say!("{:width$}  {state:7}  {}{note}", line.id, shown(&line.path));
    }
    say!("{current} current, {stale} stale, {missing} missing");
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
            say!(
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

/// A valid project, with its file and folder.
struct Loaded {
    dir: PathBuf,
    project: vidni_core::project::Project,
}

fn load(path: &Path) -> Result<Loaded, ExitCode> {
    let (file, text) = read(path)?;
    let dir = file
        .parent()
        .filter(|d| !d.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
        .to_path_buf();
    match validate::check(&text, Some(&dir)) {
        Ok(project) => Ok(Loaded { dir, project }),
        Err(problems) => Err(report(&file, &problems)),
    }
}

fn status(path: &Path) -> ExitCode {
    let Loaded { dir, project } = match load(path) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let lock = match ratify::read_lock(&dir) {
        Ok(lock) => lock,
        Err(e) => {
            eprintln!("vidni: {e}");
            return ExitCode::from(2);
        }
    };
    let states = ratify::status(&project, &dir, &lock);
    let width = states.iter().map(|(id, _)| id.len()).max().unwrap_or(0);
    for (id, state) in states {
        match state {
            RatifyStatus::Draft => say!("{id:width$}  draft"),
            RatifyStatus::Ratified => say!("{id:width$}  ratified"),
            RatifyStatus::Changed(reasons) => {
                say!("{id:width$}  changed: {}", reasons.join("; "))
            }
        }
    }
    ExitCode::SUCCESS
}

fn ratify(path: &Path, cuts: &[String]) -> ExitCode {
    let Loaded { dir, project } = match load(path) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let done = ratify::read_lock(&dir).and_then(|mut lock| {
        let done = ratify::ratify(&project, &dir, &mut lock, cuts)?;
        ratify::write_resolved(&project, &dir)?;
        ratify::write_lock(&dir, &lock)?;
        Ok(done)
    });
    match done {
        Ok(done) => {
            for (id, content) in done {
                say!(
                    "{id}: ratified (content {}, {})",
                    &content.content[..12],
                    count(content.captures.len(), "capture")
                );
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("vidni: {e}");
            ExitCode::FAILURE
        }
    }
}

fn unratify(path: &Path, cuts: &[String], refresh: bool) -> ExitCode {
    let Loaded { dir, project } = match load(path) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let done = ratify::read_lock(&dir).and_then(|mut lock| {
        let deleted = ratify::unratify(&project, &dir, &mut lock, cuts, refresh)?;
        ratify::write_lock(&dir, &lock)?;
        Ok(deleted)
    });
    match done {
        Ok(deleted) if refresh => {
            say!(
                "unratified; {} deleted, to be taken afresh",
                count(deleted.len(), "capture")
            );
            ExitCode::SUCCESS
        }
        Ok(_) => {
            say!("unratified; the captures stay, so the next draft retakes none");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("vidni: {e}");
            ExitCode::FAILURE
        }
    }
}

fn platforms() -> ExitCode {
    use vidni_core::platforms::{self, Basis, CaptionStyle, clock};
    for p in platforms::all() {
        let default = if p.enabled_by_default {
            ", on by default"
        } else {
            ""
        };
        say!("{} ({}{default})", p.id, p.name);
        let aspect = match p.aspect {
            vidni_core::project::Aspect::Landscape => "16:9",
            vidni_core::project::Aspect::Portrait => "9:16",
        };
        let mut limits = format!("  {aspect} at {}x{}", p.resolution[0], p.resolution[1]);
        match p.max_upload {
            Some(s) => limits.push_str(&format!("; uploads up to {}", clock(s))),
            None => limits.push_str("; upload length not published"),
        }
        if let Some(s) = p.max_recommended {
            limits.push_str(&format!(", recommended up to {}", clock(s)));
        }
        if let Some(mb) = p.max_size_mb {
            limits.push_str(&format!("; files up to {mb} MB"));
        }
        say!("{limits}");
        let basis = match p.loudness.basis {
            Basis::Official => "published by the platform",
            Basis::Measured => "measured, not published",
            Basis::Default => "vidni's default; the platform publishes none",
        };
        let captions = match p.captions {
            CaptionStyle::File => "uploaded as a file",
            CaptionStyle::Burned => "burned in",
        };
        say!(
            "  loudness {} LUFS, peaks {} dBTP ({basis}); captions {captions}",
            p.loudness.integrated,
            p.loudness.true_peak
        );
        let z = &p.safe_zone;
        if z.basis == "none" {
            say!("  safe zone: none");
        } else {
            say!(
                "  safe zone: top {:.0}%, bottom {:.0}%, left {:.0}%, right {:.0}% ({})",
                z.top * 100.0,
                z.bottom * 100.0,
                z.left * 100.0,
                z.right * 100.0,
                z.basis
            );
        }
        for note in &p.notes {
            say!("  note: {note}");
        }
        say!("  verified {} against:", p.verified_on);
        for source in &p.sources {
            say!("    {source}");
        }
        say!();
    }
    say!(
        "Platforms change their limits without notice; confirm one in the app before relying on it."
    );
    ExitCode::SUCCESS
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
            say!(
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
