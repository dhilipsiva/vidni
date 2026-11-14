//! The capture runtime on a local page with fixed geometry (P3-VIDNI-04).
//! These need Chromium, so they are ignored by `cargo test`; `just
//! test-capture` runs them, and so does CI.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use vidni_core::capture::{self, Source};
use vidni_core::project::Project;
use vidni_media::capture::{Frames, Limits, capture_with, frames_path};
use vidni_media::compose::image::Image;

const PROJECT: &str = r##"
schema = 1

[meta]
title = "Capture test"
default_scene = "page"

[sections.main]

[[lines]]
id = "a"
section = "main"
show = "Click Alpha."
do = [{ click = "tab-a" }]
highlight = ["tab-a"]

[[lines]]
id = "b"
section = "main"
show = "Choose two."
do = [{ select = "choice", value = "2" }]
highlight = ["choice"]
overlay = "badge"

[[lines]]
id = "c"
section = "main"
show = "Scroll the panel."
do = [{ scroll = "panel", top = 80 }]
highlight = ["row-3"]

[[lines]]
id = "d"
section = "main"
show = "Open the detail."
do = [{ expand = "detail", control = ".toggle" }]
highlight = ["code-line-1"]

[[lines]]
id = "e"
section = "main"
show = "Close it and scroll on."
do = [
  { collapse = "detail", control = ".toggle" },
  { scroll_to = "row-7", within = "panel", offset = 10 },
  { wait = "late" },
]
highlight = ["rows", "beta"]

[[lines]]
id = "f"
section = "main"
show = "Wait for a slow result."
do = [{ click = "slow" }]
highlight = ["slow"]

[[lines]]
id = "g"
section = "main"
show = "Click below the fold."
do = [{ click = "deep" }]
highlight = ["deep"]

[[lines]]
id = "h"
section = "main"
show = "Click what is covered for a moment."
do = [{ click = "arm" }, { click = "under" }]
highlight = ["pies"]

[[lines]]
id = "i"
section = "main"
show = "Point at list items."
highlight = ["kind-again", "sweet", "gauge"]

[[lines]]
id = "j"
section = "main"
show = "Go to the next page."
do = [{ click = "go" }]
highlight = ["arrived"]

[scenes.page]
kind = "web"
url = "page.html"
viewport = [640, 360]
scale = 2
setup = [{ wait = "status" }]

[scenes.page.targets]
status = { css = "#status" }
tab-a = { css = ".tab", text = "Alpha" }
beta = { any = [{ css = ".tab", attr = { name = "title", matches = "(?i)configure BETA" } }] }
choice = { css = "#choice" }
panel = { css = "#panel" }
row-3 = { css = ".row", text = "row 3" }
row-7 = { css = ".row", text = "row 7" }
rows = { css = ".row", text = { starts_with = "row" }, merge = true }
detail = { css = "#detail" }
detail-state = { css = "#detail-state" }
code-line-1 = { css = "#code", line = 1 }
late = { css = "#late" }
slow = { css = "#slow" }
deep = { css = "#deep" }
arm = { css = "#arm" }
block = { css = "#block" }
under = { css = "#under" }
go = { css = "#go" }
boom = { css = "#boom" }
ask = { css = "#ask" }
pies = { css = ".item", text_of = ".name", text = { matches = "(?i)pie" }, exclude = { contains = "Cheese" } }
sweet = { css = ".item", text_of = ".kind", text = "sweet", pick = "first" }
kind-again = { css = ".item .kind", text = "again", grow = { within = "#list", max_height = 30, margin = 8 } }
names = { css = ".item .name", unique = true }
early = { css = "#early" }
gauge = { css = "#gauge" }
arrived = { css = "#arrived" }

[scenes.page.cameras]
tabs = { union = ["tab-a", "beta"] }
all = { viewport = true }

[scenes.badge]
kind = "page"
url = "overlay.html?r={aspect}"
transparent = true
size = { landscape = [200, 100], portrait = [100, 200] }

