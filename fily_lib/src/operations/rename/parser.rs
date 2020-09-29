use std::{ffi::OsStr, path::Path, error::Error};
use crate::operations::rename::{tokenizer::{FilenamePart, FilenameVariable}, RenameFilesError};
#[allow(unused_imports)]
use log::{trace, debug, info, warn, error};

/// Used to parse a sequence of `FilenamePart`s to a `String`
///
/// Use `Parser::builder` to build
#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub struct Parser {
    incrementing_number: isize,
}

impl Parser {
    #[inline]
    pub fn builder() -> ParserBuilder {
        ParserBuilder::new()
    }

    /// Turns a list of `FilenamePart`s into a string
    pub fn parse_filename<'a, P: AsRef<Path>>(&mut self, tokens: &[FilenamePart<'a>], path: P) -> Result<String, Box<dyn Error>> {
        let mut parsed_filename = String::new();

        for token in tokens {
            match *token {
                FilenamePart::Constant(string) => parsed_filename.push_str(string),
                FilenamePart::Variable(variable) => parsed_filename.push_str(&self.parse_filename_variable(variable, &path)?),
                FilenamePart::Error => {
                    error!("A FilenamePart::Error got into parse_filename");
                    return Err(Box::from(RenameFilesError::TokenizeError));
                }
            };
        }

        Ok(parsed_filename)
    }

    fn parse_filename_variable<P: AsRef<Path>>(&mut self, variable: FilenameVariable, path: P) -> Result<String, Box<dyn Error>> {
        let path = path.as_ref();
        Ok(match variable {
            FilenameVariable::Filename => path.file_name()
                .ok_or_else(|| format!("Can't get filename of {:?} no base", path.display()))?
                .to_str()
                .ok_or_else(|| format!("Can't convert {:?} to UTF-8", path.display()))?
                .to_string(),
            FilenameVariable::FilenameBase => path.file_stem()
                .ok_or_else(|| format!("Can't get filename of {:?} no base", path.display()))?
                .to_str()
                .ok_or_else(|| format!("Can't convert {:?} to UTF-8", path.display()))?
                .to_string(),
            FilenameVariable::FilenameExtension => path.extension()
                .unwrap_or_else(|| OsStr::new(""))
                .to_str()
                .ok_or_else(|| format!("Can't convert {:?} to UTF-8", path.display()))?
                .to_string(),
            FilenameVariable::FileSize => path.metadata()?
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

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub struct ParserBuilder {
    parser: Parser,
}

impl ParserBuilder {
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

    #[inline]
    pub fn build(self) -> Parser {
        self.parser
    }
}
