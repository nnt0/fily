use std::{path::{Path, PathBuf}, error::Error, fs::canonicalize, convert::TryInto};
use walkdir::WalkDir;
use rayon::prelude::*;
#[allow(unused_imports)]
use log::{trace, debug, info, warn, error};

mod condition;
pub use condition::Condition;

/// Stores options for `find`
///
/// You can instantiate this struct directly but I recommend
/// using `FindOptionsBuilder`. It provides a couple nice helper functions
/// for creating `Condition`s
///
/// If you want to instantiate this directly you can use `Default::default()` which
/// provides a config that matches anything. So you only have to change the options
/// you care about
///
/// `options`: Contains a vec of Condition<SearchCriteria<'a>> with which you
/// you can define what criteria a file should or should not match.
/// I recommend reading the docs on `Condition` and `SearchCriteria`
///
/// `max_num_results`: limits the amount of paths returned
///
/// `max_search_depth`: limits how many subfolders deep it searches
///
/// `min_depth_from_start`: limits how many subfolders deep it should start searching. I recommend
/// setting this lower than or equals to `max_search_depth`
///
/// `ignore`: used to either ignore all files or all folders
///
/// `ignore_hidden_files`: ignore all files that start with a dot
///
/// `follow_symlinks`: if it should follow any symlinks and search in there too
#[derive(Clone, Debug)]
pub struct FindOptions<'a> {
    pub options: Vec<Condition<SearchCriteria<'a>>>,
    pub max_num_results: usize,
    pub max_search_depth: usize,
    pub min_depth_from_start: usize,
    pub ignore: Option<Ignore>,
    pub ignore_hidden_files: bool,
    pub follow_symlinks: bool,
}