[[cuts]]
id = "long"
aspect = "16:9"

[[cuts]]
id = "vertical"
aspect = "9:16"
lines = ["b"]

[[expectations]]
scene = "page"
at = "setup"
target = "early"
text = "style at start, layer in body"

[[expectations]]
scene = "page"
at = "a"
target = "status"
text = "clicked A"

[[expectations]]
scene = "page"
at = "b"
target = "status"
text = "chose 2"

[[expectations]]
scene = "page"
at = "c"
target = "status"
text = "scrolled 80"

[[expectations]]
scene = "page"
at = "d"
target = "detail-state"
text = "detail open"

[[expectations]]
scene = "page"
at = "e"
target = "detail-state"
text = "detail closed"

[[expectations]]
scene = "page"
at = "e"
target = "status"
text = "scrolled 270"

[[expectations]]
scene = "page"
at = "f"
target = "status"
text = "done"

[[expectations]]
scene = "page"
at = "g"
target = "status"
text = "clicked deep"

[[expectations]]
scene = "page"
at = "h"
target = "status"
text = "clicked under"

[[expectations]]
scene = "page"
at = "i"
target = "names"
texts = ["apple pie", "apple tart", "Cheese PIE"]

[[expectations]]
scene = "page"
at = "j"
target = "arrived"
text = "arrived"
"##;

/// The test project in a fresh folder, with `edit` applied to its file.
fn project(edit: impl Fn(&str) -> String) -> (tempfile::TempDir, Project) {
    let dir = tempfile::tempdir().unwrap();
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/capture");
    for name in ["page.html", "page2.html", "overlay.html", "remote.html"] {
        fs::copy(fixtures.join(name), dir.path().join(name)).unwrap();
    }
    let text = edit(PROJECT);
    assert_ne!(text, PROJECT, "the edit changed nothing");
    fs::write(dir.path().join("vidni.toml"), &text).unwrap();
    let project = vidni_core::validate::check(&text, Some(dir.path())).unwrap();
    (dir, project)
}

/// The project as it is.
fn unchanged(text: &str) -> String {
    format!("{text}\n")
}

/// Captures everything the project's cuts need, with `patience` seconds of
/// patience; the files, by line ID or by page scene and aspect.
fn take(dir: &Path, project: &Project, patience: u64) -> Result<Vec<(String, PathBuf)>, String> {
    let cuts: Vec<_> = project.cuts.iter().collect();
    let plan = capture::plan(project, dir, &cuts);
    let limits = Limits {
        patience: Duration::from_secs(patience),
        ..Limits::default()
    };
    let taken = capture_with(project, dir, &plan, limits)?;
    Ok(taken
        .into_iter()
        .map(|c| {
            let name = match &c.source {
                Source::Line(id) => id.clone(),
                Source::Page { scene, aspect, .. } => format!("{scene}-{aspect}"),
            };
            (name, capture::path(dir, &c.key))
        })
        .collect())
}

fn frames(dir: &Path, project: &Project, line: &str) -> Frames {
    let key = &capture::line_keys(project)[line];
    serde_json::from_str(&fs::read_to_string(frames_path(dir, key)).unwrap()).unwrap()
}

fn picture(taken: &[(String, PathBuf)], name: &str) -> Image {
    let (_, path) = taken.iter().find(|(n, _)| n == name).unwrap();
    Image::load_png(path).unwrap()
}

/// How many files a failed capture left in the captures folder.
fn left_behind(dir: &Path) -> usize {
    fs::read_dir(dir.join(capture::DIR))
        .map(|d| d.count())
        .unwrap_or(0)
}

