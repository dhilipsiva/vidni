//! The preview player (TODO.md, P7-VIDNI-03): a built cut, frame by
//! frame, with its lines to jump between.
//!
//! Frames are decoded by the pinned ffmpeg (DECISIONS.md, D9) on a thread
//! of the player's own. The window never waits for a frame: it asks for
//! one, goes on drawing, and shows the frame when it comes. When frames
//! are asked for faster than they come, as when the position is dragged,
//! only the latest request is decoded.

use eframe::egui;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};
use vidni_core::timeline::Timeline;
use vidni_media::compose::image::Image;
use vidni_media::preview::Video;

/// How wide the picture is decoded, and each picture of the contact sheet.
const WIDTH: u32 = 960;
const THUMB: u32 = 240;

/// What the decoding thread is asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Wanted {
    /// The frame to show.
    Frame(u64),
    /// The contact sheet's picture of a line, by the line's place.
    Thumb(usize, u64),
}

/// What it gives back: what was asked, the picture, and how long it took.
struct Decoded {
    wanted: Wanted,
    image: Result<Image, String>,
    took: Duration,
}

#[derive(Default)]
struct Requests {
    /// The frame to show next: a newer request takes an older one's place.
    frame: Option<u64>,
    /// The contact sheet's pictures still to make, in order.
    thumbs: Vec<(usize, u64)>,
    quit: bool,
}

/// One line of the cut on its timeline.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub id: String,
    pub show: String,
    /// When its first cue starts and its last ends, in seconds.
    pub start: f64,
    pub end: f64,
    /// The section it is in.
    pub section: String,
}

/// A cut that is playing: the frames as they are decoded, and its sound.
struct Playing {
    /// The moment that frame `from` was shown.
    started: Instant,
    from: u64,
    frames: Receiver<(u64, Image)>,
    /// A frame that is decoded and not due yet.
    waiting: Option<(u64, Image)>,
    sound: Option<Child>,
}

impl Drop for Playing {
    fn drop(&mut self) {
        if let Some(sound) = &mut self.sound {
            let _ = sound.kill();
            let _ = sound.wait();
        }
    }
}

/// A built cut, open in the preview.
pub struct Player {
    pub video: Video,
    pub lines: Vec<Line>,
    /// The frame the player is at.
    pub position: u64,
    /// The frame that is on screen, once it is decoded.
    pub shown: Option<u64>,
    texture: Option<egui::TextureHandle>,
    requests: Arc<(Mutex<Requests>, Condvar)>,
    decoded: Receiver<Decoded>,
    /// How long each frame shown took to decode, for the last few.
    pub took: Vec<Duration>,
    /// How many frames have been decoded to show.
    pub decodes: u64,
    /// Why the last frame couldn't be decoded.
    pub failed: Option<String>,
    playing: Option<Playing>,
    /// The contact sheet: a picture of each line, as they come.
    pub sheet: bool,
    thumbs: Vec<Option<egui::TextureHandle>>,
    thumbs_asked: bool,
    wake: Arc<dyn Fn() + Send + Sync>,
}

/// The lines of `timeline`, each with the ID its place in `ids` gives it.
pub fn lines(timeline: &Timeline, ids: &[String]) -> Vec<Line> {
    timeline
        .sentences
        .iter()
        .enumerate()
        .map(|(i, entry)| Line {
            id: ids
                .get(i)
                .cloned()
                .unwrap_or_else(|| format!("line {}", i + 1)),
            show: entry.show.clone(),
            start: entry.cues.first().map_or(entry.start, |cue| cue.start),
            end: entry.end,
            section: entry.segment.clone(),
        })
        .collect()
}

