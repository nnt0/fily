use std::{error::Error, fmt};
use logos::{Lexer, Logos};
// use super::RenameFilesError;
#[allow(unused_imports)]
use log::{trace, debug, info, warn, error};

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum TokenizeError {
    // TODO: Remove the UnknownError variant once we can return the actual error
    UnknownError,
    EmptyVariable,
    UnknownVariable,
    MalformedOption,
}

impl Error for TokenizeError {}

impl fmt::Display for TokenizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// impl From<TokenizeError> for RenameFilesError {
//     fn from(error: TokenizeError) -> Self {
//         RenameFilesError::TokenizeError(error)
//     }
// }

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
                return Err(TokenizeError::MalformedOption);
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
