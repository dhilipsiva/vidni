//! Capture (TODO.md, P3-VIDNI-04): a web scene's lines become stills and a
//! page scene becomes renders, through Chromium over the DevTools protocol.
//!
//! A web scene is loaded once and its setup run. Then each of its lines
//! runs in script order: the line's actions; then the scene's expectations
//! at that line are waited for and read; then its still is taken, when a
//! cut needs one that the project folder doesn't hold yet. After every
//! action the page must settle: its animations are finished, its network is
//! quiet, its fonts are loaded, and two screenshots in a row, a quarter
//! second apart, are identical. A scene's new stills are kept aside until
//! its expectations have held and no script error or dialog has come up,
//! and only then moved in, each with its frames beside it (`<key>.json`):
//! the boxes of every target of the scene and of the line's highlights,
//! in CSS pixels. Cameras are framed from them when a cut is built.
//!
//! A page that is a file of the project's is captured with a record of the
//! project's files it loaded, each with its SHA-256 (`<key>.files.json`),
//! so that the capture is known to be out of date once one of them changes
//! (TODO.md, P2-VIDNI-09).
//!
//! Chromium starts with the switches the reference's Playwright used, plus
//! `--font-render-hinting=none` and `--force-color-profile=srgb`, and the
//! page is set up as Playwright sets one up. Three things keep a still
//! from varying from one launch to the next (DECISIONS.md, T6): a web scene
//! above scale 1 is loaded twice, the first time inertly; each layer is
//! rastered as one tile; and vidni's style and highlight layer are in
//! every document from its start. vidni's runtime runs in its own isolated
//! world, where the page's scripts can't reach it.

use crate::cdp::Browser;
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, Instant};
use vidni_core::capture::{self as keys, Capture, Files, Source};
use vidni_core::onscreen::{self, Reading, Readings};
use vidni_core::project::{Action, ColorScheme, Project, Scene, SceneKind};

const RUNTIME: &str = include_str!("capture/runtime.js");

/// What a captured page shows on top of itself: the highlight boxes, in
/// the reference's look until themes arrive (P5-VIDNI-01), and no caret.
const STYLE: &str = "#vidni-layer { position: fixed; inset: 0; pointer-events: none; z-index: 2147483647; }\n\
    #vidni-layer .vidni-box { position: absolute; box-sizing: border-box; border: 2px solid #F2542D; \
    border-radius: 7px; background: rgba(242, 84, 45, 0.07); \
    box-shadow: 0 0 0 3px rgba(242, 84, 45, 0.20), 0 0 18px rgba(242, 84, 45, 0.38); }\n\
    *, *::before, *::after { caret-color: transparent !important; }";

/// The name of vidni's isolated world in a page.
const WORLD: &str = "vidni";

/// What runs in vidni's world at the start of every document: the runtime,
/// which then puts in the style and the highlight layer.
fn boot() -> String {
    format!("{RUNTIME}\nwindow.__vidni.mount({});", json!(STYLE))
}

/// Takes the focus off whatever has it, so that no still shows a focus ring.
const BLUR: &str =
    "(() => { const a = document.activeElement; if (a && a.blur) a.blur(); return true; })()";

/// Chromium's switches: Playwright 1.63's for headless Chrome, which made
/// the reference's stills, and vidni's own (DECISIONS.md, T6): no font
/// hinting, the sRGB colour profile, and one raster tile for each layer.
/// The sandbox stays on unless [`switches`] is told otherwise.
pub const SWITCHES: &[&str] = &[
    "--disable-field-trial-config",
    "--disable-background-networking",
    "--disable-background-timer-throttling",
    "--disable-backgrounding-occluded-windows",
    "--disable-back-forward-cache",
    "--disable-breakpad",
    "--disable-client-side-phishing-detection",
    "--disable-component-extensions-with-background-pages",
    "--disable-component-update",
    "--no-default-browser-check",
    "--disable-default-apps",
    "--disable-dev-shm-usage",
    "--disable-extensions",
    "--disable-features=AvoidUnnecessaryBeforeUnloadCheckSync,DestroyProfileOnBrowserClose,\
     DialMediaRouteProvider,GlobalMediaControls,HttpsUpgrades,LensOverlay,MediaRouter,\
     PaintHolding,ThirdPartyStoragePartitioning,BlockOriginHeaderModificationOnRedirect,\
     Translate,AutoDeElevate,OptimizationHints",
    "--enable-features=CDPScreenshotNewSurface",
    "--allow-pre-commit-input",
    "--disable-hang-monitor",
    "--disable-ipc-flooding-protection",
    "--disable-popup-blocking",
    "--disable-prompt-on-repost",
    "--disable-renderer-backgrounding",
    "--force-color-profile=srgb",
    "--metrics-recording-only",
    "--no-first-run",
    "--password-store=basic",
    "--use-mock-keychain",
    "--no-service-autorun",
    "--export-tagged-pdf",
    "--disable-search-engine-choice-screen",
    "--disable-infobars",
    "--disable-sync",
    "--enable-unsafe-swiftshader",
    "--headless",
    "--hide-scrollbars",
    "--mute-audio",
    "--blink-settings=primaryHoverType=2,availableHoverTypes=2,primaryPointerType=4,availablePointerTypes=4",
    "--font-render-hinting=none",
    // One raster tile for each layer. Chromium placed its tiles differently
    // from one launch to the next, and a curve's edge came out a few levels
    // different where the tiles fell differently.
    "--default-tile-width=8192",
    "--default-tile-height=8192",
    "--max-untiled-layer-width=8192",
    "--max-untiled-layer-height=8192",
];

/// The gap between the screenshots that tell a page has settled.
const SETTLE: Duration = Duration::from_millis(250);
/// A request in flight for longer than this is one the page keeps open, and
/// settling stops waiting for it.
const NETWORK: Duration = Duration::from_secs(10);
/// How many screenshots in a row must be identical for a page whose network
/// stays busy to count as settled.
const STEADY: u32 = 5;
/// A scene's staging folder untouched for this long was left by a capture
/// that was killed.
const STALE: Duration = Duration::from_secs(600);

/// How patient a capture is.
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    /// How long a page may take to load or settle, a target to appear, a
    /// click's target to come out from under something, or an expectation
    /// to hold.
    pub patience: Duration,
    /// Whether the capture runs with no network. Then anything a page
    /// asks the network for is a fault, on any page: a still taken without
    /// it would be kept as if it were whole.
    pub offline: bool,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            patience: Duration::from_secs(30),
            offline: false,
        }
    }
}

/// The Chromium that captures: `$VIDNI_CHROMIUM`, or `chromium` on the path.
pub fn chromium() -> PathBuf {
    std::env::var_os("VIDNI_CHROMIUM")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("chromium"))
}

/// The switches for this host: [`SWITCHES`], plus `--no-sandbox` when
/// `$VIDNI_NO_SANDBOX` is set, for a host whose kernel doesn't let Chromium
/// build its sandbox (some CI runners).
pub fn switches() -> Vec<&'static str> {
    let mut switches = SWITCHES.to_vec();
    if std::env::var_os("VIDNI_NO_SANDBOX").is_some_and(|v| !v.is_empty()) {
        switches.push("--no-sandbox");
    }
    switches
}