impl Player {
    /// Opens the picture at `mp4`, with the lines of its cut. `wake` is
    /// called whenever there is something new to draw.
    pub fn open(
        mp4: &Path,
        lines: Vec<Line>,
        wake: impl Fn() + Send + Sync + 'static,
    ) -> Result<Self, String> {
        let video = Video::open(mp4)?;
        let requests: Arc<(Mutex<Requests>, Condvar)> = Arc::default();
        let (tx, decoded) = mpsc::channel();
        let wake: Arc<dyn Fn() + Send + Sync> = Arc::new(wake);
        spawn_decoder(video.clone(), requests.clone(), tx, wake.clone());
        let mut player = Self {
            thumbs: vec![None; lines.len()],
            video,
            lines,
            position: 0,
            shown: None,
            texture: None,
            requests,
            decoded,
            took: Vec::new(),
            decodes: 0,
            failed: None,
            playing: None,
            sheet: false,
            thumbs_asked: false,
            wake,
        };
        player.seek(0);
        Ok(player)
    }

    /// Goes to frame `index`, and asks for it. It returns at once: the
    /// frame is shown when it has been decoded.
    pub fn seek(&mut self, index: u64) {
        self.pause();
        self.go(index);
    }

    fn go(&mut self, index: u64) {
        self.position = index.min(self.video.frames.saturating_sub(1));
        if self.shown != Some(self.position) {
            let (requests, more) = &*self.requests;
            requests.lock().expect("the requests").frame = Some(self.position);
            more.notify_one();
        }
    }

    /// Why a frame couldn't be decoded, in a line a person can use.
    fn explain(&self, error: String) -> String {
        match self.video.path.is_file() {
            false => format!(
                "{} is no longer there; build the cut again, and it plays here",
                self.video.path.display()
            ),
            true => {
                let last = error.lines().last().unwrap_or_default().trim().to_string();
                format!(
                    "{} can't be read as it was when it was opened ({last}); it may have been built again",
                    self.video.path.display()
                )
            }
        }
    }

    /// Goes to where line `i` starts: the frame its first cue shows on.
    pub fn seek_line(&mut self, i: usize) {
        if let Some(line) = self.lines.get(i) {
            self.seek(self.video.frame_at(line.start));
        }
    }

    /// The line that is said at the player's position, by its place.
    pub fn line_at(&self) -> Option<usize> {
        // A line has begun once the frame its cue starts on is reached.
        self.lines
            .iter()
            .rposition(|line| self.video.frame_at(line.start) <= self.position)
    }

    /// Whether the cut is playing.
    pub fn playing(&self) -> bool {
        self.playing.is_some()
    }

    /// Plays from the player's position, with the cut's sound when
    /// `ffplay` is at hand and `VIDNI_PREVIEW_MUTE` isn't set.
    pub fn play(&mut self) {
        if self.position + 1 >= self.video.frames {
            self.go(0);
        }
        if !self.video.path.is_file() {
            self.failed = Some(self.explain(String::new()));
            return;
        }
        let Ok(stream) = self.video.play(self.position, WIDTH) else {
            return;
        };
        // The frames are decoded ahead on a thread, a second's worth at most.
        let (tx, frames) = mpsc::sync_channel(self.video.fps.ceil() as usize);
        let wake = self.wake.clone();
        std::thread::spawn(move || {
            for frame in stream {
                if tx.send(frame).is_err() {
                    break;
                }
                wake();
            }
        });
        let muted = std::env::var_os("VIDNI_PREVIEW_MUTE").is_some();
        let sound = (!muted)
            .then(|| {
                Command::new("ffplay")
                    .args(["-nodisp", "-autoexit", "-loglevel", "quiet", "-vn", "-ss"])
                    .arg(format!("{:.3}", self.video.time_of(self.position)))
                    .arg(&self.video.path)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .ok()
            })
            .flatten();
        self.playing = Some(Playing {
            started: Instant::now(),
            from: self.position,
            frames,
            waiting: None,
            sound,
        });
    }

    /// Stops playing, at the frame that is showing.
    pub fn pause(&mut self) {
        self.playing = None;
    }

