use logos::{Lexer, Logos};
#[allow(unused_imports)]
use log::{trace, debug, info, warn, error};

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum TokenizeError {
    EmptyVariable,
    UnknownVariable,
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

#[inline]
pub fn tokenize(text: &str) -> (Vec<FilenamePart<'_>>, FilenameOptions) {
    let parts: Vec<&str> = text.split('|').collect();
    let template = parts[0];
    let options = if parts.len() > 1 { &parts[1..] } else { &[] };

    (FilenamePart::lexer(template).spanned().map(|(part, _)| part).collect(), parse_options(options))
}

fn parse_options(options: &[&str]) -> FilenameOptions {
    let mut filename_options = FilenameOptions::default();

    for option in options {
        match option.trim() {
            _ if option.starts_with("incrementing_number_starts_at") => {
                let start_num = match option.splitn(2, '=').collect::<Vec<&str>>().get(1) {
                    Some(num) => match num.parse() {
                        Ok(num) => num,
                        Err(e) => {
                            debug!("Couldn't parse value of incrementing_number_starts_at {} skipping this option", e);
                            continue;
                        }
                    }
                    None => {
                        debug!("incrementing_number_starts_at was passed without a value skipping this option");
                        continue;
                    }
                };
                filename_options.incrementing_number_starts_at = start_num;
            }
            _ => {
                debug!("Unknown option {:?} skipping it", option);
                continue;
            }
        };
    }

    filename_options
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