/// A web still's frames, written beside it as `<key>.json`: where the
/// scene's targets and the line's highlights are in the still, in CSS
/// pixels. Cameras aren't in it. They are framed from the targets when a
/// cut is built ([`Frames::camera`]), so a camera can change without a
/// new capture.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Frames {
    /// The viewport in CSS pixels, and the device pixels per CSS pixel.
    pub viewport: [u32; 2],
    pub scale: u32,
    /// The boxes of each of the scene's targets, as far as they show; none
    /// for a target that matches nothing in the still.
    pub targets: BTreeMap<String, Vec<[f64; 4]>>,
    /// Each highlight's boxes as drawn, by target.
    pub highlights: BTreeMap<String, Vec<[f64; 4]>>,
    /// The browser that took the still.
    pub browser: String,
}

/// The box around `boxes`, as the page runtime joins them; none of none.
fn union<'b>(boxes: impl Iterator<Item = &'b [f64; 4]>) -> Option<[f64; 4]> {
    let mut joined: Option<[f64; 4]> = None;
    for b in boxes {
        // Left, top, right and bottom, as the runtime's min and max take them.
        let edges = [b[0], b[1], b[0] + b[2], b[1] + b[3]];
        joined = Some(match joined {
            None => edges,
            Some(j) => [
                j[0].min(edges[0]),
                j[1].min(edges[1]),
                j[2].max(edges[2]),
                j[3].max(edges[3]),
            ],
        });
    }
    joined.map(|j| [j[0], j[1], j[2] - j[0], j[3] - j[1]])
}

impl Frames {
    /// The box that camera `name` of `scene` frames in this still: the
    /// viewport, the line's highlights together, or the scene's own camera,
    /// which is the viewport or its targets together. None when the camera
    /// isn't the scene's, or when nothing it names shows in the still.
    pub fn camera(&self, name: &str, scene: &Scene) -> Option<[f64; 4]> {
        let viewport = [
            0.0,
            0.0,
            f64::from(self.viewport[0]),
            f64::from(self.viewport[1]),
        ];
        match name {
            "viewport" => Some(viewport),
            "highlights" => union(self.highlights.values().flatten()),
            _ => {
                let camera = scene.cameras.get(name)?;
                if camera.viewport {
                    return Some(viewport);
                }
                union(
                    camera
                        .union
                        .iter()
                        .filter_map(|target| self.targets.get(target))
                        .flatten(),
                )
            }
        }
    }
}

/// The file of a web still's frames.
pub fn frames_path(project_dir: &Path, key: &str) -> PathBuf {
    keys::frames_path(project_dir, key)
}

/// Takes the captures in `plan` into the project folder. Each web scene
/// that has one is run from its setup, and its expectations are checked on
/// the way. Its new stills are moved in only when every expectation held
/// and the page raised no script error and opened no dialog; otherwise
/// none is kept and the problems are reported.
pub fn capture(project: &Project, dir: &Path, plan: &[Capture]) -> Result<Vec<Capture>, String> {
    capture_with(project, dir, plan, Limits::default())
}

/// [`capture`], with its patience given.
pub fn capture_with(
    project: &Project,
    dir: &Path,
    plan: &[Capture],
    limits: Limits,
) -> Result<Vec<Capture>, String> {
    let wanted: BTreeMap<&str, &Capture> = plan
        .iter()
        .filter_map(|c| match &c.source {
            Source::Line(id) => Some((id.as_str(), c)),
            Source::Page { .. } => None,
        })
        .collect();
    let folder = dir.join(keys::DIR);
    fs::create_dir_all(&folder).map_err(|e| format!("{}: {e}", folder.display()))?;
    // A killed capture can't clear the folder it kept its stills aside in.
    for entry in fs::read_dir(&folder).into_iter().flatten().flatten() {
        let age = entry.metadata().and_then(|m| m.modified()).ok();
        let stale = age.and_then(|at| at.elapsed().ok()) > Some(STALE);
        if stale && entry.file_name().to_string_lossy().starts_with(".scene-") {
            let _ = fs::remove_dir_all(entry.path());
        }
    }
    let mut taken = Vec::new();
    let mut readings = Readings::default();
    for (id, scene) in &project.scenes {
        let wants = project.lines.iter().any(|l| {
            l.scene_in(project) == Some(id.as_str()) && wanted.contains_key(l.id.as_str())
        });
        if !wants {
            continue;
        }
        // A picture is read, not browsed: one capture for all its lines.
        // Text is drawn, not browsed: one for each set of lines marked.
        if scene.kind == SceneKind::Image || scene.is_text() {
            let mut done: Vec<&str> = Vec::new();
            for line in project
                .lines
                .iter()
                .filter(|l| l.scene_in(project) == Some(id.as_str()))
            {
                let Some(capture) = wanted.get(line.id.as_str()) else {
                    continue;
                };
                if done.contains(&capture.key.as_str()) {
                    continue;
                }
                let taking = match scene.is_text() {
                    true => text(dir, scene, &line.highlight, &capture.key),
                    false => picture(dir, scene, &capture.key),
                };
                taking.map_err(|e| format!("scene {id}: {e}"))?;
                done.push(&capture.key);
                taken.push((*capture).clone());
            }
            continue;
        }
        // The scene's stills wait here until it has passed.
        let aside = tempfile::Builder::new()
            .prefix(".scene-")
            .tempdir_in(&folder)
            .map_err(|e| format!("{}: {e}", folder.display()))?;
        let mut page = Page::web(dir, scene, limits)?;
        let mut written = Vec::new();
        page.run_scene(
            project,
            dir,
            aside.path(),
            id,
            scene,
            &wanted,
            &mut written,
            &mut readings,
        )?;
        drop(page);
        let problems = onscreen::check_scene(project, id, &readings);
        if !problems.is_empty() {
            return Err(format!(
                "scene {id} doesn't show what the script says:\n  {}",
                problems.join("\n  ")
            ));
        }
        // The frames and the record of its files go in first: a still's
        // PNG is what marks it taken.
        for capture in &written {
            let mut files = vec![frames_path(dir, &capture.key)];
            if !capture.on_the_web() {
                files.push(keys::files_path(dir, &capture.key));
            }
            files.push(keys::path(dir, &capture.key));
            for to in files {
                let name = to.file_name().ok_or("a capture's file has a name")?;
                fs::rename(aside.path().join(name), &to)
                    .map_err(|e| format!("{}: {e}", to.display()))?;
            }
        }
        taken.extend(written);
    }
    if !readings.readings.is_empty() {
        let path = dir.join(onscreen::FILE);
        let json = serde_json::to_string_pretty(&readings).map_err(|e| e.to_string())?;
        write(&path, (json + "\n").as_bytes())?;
    }
    for capture in plan {
        if let Source::Page {
            scene,
            size,
            aspect,
        } = &capture.source
        {
            let spec = project
                .scenes
                .get(scene)
                .ok_or_else(|| format!("no scene {scene}"))?;
            let (png, files) = Page::render(dir, spec, *size, aspect, limits)
                .map_err(|e| format!("scene {scene} at {}x{}: {e}", size[0], size[1]))?;
            write(
                &keys::files_path(dir, &capture.key),
                keys::files_json(&files).as_bytes(),
            )?;
            write(&keys::path(dir, &capture.key), &png)?;
            taken.push(capture.clone());
        }
    }
    Ok(taken)
}

