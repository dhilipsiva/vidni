//! The vidni desktop app (egui/eframe), from milestone P7 of `TODO.md`.
//!
//! The app does nothing the command line can't: every action is a `vidni`
//! command, run in this process on a thread of its own ([`jobs`]), so the
//! app and the command line check the same things and say the same words.
//! [`actions`] is the list of them, and the app's help is drawn from it.

pub mod actions;
pub mod app;
pub mod cues;
pub mod editor;
pub mod export;
pub mod jobs;
pub mod preview;
pub mod recent;
pub mod studio;