impl<'a> Default for FindOptions<'a> {
    fn default() -> Self {
        FindOptions {
            options: Vec::new(),
            max_num_results: usize::MAX,
            max_search_depth: usize::MAX,
            min_depth_from_start: 0,
            ignore: None,
            ignore_hidden_files: false,
            follow_symlinks: false,
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct FindOptionsBuilder<'a> {
    find_options: FindOptions<'a>
}

impl<'a> FindOptionsBuilder<'a> {
    #[inline]
    pub fn new() -> Self {
        FindOptionsBuilder::default()
    }

    #[inline]
    pub fn build(self) -> FindOptions<'a> {
        self.find_options
    }

    #[inline]
    pub fn add_condition(&mut self, condition: Condition<SearchCriteria<'a>>) -> &mut Self {
        self.find_options.options.push(condition);
        self
    }

    #[inline]
    pub fn add_conditions(&mut self, mut conditions: Vec<Condition<SearchCriteria<'a>>>) -> &mut Self {
        self.find_options.options.append(&mut conditions);
        self
    }

    /// Adds a condition that requires all of the search criterias in `search_criterias` to match
    pub fn add_all_of_condition(&mut self, mut search_criterias: Vec<SearchCriteria<'a>>) -> &mut Self {
        if search_criterias.is_empty() {
            return self;
        }

        let mut option = Box::from(Condition::Value(search_criterias.remove(0)));

        for criteria in search_criterias {
            option = Box::from(Condition::And(Box::from(Condition::Value(criteria)), option));
        }

        self.find_options.options.push(*option);
        self
    }

    /// Adds a condition that requires any (at least one) of the search criterias in `search_criterias` to match
    pub fn add_any_of_condition(&mut self, mut search_criterias: Vec<SearchCriteria<'a>>) -> &mut Self {
        if search_criterias.is_empty() {
            return self;
        }

        let mut option = Box::from(Condition::Value(search_criterias.remove(0)));

        for criteria in search_criterias {
            option = Box::from(Condition::Or(Box::from(Condition::Value(criteria)), option));
        }

        self.find_options.options.push(*option);
        self
    }

    /// Adds a condition that requires none of the search criterias in `search_criterias` to match
    pub fn add_nothing_of_condition(&mut self, mut search_criterias: Vec<SearchCriteria<'a>>) -> &mut Self {
        if search_criterias.is_empty() {
            return self;
        }

        let mut option = Box::from(Condition::Not(Box::from(Condition::Value(search_criterias.remove(0)))));

        for criteria in search_criterias {
            option = Box::from(Condition::And(Box::from(Condition::Not(Box::from(Condition::Value(criteria)))), option));
        }

        self.find_options.options.push(*option);
        self
    }

    /// Adds a condition from a `str`. This can fail.
    /// Also this isn't actually implemented right now so it'll just panic if you call this
    #[inline]
    pub fn add_condition_from_str(&mut self, condition_str: &str) -> Result<&mut Self, Box<dyn Error>> {
        self.find_options.options.push(condition_str.try_into()?);
        Ok(self)
    }

    #[inline]
    pub fn set_max_num_results(&mut self, max_num_results: usize) -> &mut Self {
        self.find_options.max_num_results = max_num_results;
        self
    }

    #[inline]
    pub fn set_max_search_depth(&mut self, max_search_depth: usize) -> &mut Self {
        self.find_options.max_search_depth = max_search_depth;
        self
    }

    #[inline]
    pub fn set_min_depth_from_start(&mut self, min_depth_from_start: usize) -> &mut Self {
        self.find_options.min_depth_from_start = min_depth_from_start;
        self
    }

    #[inline]
    pub fn set_ignored_files(&mut self, ignored_files: Ignore) -> &mut Self {
        self.find_options.ignore = Some(ignored_files);
        self
    }

    #[inline]
    pub fn set_ignore_hidden_files(&mut self, ignore_hidden_files: bool) -> &mut Self {
        self.find_options.ignore_hidden_files = ignore_hidden_files;
        self
    }

    #[inline]
    pub fn set_follow_symlinks(&mut self, follow_symlinks: bool) -> &mut Self {
        self.find_options.follow_symlinks = follow_symlinks;
        self
    }
}

/// Used to specify a criteria a file has to match
///
/// Intended to be used with `Condition`
#[derive(Clone, Debug)]
pub enum SearchCriteria<'a> {
    Filename(Filename<'a>),
    Filesize(Filesize),
    FilePath(FilePath<'a>),
    FilenameRegex(&'a regex::Regex),
    Modified(Modified),
    Accessed(Accessed),
    Created(Created)
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Filename<'a> {
    Exact(&'a str),
    Contains(&'a str),
}

/// Filesize is in bytes
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Filesize {
    Exact(u64),
    Over(u64),
    Under(u64),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum FilePath<'a> {
    Exact(&'a str),
    Contains(&'a str),
}

/// Time is in seconds and relative to the unix epoch (1970-01-01T00:00:00Z)
///
/// The value it checks it against corresponds to the `mtime` field of `stat` on
/// Unix platforms and the `ftLastWriteTime` field on Windows platforms
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Modified {
    At(i64),
    Before(i64),
    After(i64),
}

/// Time is in seconds and relative to the unix epoch (1970-01-01T00:00:00Z)
///
/// The value it checks it against corresponds to the `atime` field of `stat` on
/// Unix platforms and the `ftLastAccessTime` field on Windows platforms
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Accessed {
    At(i64),
    Before(i64),
    After(i64),
}

/// Time is in seconds and relative to the unix epoch (1970-01-01T00:00:00Z)
///
/// The value it checks it against corresponds to the `birthtime` field of `stat` on
/// Unix platforms and the `ftCreationTime` field on Windows platforms
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Created {
    At(i64),
    Before(i64),
    After(i64),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ignore {
    Files,
    Folders,
}

/// Finds files or directories that fit all of the criteria
/// 
/// The returned `Vec` can be empty if nothing was found
///
/// If it encounters any errors while getting info on files it will just log it
/// (assuming logging is turned on) and ignore the file where the error happened
pub fn find<P: AsRef<Path>>(paths_to_search_in: &[P], find_options: &FindOptions<'_>) -> Vec<PathBuf> {
    let paths_to_search_in: Vec<PathBuf> =
        paths_to_search_in.iter().filter_map(|path| {
            match canonicalize(path) {
                Ok(path) => Some(path),
                Err(e) => {
                    info!("Error accessing {:?} {} skipping this path", path.as_ref().display(), e);
                    None
                }
            }
        }).collect();

    trace!("find paths_to_search_in: {:?} find_options: {:?}", paths_to_search_in, find_options);

    let mut results = Vec::new();

    for path in paths_to_search_in {
        let mut matching_files: Vec<PathBuf> = WalkDir::new(path)
            .min_depth(find_options.min_depth_from_start)
            .max_depth(find_options.max_search_depth)
            .follow_links(find_options.follow_symlinks)
            .into_iter()
            .collect::<Vec<Result<_, _>>>()
            .into_par_iter()
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
                        // Not sure if this is the right way to go here. Maybe we should actually filter out the file since it errored?
                        if name.starts_with('.') {
                            return None;
                        }
                    }
                }

                // Checks if all Conditions match the file
                // If any do not match the file gets filtered out
                if find_options.options.iter().all(|option| option.evaluate(&entry).unwrap_or(false)) {
                    Some(entry.into_path())
                } else {
                    None
                }
            })
            .collect();

        if results.len() + matching_files.len() >= find_options.max_num_results {
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