/// Takes an image scene's capture: its picture as a PNG (an SVG is drawn
/// at the scene's scale, with the fonts vidni is given and no others), the
/// boxes of its targets as its frames, and the record of its file.
fn picture(dir: &Path, scene: &Scene, key: &str) -> Result<(), String> {
    let name = scene.url.as_str();
    let file = dir.join(name);
    let bytes = fs::read(&file).map_err(|e| format!("{name}: {e}"))?;
    let sha = keys::sha256_hex(&bytes);
    let (png, viewport, scale) = if name.to_lowercase().ends_with(".svg") {
        let mut options = resvg::usvg::Options {
            resources_dir: file.parent().map(Path::to_path_buf),
            ..Default::default()
        };
        // Text is set in the fonts vidni pins, where they are found.
        let dirs = vidni_core::fonts::dirs();
        for font in vidni_core::fonts::all() {
            if let Some(data) = vidni_core::fonts::find(&font, &dirs)
                .ok()
                .and_then(|path| fs::read(path).ok())
            {
                options.fontdb_mut().load_font_data(data);
            }
        }
        let tree =
            resvg::usvg::Tree::from_data(&bytes, &options).map_err(|e| format!("{name}: {e}"))?;
        let scale = scene.scale.unwrap_or(1);
        let (w, h) = (tree.size().width().ceil(), tree.size().height().ceil());
        let mut pixmap = resvg::tiny_skia::Pixmap::new(w as u32 * scale, h as u32 * scale)
            .ok_or_else(|| format!("{name}: a picture needs a width and a height"))?;
        let grown = resvg::tiny_skia::Transform::from_scale(scale as f32, scale as f32);
        resvg::render(&tree, grown, &mut pixmap.as_mut());
        let png = pixmap.encode_png().map_err(|e| format!("{name}: {e}"))?;
        (png, [w as u32, h as u32], scale)
    } else {
        let image = crate::compose::image::Image::load_png(&file)?;
        (bytes, [image.width, image.height], 1)
    };
    let frames = Frames {
        viewport,
        scale,
        targets: scene
            .targets
            .iter()
            .filter_map(|(name, target)| Some((name.clone(), vec![target.rect?])))
            .collect(),
        highlights: BTreeMap::new(),
        browser: format!("vidni {}", vidni_core::VERSION),
    };
    let json = serde_json::to_string_pretty(&frames).map_err(|e| e.to_string())?;
    write(&frames_path(dir, key), (json + "\n").as_bytes())?;
    let files: Files = [(name.to_string(), Some(sha))].into();
    write(
        &keys::files_path(dir, key),
        keys::files_json(&files).as_bytes(),
    )?;
    write(&keys::path(dir, key), &png)
}

/// Takes a terminal or code scene's capture: the window of text as a
/// PNG, with `highlight`'s lines marked; the boxes of its targets as its
/// frames; and the record of the file it was read from.
fn text(dir: &Path, scene: &Scene, highlight: &[String], key: &str) -> Result<(), String> {
    let still = crate::textscene::still(dir, scene, highlight)?;
    let json = serde_json::to_string_pretty(&still.frames).map_err(|e| e.to_string())?;
    write(&frames_path(dir, key), (json + "\n").as_bytes())?;
    write(
        &keys::files_path(dir, key),
        keys::files_json(&still.files).as_bytes(),
    )?;
    // The picture goes in last: it is what marks the capture taken.
    let aside = keys::path(dir, key).with_extension("png.part");
    still.image.save_png(&aside)?;
    fs::rename(&aside, keys::path(dir, key)).map_err(|e| format!("{}: {e}", aside.display()))
}

fn write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let folder = path.parent().ok_or("a capture path has a folder")?;
    fs::create_dir_all(folder).map_err(|e| format!("{}: {e}", folder.display()))?;
    // Written aside and moved in, so an interrupted capture never looks taken.
    let part = path.with_extension("part");
    fs::write(&part, bytes)
        .and_then(|_| fs::rename(&part, path))
        .map_err(|e| format!("{}: {e}", path.display()))
}

/// The project folder as Chromium names it: with no symbolic link on the
/// way to it. The addresses of the project's files start with it.
fn root(dir: &Path) -> Result<PathBuf, String> {
    fs::canonicalize(dir).map_err(|e| format!("{}: {e}", dir.display()))
}

/// A scene's address: an http(s) URL as it is, or a file in the project
/// folder as a `file://` URL, with its query and fragment kept. The file
/// is named as the project names it, under the folder's real path, so
/// that what the page loads beside it is in the project folder too.
fn address(dir: &Path, url: &str) -> Result<String, String> {
    if url.starts_with("http://") || url.starts_with("https://") {
        return Ok(url.to_string());
    }
    let (rest, fragment) = url
        .split_once('#')
        .map_or((url, None), |(r, f)| (r, Some(f)));
    let (file, query) = rest
        .split_once('?')
        .map_or((rest, None), |(f, q)| (f, Some(q)));
    let path = root(dir)?.join(file);
    fs::metadata(&path).map_err(|e| format!("{file}: {e}"))?;
    let mut out = String::from("file://");
    for byte in path.as_os_str().as_encoded_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' => {
                out.push(char::from(*byte));
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    for (mark, part) in [('?', query), ('#', fragment)] {
        if let Some(part) = part {
            out.push(mark);
            out.push_str(part);
        }
    }
    Ok(out)
}

/// The file a `file://` address names, without its query and fragment;
/// none for any other address.
fn file_of(url: &str) -> Option<PathBuf> {
    let path = url.strip_prefix("file://")?;
    let path = path.split(['?', '#']).next().unwrap_or_default();
    let mut bytes = Vec::with_capacity(path.len());
    let mut rest = path.as_bytes();
    while let Some((&byte, after)) = rest.split_first() {
        // %41 is the byte 0x41; a % that starts no such pair is itself.
        let escaped = (byte == b'%')
            .then(|| after.get(..2))
            .flatten()
            .and_then(|hex| std::str::from_utf8(hex).ok())
            .and_then(|hex| u8::from_str_radix(hex, 16).ok());
        match escaped {
            Some(decoded) => {
                bytes.push(decoded);
                rest = &after[2..];
            }
            None => {
                bytes.push(byte);
                rest = after;
            }
        }
    }
    String::from_utf8(bytes).ok().map(PathBuf::from)
}

