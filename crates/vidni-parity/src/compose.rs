//! The Rust compositor on the reference's frozen captures (TODO.md,
//! P3-VIDNI-02): the `cuts` candidate, the T1 check that two renders of
//! every cut give the same frame stream, and the comparison of every
//! picture with the reference's own.
//!
//! Each line's shot comes from the reference's `capture.json` (the still and
//! its camera rectangles) and the example's `vidni.toml` (which camera,
//! cards and overlays), with the cameras chosen by vidni's rules
//! (docs/spec.md, Cameras).

use crate::VARIANTS;
use crate::candidate::{project, variants};
use serde_json::Value;
use std::fs;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use vidni_core::project::Project;
use vidni_core::timeline;
use vidni_media::build::camera_name;
use vidni_media::compose::cut::{Composer, LANDSCAPE, Layout, Look, PORTRAIT, Picture, Shot, Type};
use vidni_media::compose::text::Fonts;

/// The reference's three faces (caption, mono, display): the pattern
/// `compose.py` asks fontconfig for, and the sha256 of the file the golden
/// build drew with. fontconfig's weights aren't CSS weights, so these
/// patterns resolve to the bold faces.
pub const REFERENCE_FONTS: [(&str, &str); 3] = [
    (
        "IBM Plex Sans:weight=500",
        "19de5aec74215119b3f8f7d1b1f0e0eba867bee2d2c65c5761b287d67581c316",
    ),
    (
        "IBM Plex Mono:weight=400",
        "44f6da8e1146b28809182fe2a7a91a987a792ac294836ae1eeece09eaf35a6c7",
    ),
    (
        "Space Grotesk:weight=500",
        "be8709abe941dcc98f3625663b340ca20dc90cd6806ee76bd68b568097b6368f",
    ),
];

/// The three font files the look uses.
#[derive(Debug, Clone)]
pub struct Faces {
    pub caption: PathBuf,
    pub mono: PathBuf,
    pub display: PathBuf,
}

impl Faces {
    /// The files, as caption, mono and display.
    pub fn files(&self) -> [&Path; 3] {
        [&self.caption, &self.mono, &self.display]
    }

    /// Checks that each file is the face the golden build drew with.
    pub fn check(&self) -> Result<(), String> {
        for (file, (pattern, want)) in self.files().into_iter().zip(REFERENCE_FONTS) {
            let bytes = fs::read(file).map_err(|e| format!("{}: {e}", file.display()))?;
            let got = crate::sha256_hex(&bytes);
            if got != want {
                return Err(format!(
                    "{}: sha256 {got}, not the {pattern:?} face the reference drew with ({want})",
                    file.display()
                ));
            }
        }
        Ok(())
    }
}

/// The files the reference used: `fc-match` with its patterns, against the
/// host's fontconfig, as `compose.py` asked for them. fc-match answers with
/// some other font when it has none of the family, so each file is checked
/// by its sha256.
pub fn reference_faces() -> Result<Faces, String> {
    let find = |pattern: &str| -> Result<PathBuf, String> {
        let out = Command::new("fc-match")
            .env_remove("FONTCONFIG_FILE")
            .args(["-f", "%{file}", pattern])
            .output()
            .map_err(|e| format!("fc-match: {e}"))?;
        let path = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim());
        if path.is_file() {
            Ok(path)
        } else {
            Err(format!("fc-match found no file for {pattern:?}"))
        }
    };
    let faces = Faces {
        caption: find(REFERENCE_FONTS[0].0)?,
        mono: find(REFERENCE_FONTS[1].0)?,
        display: find(REFERENCE_FONTS[2].0)?,
    };
    faces.check()?;
    Ok(faces)
}

/// One cut to render: its variant, its aspect (`16x9` or `9x16`), its file
/// name and its compositor.
pub struct Job {
    pub variant: &'static str,
    pub cut: &'static str,
    pub name: String,
    pub composer: Composer,
}

fn rect(value: &Value) -> Option<[f64; 4]> {
    let v: Vec<f64> = value.as_array()?.iter().filter_map(Value::as_f64).collect();
    (v.len() == 4).then(|| [v[0], v[1], v[2], v[3]])
}

