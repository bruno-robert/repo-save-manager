use crate::constant;
use crate::crypt;
use crate::repo_save;
use serde_json;
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SaveBundleError {
    #[error("Decryption failed: {0}")]
    DecryptError(crypt::DecryptError),
    #[error("JSON error: {0}")]
    JSONError(serde_json::Error),
    #[error("Failed to get directory name")]
    NoFileName,
    #[error("Invalid file name")]
    InvalidFileName,
    #[error("Missing file")]
    MissingFile,
    #[error("Expected file")]
    ExpectedFile,
}

/// Represents a REPO Save directory (official or backup)
/// A save bundle contains a .es3 save file of the same name
/// as the bundle folder and optionally other backups of the .es3 file.
#[derive(Clone, serde::Deserialize, serde::Serialize, PartialEq)]
pub struct SaveBundle {
    /// directory location on disk
    pub location: PathBuf,

    /// save name
    pub name: String,
    /// save level
    pub level: i32,
    /// player list
    pub players: Vec<String>,
}

/// A SaveBundle represents how REPO stores a save on the disk.
/// It's a directory with a name like `REPO_SAVE_2025_04_12_15_39_47`,
/// containing a save file with the same name and the extension `es3`.
/// For eg. `REPO_SAVE_2025_04_12_15_39_47.es3`.
///
/// Sometimes, it will also contain backups of the save with names like
/// `REPO_SAVE_2025_04_12_15_39_47_BACKUP1.es3`.
///
/// The backup files are not read.
///
/// ```txt
/// - REPO_SAVE_2025_04_12_15_39_47
/// | -- REPO_SAVE_2025_04_12_15_39_47.es3
/// ```
///
impl SaveBundle {
    /// Initialise a new SaveBundle object from a save file.
    pub fn new(location: impl AsRef<Path>) -> Result<Self, SaveBundleError> {
        let name = location
            .as_ref()
            .file_name()
            .ok_or(SaveBundleError::NoFileName)?
            .to_str()
            .ok_or(SaveBundleError::InvalidFileName)?
            .to_string();
        let save_file = location.as_ref().join(format!("{}.es3", &name));
        let save_data = read_save_file(&save_file)?;
        Ok(SaveBundle {
            location: location.as_ref().to_path_buf(),
            name,
            level: *save_data
                .dictionary_of_dictionaries
                .value
                .run_stats
                .get("level")
                .unwrap_or(&0i32),
            players: save_data.player_names.value.into_values().collect(),
        })
    }

    /// Reads the save file, decrypts it and returns the Deserialised JSON data.
    pub fn get_data(&self) -> Result<repo_save::SaveGame, SaveBundleError> {
        let save_file = self.location.join(format!("{}.es3", self.name));
        let save_data = read_save_file(&save_file)?;
        Ok(save_data)
    }

    /// Refresh the save metadata stored in the struct by re-reading the save file.
    /// This is useful if the save has been updated since last read.
    ///
    /// This method modifies the following fields:
    /// - level
    /// - players
    pub fn refresh_data(&mut self) -> Result<(), SaveBundleError> {
        let save_data = read_save_file(&self.location)?;
        self.level = *save_data
            .dictionary_of_dictionaries
            .value
            .run_stats
            .get("level")
            .unwrap_or(&0i32);
        self.players = save_data.player_names.value.into_values().collect();
        // Sorting the player list makes it consistent in the UI later down the line.
        // The theoretical max len of this list is 6 players so the cost will be quite low.
        self.players.sort();
        Ok(())
    }
}

/// Read a save file by decrypting it and deserializing the JSON.
pub fn read_save_file(save_file: impl AsRef<Path>) -> Result<repo_save::SaveGame, SaveBundleError> {
    let save_file = save_file.as_ref();
    if !save_file.exists() {
        return Err(SaveBundleError::MissingFile);
    }
    if !save_file.is_file() {
        return Err(SaveBundleError::ExpectedFile);
    }
    let data: Vec<u8> = crypt::decrypt_es3(&save_file, constant::ENCRYPTION_PASS)
        .map_err(|e| SaveBundleError::DecryptError(e))?;
    let save_data = serde_json::from_slice(&data).map_err(|e| SaveBundleError::JSONError(e))?;
    Ok(save_data)
}

/// Given a path to a directory as a string, extract a Vector of
/// SaveBundle objects.
pub fn extract_save_bundles(path: impl AsRef<Path>) -> Vec<SaveBundle> {
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
