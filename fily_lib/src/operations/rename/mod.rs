use std::{fs::rename, path::Path, fmt, error::Error};
use crate::fily_err::FilyError;
#[allow(unused_imports)]
use log::{trace, debug, info, warn, error};

mod tokenizer;
use tokenizer::{FilenamePart, TokenizeError, FilenameOptions, OptionsParseError};

mod parser;
use parser::{Parser, ParseError};

#[derive(Debug)]
pub enum RenameFilesError {
    TokenizeError(FilyError<TokenizeError>),
    ParsingError(FilyError<ParseError>),
    OptionsParsingError(FilyError<OptionsParseError>),
}

impl Error for RenameFilesError {}

impl fmt::Display for RenameFilesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Renames all files based on a template
///
/// Template rules and behaviour:
///
/// Normal text will just be used directly in the name, anything within `{` and `}` will be interpreted as a variable which value can change for every file. The value of the variable will be inserted where the variable was in the template.
///
/// You can choose from a couple different variables:
///
/// * `filename` The current filename
/// * `filename_extension` The extension of the filename without the `.`. If there is no extension this will be an empty string
/// * `filename_base` The base of the filename. If there is no extension this is the same as `filename`
/// * `filesize` The size of the file in bytes
/// * `incrementing_number` A number that will increment by one after each file. By default it starts at 0 but you can change the starting point
///
/// There are also options for the template. Everything after the first `|` will be interpreted as such. Currently there is only one option:
///
/// * `incrementing_number_starts_at` Sets the starting point of `incrementing_number`. The number can be negative. Should be used like this: `{incrementing_number}|incrementing_number_starts_at=42`
///
/// # Errors
///
/// This fails if either the template or the options for renaming have an error
pub fn rename_files<P: AsRef<Path>>(files_to_rename: &[P], new_filename_template: &str) -> Result<(), RenameFilesError> {
    let files_to_rename: Vec<&Path> = files_to_rename.iter().map(AsRef::as_ref).collect();

    trace!("rename_files files_to_rename: {:?} new_filename_template: {}", files_to_rename, new_filename_template);

    let text_template_and_options: Vec<&str> = new_filename_template.splitn(2, '|').collect();
    let text_template = text_template_and_options[0];

    let filename_template = FilenamePart::from_text(text_template)?;

    let options = if let Some(text_options) = text_template_and_options.get(1) {
        FilenameOptions::new(text_options)?
    } else {
        FilenameOptions::default()
    };

    let mut parser = Parser::builder().incrementing_number(options.incrementing_number_starts_at).build();

    for path in files_to_rename {
        let filename_new = match parser.parse_filename(&filename_template, &path) {
            Ok(filename) => filename,
            Err(e) => {
                info!("parse_filename failed {}", e);
                continue;
            }
        };

        let old_path = path;
        let new_path = old_path.with_file_name(&filename_new);

        match rename(&old_path, &new_path) {
            Ok(()) => info!("Renamed {:?} to {:?}", old_path.display(), new_path.display()),
            Err(e) => info!("Failed to rename {:?} to {:?} {}", old_path.display(), new_path.display(), e),
        };
    }

    Ok(())
}