#[test]
#[ignore = "needs Chromium: just test-capture"]
fn a_scene_runs_its_actions_and_frames_each_line() {
    let (dir, project) = project(unchanged);
    let taken = take(dir.path(), &project, 30).unwrap();
    // Ten stills, and the overlay for each of the two aspects.
    assert_eq!(taken.len(), 12, "{taken:?}");
    // Every expectation held, or the capture would have failed; the
    // readings are kept.
    let readings = fs::read_to_string(dir.path().join("build/onscreen.json")).unwrap();
    assert!(readings.contains("Cheese PIE"), "{readings}");

    let scene = &project.scenes["page"];
    let a = frames(dir.path(), &project, "a");
    assert_eq!((a.viewport, a.scale), ([640, 360], 2));
    assert_eq!(a.highlights["tab-a"], vec![[10.0, 40.0, 80.0, 30.0]]);
    // Every target of the scene is measured, whether a camera names it
    // yet or not; one that shows nothing has no boxes.
    assert_eq!(a.targets.len(), scene.targets.len());
    assert_eq!(a.targets["tab-a"], vec![[10.0, 40.0, 80.0, 30.0]]);
    assert_eq!(a.targets["arrived"], Vec::<[f64; 4]>::new());
    // The cameras are framed from them.
    assert_eq!(a.camera("tabs", scene), Some([10.0, 40.0, 170.0, 30.0]));
    assert_eq!(a.camera("all", scene), Some([0.0, 0.0, 640.0, 360.0]));
    assert_eq!(a.camera("viewport", scene), a.camera("all", scene));
    assert_eq!(
        a.camera("highlights", scene),
        Some([10.0, 40.0, 80.0, 30.0])
    );
    let still = picture(&taken, "a");
    assert_eq!((still.width, still.height), (1280, 720));

    // Row 3 sits 120 px down the panel, which scrolled 80 px at once,
    // though the panel asks for smooth scrolling.
    let row = frames(dir.path(), &project, "c").highlights["row-3"][0];
    assert_eq!((row[0], row[1], row[3]), (10.0, 140.0, 40.0));
    assert!((190.0..=200.0).contains(&row[2]), "{row:?}");

    // The second line of the <pre>, padded by 6 and 3 pixels.
    let line = frames(dir.path(), &project, "d").highlights["code-line-1"][0];
    assert!((line[0] - 224.0).abs() < 1.0, "{line:?}");
    assert!((line[1] - 177.0).abs() < 3.0, "{line:?}");
    assert!((20.0..=30.0).contains(&line[3]), "{line:?}");

    // Merged rows: one box, clipped to the panel; `beta` through `any`
    // and an attribute rule with (?i).
    let e = frames(dir.path(), &project, "e");
    let rows = &e.highlights["rows"];
    assert_eq!(rows.len(), 1);
    assert_eq!((rows[0][0], rows[0][1], rows[0][3]), (10.0, 100.0, 100.0));
    assert_eq!(e.highlights["beta"], vec![[100.0, 40.0, 80.0, 30.0]]);

    // The slow result arrived 1.2 s after its click, with nothing changing
    // meanwhile; the still waited for the expectation, so the lamp is lit.
    let lit = picture(&taken, "f");
    assert_eq!(&lit.pixel(770, 110)[..3], [0x00, 0xaa, 0x00]);

    // A text rule on a child, a regular expression and an exclusion: the
    // two apple pies. Then `pick`, and a match grown to its item.
    let h = frames(dir.path(), &project, "h");
    assert_eq!(
        h.highlights["pies"],
        vec![[450.0, 100.0, 160.0, 24.0], [450.0, 172.0, 160.0, 24.0]]
    );
    let i = frames(dir.path(), &project, "i");
    assert_eq!(i.highlights["sweet"], vec![[450.0, 100.0, 160.0, 24.0]]);
    assert_eq!(
        i.highlights["kind-again"],
        vec![[450.0, 172.0, 160.0, 24.0]]
    );
    // A gauge the page holds at 40 % with a paused animation stays there.
    assert_eq!(i.highlights["gauge"], vec![[450.0, 210.0, 40.0, 10.0]]);

    // The last line's click went to another page, which asked before the
    // first was left and then swapped its body; capture followed.
    let j = frames(dir.path(), &project, "j");
    assert_eq!(j.highlights["arrived"], vec![[40.0, 60.0, 200.0, 40.0]]);
    assert_eq!(j.camera("tabs", scene), None);

    // The overlay, at each aspect's size, on a transparent background.
    let wide = picture(&taken, "badge-16x9");
    assert_eq!((wide.width, wide.height, wide.channels), (200, 100, 4));
    assert_eq!(wide.pixel(0, 0)[3], 0);
    assert_eq!(wide.pixel(100, 50), [0x20, 0x60, 0xc0, 0xff]);
    let tall = picture(&taken, "badge-9x16");
    assert_eq!((tall.width, tall.height, tall.channels), (100, 200, 4));
    assert_eq!(tall.pixel(0, 0)[3], 0);
    assert_eq!(tall.pixel(50, 100), [0xc0, 0x60, 0x20, 0xff]);
}

