//! The vidni desktop app: `vidni_app`, in a window.
//!
//! `vidni-app [PROJECT] [--script | --preview]` opens the project in that
//! folder, on its cuts, its script or its preview; with no project, the
//! app opens on the list.

use vidni_app::app::{App, Tab};

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--review-cues") {
        let values = |flag: &str| {
            args.windows(2)
                .filter(|a| a[0] == flag)
                .map(|a| a[1].clone())
                .collect::<Vec<_>>()
        };
        let options = vidni_cli::cues::Options {
            path: args
                .first()
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| ".".into()),
            cuts: values("--cut"),
            only: values("--only"),
            voice: if values("--voice").first().is_some_and(|v| v == "takes") {
                vidni_cli::cues::Audio::Takes
            } else {
                vidni_cli::cues::Audio::Draft
            },
        };
        return eframe::run_native(
            "Vidni cue review",
            eframe::NativeOptions {
                viewport: eframe::egui::ViewportBuilder::default().with_inner_size([1100.0, 900.0]),
                ..Default::default()
            },
            Box::new(move |cc| {
                Ok(Box::new(vidni_app::cues::Reviewer::new(
                    options,
                    &cc.egui_ctx,
                )))
            }),
        );
    }
    let project = args.iter().find(|arg| !arg.starts_with("--")).cloned();
    let tab = match args
        .iter()
        .find(|arg| arg.starts_with("--"))
        .map(String::as_str)
    {
        Some("--script") => Tab::Script,
        Some("--preview") => Tab::Preview,
        Some("--record") => Tab::Record,
        Some("--export") => Tab::Export,
        _ => Tab::Cuts,
    };
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("vidni")
            .with_inner_size([960.0, 720.0]),
        ..Default::default()
    };
    eframe::run_native(
        "vidni",
        options,
        Box::new(move |_| {
            let mut app = App::new(vidni_app::recent::folder());
            if let Some(folder) = project {
                app.open(std::path::Path::new(&folder));
                app.tab = tab;
            }
            Ok(Box::new(app))
        }),
    )
}
