use eframe::egui;
use repo_save_manager::{app, constant};

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