#[test]
#[ignore = "needs Chromium: just test-capture"]
fn five_captures_of_a_scene_are_identical() {
    let (one, project) = project(unchanged);
    let first = take(one.path(), &project, 30).unwrap();
    let sidecar = |dir: &Path, name: &str| {
        capture::line_keys(&project)
            .get(name)
            .map(|key| fs::read(frames_path(dir, key)).unwrap())
    };
    for run in 2..=5 {
        let (again, _) = self::project(unchanged);
        let taken = take(again.path(), &project, 30).unwrap();
        assert_eq!(taken.len(), first.len());
        for ((name, a), (_, b)) in first.iter().zip(&taken) {
            assert!(
                fs::read(a).unwrap() == fs::read(b).unwrap(),
                "{name}: capture {run} differs from capture 1"
            );
            assert_eq!(
                sidecar(one.path(), name),
                sidecar(again.path(), name),
                "{name}: the frames of capture {run}"
            );
        }
    }
}

#[test]
#[ignore = "needs Chromium: just test-capture"]
fn a_page_that_shows_other_text_is_refused_and_no_still_is_kept() {
    let (dir, project) = project(|p| p.replacen("text = \"clicked A\"", "text = \"clicked B\"", 1));
    let error = take(dir.path(), &project, 2).unwrap_err();
    assert!(
        error.contains(r#"at line "a", target "status" shows "clicked A", not "clicked B""#),
        "{error}"
    );
    assert_eq!(left_behind(dir.path()), 0);
}

#[test]
#[ignore = "needs Chromium: just test-capture"]
fn a_highlight_that_matches_nothing_names_its_line_and_target() {
    let (dir, project) = project(|p| {
        p.replacen(
            "late = { css = \"#late\" }",
            "late = { css = \"#late\" }\nnone = { css = \"#none\" }",
            1,
        )
        .replacen("highlight = [\"choice\"]", "highlight = [\"none\"]", 1)
    });
    let error = take(dir.path(), &project, 2).unwrap_err();
    assert!(error.contains("line b"), "{error}");
    assert!(
        error.contains("highlight none matches nothing on screen"),
        "{error}"
    );
    assert_eq!(left_behind(dir.path()), 0);
}

#[test]
#[ignore = "needs Chromium: just test-capture"]
fn a_project_page_that_loses_a_part_to_the_network_is_refused() {
    // The overlay's page now asks for a style sheet that nothing serves.
    let (dir, project) =
        project(|text| text.replace("overlay.html?r={aspect}", "remote.html?r={aspect}"));
    let error = take(dir.path(), &project, 30).unwrap_err();
    // Chromium says why in its own words; port 1 it refuses to try at all.
    assert!(
        error.contains(
            "the page asked the network for a part that didn't load \
             (the first: http://127.0.0.1:1/none.css (net::ERR_"
        ),
        "{error}"
    );
    // No render of it is kept, so a later capture can't mistake it for whole.
    let kept: Vec<_> = capture::for_cut(&project, &project.cuts[0])
        .into_iter()
        .filter(|c| matches!(c.source, Source::Page { .. }))
        .filter(|c| capture::path(dir.path(), &c.key).is_file())
        .collect();
    assert_eq!(kept, []);
}

#[test]
#[ignore = "needs Chromium: just test-capture"]
fn a_script_error_on_the_page_fails_its_line() {
    let (dir, project) =
        project(|p| p.replacen("{ click = \"tab-a\" }", "{ click = \"boom\" }", 1));
    let error = take(dir.path(), &project, 2).unwrap_err();
    assert!(error.contains("scene page, at line a"), "{error}");
    assert!(error.contains("the page threw: Error: boom"), "{error}");
}

#[test]
#[ignore = "needs Chromium: just test-capture"]
fn a_dialog_is_dismissed_and_fails_its_line() {
    let (dir, project) = project(|p| p.replacen("{ click = \"tab-a\" }", "{ click = \"ask\" }", 1));
    let error = take(dir.path(), &project, 2).unwrap_err();
    assert!(error.contains("scene page, at line a"), "{error}");
    assert!(
        error.contains("the page opened a dialog (alert), which was dismissed: hi"),
        "{error}"
    );
}

#[test]
#[ignore = "needs Chromium: just test-capture"]
fn a_click_refuses_to_land_on_what_covers_its_target() {
    let (dir, project) = project(|p| p.replacen("{ click = \"arm\" }", "{ click = \"block\" }", 1));
    let error = take(dir.path(), &project, 2).unwrap_err();
    assert!(error.contains("scene page, at line h"), "{error}");
    assert!(
        error.contains("target under is covered where a click would land"),
        "{error}"
    );
}

/// A page served over HTTP, to show what a file can't: a cookie the server
/// sets, and a network that stays busy.
const VISIT: &str = r##"<!doctype html>
<meta charset="utf-8">
<style>
  body { margin: 0; font: 16px/20px sans-serif; }
  div, button { position: absolute; left: 10px; height: 20px; }
  #server { top: 10px; } #local { top: 40px; } #status { top: 70px; }
  #busy { top: 100px; width: 60px; height: 30px; border: 0; }
</style>
<div id="server">SERVER</div>
<div id="local"></div>
<div id="status">quiet</div>
<button id="busy" onclick="start()">Busy</button>
<script>
  const n = Number(localStorage.getItem('n') || 0) + 1;
  localStorage.setItem('n', n);
  document.getElementById('local').textContent = 'local visit ' + n;
  function start() {
    setInterval(() => fetch('/ping?' + Date.now()), 150);
    document.getElementById('status').textContent = 'busy';
  }
</script>
"##;

const SERVED: &str = r##"
schema = 1

[meta]
title = "Served"
default_scene = "page"

[sections.main]

[[lines]]
id = "first"
section = "main"
show = "A first visit."
highlight = ["server"]

[[lines]]
id = "busy"
section = "main"
show = "A page that keeps asking."
do = [{ click = "busy" }]
highlight = ["status"]

[scenes.page]
kind = "web"
url = "http://127.0.0.1:PORT/visit"
viewport = [320, 200]
scale = 2

[scenes.page.targets]
server = { css = "#server" }
local = { css = "#local" }
status = { css = "#status" }
busy = { css = "#busy" }

[[cuts]]
id = "long"
aspect = "16:9"

[[expectations]]
scene = "page"
at = "first"
target = "server"
text = "first visit"

[[expectations]]
scene = "page"
at = "first"
target = "local"
text = "local visit 1"

[[expectations]]
scene = "page"
at = "busy"
target = "status"
text = "busy"
"##;

/// Serves [`VISIT`] at `/visit` on a local port, which it returns. The page
/// says "first visit" to a request without the cookie it sets, and "welcome
/// back" to one with it. `/ping` answers at once.
fn serve() -> u16 {
    use std::io::{BufRead, BufReader, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            std::thread::spawn(move || {
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut head = Vec::new();
                loop {
                    let mut line = String::new();
                    if reader.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
                        break;
                    }
                    head.push(line);
                }
                let path = head
                    .first()
                    .and_then(|l| l.split(' ').nth(1))
                    .unwrap_or("/");
                let seen = head
                    .iter()
                    .any(|l| l.to_lowercase().starts_with("cookie:") && l.contains("seen=1"));
                let (kind, extra, body) = if path.starts_with("/visit") {
                    let greeting = if seen { "welcome back" } else { "first visit" };
                    (
                        "text/html",
                        "Set-Cookie: seen=1; Path=/\r\n",
                        VISIT.replace("SERVER", greeting),
                    )
                } else {
                    ("text/plain", "", "ok".to_string())
                };
                let _ = write!(
                    &stream,
                    "HTTP/1.1 200 OK\r\nContent-Type: {kind}\r\nContent-Length: {}\r\n\
                     Cache-Control: no-store\r\n{extra}Connection: close\r\n\r\n{body}",
                    body.len()
                );
            });
        }
    });
    port
}

