use eframe::egui;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

pub mod app;
pub mod constant;
pub mod controller;
pub mod fs_util;
pub mod model;
pub mod repo;
pub mod rsm;
// pub mod ui;
pub mod view;

fn main() {
    // Initialize the model
    let app_state = model::AppState::default();
    let shared_state = Arc::new(Mutex::new(app_state));

    // Set up the communication channel between view and controller
    let (event_sender, event_receiver) = mpsc::channel();

    // Create the controller with the shared state and receiver
    let controller = controller::AppController::new(shared_state.clone(), event_receiver);

    // Start the controller in a separate thread
    _ = thread::spawn(move || {
        controller.handle_events();
    });

    // Configure and run the UI
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([700.0, 515.0])
            .with_min_inner_size([300.0, 220.0])
            .with_app_id(constant::APP_ID),
        ..Default::default()
    };

    // Create the view and run the UI
    let app_view = view::AppView::new(shared_state, event_sender);

    _ = eframe::run_native(
        constant::APP_NAME,
        native_options,
        Box::new(|cc| Ok(Box::new(app::RSMApp::new(cc, app_view)))),
    );

    // If we reach here, the application is closing
    // We don't need to explicitly join the controller thread as it will be terminated when the program exits
}
