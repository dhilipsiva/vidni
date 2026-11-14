//! `vidni-parity <stage> [--candidate DIR | --reference-self] [--golden DIR] [--reference DIR]`
//! `vidni-parity selftest [--golden DIR] [--reference DIR]`
//! `vidni-parity compose-t1 [--reference DIR] [--example DIR] [--fonts CAPTION MONO DISPLAY]`
//! `vidni-parity compose-stream [--reference DIR] [--example DIR]`
//! `vidni-parity capture-repeat [--example DIR]`
//! `vidni-parity product [--example DIR] [--golden DIR] [--reference DIR]`
//!
//! Compares a candidate folder with the golden outputs (TODO.md, P1-VIDNI-04).
//! The stages are timeline, narration, encode, cuts, stills and all. With
//! neither `--candidate` nor `--reference-self`, the Rust port of the stage
//! builds the candidate. `--reference-self` uses the reference pipeline's
//! own outputs as the candidate, which must pass. `selftest` shows that the harness passes the
//! reference itself and catches a deliberately perturbed output in each stage.
//! `compose-t1` renders every cut twice and compares the frame streams, and
//! `compose-stream` compares every picture of every cut with the reference's
//! own (P3-VIDNI-02). `capture-repeat` captures the example's web scene
//! five times, each from scratch, and compares every still's pixels
//! (P3-VIDNI-04). `product` checks the example's final build, as `vidni
//! final` plans it from the example's own captures and takes, against the
//! reference's cuts in the creator's voice (P3-VIDNI-06).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use vidni_media::compose::render;
use vidni_parity::candidate::build;
use vidni_parity::compose::{Faces, STREAM_TOLERANCE, Streamed, jobs, reference_faces, stream};
use vidni_parity::{STAGES, Sources, check_stage, reference_as_candidate, selftest::selftest};