#[test]
#[ignore = "needs Chromium: just test-capture"]
fn a_page_loaded_twice_is_still_a_first_visit_and_a_busy_network_settles() {
    let dir = tempfile::tempdir().unwrap();
    let text = SERVED.replace("PORT", &serve().to_string());
    fs::write(dir.path().join("vidni.toml"), &text).unwrap();
    let project = vidni_core::validate::check(&text, Some(dir.path())).unwrap();
    // At scale 2 the page is loaded twice. The first load ran no script and
    // its cookie was cleared, so the second is a first visit to the server
    // and to the page's own storage. Then the page asks the server
    // something every 150 ms for as long as it is shown, and still settles.
    let taken = take(dir.path(), &project, 20).unwrap();
    assert_eq!(taken.len(), 2, "{taken:?}");
}

/// A page in a folder of the project's, with a stylesheet that imports
/// another, a picture, a picture that isn't there, and a script that
/// loads one more picture when a button is clicked; and a card that
/// shares the stylesheets.
const SITE: &str = r##"schema = 1

[meta]
title = "Files"
default_scene = "site"

[[cuts]]
id = "long"
aspect = "16:9"

[sections.main]

[[lines]]
id = "main-1"
section = "main"
show = "A page with its parts."

[[lines]]
id = "main-2"
section = "main"
show = "A click loads one more."
do = [{ click = "more" }]

