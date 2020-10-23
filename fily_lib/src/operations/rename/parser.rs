use std::{ffi::OsStr, path::Path, io};
use super::RenameFilesError;
use super::tokenizer::{FilenamePart, FilenameVariable};
use thiserror::Error;
#[allow(unused_imports)]
use log::{trace, debug, info, warn, error};

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Failed to get the filename because the path either end with .. or there is no final component")]
    FilenameNoBase,
    #[error("Failed to get the filename base because there is no filename")]
    NoFilename,
    #[error("Failed to convert a filename to UTF-8")]
    UTF8ConversionFailed,
    #[error("Something went wrong during tokenizing, check your template")]
    TokenizeErrorInTokens,
    #[error("Failed to get info on a file")]
    IOError(io::Error),
}

impl From<ParseError> for RenameFilesError {
    fn from(error: ParseError) -> Self {
        Self::ParsingError(error)
    }
}

/// Used to parse a sequence of `FilenamePart`s to a `String`
///
/// Use `Parser::builder` to build or instantiate directly with `Default` or `Parser::new` if you don't need to change
/// the starting point of the incrementing number from 0
#[derive(Eq, PartialEq, Debug, Clone, Copy, Default)]
pub struct Parser {
    incrementing_number: isize,
}

impl Parser {
    /// Creates and returns a new `Parser` initiated with the default impl
    #[inline]
    pub fn new() -> Self {
        Parser::default()
    }

    /// Creates and returns a new `ParserBuilder`
    #[inline]
    pub fn builder() -> ParserBuilder {
        ParserBuilder::new()
    }

    /// Turns a list of `FilenamePart`s into a string
    ///
    /// # Errors
    ///
    /// Returns an error if either there was a `FilenamePart::Error` in `tokens` or
    /// if something went wrong getting info on a file
    pub fn parse_filename<'a>(&mut self, tokens: &[FilenamePart<'a>], path: impl AsRef<Path>) -> Result<String, ParseError> {
        let mut parsed_filename = String::new();

        for token in tokens {
            match *token {
                FilenamePart::Constant(string) => parsed_filename.push_str(string),
                FilenamePart::Variable(variable) => parsed_filename.push_str(&self.parse_filename_variable(variable, &path)?),
                FilenamePart::Error => {
                    error!("A FilenamePart::Error got into parse_filename");
                    return Err(ParseError::TokenizeErrorInTokens);
                }
            };
        }

        Ok(parsed_filename)
    }

    /// Produces a string from a single `FilenameVariable`
    ///
    /// Output may change depending on where `path` points to
    fn parse_filename_variable(&mut self, variable: FilenameVariable, path: impl AsRef<Path>) -> Result<String, ParseError> {
        let path = path.as_ref();
        Ok(match variable {
            FilenameVariable::Filename => path.file_name()
                .ok_or_else(|| {
                    info!("Can't get filename of {:?} no base", path.display());
                    ParseError::FilenameNoBase
                })?
                .to_str()
                .ok_or_else(|| {
                    info!("Can't convert {:?} to UTF-8", path.display());
                    ParseError::UTF8ConversionFailed
                })?
                .to_string(),
            FilenameVariable::FilenameBase => path.file_stem()
                .ok_or_else(|| {
                    info!("Can't get filename of {:?} no filename", path.display());
                    ParseError::NoFilename
                })?
                .to_str()
                .ok_or_else(|| {
                    info!("Can't convert {:?} to UTF-8", path.display());
                    ParseError::UTF8ConversionFailed
                })?
                .to_string(),
            FilenameVariable::FilenameExtension => path.extension()
                .unwrap_or_else(|| OsStr::new(""))
                .to_str()
                .ok_or_else(|| {
                    info!("Can't convert {:?} to UTF-8", path.display());
                    ParseError::UTF8ConversionFailed
                })?
                .to_string(),
            FilenameVariable::FileSize => path.metadata()
                .map_err(|e| {
                    info!("Failed to get size of {:?} {}", path.display(), e);
                    ParseError::IOError(e)
                })?
                .len()
                .to_string(),
            FilenameVariable::IncrementingNumber => {
                let num = self.incrementing_number;
                self.incrementing_number += 1;
                num.to_string()
            },
        })
    }
}

/// Used to build a `Parser`
#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub struct ParserBuilder {
    parser: Parser,
}

impl ParserBuilder {
    /// Creates a new builder
    #[inline]
    pub fn new() -> Self {
        ParserBuilder {
            parser: Parser {
                incrementing_number: 0,
            }
        }
    }

    /// Sets the start of the incrementing number if its used
    ///
    /// Default is 0
    #[inline]
    pub fn incrementing_number(&mut self, num: isize) -> &mut Self {
        self.parser.incrementing_number = num;
        self
    }

    /// Builds and returns the resulting `Parser`
    #[inline]
    pub fn build(self) -> Parser {
        self.parser
    }
}