/// The project's files that a page of the project's loads.
struct Local {
    /// The project folder ([`root`]).
    root: PathBuf,
    /// Each file the page has asked for, with its SHA-256 when it was
    /// first asked for.
    files: Files,
    /// The files the page has asked for that aren't in the project folder.
    outside: Vec<String>,
}

impl Local {
    fn new(dir: &Path) -> Result<Self, String> {
        Ok(Self {
            root: root(dir)?,
            files: Files::new(),
            outside: Vec::new(),
        })
    }

    /// Forgets what a document that is gone asked for.
    fn clear(&mut self) {
        self.files.clear();
        self.outside.clear();
    }

    /// Notes that the page asked for `url`, when that is a file.
    fn asked(&mut self, url: &str) {
        let Some(path) = file_of(url) else {
            return;
        };
        let Ok(inside) = path.strip_prefix(&self.root) else {
            self.outside.push(path.display().to_string());
            return;
        };
        let parts: Vec<String> = inside
            .components()
            .filter_map(|part| match part {
                Component::Normal(name) => Some(name.to_string_lossy().into_owned()),
                _ => None,
            })
            .collect();
        let name = parts.join("/");
        if !self.files.contains_key(&name) {
            let sha = keys::file_sha256(&self.root, &name);
            self.files.insert(name, sha);
        }
    }

    /// The record of the files the page has loaded. A file that is no
    /// longer what it was when the page asked for it may not be what the
    /// page shows, so no record is made of it.
    fn record(&self) -> Result<Files, String> {
        for (name, then) in &self.files {
            if keys::file_sha256(&self.root, name) != *then {
                return Err(format!(
                    "{name} changed while the page was being captured; capture it again"
                ));
            }
        }
        Ok(self.files.clone())
    }
}

/// Whether an evaluation failed because the page went to another document.
fn lost_its_document(error: &str) -> bool {
    error.contains("Cannot find context") || error.contains("context was destroyed")
}

/// One page in its own Chromium.
struct Page {
    browser: Browser,
    session: String,
    viewport: [u32; 2],
    scale: u32,
    limits: Limits,
    /// vidni's isolated world in the page's document.
    world: Option<i64>,
    /// The page's requests in flight, each with when it was sent.
    loading: BTreeMap<String, Instant>,
    /// The address each request asked for, by request.
    asked: BTreeMap<String, String>,
    /// What the page asked the network for and didn't get, each with why.
    unreached: Vec<String>,
    /// Whether a load that fails is a fault: for a page that is a file in
    /// the project, whose parts should all be there, and for any page when
    /// there is no network.
    whole: bool,
    /// The project's files the page loads, when it is one of them.
    local: Option<Local>,
}

impl Page {
    /// A page set up as Playwright sets one up, at `viewport` CSS pixels
    /// and `scale` device pixels per CSS pixel. Motion is always reduced.
    fn open(
        viewport: [u32; 2],
        scale: u32,
        color_scheme: &str,
        limits: Limits,
    ) -> Result<Self, String> {
        let mut browser = Browser::launch(&chromium(), &switches())?;
        let target = browser.call(None, "Target.createTarget", json!({"url": "about:blank"}))?;
        let window = browser.call(
            None,
            "Browser.getWindowForTarget",
            json!({"targetId": target["targetId"]}),
        )?;
        browser.call(
            None,
            "Browser.setWindowBounds",
            json!({
                "windowId": window["windowId"],
                "bounds": {"width": viewport[0], "height": viewport[1]},
            }),
        )?;
        let attached = browser.call(
            None,
            "Target.attachToTarget",
            json!({"targetId": target["targetId"], "flatten": true}),
        )?;
        let session = attached["sessionId"]
            .as_str()
            .ok_or("Chromium gave no session")?
            .to_string();
        let mut page = Self {
            browser,
            session,
            viewport,
            scale,
            limits,
            world: None,
            loading: BTreeMap::new(),
            asked: BTreeMap::new(),
            unreached: Vec::new(),
            whole: limits.offline,
            local: None,
        };
        page.call("Page.enable", json!({}))?;
        page.call("Inspector.enable", json!({}))?;
        page.call("Runtime.enable", json!({}))?;
        page.call("Network.enable", json!({}))?;
        page.call(
            "Emulation.setFocusEmulationEnabled",
            json!({"enabled": true}),
        )?;
        page.call(
            "Emulation.setDeviceMetricsOverride",
            json!({
                "mobile": false,
                "width": viewport[0],
                "height": viewport[1],
                "screenWidth": viewport[0],
                "screenHeight": viewport[1],
                "deviceScaleFactor": scale,
                "screenOrientation": {"angle": 0, "type": "landscapePrimary"},
            }),
        )?;
        page.call(
            "Emulation.setEmulatedMedia",
            json!({"media": "", "features": [
                {"name": "prefers-color-scheme", "value": color_scheme},
                {"name": "prefers-reduced-motion", "value": "reduce"},
                {"name": "forced-colors", "value": "none"},
                {"name": "prefers-contrast", "value": "no-preference"},
            ]}),
        )?;
        page.call(
            "Page.addScriptToEvaluateOnNewDocument",
            json!({"source": boot(), "worldName": WORLD}),
        )?;
        // Playwright's font families for headless Linux, on every host.
        page.call(
            "Page.setFontFamilies",
            json!({"fontFamilies": {
                "standard": "Times New Roman",
                "fixed": "Monospace",
                "serif": "Times New Roman",
                "sansSerif": "Arial",
                "cursive": "Comic Sans MS",
                "fantasy": "Impact",
            }}),
        )?;
        Ok(page)
    }

    /// A page for a web scene of the project in `dir`.
    fn web(dir: &Path, scene: &Scene, limits: Limits) -> Result<Self, String> {
        let scheme = match scene.color_scheme {
            Some(ColorScheme::Dark) => "dark",
            Some(ColorScheme::Light) | None => "light",
        };
        let mut page = Self::open(
            scene.viewport.unwrap_or([1280, 720]),
            scene.scale.unwrap_or(1),
            scheme,
            limits,
        )?;
        // A page on the web may lose a load that doesn't matter to it; a
        // file of the project's should have all its parts.
        if !scene.on_the_web() {
            page.whole = true;
            page.local = Some(Local::new(dir)?);
        }
        Ok(page)
    }

    /// Renders a page scene at `size` for a cut of `aspect`, which
    /// `{aspect}` in its address becomes. Returns the picture, and the
    /// record of the project's files the page loaded.
    fn render(
        dir: &Path,
        scene: &Scene,
        size: [u32; 2],
        aspect: &str,
        limits: Limits,
    ) -> Result<(Vec<u8>, Files), String> {
        let mut page = Self::open(size, 1, "light", limits)?;
        // A page scene is a file of the project's.
        page.whole = true;
        page.local = Some(Local::new(dir)?);
        page.goto(&address(dir, &scene.url.replace("{aspect}", aspect))?)?;
        if scene.transparent == Some(true) {
            page.call(
                "Emulation.setDefaultBackgroundColorOverride",
                json!({"color": {"r": 0, "g": 0, "b": 0, "a": 0}}),
            )?;
        }
        page.eval(BLUR)?;
        let png = page.settle()?;
        page.faults()?;
        let files = page.record()?;
        Ok((png, files))
    }

