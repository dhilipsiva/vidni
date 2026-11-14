//! The vidni desktop app: `vidni_app`, in a window.

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("vidni")
            .with_inner_size([960.0, 720.0]),
        ..Default::default()
    };
    eframe::run_native(
        "vidni",
        options,
        Box::new(|_| {
            Ok(Box::new(vidni_app::app::App::new(
                vidni_app::recent::folder(),
            )))
        }),
    )
}
