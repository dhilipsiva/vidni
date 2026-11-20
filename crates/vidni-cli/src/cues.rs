//! The same cue workflow is used by the command line and the desktop reviewer.
use crate::{build, load};
use clap::{Args, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::process::ExitCode;
use vidni_core::release::Voice;
use vidni_media::cues::{self, Review};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Audio {
    Draft,
    Takes,
}

impl Audio {
    pub fn voice(self) -> Voice {
        match self {
            Self::Draft => Voice::Draft,
            Self::Takes => Voice::Takes,
        }
    }
}

#[derive(Debug, Clone, Args)]
pub struct Options {
    #[arg(default_value = ".")]
    pub path: PathBuf,
    #[arg(long = "cut")]
    pub cuts: Vec<String>,
    #[arg(long = "only")]
    pub only: Vec<String>,
    #[arg(long, value_enum, default_value = "draft")]
    pub voice: Audio,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Render continuous, visibly marked review videos using proposed timings.
    Preview {
        #[command(flatten)]
        options: Options,
    },
    /// Suggest timings from local alignment; optionally render diagnostic clips.
    Propose {
        #[command(flatten)]
        options: Options,
        #[arg(long)]
        preview: bool,
    },
    /// Open synchronized visual review. Only a person verifies cues.
    Review {
        #[command(flatten)]
        options: Options,
    },
    /// Report verified, stale and missing cues without changing anything.
    Status {
        #[command(flatten)]
        options: Options,
    },
    /// Record an owner's explicit confirmation of exact continuous review videos.
    RecordConfirmation {
        #[command(flatten)]
        options: Options,
        /// File containing the owner's actual confirmation, not an assistant's assessment.
        #[arg(long)]
        statement_file: PathBuf,
        /// Exact reviewed video hash, one CUT=SHA256 for every selected cut.
        #[arg(long, required = true)]
        review_sha256: Vec<String>,
    },
}

pub fn run(command: Command) -> ExitCode {
    let result = match command {
        Command::Preview { options } => prepare_outputs(&options, true, false, true).map(|_| ()),
        Command::Propose { options, preview } => prepare(&options, true, preview).map(|reviews| {
            for review in reviews {
                report(&review);
            }
        }),
        Command::Status { options } => prepare(&options, false, false).map(|reviews| {
            for review in reviews {
                report(&review);
            }
        }),
        Command::Review { options } => open(&options),
        Command::RecordConfirmation {
            options,
            statement_file,
            review_sha256,
        } => record_confirmation(&options, &statement_file, &review_sha256),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            tell!("vidni: {e}");
            ExitCode::FAILURE
        }
    }
}

fn record_confirmation(
    options: &Options,
    statement_file: &std::path::Path,
    hashes: &[String],
) -> Result<(), String> {
    if !options.only.is_empty() {
        return Err(
            "continuous review confirmation covers whole cuts; --only is not supported".into(),
        );
    }
    let statement = std::fs::read_to_string(statement_file)
        .map_err(|e| format!("{}: {e}", statement_file.display()))?;
    if statement.trim().is_empty() {
        return Err("an explicit owner review statement is required".into());
    }
    let path = options.path.canonicalize().map_err(|e| e.to_string())?;
    let loaded = load(&path).map_err(|_| "could not load cue project")?;
    let cuts = build::chosen(&loaded.project, &options.cuts).map_err(|_| "unknown cue cut")?;
    let mut expected = std::collections::BTreeMap::new();
    for value in hashes {
        let (cut, sha) = value
            .split_once('=')
            .ok_or("use CUT=SHA256 for each reviewed video")?;
        if expected.insert(cut, sha).is_some() {
            return Err(format!("duplicate review hash for {cut}"));
        }
    }
    if expected.len() != cuts.len()
        || cuts
            .iter()
            .any(|cut| !expected.contains_key(cut.id.as_str()))
    {
        return Err(
            "supply one exact review hash for every selected cut, with no extra cuts".into(),
        );
    }
    let setup = build::setup(&loaded, false).map_err(|_| "could not load rendering tools")?;
    let lock = loaded.dir.join(vidni_core::cues::LOCK);
    let mut records = vidni_core::cues::Records::read(&lock)?;
    let mut counts = Vec::new();
    // Nothing is saved until every selected receipt passes. A failed Short
    // cannot partially approve the main, or vice versa.
    for cut in cuts {
        let project = loaded.project.for_cut(cut);
        let cut = project
            .cuts
            .iter()
            .find(|c| c.id == cut.id)
            .ok_or("missing translated cut")?;
        let pack =
            vidni_core::voices::find(&project.voice.draft, vidni_core::voices::store().as_deref())?;
        let plan = vidni_media::build::plan_with_review(
            &project,
            &loaded.dir,
            cut,
            options.voice.voice(),
            pack.as_ref(),
            &setup,
            true,
        )
        .map_err(|e| format!("{e:?}"))?;
        let (entries, confirmation) = cues::confirmed_full_review(
            &plan,
            &loaded.dir,
            expected[cut.id.as_str()],
            &statement,
            crate::assistant::on(),
        )?;
        counts.push((cut.id.clone(), entries.len()));
        records.entries.extend(entries);
        if !records.confirmations.contains(&confirmation) {
            records.confirmations.push(confirmation);
        }
    }
    records.write(&lock)?;
    for (cut, count) in counts {
        say!(
            "{cut}: recorded the owner's confirmation for {count} cues in the exact reviewed video"
        );
    }
    Ok(())
}

