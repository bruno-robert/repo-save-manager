use eframe;
use egui::{self, Frame, Modal, RichText};
use log;
use serde;
use std::path::PathBuf;
use thiserror::Error;

use crate::{fs_util, model};

#[derive(Error, Debug)]
pub enum RSMError {}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct RSMApp {
    save_directory: String,
    backup_directory: String,

    game_save_bundles: Vec<model::SaveBundle>,
    backup_save_bundles: Vec<model::SaveBundle>,

    is_restore_model_open: bool,
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
            is_restore_model_open: false,
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

    fn refresh_save_bundles(&mut self) {
        self.game_save_bundles = model::extract_save_bundles(&self.save_directory);
        self.backup_save_bundles = model::extract_save_bundles(&self.backup_directory);
    }

    fn save_panel(&mut self, ui: &mut egui::Ui, save_type: &SaveDirType) {
        ui.vertical(|ui| {
            ui.set_width(ui.available_width());
            let name = match save_type {
                SaveDirType::GameSave => "Game Saves",
                SaveDirType::BackupSave => "Backup Saves",
            };
            ui.label(RichText::new(name).size(16.0));
            ui.add_space(16.0);

            let save_bundles = match save_type {
                SaveDirType::GameSave => self.game_save_bundles.clone(),
                SaveDirType::BackupSave => self.backup_save_bundles.clone(),
            };

            for save_bundle in save_bundles {
                self.save_bundle_container(ui, save_type, save_bundle);
                ui.add_space(8.0);
            }
        });
    }

    fn save_bundle_container(
        &mut self,
        ui: &mut egui::Ui,
        save_type: &SaveDirType,
        save_bundle: model::SaveBundle,
    ) {
        let response = ui.response();
        let visuals = ui.style().interact(&response);
        // frame creates the borders around each save
        Frame::canvas(ui.style())
            .fill(visuals.bg_fill.gamma_multiply(0.3))
            .stroke(visuals.bg_stroke)
            .show(ui, |ui| {
                // grid creates rows of data displayed in each save
                egui::Grid::new(format!("save_grid_{}_{:?}", save_bundle.name, save_type))
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        self.save_bundle_container_grid_contents(ui, save_type, &save_bundle);
                    });
            });
    }

    pub fn save_bundle_container_grid_contents(
        &mut self,
        ui: &mut egui::Ui,
        save_type: &SaveDirType,
        save_bundle: &model::SaveBundle,
    ) {
        ui.label("Name");
        ui.label(&save_bundle.name);

        ui.end_row();

        ui.label("Level");
        ui.label(format!("{}", save_bundle.level + 1));
        ui.end_row();

        ui.label("Players");
        ui.label(save_bundle.players.join("\n"));
        ui.end_row();

        ui.label("Actions");
        match &save_type {
            SaveDirType::GameSave => {
                if ui.button("Backup Save").clicked() {
                    let res = fs_util::copy_directory(
                        &save_bundle.location,
                        &self.backup_directory,
                        true,
                    );
                    match res {
                        Ok(_) => {
                            println!("Save backed up successfully");
                            self.refresh_save_bundles();
                        }
                        Err(err) => {
                            println!("Error backing up save: {}", err);
                        }
                    }
                }
            }
            SaveDirType::BackupSave => {
                if ui.button("Restore Backup").clicked() {
                    let res =
                        fs_util::copy_directory(&save_bundle.location, &self.save_directory, true);
                    if let Err(err) = res {
                        match err {
                            fs_util::SaveManagerError::SaveExists => {
                                self.is_restore_model_open = true;
                            }
                            _ => {
                                log::error!("Error restoring save.");
                            }
                        }
                    } else {
                        println!("Save restored successfully");
                        self.refresh_save_bundles();
                    }
                }
            }
        }

        if self.is_restore_model_open {
            let modal = Modal::new(egui::Id::new(format!(
                "modal{}{:?}",
                &save_bundle.name, &save_type
            )))
            .show(ui.ctx(), |ui| {
                ui.set_width(200.0);
                ui.heading("Warning!");
                ui.label(format!("Backup will overwite save {}.", &save_bundle.name));
                ui.label(format!("save location {:?}.", &save_bundle.location));
                ui.label("Do you want to proceed?");
                ui.label("This action cannot be undone.");

                ui.add_space(32.0);

                egui::Sides::new().show(
                    ui,
                    |_ui| {},
                    |ui| {
                        if ui.button("Yes").clicked() {
                            let rest = fs_util::copy_directory(
                                &save_bundle.location,
                                &self.save_directory,
                                true,
                            );
                            self.refresh_save_bundles();
                            match rest {
                                Ok(_) => println!("Save restored successfully"),
                                Err(e) => log::error!("Error restoring save: {}", e),
                            }
                            self.is_restore_model_open = false;
                        }

                        if ui.button("No").clicked() {
                            self.is_restore_model_open = false;
                        }
                    },
                );
            });

            if modal.should_close() {
                self.is_restore_model_open = false;
            }
        }
    }
}

impl eframe::App for RSMApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("R.E.P.O. Save Manager");

            ui.horizontal(|ui| {
                ui.label("Game Save Directory");
                ui.text_edit_singleline(&mut self.save_directory);
                if ui.button("Browse...").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.save_directory = path.display().to_string();
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Backup Directory");
                ui.text_edit_singleline(&mut self.backup_directory);
                if ui.button("Browse...").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.backup_directory = path.display().to_string();
                    }
                }
            });
            if ui.button("Refresh Saves").clicked() {
                self.refresh_save_bundles();
            }

            ui.separator();

            // Sync Pannel
            ui.horizontal(|ui| {
                ui.set_width(ui.available_width());
                // Save Pannel (Left)
                ui.vertical(|ui| {
                    ui.set_width(ui.available_width() / 2.0);
                    self.save_panel(ui, &SaveDirType::GameSave);
                });

                // Backup Pannel (Right)
                ui.vertical(|ui| {
                    ui.set_width(ui.available_width());
                    self.save_panel(ui, &SaveDirType::BackupSave);
                });
            });
        });
    }
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
pub enum SaveDirType {
    GameSave,
    BackupSave,
}
