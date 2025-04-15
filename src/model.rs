use crate::constant;
use crate::crypt;
use crate::repo_save;
use serde_json;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SaveDirectoryError {
    #[error("Decryption failed: {0}")]
    DecryptError(crypt::DecryptError),
    #[error("JSON error: {0}")]
    JSONError(serde_json::Error),
    #[error("Failed to get directory name")]
    NoFileName,
    #[error("Invalid file name encoding")]
    InvalidFileName,
}

/// Represents a REPO Save directory (official or backup)
/// A save bundle contains a .es3 save file of the same name
/// as the bundle folder and optionally other backups of the .es3 file.
#[derive(Clone, serde::Deserialize, serde::Serialize, PartialEq)]
pub struct SaveBundle {
    /// directory location on disk
    pub location: PathBuf,

    /// optional name (this can be useful to manage multiple
    /// backups), defaults to the official name (location.name)
    pub name: String,
}

impl SaveBundle {
    pub fn new(location: PathBuf) -> Result<Self, SaveDirectoryError> {
        let name = location
            .file_name()
            .ok_or(SaveDirectoryError::NoFileName)?
            .to_str()
            .ok_or(SaveDirectoryError::InvalidFileName)?
            .to_string();
        Ok(SaveBundle { location, name })
    }

    pub fn get_data(&self) -> Result<repo_save::SaveGame, SaveDirectoryError> {
        let mut save_file = self.location.clone();
        save_file.push(format!("{}.es3", self.name));
        let data: Vec<u8> = crypt::decrypt_es3(&save_file, constant::ENCRYPTION_PASS)
            .map_err(|e| SaveDirectoryError::DecryptError(e))?;
        let save_data =
            serde_json::from_slice(&data).map_err(|e| SaveDirectoryError::JSONError(e))?;
        Ok(save_data)
    }
}

/// Given a path to a directory as a string, extract a Vector of
/// SaveDirectory objects.
pub fn extract_save_bundles(path: &str) -> Vec<SaveBundle> {
    let mut save_bundles = Vec::new();
    let entries = match std::fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) => {
            log::error!(e:err; "Error occurred when reading directory.");
            return save_bundles;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                log::error!(e:err; "Error occurred when reading sub-directory.");
                continue;
            }
        };

        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(e) => {
                log::error!(e:err; "Error occurred when determining file-type.");
                continue;
            }
        };

        if !file_type.is_dir() {
            continue;
        }
        let save_bundle = match SaveBundle::new(entry.path()) {
            Ok(sd) => sd,
            Err(e) => {
                log::error!(e:err; "SaveDirectoryError occured.");
                continue;
            }
        };
        save_bundles.push(save_bundle);
    }
    save_bundles
}
