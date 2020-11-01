use std::{path::Path, fs::read};
use crc32fast::Hasher;
#[allow(unused_imports)]
use log::{trace, debug, info, warn, error};

/// This function does almost the same as `find_duplicate_files()` but instead of loading the contents of the file into memory and
/// comparing them, it hashes them, only stores the resulting crc32 hash, and compares the hashes. This can significantly reduce
/// the required amount of RAM needed but introduces the chance of a false positive happening. Since there are only 2^32 possible
/// unique hashes there will always be the probability of a collision. At 10000 files the probability of at least
/// one collision is at 0.01157388106%.
/// This is a pretty slim chance of a false duplicate so we should be fine most of the time but if you need to make sure that you
/// don't accidentally delete a file that is not a duplicate you'll have to check them by hand.
///
/// This will log any errors it encounters and just ignore the file that produced an error
pub fn find_duplicate_files_hash<P: AsRef<Path>>(files_to_check: &[P]) -> Vec<(&Path, &Path)> {
    let files_to_check: Vec<&Path> = files_to_check.iter().map(AsRef::as_ref).collect();

    trace!("find_duplicate_files_hash files_to_check: {:?}", files_to_check);

    let mut files_to_check: Vec<FileWithHash<&Path>> = files_to_check.into_iter().filter_map(|path| {
        let len = match path.metadata() {
            Ok(metadata) => metadata.len(),
            Err(e) => {
                info!("Error accessing {:?} {} ignoring this file", path.display(), e);
                return None;
            }
        };

        Some(FileWithHash {
            len,
            path,
            hash: None,
        })
    }).collect();

    let hasher = Hasher::new();
    let mut duplicates = Vec::new();
    let files_to_check_len = files_to_check.len();

    for i in 0..files_to_check_len {
        for j in i + 1..files_to_check_len {
            if files_to_check[i].len != files_to_check[j].len {
                continue;
            }

            // Working around the borrow checker is fun
            let file = &mut files_to_check[i];
            let hash = match file.hash {
                Some(_) => file.hash.take().unwrap(),
                None => match read(&file.path) {
                    Ok(contents) => {
                        let mut hasher = hasher.clone();
                        hasher.update(&contents);
                        hasher.finalize()
                    },
                    Err(e) => {
                        info!("Couldn't read {:?} {}", file.path.display(), e);
                        continue;
                    }
                }
            };

            let other_file = &mut files_to_check[j];
            let other_hash = match other_file.hash {
                Some(_) => other_file.hash.take().unwrap(),
                None => match read(&other_file.path) {
                    Ok(contents) => {
                        let mut hasher = hasher.clone();
                        hasher.update(&contents);
                        hasher.finalize()
                    },
                    Err(e) => {
                        info!("Couldn't read {:?} {}", other_file.path.display(), e);
                        continue;
                    }
                }
            };

            if hash == other_hash {
                duplicates.push((files_to_check[i].path.clone(), files_to_check[j].path.clone()));
            }

            files_to_check[i].hash = Some(hash);
            files_to_check[j].hash = Some(other_hash);
        }

        files_to_check[i].hash = None;
    }

    debug!("Found {} duplicates", duplicates.len());

    duplicates
}

/// Finds files that are exact duplicates
///
/// This can use a lot of resources depending on the amount of files to process and their size.
/// Worst case scenario for memory usage is that it has the contents of every file it should check in memory.
/// This should only happen if all files have exactly the same amount of bytes. While that probably won't
/// happen in real life a comparison between just two duplicates that are both 10 GB would still result in a memory
/// usage of at least 20 GB.
///
/// If you know that you won't be able to provide the needed memory, use `find_duplicate_files_hash()`
///
/// This will log any errors it encounters and just ignore the file that produced an error
pub fn find_duplicate_files<P: AsRef<Path>>(files_to_check: &[P]) -> Vec<(&Path, &Path)> {
    let files_to_check: Vec<&Path> = files_to_check.iter().map(AsRef::as_ref).collect();

    trace!("find_duplicate_files files_to_check: {:?}", files_to_check);

    let mut files_to_check: Vec<FileWithContents<&Path>> = files_to_check.into_iter().filter_map(|path| {
        let len = match path.metadata() {
            Ok(metadata) => metadata.len(),
            Err(e) => {
                info!("Error accessing {:?} {} ignoring this file", path.display(), e);
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

            // Working around the borrow checker is fun
            let file = &mut files_to_check[i];
            let contents = match file.contents {
                Some(_) => file.contents.take().unwrap(),
                None => match read(&file.path) {
                    Ok(contents) => contents,
                    Err(e) => {
                        info!("Couldn't read {:?} {}", file.path.display(), e);
                        continue;
                    }
                }
            };

            let other_file = &mut files_to_check[j];
            let other_contents = match other_file.contents {
                Some(_) => other_file.contents.take().unwrap(),
                None => match read(&other_file.path) {
                    Ok(contents) => contents,
                    Err(e) => {
                        info!("Couldn't read {:?} {}", other_file.path.display(), e);
                        continue;
                    }
                }
            };

            if contents == other_contents {
                duplicates.push((files_to_check[i].path.clone(), files_to_check[j].path.clone()));
            }

            files_to_check[i].contents = Some(contents);
            files_to_check[j].contents = Some(other_contents);
        }

        files_to_check[i].contents = None;
    }

    debug!("Found {} duplicates", duplicates.len());

    duplicates
}

struct FileWithContents<T: AsRef<Path>> {
    path: T,
    len: u64,
    contents: Option<Vec<u8>>,
}

struct FileWithHash<T: AsRef<Path>> {
    path: T,
    len: u64,
    hash: Option<u32>,
}
