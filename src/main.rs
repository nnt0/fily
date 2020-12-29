#![warn(clippy::cargo, clippy::pedantic)]
#![warn(rust_2018_idioms)]

use std::{error::Error, io::{self, stdin, Read, BufRead}};
#[allow(unused_imports)]
use log::{trace, debug, info, warn, error};

use fily_lib::{
    rename::rename_files,
    duplicates::{find_duplicate_files, find_duplicate_files_hash},
    find::{find, FindOptionsBuilder},
    move_files::move_files,
    similar_images::{find_similar_images, SimilarImagesOptions},
    check_image_formats::check_image_formats,
    delete::{delete, safe_delete},
};

mod cli_options;

use cli_options::{CLIOptions, Subcommand};

// TODO?: create a "create_file" module? How would that work? Naming? Contents?
// TODO?: create "fill_file_with" module? what contents? where do we get them from?
// TODO?: create a check_encoding module? checks if the input text (or text in file) has broken codepoints in it. take what encoding it is as input for each file?
// TODO: actual error reporting on tokenizing rename template
// TODO: find
//       * add TryFrom<&str> for Condition<SearchCriteria<'a>> so you're able to make arbitrary combinations of conditions. Need a parser for that?
//       * --exec and --exec_dir commands?
//       * add only_return_directories flag? As in don't return the actual file but the directory it's in. This could enable some short circuting
//       * max_num_results_per_folder option? do we include the results in subfolders or for every individual folder?
//       * add IsFile and IsFolder SearchCriteria?

fn main() -> Result<(), Box<dyn Error>> {
    // Doing this so the Display impl of the error gets used
    // It isn't shown if we just return the error
    if let Err(e) = start() {
        eprintln!("{}", e);
        return Err(e);
    }

    Ok(())
}