    /// The record of the project's files the page has loaded; empty for a
    /// page on the web.
    fn record(&self) -> Result<Files, String> {
        self.local.as_ref().map_or(Ok(Files::new()), Local::record)
    }

    /// Forgets the requests of a document that is gone, before `url` loads.
    /// The file `url` names is noted now, before Chromium reads it.
    fn forget_requests(&mut self, url: &str) {
        self.browser.forget_events();
        self.loading.clear();
        self.asked.clear();
        self.unreached.clear();
        if let Some(local) = &mut self.local {
            local.clear();
            local.asked(url);
        }
    }

    fn call(&mut self, method: &str, params: Value) -> Result<Value, String> {
        self.browser.call(Some(&self.session), method, params)
    }

    /// Evaluates `expression` in vidni's world (once installed) and returns
    /// its value, awaiting a promise.
    fn eval_once(&mut self, expression: &str) -> Result<Value, String> {
        let mut params =
            json!({"expression": expression, "returnByValue": true, "awaitPromise": true});
        if let Some(world) = self.world {
            params["contextId"] = json!(world);
        }
        let result = self.call("Runtime.evaluate", params)?;
        if let Some(e) = result.get("exceptionDetails") {
            let message = e["exception"]["description"]
                .as_str()
                .or(e["text"].as_str())
                .unwrap_or("an exception");
            return Err(message.lines().next().unwrap_or(message).to_string());
        }
        Ok(result["result"]["value"].clone())
    }

    /// [`Self::eval_once`]; when the page has gone to another document
    /// meanwhile, waits for it to load, installs the runtime there, and
    /// evaluates again.
    fn eval(&mut self, expression: &str) -> Result<Value, String> {
        match self.eval_once(expression) {
            Err(e) if lost_its_document(&e) => {
                self.browser
                    .event(&self.session, "Page.loadEventFired", self.limits.patience)?;
                self.install()?;
                self.eval_once(expression)
            }
            other => other,
        }
    }

    /// Calls the runtime: `window.__vidni.<call>`.
    fn runtime(&mut self, call: &str) -> Result<Value, String> {
        self.eval(&format!("window.__vidni.{call}"))
    }

    /// Loads `url`: waits for its load event, then finds the runtime.
    ///
    /// At a scale other than 1 the page is loaded twice. A browser's first
    /// load at an emulated scale comes out in one of two renderings, launch
    /// by launch; a second load in the same browser comes out in one. The
    /// first load is kept inert: none of the page's scripts run, and the
    /// cookies and storage it leaves are cleared, so that the second load,
    /// the one captured, is a first visit too (DECISIONS.md, T6).
    fn goto(&mut self, url: &str) -> Result<(), String> {
        let shown = |e: String| format!("{url}: {e}");
        self.forget_requests(url);
        self.world = None;
        if self.scale != 1 {
            // The first load runs none of the page's scripts.
            self.call(
                "Emulation.setScriptExecutionDisabled",
                json!({"value": true}),
            )?;
        }
        let navigated = self.call("Page.navigate", json!({"url": url}))?;
        if let Some(error) = navigated["errorText"].as_str() {
            return Err(format!("{url}: {error}"));
        }
        self.browser
            .event(&self.session, "Page.loadEventFired", self.limits.patience)
            .map_err(shown)?;
        if self.scale != 1 {
            // What the first load left behind is cleared, so that the second
            // sees a first visit too.
            let scale = self.scale;
            let deadline = Instant::now() + self.limits.patience;
            while self.eval_once("window.devicePixelRatio").map_err(shown)? != json!(scale) {
                if Instant::now() > deadline {
                    return Err(format!("{url}: the page never took the scale {scale}"));
                }
                self.browser.idle(Duration::from_millis(20))?;
            }
            self.screenshot().map_err(shown)?;
            let origin = self.eval_once("window.location.origin").map_err(shown)?;
            self.call("Network.clearBrowserCookies", json!({}))?;
            if let Some(origin) = origin.as_str().filter(|o| *o != "null") {
                self.call(
                    "Storage.clearDataForOrigin",
                    json!({"origin": origin, "storageTypes": "all"}),
                )?;
            }
            self.call(
                "Emulation.setScriptExecutionDisabled",
                json!({"value": false}),
            )?;
            self.forget_requests(url);
            self.call("Page.reload", json!({}))?;
            self.browser
                .event(&self.session, "Page.loadEventFired", self.limits.patience)
                .map_err(shown)?;
        }
        self.install()
    }

    /// Finds vidni's world in the page's document as it is now. The runtime
    /// is already there, with its style and highlight layer: [`boot`] runs
    /// at every document's start. A document that began before that was
    /// arranged gets them now.
    fn install(&mut self) -> Result<(), String> {
        // The navigation that brought this document is the one handled here.
        self.browser.take(&self.session, "Page.frameNavigated");
        let tree = self.call("Page.getFrameTree", json!({}))?;
        let world = self.call(
            "Page.createIsolatedWorld",
            json!({"frameId": tree["frameTree"]["frame"]["id"], "worldName": WORLD}),
        )?;
        self.world = Some(
            world["executionContextId"]
                .as_i64()
                .ok_or("Chromium made no world")?,
        );
        if self.eval_once("typeof window.__vidni === 'object'")? != json!(true) {
            self.eval_once(&boot())?;
        }
        Ok(())
    }

    /// When the page has gone to another document, waits for it to load
    /// and installs the runtime there.
    fn follow(&mut self) -> Result<(), String> {
        let moved = self
            .browser
            .take(&self.session, "Page.frameNavigated")
            .iter()
            .any(|e| e["frame"]["parentId"].is_null());
        if moved {
            self.browser
                .event(&self.session, "Page.loadEventFired", self.limits.patience)?;
            self.install()?;
        }
        Ok(())
    }

    /// Fails when the page has raised a script error or opened a dialog
    /// since last asked.
    fn faults(&mut self) -> Result<(), String> {
        let mut faults: Vec<String> = self
            .browser
            .take(&self.session, "Runtime.exceptionThrown")
            .iter()
            .map(|e| {
                let details = &e["exceptionDetails"];
                let text = details["exception"]["description"]
                    .as_str()
                    .or(details["text"].as_str())
                    .unwrap_or("an exception");
                format!("the page threw: {}", text.lines().next().unwrap_or(text))
            })
            .collect();
        for dialog in self
            .browser
            .take(&self.session, "Page.javascriptDialogOpening")
        {
            // Leaving a page that asks first is allowed, as a person would.
            if dialog["type"] == "beforeunload" {
                continue;
            }
            faults.push(format!(
                "the page opened a dialog ({}), which was dismissed: {}",
                dialog["type"].as_str().unwrap_or("JavaScript"),
                dialog["message"].as_str().unwrap_or_default()
            ));
        }
        if self.whole
            && let Some(first) = self.unreached.first()
        {
            faults.push(format!(
                "the page asked the network for {} that didn't load (the first: {first})",
                match self.unreached.len() {
                    1 => "a part".to_string(),
                    n => format!("{n} parts"),
                }
            ));
            self.unreached.clear();
        }
        // What a page of the project's loads from elsewhere on this
        // machine isn't the project's, and no record could vouch for it.
        if let Some(local) = &mut self.local
            && let Some(first) = local.outside.first()
        {
            faults.push(format!(
                "the page loaded {} from outside the project folder (the first: {first})",
                match local.outside.len() {
                    1 => "a file".to_string(),
                    n => format!("{n} files"),
                }
            ));
            local.outside.clear();
        }
        match faults.is_empty() {
            true => Ok(()),
            false => Err(faults.join("; ")),
        }
    }

