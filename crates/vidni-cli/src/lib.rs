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

/// Says a line: to stdout, or to whoever is listening in this thread's
/// place ([`console::listen`]). A reader that has gone away, as in
/// `vidni takes | head`, isn't an error, so a failed write is ignored
/// instead of panicking as `println!` does.
macro_rules! say {
    () => {
        $crate::console::line($crate::console::Stream::Out, "")
    };
    ($($arg:tt)*) => {
        $crate::console::line($crate::console::Stream::Out, &format!($($arg)*))
    };
}

/// Says a line of what went wrong or what is going on: to stderr, or to
/// whoever is listening in this thread's place.
macro_rules! tell {
    ($($arg:tt)*) => {
        $crate::console::line($crate::console::Stream::Err, &format!($($arg)*))
    };
}

pub mod console;

mod assistant;
mod backup;
mod build;
pub mod exports;
mod offline;
mod published;
mod record;
mod repro;
#[cfg(unix)]
mod setup;
mod takes;
mod upload;

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
        /// The folder to make the project's folder in. Default: this one.
        #[arg(long = "in", value_name = "DIR", default_value = ".")]
        parent: PathBuf,
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
        /// The cut whose lines to list; repeat for more. Default: every line.
        #[arg(long = "cut", value_name = "ID")]
        cuts: Vec<String>,
        /// Instead, list one line's raw takes, with the one its kept take
        /// is cut from.
        #[arg(long, value_name = "ID", conflicts_with_all = ["cuts", "link"])]
        line: Option<String>,
        /// With --line: make raw take N the line's kept take, trimmed as
        /// vidni trims a new take.
        #[arg(long, value_name = "N", requires = "line")]
        pick: Option<u32>,
        /// With --line: cut the kept take from FROM to TO seconds of its
        /// raw take (or of the one --pick names), as `0.35:2.8`.
        #[arg(long, value_name = "FROM:TO", requires = "line")]
        trim: Option<String>,
        /// With --line: the takes of the line's words in this language,
        /// for a cut that is dubbed in it.
        #[arg(long, value_name = "TAG", requires = "line")]
        language: Option<String>,
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
    Platforms {
        /// Instead, write a test picture and a 15-second clip of it into
        /// DIR, to measure what an app covers: upload the clip privately,
        /// and read off the squares its buttons and labels hide.
        #[arg(long, value_name = "DIR")]
        grid: Option<PathBuf>,
    },
    /// For the portable bundle: fetch the tools vidni drives, the draft
    /// voice and the listening model, each checked by its SHA-256, into
    /// your cache. It needs no root.
    Setup {
        /// A folder that holds the tools' archive already, where there is
        /// no network to fetch it over.
        #[arg(long, value_name = "DIR", conflicts_with = "mirror")]
        from: Option<PathBuf>,
        /// Fetch the tools' archive from this address in the place of
        /// vidni's release: the folder of a mirror that holds it.
        #[arg(long, value_name = "URL")]
        mirror: Option<String>,
    },
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
    /// List the platforms each cut is made for, what its final build's QA
    /// and its last export found, and each platform's upload folder; or,
    /// with --cut and --add or --remove, make a cut for one platform more
    /// or less.
    Exports {
        /// A project folder, or its vidni.toml.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// The cut to list or change; repeat to list more. Default: every cut.
        #[arg(long = "cut", value_name = "ID")]
        cuts: Vec<String>,
        /// Make the cut for this platform as well.
        #[arg(
            long,
            value_name = "PLATFORM",
            requires = "cuts",
            conflicts_with = "remove"
        )]
        add: Option<String>,
        /// Make the cut for this platform no longer.
        #[arg(long, value_name = "PLATFORM", requires = "cuts")]
        remove: Option<String>,
    },
    /// Start an assistant's session on a project: note every file as it
    /// is, so that `vidni validate`, run with VIDNI_ASSISTANT set, can say
    /// what was changed that an assistant may not change.
    Assistant {
        /// A project folder, or its vidni.toml.
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// List what each ratified cut froze: its content, lines, scenes and
    /// capture files, each with its hash.
    Frozen {
        /// A project folder, or its vidni.toml.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// The cut to list; repeat for more. Default: every cut.
        #[arg(long = "cut", value_name = "ID")]
        cuts: Vec<String>,
    },
    /// List the review notes in notes.md, a line each; or, with --line and
    /// --set, write the note on a line.
    Notes {
        /// A project folder, or its vidni.toml.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// The line the note is on.
        #[arg(long, value_name = "ID", requires = "set")]
        line: Option<String>,
        /// The note. An empty one takes the line's note away.
        #[arg(long, value_name = "TEXT", requires = "line")]
        set: Option<String>,
    },
    /// Say what has changed: each line that is new, gone or changed since
    /// the last build, and each cut that has changed since it was ratified.
    Changes {
        /// A project folder, or its vidni.toml.
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Record where a cut was published, in published.json; or, with no
    /// flags, list what is published where.
    Published {
        /// A project folder, or its vidni.toml.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// The cut that was uploaded.
        #[arg(long, value_name = "ID", requires_all = ["platform", "url"])]
        cut: Option<String>,
        /// The platform it was uploaded to.
        #[arg(long, value_name = "ID", requires_all = ["cut", "url"])]
        platform: Option<String>,
        /// The video's address there.
        #[arg(long, requires_all = ["cut", "platform"])]
        url: Option<String>,
        /// The release that was uploaded. Default: the cut's newest.
        #[arg(long, value_name = "TAG", requires = "cut")]
        release: Option<String>,
    },
    /// Upload a cut's newest release to YouTube, after asking: you type
    /// the release's name before anything is sent. It needs your own
    /// OAuth client, kept outside the project (docs/upload.md). vidni
    /// uploads nothing unless you run this.
    Upload {
        /// A project folder, or its vidni.toml.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// The cut to upload.
        #[arg(long, value_name = "ID", required_unless_present = "login")]
        cut: Option<String>,
        /// `youtube`, or `youtube-shorts` for a 9:16 cut.
        #[arg(long, value_name = "ID", default_value = "youtube")]
        platform: String,
        /// Who can see it: private, unlisted or public.
        #[arg(long, default_value = "private")]
        privacy: String,
        /// Sign in to YouTube in your browser, and keep the token.
        #[arg(long, conflicts_with = "cut")]
        login: bool,
    },
    /// Say where a line is: the cuts that say it, when in each, each
    /// cut's releases, and where each is published.
    Where {
        /// The line's ID.
        line: String,
        /// A project folder, or its vidni.toml.
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Back up the project's assets, its takes and the captures its
    /// ratified cuts froze, to two places, and list them in assets.lock.
    Backup {
        /// A project folder, or its vidni.toml.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// A place to back up to: a folder on another disk, or one mounted
        /// from another machine. Give it twice. Default: $VIDNI_BACKUP.
        #[arg(long = "to", value_name = "DIR")]
        to: Vec<PathBuf>,
    },
    /// Check the project's assets against assets.lock, and each backup too.
    Verify {
        /// A project folder, or its vidni.toml.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// A backup to check as well; repeat for more. Default: $VIDNI_BACKUP.
        #[arg(long = "to", value_name = "DIR")]
        to: Vec<PathBuf>,
    },
    /// Bring back the assets that are missing, from a backup.
    Restore {
        /// A project folder, or its vidni.toml.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// A backup to restore from; repeat for more. Default: $VIDNI_BACKUP.
        #[arg(long = "from", value_name = "DIR")]
        from: Vec<PathBuf>,
    },
    /// Delete what can be made again: draft clips, drafts, final builds,
    /// and captures no cut shows. Takes, frozen captures and delivered
    /// files are never touched.
    Gc {
        /// A project folder, or its vidni.toml.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Say what would be deleted, and delete nothing.
        #[arg(long)]
        dry_run: bool,
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

/// The `vidni` command, as the shell runs it: the process's arguments,
/// stdout and stderr, and its signals.
pub fn main() -> ExitCode {
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
    dispatch(Cli::parse().command)
}

/// A `vidni` command run inside another program, as the desktop app runs
/// them: `args` are what follows `vidni` on a command line. What the
/// command says goes to whoever [`console::listen`]s on this thread, or
/// to stdout and stderr. Returns the exit code the command line would
/// give: 0, 1 for a refusal or a failure, 2 for something missing.
pub fn run<S: AsRef<str>>(args: &[S]) -> u8 {
    let line = std::iter::once("vidni").chain(args.iter().map(AsRef::as_ref));
    let code = match Cli::try_parse_from(line) {
        Ok(cli) => dispatch(cli.command),
        Err(e) => {
            // Help is what was asked for; anything else is what went wrong.
            let stream = match e.use_stderr() {
                true => console::Stream::Err,
                false => console::Stream::Out,
            };
            for line in e.to_string().lines() {
                console::line(stream, line);
            }
            return if e.use_stderr() { 2 } else { 0 };
        }
    };
    [0u8, 1, 2, 130]
        .into_iter()
        .find(|n| ExitCode::from(*n) == code)
        .unwrap_or(1)
}

/// Why `command` isn't an assistant's to run, when it isn't.
fn a_persons(command: &Command) -> Option<&'static str> {
    Some(match command {
        Command::Ratify { .. } | Command::Unratify { .. } => {
            "ratifying a cut is a person's decision: ask them to run it"
        }
        Command::Approve { .. } => "approving a release is a person's decision: ask them to run it",
        Command::Record { .. } => "recording is the creator's own voice",
        Command::Takes { pick: Some(_), .. } | Command::Takes { trim: Some(_), .. } => {
            "choosing and trimming takes is the creator's"
        }
        Command::Published { cut: Some(_), .. } => {
            "an assistant never uploads, and doesn't record that something was uploaded"
        }
        Command::Backup { .. } | Command::Restore { .. } | Command::Gc { .. } => {
            "moving and deleting assets is a person's to do"
        }
        Command::Setup { .. } => "setting vidni up is a person's to do",
        Command::Upload { .. } => "an assistant never uploads",
        _ => return None,
    })
}

/// The files of the project that `command` writes when it succeeds, and
/// the project it was given: an assistant's session notes them again.
fn writes(command: &Command) -> Option<(PathBuf, &'static [&'static str])> {
    Some(match command {
        Command::Ratify { path, .. } | Command::Unratify { path, .. } => {
            (path.clone(), &["ratified.lock"])
        }
        Command::Approve { path, .. } => (path.clone(), &["release.json"]),
        Command::Published { path, .. } | Command::Upload { path, .. } => {
            (path.clone(), &["published.json"])
        }
        Command::Draft { path, .. } | Command::Speak { path } | Command::Hear { path, .. } => {
            (path.clone(), &["voice.lock"])
        }
        _ => return None,
    })
}

fn dispatch(command: Command) -> ExitCode {
    // An assistant runs what is an assistant's to run (docs/assistant.md).
    if assistant::on()
        && let Some(why) = a_persons(&command)
    {
        tell!("vidni: {why} ({} is set)", assistant::VAR);
        return ExitCode::FAILURE;
    }
    let written = writes(&command);
    let code = dispatch_one(command);
    if code == ExitCode::SUCCESS
        && let Some((path, names)) = written
    {
        assistant::wrote(&path, names);
    }
    code
}

fn dispatch_one(command: Command) -> ExitCode {
    // A command that says it runs with no network is held to it.
    if offline::inside()
        && let Err(e) = offline::verify()
    {
        tell!("vidni: {e}");
        return ExitCode::from(2);
    }
    match command {
        Command::New { name, from, parent } => build::new(&name, from.as_deref(), &parent),
        Command::Plan { path, cuts } => {
            by_language(&path, &cuts, true, |cuts| build::plan(&path, cuts))
        }
        Command::Record {
            path,
            cuts,
            section,
            from,
            only,
            test,
        } => by_language(&path, &cuts, false, |cuts| {
            record::record(
                &path,
                cuts,
                section.as_deref(),
                from.as_deref(),
                &only,
                test,
            )
        }),
        Command::Validate { path } => validate(&path),
        Command::Fmt { path, check } => fmt(&path, check),
        Command::Takes {
            path,
            cuts,
            line: Some(line),
            pick,
            trim,
            language,
            ..
        } => {
            let _ = cuts;
            // A line's takes in another language are of its words in it.
            LANGUAGE.with(|slot| *slot.borrow_mut() = language);
            let code = takes::line(&path, &line, pick, trim.as_deref());
            LANGUAGE.with(|slot| *slot.borrow_mut() = None);
            code
        }
        Command::Takes {
            path, cuts, link, ..
        } => takes(&path, &cuts, link.as_deref()),
        Command::Status { path } => status(&path),
        Command::Ratify { path, cuts } => ratify(&path, &cuts),
        Command::Unratify {
            path,
            cuts,
            refresh,
        } => unratify(&path, &cuts, refresh),
        Command::Hear { path, lines, draft } => hear(&path, &lines, draft),
        Command::Capture { path } => build::capture(&path),
        Command::Platforms { grid: None } => platforms(),
        Command::Platforms { grid: Some(dir) } => grid(&dir),
        #[cfg(unix)]
        Command::Setup { from, mirror } => setup::setup(from.as_deref(), mirror.as_deref()),
        #[cfg(not(unix))]
        Command::Setup { .. } => {
            tell!("vidni: `vidni setup` is for the portable bundle on Linux");
            ExitCode::FAILURE
        }
        Command::Voices { action: None } => voices(),
        Command::Voices {
            action: Some(VoicesAction::Fetch { id }),
        } => fetch_voice(&id),
        Command::Speak { path } => by_language(&path, &[], true, |_| build::speak(&path)),
        // With --offline, the command runs again in a namespace with no network.
        Command::Draft { offline: true, .. } | Command::Final { offline: true, .. }
            if !offline::inside() =>
        {
            offline::again()
        }
        Command::Draft {
            path, cuts, again, ..
        } => by_language(&path, &cuts, true, |cuts| build::draft(&path, cuts, again)),
        Command::Final {
            path, cuts, again, ..
        } => by_language(&path, &cuts, true, |cuts| build::finish(&path, cuts, again)),
        Command::Approve { path, cuts } => {
            by_language(&path, &cuts, true, |cuts| build::approve(&path, cuts))
        }
        Command::Published {
            path,
            cut: Some(cut),
            platform: Some(platform),
            url: Some(url),
            release,
        } => published::record(&path, &cut, &platform, &url, release.as_deref()),
        Command::Published { path, .. } => published::list(&path),
        Command::Where { line, path } => published::whereabouts(&path, &line),
        Command::Upload { login: true, .. } => upload::login(),
        Command::Upload {
            path,
            cut,
            platform,
            privacy,
            ..
        } => upload::upload(&path, &cut.unwrap_or_default(), &platform, &privacy),
        Command::Notes { path, line, set } => notes(&path, line.as_deref().zip(set.as_deref())),
        Command::Changes { path } => changes(&path),
        Command::Frozen { path, cuts } => frozen(&path, &cuts),
        Command::Assistant { path } => assistant::start(&path),
        Command::Exports {
            path,
            cuts,
            add,
            remove,
        } => match (add.or(remove.clone()), cuts.as_slice()) {
            (None, _) => exports::list(&path, &cuts),
            (Some(platform), [cut]) => exports::target(&path, cut, &platform, remove.is_none()),
            (Some(_), _) => {
                tell!("vidni: --add and --remove change one cut: name it with one --cut");
                ExitCode::from(2)
            }
        },
        Command::Backup { path, to } => backup::backup(&path, &to),
        Command::Verify { path, to } => backup::verify(&path, &to),
        Command::Restore { path, from } => backup::restore(&path, &from),
        Command::Gc { path, dry_run } => backup::gc(&path, dry_run),
        Command::Repro {
            path,
            cuts,
            draft,
            offline,
            keep,
        } => by_language(&path, &cuts, true, |cuts| {
            repro::repro(&path, cuts, draft, offline, keep.as_deref())
        }),
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
            tell!("vidni: {}: {e}", file.display());
            Err(ExitCode::from(2))
        }
    }
}

fn report(file: &Path, problems: &[validate::Diagnostic]) -> ExitCode {
    for problem in problems {
        tell!("{}:{problem}", file.display());
    }
    tell!("{}", count(problems.len(), "problem"));
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
        tell!(
            "{}: not in the canonical layout; run `vidni fmt`",
            file.display()
        );
        ExitCode::FAILURE
    } else if let Err(e) = fs::write(&file, formatted) {
        tell!("vidni: {}: {e}", file.display());
        ExitCode::from(2)
    } else {
        say!("{}: formatted", file.display());
        ExitCode::SUCCESS
    }
}

