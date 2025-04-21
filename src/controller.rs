use log;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use thiserror::Error;

use crate::fs_util;
use crate::model::AppState;
use crate::rsm::save_bundle::SaveBundle;

#[derive(Debug, Error)]
pub enum ControllerError {
    #[error("Backup failed: {0}")]
    BackupFailed(String),

    #[error("Delete backup failed: {0}")]
    DeleteBackupFailed(String),

    #[error("Restore backup failed: {0}")]
    RestoreBackupFailed(String),
}

type ControllerResult<T> = Result<T, ControllerError>;

#[derive(Debug)]
pub enum AppEvent {
    // Directory operations
    UpdateSaveDirectory(String),
    UpdateBackupDirectory(String),
    RefreshSaves,

    // Save management
    BackupSave(String),
    RequestRestoreBackup(String),
    ConfirmRestoreBackup(String),
    CancelRestoreBackup,

    // Backup management
    RequestDeleteBackup(String),
    ConfirmDeleteBackup(String),
    CancelDeleteBackup,

    // App lifecycle
    Exit,
}

pub struct AppController {
    state: Arc<Mutex<AppState>>,
    event_receiver: Receiver<AppEvent>,
}

impl AppController {
    pub fn new(state: Arc<Mutex<AppState>>, event_receiver: Receiver<AppEvent>) -> Self {
        AppController {
            state,
            event_receiver,
        }
    }

    pub fn handle_events(&self) {
        log::info!("Controller event loop started");

        while let Ok(event) = self.event_receiver.recv() {
            log::debug!("Received event: {:?}", event);

            let mut event_result: Option<ControllerResult<()>> = None;
            match event {
                AppEvent::UpdateSaveDirectory(dir) => {
                    if let Ok(mut state) = self.state.lock() {
                        state.update_save_directory(dir);
                    }
                }

                AppEvent::UpdateBackupDirectory(dir) => {
                    if let Ok(mut state) = self.state.lock() {
                        state.update_backup_directory(dir);
                    }
                }

                AppEvent::RefreshSaves => {
                    if let Ok(mut state) = self.state.lock() {
                        state.refresh_save_bundles();
                    }
                }

                AppEvent::BackupSave(name) => {
                    event_result = Some(self.on_backup_save(name));
                }

                AppEvent::RequestRestoreBackup(name) => {
                    event_result = Some(self.on_request_restore_backup(name));
                }

                AppEvent::ConfirmRestoreBackup(name) => {
                    event_result = Some(self.on_confirm_restore_backup(name));
                }

                AppEvent::CancelRestoreBackup => {
                    if let Ok(mut state) = self.state.lock() {
                        state.clear_restore_confirmation();
                    }
                }

                AppEvent::RequestDeleteBackup(name) => {
                    if let Ok(mut state) = self.state.lock() {
                        state.confirm_delete_backup(name);
                    }
                }

                AppEvent::ConfirmDeleteBackup(name) => {
                    event_result = Some(self.on_confirm_delete_backup(name));
                }

                AppEvent::CancelDeleteBackup => {
                    if let Ok(mut state) = self.state.lock() {
                        state.clear_delete_confirmation();
                    }
                }

                AppEvent::Exit => {
                    log::info!("Exit requested");
                    break;
                }
            }

            if let Some(result) = event_result {
                if let Err(err) = result {
                    log::error!("Error occurred: {}", err);
                }
            }
        }

        log::info!("Controller event loop terminated");
    }

    fn on_confirm_delete_backup(&self, name: String) -> ControllerResult<()> {
        if let Ok(mut state) = self.state.lock() {
            let backup_bundle = get_backup_save_bundle(&state, &name)
                .map_err(|err_msg| ControllerError::DeleteBackupFailed(err_msg))?;
            std::fs::remove_dir_all(&backup_bundle.location)
                .map_err(|e| ControllerError::DeleteBackupFailed(e.to_string()))?;
            state.confirm_backup_deletion_name = None;
            state.refresh_save_bundles();
        }
        Ok(())
    }

    fn on_backup_save(&self, name: String) -> ControllerResult<()> {
        if let Ok(mut state) = self.state.lock() {
            let save_bundle = get_game_save_bundle(&state, &name)
                .map_err(|err_msg| ControllerError::BackupFailed(err_msg))?;
            fs_util::copy_directory(&save_bundle.location, &state.backup_directory, true)
                .map_err(|e| ControllerError::BackupFailed(e.to_string()))?;
            state.refresh_save_bundles();
        }
        Ok(())
    }

    fn on_request_restore_backup(&self, name: String) -> ControllerResult<()> {
        if let Ok(mut state) = self.state.lock() {
            let backup_bundle = get_backup_save_bundle(&state, &name)
                .map_err(|err_msg| ControllerError::RestoreBackupFailed(err_msg))?;

            let res =
                fs_util::copy_directory(&backup_bundle.location, &state.save_directory, false);
            if let Err(err) = res {
                match err {
                    fs_util::SaveManagerError::SaveExists => {
                        state.confirm_restore_backup_name = Some(name);
                    }
                    error => {
                        return Err(ControllerError::RestoreBackupFailed(error.to_string()));
                    }
                }
            }
            state.refresh_save_bundles();
        }
        Ok(())
    }

    fn on_confirm_restore_backup(&self, name: String) -> ControllerResult<()> {
        if let Ok(mut state) = self.state.lock() {
            let backup_bundle = get_backup_save_bundle(&state, &name)
                .map_err(|err_msg| ControllerError::RestoreBackupFailed(err_msg))?;

            let res = fs_util::copy_directory(&backup_bundle.location, &state.save_directory, true);
            if let Err(err) = res {
                return Err(ControllerError::RestoreBackupFailed(err.to_string()));
            }
            state.confirm_restore_backup_name = None;
            state.refresh_save_bundles();
        }
        Ok(())
    }
}

// == Helper functions == //

/// Extract backup save bundle from state, returns Err with message if not found.
fn get_backup_save_bundle<'a>(
    state: &'a AppState,
    name: &String,
) -> Result<&'a SaveBundle, String> {
    let backup_bundle = state
        .backup_save_bundles
        .iter()
        .find(|s| s.name == *name)
        .ok_or(format!("Backup Save bundle with name `{name}` not found"))?;
    Ok(backup_bundle)
}

/// Extract game save bundle from state, returns Err with message if not found.
fn get_game_save_bundle<'a>(state: &'a AppState, name: &String) -> Result<&'a SaveBundle, String> {
    let game_save_bundle = state
        .game_save_bundles
        .iter()
        .find(|s| s.name == *name)
        .ok_or(format!("Game Save bundle with name `{name}` not found"))?;
    Ok(game_save_bundle)
}