fn report(review: &Review) {
    let verified = review.cues.iter().filter(|c| c.verified).count();
    say!(
        "{}: {verified}/{} cues verified",
        review.cut,
        review.cues.len()
    );
    for cue in &review.cues {
        if let Some(display) = &cue.display {
            say!(
                "{}: visible {:.2}s{}",
                cue.key,
                (display.end - display.start) as f64 / 30.0,
                display
                    .cutoff
                    .as_ref()
                    .map_or(String::new(), |why| format!("; {why}"))
            );
        }
        if !cue.verified {
            let problem = cue.problem.as_deref().unwrap_or("unverified");
            let prefix = format!("{}: ", cue.key);
            say!(
                "{}: {}{}",
                cue.key,
                problem.strip_prefix(&prefix).unwrap_or(problem),
                if cue.interval.is_some() {
                    "; timing proposal ready"
                } else {
                    "; set timing in review"
                }
            );
        }
    }
    for preview in review.previews.values() {
        say!("preview: {}", preview.path.display());
    }
}

/// Prepare or refresh review data. With `propose=false` this is read-only.
pub fn prepare(options: &Options, propose: bool, preview: bool) -> Result<Vec<Review>, String> {
    prepare_outputs(options, propose, preview, false)
}

fn prepare_outputs(
    options: &Options,
    propose: bool,
    preview: bool,
    full: bool,
) -> Result<Vec<Review>, String> {
    if full && !options.only.is_empty() {
        return Err(
            "a continuous review video uses the entire cut; use --cut to choose one".into(),
        );
    }
    // Review receipts survive being opened from a different working directory.
    let path = options.path.canonicalize().map_err(|e| e.to_string())?;
    let loaded = load(&path).map_err(|_| "could not load cue project")?;
    let cuts = build::chosen(&loaded.project, &options.cuts).map_err(|_| "unknown cue cut")?;
    let voice = options.voice.voice();
    if propose {
        if voice == Voice::Draft {
            build::speak_lines(&loaded).map_err(|_| "could not prepare narration")?;
            // Account for this command's lock even if capture or review fails.
            crate::assistant::wrote(&options.path, &["voice.lock"]);
        }
        build::take_captures(&loaded, &cuts).map_err(|_| "could not capture measured targets")?;
    }
    let setup = build::setup(&loaded, false).map_err(|_| "could not load rendering tools")?;
    let mut reviews = Vec::new();
    for cut in cuts {
        let project = loaded.project.for_cut(cut);
        let cut = project
            .cuts
            .iter()
            .find(|c| c.id == cut.id)
            .ok_or("missing translated cut")?;
        let pack =
            vidni_core::voices::find(&project.voice.draft, vidni_core::voices::store().as_deref())?;
        let make = || {
            vidni_media::build::plan_with_review(
                &project,
                &loaded.dir,
                cut,
                voice,
                pack.as_ref(),
                &setup,
                true,
            )
            .map_err(|stop| format!("{stop:?}"))
        };
        let mut plan = make()?;
        if propose {
            let selected: Vec<_> = plan
                .emphasis
                .iter()
                .filter(|c| options.only.is_empty() || options.only.contains(&c.line_id))
                .cloned()
                .collect();
            for note in cues::propose(&loaded.dir, &selected)? {
                tell!("{note}");
            }
            plan = make()?;
        }
        if full {
            let path = cues::full_preview(&plan, &loaded.dir, &mut |frame, total| {
                if frame % 900 == 0 || frame == total {
                    say!("{} review: {frame}/{total} frames", cut.id);
                }
            })?;
            say!("review video: {}", path.display());
        }
        let review = if preview {
            cues::previews(&plan, &loaded.dir, &options.only)?
        } else {
            Review {
                schema: cues::REVIEW_SCHEMA,
                cut: cut.id.clone(),
                voice,
                cues: plan.emphasis,
                previews: Default::default(),
            }
        };
        reviews.push(review);
    }
    Ok(reviews)
}

