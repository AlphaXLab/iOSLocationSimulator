mod app;
mod daemon;
mod geocoding;
mod models;

use app::LocationSimulatorApp;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Location Simulator")
            .with_inner_size([1180.0, 780.0])
            .with_min_inner_size([900.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Location Simulator",
        native_options,
        Box::new(|cc| Ok(Box::new(LocationSimulatorApp::new(cc)))),
    )
}
