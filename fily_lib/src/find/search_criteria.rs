use std::{num::ParseIntError, convert::TryFrom, error::Error, fmt};

/// Used to specify a criteria a file has to match
///
/// There are criterias for the filename, filesize, path,
/// filename but with a regex that has to match, the last time it was modified,
/// the last time it was accessed and the time it was created
#[derive(Debug, Clone)]
pub enum SearchCriteria {
    Filename(Filename),
    Filesize(Filesize),
    FilePath(FilePath),
    FilenameRegex(regex::Regex),
    Modified(Modified),
    Accessed(Accessed),
    Created(Created)
}

#[derive(Debug, Clone, PartialEq)]
pub enum SearchCriteriaParsingError {
    /// The criteria is missing the =\"<value>\" part
    NoValue,

    /// There is at least one missing double quote at the start or the end of the value
    MissingDoubleQuotes,

    /// The criteria passed isn't known
    UnknownCriteria,

    /// Error parsing the value to a number
    MalformedNumber,

    /// Error parsing the regex
    MalformedRegex(regex::Error),
}

impl Error for SearchCriteriaParsingError {}

impl fmt::Display for SearchCriteriaParsingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<ParseIntError> for SearchCriteriaParsingError {
    fn from(_: ParseIntError) -> Self {
        Self::MalformedNumber
    }
}

impl From<regex::Error> for SearchCriteriaParsingError {
    fn from(error: regex::Error) -> Self {
        Self::MalformedRegex(error)
    }
}

impl TryFrom<&str> for SearchCriteria {
    type Error = SearchCriteriaParsingError;

    // For some reason the <value> in between the "" doesn't get shown in docs if the backshlash isn't there...
    /// Expects a criteria in this format: <criteria_name>="\<value>"
    ///
    /// Possible criterias are:
    /// * `filename_exact`
    /// * `filename_contains`
    /// * `filesize_exact`
    /// * `filesize_over`
    /// * `filesize_under`
    /// * `filepath_exact`
    /// * `filepath_contains`
    /// * `filenameregex`
    /// * `modified_at`
    /// * `modified_before`
    /// * `modified_after`
    /// * `accessed_at`
    /// * `accessed_before`
    /// * `accessed_after`
    /// * `created_at`
    /// * `created_before`
    /// * `created_after`
    ///
    /// `filesize_*` and `filepath_*` expect a string
    ///
    /// `filesize_*` expects a number that is >= 0
    ///
    /// `filenameregex` expects a regex in string form
    ///
    /// `modified_*`, `accessed_*` and `created_*` expect a number that is a timestamp relative to
    /// the unix epoch in seconds. This number can be negative
    fn try_from(search_criteria_str: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = search_criteria_str.trim().splitn(2, '=').collect();

        if parts.len() == 1 {
            return Err(SearchCriteriaParsingError::NoValue);
        }

        let criteria_name = parts[0];

        if !criteria_name.starts_with('"') || !criteria_name.ends_with('"') {
            return Err(SearchCriteriaParsingError::MissingDoubleQuotes);
        }

        let criteria_name = &criteria_name[1..criteria_name.len() - 1];
        let value = parts[1].to_string();

        Ok(match criteria_name {
            "filename_exact" => SearchCriteria::Filename(Filename::Exact(value)),
            "filename_contains" => SearchCriteria::Filename(Filename::Contains(value)),
            "filesize_exact" => {
                let size = value.parse()?;

                SearchCriteria::Filesize(Filesize::Exact(size))
            }
            "filesize_over" => {
                let size = value.parse()?;

                SearchCriteria::Filesize(Filesize::Over(size))
            }
            "filesize_under" => {
                let size = value.parse()?;

                SearchCriteria::Filesize(Filesize::Under(size))
            }
            "filepath_exact" => SearchCriteria::FilePath(FilePath::Exact(value)),
            "filepath_contains" => SearchCriteria::FilePath(FilePath::Contains(value)),
            "filenameregex" => {
                let regex = regex::Regex::new(&value)?;

                SearchCriteria::FilenameRegex(regex)
            }
            "modified_at" => {
                let timestamp = value.parse()?;

                SearchCriteria::Modified(Modified::At(timestamp))
            }
            "modified_before" => {
                let timestamp = value.parse()?;

                SearchCriteria::Modified(Modified::Before(timestamp))
            }
            "modified_after" => {
                let timestamp = value.parse()?;

                SearchCriteria::Modified(Modified::After(timestamp))
            }
            "accessed_at" => {
                let timestamp = value.parse()?;

                SearchCriteria::Accessed(Accessed::At(timestamp))
            }
            "accessed_before" => {
                let timestamp = value.parse()?;

                SearchCriteria::Accessed(Accessed::Before(timestamp))
            }
            "accessed_after" => {
                let timestamp = value.parse()?;

                SearchCriteria::Accessed(Accessed::After(timestamp))
            }
            "created_at" => {
                let timestamp = value.parse()?;

                SearchCriteria::Created(Created::At(timestamp))
            }
            "created_before" => {
                let timestamp = value.parse()?;

                SearchCriteria::Created(Created::Before(timestamp))
            }
            "created_after" => {
                let timestamp = value.parse()?;

                SearchCriteria::Created(Created::After(timestamp))
            }
            _ => return Err(SearchCriteriaParsingError::UnknownCriteria),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Filename {
    Exact(String),
    Contains(String),
}

/// Filesize is in bytes
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Filesize {
    Exact(u64),
    Over(u64),
    Under(u64),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FilePath {
    Exact(String),
    Contains(String),
}

/// Time is in seconds and relative to the unix epoch (1970-01-01T00:00:00Z)
///
/// The value it checks it against corresponds to the `mtime` field of `stat` on
/// Unix platforms and the `ftLastWriteTime` field on Windows platforms
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Modified {
    At(i64),
    Before(i64),
    After(i64),
}

/// Time is in seconds and relative to the unix epoch (1970-01-01T00:00:00Z)
///
/// The value it checks it against corresponds to the `atime` field of `stat` on
/// Unix platforms and the `ftLastAccessTime` field on Windows platforms
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Accessed {
    At(i64),
    Before(i64),
    After(i64),
}

/// Time is in seconds and relative to the unix epoch (1970-01-01T00:00:00Z)
///
/// The value it checks it against corresponds to the `birthtime` field of `stat` on
/// Unix platforms and the `ftCreationTime` field on Windows platforms
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Created {
    At(i64),
    Before(i64),
    After(i64),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Ignore {
    Files,
    Folders,
}
