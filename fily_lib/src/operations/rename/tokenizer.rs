use std::{fmt, error::Error};
use logos::Logos;
use super::RenameFilesError;
use crate::fily_err::FilyError;
#[allow(unused_imports)]
use log::{trace, debug, info, warn, error};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TokenizeError {
    // TODO: Remove the UnknownError variant once we can return the actual error

    /// Something went wrong during tokenizing but we don't know what because that one library I use doesn't support returning errors containing enums and it's really annoying so this is a placeholder until it gets implemented. Good luck figuring out whats wrong with you input
    UnknownError,

    /// An unknown variable was found
    UnknownVariable,
}

impl Error for TokenizeError {}

impl fmt::Display for TokenizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<FilyError<TokenizeError>> for RenameFilesError {
    fn from(err: FilyError<TokenizeError>) -> Self {
        RenameFilesError::TokenizeError(err)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Logos)]
pub enum FilenameToken<'a> {
    #[regex("[^{}]+")]
    Constant(&'a str),
    #[regex(r"\{[^{}]*\}", |var| { let var = var.slice(); &var[1..var.len()] })]
    Variable(&'a str),
    #[error]
    Error,
}

impl<'a> FilenameToken<'a> {
    pub fn tokenize(text: &'a str) -> Vec<Self> {
        FilenameToken::lexer(text).spanned().map(|(part, _)| part).collect()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FilenamePart<'a> {
    Constant(&'a str),
    Variable(FilenameVariable),
}

impl<'a> FilenamePart<'a> {
    pub fn from_text(text: &'a str) -> Result<Vec<Self>, FilyError<TokenizeError>> {
        let tokens = FilenameToken::tokenize(text);

        if tokens.contains(&FilenameToken::Error) {
            return Err(FilyError::new(TokenizeError::UnknownError, "Something went wrong during tokenizing"));
        }

        let result: Result<Vec<FilenamePart<'a>>, FilyError<TokenizeError>> = tokens.into_iter()
            .map(|token| Ok(match token {
                FilenameToken::Constant(string) => FilenamePart::Constant(string),
                FilenameToken::Variable(var) => FilenamePart::Variable(FilenameVariable::from_text(var)?),
                _ => unreachable!("Error variant in vec after checking for it"),
        })).collect();

        Ok(result?)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FilenameVariable {
    Filename,
    FilenameExtension,
    FilenameBase,
    FileSize,
    IncrementingNumber,
}

impl FilenameVariable {
    pub fn from_text(var: &str) -> Result<Self, FilyError<TokenizeError>> {
        Ok(match var {
            "filename" => FilenameVariable::Filename,
            "filename_extension" => FilenameVariable::FilenameExtension,
            "filename_base" => FilenameVariable::FilenameBase,
            "filesize" => FilenameVariable::FileSize,
            "incrementing_number" => FilenameVariable::IncrementingNumber,
            _ => return Err(FilyError::new_with_context(TokenizeError::UnknownVariable, || format!("Unknown variable {:?}", var))),
        })
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum OptionsParseError {
    /// Failed to parse the value of an option or there was no value even though there should've been one
    MalformedOption,

    /// An unknown option was passed
    UnknownOption,
}

impl Error for OptionsParseError {}

impl fmt::Display for OptionsParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<FilyError<OptionsParseError>> for RenameFilesError {
    fn from(err: FilyError<OptionsParseError>) -> Self {
        RenameFilesError::OptionsParsingError(err)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct FilenameOptions {
    pub incrementing_number_starts_at: isize,
}

impl FilenameOptions {
    /// Parses options for a `rename` template
    ///
    /// Read the docs of the `rename` function for an explanation on how the options format looks like
    ///
    /// If you have options which are already separated use the `FilenameOptions::parse_options` function
    ///
    /// # Errors
    ///
    /// This will fail if
    ///
    /// * the options aren't seperated by a `|`
    /// * the input includes an unknown option
    /// * if an option which requires a value does not have one or it failed to parse it
    pub fn new(text: &str) -> Result<Self, FilyError<OptionsParseError>> {
        let options: Vec<&str> = text.split('|').collect();

        FilenameOptions::parse_options(&options)
    }

    /// Parses options that are not separated with a `|`
    pub fn parse_options(options: &[&str]) -> Result<Self, FilyError<OptionsParseError>> {
        let mut filename_options = FilenameOptions::default();

        for option in options {
            match option.trim() {
                _ if option.starts_with("incrementing_number_starts_at") => {
                    let start_num = option.splitn(2, '=')
                        .nth(1)
                        .ok_or_else(|| FilyError::new(OptionsParseError::MalformedOption, "incrementing_number_starts_at was passed without a value"))?
                        .parse()
                        .map_err(|_| FilyError::new(OptionsParseError::MalformedOption, "Couldn't parse value of incrementing_number_starts_at"))?;

                    filename_options.incrementing_number_starts_at = start_num;
                }
                _ => return Err(FilyError::new_with_context(OptionsParseError::UnknownOption, || format!("Unknown option {:?}", option))),
            };
        }

        Ok(filename_options)
    }
}
