use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

use crate::{constant, fs_util, repo, rsm};

#[derive(Error, Debug)]
pub enum RSMError {}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct AppState {
    pub save_directory: String,
    pub backup_directory: String,

    pub game_save_bundles: Vec<rsm::save_bundle::SaveBundle>,
    pub backup_save_bundles: Vec<rsm::save_bundle::SaveBundle>,

    /// If not None, contains the name of a backup to restore.
    /// When not None, this triggers a popup to restore backup with overwrite power.
    #[serde(skip)]
    pub confirm_restore_backup_name: Option<String>,

    /// If not None, contains the name of a backup to delete.
    /// When not None, this triggers a popup to delete a backup.
    #[serde(skip)]
    pub confirm_backup_deletion_name: Option<String>,
}

impl Default for AppState {
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
            save_directory,
            backup_directory,
            game_save_bundles: Vec::new(),
            backup_save_bundles: Vec::new(),
            confirm_restore_backup_name: None,
            confirm_backup_deletion_name: None,
        }
    }
}

impl AppState {
    pub fn refresh_save_bundles(&mut self) {
        self.game_save_bundles = rsm::save_bundle::extract_save_bundles(&self.save_directory);
        self.backup_save_bundles = rsm::save_bundle::extract_save_bundles(&self.backup_directory);
    }

    pub fn update_save_directory(&mut self, new_directory: String) {
        self.save_directory = new_directory;
        self.refresh_save_bundles();
    }

    pub fn update_backup_directory(&mut self, new_directory: String) {
        self.backup_directory = new_directory;
        self.refresh_save_bundles();
    }

    pub fn confirm_restore_backup(&mut self, backup_name: String) {
        self.confirm_restore_backup_name = Some(backup_name);
    }

    pub fn confirm_delete_backup(&mut self, backup_name: String) {
        self.confirm_backup_deletion_name = Some(backup_name);
    }

    pub fn clear_restore_confirmation(&mut self) {
        self.confirm_restore_backup_name = None;
    }

    pub fn clear_delete_confirmation(&mut self) {
        self.confirm_backup_deletion_name = None;
    }
}