    /// A PNG of the viewport, as Playwright takes one.
    fn screenshot(&mut self) -> Result<Vec<u8>, String> {
        let metrics = self.call("Page.getLayoutMetrics", json!({}))?;
        let view = &metrics["cssVisualViewport"];
        let clip = json!({
            "x": view["pageX"].as_f64().unwrap_or(0.0),
            "y": view["pageY"].as_f64().unwrap_or(0.0),
            "width": self.viewport[0],
            "height": self.viewport[1],
            "scale": 1,
        });
        let shot = self.call(
            "Page.captureScreenshot",
            json!({"format": "png", "clip": clip, "captureBeyondViewport": false}),
        )?;
        base64::engine::general_purpose::STANDARD
            .decode(shot["data"].as_str().ok_or("Chromium sent no image")?)
            .map_err(|e| format!("a screenshot: {e}"))
    }

    /// Whether the page's network has been busy since last asked: a request
    /// was sent or finished, or one is still in flight (and hasn't been for
    /// longer than [`NETWORK`]).
    fn network_busy(&mut self) -> bool {
        let mut active = false;
        for e in self
            .browser
            .take(&self.session, "Network.requestWillBeSent")
        {
            let id = e["requestId"].as_str().unwrap_or_default().to_string();
            let url = e["request"]["url"].as_str().unwrap_or_default();
            if let Some(local) = &mut self.local {
                local.asked(url);
            }
            self.asked.insert(id.clone(), url.to_string());
            self.loading.insert(id, Instant::now());
            active = true;
        }
        for method in ["Network.loadingFinished", "Network.loadingFailed"] {
            for e in self.browser.take(&self.session, method) {
                let id = e["requestId"].as_str().unwrap_or_default();
                self.loading.remove(id);
                active = true;
                // A load the page gave up itself, as on leaving, is no loss.
                let lost = method == "Network.loadingFailed" && e["canceled"] != true;
                if let (true, Some(url)) = (lost, self.asked.get(id))
                    && ["http://", "https://", "ws://", "wss://"]
                        .iter()
                        .any(|scheme| url.starts_with(scheme))
                {
                    let why = e["errorText"].as_str().unwrap_or("it failed to load");
                    self.unreached.push(format!("{url} ({why})"));
                }
            }
        }
        active || self.loading.values().any(|sent| sent.elapsed() < NETWORK)
    }

    /// Lets the page settle, and returns its picture then: its animations
    /// are finished (and endless ones stopped, as Playwright stops them for
    /// a screenshot), its fonts are loaded, and two screenshots in a row,
    /// [`SETTLE`] apart, are identical with the network quiet between them.
    /// A page that keeps its network busy has settled once [`STEADY`]
    /// screenshots in a row are identical.
    fn settle(&mut self) -> Result<Vec<u8>, String> {
        let deadline = Instant::now() + self.limits.patience;
        let mut last: Option<Vec<u8>> = None;
        let mut same = 0;
        loop {
            self.follow()?;
            self.runtime("calm()")?;
            self.eval("document.fonts.ready.then(() => true)")?;
            let shot = self.screenshot()?;
            let busy = self.network_busy();
            same = if last.as_ref() == Some(&shot) {
                same + 1
            } else {
                0
            };
            if (same >= 1 && !busy) || same >= STEADY {
                return Ok(shot);
            }
            if Instant::now() > deadline {
                return Err(format!(
                    "the page was still changing after {} s",
                    self.limits.patience.as_secs()
                ));
            }
            last = Some(shot);
            self.browser.idle(SETTLE)?;
        }
    }

    /// Clicks at a point, with the mouse, as a person would: the pointer
    /// stays there, so hover styles show as they do for a person.
    fn click(&mut self, point: &Value) -> Result<(), String> {
        let (x, y) = (point[0].as_f64(), point[1].as_f64());
        let (Some(x), Some(y)) = (x, y) else {
            return Err(format!("no point to click: {point}"));
        };
        for (kind, buttons) in [("mouseMoved", 0), ("mousePressed", 1), ("mouseReleased", 0)] {
            let mut event = json!({"type": kind, "x": x, "y": y, "buttons": buttons});
            if kind != "mouseMoved" {
                event["button"] = json!("left");
                event["clickCount"] = json!(1);
            }
            self.call("Input.dispatchMouseEvent", event)?;
        }
        Ok(())
    }