/// Each line's shot in `layout`, from the reference's captures.
fn shots(
    project: &Project,
    capture: &Value,
    layout: &Layout,
    build: &Path,
) -> Result<Vec<Shot>, String> {
    let viewport = [
        capture["view"]["width"]
            .as_f64()
            .ok_or("capture.json: no view width")?,
        capture["view"]["height"]
            .as_f64()
            .ok_or("capture.json: no view height")?,
    ];
    let scale = capture["dpr"].as_f64().ok_or("capture.json: no dpr")?;
    let captured = capture["shots"]
        .as_array()
        .ok_or("capture.json: no shots")?;
    if captured.len() != project.lines.len() {
        return Err(format!(
            "{} captures for {} lines",
            captured.len(),
            project.lines.len()
        ));
    }
    let portrait = layout.name == "9x16";
    project
        .lines
        .iter()
        .zip(captured)
        .map(|(line, shot)| {
            let label = project
                .sections
                .get(&line.section)
                .map(|s| s.label.clone())
                .unwrap_or_default();
            let overlay = line
                .overlay
                .as_ref()
                .map(|o| build.join(format!("cards/{o}-{}.png", layout.name)));
            let picture = match shot["card"].as_str() {
                Some(card) => {
                    Picture::Card(build.join(format!("cards/{card}-{}.png", layout.name)))
                }
                None => {
                    let scene_id = line.scene_in(project).ok_or("a line with no scene")?;
                    let scene = project.scenes.get(scene_id).ok_or("an unknown scene")?;
                    // The reference calls the highlights' camera `hl`.
                    let highlights = shot["cameras"]["hl"][2].as_f64();
                    let name = camera_name(line, scene, portrait, highlights);
                    let key = match name.as_str() {
                        "highlights" => "hl",
                        other => other,
                    };
                    let camera = if name == "viewport" {
                        [0.0, 0.0, viewport[0], viewport[1]]
                    } else {
                        rect(&shot["cameras"][key]).ok_or_else(|| {
                            format!("line {}: no {key} camera in capture.json", line.id)
                        })?
                    };
                    let still = shot["still"]
                        .as_str()
                        .ok_or_else(|| format!("line {}: no still", line.id))?;
                    Picture::Still {
                        path: build.join(still),
                        camera,
                        viewport,
                        scale,
                    }
                }
            };
            Ok(Shot {
                label,
                picture,
                overlay,
                strip: None,
            })
        })
        .collect()
}

/// The four cuts (two variants, two aspects) on the reference's captures.
pub fn jobs(example: &Path, reference: &Path, faces: &Faces) -> Result<Vec<Job>, String> {
    let project = project(example)?;
    let build = reference.join("build");
    let capture: Value = serde_json::from_str(
        &fs::read_to_string(build.join("capture.json"))
            .map_err(|e| format!("capture.json: {e}"))?,
    )
    .map_err(|e| format!("capture.json: {e}"))?;
    let mut fonts = Fonts::new();
    let faces = Type {
        caption: fonts.load(&faces.caption)?,
        mono: fonts.load(&faces.mono)?,
        display: fonts.load(&faces.display)?,
    };
    let subtitle = project.meta.subtitle.clone().unwrap_or_default();
    // The example's theme is the reference's look.
    let look = Look::of(&vidni_core::theme::load(example, &project.style.theme)?);
    let mut jobs = Vec::new();
    for ((variant, suffix), clips) in VARIANTS
        .into_iter()
        .zip(variants(&project, example, reference)?)
    {
        let infos: Vec<_> = clips.iter().map(|(_, c)| c.clone()).collect();
        let timeline = timeline::build(&project, &infos)?;
        for layout in [LANDSCAPE, PORTRAIT] {
            let shots = shots(&project, &capture, &layout, &build)?;
            let composer = Composer::new(
                layout, &timeline, shots, &mut fonts, &faces, &look, &subtitle,
            )?;
            jobs.push(Job {
                variant,
                cut: layout.name,
                name: format!("nibli-syllogism{suffix}-{}", layout.name),
                composer,
            });
        }
    }
    Ok(jobs)
}

/// The largest difference allowed between a channel value of vidni's
/// picture and the reference's. swash's anti-aliasing at glyph edges isn't
/// quite FreeType's.
pub const STREAM_TOLERANCE: u8 = 2;

