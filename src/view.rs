use eframe::egui;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use crate::controller::AppEvent;
use crate::model::AppState;
use crate::rsm;

// Main view struct
pub struct AppView {
    pub state: Arc<Mutex<AppState>>,
    event_sender: Sender<AppEvent>,
    had_focus: bool,
}

#[derive(Clone, PartialEq, Deserialize, Serialize, Debug)]
pub enum SaveDirType {
    GameSave,
    BackupSave,
}

impl AppView {
    pub fn new(state: Arc<Mutex<AppState>>, event_sender: Sender<AppEvent>) -> Self {
        AppView {
            state,
            event_sender,
            had_focus: false,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        self.handle_focus(ctx);
        self.ui_top_panel(ctx);
        self.ui_central_panel(ctx);
    }

    fn handle_focus(&mut self, ctx: &egui::Context) {
        let has_focus = ctx.input(|i| i.focused);
        if has_focus && !self.had_focus {
            self.on_regain_focus();
        }
        self.had_focus = has_focus;
    }

    fn on_regain_focus(&mut self) {
        self.event_sender.send(AppEvent::RefreshSaves).unwrap();
    }

    fn ui_top_panel(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        let _ = self.event_sender.send(AppEvent::Exit);
                    }
                });
                ui.add_space(16.0);

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });
    }

    fn ui_central_panel(&self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.cmp_central_file_panel(ui);
            ui.separator();
            self.cmp_central_sync_panel(ui);
        });
    }

    fn cmp_central_file_panel(&self, ui: &mut egui::Ui) {
        ui.heading("R.E.P.O. Save Manager");

        let state_lock = self.state.lock().unwrap();
        let mut save_directory = state_lock.save_directory.clone();
        let mut backup_directory = state_lock.backup_directory.clone();
        drop(state_lock); // Release the lock before UI interactions

        ui.horizontal(|ui| {
            ui.label("Game Save Directory");
            ui.text_edit_singleline(&mut save_directory);
            if ui.button("Browse...").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    save_directory = path.display().to_string();
                    self.event_sender
                        .send(AppEvent::UpdateSaveDirectory(save_directory))
                        .unwrap();
                }
            }
        });

        ui.horizontal(|ui| {
            ui.label("Backup Directory");
            ui.text_edit_singleline(&mut backup_directory);
            if ui.button("Browse...").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    backup_directory = path.display().to_string();
                    self.event_sender
                        .send(AppEvent::UpdateBackupDirectory(backup_directory))
                        .unwrap();
                }
            }
        });

        if ui.button("Refresh Saves").clicked() {
            self.event_sender.send(AppEvent::RefreshSaves).unwrap();
        }
    }

    fn cmp_central_sync_panel(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.set_width(ui.available_width());

            // Save Panel (Left)
            ui.vertical(|ui| {
                ui.set_width(ui.available_width() / 2.0);
                self.cmp_save_panel(ui, &SaveDirType::GameSave);
            });

            // Backup Panel (Right)
            ui.vertical(|ui| {
                ui.set_width(ui.available_width());
                self.cmp_save_panel(ui, &SaveDirType::BackupSave);
            });
        });

        self.cmp_restore_confirmation_modal(ui);
        self.cmp_delete_confirmation_modal(ui);
    }

    fn cmp_save_panel(&self, ui: &mut egui::Ui, save_type: &SaveDirType) {
        let state_guard = self.state.lock().unwrap();

        ui.vertical(|ui| {
            ui.set_width(ui.available_width());
            let name = match save_type {
                SaveDirType::GameSave => "Game Saves",
                SaveDirType::BackupSave => "Backup Saves",
            };
            ui.label(egui::RichText::new(name).size(16.0));
            ui.add_space(16.0);

            let save_bundles = match save_type {
                SaveDirType::GameSave => state_guard.game_save_bundles.clone(),
                SaveDirType::BackupSave => state_guard.backup_save_bundles.clone(),
            };
            drop(state_guard);

            for save_bundle in save_bundles {
                self.cmp_save_bundle_container(ui, save_type, save_bundle);
                ui.add_space(8.0);
            }
        });
    }

    fn cmp_save_bundle_container(
        &self,
        ui: &mut egui::Ui,
        save_type: &SaveDirType,
        save_bundle: rsm::save_bundle::SaveBundle,
    ) {
        let response = ui.response();
        let visuals = ui.style().interact(&response);

        egui::Frame::canvas(ui.style())
            .fill(visuals.bg_fill.gamma_multiply(0.3))
            .stroke(visuals.bg_stroke)
            .show(ui, |ui| {
                egui::Grid::new(format!("save_grid_{}_{:?}", save_bundle.name, save_type))
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        self.cmp_save_bundle_container_grid_contents(ui, save_type, &save_bundle);
                    });
            });
    }

    fn cmp_save_bundle_container_grid_contents(
        &self,
        ui: &mut egui::Ui,
        save_type: &SaveDirType,
        save_bundle: &rsm::save_bundle::SaveBundle,
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
        match save_type {
            SaveDirType::GameSave => {
                if ui.button("Backup").clicked() {
                    self.event_sender
                        .send(AppEvent::BackupSave(save_bundle.name.clone()))
                        .unwrap();
                }
            }
            SaveDirType::BackupSave => {
                ui.horizontal(|ui| {
                    if ui.button("Restore").clicked() {
                        self.event_sender
                            .send(AppEvent::RequestRestoreBackup(save_bundle.name.clone()))
                            .unwrap();
                    }

                    if ui
                        .button(egui::RichText::new("Delete").color(egui::Color32::RED))
                        .clicked()
                    {
                        self.event_sender
                            .send(AppEvent::RequestDeleteBackup(save_bundle.name.clone()))
                            .unwrap();
                    }
                });
            }
        }
    }

    fn cmp_restore_confirmation_modal(&self, ui: &mut egui::Ui) {
        let state_guard = self.state.lock().unwrap();

        if let Some(backup_name) = &state_guard.confirm_restore_backup_name {
            let backup_name = backup_name.clone();

            let backup_save_bundle = state_guard
                .backup_save_bundles
                .iter()
                .find(|s| &s.name == &backup_name)
                .cloned();

            let target_save_bundle = state_guard
                .game_save_bundles
                .iter()
                .find(|s| &s.name == &backup_name)
                .cloned();

            // Drop the lock before showing the modal
            drop(state_guard);

            if let (Some(backup_save_bundle), Some(target_save_bundle)) =
                (backup_save_bundle, target_save_bundle)
            {
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
                                self.event_sender
                                    .send(AppEvent::ConfirmRestoreBackup(backup_name.clone()))
                                    .unwrap();
                            }

                            if ui.button("No").clicked() {
                                self.event_sender
                                    .send(AppEvent::CancelRestoreBackup)
                                    .unwrap();
                            }
                        },
                    );
                });

                if modal.should_close() {
                    self.event_sender
                        .send(AppEvent::CancelRestoreBackup)
                        .unwrap();
                }
            }
        }
    }

    fn cmp_delete_confirmation_modal(&self, ui: &mut egui::Ui) {
        let state_guard = self.state.lock().unwrap();

        if let Some(backup_name) = &state_guard.confirm_backup_deletion_name {
            let backup_name = backup_name.clone();

            let backup_save_bundle = state_guard
                .backup_save_bundles
                .iter()
                .find(|s| &s.name == &backup_name)
                .cloned();

            // Drop the lock before showing the modal
            drop(state_guard);

            if let Some(backup_save_bundle) = backup_save_bundle {
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
                                    self.event_sender
                                        .send(AppEvent::ConfirmDeleteBackup(backup_name.clone()))
                                        .unwrap();
                                }

                                if ui.button("No").clicked() {
                                    self.event_sender
                                        .send(AppEvent::CancelDeleteBackup)
                                        .unwrap();
                                }
                            },
                        );
                    });

                if modal.should_close() {
                    self.event_sender
                        .send(AppEvent::CancelDeleteBackup)
                        .unwrap();
                }
            }
        }
    }
}
