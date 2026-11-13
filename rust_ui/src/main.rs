mod app;
pub mod config;
mod daemon;
mod geocoding;
pub mod icons;
mod models;
pub mod theme;
pub mod widgets;

use app::LocationSimulatorApp;

fn main() -> eframe::Result<()> {
    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_title("Location Simulator")
        .with_inner_size([1180.0, 780.0])
        .with_min_inner_size([960.0, 620.0]);

    #[cfg(target_os = "macos")]
    {
        viewport = viewport
            .with_fullsize_content_view(true)
            .with_titlebar_shown(false)
            .with_title_shown(false)
            .with_titlebar_buttons_shown(true);
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Location Simulator",
        native_options,
        Box::new(|cc| Ok(Box::new(LocationSimulatorApp::new(cc)))),
    )
}
