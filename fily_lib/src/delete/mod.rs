use std::{path::Path, fs::{OpenOptions, remove_dir_all, remove_file}, io::{self, SeekFrom}};
use crate::fily_err::{Context, FilyError};
#[allow(unused_imports)]
use log::{trace, debug, info, warn, error};

/// Deletes files and folders
///
/// Files will just get removed directly. If the path points to a folder it will first
/// recursively remove all the files and folders in that folder and then remove the folder
/// the path points to
///
/// Symbolic links will not be followed. This will delete the symlink itself
///
/// # Errors
///
/// Returns an error if an error occurs while trying to delete the file or folder the path points to
pub fn delete(path: impl AsRef<Path>) -> Result<(), FilyError<io::Error>> {
    let path = path.as_ref();

    trace!("delete path_to_delete: {:?}", path.display());

    remove_dir_all(path).with_context(|| format!("Error removing {:?}", path.display()))?;

    Ok(())
}

/// Overwrites the file with zeroes first then deletes it
///
/// Symbolic links will not be followed. This will delete and overwrite the symlink itself
///
/// Note that this does NOT make sure that the file can't
/// be recovered in any way whatsoever but it does make
/// it unlikely
///
/// # Errors
///
/// This errors if
///
/// * `path` points to a folder
/// * the `overwrite_file_with_zeroes` function fails
/// * it fails to remove the file after overwriting it
pub fn safe_delete(path: impl AsRef<Path>) -> Result<(), FilyError<io::Error>> {
    let path = path.as_ref();

    trace!("safe_delete path_to_delete: {:?}", path.display());

    overwrite_file_with_zeroes(path)
        .with_context(|| format!("\nFailed to overwrite {:?} with zeroes", path.display()))?;

    remove_file(path).with_context(|| format!("Failed to remove {:?}", path.display()))?;

    Ok(())
}

/// Overwrites every byte of a file with a zero byte
///
/// Symbolic links will not be followed. This will overwrite the symlink itself
///
/// Note that this does NOT make sure that the files original
/// data can't be recovered in any way whatsoever but it does make
/// it unlikely
///
/// # Errors
///
/// This errors if
///
/// * `path` points to a folder
/// * the file doesn't exist/can't be opened/can't be written to
/// * it fails to sync the new data to the disk. Altough in this case
///   the data of the file should still be all zeroes as soon as
///   the OS writes the modified buffer to the disk
pub fn overwrite_file_with_zeroes(path: impl AsRef<Path>) -> Result<(), FilyError<io::Error>> {
    let path = path.as_ref();

    trace!("overwrite_file_with_zeroes path: {:?}", path.display());

    let file = OpenOptions::new()
        .write(true)
        .open(path)
        .with_context(|| format!("Failed to open {:?}", path.display()))?;

    let metadata = file.metadata()
        .with_context(|| format!("Failed to get metadata of {:?}", path.display()))?;

    if metadata.is_dir() {
        return Err(FilyError::new(
            io::Error::new(io::ErrorKind::InvalidInput, "Can't overwrite a folder"),
            "Can't overwrite a folder"
        ));
    }

    let len = metadata.len() as usize;

    overwrite_with_zeroes(&file, len)
        .with_context(|| format!("Error while trying to overwrite {:?}", path.display()))?;

    file.sync_data()
        .context("Error syncing the file to disk. The file was overwritten with zeroes but the old data could still be available on the disk for a bit")?;

    Ok(())
}

/// Overwrites a buffer with `len` amount of zeroes
///
/// If `len` is bigger than the buffers length it will add zeroes to the end of it
///
/// If whatever you're trying to overwrite doesn't implement [Seek](std::io::Seek) try wrapping it in a [Cursor](std::io::Cursor)
///
/// # Errors
///
/// This errors if an error occurs while writing to the buffer or it fails to flush the written data
pub fn overwrite_with_zeroes<W: io::Write + io::Seek>(mut buf: W, len: usize) -> Result<(), io::Error> {
    buf.seek(SeekFrom::Start(0)).expect("Error at a non negative offset");

    let zero_buffer = vec![0_u8; 100_000];
    let zero_buffer_len = zero_buffer.len();
    let amount_full_buffer_writes = len / zero_buffer_len;
    let remaining_bytes_to_overwrite = len % zero_buffer_len;

    for _ in 0..amount_full_buffer_writes {
        buf.write_all(&zero_buffer)?;
    }

    if remaining_bytes_to_overwrite > 0 {
        buf.write_all(&zero_buffer[0..remaining_bytes_to_overwrite])?;
    }

    buf.flush()?;

    Ok(())
}
