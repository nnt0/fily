use std::{path::Path, fs::read, io};
use crc32fast::Hasher;
use crate::fily_err::{Context, FilyError};
#[allow(unused_imports)]
use log::{trace, debug, info, warn, error};

/// Finds files that are exact duplicates
///
/// This function does almost the same as `find_duplicate_files` but instead of loading the contents of the file into memory and
/// comparing them, it hashes them, only stores the resulting crc32 hash, and compares the hashes. This can significantly reduce
/// the required amount of RAM needed but introduces the chance of a false positive happening. Since there are only 2^32 possible
/// unique hashes there will always be the probability of a collision. At 10000 files the probability of at least
/// one collision is at 0.01157388106%.
/// This is a pretty slim chance of a false duplicate so we should be fine most of the time but if you need to make sure that you
/// don't accidentally delete a file that is not a duplicate you'll have to check them by hand.
///
/// This function returns a tuple with two elements. The first is a `Vec` containing two paths to files that are the same.
/// The second `Vec` contains the paths which caused an error as well as the error that occured.
pub fn find_duplicate_files_hash<P: AsRef<Path>>(files_to_check: &[P]) -> (Vec<(&Path, &Path)>, Vec<(&Path, FilyError<io::Error>)>) {
    let files_to_check: Vec<&Path> = files_to_check.iter().map(AsRef::as_ref).collect();

    trace!("find_duplicate_files_hash files_to_check: {:?}", files_to_check);

    let mut errors = Vec::new();
    let mut files_to_check: Vec<FileWithHash<'_>> = files_to_check.into_iter().filter_map(|path| {
        let len = match get_len_of_file(path) {
            Ok(len) => len,
            Err(e) => {
                errors.push((path, e));
                return None;
            }
        };

        Some(FileWithHash {
            len,
            path,
            hash: None,
        })
    }).collect();

    let mut duplicates = Vec::new();
    let files_to_check_len = files_to_check.len();

    for i in 0..files_to_check_len {
        for j in i + 1..files_to_check_len {
            if files_to_check[i].len != files_to_check[j].len {
                continue;
            }

            // Working around the borrow checker is fun
            let file = &mut files_to_check[i];
            let hash = match file.get_hash() {
                Ok(hash) => hash,
                Err(e) => {
                    errors.push((file.path, e));
                    continue;
                }
            };

            let other_file = &mut files_to_check[j];
            let other_hash = match other_file.get_hash() {
                Ok(hash) => hash,
                Err(e) => {
                    errors.push((other_file.path, e));
                    continue;
                }
            };

            if hash == other_hash {
                duplicates.push((files_to_check[i].path, files_to_check[j].path));
            }
        }
    }

    debug!("Found {} duplicates", duplicates.len());

    (duplicates, errors)
}

/// Finds files that are exact duplicates
///
/// This can use a lot of resources depending on the amount of files to process and their size.
/// Worst case scenario for memory usage is that it has the contents of every file it should check in memory.
/// This should only happen if all files have exactly the same amount of bytes. While that probably won't
/// happen in real life a comparison between just two duplicates that are both 10 GB would still result in a memory
/// usage of at least 20 GB.
///
/// If you know that you won't be able to provide the needed memory, use `find_duplicate_files_hash`
///
/// This function returns a tuple with two elements. The first is a `Vec` containing two paths to files that are the same.
/// The second `Vec` contains the paths which caused an error as well as the error that occured.
pub fn find_duplicate_files<P: AsRef<Path>>(files_to_check: &[P]) -> (Vec<(&Path, &Path)>, Vec<(&Path, FilyError<io::Error>)>) {
    let files_to_check: Vec<&Path> = files_to_check.iter().map(AsRef::as_ref).collect();

    trace!("find_duplicate_files files_to_check: {:?}", files_to_check);

    let mut errors = Vec::new();
    let mut files_to_check: Vec<FileWithContents<'_>> = files_to_check.into_iter().filter_map(|path| {
        let len = match get_len_of_file(path) {
            Ok(len) => len,
            Err(e) => {
                errors.push((path, e));
                return None;
            }
        };

        Some(FileWithContents {
            len,
            path,
            contents: None,
        })
    }).collect();

    let mut duplicates = Vec::new();
    let files_to_check_len = files_to_check.len();

    for i in 0..files_to_check_len {
        for j in i + 1..files_to_check_len {
            if files_to_check[i].len != files_to_check[j].len {
                continue;
            }

            let file = &mut files_to_check[i];
            let contents = match file.get_contents() {
                Ok(contents) => contents,
                Err(e) => {
                    errors.push((file.path, e));
                    continue;
                }
            };

            let other_file = &mut files_to_check[j];
            let other_contents = match other_file.get_contents() {
                Ok(contents) => contents,
                Err(e) => {
                    errors.push((other_file.path, e));
                    continue;
                }
            };

            if contents == other_contents {
                duplicates.push((files_to_check[i].path, files_to_check[j].path));
            }

            files_to_check[i].contents = Some(contents);
            files_to_check[j].contents = Some(other_contents);
        }

        files_to_check[i].contents = None;
    }

    debug!("Found {} duplicates", duplicates.len());

    (duplicates, errors)
}

