use std::{fs::metadata, path::Path, convert::TryFrom};
use super::{Filename, Filesize, FilePath, Modified, Accessed, Created, SearchCriteria};
use regex::Regex;
use filetime::FileTime;
#[allow(unused_imports)]
use log::{trace, debug, info, warn, error};

/// Used to build expressions which are used to determine if a file matches the search criteria
///
/// A specific file can be checked with `evaluate()`
///
/// If you expect a certain criteria to be more likely to evaluate to false or true
/// you should try to always put the one that you expect to be more likely to evaluate to
/// false on the left site of an `And` condition and the one you expect to be more likely
/// to evaluate to true on the left site of an `Or` condition. Additionally, try to put
/// the ones which you expect to fail the condition as high up as possible (i.e. not nested 10 layers deep).
/// That way we can make use of short circuting and possibly reduce the time it takes to
/// evaluate the condition.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Condition<T> {
    Not(Box<Condition<T>>),
    And(Box<Condition<T>>, Box<Condition<T>>),
    Or(Box<Condition<T>>, Box<Condition<T>>),
    Value(T),
}

impl<'a> TryFrom<&str> for Condition<SearchCriteria<'a>> {
    type Error = &'static str;

    fn try_from(_condition_str: &str) -> Result<Self, Self::Error> {
        todo!("no idea how to implement this");
    }
}

impl<'a> Condition<SearchCriteria<'a>> {
    /// Checks if the file that `path` points to matches the condition
    ///
    /// Returns `Ok(true)` if it does and `Ok(false)` if it doesn't. If an error
    /// occured while trying to get info on the file it'll return `Err(())` and log
    /// the error (assuming logging is turned on)
    pub fn evaluate(&self, path: &impl AsRef<Path>) -> Result<bool, ()> {
        // TODO: Maybe we should return an actual eror here?
        match self {
            Self::And(condition1, condition2) => Ok(condition1.evaluate(path)? && condition2.evaluate(path)?),
            Self::Not(condition) => Ok(!condition.evaluate(path)?),
            Self::Or(condition1, condition2) => Ok(condition1.evaluate(path)? || condition2.evaluate(path)?),
            Self::Value(search_criteria) => {
                let path = path.as_ref();

                Ok(match search_criteria {
                    SearchCriteria::Filename(filename_options) => self.filename_matches(path, filename_options)?,
                    SearchCriteria::Filesize(filesize_options) => self.filesize_matches(path, filesize_options)?,
                    SearchCriteria::FilePath(filepath_options) => self.filepath_matches(path, filepath_options)?,
                    SearchCriteria::FilenameRegex(filename_regex) => self.filename_regex_matches(path, filename_regex)?,
                    SearchCriteria::Modified(modified_options) => self.modification_time_matches(path, modified_options)?,
                    SearchCriteria::Accessed(access_options) => self.access_time_matches(path, access_options)?,
                    SearchCriteria::Created(creation_options) => self.creation_time_matches(path, creation_options)?,
                })
            }
        }
    }

    fn filename_matches(&self, path: &Path, filename_options: &Filename<'_>) -> Result<bool, ()> {
        let filename = match path.file_name() {
            Some(filename_osstr) => match filename_osstr.to_str() {
                Some(filename) => filename,
                None => {
                    info!("Failed to convert filename of {:?} to UTF-8 skipping entry", path.display());
                    return Err(());
                }
            }
            None => {
                info!("Failed to get filename of {:?} skipping entry", path.display());
                return Err(());
            }
        };

        Ok(match *filename_options {
            Filename::Exact(exact_name) => filename == exact_name,
            Filename::Contains(substring) => filename.contains(substring),
        })
    }

    fn filesize_matches(&self, path: &Path, filesize_options: &Filesize) -> Result<bool, ()> {
        let filesize = match metadata(path) {
            Ok(metadata) => metadata.len(),
            Err(e) => {
                info!("IO Error {:?} {:?} ignoring this file", e, path.display());
                return Err(());
            }
        };

        Ok(match *filesize_options {
            Filesize::Exact(exact_size) => filesize == exact_size,
            Filesize::Over(over_this_size) => filesize > over_this_size,
            Filesize::Under(under_this_size) => filesize < under_this_size,
        })
    }

    fn filepath_matches(&self, path: &Path, filepath_options: &FilePath<'_>) -> Result<bool, ()> {
        let path = match path.as_os_str().to_str() {
            Some(path) => path,
            None => {
                info!("Failed to convert path {:?} to UTF-8 skipping entry", path.display());
                return Err(());
            }
        };

        Ok(match *filepath_options {
            FilePath::Exact(exact_path) => path == exact_path,
            FilePath::Contains(substring) => path.contains(substring),
        })
    }

    fn filename_regex_matches(&self, path: &Path, filename_regex: &Regex) -> Result<bool, ()> {
        let filename = match path.file_name() {
            Some(filename_osstr) => match filename_osstr.to_str() {
                Some(filename) => filename,
                None => {
                    info!("Failed to convert filename of {:?} to UTF-8 skipping entry", path.display());
                    return Err(());
                }
            }
            None => {
                info!("Failed to get filename of {:?} skipping entry", path.display());
                return Err(());
            }
        };

        Ok(filename_regex.is_match(filename))
    }

    fn modification_time_matches(&self, path: &Path, modified_options: &Modified) -> Result<bool, ()> {
        let metadata = match metadata(path) {
            Ok(metadata) => metadata,
            Err(e) => {
                info!("Failed to get metadata of {:?} {} skipping entry", path.display(), e);
                return Err(());
            }
        };

        let last_modification_time = FileTime::from_last_modification_time(&metadata).unix_seconds();

        Ok(match *modified_options {
            Modified::At(at_this_time) => last_modification_time == at_this_time,
            Modified::Before(before_this_time) => last_modification_time < before_this_time,
            Modified::After(after_this_time) => last_modification_time > after_this_time,
        })
    }

    fn access_time_matches(&self, path: &Path, access_options: &Accessed) -> Result<bool, ()> {
        let metadata = match metadata(path) {
            Ok(metadata) => metadata,
            Err(e) => {
                info!("Failed to get metadata of {:?} {} skipping entry", path.display(), e);
                return Err(());
            }
        };

        let last_access_time = FileTime::from_last_modification_time(&metadata).unix_seconds();

        Ok(match *access_options {
            Accessed::At(at_this_time) => last_access_time == at_this_time,
            Accessed::Before(before_this_time) => last_access_time < before_this_time,
            Accessed::After(after_this_time) => last_access_time > after_this_time,
        })
    }

    fn creation_time_matches(&self, path: &Path, creation_options: &Created) -> Result<bool, ()> {
        let metadata = match metadata(path) {
            Ok(metadata) => metadata,
            Err(e) => {
                info!("Failed to get metadata of {:?} {} skipping entry", path.display(), e);
                return Err(());
            }
        };

        let creation_time = FileTime::from_last_modification_time(&metadata).unix_seconds();

        Ok(match *creation_options {
            Created::At(at_this_time) => creation_time == at_this_time,
            Created::Before(before_this_time) => creation_time < before_this_time,
            Created::After(after_this_time) => creation_time > after_this_time,
        })
    }
}