/// What comparing one cut with the reference's own pictures found.
#[derive(Debug)]
pub struct Streamed {
    /// The reference's frame count and vidni's.
    pub frames: (u64, u64),
    /// How many pictures were compared: one per frame where the reference's
    /// picture changes.
    pub pictures: usize,
    /// The first frame where only one of the two changes its picture.
    pub schedule: Option<u64>,
    /// The largest difference between two channel values, and its frame.
    pub largest: (u8, u64),
    /// The pictures that differ by more than [`STREAM_TOLERANCE`].
    pub over: Vec<u64>,
}

impl Streamed {
    pub fn passed(&self) -> bool {
        self.frames.0 == self.frames.1 && self.schedule.is_none() && self.over.is_empty()
    }
}

/// Compares every picture of `job` with the reference's own, which
/// `tools/reference_frames.py --stream` sends from `compose.py` in the
/// reference's nix shell: the frames where the picture changes, and each
/// changed picture's pixels. This covers what the sampled frames of the
/// `cuts` check can miss: crossfades, camera moves, captions coming and
/// going, and the frames vidni reuses instead of drawing again.
pub fn stream(job: &mut Job, reference: &Path) -> Result<Streamed, String> {
    let tool = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/reference_frames.py");
    let reference =
        fs::canonicalize(reference).map_err(|e| format!("{}: {e}", reference.display()))?;
    let mut child = Command::new("nix")
        .env_remove("LD_LIBRARY_PATH")
        .env_remove("FONTCONFIG_FILE")
        .arg("develop")
        .arg(&reference)
        .args(["--command", "python3", "-I", "-B"])
        .arg(&tool)
        .arg("--reference")
        .arg(&reference)
        .args(["--variant", job.variant, "--fonts"])
        .args(REFERENCE_FONTS.map(|(_, sha256)| sha256))
        .args(["--stream", job.cut])
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("nix develop: {e}"))?;
    let mut input = BufReader::with_capacity(1 << 24, child.stdout.take().ok_or("no stdout")?);
    let (w, h) = job.composer.size();
    let mut theirs = vec![0u8; (w * h * 3) as usize];
    let name = job.name.clone();
    let mut word = [0u8; 8];
    let mut read_u64 = |input: &mut BufReader<_>| -> Result<u64, String> {
        input
            .read_exact(&mut word)
            .map_err(|e| format!("{name}: the reference's stream ended early: {e}"))?;
        Ok(u64::from_le_bytes(word))
    };
    let mut changes = Vec::new();
    let mut largest = (0u8, 0u64);
    let mut over = Vec::new();
    let count = loop {
        let f = read_u64(&mut input)?;
        if f == u64::MAX {
            break read_u64(&mut input)?;
        }
        input
            .read_exact(&mut theirs)
            .map_err(|e| format!("{}: frame {f}: {e}", job.name))?;
        changes.push(f);
        let ours = job.composer.frame(f)?;
        let difference = ours
            .data
            .iter()
            .zip(&theirs)
            .map(|(a, b)| a.abs_diff(*b))
            .max()
            .unwrap_or(0);
        if difference > largest.0 {
            largest = (difference, f);
        }
        if difference > STREAM_TOLERANCE {
            over.push(f);
        }
    };
    let status = child.wait().map_err(|e| e.to_string())?;
    if !status.success() {
        return Err(format!(
            "{}: the reference's stream failed: {status}",
            job.name
        ));
    }
    let mut ours = Vec::new();
    let mut last = None;
    for f in 0..job.composer.frames() {
        let key = job.composer.key(f);
        if last != Some(key) {
            ours.push(f);
        }
        last = Some(key);
    }
    let schedule = changes
        .iter()
        .zip(&ours)
        .find(|(a, b)| a != b)
        .map(|(a, b)| *a.min(b))
        .or_else(|| {
            (changes.len() != ours.len()).then(|| {
                let n = changes.len().min(ours.len());
                changes.get(n).or(ours.get(n)).copied().unwrap_or_default()
            })
        });
    Ok(Streamed {
        frames: (count, job.composer.frames()),
        pictures: changes.len(),
        schedule,
        largest,
        over,
    })
}
