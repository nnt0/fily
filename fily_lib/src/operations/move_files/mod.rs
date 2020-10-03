use std::{fs::{canonicalize, rename, create_dir_all}, error::Error, path::Path, io, fmt};
// use dialoguer::Confirm;
#[allow(unused_imports)]
use log::{trace, debug, info, warn, error};

#[derive(Debug)]
pub enum MoveFilesError {
    CouldNotCreateMoveToPath(io::Error),
    MoveToPointsToAFile,
    IOError(io::Error),
}

impl Error for MoveFilesError {}

impl fmt::Display for MoveFilesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Moves all files from one place to another
///
/// # Errors
///
/// Fails if `move_to` does not exist and it fails to create it, if `move_to` points to a file or if an error occurs canonicalizing `move_to`
pub fn move_files<T, U>(move_to: T, files_to_move: &[U], you_sure_prompt: bool) -> Result<(), MoveFilesError> where
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

    let files_to_move = {
        let mut files_to_move_canonicalized = Vec::with_capacity(files_to_move.len());

        for path in files_to_move {
            files_to_move_canonicalized.push(match canonicalize(path) {
                Ok(path) => path,
                Err(e) => {
                    info!("Error accessing {:?} {} skipping this file", path.as_ref().display(), e);
                    continue;
                }
            });
        }

        files_to_move_canonicalized
    };

    trace!("move move_to: {:?} files_to_move: {:?} you_sure_prompt: {}", move_to.display(), files_to_move, you_sure_prompt);

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

    // For some reason we get errors if this is used in a pipe. Not sure why. I'll just uncomment it for now
    // Not sure if this is even useful
    // if you_sure_prompt && !Confirm::new().with_prompt(format!("Moving {} files to {} . Are you sure?", files_to_move.len(), move_to.display())).default(false).interact()? {
    //     debug!("Aborting because of negative prompt response");
    //     return Ok(());
    // }

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
