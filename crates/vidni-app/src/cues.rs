//! Short synchronized cue reviews, using the existing frame-accurate player.
use crate::preview::Player;
use eframe::egui;
use std::sync::mpsc::{self, Receiver};
use vidni_cli::cues::{self, Options};
use vidni_core::cues::Interval;
use vidni_media::cues::{Cue, Review};

pub struct Reviewer {
    options: Options,
    reviews: Vec<Review>,
    keys: Vec<String>,
    at: usize,
    start: f64,
    end: f64,
    player: Option<Player>,
    preview: Option<vidni_media::cues::Preview>,
    job: Option<Receiver<Result<Vec<Review>, String>>>,
    note: String,
    looping: bool,
    played: bool,
}

impl Reviewer {
    pub fn new(options: Options, ctx: &egui::Context) -> Self {
        let mut this = Self {
            options,
            reviews: Vec::new(),
            keys: Vec::new(),
            at: 0,
            start: 0.0,
            end: 0.0,
            player: None,
            preview: None,
            job: None,
            note: String::new(),
            looping: false,
            played: false,
        };
        if std::env::var_os("VIDNI_ASSISTANT").is_some_and(|v| !v.is_empty()) {
            this.note =
                "Cue verification belongs to the person watching and hearing the preview.".into();
        } else {
            this.refresh(ctx, false);
        }
        this
    }

    fn cue(&self) -> Option<&Cue> {
        let key = self.keys.get(self.at)?;
        self.reviews
            .iter()
            .flat_map(|r| &r.cues)
            .find(|c| &c.key == key)
    }

    fn refresh(&mut self, ctx: &egui::Context, render: bool) {
        if self.job.is_some() {
            return;
        }
        let mut options = self.options.clone();
        if render && let Some(cue) = self.cue() {
            options.only = vec![cue.line_id.clone()];
            options.cuts = vec![cue.key.split('/').next().unwrap_or_default().to_string()];
        }
        let (send, receive) = mpsc::channel();
        let ctx = ctx.clone();
        self.note = if render {
            "Rendering this line’s diagnostic preview…"
        } else {
            "Loading cues and local alignment proposals…"
        }
        .into();
        self.player = None;
        self.preview = None;
        self.played = false;
        std::thread::spawn(move || {
            let _ = send.send(cues::prepare(&options, true, render));
            ctx.request_repaint();
        });
        self.job = Some(receive);
    }

    fn poll(&mut self, ctx: &egui::Context) {
        let Some(result) = self.job.as_ref().and_then(|job| job.try_recv().ok()) else {
            return;
        };
        self.job = None;
        match result {
            Err(e) => self.note = e,
            Ok(reviews) => {
                let first = self.keys.is_empty();
                for review in reviews {
                    if let Some(old) = self.reviews.iter_mut().find(|r| r.cut == review.cut) {
                        *old = review;
                    } else {
                        self.reviews.push(review);
                    }
                }
                self.keys = self
                    .reviews
                    .iter()
                    .flat_map(|r| &r.cues)
                    .filter(|c| {
                        self.options.only.is_empty() || self.options.only.contains(&c.line_id)
                    })
                    .map(|c| c.key.clone())
                    .collect();
                if first {
                    self.at = self
                        .keys
                        .iter()
                        .position(|k| {
                            self.reviews
                                .iter()
                                .flat_map(|r| &r.cues)
                                .any(|c| &c.key == k && !c.verified)
                        })
                        .unwrap_or(0);
                }
                self.at = self.at.min(self.keys.len().saturating_sub(1));
                self.select(ctx);
            }
        }
    }

    fn select(&mut self, ctx: &egui::Context) {
        self.player = None;
        self.preview = None;
        self.played = false;
        let Some(cue) = self.cue().cloned() else {
            self.note = "This selection has no emphasis cues.".into();
            return;
        };
        self.start = cue
            .interval
            .map_or(0.0, |i| i.start as f64 / f64::from(cue.binding.rate));
        self.end = cue.interval.map_or(cue.binding.samples, |i| i.end) as f64
            / f64::from(cue.binding.rate);
        let preview = self
            .reviews
            .iter()
            .find_map(|r| r.previews.get(&cue.line_id))
            .cloned();
        let current: Vec<_> = self
            .reviews
            .iter()
            .flat_map(|r| r.cues.iter().cloned())
            .collect();
        if let Some(preview) = preview.filter(|p| {
            p.current(&current) && {
                p.cues.iter().any(|c| {
                    c.key == cue.key && c.binding == cue.binding && c.interval == cue.interval
                })
            }
        }) {
            if vidni_core::release::file_hash(&preview.path)
                .ok()
                .as_deref()
                != Some(&preview.sha256)
            {
                self.note = "The preview file changed; rebuild it before reviewing.".into();
                return;
            }
            let repaint = ctx.clone();
            match Player::open(&preview.path, Vec::new(), move || repaint.request_repaint()) {
                Ok(player) => {
                    self.player = Some(player);
                    self.preview = Some(preview);
                    self.note = "Play the phrase and full highlight hold, then verify.".into();
                }
                Err(e) => self.note = e,
            }
        } else {
            self.refresh(ctx, true);
        }
    }

