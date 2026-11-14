//! `vidni speak`, `draft`, `final` and `approve`: the commands that make a
//! cut. The gates are in `vidni_media::build` and `vidni_core::release`;
//! here they are only run and reported.

use crate::{Loaded, count, load};
use std::path::Path;
use std::process::ExitCode;
use vidni_core::project::{Cut, Project};
use vidni_core::release::{self, Approval, Record};
use vidni_core::voices::{self, Pack};
use vidni_core::{capture, ratify};
use vidni_media::build::{self, Setup, Step, Stop};

/// The cuts named, or every cut when none is.
pub(crate) fn chosen<'p>(project: &'p Project, ids: &[String]) -> Result<Vec<&'p Cut>, ExitCode> {
    if ids.is_empty() {
        return Ok(project.cuts.iter().collect());
    }
    ids.iter()
        .map(|id| {
            project.cuts.iter().find(|c| &c.id == id).ok_or_else(|| {
                eprintln!("vidni: there is no cut {id:?}");
                ExitCode::FAILURE
            })
        })
        .collect()
}

/// Reports why a build stopped. A refusal is the creator's to fix, and
/// exits 1; so does a tool that failed part of the way.
pub(crate) fn stopped(stop: Stop) -> ExitCode {
    match stop {
        Stop::Refused(reasons) => {
            for reason in reasons {
                eprintln!("vidni: {reason}");
            }
        }
        Stop::Failed(e) => eprintln!("vidni: {e}"),
    }
    ExitCode::FAILURE
}

/// Reports that something a command needs is missing or unreadable, as
/// exit code 2.
pub(crate) fn cant(why: String) -> ExitCode {
    eprintln!("vidni: {why}");
    ExitCode::from(2)
}

/// What a build runs with, or why it can't start. With `again`, a cut is
/// built even when nothing has changed since its last build.
pub(crate) fn setup(loaded: &Loaded, again: bool) -> Result<Setup, ExitCode> {
    build::check_tools()
        .and_then(|()| Setup::find(&loaded.project, &loaded.dir))
        .map(|setup| Setup { again, ..setup })
        .map_err(cant)
}

/// Prints a build's stages, and its frames at each tenth, to stderr.
fn progress(cut: &str) -> impl FnMut(Step) + '_ {
    let mut tenth = 0;
    move |step| match step {
        Step::Stage(stage) => {
            tenth = 0;
            eprintln!("{cut}: {stage}");
        }
        Step::Kept => {
            eprintln!("{cut}: nothing has changed since it was built; nothing to render");
        }
        Step::Frames { file, done, of } => {
            let now = done * 10 / of.max(1);
            if now > tenth {
                tenth = now;
                eprintln!("{cut}: {file}: {done} of {of} frames");
            }
        }
    }
}

/// Speaks every line in the draft voice, and says what was spoken.
/// Returns the voice's pack.
pub(crate) fn speak_lines(loaded: &Loaded) -> Result<Pack, ExitCode> {
    use vidni_media::tts::{self, From, Piper};
    let Loaded { file, dir, project } = loaded;
    let store = voices::store();
    let id = &project.voice.draft;
    let pack = match voices::find(id, store.as_deref()) {
        Ok(Some(pack)) => pack,
        Ok(None) => {
            eprintln!(
                "vidni: the draft voice {id} isn't a voice vidni knows, and the voice store has no pack.toml for it"
            );
            return Err(ExitCode::FAILURE);
        }
        Err(e) => {
            eprintln!("vidni: {e}");
            return Err(ExitCode::from(2));
        }
    };
    let Some(store) = store else {
        eprintln!("vidni: no voice store; set VIDNI_VOICES or HOME");
        return Err(ExitCode::from(2));
    };
    // A voice that isn't fetched, or a Piper that isn't there, is something
    // missing; a take that fails part of the way is a failed build.
    let engine = Piper::new(&pack, &voices::pack_dir(&store, id)).map_err(cant)?;
    let drafted = tts::draft(project, dir, &pack, &engine).map_err(|e| {
        eprintln!("vidni: {e}");
        ExitCode::FAILURE
    })?;
    let n = |from: From| drafted.iter().filter(|d| d.from == from).count();
    say!(
        "{}: draft voice {id}: {}; {} chosen afresh from {} takes each, {} spoken from {}, {} from the cache",
        file.display(),
        count(drafted.len(), "line"),
        n(From::Chosen),
        tts::ATTEMPTS,
        n(From::Lock),
        voices::LOCK,
        n(From::Cache)
    );
    if !pack.commercial {
        eprintln!(
            "{}: warning: the draft voice {id} is licensed {}, which doesn't allow commercial use",
            file.display(),
            pack.licence
        );
    }
    Ok(pack)
}

