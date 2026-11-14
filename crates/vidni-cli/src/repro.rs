//! `vidni repro`: the same cuts built twice, each from a copy of the
//! project at its own path, in a clean environment, on its own set of
//! cores; then every hash of the two builds is compared (TODO.md,
//! P3-VIDNI-06). A build that depends on where it runs, on the
//! environment or on how many cores it has shows up as a difference.
//! Linux only: it needs `taskset`, from util-linux.

use crate::build::{cant, chosen, setup, stopped};
use crate::{Loaded, load, offline};
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs::{self, File};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitCode, Stdio};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use vidni_core::project::{Cut, Project};
use vidni_core::release::{self, Record, Voice};
use vidni_core::{capture, fonts, ratify, voices};
use vidni_media::build::Stop;

/// The time a clean build is told it is, when the caller's environment
/// doesn't say: 1980-01-01, as many build tools take it.
const EPOCH: &str = "315532800";

/// What a running repro has made that an interrupt must undo.
#[derive(Default)]
struct Scratch {
    /// The temporary folder that holds the copies; none with `--keep`.
    remove: Option<PathBuf>,
    /// Each running build's process group.
    groups: Vec<u32>,
}

/// Set while a repro runs.
static SCRATCH: Mutex<Option<Scratch>> = Mutex::new(None);
/// Whether the running repro was asked to stop.
static INTERRUPTED: AtomicBool = AtomicBool::new(false);