    /// Takes in the frames that have been decoded since the last call,
    /// and, while playing, moves on with the clock.
    pub fn poll(&mut self, ctx: &egui::Context) {
        while let Ok(decoded) = self.decoded.try_recv() {
            match (decoded.wanted, decoded.image) {
                // A frame is shown as soon as it is decoded, even when
                // the position has moved on since it was asked for: the
                // picture then follows a drag at the decoder's pace, and
                // the frame for where the drag is now is already on its way.
                (Wanted::Frame(index), Ok(image)) => {
                    self.show(ctx, index, &image);
                    self.decodes += 1;
                    self.took.push(decoded.took);
                    if self.took.len() > 64 {
                        self.took.remove(0);
                    }
                    self.failed = None;
                }
                (Wanted::Frame(index), Err(e)) if index == self.position => {
                    self.failed = Some(self.explain(e));
                }
                (Wanted::Frame(_), Err(_)) => {}
                (Wanted::Thumb(i, _), Ok(image)) => {
                    if let Some(slot) = self.thumbs.get_mut(i) {
                        *slot = Some(texture(ctx, &format!("thumb {i}"), &image));
                    }
                }
                (Wanted::Thumb(..), Err(_)) => {}
            }
        }
        let Some(playing) = &mut self.playing else {
            return;
        };
        // The frame that is due now, by the clock: every frame up to it
        // is taken, and the last of them shown. A frame that isn't due
        // yet waits for its moment.
        let due = playing.from + (playing.started.elapsed().as_secs_f64() * self.video.fps) as u64;
        let mut latest = None;
        let mut ended = false;
        loop {
            if playing.waiting.is_none() {
                match playing.frames.try_recv() {
                    Ok(frame) => playing.waiting = Some(frame),
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        ended = true;
                        break;
                    }
                }
            }
            match playing.waiting.take_if(|(index, _)| *index <= due) {
                Some(frame) => latest = Some(frame),
                None => break,
            }
        }
        if let Some((index, image)) = latest {
            self.position = index;
            self.show(ctx, index, &image);
        }
        match ended {
            true => self.pause(),
            false => ctx.request_repaint_after(Duration::from_secs_f64(1.0 / self.video.fps)),
        }
    }

    fn show(&mut self, ctx: &egui::Context, index: u64, image: &Image) {
        let size = [image.width as usize, image.height as usize];
        let pixels = egui::ColorImage::from_rgb(size, &image.data);
        // One texture, given each new picture in turn.
        match &mut self.texture {
            Some(texture) => texture.set(pixels, egui::TextureOptions::LINEAR),
            None => {
                self.texture =
                    Some(ctx.load_texture("preview", pixels, egui::TextureOptions::LINEAR))
            }
        }
        self.shown = Some(index);
    }

    /// Asks for the contact sheet's pictures, once.
    fn ask_for_thumbs(&mut self) {
        if self.thumbs_asked {
            return;
        }
        self.thumbs_asked = true;
        let (requests, more) = &*self.requests;
        let mut requests = requests.lock().expect("the requests");
        // Each line's picture is the frame halfway through it.
        requests.thumbs = self
            .lines
            .iter()
            .enumerate()
            .map(|(i, line)| (i, self.video.frame_at((line.start + line.end) / 2.0)))
            .collect();
        more.notify_one();
    }

    /// Draws the player: the picture, the position with the lines marked
    /// on it, and the lines to jump to; or the contact sheet.
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        self.poll(ui.ctx());
        ui.horizontal(|ui| {
            let label = if self.playing() { "Pause" } else { "Play" };
            if ui.button(label).clicked() {
                match self.playing() {
                    true => self.pause(),
                    false => self.play(),
                }
            }
            if ui.button("Previous line").clicked() {
                match self.line_at() {
                    // Before the first line, the start of the cut is what is before.
                    None => self.seek(0),
                    Some(at) => {
                        // From inside a line, its own start comes first.
                        let start = self.video.frame_at(self.lines[at].start);
                        match start < self.position {
                            true => self.seek_line(at),
                            false => self.seek_line(at.saturating_sub(1)),
                        }
                    }
                }
            }
            if ui.button("Next line").clicked() {
                let next = self.line_at().map_or(0, |at| at + 1);
                if next < self.lines.len() {
                    self.seek_line(next);
                }
            }
            ui.checkbox(&mut self.sheet, "Contact sheet");
            ui.label(format!(
                "{} of {}, frame {}",
                clock(self.video.time_of(self.position)),
                clock(self.video.seconds()),
                self.position
            ));
        });
        if let Some(why) = &self.failed {
            ui.colored_label(ui.visuals().error_fg_color, why);
        }
        if self.sheet {
            self.ask_for_thumbs();
            self.contact_sheet(ui);
            return;
        }
        // The picture, as large as the window has room for.
        // A tall picture is fitted by its height, so that the picture and
        // the position under it are on screen together.
        let (w, h) = self.video.size(WIDTH);
        let room = ui.ctx().content_rect().height() * 0.62;
        let scale = (ui.available_width() / w as f32)
            .min(room / h as f32)
            .min(1.0);
        let size = egui::vec2(w as f32 * scale, h as f32 * scale);
        match &self.texture {
            Some(texture) => {
                ui.add(egui::Image::new((texture.id(), size)).alt_text("The cut's picture"));
            }
            None => {
                ui.allocate_space(size);
            }
        }
        // The ticks and the slider under them are one scale: as wide as
        // each other, so a tick is above the place the slider gives it.
        let width = size.x.max(240.0);
        self.markers(ui, width);
        let last = self.video.frames.saturating_sub(1);
        let mut position = self.position;
        ui.spacing_mut().slider_width = width;
        let label = ui.label("Position");
        let slider = ui
            .add(egui::Slider::new(&mut position, 0..=last).show_value(false))
            .labelled_by(label.id);
        if slider.changed() {
            self.seek(position);
        }
        ui.add_space(6.0);
        let current = self.line_at();
        let mut section = None;
        for i in 0..self.lines.len() {
            let line = self.lines[i].clone();
            if section.as_ref() != Some(&line.section) {
                ui.strong(&line.section);
                section = Some(line.section.clone());
            }
            let row = format!("{}  [{}]  {}", clock(line.start), line.id, line.show);
            if ui.selectable_label(current == Some(i), row).clicked() {
                self.seek_line(i);
            }
        }
    }

    /// A strip as wide as the picture, with a tick where each line starts
    /// and a mark at the player's position. A click on it goes there.
    fn markers(&mut self, ui: &mut egui::Ui, width: f32) {
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(width, 18.0), egui::Sense::click_and_drag());
        let total = self.video.seconds().max(1e-6);
        let x_of = |seconds: f64| rect.left() + (seconds / total) as f32 * rect.width();
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 2.0, ui.visuals().extreme_bg_color);
        let (tick, here) = (
            ui.visuals().weak_text_color(),
            ui.visuals().selection.bg_fill,
        );
        for line in &self.lines {
            let x = x_of(line.start);
            painter.line_segment(
                [
                    egui::pos2(x, rect.top() + 4.0),
                    egui::pos2(x, rect.bottom()),
                ],
                egui::Stroke::new(1.0, tick),
            );
        }
        let x = x_of(self.video.time_of(self.position));
        painter.line_segment(
            [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
            egui::Stroke::new(3.0, here),
        );
        let seconds_at =
            |x: f32| f64::from(((x - rect.left()) / rect.width()).clamp(0.0, 1.0)) * total;
        // Under the pointer: the line that is said there.
        if let Some(pointer) = response.hover_pos() {
            let at = seconds_at(pointer.x);
            if let Some(line) = self.lines.iter().rfind(|line| line.start <= at) {
                response
                    .clone()
                    .on_hover_text_at_pointer(format!("[{}]  {}", line.id, line.show));
            }
        }
        // A click goes there, and so does a drag, all the way along.
        if let Some(pointer) = response
            .interact_pointer_pos()
            .filter(|_| response.clicked() || response.dragged())
        {
            self.seek(self.video.frame_at(seconds_at(pointer.x)));
        }
    }

    /// A picture of every line, in rows; a click on one goes to its line.
    fn contact_sheet(&mut self, ui: &mut egui::Ui) {
        let (w, h) = self.video.size(THUMB);
        let size = egui::vec2(w as f32, h as f32);
        let across = ((ui.available_width() / (size.x + 12.0)) as usize).max(1);
        let mut picked = None;
        egui::Grid::new("contact sheet").show(ui, |ui| {
            for (i, line) in self.lines.iter().enumerate() {
                ui.vertical(|ui| {
                    let name = format!("[{}]", line.id);
                    // A line's picture is a button named by the line.
                    let clicked = match &self.thumbs[i] {
                        Some(texture) => ui
                            .add(egui::Button::image_and_text(
                                egui::Image::new((texture.id(), size)),
                                name,
                            ))
                            .on_hover_text(&line.show)
                            .clicked(),
                        // Until its picture comes, the line's name is the button.
                        None => {
                            ui.allocate_space(size);
                            ui.button(&name).clicked()
                        }
                    };
                    if clicked {
                        picked = Some(i);
                    }
                });
                if (i + 1) % across == 0 {
                    ui.end_row();
                }
            }
        });
        if let Some(i) = picked {
            self.sheet = false;
            self.seek_line(i);
        }
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        let (requests, more) = &*self.requests;
        if let Ok(mut requests) = requests.lock() {
            requests.quit = true;
        }
        more.notify_one();
    }
}