fn open(options: &Options) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let sibling = exe.with_file_name("vidni-app");
    let app = if sibling.is_file() {
        sibling
    } else {
        PathBuf::from("vidni-app")
    };
    let mut command = std::process::Command::new(app);
    if let Some(libs) = std::env::var_os("VIDNI_WINDOW_LIBS") {
        let mut paths = std::env::split_paths(&libs).collect::<Vec<_>>();
        if let Some(existing) = std::env::var_os("LD_LIBRARY_PATH") {
            paths.extend(std::env::split_paths(&existing));
        }
        command.env(
            "LD_LIBRARY_PATH",
            std::env::join_paths(paths).map_err(|e| e.to_string())?,
        );
    }
    command
        .arg(&options.path)
        .arg("--review-cues")
        .arg("--voice")
        .arg(if matches!(options.voice, Audio::Draft) {
            "draft"
        } else {
            "takes"
        });
    for cut in &options.cuts {
        command.arg("--cut").arg(cut);
    }
    for line in &options.only {
        command.arg("--only").arg(line);
    }
    let status = command
        .status()
        .map_err(|e| format!("could not open vidni-app for cue review: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("cue review did not finish successfully".into())
    }
}

/// The app cannot acknowledge a preview of one binding as a different binding.
pub fn verify(
    options: &Options,
    expected: &cues::Cue,
    interval: vidni_core::cues::Interval,
    preview: &cues::Preview,
) -> Result<(), String> {
    let reviews = prepare(options, false, false)?;
    let all: Vec<_> = reviews
        .iter()
        .flat_map(|r| r.cues.iter().cloned())
        .collect();
    if !preview.current(&all) {
        return Err("preview context changed; rebuild and play it before verifying".into());
    }
    let current = reviews
        .iter()
        .flat_map(|r| &r.cues)
        .find(|c| c.key == expected.key)
        .ok_or("cue no longer exists")?;
    let dir = vidni_core::validate::project_file(&options.path)
        .parent()
        .ok_or("project has no folder")?
        .to_path_buf();
    cues::verify(&dir, expected, current, interval)?;
    crate::assistant::wrote(&options.path, &[vidni_core::cues::LOCK]);
    Ok(())
}

/// Save a person's changed boundary as a proposal, requiring a fresh preview.
pub fn adjust(
    options: &Options,
    expected: &cues::Cue,
    interval: vidni_core::cues::Interval,
) -> Result<(), String> {
    if crate::assistant::on() {
        return Err("an assistant proposes cues but does not edit a person's verification".into());
    }
    let reviews = prepare(options, false, false)?;
    let current = reviews
        .iter()
        .flat_map(|r| &r.cues)
        .find(|c| c.key == expected.key)
        .ok_or("cue no longer exists")?;
    if current.binding != expected.binding {
        return Err("cue inputs changed; refresh before editing".into());
    }
    interval.check(&current.binding)?;
    let file = vidni_core::validate::project_file(&options.path);
    let dir = file.parent().ok_or("project has no folder")?;
    let mut locked = vidni_core::cues::Records::read(&dir.join(vidni_core::cues::LOCK))?;
    if locked.entries.remove(&current.key).is_some() {
        locked.write(&dir.join(vidni_core::cues::LOCK))?;
        crate::assistant::wrote(&options.path, &[vidni_core::cues::LOCK]);
    }
    let mut proposals = vidni_core::cues::Records::read(&dir.join(vidni_core::cues::PROPOSALS))?;
    proposals.entries.insert(
        current.key.clone(),
        vidni_core::cues::Record {
            binding: current.binding.clone(),
            interval,
        },
    );
    proposals.write(&dir.join(vidni_core::cues::PROPOSALS))
}
