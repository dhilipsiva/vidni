//! The `vidni` command line. Its commands arrive with the milestones of
//! `TODO.md`; each calls the same core functions as the desktop app.

use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use vidni_core::ratify::{self, Status as RatifyStatus};
use vidni_core::takes::Status;
use vidni_core::validate;
use vidni_core::voices;

/// Prints a line to stdout. A reader that has gone away, as in
/// `vidni takes | head`, isn't an error, so a failed write is ignored
/// instead of panicking as `println!` does.
macro_rules! say {
    ($($arg:tt)*) => {{
        use std::io::Write as _;
        let _ = writeln!(std::io::stdout(), $($arg)*);
    }};
}

mod build;
mod offline;
mod record;
mod repro;

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
    /// Start a project: a folder with a vidni.toml and the page its scene
    /// shows, which `vidni draft` renders as it is.
    New {
        /// The project's name, and its folder's: lowercase words joined by
        /// hyphens, such as my-first-video.
        name: String,
        /// Take the script from an outline in Markdown: its headings become
        /// sections, and its paragraphs and list items become lines.
        #[arg(long, value_name = "OUTLINE")]
        from: Option<PathBuf>,
    },
    /// Print the recording sheet: each line to read, with whether it has a
    /// take and how long the draft voice took over it.
    Plan {
        /// A project folder, or its vidni.toml.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// The cut whose lines to list; repeat for more. Default: every cut.
        #[arg(long = "cut", value_name = "ID")]
        cuts: Vec<String>,
    },
    /// Record your own takes in the terminal, one line at a time, with
    /// single keys: space starts a take and stops it.
    Record {
        /// A project folder, or its vidni.toml.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// The cut whose lines to record; repeat for more. Default: every cut.
        #[arg(long = "cut", value_name = "ID")]
        cuts: Vec<String>,
        /// Go through one section's lines only.
        #[arg(long, value_name = "ID")]
        section: Option<String>,
        /// Start at this line, not at the first one still to record.
        #[arg(long, value_name = "LINE")]
        from: Option<String>,
        /// Record this line again; repeat for more.
        #[arg(long = "only", value_name = "LINE")]
        only: Vec<String>,
        /// Check the microphone and the room, and record no line.
        #[arg(long)]
        test: bool,
    },
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
    /// Listen to each line's audio with whisper, and say how close what it
    /// hears is to the script. It only advises: nothing it hears changes an
    /// output.
    Hear {
        /// A project folder, or its vidni.toml.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// The line to listen to; repeat for more. Default: every line.
        #[arg(long = "line", value_name = "ID")]
        lines: Vec<String>,
        /// Listen to the draft voice's clips, even where a line has a take.
        #[arg(long)]
        draft: bool,
    },
    /// Take the captures the cuts need and the project folder doesn't hold
    /// yet: a still of each web line, and each page scene at each size.
    Capture {
        /// A project folder, or its vidni.toml.
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// List the platform profiles, with the date each was checked and its sources.
    Platforms,
    /// List the draft voices vidni can fetch, with each one's licence; or fetch one.
    Voices {
        #[command(subcommand)]
        action: Option<VoicesAction>,
    },
    /// Speak every line in the draft voice. A line's first take is one of a
    /// few seeded ones, and its seed is recorded in voice.lock; every later
    /// run repeats that take.
    Speak {
        /// A project folder, or its vidni.toml.
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Draft the cuts, to watch before recording: speak every line in the
    /// draft voice, take the captures that are missing, and render each cut
    /// into build/draft.
    Draft {
        /// A project folder, or its vidni.toml.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// The cut to draft; repeat for more. Default: every cut.
        #[arg(long = "cut", value_name = "ID")]
        cuts: Vec<String>,
        /// Render each cut again, even when nothing has changed since its
        /// last draft.
        #[arg(long)]
        again: bool,
        /// Run with no network, to show that the draft needs none.
        #[arg(long)]
        offline: bool,
    },
    /// Build the final cuts from your takes, into out. A cut is refused
    /// unless it is ratified and unchanged, each of its lines has a current
    /// take, and its build passes QA.
    Final {
        /// A project folder, or its vidni.toml.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// The cut to build; repeat for more. Default: every cut.
        #[arg(long = "cut", value_name = "ID")]
        cuts: Vec<String>,
        /// Render each cut again, even when nothing has changed since its
        /// last final build.
        #[arg(long)]
        again: bool,
        /// Run with no network, to show that the build needs none.
        #[arg(long)]
        offline: bool,
    },
    /// Approve the final cuts: record each one's build in release.json, as
    /// a release with its tag and hashes.
    Approve {
        /// A project folder, or its vidni.toml.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// The cut to approve; repeat for more. Default: every cut.
        #[arg(long = "cut", value_name = "ID")]
        cuts: Vec<String>,
    },
    /// Check that a build repeats: build the cuts twice from copies of the
    /// project, in clean environments, at two paths and on two sets of
    /// cores, and compare every hash.
    Repro {
        /// A project folder, or its vidni.toml.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// The cut to build; repeat for more. Default: every cut.
        #[arg(long = "cut", value_name = "ID")]
        cuts: Vec<String>,
        /// Build the drafts, in the draft voice, instead of the final cuts.
        #[arg(long)]
        draft: bool,
        /// Run both builds with no network.
        #[arg(long)]
        offline: bool,
        /// Keep the two builds in DIR, an empty or new folder, to look at.
        #[arg(long, value_name = "DIR")]
        keep: Option<PathBuf>,
    },
    /// Print the JSON Schema of vidni.toml, for editors.
    Schema,
}

#[derive(Subcommand)]
enum VoicesAction {
    /// Download a voice's files into the voice store, checking each one's SHA-256.
    Fetch {
        /// The voice, as `voice.draft` names it, such as piper:en_US-joe-medium.
        id: String,
    },
}

fn main() -> ExitCode {
    // On Ctrl-C or a termination signal, a running Chromium is asked to stop,
    // so that the capture unwinds and leaves nothing behind, and so are the
    // two builds of a repro, and a recording session, which has the
    // terminal to put back. With none running, or on a second signal,
    // vidni exits at once.
    let _ = ctrlc::set_handler(|| {
        // Each is asked: only one of them runs in a process.
        let (capture, builds) = (vidni_media::cdp::interrupt(), repro::interrupt());
        let recording = record::interrupt();
        if !capture && !builds && !recording {
            std::process::exit(130);
        }
    });
    let command = Cli::parse().command;
    // A command that says it runs with no network is held to it.
    if offline::inside()
        && let Err(e) = offline::verify()
    {
        eprintln!("vidni: {e}");
        return ExitCode::from(2);
    }
    match command {
        Command::New { name, from } => build::new(&name, from.as_deref()),
        Command::Plan { path, cuts } => build::plan(&path, &cuts),
        Command::Record {
            path,
            cuts,
            section,
            from,
            only,
            test,
        } => record::record(
            &path,
            &cuts,
            section.as_deref(),
            from.as_deref(),
            &only,
            test,
        ),
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
        Command::Hear { path, lines, draft } => hear(&path, &lines, draft),
        Command::Capture { path } => build::capture(&path),
        Command::Platforms => platforms(),
        Command::Voices { action: None } => voices(),
        Command::Voices {
            action: Some(VoicesAction::Fetch { id }),
        } => fetch_voice(&id),
        Command::Speak { path } => build::speak(&path),
        // With --offline, the command runs again in a namespace with no network.
        Command::Draft { offline: true, .. } | Command::Final { offline: true, .. }
            if !offline::inside() =>
        {
            offline::again()
        }
        Command::Draft {
            path, cuts, again, ..
        } => build::draft(&path, &cuts, again),
        Command::Final {
            path, cuts, again, ..
        } => build::finish(&path, &cuts, again),
        Command::Approve { path, cuts } => build::approve(&path, &cuts),
        Command::Repro {
            path,
            cuts,
            draft,
            offline,
            keep,
        } => repro::repro(&path, &cuts, draft, offline, keep.as_deref()),
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
    file: PathBuf,
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
        Ok(project) => Ok(Loaded { file, dir, project }),
        Err(problems) => Err(report(&file, &problems)),
    }
}

fn status(path: &Path) -> ExitCode {
    let Loaded { dir, project, .. } = match load(path) {
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
    let releases = match vidni_core::release::read(&dir) {
        Ok(releases) => releases,
        Err(e) => {
            eprintln!("vidni: {e}");
            return ExitCode::from(2);
        }
    };
    let states = ratify::status(&project, &dir, &lock);
    let width = states.iter().map(|(id, _)| id.len()).max().unwrap_or(0);
    for ((id, state), cut) in states.into_iter().zip(&project.cuts) {
        use vidni_core::release::Standing;
        let state = match state {
            RatifyStatus::Draft => "draft".to_string(),
            RatifyStatus::Ratified => "ratified".to_string(),
            RatifyStatus::Changed(reasons) => format!("changed: {}", reasons.join("; ")),
        };
        // And how it stands against its newest release, when it has one.
        match vidni_core::release::standing(&project, &dir, &releases, cut) {
            Standing::Unreleased => say!("{id:width$}  {state}"),
            Standing::Current(tag) => say!("{id:width$}  {state}; approved as {tag}"),
            Standing::Outdated(tag, changed) => say!(
                "{id:width$}  {state}; approved as {tag}, and changed since in {}",
                changed.join(", ")
            ),
        }
    }
    ExitCode::SUCCESS
}

fn ratify(path: &Path, cuts: &[String]) -> ExitCode {
    let Loaded { dir, project, .. } = match load(path) {
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
    let Loaded { dir, project, .. } = match load(path) {
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
            for w in validate::warnings(&text, &p, voices::store().as_deref()) {
                eprintln!(
                    "{}:{}:{}: warning: {}",
                    file.display(),
                    w.line,
                    w.column,
                    w.message
                );
            }
            ExitCode::SUCCESS
        }
        Err(problems) => report(&file, &problems),
    }
}

/// "1 line", "2 lines".
fn count(n: usize, noun: &str) -> String {
    format!("{n} {noun}{}", if n == 1 { "" } else { "s" })
}

fn voices() -> ExitCode {
    let store = voices::store();
    for p in voices::all() {
        let fetched = store
            .as_deref()
            .is_some_and(|s| voices::check_files(&p, &voices::pack_dir(s, &p.id)).is_ok());
        let use_ = if p.commercial {
            "commercial use allowed"
        } else {
            "no commercial use"
        };
        let state = if fetched { "fetched" } else { "not fetched" };
        say!("{} ({}): {}, {use_}; {state}", p.id, p.language, p.licence);
        say!("  {}", p.notes);
        say!("  licence from {} (checked {})", p.source, p.checked);
    }
    match store {
        Some(s) => say!("voice store: {}", s.display()),
        None => say!("voice store: none; set VIDNI_VOICES or HOME"),
    }
    ExitCode::SUCCESS
}

fn fetch_voice(id: &str) -> ExitCode {
    let Some(store) = voices::store() else {
        eprintln!("vidni: no voice store; set VIDNI_VOICES or HOME");
        return ExitCode::from(2);
    };
    let Some(pack) = voices::all().into_iter().find(|p| p.id == id) else {
        eprintln!("vidni: {id} isn't a voice vidni can fetch; `vidni voices` lists them");
        return ExitCode::FAILURE;
    };
    match vidni_media::tts::fetch(&pack, &store) {
        Ok(dir) => {
            say!("{id}: fetched into {}", dir.display());
            if !pack.commercial {
                say!(
                    "{id}: licensed {}, which doesn't allow commercial use",
                    pack.licence
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

fn hear(path: &Path, only: &[String], draft: bool) -> ExitCode {
    use vidni_media::hear::{self, Heard, Kind, Listener, WORTH_A_LISTEN};
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
    if let Some(unknown) = only
        .iter()
        .find(|id| !project.lines.iter().any(|l| l.id == **id))
    {
        eprintln!("vidni: {}: no line {unknown}", file.display());
        return ExitCode::FAILURE;
    }
    let cant = |why: &str| {
        eprintln!("vidni: the listening check can't run: {why}");
        ExitCode::from(2)
    };
    let language = &project.meta.language;
    if !hear::understands(language) {
        return cant(&format!(
            "it knows English only, and this script's language is {language}"
        ));
    }
    let listener = match Listener::find() {
        Ok(listener) => listener,
        Err(e) => return cant(&e),
    };
    // Without the draft voice's pack, no draft clip can be told to be for
    // a line's present wording, so only takes are heard.
    let found = voices::find(&project.voice.draft, voices::store().as_deref())
        .and_then(|pack| hear::clips(&project, dir, pack.as_ref(), draft));
    let clips = match found {
        Ok(clips) => clips,
        Err(e) => {
            eprintln!("vidni: {e}");
            return ExitCode::from(2);
        }
    };
    let width = project.lines.iter().map(|l| l.id.len()).max().unwrap_or(0);
    let (mut close, mut listen, mut nothing, mut missing, mut unheard) = (0, 0, 0, 0, 0);
    for (id, clip) in clips {
        if !only.is_empty() && !only.contains(&id) {
            continue;
        }
        let Some(clip) = clip else {
            missing += 1;
            say!("{id:width$}  no audio");
            continue;
        };
        let kind = match clip.kind {
            Kind::Take => "take",
            Kind::Draft => "draft",
        };
        match listener.hear(&clip.path) {
            Heard::Words(words) => {
                // Judged as it is shown, to two decimals.
                let score = hear::closeness(&clip.script, &words, &project.voice.heard_as);
                let score = (score * 100.0).round() / 100.0;
                say!("{id:width$}  {kind:5}  {score:.2}");
                if score < WORTH_A_LISTEN {
                    listen += 1;
                    say!("{:width$}    heard:  {words}", "");
                    say!("{:width$}    script: {}", "", clip.script);
                } else {
                    close += 1;
                }
            }
            Heard::Nothing => {
                nothing += 1;
                say!("{id:width$}  {kind:5}  nothing to hear");
            }
            Heard::Unavailable(why) => {
                unheard += 1;
                say!("{id:width$}  {kind:5}  not heard");
                eprintln!("vidni: {why}");
            }
        }
    }
    say!(
        "{}: {close} heard as scripted, {listen} worth a listen (below {WORTH_A_LISTEN}), \
         {nothing} with nothing to hear, {missing} with no audio",
        file.display()
    );
    if unheard > 0 {
        eprintln!("vidni: {} couldn't be listened to", count(unheard, "line"));
        return ExitCode::from(2);
    }
    ExitCode::SUCCESS
}