/// Ends a build: it leads a process group of its own, so its ffmpeg ends
/// with it and writes nothing more.
fn end(group: u32) {
    let _ = Command::new("kill")
        .args(["-KILL", "--", &format!("-{group}")])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

/// Asks a running repro to stop: it ends its builds, removes its copies
/// and exits with 130. Returns false when none is running, and the caller,
/// a signal handler, should then exit at once. Asked a second time, while
/// the repro is still stopping, it ends the builds and removes the copies
/// itself, and returns false too.
pub fn interrupt() -> bool {
    let Ok(scratch) = SCRATCH.lock() else {
        return false;
    };
    let Some(scratch) = scratch.as_ref() else {
        return false;
    };
    if !INTERRUPTED.swap(true, Ordering::SeqCst) {
        return true;
    }
    for group in &scratch.groups {
        end(*group);
    }
    if let Some(root) = &scratch.remove {
        let _ = fs::remove_dir_all(root);
    }
    false
}

/// Marks a repro as running until it is dropped.
struct Running;

impl Running {
    fn start() -> Self {
        if let Ok(mut scratch) = SCRATCH.lock() {
            *scratch = Some(Scratch::default());
        }
        Running
    }

    fn note(&self, change: impl FnOnce(&mut Scratch)) {
        if let Ok(mut scratch) = SCRATCH.lock()
            && let Some(scratch) = scratch.as_mut()
        {
            change(scratch);
        }
    }
}

impl Drop for Running {
    fn drop(&mut self) {
        if let Ok(mut scratch) = SCRATCH.lock() {
            *scratch = None;
        }
    }
}

/// Exit code 130 with a word about it, when the repro was asked to stop.
fn interrupted(what: &str) -> Result<(), ExitCode> {
    if INTERRUPTED.load(Ordering::SeqCst) {
        eprintln!("vidni: interrupted; {what}");
        return Err(ExitCode::from(130));
    }
    Ok(())
}

/// Waits for both builds; `None` when they were interrupted, and ended.
fn wait(children: &mut [Child]) -> Result<Option<Vec<std::process::ExitStatus>>, String> {
    loop {
        if INTERRUPTED.load(Ordering::SeqCst) {
            for child in children.iter_mut() {
                end(child.id());
                let _ = child.kill();
                let _ = child.wait();
            }
            return Ok(None);
        }
        let mut done = Vec::new();
        for child in children.iter_mut() {
            match child.try_wait().map_err(|e| e.to_string())? {
                Some(status) => done.push(status),
                None => break,
            }
        }
        if done.len() == children.len() {
            return Ok(Some(done));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// The CPUs in a list such as `0-3,8,10-11`, as the kernel writes them.
fn cpus(list: &str) -> Vec<usize> {
    list.trim()
        .split(',')
        .filter_map(|part| {
            let (first, last) = part.split_once('-').unwrap_or((part, part));
            Some(first.trim().parse().ok()?..=last.trim().parse().ok()?)
        })
        .flatten()
        .collect()
}

/// Two sets of the CPUs this process may use, which differ in their CPUs
/// and in how many they are, as far as there are enough: two against three
/// of five or more, one against the rest of fewer.
fn core_sets(allowed: &[usize]) -> (Vec<usize>, Vec<usize>) {
    match allowed.len() {
        0 | 1 => (allowed.to_vec(), allowed.to_vec()),
        n @ 2..=4 => (allowed[..1].to_vec(), allowed[1..n].to_vec()),
        n => (allowed[..2].to_vec(), allowed[n - 3..].to_vec()),
    }
}

/// The CPUs this process may run on.
fn allowed_cpus() -> Result<Vec<usize>, String> {
    let status =
        fs::read_to_string("/proc/self/status").map_err(|e| format!("/proc/self/status: {e}"))?;
    let found = status
        .lines()
        .find_map(|l| l.strip_prefix("Cpus_allowed_list:"))
        .map(cpus)
        .unwrap_or_default();
    if found.is_empty() {
        return Err("/proc/self/status names no CPU this process may use".into());
    }
    Ok(found)
}

fn list(cpus: &[usize]) -> String {
    let ids: Vec<String> = cpus.iter().map(usize::to_string).collect();
    ids.join(",")
}

/// The files a build of `cuts` reads, as paths inside the project folder:
/// the project's file and locks, the files its scenes name, each line's
/// clip in `takes`, and the cuts' captures with their frames. A capture of
/// one of the project's own pages is checked against the files it was made
/// from, so they go along, with the record of them. Nothing else of the
/// folder is copied, so nothing else can matter to a build.
fn inputs(
    project: &Project,
    dir: &Path,
    file: &Path,
    cuts: &[&Cut],
    takes: &[PathBuf],
) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = vec![file.to_path_buf()];
    // The locks are read when they are there.
    for lock in [voices::LOCK, ratify::LOCK] {
        if dir.join(lock).is_file() {
            files.push(dir.join(lock));
        }
    }
    files.extend(project.scene_files().iter().map(|name| dir.join(name)));
    files.extend(takes.iter().cloned());
    for capture in cuts.iter().flat_map(|cut| capture::for_cut(project, cut)) {
        files.push(capture::path(dir, &capture.key));
        if matches!(capture.source, capture::Source::Line(_)) {
            files.push(capture::frames_path(dir, &capture.key));
        }
        if let Some(made_from) = capture::files(dir, &capture.key) {
            files.push(capture::files_path(dir, &capture.key));
            // One that wasn't there when the capture was taken still isn't.
            let there = made_from.iter().filter(|(_, sha)| sha.is_some());
            files.extend(there.map(|(name, _)| dir.join(name)));
        }
    }
    files.sort();
    files.dedup();
    files
}

/// Copies `files`, each a regular file in the project folder `from`, to
/// the same place in `to`.
fn copy_inputs(from: &Path, to: &Path, files: &[PathBuf]) -> Result<(), ExitCode> {
    for file in files {
        interrupted("nothing was built")?;
        let inside = file.strip_prefix(from).map_err(|_| {
            cant(format!(
                "{}: a build reads only files in the project folder",
                file.display()
            ))
        })?;
        // A pipe or a socket would never finish being copied.
        if !fs::metadata(file).is_ok_and(|meta| meta.is_file()) {
            return Err(cant(format!("{}: not a file to copy", file.display())));
        }
        let copy = to.join(inside);
        let made = copy
            .parent()
            .map_or(Ok(()), fs::create_dir_all)
            .and_then(|()| fs::copy(file, &copy).map(drop));
        made.map_err(|e| cant(format!("{}: {e}", copy.display())))?;
    }
    Ok(())
}

/// One of the two builds.
struct Run {
    /// The copy of the project it builds: its folder.
    project: PathBuf,
    cores: Vec<usize>,
    log: PathBuf,
}

/// What both builds are started with: the same clean environment and the
/// same command, apart from their cores and paths.
struct Recipe {
    exe: PathBuf,
    command: &'static str,
    /// The project file's name, which each copy keeps.
    file: OsString,
    cuts: Vec<String>,
    /// `unshare …`, when the builds run with no network.
    offline: Option<Vec<OsString>>,
    environment: Vec<(String, OsString)>,
}

impl Recipe {
    fn start(&self, run: &Run, root: &Path, name: &str) -> Result<Child, String> {
        let log = File::create(&run.log).map_err(|e| format!("{}: {e}", run.log.display()))?;
        let errors = log.try_clone().map_err(|e| e.to_string())?;
        // Each build has a home and a temporary folder of its own, and empty.
        let (home, tmp) = (
            root.join(format!("{name}-home")),
            root.join(format!("{name}-tmp")),
        );
        for folder in [&home, &tmp] {
            fs::create_dir_all(folder).map_err(|e| format!("{}: {e}", folder.display()))?;
        }
        let mut command = Command::new("taskset");
        command.arg("-c").arg(list(&run.cores));
        if let Some(wrapper) = &self.offline {
            command.args(wrapper);
        }
        command
            .arg(&self.exe)
            .arg(self.command)
            .arg(run.project.join(&self.file));
        for cut in &self.cuts {
            command.arg("--cut").arg(cut);
        }
        command
            .env_clear()
            .envs(
                self.environment
                    .iter()
                    .map(|(k, v)| (k.as_str(), v.as_os_str())),
            )
            .env("HOME", &home)
            .env("TMPDIR", &tmp)
            .stdin(Stdio::null())
            .stdout(log)
            .stderr(errors)
            // A group of its own, so that an interrupt can end all of it.
            .process_group(0);
        if self.offline.is_some() {
            let (mark, value) = offline::mark();
            command.env(mark, value);
        }
        command
            .spawn()
            .map_err(|e| format!("taskset: {e}; vidni repro needs taskset, from util-linux"))
    }
}

/// The last lines of a build's log, for when it failed.
fn tail(log: &Path) -> String {
    let text = fs::read_to_string(log).unwrap_or_default();
    let lines: Vec<&str> = text.lines().collect();
    lines[lines.len().saturating_sub(12)..].join("\n")
}

fn short(hash: &str) -> String {
    hash.chars().take(16).collect()
}

/// Prints one cut's hashes, which the two builds share, and returns what
/// differs between them.
fn compare(cut: &str, a: &Record, b: &Record) -> Vec<String> {
    let differences = a.differences(b);
    if !differences.is_empty() {
        return differences;
    }
    say!("{cut}: the two builds agree");
    say!("  t1 timeline         {}", short(&a.t1.timeline));
    say!(
        "  t1 captions         {} (srt), {} (vtt)",
        short(&a.t1.srt),
        short(&a.t1.vtt)
    );
    say!("  t1 pcm              {}", short(&a.t1.pcm));
    say!("  t1 pcm, normalized  {}", short(&a.t1.pcm_normalized));
    for (name, output) in &a.outputs {
        say!(
            "  t1 frames           {}  {name}, {} frames",
            short(&output.frames),
            output.frame_count
        );
        say!("  t2 packets          {}  {name}", short(&output.packets));
        say!("  t2 file             {}  {name}", short(&output.mp4));
    }
    differences
}

pub fn repro(
    path: &Path,
    cuts: &[String],
    draft: bool,
    offline: bool,
    keep: Option<&Path>,
) -> ExitCode {
    let loaded = match load(path) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let running = Running::start();
    match run(&loaded, cuts, draft, offline, keep, &running) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(code) => code,
    }
}

/// `path`, with every link in it followed.
fn real(path: &Path) -> Result<PathBuf, ExitCode> {
    fs::canonicalize(path).map_err(|e| cant(format!("{}: {e}", path.display())))
}

fn run(
    loaded: &Loaded,
    cuts: &[String],
    draft: bool,
    offline: bool,
    keep: Option<&Path>,
    running: &Running,
) -> Result<bool, ExitCode> {
    let Loaded { file, dir, project } = loaded;
    let cuts: Vec<&Cut> = chosen(project, cuts)?;
    let voice = if draft { Voice::Draft } else { Voice::Takes };

    // What the builds need is checked here, where it can be said plainly:
    // first what the creator can fix, then the tools.
    let mut seen = BTreeSet::new();
    let captures: Vec<capture::Capture> = cuts
        .iter()
        .flat_map(|cut| capture::for_cut(project, cut))
        .filter(|c| seen.insert(c.key.clone()))
        .collect();
    let unready = capture::refusals(dir, &captures);
    if !unready.is_empty() {
        return Err(stopped(Stop::Refused(vec![format!(
            "{}; repro builds from the captures it finds, so `vidni draft` comes first",
            unready.join("; ")
        )])));
    }
    let mut takes = Vec::new();
    if voice == Voice::Takes {
        let lock = ratify::read_lock(dir).map_err(cant)?;
        let refusals: Vec<String> = cuts
            .iter()
            .flat_map(|cut| release::refusals(project, dir, &lock, cut))
            .collect();
        if !refusals.is_empty() {
            return Err(stopped(Stop::Refused(refusals)));
        }
        for cut in &cuts {
            let clips = release::takes_of(project, dir, cut)
                .map_err(|refusals| stopped(Stop::Refused(refusals)))?;
            takes.extend(clips.into_iter().map(|(_, clip)| clip));
        }
    }
    setup(false)?;
    let mut environment: Vec<(String, OsString)> = vec![
        ("PATH".into(), std::env::var_os("PATH").unwrap_or_default()),
        ("TZ".into(), "UTC".into()),
        ("LC_ALL".into(), "C.UTF-8".into()),
        ("LANG".into(), "C.UTF-8".into()),
        (
            "SOURCE_DATE_EPOCH".into(),
            std::env::var_os("SOURCE_DATE_EPOCH").unwrap_or_else(|| EPOCH.into()),
        ),
        (
            "VIDNI_FONTS".into(),
            std::env::join_paths(fonts::dirs())
                .map_err(|e| cant(format!("the font folders: {e}")))?,
        ),
    ];
    // A draft speaks its lines again from voice.lock, so it needs the voice.
    if voice == Voice::Draft {
        let store = voices::store()
            .ok_or_else(|| cant("no voice store; set VIDNI_VOICES or HOME".into()))?;
        environment.push(("VIDNI_VOICES".into(), store.into()));
    }
    let wrapper = match offline {
        true => Some(offline::wrapper().map_err(cant)?),
        false => None,
    };
    let (first, second) = core_sets(&allowed_cpus().map_err(cant)?);

    // The copies go into a folder of their own, outside the project: a
    // copy inside what it copies would be part of the next one.
    let kept;
    let root = match keep {
        Some(folder) => {
            let empty = fs::read_dir(folder).map_or(true, |mut entries| entries.next().is_none());
            if !empty {
                return Err(cant(format!(
                    "{}: --keep needs an empty folder, or a new one",
                    folder.display()
                )));
            }
            let made = !folder.exists();
            fs::create_dir_all(folder).map_err(|e| cant(format!("{}: {e}", folder.display())))?;
            let root = real(folder)?;
            // A folder made here and then refused isn't left behind.
            if made && root.starts_with(real(dir)?) {
                let _ = fs::remove_dir(&root);
            }
            root
        }
        None => {
            kept = tempfile::Builder::new()
                .prefix("vidni-repro-")
                .tempdir()
                .map_err(|e| cant(format!("a temporary folder: {e}")))?;
            let root = real(kept.path())?;
            running.note(|scratch| scratch.remove = Some(root.clone()));
            root
        }
    };
    let home = real(dir)?;
    if root.starts_with(&home) {
        return Err(cant(format!(
            "{}: the copies can't be kept inside the project they are copies of; {}",
            root.display(),
            match keep {
                Some(_) => "give --keep a folder outside it",
                None => "set TMPDIR to a folder outside it",
            }
        )));
    }
    let name = file
        .file_name()
        .map(OsString::from)
        .ok_or_else(|| cant(format!("{}: no file name", file.display())))?;
    let runs = [
        Run {
            project: root.join("a/project"),
            cores: first,
            log: root.join("a.log"),
        },
        Run {
            project: root.join("b/somewhere/else/the-same-project"),
            cores: second,
            log: root.join("b.log"),
        },
    ];
    let files = inputs(project, dir, file, &cuts, &takes);
    for run in &runs {
        copy_inputs(dir, &run.project, &files)?;
    }
    interrupted("nothing was built")?;
    let recipe = Recipe {
        exe: std::env::current_exe().map_err(|e| cant(format!("vidni's own file: {e}")))?,
        command: if draft { "draft" } else { "final" },
        file: name,
        cuts: cuts.iter().map(|c| c.id.clone()).collect(),
        offline: wrapper,
        environment,
    };
    eprintln!(
        "repro: building twice{}: on cores {} and on cores {}",
        if offline { ", with no network" } else { "" },
        list(&runs[0].cores),
        list(&runs[1].cores)
    );
    if runs[0].cores == runs[1].cores {
        eprintln!("repro: this process may use one core only, so both builds share it");
    }
    let mut children = Vec::new();
    for (run, name) in runs.iter().zip(["a", "b"]) {
        match recipe.start(run, &root, name) {
            Ok(child) => {
                running.note(|scratch| scratch.groups.push(child.id()));
                children.push(child);
            }
            Err(e) => {
                // The other build, if it started, doesn't outlive this one.
                INTERRUPTED.store(true, Ordering::SeqCst);
                let _ = wait(&mut children);
                return Err(cant(e));
            }
        }
    }
    let Some(statuses) = wait(&mut children).map_err(cant)? else {
        eprintln!("vidni: interrupted; the two builds were stopped");
        return Err(ExitCode::from(130));
    };
    running.note(|scratch| scratch.groups.clear());
    let mut failed = false;
    for (status, run) in statuses.into_iter().zip(&runs) {
        if !status.success() {
            failed = true;
            eprintln!(
                "vidni: the build on cores {} failed ({status}):\n{}",
                list(&run.cores),
                tail(&run.log)
            );
        }
    }
    if failed {
        return Err(ExitCode::FAILURE);
    }

    // The resolved project is the whole project's, so it is compared once,
    // not cut by cut.
    let mut equal = true;
    let resolved =
        |run: &Run| release::file_hash(&run.project.join(ratify::RESOLVED)).map_err(cant);
    let (a, b) = (resolved(&runs[0])?, resolved(&runs[1])?);
    if a == b {
        say!("the resolved project: the two builds agree");
        say!("  t1 resolved         {}", short(&a));
    } else {
        equal = false;
        eprintln!("vidni: the builds differ in: t1 resolved");
    }
    for cut in &cuts {
        let read = |from: &Path| {
            Record::read(from, voice, &cut.id)
                .and_then(|r| {
                    r.ok_or_else(|| format!("{}: cut {} left no record", from.display(), cut.id))
                })
                .map_err(cant)
        };
        let (a, b) = (read(&runs[0].project)?, read(&runs[1].project)?);
        let mut differences = compare(&cut.id, &a, &b);
        // The project's own build was made in the creator's environment:
        // of the same inputs, it must be the same build too.
        match Record::read(dir, voice, &cut.id).map_err(cant)? {
            Some(own) if own.inputs == a.inputs => {
                let against = own.differences(&a);
                if against.is_empty() && differences.is_empty() {
                    say!("  and they agree with the build in {}", voice.dir());
                }
                differences.extend(
                    against
                        .into_iter()
                        .map(|d| format!("{d} (against the build in {})", voice.dir())),
                );
            }
            Some(_) => say!(
                "  the build in {} is of other inputs, so it isn't compared",
                voice.dir()
            ),
            None => {}
        }
        if !differences.is_empty() {
            equal = false;
            eprintln!(
                "vidni: cut {}: the builds differ in: {}",
                cut.id,
                differences.join(", ")
            );
        }
    }
    let where_ = match keep {
        Some(_) => format!("; kept in {}", root.display()),
        None => String::new(),
    };
    if equal {
        say!(
            "repro: T1 and T2 are equal across two builds, on cores {} and {}{where_}",
            list(&runs[0].cores),
            list(&runs[1].cores)
        );
    } else {
        eprintln!("vidni: repro: the two builds differ{where_}");
    }
    Ok(equal)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_lists_are_read_as_the_kernel_writes_them() {
        assert_eq!(cpus("0-3"), [0, 1, 2, 3]);
        assert_eq!(cpus("\t0-1,4,6-7\n"), [0, 1, 4, 6, 7]);
        assert_eq!(cpus("5"), [5]);
        assert_eq!(cpus(""), Vec::<usize>::new());
    }

    #[test]
    fn the_two_core_sets_differ_as_far_as_the_cores_allow() {
        let all: Vec<usize> = (0..32).collect();
        assert_eq!(core_sets(&all), (vec![0, 1], vec![29, 30, 31]));
        assert_eq!(core_sets(&all[..5]), (vec![0, 1], vec![2, 3, 4]));
        assert_eq!(core_sets(&all[..4]), (vec![0], vec![1, 2, 3]));
        assert_eq!(core_sets(&all[..2]), (vec![0], vec![1]));
        assert_eq!(core_sets(&all[..1]), (vec![0], vec![0]));
    }

    const PROJECT: &str = r#"schema = 1

[meta]
title = "A copy"
default_scene = "page"

[[cuts]]
id = "long"
aspect = "16:9"

[[cuts]]
id = "short"
aspect = "9:16"
lines = ["main-2"]

[sections.main]

[[lines]]
id = "main-1"
section = "main"
show = "A line on the page."

[[lines]]
id = "main-2"
section = "main"
show = "A line on a card."
scene = "card"

[scenes.page]
kind = "web"
url = "https://example.com"

[scenes.card]
kind = "page"
url = "cards/card-{aspect}.html?v=1"
"#;

    #[test]
    fn a_copy_holds_what_a_build_reads_and_nothing_else() {
        let from = tempfile::tempdir().unwrap();
        let to = tempfile::tempdir().unwrap();
        let dir = from.path();
        let write = |path: &str| {
            let path = dir.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, b"x").unwrap();
        };
        for path in [
            "vidni.toml",
            "voice.lock",
            "cards/card-16x9.html",
            "cards/card-9x16.html",
            "cards/card.css",
            "cards/unused.css",
            "notes.md",
            "takes/main-1/raw/0123-1.wav",
            "takes/_room/tone.wav",
            "build/voice/main-1/k-1.wav",
            "build/final/long.mp4",
            "build/captures/unused.png",
            "out/long.mp4",
            "node_modules/a/b.js",
            ".git/HEAD",
        ] {
            write(path);
        }
        fs::write(dir.join("vidni.toml"), PROJECT).unwrap();
        let project = vidni_core::validate::check(PROJECT, Some(dir)).unwrap();
        let cuts: Vec<&Cut> = project.cuts.iter().collect();
        // The takes and the captures the cuts need.
        let mut takes = Vec::new();
        for take in vidni_core::takes::status(&project, dir) {
            fs::create_dir_all(take.path.parent().unwrap()).unwrap();
            fs::write(&take.path, b"a take").unwrap();
            takes.push(take.path);
        }
        let mut wanted = vec![
            "cards/card-16x9.html".to_string(),
            "cards/card-9x16.html".to_string(),
            // What a card loaded when it was captured goes along with it.
            "cards/card.css".to_string(),
            "vidni.toml".to_string(),
            "voice.lock".to_string(),
        ];
        for c in capture::plan(&project, dir, &cuts) {
            fs::create_dir_all(dir.join(capture::DIR)).unwrap();
            fs::write(capture::path(dir, &c.key), b"a capture").unwrap();
            wanted.push(format!("build/captures/{}.png", c.key));
            if matches!(c.source, capture::Source::Line(_)) {
                fs::write(capture::frames_path(dir, &c.key), br#"{"targets":{}}"#).unwrap();
                wanted.push(format!("build/captures/{}.json", c.key));
            }
            // A card's render comes with the record of its files: the
            // card's own, and a stylesheet that goes along with it.
            if let Some(file) = &c.file {
                let made_from = [
                    (file.clone(), capture::file_sha256(dir, file)),
                    (
                        "cards/card.css".into(),
                        capture::file_sha256(dir, "cards/card.css"),
                    ),
                    ("cards/gone.png".into(), None),
                ]
                .into();
                fs::write(
                    capture::files_path(dir, &c.key),
                    capture::files_json(&made_from),
                )
                .unwrap();
                wanted.push(format!("build/captures/{}.files.json", c.key));
            }
        }
        wanted.extend(takes.iter().map(|t| release::relative(dir, t)));
        wanted.sort();

        let files = inputs(&project, dir, &dir.join("vidni.toml"), &cuts, &takes);
        let copy = to.path().join("copy");
        copy_inputs(dir, &copy, &files).unwrap();
        let mut copied = Vec::new();
        let mut folders = vec![copy.clone()];
        while let Some(folder) = folders.pop() {
            for entry in fs::read_dir(folder).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    folders.push(path);
                } else {
                    copied.push(release::relative(&copy, &path));
                }
            }
        }
        copied.sort();
        assert_eq!(copied, wanted);
        // The short cut alone needs one take and its own card, with the
        // record of the card's files and the stylesheet it names.
        let short = inputs(
            &project,
            dir,
            &dir.join("vidni.toml"),
            &cuts[1..],
            &takes[1..],
        );
        assert_eq!(short.len(), 8, "{short:?}");
    }

    #[test]
    fn a_copy_refuses_what_isnt_a_file() {
        let from = tempfile::tempdir().unwrap();
        let to = tempfile::tempdir().unwrap();
        let dir = from.path();
        fs::create_dir_all(dir.join("cards")).unwrap();
        // A folder where a file should be, and a file that isn't there.
        for missing in [dir.join("cards"), dir.join("gone.html")] {
            assert!(copy_inputs(dir, to.path(), &[missing]).is_err());
        }
        // And a file from outside the project.
        let outside = to.path().join("outside.txt");
        fs::write(&outside, b"x").unwrap();
        assert!(copy_inputs(dir, &to.path().join("copy"), &[outside]).is_err());
    }
}