    fn interval(&self, cue: &Cue) -> Interval {
        Interval {
            start: (self.start * f64::from(cue.binding.rate)).round() as u64,
            end: (self.end * f64::from(cue.binding.rate)).round() as u64,
        }
    }

    fn verify(&mut self, cue: &Cue, interval: Interval) -> Result<(), String> {
        let preview = self
            .preview
            .as_ref()
            .ok_or("Rebuild the preview before verifying.")?;
        if !self.played
            || vidni_core::release::file_hash(&preview.path)? != preview.sha256
            || !preview.cues.iter().any(|shown| {
                shown.key == cue.key
                    && shown.binding == cue.binding
                    && shown.interval == Some(interval)
            })
        {
            return Err("Play a current preview of these boundaries before verifying.".into());
        }
        cues::verify(&self.options, cue, interval, preview)
    }

    fn sample_here(&self, cue: &Cue) -> f64 {
        self.player
            .as_ref()
            .zip(self.preview.as_ref())
            .map_or(0.0, |(player, preview)| {
                ((preview.first_frame + player.position) as f64 / 30.0 - cue.line_start)
                    .max(0.0)
                    .min(cue.binding.samples as f64 / f64::from(cue.binding.rate))
            })
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        self.poll(&ctx);
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Review narration highlights");
            ui.label("Verified timing belongs to this exact audio take and layout.");
            ui.label(&self.note);
            if self.job.is_some() {
                ui.spinner();
                return;
            }
            let Some(cue) = self.cue().cloned() else { return };
            ui.horizontal_wrapped(|ui| {
                if ui.add_enabled(self.at > 0, egui::Button::new("Previous cue")).clicked() {
                    self.at -= 1;
                    self.select(&ctx);
                }
                ui.label(format!("{} / {} · {}", self.at + 1, self.keys.len(), cue.key));
                if ui.add_enabled(self.at + 1 < self.keys.len(), egui::Button::new("Next cue")).clicked() {
                    self.at += 1;
                    self.select(&ctx);
                }
            });
            ui.label(&cue.spoken);
            ui.strong(format!("Emphasis phrase: {} · {}", cue.phrase,
                if cue.verified { "verified" } else { "unverified" }));
            if let Some(display) = &cue.display {
                ui.label(format!("Visible for {:.2} seconds · cut {:.2}–{:.2}s{}", (display.end-display.start) as f64/30.0, display.start as f64/30.0,display.end as f64/30.0,
                    display.cutoff.as_ref().map_or(String::new(),|why|format!(" · ends early: {why}"))));
            }
            let here = self.sample_here(&cue);
            let duration = cue.binding.samples as f64 / f64::from(cue.binding.rate);
            ui.horizontal_wrapped(|ui| {
                let start_label = ui.label("Spoken start (seconds)");
                ui.add(egui::DragValue::new(&mut self.start).speed(1.0 / 30.0)
                    .range(0.0..=duration).max_decimals(3)).labelled_by(start_label.id);
                if ui.add_enabled(self.player.is_some(), egui::Button::new("Start here")).clicked() {
                    self.start = here;
                }
                let end_label = ui.label("Spoken end (seconds)");
                ui.add(egui::DragValue::new(&mut self.end).speed(1.0 / 30.0)
                    .range(0.0..=duration).max_decimals(3)).labelled_by(end_label.id);
                if ui.add_enabled(self.player.is_some(), egui::Button::new("End here")).clicked() {
                    self.end = here;
                }
            });
            let interval = self.interval(&cue);
            let changed = cue.interval != Some(interval);
            ui.horizontal_wrapped(|ui| {
                if ui.add_enabled(changed, egui::Button::new("Save timing and rebuild preview")).clicked() {
                    match cues::adjust(&self.options, &cue, interval) {
                        Ok(()) => self.refresh(&ctx, true),
                        Err(e) => self.note = e,
                    }
                }
                let can_verify = !changed && self.played && !cue.verified && self.preview.is_some();
                if ui.add_enabled(can_verify, egui::Button::new("Verify this cue")).clicked() {
                    match self.verify(&cue, interval) {
                        Ok(()) => {
                            for c in self.reviews.iter_mut().flat_map(|r| &mut r.cues).filter(|c| c.key == cue.key) {
                                c.verified = true;
                                c.problem = None;
                            }
                            self.note = "Verified for this take and layout.".into();
                        }
                        Err(e) => self.note = e,
                    }
                }
                ui.checkbox(&mut self.looping, "Loop phrase and hold");
            });
            if let Some(player) = &mut self.player {
                ui.horizontal(|ui| {
                    if ui.button("−1 frame").clicked() {
                        player.seek(player.position.saturating_sub(1));
                    }
                    if ui.button("+1 frame").clicked() {
                        player.seek(player.position + 1);
                    }
                });
                let playing = player.playing();
                player.ui(ui);
                if player.playing() && player.has_audio() {
                    self.played = true;
                }
                if player.playing() && !player.has_audio() {
                    self.note = "Audio playback is unavailable. Run cue review with ffplay and audio enabled before verifying.".into();
                }
                if self.looping && playing && !player.playing() && player.position + 2 >= player.video.frames {
                    player.seek(0);
                    player.play();
                }
            }
        });
    }
}

