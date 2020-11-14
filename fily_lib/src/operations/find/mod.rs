use std::{path::{Path, PathBuf}, error::Error, convert::TryInto};
use walkdir::WalkDir;
#[allow(unused_imports)]
use log::{trace, debug, info, warn, error};

mod condition;
pub use condition::Condition;

mod condition_try_from;

mod search_criteria;
pub use search_criteria::*;

mod find_options;
pub use find_options::*;

/// Finds files or directories that fit all of the criteria
/// 
/// The returned `Vec` can be empty if nothing was found
///
/// If it encounters any errors while getting info on files it will just log it
/// (assuming logging is turned on) and ignore the file where the error happened
pub fn find<P: AsRef<Path>>(paths_to_search_in: &[P], find_options: &FindOptions) -> Vec<PathBuf> {
    let paths_to_search_in: Vec<&Path> = paths_to_search_in.iter().map(AsRef::as_ref).collect();

    trace!("find paths_to_search_in: {:?} find_options: {:?}", paths_to_search_in, find_options);

    let mut results = Vec::new();

    for path in paths_to_search_in {
        let mut matching_files: Vec<PathBuf> = WalkDir::new(path)
            .min_depth(find_options.min_depth_from_start)
            .max_depth(find_options.max_search_depth)
            .follow_links(find_options.follow_symlinks)
            .into_iter()
            .filter_map(|entry| {
                if let Err(e) = entry {
                    info!("Error accessing a file {} ignoring it", e);
                    return None;
                }

                let entry = entry.unwrap();

                if let Some(ignore) = find_options.ignore {
                    let file_type = entry.file_type();
                    match ignore {
                        Ignore::Files => if file_type.is_file() {
                            return None;
                        }
                        Ignore::Folders => if file_type.is_dir() {
                            return None;
                        }
                    };
                }

                if find_options.ignore_hidden_files {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.starts_with('.') {
                            return None;
                        }
                    }
                    // Not sure if this is the right way to go here. Maybe we should actually filter out the file since it errored?
                }

                // Checks if all Conditions match the file
                // If any do not match, the file gets filtered out
                if find_options.options.iter().all(|option| option.evaluate(&entry).unwrap_or(false)) {
                    Some(entry.into_path())
                } else {
                    None
                }
            })
            .collect();

        if results.len() + matching_files.len() > find_options.max_num_results {
            // Shorten the vec in case there are too many items so we don't return more
            // paths than we should
            matching_files.truncate(find_options.max_num_results - results.len());
            results.append(&mut matching_files);

            debug!("Max amount of results ({}) reached. Exiting early", find_options.max_num_results);

            break;
        }

        results.append(&mut matching_files);
    }

    debug!("Found {} files", results.len());

    results
}
