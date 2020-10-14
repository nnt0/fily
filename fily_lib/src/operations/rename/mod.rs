use std::{fs::{canonicalize, rename}, path::{Path, PathBuf}};
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
/// # Errors
///
/// This fails if either the template or the options for renaming have an error
pub fn rename_files<P: AsRef<Path>>(files_to_rename: &[P], new_filename_template: &str) -> Result<(), TokenizeError> {
    let files_to_rename: Vec<PathBuf> =
        files_to_rename.iter().filter_map(|path| {
            match canonicalize(path) {
                Ok(path) => Some(path),
                Err(e) => {
                    info!("Error accessing {:?} {} skipping this path", path.as_ref().display(), e);
                    None
                }
            }
        }).collect();

    trace!("rename_files files_to_rename: {:?} new_filename_template: {}", files_to_rename, new_filename_template);

    let (filename_tokenized, options) = tokenize(new_filename_template);

    if filename_tokenized.contains(&FilenamePart::Error) {
        info!("Tokenize error");
        // TODO: This should just return the actual Error but we have to wait until Logos supports returning actual errors
        return Err(TokenizeError::UnknownError);
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
