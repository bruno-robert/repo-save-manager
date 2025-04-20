use eframe;
use egui;
use serde;
use std::path::PathBuf;
use thiserror::Error;

use crate::{constant, fs_util, repo, rsm};

#[derive(Error, Debug)]
pub enum RSMError {}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct RSMApp {
    pub save_directory: String,
    pub backup_directory: String,

    pub game_save_bundles: Vec<rsm::save_bundle::SaveBundle>,
    pub backup_save_bundles: Vec<rsm::save_bundle::SaveBundle>,

    /// If not None, contains the name of a backup to restore.
    /// When not None, this triggers a popup to restore backup with overwrite power.
    #[serde(skip)]
    pub confirm_retore_backup_name: Option<String>,

    /// If not None, contains the name of a backup to delete.
    /// When not None, this triggers a popup to delete a backup.
    #[serde(skip)]
    pub confirm_backup_deletion_name: Option<String>,

    /// true if application window had focus last frame
    #[serde(skip)]
    had_focus: bool,
}

impl Default for RSMApp {
    fn default() -> Self {
        // $HOME environment variable path
        let save_directory: String =
            fs_util::first_existing_dir(repo::utils::get_repo_save_paths())
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();

        let project_dir = directories_next::ProjectDirs::from("", "", constant::APP_ID);
        let backup_directory: String;
        if let Some(project_dir) = project_dir {
            backup_directory = project_dir
                .data_dir()
                .join("backups")
                .to_string_lossy()
                .to_string();
        } else {
            let home_path = PathBuf::from(std::env::var("HOME").unwrap_or_default());
            backup_directory = home_path
                .join(".local/share/rsm/backups")
                .to_string_lossy()
                .to_string();
        }

        Self {
            // Example stuff:
            save_directory,
            backup_directory,
            game_save_bundles: Vec::new(),
            backup_save_bundles: Vec::new(),
            confirm_retore_backup_name: None,
            confirm_backup_deletion_name: None,
            had_focus: false,
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
        self.game_save_bundles = rsm::save_bundle::extract_save_bundles(&self.save_directory);
        self.backup_save_bundles = rsm::save_bundle::extract_save_bundles(&self.backup_directory);
    }

    /// Called when application window gains focus
    fn on_regain_focus(&mut self) {
        self.refresh_save_bundles();
    }

    /// Checks if the focus has changed since last frame and calls focus callbacks a required
    fn handle_focus(&mut self, ctx: &egui::Context) {
        let has_focus = ctx.input(|i| i.focused);
        if has_focus && !self.had_focus {
            self.on_regain_focus();
        }
        self.had_focus = has_focus;
    }
}

impl eframe::App for RSMApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_focus(ctx);
        self.ui_top_pannel(ctx);
        self.ui_central_pannel(ctx);
    }
}
