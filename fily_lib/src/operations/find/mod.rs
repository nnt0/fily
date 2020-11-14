use std::path::{Path, PathBuf};
use walkdir::WalkDir;
#[allow(unused_imports)]
use log::{trace, debug, info, warn, error};

mod condition;
pub use condition::{Condition, ConditionEvalError};

mod condition_try_from;

mod search_criteria;
pub use search_criteria::*;

mod find_options;
pub use find_options::*;

/// Finds files or directories that fit all of the criteria
///
/// This function returns a tuple of two `Vec`s. The first one contains paths to the files that
/// matched all of the conditions. The second one contains the errors that occured during the
/// evaluation of the conditions on a file. These files could theoretically also match the conditions
/// but we don't know if they do since an error occured.
/// 
/// The returned `Vec`s can be empty if nothing was found or no error occured
pub fn find<P: AsRef<Path>>(paths_to_search_in: &[P], find_options: &FindOptions) -> (Vec<PathBuf>, Vec<(PathBuf, ConditionEvalError)>) {
    let paths_to_search_in: Vec<&Path> = paths_to_search_in.iter().map(AsRef::as_ref).collect();

    trace!("find paths_to_search_in: {:?} find_options: {:?}", paths_to_search_in, find_options);

    let mut results = Vec::new();
    let mut errors = Vec::new();

    for path in paths_to_search_in {
        let mut matching_files: Vec<PathBuf> = WalkDir::new(path)
            .min_depth(find_options.min_depth_from_start)
            .max_depth(find_options.max_search_depth)
            .follow_links(find_options.follow_symlinks)
            .into_iter()
            .filter_map(|entry| {
                if let Err(e) = entry {
                    info!("Error accessing a file {}", e);
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

                let path = entry.path();

                // Checks if all Conditions match the file
                // If any do not match, the file gets filtered out
                if find_options.options.iter().all(|option| option.evaluate(&entry).unwrap_or_else(|err| {
                    errors.push((path.to_path_buf(), err));
                    false
                })) {
                    Some(path.to_path_buf())
                } else {
                    None
                }
            })
            .collect();

        results.append(&mut matching_files);

        if results.len() >= find_options.max_num_results {
            // Shorten the vec in case there are too many items so we don't return more
            // paths than we should
            results.truncate(find_options.max_num_results);

            debug!("Max amount of results ({}) reached. Exiting early", find_options.max_num_results);

            break;
        }
    }

    debug!("Found {} files", results.len());

    (results, errors)
}
