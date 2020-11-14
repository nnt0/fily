use std::{io, fmt, error::Error};
use super::{Filename, Filesize, FilePath, Modified, Accessed, Created, SearchCriteria};
use regex::Regex;
use filetime::FileTime;
use walkdir::DirEntry;
use crate::fily_err::{Context, FilyError, PathOrFilenameError};
#[allow(unused_imports)]
use log::{trace, debug, info, warn, error};

/// Used to build expressions which are used to determine if a file matches the search criteria
///
/// A specific file can be checked with the `evaluate` function
///
/// If you expect a certain criteria to be more likely to evaluate to false or true
/// you should try to always put the one that you expect to be more likely to evaluate to
/// false on the left site of an `And` condition and the one you expect to be more likely
/// to evaluate to true on the left site of an `Or` condition. Additionally, try to put
/// the ones which you expect to fail the condition as high up as possible (i.e. not nested 10 layers deep).
/// That way we can make use of short circuting and possibly reduce the time it takes to
/// evaluate the condition.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Condition<T> {
    Not(Box<Condition<T>>),
    And(Box<Condition<T>>, Box<Condition<T>>),
    Or(Box<Condition<T>>, Box<Condition<T>>),
    Value(T),
}

#[derive(Debug)]
pub enum ConditionEvalError {
    PathErr(FilyError<PathOrFilenameError>),
    IOErr(FilyError<io::Error>),
}

impl Error for ConditionEvalError {}

impl fmt::Display for ConditionEvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<FilyError<PathOrFilenameError>> for ConditionEvalError {
    fn from(err: FilyError<PathOrFilenameError>) -> Self {
        ConditionEvalError::PathErr(err)
    }
}

impl From<FilyError<io::Error>> for ConditionEvalError {
    fn from(err: FilyError<io::Error>) -> Self {
        ConditionEvalError::IOErr(err)
    }
}

impl Condition<SearchCriteria> {
    /// Checks if the file that `dir_entry` points to matches the condition
    ///
    /// Returns `true` if it does and `false` if it doesn't
    ///
    /// # Errors
    ///
    /// Fails if a file operation fails. i.e. Getting the filename, filesize...
    pub fn evaluate(&self, dir_entry: &DirEntry) -> Result<bool, ConditionEvalError> {
        match self {
            Self::And(condition1, condition2) => Ok(condition1.evaluate(dir_entry)? && condition2.evaluate(dir_entry)?),
            Self::Not(condition) => Ok(!condition.evaluate(dir_entry)?),
            Self::Or(condition1, condition2) => Ok(condition1.evaluate(dir_entry)? || condition2.evaluate(dir_entry)?),
            Self::Value(search_criteria) => {
                Ok(match search_criteria {
                    SearchCriteria::Filename(filename_options) => Self::filename_matches(dir_entry, filename_options)?,
                    SearchCriteria::Filesize(filesize_options) => Self::filesize_matches(dir_entry, filesize_options)?,
                    SearchCriteria::FilePath(filepath_options) => Self::filepath_matches(dir_entry, filepath_options)?,
                    SearchCriteria::FilenameRegex(filename_regex) => Self::filename_regex_matches(dir_entry, filename_regex)?,
                    SearchCriteria::Modified(modified_options) => Self::modification_time_matches(dir_entry, modified_options)?,
                    SearchCriteria::Accessed(access_options) => Self::access_time_matches(dir_entry, access_options)?,
                    SearchCriteria::Created(creation_options) => Self::creation_time_matches(dir_entry, creation_options)?,
                })
            }
        }
    }

    fn filename_matches(dir_entry: &DirEntry, filename_options: &Filename) -> Result<bool, FilyError<PathOrFilenameError>> {
        let path = dir_entry.path();
        let filename = path.file_name()
            .ok_or_else(|| FilyError::new_with_context(PathOrFilenameError::NoFilename, || format!("Failed to get filename of {:?}", path.display())))?
            .to_str()
            .ok_or_else(|| FilyError::new_with_context(PathOrFilenameError::UTF8ConversionFailed, || format!("Failed to convert filename of {:?} to UTF-8", path.display())))?;

        Ok(match filename_options {
            Filename::Exact(exact_name) => filename == exact_name,
            Filename::Contains(substring) => filename.contains(substring),
        })
    }

