use std::{fs::rename, path::Path};
use thiserror::Error;
#[allow(unused_imports)]
use log::{trace, debug, info, warn, error};

mod tokenizer;
use tokenizer::{tokenize, FilenamePart, TokenizeError};

mod parser;
use parser::{Parser, ParseError};

#[derive(Error, Debug)]
pub enum RenameFilesError {
    #[error("Something went wrong during tokenizing")]
    TokenizeError(TokenizeError),
    #[error("Something went wrong during parsing")]
    ParsingError(ParseError),
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

    let (filename_tokenized, options) = tokenize(new_filename_template);

    if filename_tokenized.contains(&FilenamePart::Error) {
        info!("Tokenize error");
        // TODO: This should just return the actual Error but we have to wait until Logos supports returning actual errors
        return Err(RenameFilesError::TokenizeError(TokenizeError::UnknownError));
    }

    let options = options?;

    let mut parser = Parser::builder().incrementing_number(options.incrementing_number_starts_at).build();

    for path in files_to_rename {
        let filename_new = match parser.parse_filename(&filename_tokenized, &path) {
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