[[lines]]
id = "main-3"
section = "main"
show = "A card."
scene = "card"

[scenes.site]
kind = "web"
url = "site/index.html?x=1#top"
viewport = [320, 180]

[scenes.site.targets]
more = { css = "button" }

[scenes.card]
kind = "page"
url = "site/card.html"
size = { landscape = [320, 180], portrait = [180, 320] }
"##;

/// The files of [`SITE`], by name.
const SITE_FILES: [(&str, &str); 7] = [
    (
        "site/index.html",
        r#"<!doctype html>
<html><head><meta charset="utf-8">
<link rel="stylesheet" href="css/style.css">
</head><body>
<h1>Files</h1>
<img src="img/dot.svg" width="20" height="20" alt="">
<img src="img/gone.png" width="20" height="20" alt="">
<button onclick="more()">More</button>
<script src="app.js"></script>
</body></html>
"#,
    ),
    (
        "site/css/style.css",
        "@import url(\"more.css\");\nbody { margin: 8px; font: 16px sans-serif; }\n",
    ),
    ("site/css/more.css", "h1 { color: #1d3557; }\n"),
    (
        "site/img/dot.svg",
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><circle cx="5" cy="5" r="4" fill="#e63946"/></svg>"##,
    ),
    (
        "site/img/late.svg",
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><rect width="10" height="10" fill="#2a9d8f"/></svg>"##,
    ),
    (
        "site/app.js",
        "function more() {\n  const img = new Image(20, 20);\n  img.src = \"img/late.svg\";\n  document.body.append(img);\n}\n",
    ),
    (
        "site/card.html",
        "<!doctype html><link rel=\"stylesheet\" href=\"css/style.css\"><h1>Card</h1>\n",
    ),
];