/// Takes the captures `cuts` need and the project folder doesn't hold yet.
pub(crate) fn take_captures(loaded: &Loaded, cuts: &[&Cut]) -> Result<(), ExitCode> {
    let Loaded { file, dir, project } = loaded;
    let plan = capture::plan(project, dir, cuts);
    if plan.is_empty() {
        say!("{}: every capture is taken", file.display());
        return Ok(());
    }
    // A still that a ratified cut froze is never taken again.
    let lock = ratified(dir)?;
    let frozen = ratify::frozen(dir, &lock, &plan);
    if !frozen.is_empty() {
        return Err(stopped(Stop::Refused(frozen)));
    }
    // Without the network, a page on the web can only fail slowly; a page
    // in the project folder is captured as ever.
    let web: Vec<&capture::Capture> = plan.iter().filter(|c| c.on_the_web()).collect();
    if let (true, Some(first)) = (crate::offline::inside(), web.first()) {
        eprintln!(
            "vidni: offline, and {} of web pages {} taken yet (the first: {})",
            count(web.len(), "capture"),
            if web.len() == 1 { "isn't" } else { "aren't" },
            first.what
        );
        return Err(ExitCode::FAILURE);
    }
    // A capture that is there and is taken again is one whose files changed.
    let changed = capture::changed_under(dir, &plan);
    if !changed.is_empty() {
        say!(
            "{}: {} changed, so what was captured from {} is taken again",
            file.display(),
            capture::have(&changed),
            if changed.len() == 1 { "it" } else { "them" }
        );
    }
    // With no network, whatever a page asks the network for is a fault.
    let limits = vidni_media::capture::Limits {
        offline: crate::offline::inside(),
        ..Default::default()
    };
    match vidni_media::capture::capture_with(project, dir, &plan, limits) {
        Ok(taken) => {
            say!(
                "{}: {} taken",
                file.display(),
                count(taken.len(), "capture")
            );
            Ok(())
        }
        Err(_) if vidni_media::cdp::interrupted() => {
            eprintln!("vidni: interrupted; no still of an unfinished scene was kept");
            Err(ExitCode::from(130))
        }
        Err(e) => {
            eprintln!("vidni: {e}");
            Err(ExitCode::FAILURE)
        }
    }
}

/// Says what a build of the project in `dir` made, a line for each file.
fn built(record: &Record, dir: &Path) {
    for line in build::summary(record, dir) {
        say!("{line}");
    }
}

pub fn speak(path: &Path) -> ExitCode {
    match load(path).and_then(|loaded| speak_lines(&loaded)) {
        Ok(_) => ExitCode::SUCCESS,
        Err(code) => code,
    }
}

pub fn capture(path: &Path) -> ExitCode {
    let done = load(path).and_then(|loaded| {
        let cuts: Vec<&Cut> = loaded.project.cuts.iter().collect();
        take_captures(&loaded, &cuts)
    });
    match done {
        Ok(()) => ExitCode::SUCCESS,
        Err(code) => code,
    }
}

pub fn draft(path: &Path, cuts: &[String], again: bool) -> ExitCode {
    let loaded = match load(path) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let made = chosen(&loaded.project, cuts).and_then(|cuts| {
        // What can't start is found out before anything is spoken.
        let setup = setup(&loaded, again)?;
        let pack = speak_lines(&loaded)?;
        take_captures(&loaded, &cuts)?;
        for cut in cuts {
            let mut progress = progress(&cut.id);
            let Loaded { dir, project, .. } = &loaded;
            let record =
                build::draft(project, dir, cut, &pack, &setup, &mut progress).map_err(stopped)?;
            built(&record, dir);
            // A draft is for watching, so what QA finds is said, not refused.
            for problem in record.problems() {
                eprintln!("vidni: warning: {problem}");
            }
        }
        Ok(())
    });
    match made {
        Ok(()) => ExitCode::SUCCESS,
        Err(code) => code,
    }
}

/// The lock of ratified cuts, or why it can't be read.
fn ratified(dir: &Path) -> Result<ratify::Lock, ExitCode> {
    ratify::read_lock(dir).map_err(cant)
}

