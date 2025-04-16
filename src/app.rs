use eframe;
use egui;
use serde;
use std::path::PathBuf;
use thiserror::Error;

use crate::save_bundle;

#[derive(Error, Debug)]
pub enum RSMError {}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct RSMApp {
    pub save_directory: String,
    pub backup_directory: String,

    pub game_save_bundles: Vec<save_bundle::SaveBundle>,
    pub backup_save_bundles: Vec<save_bundle::SaveBundle>,

    #[serde(skip)]
    pub confirm_retore_backup_name: Option<String>,
}

impl Default for RSMApp {
    fn default() -> Self {
        // $HOME environment variable path
        let home_path = PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let save_directory: PathBuf = home_path
            .join("Documents")
            .join("REPO GAME SAVES")
            .join("saves");

        let backup_directory: PathBuf = home_path.join(".local/share/rsm/backups");

        Self {
            // Example stuff:
            save_directory: save_directory.to_string_lossy().to_string(),
            backup_directory: backup_directory.to_string_lossy().to_string(),
            game_save_bundles: Vec::new(),
            backup_save_bundles: Vec::new(),
            confirm_retore_backup_name: None,
        }
    }
}

impl RSMApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        let mut instance = Self::default();
        instance.refresh_save_bundles();
        instance
    }

    pub fn refresh_save_bundles(&mut self) {
        self.game_save_bundles = save_bundle::extract_save_bundles(&self.save_directory);
        self.backup_save_bundles = save_bundle::extract_save_bundles(&self.backup_directory);
    }
}

impl eframe::App for RSMApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.ui_top_pannel(ctx);
        self.ui_central_pannel(ctx);
    }
}