#[test]
#[ignore = "needs Chromium: just test-capture"]
fn a_capture_of_a_project_page_records_the_files_it_was_made_from() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    for (name, text) in SITE_FILES {
        fs::create_dir_all(dir.join(name).parent().unwrap()).unwrap();
        fs::write(dir.join(name), text).unwrap();
    }
    let project = vidni_core::validate::check(SITE, Some(dir)).unwrap();
    let cuts: Vec<_> = project.cuts.iter().collect();
    let planned = || -> Vec<String> {
        capture::plan(&project, dir, &cuts)
            .into_iter()
            .map(|c| c.what)
            .collect()
    };
    let all = [
        "line main-1",
        "line main-2",
        "page card at 320x180 for 16x9",
    ];
    assert_eq!(planned(), all);
    let taken = take(dir, &project, 30).unwrap();
    assert_eq!(taken.len(), 3);
    assert_eq!(planned(), Vec::<String>::new());

    // Each capture's record: the project's files its page had asked for
    // by then, with what they held; none for one that wasn't there.
    let captures = capture::for_cut(&project, cuts[0]);
    let record = |what: &str| -> Vec<(String, bool)> {
        let c = captures.iter().find(|c| c.what == what).unwrap();
        let files = capture::files(dir, &c.key).unwrap();
        for (name, sha) in &files {
            assert_eq!(*sha, capture::file_sha256(dir, name), "{name}");
        }
        files
            .into_iter()
            .map(|(name, sha)| (name, sha.is_some()))
            .collect()
    };
    let named = |names: &[(&str, bool)]| -> Vec<(String, bool)> {
        names
            .iter()
            .map(|(n, there)| (n.to_string(), *there))
            .collect()
    };
    let page = [
        ("site/app.js", true),
        ("site/css/more.css", true),
        ("site/css/style.css", true),
        ("site/img/dot.svg", true),
        ("site/img/gone.png", false),
        ("site/index.html", true),
    ];
    assert_eq!(record("line main-1"), named(&page));
    // The second line's click loaded one more picture.
    let mut clicked = page.to_vec();
    clicked.insert(5, ("site/img/late.svg", true));
    assert_eq!(record("line main-2"), named(&clicked));
    assert_eq!(
        record("page card at 320x180 for 16x9"),
        named(&[
            ("site/card.html", true),
            ("site/css/more.css", true),
            ("site/css/style.css", true),
        ])
    );

    // The imported stylesheet changes: every capture that loaded it is out
    // of date, and taken again it shows the change.
    let before = picture(&taken, "main-1");
    fs::write(dir.join("site/css/more.css"), "h1 { color: #e63946; }\n").unwrap();
    assert_eq!(planned(), all);
    let retaken = take(dir, &project, 30).unwrap();
    assert_eq!(retaken.len(), 3);
    assert_ne!(picture(&retaken, "main-1").data, before.data);
    assert_eq!(planned(), Vec::<String>::new());
    // What only the page loads leaves the card's render as it is; so does
    // a file that turns up where the page had found none.
    fs::write(dir.join("site/app.js"), "function more() {}\n").unwrap();
    assert_eq!(planned(), &all[..2]);
    fs::write(dir.join("site/app.js"), SITE_FILES[5].1).unwrap();
    fs::write(dir.join("site/img/gone.png"), b"not a picture").unwrap();
    assert_eq!(planned(), &all[..2]);
    // A picture loaded only after the click is the second line's alone.
    fs::remove_file(dir.join("site/img/gone.png")).unwrap();
    fs::write(dir.join("site/img/late.svg"), SITE_FILES[3].1).unwrap();
    assert_eq!(planned(), &all[1..2]);
}

