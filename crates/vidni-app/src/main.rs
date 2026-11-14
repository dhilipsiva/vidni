//! The vidni desktop app: `vidni_app`, in a window.
//!
//! `vidni-app [PROJECT] [--script]` opens the project in that folder, on
//! its cuts or on its script; with no project, the app opens on the list.

use vidni_app::app::{App, Tab};

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let project = args.iter().find(|arg| !arg.starts_with("--")).cloned();
    let script = args.iter().any(|arg| arg == "--script");
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
                if script {
                    app.tab = Tab::Script;
                }
            }
            Ok(Box::new(app))
        }),
    )
}