pub fn finish(path: &Path, cuts: &[String], again: bool) -> ExitCode {
    let loaded = match load(path) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let Loaded { dir, project, .. } = &loaded;
    let made = chosen(project, cuts).and_then(|cuts| {
        let lock = ratified(dir)?;
        // Every cut's gates come first, so that nothing is rendered for
        // a build that would be refused.
        let refusals: Vec<String> = cuts
            .iter()
            .flat_map(|cut| release::refusals(project, dir, &lock, cut))
            .collect();
        if !refusals.is_empty() {
            return Err(stopped(Stop::Refused(refusals)));
        }
        let setup = setup(&loaded, again)?;
        // Sessions that differ in level are worth a listen; they stop nothing.
        for jump in crate::takes::jumps(project, dir) {
            eprintln!("vidni: warning: {jump}");
        }
        for cut in cuts {
            let mut progress = progress(&cut.id);
            let record = build::final_build(project, dir, &lock, cut, &setup, &mut progress)
                .map_err(stopped)?;
            built(&record, dir);
            // What goes with the cut where it is uploaded, made afresh.
            for warning in build::exports(project, dir, cut, &setup).map_err(stopped)? {
                eprintln!("vidni: warning: {warning}");
            }
            say!(
                "{}: delivered to {}; `vidni approve --cut {}` releases it",
                cut.id,
                dir.join(release::OUT).display(),
                cut.id
            );
            for platform in cut.platforms() {
                say!(
                    "  {}: the upload for {platform}, with its metadata.md",
                    dir.join(release::OUT)
                        .join(&platform)
                        .join(&cut.id)
                        .display()
                );
            }
        }
        Ok(())
    });
    match made {
        Ok(()) => ExitCode::SUCCESS,
        Err(code) => code,
    }
}

/// Says a release's tag and hashes.
fn released(release: &release::Release) {
    let build = &release.build;
    say!("  content         {}", build.sources.content);
    say!("  t1 timeline     {}", build.t1.timeline);
    say!(
        "  t1 captions     {} (srt), {} (vtt)",
        build.t1.srt,
        build.t1.vtt
    );
    if let Some(chapters) = &build.t1.chapters {
        say!("  t1 chapters     {chapters}");
    }
    say!("  t1 pcm          {}", build.t1.pcm);
    for (name, output) in &build.outputs {
        say!(
            "  t1 frames       {}  {name}, {} frames",
            output.frames,
            output.frame_count
        );
        say!("  t2 packets      {}  {name}", output.packets);
        say!("  t2 file         {}  {name}", output.mp4);
    }
}

pub fn approve(path: &Path, cuts: &[String]) -> ExitCode {
    let loaded = match load(path) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let Loaded { dir, project, .. } = &loaded;
    let approved = chosen(project, cuts).and_then(|cuts| {
        let lock = ratified(dir)?;
        // The releases so far must be readable before one is added.
        release::read(dir).map_err(cant)?;
        // Each cut stands alone: one that is refused doesn't keep another
        // from being approved. The gates come before the tools, so that a
        // refusal needs none.
        let mut refused = None;
        let mut allowed = Vec::new();
        for cut in cuts {
            match release::refusals(project, dir, &lock, cut) {
                refusals if refusals.is_empty() => allowed.push(cut),
                refusals => refused = Some(stopped(Stop::Refused(refusals))),
            }
        }
        if allowed.is_empty() {
            return refused.map_or(Ok(()), Err);
        }
        let setup = setup(&loaded, false)?;
        let day = release::today();
        for cut in allowed {
            match build::approve(project, dir, &lock, cut, &setup, &day) {
                Ok(Approval::Approved(release)) => {
                    say!(
                        "{}: approved as {}, on {}",
                        cut.id,
                        release.tag,
                        release.approved
                    );
                    released(&release);
                }
                Ok(Approval::Already(release)) => {
                    say!(
                        "{}: already approved as {}, on {}",
                        cut.id,
                        release.tag,
                        release.approved
                    );
                    released(&release);
                }
                Err(stop) => refused = Some(stopped(stop)),
            }
        }
        refused.map_or(Ok(()), Err)
    });
    match approved {
        Ok(()) => ExitCode::SUCCESS,
        Err(code) => code,
    }
}

