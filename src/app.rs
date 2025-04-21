use crate::{model, view};
// App for egui integration
pub struct RSMApp {
    view: view::AppView,
}

impl RSMApp {
    pub fn new(cc: &eframe::CreationContext<'_>, view: view::AppView) -> Self {
        // Load previous app state if persistence is enabled
        if let Some(storage) = cc.storage {
            if let Some(state) = eframe::get_value::<model::AppState>(storage, eframe::APP_KEY) {
                if let Ok(mut current_state) = view.state.lock() {
                    // Only restore persistent fields
                    current_state.save_directory = state.save_directory;
                    current_state.backup_directory = state.backup_directory;

                    // Refresh save bundles based on loaded directories
                    current_state.refresh_save_bundles();
                }
            }
        }

        Self { view }
    }
}

impl eframe::App for RSMApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Ok(state) = self.view.state.lock() {
            eframe::set_value(storage, eframe::APP_KEY, &*state);
        }
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.view.update(ctx);
    }
}
