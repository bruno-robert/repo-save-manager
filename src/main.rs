use eframe::egui;

pub mod app;
pub mod constant;
pub mod crypt;
pub mod fs_util;
pub mod repo_save;
pub mod save_bundle;
pub mod ui;

fn main() {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([700.0, 515.0])
            .with_min_inner_size([300.0, 220.0])
            .with_app_id(constant::APP_ID),
        ..Default::default()
    };
    _ = eframe::run_native(
        constant::APP_NAME,
        native_options,
        Box::new(|cc| Ok(Box::new(app::RSMApp::new(cc)))),
    );
}