    fn filesize_matches(dir_entry: &DirEntry, filesize_options: &Filesize) -> Result<bool, FilyError<io::Error>> {
        let filesize = dir_entry.metadata()
            .map_err(io::Error::from)
            .with_context(|| format!("Failed to get metadata of {:?}", dir_entry.path().display()))?
            .len();

        Ok(match *filesize_options {
            Filesize::Exact(exact_size) => filesize == exact_size,
            Filesize::Over(over_this_size) => filesize > over_this_size,
            Filesize::Under(under_this_size) => filesize < under_this_size,
        })
    }

    fn filepath_matches(dir_entry: &DirEntry, filepath_options: &FilePath) -> Result<bool, FilyError<PathOrFilenameError>> {
        let path = dir_entry.path();
        let path = path.to_str()
            .ok_or_else(|| FilyError::new_with_context(PathOrFilenameError::UTF8ConversionFailed, || format!("Failed to convert path {:?} to UTF-8", path.display())))?;

        Ok(match filepath_options {
            FilePath::Exact(exact_path) => path == exact_path,
            FilePath::Contains(substring) => path.contains(substring),
        })
    }

    fn filename_regex_matches(dir_entry: &DirEntry, filename_regex: &Regex) -> Result<bool, FilyError<PathOrFilenameError>> {
        let path = dir_entry.path();
        let filename = path.file_name()
            .ok_or_else(|| FilyError::new_with_context(PathOrFilenameError::NoFilename, || format!("Failed to get filename of {:?}", path.display())))?
            .to_str()
            .ok_or_else(|| FilyError::new_with_context(PathOrFilenameError::UTF8ConversionFailed, || format!("Failed to convert filename of {:?} to UTF-8", path.display())))?;

        Ok(filename_regex.is_match(filename))
    }

    fn modification_time_matches(dir_entry: &DirEntry, modified_options: &Modified) -> Result<bool, FilyError<io::Error>> {
        let metadata = dir_entry.metadata()
            .map_err(io::Error::from)
            .with_context(|| format!("Failed to get metadata of {:?}", dir_entry.path().display()))?;

        let last_modification_time = FileTime::from_last_modification_time(&metadata).unix_seconds();

        Ok(match *modified_options {
            Modified::At(at_this_time) => last_modification_time == at_this_time,
            Modified::Before(before_this_time) => last_modification_time < before_this_time,
            Modified::After(after_this_time) => last_modification_time > after_this_time,
        })
    }

    fn access_time_matches(dir_entry: &DirEntry, access_options: &Accessed) -> Result<bool, FilyError<io::Error>> {
        let metadata = dir_entry.metadata()
            .map_err(io::Error::from)
            .with_context(|| format!("Failed to get metadata of {:?}", dir_entry.path().display()))?;

        let last_access_time = FileTime::from_last_access_time(&metadata).unix_seconds();

        Ok(match *access_options {
            Accessed::At(at_this_time) => last_access_time == at_this_time,
            Accessed::Before(before_this_time) => last_access_time < before_this_time,
            Accessed::After(after_this_time) => last_access_time > after_this_time,
        })
    }

    fn creation_time_matches(dir_entry: &DirEntry, creation_options: &Created) -> Result<bool, FilyError<io::Error>> {
        let metadata = dir_entry.metadata()
            .map_err(io::Error::from)
            .with_context(|| format!("Failed to get metadata of {:?}", dir_entry.path().display()))?;

        let creation_time = FileTime::from_creation_time(&metadata)
            .ok_or_else(|| FilyError::new_with_context(io::Error::new(io::ErrorKind::Other, "Unsupported"), || format!("Failed to get creation time of {:?}", dir_entry.path().display())))?
            .unix_seconds();

        Ok(match *creation_options {
            Created::At(at_this_time) => creation_time == at_this_time,
            Created::Before(before_this_time) => creation_time < before_this_time,
            Created::After(after_this_time) => creation_time > after_this_time,
        })
    }
}
