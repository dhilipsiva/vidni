//! `vidni-parity <stage> [--candidate DIR | --reference-self] [--golden DIR] [--reference DIR]`
//! `vidni-parity selftest [--golden DIR] [--reference DIR]`
//!
//! Compares a candidate folder with the golden outputs (TODO.md, P1-VIDNI-04).
//! The stages are timeline, narration, encode, cuts, stills and all.
//! `--reference-self` uses the reference pipeline's own outputs as the
//! candidate, which must pass. `selftest` shows that the harness passes the
//! reference itself and catches a deliberately perturbed output in each stage.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use vidni_parity::{STAGES, Sources, check_stage, reference_as_candidate, selftest::selftest};

const USAGE: &str = "usage: vidni-parity <timeline|narration|encode|cuts|stills|all> \
                     [--candidate DIR | --reference-self] [--golden DIR] [--reference DIR]\n       \
                     vidni-parity selftest [--golden DIR] [--reference DIR]";

fn main() -> ExitCode {
    match run() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("vidni-parity: {e}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<bool, String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut args = std::env::args().skip(1);
    let stage = args.next().ok_or(USAGE)?;
    if !["all", "selftest"].contains(&stage.as_str()) && !STAGES.contains(&stage.as_str()) {
        return Err(format!("unknown stage {stage}\n{USAGE}"));
    }
    let mut candidate = None;
    let mut reference_self = false;
    let mut golden = root.join("golden");
    let mut reference = std::env::var_os("VIDNI_REFERENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("reference"));
    while let Some(flag) = args.next() {
        let mut value = || {
            args.next()
                .map(PathBuf::from)
                .ok_or_else(|| format!("{flag} needs a value"))
        };
        match flag.as_str() {
            "--candidate" => candidate = Some(value()?),
            "--golden" => golden = value()?,
            "--reference" => reference = value()?,
            "--reference-self" => reference_self = true,
            _ => return Err(format!("unknown argument {flag}\n{USAGE}")),
        }
    }
    let existing = |p: PathBuf| {
        p.canonicalize()
            .map_err(|e| format!("{}: {e}", p.display()))
    };
    let sources = Sources {
        golden: existing(golden)?,
        reference: existing(reference)?,
    };
    let scratch = std::env::temp_dir().join(format!("vidni-parity-{}", std::process::id()));
    fs::create_dir_all(&scratch).map_err(|e| e.to_string())?;
    let result = match (stage.as_str(), candidate, reference_self) {
        ("selftest", None, false) => selftest(&sources, &scratch),
        ("selftest", _, _) => Err(format!("selftest takes no candidate\n{USAGE}")),
        (_, Some(c), false) => compare(&stage, &c, &sources, &scratch),
        (_, None, true) => {
            let c = scratch.join("candidate");
            fs::create_dir_all(&c)
                .map_err(|e| e.to_string())
                .and_then(|_| reference_as_candidate(&sources.reference, &c))
                .and_then(|_| compare(&stage, &c, &sources, &scratch))
        }
        _ => Err(format!(
            "give exactly one of --candidate and --reference-self\n{USAGE}"
        )),
    };
    let _ = fs::remove_dir_all(&scratch);
    result
}

fn compare(
    stage: &str,
    candidate: &Path,
    sources: &Sources,
    scratch: &Path,
) -> Result<bool, String> {
    let report = check_stage(stage, candidate, sources, scratch)?;
    let passed = report.checks - report.failures.len();
    println!("parity {stage}: {passed} of {} checks pass", report.checks);
    for f in &report.failures {
        println!("  FAIL {f}");
    }
    Ok(report.failures.is_empty() && report.checks > 0)
}
