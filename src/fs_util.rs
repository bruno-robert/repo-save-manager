//! File system utilities used by RSM internally.

use std::path::{Path, PathBuf};
use std::{fs, io};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SaveManagerError {
    #[error("Save already exists")]
    SaveExists,
    #[error("IO error: {0}")]
    IOError(#[from] io::Error),
    #[error("FsExtra error: {0}")]
    FsExtraError(#[from] fs_extra::error::Error),
}

/// Returns the first existing path that's a directory (if one exists).
/// I no valid directory was found, returns None.
pub fn first_existing_dir(paths: Vec<PathBuf>) -> Option<PathBuf> {
    for path in paths {
        if path.exists() && path.is_dir() {
            log::debug!("Found existing save directory: {:?}", &path);
            return Some(path);
        }
    }

    log::warn!("No existing save directories found");
    None
}

/// Copy a directory recursively.
///
/// This function copies the contents of a directory to another directory.
/// If the destination directory already exists, it will be overwritten if `overwrite` is true.
/// Otherwise, an error will be returned.
///
/// # Errors
///
/// Returns an error if the source directory does not exist or if there is an IO error.
/// If `overwrite` is false and the destination directory already exists, an error will be returned.
pub fn copy_directory(
    source_dir: impl AsRef<Path>,
    destination_root: impl AsRef<Path>,
    overwrite: bool,
) -> Result<(), SaveManagerError> {
    let source_dir_path = source_dir.as_ref();
    let destination_root_path = destination_root.as_ref();

    let destination_dir_path = destination_root_path.join(source_dir_path.file_name().unwrap());

    // Check if destination already exists
    if destination_dir_path.exists() && !overwrite {
        return Err(SaveManagerError::SaveExists);
    }

    // Create the destination_root directory if it doesn't exist
    if !destination_root_path.exists() {
        std::fs::create_dir_all(&source_dir_path).map_err(|e| {
            log::error!(e:err; "Failed to create backup directory");
            SaveManagerError::IOError(e)
        })?;
    }

    // If the destination dir exists, delete it
    if destination_dir_path.exists() {
        std::fs::remove_dir_all(&destination_dir_path).map_err(|e| {
            log::error!(e:err; "Failed to remove existing destination directory");
            SaveManagerError::IOError(e)
        })?;
    }

    // Create an empty destination dir
    fs::create_dir_all(destination_dir_path).map_err(|e| {
        log::error!(e:err; "Failed to create destination directory");
        SaveManagerError::IOError(e)
    })?;

    // copy source_dir_path to destination_root
    fs_extra::dir::copy(
        &source_dir_path,
        &destination_root_path,
        &fs_extra::dir::CopyOptions::new(),
    )
    .map_err(|e| {
        log::error!(e:err; "Failed to copy directory");
        SaveManagerError::FsExtraError(e)
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::{TempDir, tempdir};

    mod first_existing_dir {
        use super::*;

        /// Test that if a valid path exists, it's returned
        #[test]
        fn valid_path() {
            // Arrange
            // Setup - create temporary directories for testing
            let temp_dir = tempdir().unwrap();
            let root_path = temp_dir.path().to_path_buf();

            // Create one directory that exists
            let existing_dir = root_path.join("existing");
            fs::create_dir(&existing_dir).unwrap();

            // Create paths to test - one exists, others don't
            let paths = vec![
                root_path.join("nonexistent1"),
                existing_dir.clone(),
                root_path.join("nonexistent2"),
            ];

            // Act
            let result = first_existing_dir(paths);

            // Assert
            assert!(result.is_some());
            assert_eq!(result.unwrap(), existing_dir);
        }

        /// Test that if no valid path exists, None is returned
        #[test]
        fn no_valid_path() {
            // Arrange
            let temp_dir = tempdir().unwrap();
            let root_path = temp_dir.path().to_path_buf();

            let nonexistent_paths = vec![
                root_path.join("nonexistent1"),
                root_path.join("nonexistent2"),
            ];

            // Act
            let result = first_existing_dir(nonexistent_paths);

            // Assert
            assert!(result.is_none(), "Should return None when no paths exist");
        }
    }

    mod copy_save_bundle {
        use super::*;

        #[test]
        fn first_time_succeeds() -> Result<(), Box<dyn std::error::Error>> {
            // Arrange
            let (_tempdir, _source_root_dir, source_dir) = sample_dir("src_")?;
            let dest_dir = tempdir()?.into_path();

            // Act
            let result = copy_directory(&source_dir, &dest_dir, false);

            // Assert
            assert!(result.is_ok());
            assert_dirs_equal(&source_dir, &dest_dir.join(source_dir.file_name().unwrap()));

            Ok(())
        }

        #[test]
        fn without_overwrite_fails() -> Result<(), Box<dyn std::error::Error>> {
            // Arrange
            let (_tempdir, _source_root_dir, source_dir) = sample_dir("src_")?;
            let (_tempdir, dest_root_dir, _dest_dir) = sample_dir("dest_")?;

            // Act
            let result = copy_directory(&source_dir, &dest_root_dir, false);

            // Assert
            assert!(matches!(result, Err(SaveManagerError::SaveExists)));

            Ok(())
        }

        #[test]
        fn with_overwrite_succeeds() -> Result<(), Box<dyn std::error::Error>> {
            // Arrange
            let (_tempdir1, _source_root_dir, source_dir) = sample_dir("src_")?;
            let (_tempdir2, dest_root_dir, dest_dir) = sample_dir("dest_")?;

            // Act
            let result = copy_directory(&source_dir, &dest_root_dir, true);

            // Assert
            assert!(result.is_ok(), "directory copy failed");
            assert_dirs_equal(&source_dir, &dest_dir);
            let _ = _tempdir1.close();
            let _ = _tempdir2.close();

            Ok(())
        }
    }

    /// Helper function to set up test directories with sample files
    ///
    /// Returns
    ///     The reference to the TempDir object (keep it in scope or else temp dir is deleted).
    ///     PathBuf to the temp dir.
    ///     PathBuf to the only directory in the temp dir.
    fn sample_dir(
        file_prefix: &str,
    ) -> Result<(TempDir, PathBuf, PathBuf), Box<dyn std::error::Error>> {
        // Create temporary directories
        let tempdir = tempdir()?;
        // root temporary dir
        let source_root_dir: PathBuf = tempdir.path().to_path_buf();

        // first and only subfolder in temporary dir
        let source_dir = source_root_dir.join("base_subdir");
        fs::create_dir_all(&source_dir)?;

        // Create test files in the source directory
        let file1_path = source_dir.join(format!("{file_prefix}file1.txt"));
        let mut file1 = File::create(&file1_path)?;
        writeln!(file1, "This is test file 1")?;

        // Create a subdirectory with a file
        let subdir_path = source_dir.join("subdir");
        fs::create_dir(&subdir_path)?;
        let file2_path = subdir_path.join(format!("{file_prefix}file2.txt"));
        let mut file2 = File::create(&file2_path)?;
        writeln!(file2, "This is test file 2")?;

        Ok((tempdir, source_root_dir, source_dir))
    }

    /// Helper function to assert that files were copied correctly
    fn assert_dirs_equal(source_dir: impl AsRef<Path>, dest_dir: impl AsRef<Path>) {
        let source_path = source_dir.as_ref();
        let dest_path = dest_dir.as_ref();

        // Check if both directories exist
        assert!(source_path.exists(), "Source directory does not exist");
        assert!(dest_path.exists(), "Destination directory does not exist");

        // Get all entries in the source directory recursively
        let source_entries = walkdir::WalkDir::new(source_path)
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .expect("Failed to walk source directory");

        for entry in source_entries {
            let rel_path = entry
                .path()
                .strip_prefix(source_path)
                .expect("Failed to strip prefix");
            let dest_entry_path = dest_path.join(rel_path);

            if entry.file_type().is_file() {
                // For files, check if they exist and have the same content
                assert!(
                    dest_entry_path.exists(),
                    "Destination file does not exist: {:?}",
                    rel_path
                );

                let source_content = fs::read(entry.path()).expect("Failed to read source file");
                let dest_content =
                    fs::read(&dest_entry_path).expect("Failed to read destination file");

                assert_eq!(
                    source_content, dest_content,
                    "File contents differ for {:?}",
                    rel_path
                );
            } else if entry.file_type().is_dir() && entry.path() != source_path {
                // For directories, just check if they exist
                assert!(
                    dest_entry_path.exists(),
                    "Destination directory does not exist: {:?}",
                    rel_path
                );
                assert!(
                    dest_entry_path.is_dir(),
                    "Expected directory at {:?}",
                    rel_path
                );
            }
        }
    }
}
