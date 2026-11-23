// In release builds, do not open a console next to the window on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod engine;
mod ui;

use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(ui::WINDOW_SIZE)
            .with_min_inner_size([480.0, 560.0])
            // Icon of the title bar and the taskbar. Without it, eframe shows the egui logo.
            .with_icon(
                eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png"))
                    .expect("invalid assets/icon.png"),
            ),
        ..Default::default()
    };

    eframe::run_native(
        "Backups App",
        options,
        Box::new(|cc| {
            ui::apply_theme(&cc.egui_ctx);
            Ok(Box::new(ui::BackupsApp::new()))
        }),
    )
}

/// Creates an empty temporary folder for a test. Each test must use a different `name`.
#[cfg(test)]
fn test_dir(name: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("backups-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}