const USAGE: &str = "usage: vidni-parity <timeline|narration|encode|cuts|stills|all> \
                     [--candidate DIR | --reference-self] [--golden DIR] [--reference DIR]\n       \
                     vidni-parity selftest [--golden DIR] [--reference DIR]\n       \
                     vidni-parity compose-t1 [--reference DIR] [--example DIR] [--fonts CAPTION MONO DISPLAY]\n       \
                     vidni-parity compose-stream [--reference DIR] [--example DIR]\n       \
                     vidni-parity capture-repeat [--example DIR]\n       \
                     vidni-parity product [--example DIR] [--golden DIR] [--reference DIR]";

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
    if ![
        "all",
        "selftest",
        "compose-t1",
        "compose-stream",
        "capture-repeat",
        "product",
    ]
    .contains(&stage.as_str())
        && !STAGES.contains(&stage.as_str())
    {
        return Err(format!("unknown stage {stage}\n{USAGE}"));
    }
    let mut candidate = None;
    let mut reference_self = false;
    let mut golden = root.join("golden");
    let mut reference = std::env::var_os("VIDNI_REFERENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("reference"));
    let mut example = root.join("examples/nibli-syllogism");
    let mut fonts: Option<Faces> = None;
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
            "--example" => example = value()?,
            "--fonts" => {
                let mut next = || {
                    args.next()
                        .map(PathBuf::from)
                        .ok_or("--fonts needs three files")
                };
                fonts = Some(Faces {
                    caption: next()?,
                    mono: next()?,
                    display: next()?,
                });
            }
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
        ("compose-t1", None, false) => compose_t1(&example, &sources.reference, fonts),
        ("compose-stream", None, false) => compose_stream(&example, &sources.reference),
        ("capture-repeat", None, false) => vidni_parity::capture::repeat(&example, &scratch, 5),
        ("product", None, false) => {
            let mut report = vidni_parity::Report::default();
            vidni_parity::product::check(&example, &sources, &scratch, &mut report);
            Ok(print("product", &report))
        }
        ("selftest", _, _) => Err(format!("selftest takes no candidate\n{USAGE}")),
        (_, Some(c), false) => compare(&stage, &c, &sources, &scratch),
        (_, None, true) => {
            let c = scratch.join("candidate");
            fs::create_dir_all(&c)
                .map_err(|e| e.to_string())
                .and_then(|_| reference_as_candidate(&sources, &c, &scratch))
                .and_then(|_| compare(&stage, &c, &sources, &scratch))
        }
        ("all", None, false) => Err(format!(
            "all needs --candidate or --reference-self; check the ported stages one by one\n{USAGE}"
        )),
        (_, None, false) => {
            let c = scratch.join("candidate");
            fs::create_dir_all(&c)
                .map_err(|e| e.to_string())
                .and_then(|_| build(&stage, &example, &sources.reference, &c))
                .and_then(|_| compare(&stage, &c, &sources, &scratch))
        }
        _ => Err(format!(
            "give at most one of --candidate and --reference-self\n{USAGE}"
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
    Ok(print(
        stage,
        &check_stage(stage, candidate, sources, scratch)?,
    ))
}

/// Prints a stage's report; true when every check passed.
fn print(stage: &str, report: &vidni_parity::Report) -> bool {
    let passed = report.checks - report.failures.len();
    println!("parity {stage}: {passed} of {} checks pass", report.checks);
    if let Some((ssim, what)) = &report.lowest {
        println!("  lowest SSIM {ssim:.4}, {what}");
    }
    if let Some((difference, what)) = &report.largest {
        println!("  largest pixel difference {difference} of 255, {what}");
    }
    for f in &report.failures {
        println!("  FAIL {f}");
    }
    report.failures.is_empty() && report.checks > 0
}

/// T1 for the compositor: renders every cut twice, from scratch each time,
/// and compares the frame streams' hashes.
fn compose_t1(example: &Path, reference: &Path, fonts: Option<Faces>) -> Result<bool, String> {
    let faces = match fonts {
        Some(faces) => faces,
        None => reference_faces()?,
    };
    faces.check()?;
    for face in faces.files() {
        let bytes = fs::read(face).map_err(|e| format!("{}: {e}", face.display()))?;
        println!(
            "font {} sha256 {}",
            face.display(),
            vidni_parity::sha256_hex(&bytes)
        );
    }
    let mut renders: Vec<Vec<(String, u64, u64, String)>> = Vec::new();
    for _ in 0..2 {
        let mut hashes = Vec::new();
        for mut job in jobs(example, reference, &faces)? {
            let frames = job.composer.frames();
            let (hash, drawn) = render(&mut job.composer, |_, _| Ok(()))?;
            hashes.push((job.name, frames, drawn, hash));
        }
        renders.push(hashes);
    }
    let mut same = true;
    for (a, b) in renders[0].iter().zip(&renders[1]) {
        let agree = a.3 == b.3;
        same &= agree;
        println!(
            "{}: {} frames ({} drawn); frame stream sha256 {}; second render {}",
            a.0,
            a.1,
            a.2,
            a.3,
            if agree { "identical" } else { "DIFFERENT" }
        );
    }
    Ok(same)
}

/// Compares every picture of every cut with the reference's own (see
/// `compose::stream`), the four cuts at once.
fn compose_stream(example: &Path, reference: &Path) -> Result<bool, String> {
    let faces = reference_faces()?;
    let all = jobs(example, reference, &faces)?;
    let results: Vec<(String, Result<Streamed, String>)> = std::thread::scope(|scope| {
        let handles: Vec<_> = all
            .into_iter()
            .map(|mut job| {
                scope.spawn(move || {
                    let result = stream(&mut job, reference);
                    (job.name, result)
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| {
                h.join()
                    .unwrap_or_else(|_| ("a cut".into(), Err("its thread panicked".into())))
            })
            .collect()
    });
    let mut passed = true;
    for (name, result) in results {
        let s = match result {
            Ok(s) => s,
            Err(e) => {
                passed = false;
                println!("{name}: FAIL {e}");
                continue;
            }
        };
        passed &= s.passed();
        println!(
            "{name}: {} frames; {} pictures, changing at the same frames as the reference's: {}; \
             largest difference {} of 255 (frame {})",
            s.frames.1,
            s.pictures,
            if s.schedule.is_none() { "yes" } else { "NO" },
            s.largest.0,
            s.largest.1
        );
        if s.frames.0 != s.frames.1 {
            println!("  FAIL the reference has {} frames", s.frames.0);
        }
        if let Some(f) = s.schedule {
            println!("  FAIL only one of the two changes its picture at frame {f}");
        }
        if let Some(f) = s.over.first() {
            println!(
                "  FAIL {} pictures differ by more than {STREAM_TOLERANCE}, the first at frame {f}",
                s.over.len()
            );
        }
    }
    Ok(passed)
}