#[test]
#[ignore = "needs Chromium: just test-capture"]
fn a_project_page_that_loads_a_file_from_outside_the_project_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    // The stylesheet is beside the project folder, not in it.
    let project_dir = dir.join("project");
    fs::write(dir.join("shared.css"), "h1 { color: #1d3557; }\n").unwrap();
    for (name, text) in SITE_FILES {
        fs::create_dir_all(project_dir.join(name).parent().unwrap()).unwrap();
        let text = text.replace("href=\"css/style.css\"", "href=\"../../shared.css\"");
        fs::write(project_dir.join(name), text).unwrap();
    }
    let project = vidni_core::validate::check(SITE, Some(&project_dir)).unwrap();
    let error = take(&project_dir, &project, 30).unwrap_err();
    let shared = fs::canonicalize(dir.join("shared.css")).unwrap();
    assert_eq!(
        error,
        format!(
            "scene site, at setup: the page loaded a file from outside the project folder (the first: {})",
            shared.display()
        )
    );
    assert_eq!(left_behind(&project_dir), 0);
}

#[test]
#[ignore = "needs mmdc and Chromium, as the dev shell has them: just test-capture"]
fn a_mermaid_diagram_is_drawn_the_same_each_time_at_its_scale() {
    use vidni_core::capture as keys;
    let dir = tempfile::tempdir().unwrap();
    let text = "schema = 1\n\n[meta]\ntitle = \"A diagram\"\ndefault_scene = \"flow\"\n\n\
                [[cuts]]\nid = \"long\"\naspect = \"16:9\"\n\n[sections.main]\n\n\
                [[lines]]\nid = \"main-1\"\nsection = \"main\"\nshow = \"A diagram.\"\n\n\
                [scenes.flow]\nkind = \"image\"\nurl = \"flow.mmd\"\nscale = 2\n\n\
                [scenes.flow.targets]\nfirst = { rect = [0, 0, 100, 50] }\n";
    std::fs::write(dir.path().join("vidni.toml"), text).unwrap();
    std::fs::write(
        dir.path().join("flow.mmd"),
        "flowchart LR\n  a[One] --> b[Two]\n",
    )
    .unwrap();
    let project = vidni_core::validate::check(text, Some(dir.path())).unwrap();
    let cut = &project.cuts[0];
    let wanted = keys::for_cut(&project, cut);
    assert_eq!(wanted.len(), 1);
    let take = || {
        let _ = std::fs::remove_dir_all(dir.path().join("build"));
        vidni_media::capture::capture(&project, dir.path(), &wanted).unwrap();
        std::fs::read(keys::path(dir.path(), &wanted[0].key)).unwrap()
    };
    let (first, second) = (take(), take());
    assert_eq!(first, second, "two drawings of one diagram differ");
    // Drawn at twice its size, and measured at its own.
    let image = Image::load_png(&keys::path(dir.path(), &wanted[0].key)).unwrap();
    let frames: vidni_media::capture::Frames = serde_json::from_str(
        &std::fs::read_to_string(keys::frames_path(dir.path(), &wanted[0].key)).unwrap(),
    )
    .unwrap();
    assert_eq!(frames.scale, 2);
    assert_eq!(
        (image.width, image.height),
        (frames.viewport[0] * 2, frames.viewport[1] * 2)
    );
    assert_eq!(frames.targets["first"], [[0.0, 0.0, 100.0, 50.0]]);
    // The diagram changes: the capture is of another file, so it is stale.
    std::fs::write(
        dir.path().join("flow.mmd"),
        "flowchart LR\n  a[One] --> b[Two] --> c[Three]\n",
    )
    .unwrap();
    assert_ne!(take(), first);
    // A diagram Mermaid can't read is said, and nothing is taken.
    std::fs::write(dir.path().join("flow.mmd"), "this is no diagram\n").unwrap();
    let _ = std::fs::remove_dir_all(dir.path().join("build"));
    let why = vidni_media::capture::capture(&project, dir.path(), &wanted).unwrap_err();
    assert!(why.contains("flow.mmd: Mermaid couldn't draw it"), "{why}");
    assert!(!keys::path(dir.path(), &wanted[0].key).exists());
}