impl eframe::App for Reviewer {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::{
        Harness,
        kittest::{NodeT, Queryable},
    };
    use vidni_core::{
        cues::Binding,
        release::{self, Voice},
    };

    fn reviewer(path: &std::path::Path) -> Reviewer {
        let cue = Cue {
            key: "long/draft/parcel/capital".into(),
            line: 0,
            line_id: "parcel".into(),
            phrase: "capital P".into(),
            occurrence: None,
            spoken: "A capital P names the parcel.".into(),
            audio: path.join("audio.wav"),
            line_start: 0.6,
            binding: Binding {
                audio: "audio".into(),
                rate: 48000,
                samples: 96000,
                words: "words".into(),
                definition: "cue".into(),
                visual: "picture".into(),
            },
            boxes: vec![[100.0, 100.0, 30.0, 50.0]],
            interval: Some(Interval {
                start: 12000,
                end: 60000,
            }),
            display: None,
            verified: false,
            problem: Some("unverified".into()),
        };
        Reviewer {
            options: Options {
                path: path.to_path_buf(),
                cuts: vec![],
                only: vec![],
                voice: cues::Audio::Draft,
            },
            keys: vec![cue.key.clone()],
            reviews: vec![Review {
                schema: 1,
                cut: "long".into(),
                voice: Voice::Draft,
                cues: vec![cue],
                previews: Default::default(),
            }],
            at: 0,
            start: 0.25,
            end: 1.25,
            player: None,
            preview: None,
            job: None,
            note: String::new(),
            looping: false,
            played: false,
        }
    }

    #[test]
    fn review_controls_cannot_verify_without_a_played_preview() {
        let dir = tempfile::tempdir().unwrap();
        let mut window = Harness::builder().with_size([900.0, 700.0]).build_ui_state(
            |ui, state: &mut Reviewer| state.show(ui),
            reviewer(dir.path()),
        );
        window.run();
        assert!(
            window
                .get_by_label("Verify this cue")
                .accesskit_node()
                .is_disabled()
        );
        assert!(
            window
                .get_by_label("Start here")
                .accesskit_node()
                .is_disabled()
        );
        window.get_by_label("Loop phrase and hold").click();
        window.run();
        assert!(window.state().looping);
        assert!(!dir.path().join(vidni_core::cues::LOCK).exists());
    }

    #[test]
    fn changing_a_preview_after_playback_cannot_verify_it() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("preview.mp4");
        std::fs::write(&path, b"the preview originally reviewed").unwrap();
        let mut reviewer = reviewer(dir.path());
        let cue = reviewer.cue().unwrap().clone();
        reviewer.played = true;
        reviewer.preview = Some(vidni_media::cues::Preview {
            line_id: cue.line_id.clone(),
            path: path.clone(),
            sha256: release::file_hash(&path).unwrap(),
            first_frame: 0,
            end_frame: 90,
            inputs: "test".into(),
            cues: vec![cue.clone()],
            context: vec![],
        });
        std::fs::write(&path, b"different preview bytes").unwrap();
        assert!(
            reviewer
                .verify(&cue, cue.interval.unwrap())
                .unwrap_err()
                .contains("current preview")
        );
        assert!(!dir.path().join(vidni_core::cues::LOCK).exists());
    }
}
