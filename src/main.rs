mod app;
mod models;
mod storage;
mod theme;
mod widgets;

use app::CrudApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Rust Control")
            .with_inner_size([1360.0, 860.0])
            .with_min_inner_size([640.0, 560.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Rust Control",
        options,
        Box::new(|cc| Ok(Box::new(CrudApp::new(cc)))),
    )
}
