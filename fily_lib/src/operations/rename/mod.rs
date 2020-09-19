use std::{fs::{canonicalize, rename}, path::Path, error::Error, fmt};
#[allow(unused_imports)]
use log::{trace, debug, info, warn, error};

mod tokenizer;
use tokenizer::{tokenize, FilenamePart, TokenizeError};

mod parser;
use parser::Parser;

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum RenameFilesError {
    TokenizeError,
}

impl Error for RenameFilesError {}

impl fmt::Display for RenameFilesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Renames all files based on a template
pub fn rename_files<P: AsRef<Path>>(
    files_to_rename: &[P],
    new_filename_template: &str,
) -> Result<(), Box<dyn Error>> {
    let files_to_rename = {
        let mut files_to_rename_canonicalized = Vec::with_capacity(files_to_rename.len());

        for path in files_to_rename {
            files_to_rename_canonicalized.push(match canonicalize(path) {
                Ok(path) => path,
                Err(e) => {
                    info!("Error accessing {:?} {} skipping this file", path.as_ref().display(), e);
                    continue;
                }
            });
        }

        files_to_rename_canonicalized
    };

    trace!("rename_files files_to_rename: {:?} new_filename_template: {}", files_to_rename, new_filename_template);

    let (filename_tokenized, options) = tokenize(new_filename_template);

    if filename_tokenized.contains(&FilenamePart::Error) {
        info!("Tokenize error");
        // TODO: This should just return the actual Error but we have to wait until Logos supports returning actual errors
        return Err(Box::from(RenameFilesError::TokenizeError));
    }

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
