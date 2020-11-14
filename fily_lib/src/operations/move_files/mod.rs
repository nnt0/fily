use std::{fs::{rename, create_dir_all}, path::Path, io, fmt, error::Error};
use crate::fily_err::{Context, FilyError};
#[allow(unused_imports)]
use log::{trace, debug, info, warn, error};

#[derive(Debug)]
pub enum MoveFilesError {
    /// The path to which the files should be moved to points to a file. The path has to point to a folder
    MoveToPointsToAFile,

    /// Failed to get the filename of a file
    FileWithoutFilename,

    /// If it fails a io operation
    IOError(io::Error),
}

impl Error for MoveFilesError {}

impl fmt::Display for MoveFilesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<FilyError<io::Error>> for FilyError<MoveFilesError> {
    fn from(err: FilyError<io::Error>) -> Self {
        let (io_err, context) = err.destructure();

        FilyError::new(MoveFilesError::IOError(io_err), context)
    }
}

/// Moves files
///
/// This function will return, if it doesn't fail, a `Vec` containing all paths which had an error.
/// Those files weren't moved.
///
/// # Errors
///
/// This function only fails if something happens that prevents it from moving any of the files.
///
/// Such a thing could be:
///
/// * `move_to` doesn't exist and it failed to create it
/// * it failed to get the metadata of the path `move_to` points to
/// * `move_to` points to a file
pub fn move_files<P: AsRef<Path>>(move_to: impl AsRef<Path>, files_to_move: &[P]) -> Result<Vec<(&Path, FilyError<MoveFilesError>)>, FilyError<MoveFilesError>> {
    let move_to = move_to.as_ref();
    let files_to_move: Vec<&Path> = files_to_move.iter().map(AsRef::as_ref).collect();

    trace!("move move_to: {:?} files_to_move: {:?}", move_to.display(), files_to_move);

    if !move_to.exists() {
        create_dir_all(&move_to)
            .with_context(|| format!("Failed to create move_to path: {:?}", move_to.display()))?;
    }

    let is_file = move_to.metadata()
        .with_context(|| format!("Error getting metadata of {:?}", move_to.display()))?
        .is_file();

    if is_file {
        return Err(FilyError::new_with_context(
            MoveFilesError::MoveToPointsToAFile,
            || format!("move_to points to a file {:?}", move_to.display())
        ));
    }

    let mut errors = Vec::new();

    for old_path in files_to_move {
        let filename = match old_path.file_name() {
            Some(filename) => filename,
            None => {
                errors.push((
                    old_path,
                    FilyError::new_with_context(MoveFilesError::FileWithoutFilename, || format!("Failed to get filename of {:?}", old_path.display()))
                ));
                continue;
            }
        };

        let mut new_path = move_to.to_path_buf();
        new_path.push(filename);

        if let Err(e) = rename(&old_path, &new_path) {
            errors.push((
                old_path,
                FilyError::new_with_context(MoveFilesError::IOError(e), || format!("Error moving {:?}", old_path.display()))
            ));
        };
    }

    Ok(errors)
}
