//! The vidni desktop app: `vidni_app`, in a window.
//!
//! `vidni-app [PROJECT] [--script | --preview]` opens the project in that
//! folder, on its cuts, its script or its preview; with no project, the
//! app opens on the list.

use vidni_app::app::{App, Tab};

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let project = args.iter().find(|arg| !arg.starts_with("--")).cloned();
    let tab = match args
        .iter()
        .find(|arg| arg.starts_with("--"))
        .map(String::as_str)
    {
        Some("--script") => Tab::Script,
        Some("--preview") => Tab::Preview,
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
