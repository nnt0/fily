use std::{fs::{canonicalize, rename, create_dir_all}, path::{Path, PathBuf}, io};
use thiserror::Error;
#[allow(unused_imports)]
use log::{trace, debug, info, warn, error};

#[derive(Error, Debug)]
pub enum MoveFilesError {
    #[error("The path to which the files should be moved to doesn't exist. Failed trying to create it")]
    CouldNotCreateMoveToPath(io::Error),
    #[error("The path to which the files should be moved to points to a file. The path has to point to a folder")]
    MoveToPointsToAFile,
    #[error("An error occured while getting info on the path to which the files should be moved to")]
    IOError(io::Error),
}

/// Moves all files from one place to another
///
/// # Errors
///
/// Fails if `move_to` does not exist and it fails to create it, if `move_to` points to a file or if an error occurs canonicalizing `move_to`
pub fn move_files<T, U>(move_to: T, files_to_move: &[U]) -> Result<(), MoveFilesError> where
    T: AsRef<Path>,
    U: AsRef<Path> {
    let move_to = move_to.as_ref();
    if !move_to.exists() {
        if let Err(e) = create_dir_all(&move_to) {
            info!("Failed to create move_to path: {:?} {}", move_to.display(), e);
            return Err(MoveFilesError::CouldNotCreateMoveToPath(e));
        }
    }

    let move_to = match canonicalize(&move_to) {
        Ok(path) => path,
        Err(e) => {
            info!("Error accessing {:?} {}", move_to.display(), e);
            return Err(MoveFilesError::IOError(e));
        }
    };

    let files_to_move: Vec<PathBuf> =
        files_to_move.iter().filter_map(|path| {
            match canonicalize(path) {
                Ok(path) => Some(path),
                Err(e) => {
                    info!("Error accessing {:?} {} skipping this path", path.as_ref().display(), e);
                    None
                }
            }
        }).collect();

    trace!("move move_to: {:?} files_to_move: {:?}", move_to.display(), files_to_move);

    match move_to.metadata() {
        Ok(metadata) => if metadata.file_type().is_file() {
            info!("move_to points to a file {:?}", move_to.display());
            return Err(MoveFilesError::MoveToPointsToAFile);
        }
        Err(e) => {
            info!("Error accessing {:?} {}", move_to.display(), e);
            return Err(MoveFilesError::IOError(e));
        }
    };

    for old_path in files_to_move {
        let mut new_path = move_to.clone();
        new_path.push(match old_path.file_name() {
            Some(filename) => filename,
            None => {
                info!("Failed to get filename from {:?} skipping moving of this file", old_path.display());
                continue;
            }
        });
        match rename(&old_path, &new_path) {
            Ok(()) => info!("Moved {:?} to {:?}", old_path.display(), new_path.display()),
            Err(e) => info!("Error moving {:?} {}", old_path.display(), e),
        };
    }

    Ok(())
}