fn takes(path: &Path, cuts: &[String], link: Option<&Path>) -> ExitCode {
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
    let mut lines = vidni_core::takes::status(&project, dir);
    if !cuts.is_empty() {
        let ids: Vec<String> = match build::chosen(&project, cuts) {
            Ok(cuts) => cuts.iter().flat_map(|c| c.line_ids(&project)).collect(),
            Err(code) => return code,
        };
        lines.retain(|l| ids.contains(&l.id));
    }
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
    for jump in takes::jumps(&project, dir) {
        tell!("vidni: warning: {jump}");
    }
    ExitCode::SUCCESS
}

fn link_takes(lines: &[vidni_core::takes::LineTake], dir: &Path) -> ExitCode {
    let not_current: Vec<&str> = lines
        .iter()
        .filter(|l| !matches!(l.status, Status::Current(_)))
        .map(|l| l.id.as_str())
        .collect();
    if !not_current.is_empty() {
        tell!(
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
        tell!("vidni: {} must be a new or empty folder", dir.display());
        return ExitCode::from(2);
    }
    let linked = fs::create_dir_all(dir).and_then(|_| {
        lines.iter().enumerate().try_for_each(|(k, line)| {
            let target = std::path::absolute(&line.path)?;
            let link = dir.join(format!("{k:03}.wav"));
            #[cfg(unix)]
            return std::os::unix::fs::symlink(target, link);
            #[cfg(windows)]
            return std::os::windows::fs::symlink_file(target, link);
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
            tell!("vidni: {}: {e}", dir.display());
            ExitCode::from(2)
        }
    }
}

/// A valid project, with its file and folder.
/// The file that starts vidni again: the bundle's own starter where
/// vidni runs from its portable bundle, which hands the program to the
/// bundle's loader; the program itself otherwise.
pub(crate) fn own_exe() -> std::io::Result<PathBuf> {
    let set = |name: &str| std::env::var_os(name).filter(|v| !v.is_empty());
    match (set("VIDNI_BUNDLE"), set("VIDNI_EXE")) {
        (Some(_), Some(starter)) => Ok(PathBuf::from(starter)),
        _ => std::env::current_exe(),
    }
}

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
        // A command that works a language at a time gets the project as
        // that language's cuts have it.
        Ok(project) => Ok(Loaded {
            file,
            dir,
            project: match LANGUAGE.with(|language| language.borrow().clone()) {
                Some(language) => project.in_language(&language),
                None => project,
            },
        }),
        Err(problems) => Err(report(&file, &problems)),
    }
}

thread_local! {
    /// The language the command at work speaks, while it works one
    /// language at a time ([`by_language`]).
    static LANGUAGE: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

/// Runs `run` once for each language that the cuts asked for speak, with
/// that language's cuts. Inside, [`load`] gives the project in that
/// language: its cuts, its lines' words, its draft voice. A project in
/// one language runs once.
///
/// With no cut named, `all` says whether every language is meant, or the
/// script's own only: a recording session is in one language.
fn by_language(
    path: &Path,
    cuts: &[String],
    all: bool,
    run: impl Fn(&[String]) -> ExitCode,
) -> ExitCode {
    let loaded = match load(path) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let project = &loaded.project;
    let chosen = match build::chosen(project, cuts) {
        Ok(chosen) => chosen,
        Err(code) => return code,
    };
    let mut languages = project.languages();
    if cuts.is_empty() && !all {
        languages.truncate(1);
    }
    for language in languages {
        let ids: Vec<String> = chosen
            .iter()
            .filter(|cut| project.language_of(cut) == language)
            .map(|cut| cut.id.clone())
            .collect();
        if ids.is_empty() {
            continue;
        }
        LANGUAGE.with(|slot| *slot.borrow_mut() = Some(language));
        let code = run(&ids);
        LANGUAGE.with(|slot| *slot.borrow_mut() = None);
        if code != ExitCode::SUCCESS {
            return code;
        }
    }
    ExitCode::SUCCESS
}

fn status(path: &Path) -> ExitCode {
    let Loaded { dir, project, .. } = match load(path) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let lock = match ratify::read_lock(&dir) {
        Ok(lock) => lock,
        Err(e) => {
            tell!("vidni: {e}");
            return ExitCode::from(2);
        }
    };
    let releases = match vidni_core::release::read(&dir) {
        Ok(releases) => releases,
        Err(e) => {
            tell!("vidni: {e}");
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

/// Lists the notes, or writes the note `set` names: a line and its note.
fn notes(path: &Path, set: Option<(&str, &str)>) -> ExitCode {
    use vidni_core::notes;
    let Loaded { dir, project, .. } = match load(path) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let mut all = match notes::read(&dir) {
        Ok(all) => all,
        Err(e) => {
            tell!("vidni: {e}");
            return ExitCode::from(2);
        }
    };
    if let Some((line, note)) = set {
        if !project.lines.iter().any(|l| l.id == line) {
            tell!("vidni: there is no line {line:?}");
            return ExitCode::FAILURE;
        }
        all.set(line, note);
        if let Err(e) = notes::write(&dir, &all) {
            tell!("vidni: {e}");
            return ExitCode::FAILURE;
        }
        match note.trim().is_empty() {
            true => say!("{line}: its note is taken out of {}", notes::FILE),
            false => say!("{line}: noted in {}", notes::FILE),
        }
        return ExitCode::SUCCESS;
    }
    let listed = all.all();
    for (line, note) in &listed {
        // A note on a line that is gone is still someone's words.
        let gone = match project.lines.iter().any(|l| &l.id == line) {
            true => "",
            false => " (no such line now)",
        };
        say!("{line}{gone}: {}", note.replace('\n', " "));
    }
    if listed.is_empty() {
        say!("no notes; `vidni notes --line ID --set TEXT` writes one");
    }
    ExitCode::SUCCESS
}

/// Says what has changed since the last build, and since each cut was ratified.
fn changes(path: &Path) -> ExitCode {
    let Loaded { dir, project, .. } = match load(path) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let built = dir.join(ratify::RESOLVED);
    match fs::read_to_string(&built) {
        Err(_) => say!("since the last build: there is no build yet to compare with"),
        Ok(text) => match vidni_core::changes::parse(&text) {
            Err(e) => {
                tell!("vidni: {}: {e}", built.display());
                return ExitCode::from(2);
            }
            Ok(before) => {
                let found = vidni_core::changes::between(&before, &ratify::resolved(&project));
                match found.is_empty() {
                    true => say!("since the last build: nothing"),
                    false => say!("since the last build:"),
                }
                for change in found {
                    say!("  {change}");
                }
            }
        },
    }
    let lock = match ratify::read_lock(&dir) {
        Ok(lock) => lock,
        Err(e) => {
            tell!("vidni: {e}");
            return ExitCode::from(2);
        }
    };
    say!("since each cut was ratified:");
    for (id, state) in ratify::status(&project, &dir, &lock) {
        match state {
            RatifyStatus::Draft => say!("  {id}: not ratified"),
            RatifyStatus::Ratified => say!("  {id}: nothing"),
            RatifyStatus::Changed(reasons) => say!("  {id}: {}", reasons.join("; ")),
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
            tell!("vidni: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Lists what each ratified cut froze, with its hashes, as `ratified.lock`
/// holds them.
fn frozen(path: &Path, cuts: &[String]) -> ExitCode {
    let Loaded { dir, project, .. } = match load(path) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    if let Some(unknown) = cuts
        .iter()
        .find(|id| !project.cuts.iter().any(|cut| &cut.id == *id))
    {
        tell!("vidni: there is no cut {unknown:?}");
        return ExitCode::FAILURE;
    }
    let lock = match ratify::read_lock(&dir) {
        Ok(lock) => lock,
        Err(e) => {
            tell!("vidni: {e}");
            return ExitCode::from(2);
        }
    };
    let asked = |id: &String| cuts.is_empty() || cuts.contains(id);
    for cut in project.cuts.iter().filter(|cut| asked(&cut.id)) {
        let Some(locked) = lock.cuts.get(&cut.id) else {
            say!("{}: not ratified", cut.id);
            continue;
        };
        say!("{}: content {}", cut.id, locked.content);
        for (what, hashes) in [("line", &locked.lines), ("scene", &locked.scenes)] {
            for (id, hash) in hashes {
                say!("  {what} {id}  {hash}");
            }
        }
        // A capture is a file: its place in the project, and the file's hash.
        let file = |path: PathBuf| {
            path.strip_prefix(&dir)
                .unwrap_or(&path)
                .display()
                .to_string()
        };
        for (key, sha) in &locked.captures {
            say!(
                "  capture {}  sha256 {sha}",
                file(vidni_core::capture::path(&dir, key))
            );
        }
        for (key, sha) in &locked.frames {
            say!(
                "  frames {}  sha256 {sha}",
                file(vidni_core::capture::frames_path(&dir, key))
            );
        }
    }
    ExitCode::SUCCESS
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
            tell!("vidni: {e}");
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

/// Writes the 9:16 test picture and its clip into `dir`.
fn grid(dir: &Path) -> ExitCode {
    use vidni_core::theme::Theme;
    use vidni_media::build::Faces;
    let made = Faces::pinned(&Theme::default_theme(), &vidni_core::fonts::dirs())
        .and_then(|faces| vidni_media::review::grid_clip((1080, 1920), 15, &faces.mono, dir));
    match made {
        Ok([png, mp4]) => {
            say!(
                "{}: 1080x1920, in squares of 120 px, each with the place of its top left corner",
                png.display()
            );
            say!("{}: 15 s of it, encoded as a cut is", mp4.display());
            say!(
                "Upload the clip privately, take a screenshot of it playing, and read off \
                 the squares the app's buttons and labels hide."
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            tell!("vidni: {e}");
            ExitCode::FAILURE
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
            say!(
                "{}: ok: {} in {}, {}, {}",
                file.display(),
                count(p.lines.len(), "line"),
                count(p.sections.len(), "section"),
                count(p.scenes.len(), "scene"),
                count(p.cuts.len(), "cut")
            );
            for w in validate::warnings(&text, &p, voices::store().as_deref()) {
                tell!(
                    "{}:{}:{}: warning: {}",
                    file.display(),
                    w.line,
                    w.column,
                    w.message
                );
            }
            // For an assistant, the folder is held to its session's start.
            if assistant::on() {
                let broken = assistant::broken(path);
                for problem in &broken {
                    tell!("vidni: {problem}");
                }
                if !broken.is_empty() {
                    tell!(
                        "{} that the assistant protocol forbids (docs/assistant.md)",
                        count(broken.len(), "change")
                    );
                    return ExitCode::FAILURE;
                }
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
        tell!("vidni: no voice store; set VIDNI_VOICES or HOME");
        return ExitCode::from(2);
    };
    let Some(pack) = voices::all().into_iter().find(|p| p.id == id) else {
        tell!("vidni: {id} isn't a voice vidni can fetch; `vidni voices` lists them");
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
            tell!("vidni: {e}");
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
        tell!("vidni: {}: no line {unknown}", file.display());
        return ExitCode::FAILURE;
    }
    let cant = |why: &str| {
        tell!("vidni: the listening check can't run: {why}");
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
            tell!("vidni: {e}");
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
                tell!("vidni: {why}");
            }
        }
    }
    say!(
        "{}: {close} heard as scripted, {listen} worth a listen (below {WORTH_A_LISTEN}), \
         {nothing} with nothing to hear, {missing} with no audio",
        file.display()
    );
    if unheard > 0 {
        tell!("vidni: {} couldn't be listened to", count(unheard, "line"));
        return ExitCode::from(2);
    }
    ExitCode::SUCCESS
}
