mod app;
mod core;
mod ui;

use app::GateApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0]) // 16:9 aspect ratio
            .with_title("Gate"),
        ..Default::default()
    };
    eframe::run_native(
        "Gate",
        native_options,
        Box::new(|cc| Ok(Box::new(GateApp::new(cc)))),
    )
}