/// `seconds` as `m:ss.t`.
fn clock(seconds: f64) -> String {
    let tenths = (seconds.max(0.0) * 10.0).floor() as u64;
    format!("{}:{:02}.{}", tenths / 600, tenths / 10 % 60, tenths % 10)
}

fn texture(ctx: &egui::Context, name: &str, image: &Image) -> egui::TextureHandle {
    let size = [image.width as usize, image.height as usize];
    let pixels = egui::ColorImage::from_rgb(size, &image.data);
    ctx.load_texture(name, pixels, egui::TextureOptions::LINEAR)
}

/// The decoding thread: the frame wanted comes first, then the contact
/// sheet's pictures, one at a time, so that a seek never waits for more
/// than one of them.
fn spawn_decoder(
    video: Video,
    requests: Arc<(Mutex<Requests>, Condvar)>,
    decoded: Sender<Decoded>,
    wake: Arc<dyn Fn() + Send + Sync>,
) {
    std::thread::spawn(move || {
        loop {
            let wanted = {
                let (lock, more) = &*requests;
                let mut requests = lock.lock().expect("the requests");
                loop {
                    if requests.quit {
                        return;
                    }
                    if let Some(index) = requests.frame.take() {
                        break Wanted::Frame(index);
                    }
                    if !requests.thumbs.is_empty() {
                        let (i, index) = requests.thumbs.remove(0);
                        break Wanted::Thumb(i, index);
                    }
                    requests = more.wait(requests).expect("the requests");
                }
            };
            let began = Instant::now();
            let image = match wanted {
                Wanted::Frame(index) => video.frame(index, WIDTH),
                Wanted::Thumb(_, index) => video.frame(index, THUMB),
            };
            let sent = decoded.send(Decoded {
                wanted,
                image,
                took: began.elapsed(),
            });
            if sent.is_err() {
                return;
            }
            wake();
        }
    });
}

/// The pictures a build of `cut` made, in the folder of `voice`'s builds:
/// each file with whether its captions are drawn in.
pub fn pictures(
    project_dir: &Path,
    voice: vidni_core::release::Voice,
    cut: &str,
) -> Vec<(PathBuf, bool)> {
    let Ok(Some(record)) = vidni_core::release::Record::read(project_dir, voice, cut) else {
        return Vec::new();
    };
    let folder = project_dir.join(voice.dir());
    record
        .outputs
        .iter()
        .map(|(name, output)| (folder.join(name), output.captions))
        .filter(|(path, _)| path.is_file())
        .collect()
}