struct FileWithContents<'a> {
    path: &'a Path,
    len: u64,
    contents: Option<Vec<u8>>,
}

impl FileWithContents<'_> {
    /// Returns the contents of the file if they're already loaded
    ///
    /// If the contents of the file have not been loaded yet it will
    /// first load and store it
    ///
    /// If the contents are already stored this will move ownership of them
    /// and set `contents` to `None`. If you don't want this function to
    /// read the file again on the next call set `contents` to the `Vec` this
    /// function returned
    ///
    /// # Errors
    ///
    /// This function returns an error if it fails to read the file
    fn get_contents(&mut self) -> Result<Vec<u8>, FilyError<io::Error>> {
        if self.contents.is_some() {
            Ok(self.contents.take().unwrap())
        } else {
            read(self.path)
                .with_context(|| format!("Couldn't read contents of {:?}", self.path.display()))
        }
    }
}

struct FileWithHash<'a> {
    path: &'a Path,
    len: u64,
    hash: Option<u32>,
}

impl FileWithHash<'_> {
    /// Returns the hash of the file if it has already been calculated
    ///
    /// If the hash of the file has not been calculated yet it will
    /// first load the contents of the file and then calculate its hash
    ///
    /// # Errors
    ///
    /// This function returns an error if it fails to calculate the hash of the file
    fn get_hash(&mut self) -> Result<u32, FilyError<io::Error>> {
        if let Some(hash) = self.hash {
            Ok(hash)
        } else {
            self.hash = Some(crc32_of_file(self.path)
                .with_context(|| format!("Couldn't read contents of {:?}", self.path.display()))?);

            Ok(*self.hash.as_ref().unwrap())
        }
    }
}

/// Returns the number of bytes a file has
///
/// # Errors
///
/// This function returns an error if it fails to get the files metadata
pub fn get_len_of_file(path: impl AsRef<Path>) -> Result<u64, FilyError<io::Error>> {
    let path = path.as_ref();

    path.metadata()
        .with_context(|| format!("Failed to get metadata of {:?}", path.display()))
        .map(|metadata| metadata.len())
}

/// Calculates the crc32 of a bytes slice
#[must_use]
pub fn crc32_from_bytes(bytes: &[u8]) -> u32 {
    let mut hasher = Hasher::new();

    hasher.update(bytes);
    hasher.finalize()
}

/// Calculates the crc32 of the contents of a file
///
/// # Errors
///
/// This function returns an error if
///
/// * the path doesn't exist
/// * the path points to a folder
/// * the path can't be opened/read from
pub fn crc32_of_file(path: impl AsRef<Path>) -> Result<u32, FilyError<io::Error>> {
    let path = path.as_ref();

    let contents = read(&path)
        .with_context(|| format!("Couldn't read contents of {:?}", path.display()))?;

    Ok(crc32_from_bytes(contents.as_slice()))
}
