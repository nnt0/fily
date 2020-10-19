use logos::{Lexer, Logos};
use thiserror::Error;
use super::RenameFilesError;
#[allow(unused_imports)]
use log::{trace, debug, info, warn, error};

#[derive(Error, PartialEq, Eq, Debug, Clone, Copy)]
pub enum TokenizeError {
    // TODO: Remove the UnknownError variant once we can return the actual error
    #[error("Something went wrong during tokenizing but we don't know what because that one library I use doesn't support returning errors containing enums and it's really annoying so this is a placeholder until it gets implemented. Good luck figuring out whats wrong with you input")]
    UnknownError,
    #[error("A variable was empty")]
    EmptyVariable,
    #[error("An unknown variable was passed")]
    UnknownVariable,
    #[error("Failed to parse the value of an option or there was none even though there should've been one")]
    MalformedOption,
    #[error("An unknown option was passed")]
    UnknownOption,
}

impl From<TokenizeError> for RenameFilesError {
    fn from(error: TokenizeError) -> Self {
        RenameFilesError::TokenizeError(error)
    }
}

#[derive(Logos, PartialEq, Eq, Debug, Clone, Copy)]
pub enum FilenamePart<'a> {
    #[regex("[^{}]+")]
    Constant(&'a str),
    #[regex(r"\{[^{}]*\}", tokenize_variable)]
    Variable(FilenameVariable),
    #[error]
    Error,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum FilenameVariable {
    Filename,
    FilenameExtension,
    FilenameBase,
    FileSize,
    IncrementingNumber,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Default)]
pub struct FilenameOptions {
    pub incrementing_number_starts_at: isize,
}

/// Turns a template for `rename` into a list of tokens and options for it
///
/// Read the docs of the `rename` function for an explanation on how the template has to look like
///
/// The options always get returned even if there were no options in `text`
///
/// # Errors
///
/// The `Vec` of `FilenamePart`s can include a `FilenamePart::Error` if there is an error in the template
///
/// Unfortunately it can't tell you what exactly went wrong because the library I'm using dosn't support
/// it yet...
///
/// The parsing of the options can fail. Look at the docs of the `rename` function to see how the options
/// should look like
#[inline]
pub fn tokenize(text: &str) -> (Vec<FilenamePart<'_>>, Result<FilenameOptions, TokenizeError>) {
    let parts: Vec<&str> = text.split('|').collect();
    let template = parts[0];
    let options = if parts.len() > 1 { &parts[1..] } else { &[] };

    (FilenamePart::lexer(template).spanned().map(|(part, _)| part).collect(), parse_options(options))
}

fn parse_options(options: &[&str]) -> Result<FilenameOptions, TokenizeError> {
    let mut filename_options = FilenameOptions::default();

    for option in options {
        match option.trim() {
            _ if option.starts_with("incrementing_number_starts_at") => {
                let start_num = if let Some(num) = option.splitn(2, '=').nth(1) {
                    match num.parse() {
                        Ok(num) => num,
                        Err(e) => {
                            debug!("Couldn't parse value of incrementing_number_starts_at {} skipping this option", e);
                            return Err(TokenizeError::MalformedOption);
                        }
                    }
                } else {
                    debug!("incrementing_number_starts_at was passed without a value skipping this option");
                    return Err(TokenizeError::MalformedOption);
                };

                filename_options.incrementing_number_starts_at = start_num;
            }
            _ => {
                debug!("Unknown option {:?} skipping it", option);
                return Err(TokenizeError::UnknownOption);
            }
        };
    }

    Ok(filename_options)
}

fn tokenize_variable<'a>(lex: &mut Lexer<'a, FilenamePart<'a>>) -> Result<FilenameVariable, TokenizeError> {
    let variable = lex.slice();
    let variable = &variable[1..variable.len() - 1];

    if variable.is_empty() {
        return Err(TokenizeError::EmptyVariable);
    }

    Ok(match variable {
        "filename" => FilenameVariable::Filename,
        "filename_extension" => FilenameVariable::FilenameExtension,
        "filename_base" => FilenameVariable::FilenameBase,
        "filesize" => FilenameVariable::FileSize,
        "incrementing_number" => FilenameVariable::IncrementingNumber,
        _ => return Err(TokenizeError::UnknownVariable),
    })
}