pub fn new(name: &str, from: Option<&Path>) -> ExitCode {
    use vidni_core::starter;
    if !vidni_core::validate::valid_id(name) {
        eprintln!(
            "vidni: a project's name is lowercase letters and digits, in words joined by hyphens, such as my-first-video; {name:?} isn't"
        );
        return ExitCode::FAILURE;
    }
    let outline = match from.map(std::fs::read_to_string) {
        None => None,
        Some(Ok(text)) => Some(starter::outline(&text)),
        Some(Err(e)) => {
            let file = from.unwrap_or(Path::new(""));
            return cant(format!("{}: {e}", file.display()));
        }
    };
    let dir = Path::new(name);
    let made = starter::files(&starter::title_of(name), outline.as_ref())
        .and_then(|files| starter::write(dir, &files).map(|()| files));
    let files = match made {
        Ok(files) => files,
        Err(e) => {
            eprintln!("vidni: {e}");
            return ExitCode::FAILURE;
        }
    };
    // Said as `vidni validate` says it, from the file just written.
    let project = files
        .first()
        .and_then(|(_, text)| vidni_core::validate::check(text, Some(dir)).ok());
    let Some(project) = project else {
        return cant(format!("{name}: the new project didn't come out valid"));
    };
    say!("{name}: a new project, {:?}", project.meta.title);
    say!(
        "  {name}/{}  {} in {}, {}, {}",
        starter::PROJECT_FILE,
        count(project.lines.len(), "line"),
        count(project.sections.len(), "section"),
        count(project.scenes.len(), "scene"),
        count(project.cuts.len(), "cut")
    );
    say!("  {name}/{}   the page its scene shows", starter::PAGE_FILE);
    say!("  {name}/.gitignore  keeps build/, out/ and takes/ out of git");
    say!("next: `vidni draft {name}` renders it as it is");
    ExitCode::SUCCESS
}

/// A length in seconds as minutes and seconds: `5:06`.
fn clock(seconds: f64) -> String {
    let whole = seconds.round() as u64;
    format!("{}:{:02}", whole / 60, whole % 60)
}

pub fn plan(path: &Path, cuts: &[String]) -> ExitCode {
    use vidni_media::sheet::{self, Take};
    let loaded = match load(path) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let Loaded { dir, project, .. } = &loaded;
    let listed = chosen(project, cuts).and_then(|cuts| {
        // The draft voice's pack tells which clips are of the lines as
        // they are now; without it the sheet has no pace.
        let pack = voices::find(&project.voice.draft, voices::store().as_deref()).map_err(cant)?;
        sheet::sheet(project, dir, &cuts, pack.as_ref()).map_err(cant)
    });
    let rows = match listed {
        Ok(rows) => rows,
        Err(code) => return code,
    };
    let n = |take: Take| rows.iter().filter(|r| r.take == take).count();
    let words: usize = rows.iter().map(|r| r.text.split_whitespace().count()).sum();
    let mut sections: Vec<&str> = rows.iter().map(|r| r.section.as_str()).collect();
    sections.dedup();
    let paced: Vec<f64> = rows.iter().filter_map(|r| r.pace).collect();
    let pace = match paced.len() {
        0 => "the draft voice hasn't spoken them yet".to_string(),
        spoken if spoken == rows.len() => format!(
            "about {} in the draft voice, without the pauses",
            clock(paced.iter().sum())
        ),
        spoken => format!("{spoken} of them spoken by the draft voice"),
    };
    say!("{}: the recording sheet", project.meta.title);
    say!(
        "{} in {}, {}; {} recorded, {} stale, {} to record; {pace}",
        count(rows.len(), "line"),
        count(sections.len(), "section"),
        count(words, "word"),
        n(Take::Recorded),
        n(Take::Stale),
        n(Take::Missing)
    );
    let width = rows.iter().map(|r| r.id.len()).max().unwrap_or(0);
    let mut section = None;
    for row in &rows {
        if section != Some(&row.section) {
            section = Some(&row.section);
            say!("");
            match row.label.is_empty() {
                true => say!("({})", row.section),
                false => say!("{}", row.label),
            }
        }
        let mark = match row.take {
            Take::Recorded => "[x]",
            Take::Stale => "[~]",
            Take::Missing => "[ ]",
        };
        let pace = match row.pace {
            Some(seconds) => format!("{seconds:5.1} s"),
            None => "      -".to_string(),
        };
        say!("  {mark} {:width$}  {pace}  {}", row.id, row.text);
    }
    ExitCode::SUCCESS
}
