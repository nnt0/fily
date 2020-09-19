use std::{fs::{canonicalize, rename, create_dir_all}, error::Error, path::Path};
use dialoguer::Confirm;
#[allow(unused_imports)]
use log::{trace, debug, info, warn, error};

/// Moves all files from one place to another
pub fn move_files<T, U>(move_to: T, files_to_move: &[U], you_sure_prompt: bool) -> Result<(), Box<dyn Error>> where
    T: AsRef<Path>,
    U: AsRef<Path> {
    if !move_to.as_ref().exists() {
        create_dir_all(&move_to)?;
    }

    let move_to = match canonicalize(&move_to) {
        Ok(path) => path,
        Err(e) => {
            let err_msg = format!("Error accessing {:?} {}", move_to.as_ref().display(), e);
            info!("{}", err_msg);
            return Err(Box::from(err_msg));
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
            return Err(Box::from("move_to points to a file"));
        }
        Err(e) => {
            let err_msg = format!("Error accessing {:?} {}", move_to.display(), e);
            info!("{}", err_msg);
            return Err(Box::from(err_msg));
        }
    };

    // For some reason we get errors if this is used in a pipe. Not sure why. I'll just uncomment is for now
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
            Ok(_) => info!("Moved {:?} to {:?}", old_path.display(), new_path.display()),
            Err(e) => info!("Error moving {:?} {}", old_path.display(), e),
        };
    }

    Ok(())
}