    /// Asks the runtime where to click until it says: a target covered by
    /// something else, say while that is going away, is asked for again.
    fn point(&mut self, call: &str) -> Result<Value, String> {
        let deadline = Instant::now() + self.limits.patience;
        loop {
            match self.runtime(call) {
                Ok(point) => return Ok(point),
                Err(e) if e.contains("is covered") && Instant::now() < deadline => {
                    self.browser.idle(Duration::from_millis(100))?;
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Waits until a target matches something on screen.
    fn wait_for(&mut self, name: &str, spec: &str) -> Result<(), String> {
        let deadline = Instant::now() + self.limits.patience;
        while self.runtime(&format!("present({spec})"))? != json!(true) {
            if Instant::now() > deadline {
                return Err(format!("target {name} never showed"));
            }
            self.browser.idle(Duration::from_millis(100))?;
        }
        Ok(())
    }

    /// Does what one action says, once its target shows.
    fn perform(&mut self, scene: &Scene, action: &Action) -> Result<(), String> {
        let spec = |name: &str| -> Result<String, String> {
            let target = scene
                .targets
                .get(name)
                .ok_or_else(|| format!("no target {name}"))?;
            serde_json::to_string(target).map_err(|e| e.to_string())
        };
        let (verb, name) = *action.verbs().first().ok_or("an action without a verb")?;
        let (target, quoted) = (spec(name)?, json!(name));
        self.wait_for(name, &target)?;
        match verb {
            "click" => {
                let point = self.point(&format!("point({quoted}, {target})"))?;
                self.click(&point)?;
            }
            "select" => {
                let value = json!(action.value.as_deref().unwrap_or_default());
                self.runtime(&format!("select({quoted}, {target}, {value})"))?;
            }
            "expand" | "collapse" => {
                let control = json!(action.control);
                let open = verb == "expand";
                let point =
                    self.point(&format!("toggle({quoted}, {target}, {control}, {open})"))?;
                if !point.is_null() {
                    self.click(&point)?;
                }
            }
            "scroll" => {
                let top = json!(action.top.unwrap_or_default());
                self.runtime(&format!("scroll({quoted}, {target}, {top})"))?;
            }
            "scroll_to" => {
                let within_name = action.within.as_deref().unwrap_or_default();
                let within = spec(within_name)?;
                let offset = json!(action.offset.unwrap_or_default());
                self.runtime(&format!(
                    "scrollTo({quoted}, {target}, {}, {within}, {offset})",
                    json!(within_name)
                ))?;
            }
            "wait" => {}
            other => return Err(format!("unknown action {other}")),
        }
        Ok(())
    }

    /// Runs `actions` in order, letting the page settle after each. After
    /// the last one nothing is left focused.
    fn step(&mut self, scene: &Scene, actions: &[Action]) -> Result<(), String> {
        for (i, action) in actions.iter().enumerate() {
            self.perform(scene, action)?;
            self.follow()?;
            if i + 1 == actions.len() {
                self.eval(BLUR)?;
            }
            self.settle()?;
            self.faults()?;
        }
        Ok(())
    }

    /// Runs a web scene from its setup through its lines, writing the wanted
    /// stills and their frames into `aside` and reading its expectations.
    #[allow(clippy::too_many_arguments)]
    fn run_scene(
        &mut self,
        project: &Project,
        dir: &Path,
        aside: &Path,
        id: &str,
        scene: &Scene,
        wanted: &BTreeMap<&str, &Capture>,
        written: &mut Vec<Capture>,
        readings: &mut Readings,
    ) -> Result<(), String> {
        let shown = |at: &str| {
            let at = at.to_string();
            move |e: String| format!("scene {id}, at {at}: {e}")
        };
        let browser = self.browser.call(None, "Browser.getVersion", json!({}))?["product"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        let start = (|| -> Result<(), String> {
            self.goto(&address(dir, &scene.url)?)?;
            self.eval(BLUR)?;
            self.settle()?;
            self.faults()?;
            self.step(scene, &scene.setup)?;
            self.read(project, id, scene, "setup", readings)
        })();
        start.map_err(shown("setup"))?;
        for line in project
            .lines
            .iter()
            .filter(|l| l.scene_in(project) == Some(id))
        {
            let step = (|| -> Result<(), String> {
                self.step(scene, &line.actions)?;
                self.read(project, id, scene, &line.id, readings)?;
                let Some(capture) = wanted.get(line.id.as_str()) else {
                    return Ok(());
                };
                let frames = self.still(scene, &line.highlight, &browser)?;
                let png = self.settle()?;
                self.faults()?;
                let json = serde_json::to_string_pretty(&frames).map_err(|e| e.to_string())?;
                write(
                    &aside.join(format!("{}.json", capture.key)),
                    (json + "\n").as_bytes(),
                )?;
                if !capture.on_the_web() {
                    write(
                        &aside.join(format!("{}.files.json", capture.key)),
                        keys::files_json(&self.record()?).as_bytes(),
                    )?;
                }
                write(&aside.join(format!("{}.png", capture.key)), &png)?;
                written.push((*capture).clone());
                Ok(())
            })();
            step.map_err(shown(&format!("line {}", line.id)))?;
        }
        Ok(())
    }

    /// Draws a line's highlights, and measures them and the scene's targets.
    fn still(
        &mut self,
        scene: &Scene,
        highlight: &[String],
        browser: &str,
    ) -> Result<Frames, String> {
        let target = |name: &str| -> Result<Value, String> {
            let target = scene
                .targets
                .get(name)
                .ok_or_else(|| format!("no target {name}"))?;
            serde_json::to_value(target).map_err(|e| e.to_string())
        };
        let pairs: Vec<Value> = highlight
            .iter()
            .map(|name| Ok(json!([name, target(name)?])))
            .collect::<Result<_, String>>()?;
        let drawn = self.runtime(&format!("highlight({})", Value::Array(pairs)))?;
        // Every target is measured, not only the ones today's cameras name.
        let all: Vec<Value> = scene
            .targets
            .keys()
            .map(|name| Ok(json!([name, target(name)?])))
            .collect::<Result<_, String>>()?;
        let measured = self.runtime(&format!("targets({})", Value::Array(all)))?;
        Ok(Frames {
            viewport: self.viewport,
            scale: self.scale,
            targets: serde_json::from_value(measured).map_err(|e| e.to_string())?,
            highlights: serde_json::from_value(drawn).map_err(|e| e.to_string())?,
            browser: browser.to_string(),
        })
    }

    /// Reads the targets of the scene's expectations at `at`. Each is
    /// waited for first: the page may take a moment to show what the script
    /// says it shows. One that never holds is read as it is, for the
    /// report. The page settles again after a wait.
    fn read(
        &mut self,
        project: &Project,
        id: &str,
        scene: &Scene,
        at: &str,
        readings: &mut Readings,
    ) -> Result<(), String> {
        let mut waited = false;
        for e in project
            .expectations
            .iter()
            .filter(|e| e.scene == id && e.at == at)
        {
            let target = scene
                .targets
                .get(&e.target)
                .ok_or_else(|| format!("no target {}", e.target))?;
            let spec = serde_json::to_string(target).map_err(|e| e.to_string())?;
            let deadline = Instant::now() + self.limits.patience;
            let texts = loop {
                let texts: Vec<String> =
                    serde_json::from_value(self.runtime(&format!("read({spec})"))?)
                        .map_err(|e| e.to_string())?;
                if onscreen::holds(e, &texts) || Instant::now() > deadline {
                    break texts;
                }
                waited = true;
                self.browser.idle(Duration::from_millis(100))?;
            };
            readings.readings.push(Reading {
                scene: id.to_string(),
                at: at.to_string(),
                target: e.target.clone(),
                texts,
            });
        }
        if waited {
            self.settle()?;
            self.faults()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_camera_is_framed_from_the_targets_as_the_scene_names_them_now() {
        let project = vidni_core::validate::check(
            r##"schema = 1

[meta]
title = "Cameras"
default_scene = "page"

[[cuts]]
id = "long"
aspect = "16:9"

[sections.main]

[[lines]]
id = "main-1"
section = "main"
show = "A line."

[scenes.page]
kind = "web"
url = "https://example.com"

[scenes.page.targets]
tab-a = { css = "#a" }
tab-b = { css = "#b" }
gone = { css = "#gone" }

[scenes.page.cameras]
tabs = { union = ["tab-a", "tab-b"] }
first = { union = ["tab-a"] }
all = { viewport = true }
nothing = { union = ["gone"] }
"##,
            None,
        )
        .unwrap_or_else(|p| panic!("{p:?}"));
        let scene = project.scenes["page"].clone();
        let frames = Frames {
            viewport: [640, 360],
            scale: 2,
            targets: [
                ("tab-a".to_string(), vec![[10.0, 40.0, 80.0, 30.0]]),
                (
                    "tab-b".to_string(),
                    vec![[100.0, 40.5, 80.0, 30.0], [100.0, 90.0, 20.25, 10.0]],
                ),
                ("gone".to_string(), Vec::new()),
            ]
            .into(),
            highlights: [("tab-b".to_string(), vec![[100.0, 40.5, 80.0, 30.0]])].into(),
            browser: "a stand-in".into(),
        };
        assert_eq!(
            frames.camera("tabs", &scene),
            Some([10.0, 40.0, 170.0, 60.0])
        );
        assert_eq!(
            frames.camera("first", &scene),
            Some([10.0, 40.0, 80.0, 30.0])
        );
        let viewport = Some([0.0, 0.0, 640.0, 360.0]);
        assert_eq!(frames.camera("all", &scene), viewport);
        assert_eq!(frames.camera("viewport", &scene), viewport);
        assert_eq!(
            frames.camera("highlights", &scene),
            Some([100.0, 40.5, 80.0, 30.0])
        );
        // A camera whose targets show nothing frames nothing, and so does
        // one the scene doesn't have.
        assert_eq!(frames.camera("nothing", &scene), None);
        assert_eq!(frames.camera("elsewhere", &scene), None);
        // The camera is the scene's as it is now: edited after the capture,
        // it frames its new targets, with no new capture.
        let mut edited = scene.clone();
        edited.cameras.get_mut("tabs").unwrap().union = vec!["tab-b".into()];
        assert_eq!(
            frames.camera("tabs", &edited),
            Some([100.0, 40.5, 80.0, 59.5])
        );
        // Without highlights there is no highlights camera.
        let plain = Frames {
            highlights: BTreeMap::new(),
            ..frames
        };
        assert_eq!(plain.camera("highlights", &scene), None);
    }

    #[test]
    fn addresses_keep_urls_and_turn_project_files_into_file_urls() {
        let dir = tempfile::tempdir().unwrap();
        let odd = dir.path().join("videos #2 %41");
        fs::create_dir_all(odd.join("cards")).unwrap();
        fs::write(odd.join("cards/card.html"), "").unwrap();
        assert_eq!(
            address(&odd, "https://example.com/x?y=1").unwrap(),
            "https://example.com/x?y=1"
        );
        let file = address(&odd, "cards/card.html?v=intro&r=16x9#top").unwrap();
        assert!(file.starts_with("file:///"), "{file}");
        assert!(
            file.ends_with("/videos%20%232%20%2541/cards/card.html?v=intro&r=16x9#top"),
            "{file}"
        );
        assert!(address(&odd, "cards/missing.html").is_err());
    }

    #[test]
    fn a_file_address_names_its_file() {
        let named = |url: &str| file_of(url).map(|path| path.display().to_string());
        assert_eq!(
            named("file:///a/b%20c/d.html?x=1#top").as_deref(),
            Some("/a/b c/d.html")
        );
        assert_eq!(
            named("file:///a/100%25/%E2%9C%93.css").as_deref(),
            Some("/a/100%/✓.css")
        );
        // A % that starts no escape is itself.
        assert_eq!(
            named("file:///a/50%/b%zz%4").as_deref(),
            Some("/a/50%/b%zz%4")
        );
        for other in ["https://example.com/a.css", "data:text/css,", "about:blank"] {
            assert_eq!(named(other), None, "{other}");
        }
        // What `address` writes is read back as the file it was given.
        let dir = tempfile::tempdir().unwrap();
        let odd = dir.path().join("videos #2 %41");
        fs::create_dir_all(odd.join("cards")).unwrap();
        fs::write(odd.join("cards/card é.html"), "").unwrap();
        let url = address(&odd, "cards/card é.html?v=intro#top").unwrap();
        assert_eq!(
            file_of(&url),
            Some(root(&odd).unwrap().join("cards/card é.html"))
        );
    }

    #[test]
    fn a_project_pages_files_are_noted_once_and_must_stay_as_they_were() {
        let dir = tempfile::tempdir().unwrap();
        let dir = dir.path();
        fs::create_dir_all(dir.join("site/css")).unwrap();
        fs::write(dir.join("site/index.html"), "<!doctype html>").unwrap();
        fs::write(dir.join("site/css/style.css"), "body {}").unwrap();
        let mut local = Local::new(dir).unwrap();
        let url = |name: &str| format!("file://{}/{name}", local_root(dir));
        fn local_root(dir: &Path) -> String {
            root(dir).unwrap().display().to_string()
        }
        local.asked(&address(dir, "./site/index.html?x=1#top").unwrap());
        local.asked(&url("site/index.html?x=1"));
        local.asked(&url("site/css/style.css"));
        local.asked(&url("site/img/gone.png"));
        // What isn't a file is the network's business, not the record's.
        local.asked("https://example.com/x.css");
        local.asked("data:image/png;base64,AAAA");
        let sha = |name: &str| keys::file_sha256(dir, name);
        let noted: Files = [
            ("site/css/style.css".to_string(), sha("site/css/style.css")),
            ("site/img/gone.png".to_string(), None),
            ("site/index.html".to_string(), sha("site/index.html")),
        ]
        .into();
        assert!(noted["site/index.html"].is_some());
        assert_eq!(local.record().unwrap(), noted);
        assert_eq!(local.outside, Vec::<String>::new());

        // A file that changes while the page is being captured may not be
        // what the page shows, so no record is made.
        fs::write(dir.join("site/css/style.css"), "body { color: red }").unwrap();
        local.asked(&url("site/css/style.css"));
        assert_eq!(
            local.record().unwrap_err(),
            "site/css/style.css changed while the page was being captured; capture it again"
        );
        // So does one that turns up after the page asked for it in vain.
        fs::write(dir.join("site/css/style.css"), "body {}").unwrap();
        fs::create_dir_all(dir.join("site/img")).unwrap();
        fs::write(dir.join("site/img/gone.png"), "png").unwrap();
        assert!(
            local
                .record()
                .unwrap_err()
                .starts_with("site/img/gone.png changed")
        );

        // A file from elsewhere on the machine isn't the project's.
        let elsewhere = tempfile::tempdir().unwrap();
        let outside = root(elsewhere.path()).unwrap().join("shared.css");
        local.asked(&format!("file://{}", outside.display()));
        assert_eq!(local.outside, [outside.display().to_string()]);
        // A new document starts with nothing noted.
        local.clear();
        assert_eq!(local.record().unwrap(), Files::new());
        assert!(local.outside.is_empty());
    }

    #[test]
    fn the_runtime_is_installed_as_window_vidni() {
        assert!(RUNTIME.contains("window.__vidni = "));
        for call in [
            "calm(",
            "present(",
            "point(",
            "toggle(",
            "select(",
            "scroll(",
            "scrollTo(",
            "highlight(",
            "targets(",
            "read(",
        ] {
            assert!(RUNTIME.contains(call), "{call}");
        }
    }

    #[test]
    fn the_sandbox_is_on_unless_the_host_turns_it_off() {
        assert!(!SWITCHES.contains(&"--no-sandbox"));
    }
}
