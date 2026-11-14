//! One line's takes (TODO.md, P4-VIDNI-04): its raw takes, which of them
//! its kept take is cut from, and picking another or cutting it elsewhere.
//! A kept take is always made from a raw one, which is never changed, so
//! nothing done here can lose a recording.

use crate::{Loaded, load};
use std::path::Path;
use std::process::ExitCode;
use vidni_core::project::Project;
use vidni_core::spoken::spoken_hash;
use vidni_core::takes::{self, Status, Trim};
use vidni_record::{clean, take};

/// `from:to` in seconds, as `--trim` takes it.
fn ends(text: &str) -> Result<(f64, f64), String> {
    let wrong = || format!("--trim takes FROM:TO in seconds, such as 0.35:2.8, not {text:?}");
    let (from, to) = text.split_once(':').ok_or_else(wrong)?;
    let (from, to): (f64, f64) = (
        from.trim().parse().map_err(|_| wrong())?,
        to.trim().parse().map_err(|_| wrong())?,
    );
    match from.is_finite() && to.is_finite() && from >= 0.0 && to > from {
        true => Ok((from, to)),
        false => Err(wrong()),
    }
}

/// Lists the raw takes of line `id`; or, with `pick` or `trim`, makes its
/// kept take again from the raw take picked, cut where `trim` says.
pub fn line(path: &Path, id: &str, pick: Option<u32>, trim: Option<&str>) -> ExitCode {
    let Loaded { dir, project, .. } = match load(path) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };
    let Some(line) = project.lines.iter().find(|l| l.id == id) else {
        eprintln!("vidni: there is no line {id:?}");
        return ExitCode::FAILURE;
    };
    let hash = spoken_hash(line, &project.voice);
    let kept = takes::kept(&dir, id, &hash);
    let raws = takes::raws(&dir, id, &hash);
    let done = match (pick, trim) {
        (None, None) => list(&dir, id, &line.show, &kept, &raws),
        _ => repick(&dir, id, &kept, &raws, pick, trim),
    };
    match done {
        Ok(()) => ExitCode::SUCCESS,
        Err(why) => {
            eprintln!("vidni: {why}");
            ExitCode::FAILURE
        }
    }
}

fn shown(dir: &Path, path: &Path) -> String {
    path.strip_prefix(dir).unwrap_or(path).display().to_string()
}

fn list(
    dir: &Path,
    id: &str,
    words: &str,
    kept: &Path,
    raws: &[(u32, std::path::PathBuf)],
) -> Result<(), String> {
    say!("{id}: {words}");
    if raws.is_empty() {
        say!("  no raw take of these words; `vidni record --only {id}` records one");
    }
    let picked = takes::pick(kept)?.filter(|_| kept.is_file());
    for (n, raw) in raws {
        let (samples, rate) = take::read(raw)?;
        let seconds = samples.len() as f64 / f64::from(rate.max(1));
        let mark = match picked.as_ref().is_some_and(|p| p.raw == *n) {
            true => "*",
            false => " ",
        };
        let session = match takes::session(raw)? {
            Some(s) => format!("recorded {} from {}", s.recorded, s.microphone),
            None => "recorded before sessions were noted".to_string(),
        };
        say!(
            "{mark} {n:>2}  {}  {seconds:.2} s, peak {:.1} dB; {session}",
            shown(dir, raw),
            clean::peak_db(&samples)
        );
    }
    match (&picked, kept.is_file()) {
        (Some(pick), _) => {
            let rate = take::read(kept)?.1;
            let at = |sample: usize| sample as f64 / f64::from(rate.max(1));
            say!(
                "kept: {}, raw take {} from {:.2} s to {:.2} s ({})",
                shown(dir, kept),
                pick.raw,
                at(pick.from),
                at(pick.to),
                match pick.trim {
                    Trim::Auto => "where vidni found the words",
                    Trim::Set => "where --trim put its ends",
                }
            );
        }
        (None, true) => say!(
            "kept: {}; which raw take it is from wasn't recorded",
            shown(dir, kept)
        ),
        (None, false) => say!("kept: no take of these words"),
    }
    Ok(())
}

fn repick(
    dir: &Path,
    id: &str,
    kept: &Path,
    raws: &[(u32, std::path::PathBuf)],
    pick: Option<u32>,
    trim: Option<&str>,
) -> Result<(), String> {
    let trim = trim.map(ends).transpose()?;
    // The raw take: the one named, or the one the kept take is from.
    let number = match pick {
        Some(n) => n,
        None => takes::pick(kept)?.map(|p| p.raw).ok_or_else(|| {
            format!(
                "which raw take line {id}'s kept take is from wasn't recorded; name one with --pick N"
            )
        })?,
    };
    let Some((_, raw)) = raws.iter().find(|(n, _)| *n == number) else {
        let have: Vec<String> = raws.iter().map(|(n, _)| n.to_string()).collect();
        return Err(match have.is_empty() {
            true => format!("line {id} has no raw take of its present words"),
            false => format!(
                "line {id} has no raw take {number}; it has {}",
                have.join(", ")
            ),
        });
    };
    let candidate = take::cut(raw, kept, trim)?.map_err(|refused| refused.to_string())?;
    let (from, to, seconds) = (candidate.from, candidate.to, candidate.seconds);
    let rate = take::read(raw)?.1;
    candidate.keep()?;
    let at = |sample: usize| sample as f64 / f64::from(rate.max(1));
    say!(
        "{id}: its kept take is raw take {number}, from {:.2} s to {:.2} s: {seconds:.2} s",
        at(from),
        at(to)
    );
    say!("  {}", shown(dir, kept));
    Ok(())
}

/// What to say of sessions whose takes are louder or quieter than the
/// rest: one line each. Lines whose take has no session on record are
/// left out, as nothing is known of them.
pub fn jumps(project: &Project, dir: &Path) -> Vec<String> {
    let mut levels = Vec::new();
    for line in takes::status(project, dir) {
        let Status::Current(kept) = &line.status else {
            continue;
        };
        let session = takes::pick(kept)
            .ok()
            .flatten()
            .and_then(|pick| takes::session(&takes::raw(dir, &line.id, &line.hash, pick.raw)).ok())
            .flatten();
        if let (Some(session), Ok((samples, rate))) = (session, take::read(kept)) {
            let level = clean::speech_db(&samples, rate);
            levels.push((line.id.clone(), session.session, level));
        }
    }
    takes::jumps(&levels)
        .iter()
        .map(|jump| {
            let lines: Vec<&str> = jump.lines.iter().take(5).map(String::as_str).collect();
            let more = match jump.lines.len() - lines.len() {
                0 => String::new(),
                n => format!(" and {n} more"),
            };
            format!(
                "the takes of the session of {} are {:.1} dB {} than those of {}: {}{more}; \
                 listen across the join, and record them again if it is heard",
                jump.session,
                jump.by.abs(),
                if jump.by > 0.0 { "louder" } else { "quieter" },
                jump.than,
                lines.join(", ")
            )
        })
        .collect()
}