fn start() -> Result<(), Box<dyn Error>> {
    let options = CLIOptions::new()?;

    if let Err(e) = setup_logger(options.log_level) {
        eprintln!("Error while setting up logging {}", e);

        if options.strict_logging {
            return Err(Box::from("Failed to set up logging"));
        }
    }

    match options.subcommand {
        Subcommand::Find {
            paths_to_search_in,
            conditions,
            max_num_results,
            max_search_depth,
            min_depth_from_start,
            ignore,
            ignore_hidden_files,
            follow_symlinks,
            output_separator,
        } => {
            let mut find_options_builder = FindOptionsBuilder::new();

            find_options_builder.add_conditions(conditions)
                .set_max_num_results(max_num_results)
                .set_max_search_depth(max_search_depth)
                .set_min_depth_from_start(min_depth_from_start)
                .set_ignored_files(ignore)
                .set_ignore_hidden_files(ignore_hidden_files)
                .set_follow_symlinks(follow_symlinks);

            let find_options = find_options_builder.build();

            let results = find(&paths_to_search_in, &find_options);

            for (path, err) in results.1 {
                info!("{:?} {}", path.display(), err);
            }

            println!("{}", results.0
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<String>>()
                .join(&output_separator)
            );
        }
        Subcommand::Rename {
            template,
        } => {
            let files_to_rename = if let Some(separator) = options.input_path_separator {
                get_stdin_split(&separator)?
            } else {
                get_stdin_as_lines()?
            };

            rename_files(&files_to_rename, &template)?;
        }
        Subcommand::Duplicates {
            use_hash_version,
        } => {
            let files_to_check = if let Some(separator) = options.input_path_separator {
                get_stdin_split(&separator)?
            } else {
                get_stdin_as_lines()?
            };

            let result;

            if use_hash_version {
                result = find_duplicate_files_hash(&files_to_check);
            } else {
                result = find_duplicate_files(&files_to_check);
            }

            for (path, err) in result.1 {
                info!("{:?} {}", path.display(), err);
            }

            println!("{}", result.0
                .iter()
                .map(|duplicates| format!("{}, {}", duplicates.0.display(), duplicates.1.display()))
                .collect::<Vec<String>>()
                .join("\n")
            );
        }
        Subcommand::Move {
            move_to,
        } => {
            let files_to_move = if let Some(separator) = options.input_path_separator {
                get_stdin_split(&separator)?
            } else {
                get_stdin_as_lines()?
            };

            let paths_with_errors = move_files(move_to, &files_to_move)?;

            for (path, err) in paths_with_errors {
                info!("Failed to move {:?} {}", path.display(), err);
            }
        }
        Subcommand::SimilarImages {
            hash_alg,
            resize_filter,
            hash_width,
            hash_height,
            threshold,
        } => {
            let images_to_check = if let Some(separator) = options.input_path_separator {
                get_stdin_split(&separator)?
            } else {
                get_stdin_as_lines()?
            };
            let mut similar_images_options = SimilarImagesOptions::default();

            similar_images_options.hash_alg = hash_alg;

            similar_images_options.filter_type = resize_filter;

            similar_images_options.hash_size = (hash_width, hash_height);

            similar_images_options.threshold = threshold;

            let results = find_similar_images(&images_to_check, similar_images_options);

            for (path, err) in results.1 {
                info!("{:?} {}", path.display(), err);
            }

            println!("{}", results.0
                .iter()
                .map(|similar_images| format!("{}, {}", similar_images.0.display(), similar_images.1.display()))
                .collect::<Vec<String>>()
                .join("\n")
            );
        }
        Subcommand::CheckImageFormats => {
            let images_to_check = if let Some(separator) = options.input_path_separator {
                get_stdin_split(&separator)?
            } else {
                get_stdin_as_lines()?
            };

            let results = check_image_formats(&images_to_check);

            for (path, err) in results.1 {
                info!("{:?} {}", path.display(), err);
            }

            println!("{}", results.0
                .iter()
                .map(|wrong_format_image| format!("{}, {}, {}", wrong_format_image.0.display(), wrong_format_image.1, wrong_format_image.2))
                .collect::<Vec<String>>()
                .join("\n")
            );
        }
        Subcommand::Delete {
            safe_delete_files,
        } => {
            let paths_to_delete = if let Some(separator) = options.input_path_separator {
                get_stdin_split(&separator)?
            } else {
                get_stdin_as_lines()?
            };

            if safe_delete_files {
                for path in paths_to_delete {
                    match safe_delete(&path) {
                        Ok(()) => info!("Safe deleted {:?}", path),
                        Err(e) => info!("Error while safe deleting {:?} {}", path, e),
                    }
                }
            } else {
                for path in paths_to_delete {
                    match delete(&path) {
                        Ok(()) => info!("Deleted {:?}", path),
                        Err(e) => info!("Error deleting {:?} {}", path, e),
                    }
                }
            }
        }
    };

    Ok(())
}

/// Sets up the logger backend for `log`
///
/// Sends all logs to a file called `fily.log`
fn setup_logger(log_level: log::LevelFilter) -> Result<(), Box<dyn Error>> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Utc::now().to_rfc3339(),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log_level)
        .chain(fern::log_file("fily.log")?)
        .apply()?;

    Ok(())
}

/// Splits the input from stdin with `separator` and stores the resulting `String`s in a `Vec`
///
/// Fails if input is not valid UTF-8
fn get_stdin_split(separator: &str) -> Result<Vec<String>, io::Error> {
    let mut input = String::new();
    stdin().lock().read_to_string(&mut input)?;

    let input: Vec<String> = input.split(separator).map(ToString::to_string).collect();

    Ok(input)
}

/// Collects stdin into a `Vec`. Each `String` is a line that is either separated by \n or \r\n
///
/// Fails if stdin produces an error
fn get_stdin_as_lines() -> Result<Vec<String>, io::Error> {
    // collect() turns the array of Result<String, io::Error> to
    // Result<Vec<String>, io::Error> because Result implements FromIter
    stdin().lock().lines().collect()
}
