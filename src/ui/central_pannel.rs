use crate::app::RSMApp;
use crate::{fs_util, save_bundle};
use egui;

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
pub enum SaveDirType {
    GameSave,
    BackupSave,
}

impl RSMApp {
    pub fn ui_central_pannel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.cmp_central_file_pannel(ui);

            ui.separator();

            self.cmp_central_sync_pannel(ui);
        });
    }

    /// The file pannel contains the save directory locations.
    pub fn cmp_central_file_pannel(&mut self, ui: &mut egui::Ui) {
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
    }

    /// The sync pannel is composed of a left (GameSave) and right (BackupSave)
    /// sub-pannels. This allows easy backup and restore operations.
    pub fn cmp_central_sync_pannel(&mut self, ui: &mut egui::Ui) {
        // Sync Pannel
        ui.horizontal(|ui| {
            ui.set_width(ui.available_width());
            // Save Pannel (Left)
            ui.vertical(|ui| {
                ui.set_width(ui.available_width() / 2.0);
                self.cmp_save_panel(ui, &SaveDirType::GameSave);
            });

            // Backup Pannel (Right)
            ui.vertical(|ui| {
                ui.set_width(ui.available_width());
                self.cmp_save_panel(ui, &SaveDirType::BackupSave);
            });
        });

        self.cmp_restore_confirmation_modal(ui);
        self.cmp_delete_confirmation_modal(ui);
    }

    pub fn cmp_save_panel(&mut self, ui: &mut egui::Ui, save_type: &SaveDirType) {
        ui.vertical(|ui| {
            ui.set_width(ui.available_width());
            let name = match save_type {
                SaveDirType::GameSave => "Game Saves",
                SaveDirType::BackupSave => "Backup Saves",
            };
            ui.label(egui::RichText::new(name).size(16.0));
            ui.add_space(16.0);

            let save_bundles = match save_type {
                SaveDirType::GameSave => self.game_save_bundles.clone(),
                SaveDirType::BackupSave => self.backup_save_bundles.clone(),
            };

            for save_bundle in save_bundles {
                self.cmp_save_bundle_container(ui, save_type, save_bundle);
                ui.add_space(8.0);
            }
        });
    }

    pub fn cmp_save_bundle_container(
        &mut self,
        ui: &mut egui::Ui,
        save_type: &SaveDirType,
        save_bundle: save_bundle::SaveBundle,
    ) {
        let response = ui.response();
        let visuals = ui.style().interact(&response);
        // frame creates the borders around each save
        egui::Frame::canvas(ui.style())
            .fill(visuals.bg_fill.gamma_multiply(0.3))
            .stroke(visuals.bg_stroke)
            .show(ui, |ui| {
                // grid creates rows of data displayed in each save
                egui::Grid::new(format!("save_grid_{}_{:?}", save_bundle.name, save_type))
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        self.cmp_save_bundle_container_grid_contents(ui, save_type, &save_bundle);
                    });
            });
    }

    pub fn cmp_save_bundle_container_grid_contents(
        &mut self,
        ui: &mut egui::Ui,
        save_type: &SaveDirType,
        save_bundle: &save_bundle::SaveBundle,
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
                if ui.button("Backup").clicked() {
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
                ui.horizontal(|ui| {
                    if ui.button("Restore").clicked() {
                        // try and restore but don't overwrite
                        let res = fs_util::copy_directory(
                            &save_bundle.location,
                            &self.save_directory,
                            false,
                        );
                        if let Err(err) = res {
                            match err {
                                // if save exists, it will return Err, catch it and open the confirmation modal
                                fs_util::SaveManagerError::SaveExists => {
                                    self.confirm_retore_backup_name =
                                        Some(save_bundle.name.clone());
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

                    if ui
                        .button(egui::RichText::new("Delete").color(egui::Color32::RED))
                        .clicked()
                    {
                        self.confirm_backup_deletion_name = Some(save_bundle.name.clone());
                    }
                });
            }
        }
    }

    /// confirmation modal, activated when a restoring a backup would overwrite a save
    pub fn cmp_restore_confirmation_modal(&mut self, ui: &mut egui::Ui) {
        if let Some(backup_name) = &self.confirm_retore_backup_name {
            let backup_save_bundle = &self
                .backup_save_bundles
                .iter()
                .find(|s| &s.name == backup_name)
                .unwrap()
                .clone();
            let target_save_bundle = &self
                .game_save_bundles
                .iter()
                .find(|s| &s.name == backup_name)
                .unwrap()
                .clone();

            let modal = egui::Modal::new(egui::Id::new("restore_modal")).show(ui.ctx(), |ui| {
                ui.set_width(400.0);
                ui.heading("Warning!");
                ui.label(format!(
                    "Save will be overwritten {}.",
                    &target_save_bundle.name
                ));
                ui.label(format!(
                    "Level: {} ->  {}",
                    &target_save_bundle.level, &backup_save_bundle.level
                ));
                ui.label("Do you want to proceed?");
                ui.label("This action cannot be undone.");

                ui.add_space(32.0);

                egui::Sides::new().show(
                    ui,
                    |_ui| {},
                    |ui| {
                        if ui.button("Yes").clicked() {
                            let res = fs_util::copy_directory(
                                &backup_save_bundle.location,
                                &self.save_directory,
                                true,
                            );
                            self.refresh_save_bundles();
                            match res {
                                Ok(_) => println!("Save restored successfully"),
                                Err(e) => log::error!("Error restoring save: {}", e),
                            }
                            self.confirm_retore_backup_name = None;
                        }

                        if ui.button("No").clicked() {
                            self.confirm_retore_backup_name = None;
                        }
                    },
                );
            });

            if modal.should_close() {
                self.confirm_retore_backup_name = None;
            }
        }
    }

    pub fn cmp_delete_confirmation_modal(&mut self, ui: &mut egui::Ui) {
        if let Some(backup_name) = &self.confirm_backup_deletion_name {
            let backup_save_bundle = &self
                .backup_save_bundles
                .iter()
                .find(|s| &s.name == backup_name)
                .unwrap()
                .clone();

            let modal =
                egui::Modal::new(egui::Id::new("backup_delete_modal")).show(ui.ctx(), |ui| {
                    ui.set_width(400.0);
                    ui.heading("Warning!");
                    ui.label(format!(
                        "Backup will be deleted {}.",
                        &backup_save_bundle.name
                    ));
                    ui.label("Do you want to proceed?");
                    ui.label("This action cannot be undone.");

                    ui.add_space(32.0);

                    egui::Sides::new().show(
                        ui,
                        |_ui| {},
                        |ui| {
                            if ui
                                .button(egui::RichText::new("Yes").color(egui::Color32::RED))
                                .clicked()
                            {
                                // Delete the backup save directory
                                if let Err(e) =
                                    std::fs::remove_dir_all(&backup_save_bundle.location)
                                {
                                    log::error!("Failed to delete backup: {}", e);
                                } else {
                                    log::info!(
                                        "Backup {} deleted successfully",
                                        backup_save_bundle.name
                                    );
                                }
                                self.refresh_save_bundles();
                                self.confirm_backup_deletion_name = None;
                            }

                            if ui.button("No").clicked() {
                                self.confirm_backup_deletion_name = None;
                            }
                        },
                    );
                });

            if modal.should_close() {
                self.confirm_backup_deletion_name = None;
            }
        }
    }
}
