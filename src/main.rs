#![warn(clippy::cargo, clippy::pedantic)]
#![warn(rust_2018_idioms)]

use std::{error::Error, io::{self, stdin, Read, BufRead}, ffi::OsStr};
use clap::{crate_name, crate_version, App, AppSettings, Arg, SubCommand};
use regex::Regex;
#[allow(unused_imports)]
use log::{trace, debug, info, warn, error};

use fily_lib::operations::{
    rename::rename_files,
    duplicates::{find_duplicate_files, find_duplicate_files_hash},
    find::{find, FindOptionsBuilder, Filename, Filesize, FilePath, Modified, Accessed, Created, Ignore, Condition, SearchCriteria},
    move_files::move_files,
    similar_images::{find_similar_images, SimilarImagesOptions, HashAlg, FilterType},
    check_image_formats::{check_image_formats, CheckImageFormatsError},
};

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
        return Err(Box::from(e));
    }

    Ok(())
}

fn start() -> Result<(), Box<dyn Error>> {
    let app = App::new(crate_name!())
        .about("Does stuff with files")
        .version(crate_version!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::WaitOnError)
        .setting(AppSettings::VersionlessSubcommands)
        .arg(
            Arg::with_name("log_level")
                .value_name("log_level")
                .default_value("off")
                .possible_values(&["off", "trace", "debug", "info", "warn", "error"])
                .env("FILY_LOGLEVEL")
                .short("l")
                .long("log_level")
                .help("Sets the loglevel")
        )
        .arg(
            Arg::with_name("fail_on_no_logging")
                .env("FILY_STRICT_LOGGING")
                .takes_value(false)
                .short("s")
                .long("strict_logging")
                .help("Don't run if logging fails for some reason and panic instead before doing anything. This is a failsafe if you need to make sure that actions are always recorded. This can also be set by creating an environment variable called FILY_STRICT_LOGGING")
        )
        .arg(
            Arg::with_name("input_path_separator")
                .value_name("input_path_separator")
                .env("FILY_INPUT_PATH_SEPARATOR")
                .short("p")
                .long("input_path_separator")
                .help("Changes the expected separator of the paths to the value you pass. Default separator is a new line")
        )
        .subcommand(
            SubCommand::with_name("find")
                .about("Finds files and folders")
                .setting(AppSettings::ArgRequiredElseHelp)
                .setting(AppSettings::DeriveDisplayOrder)
                .setting(AppSettings::WaitOnError)
                .setting(AppSettings::UnifiedHelpMessage)
                .arg(
                    Arg::with_name("paths_to_search_in")
                        .value_name("paths_to_search_in")
                        .required(true)
                        .multiple(true)
                        .short("p")
                        .long("paths_to_search_in")
                        .help("Search starts at this/these path(s). If the path points to a file instead of a folder it will find that file and only that file for that path")
                )
                .arg(
                    Arg::with_name("filename_exact")
                        .value_name("filename_exact")
                        .conflicts_with_all(&["filename_contains", "filename_regex", "filename_regex_ignore"])
                        .short("e")
                        .long("filename_exact")
                        .help("A file/folder name has to match this exactly to be considered a match")
                )
                .arg(
                    Arg::with_name("filename_contains")
                        .value_name("filename_contains")
                        .multiple(true)
                        .short("c")
                        .long("filename_contains")
                        .help("A filename has to contain all of the passed strings to be considered a match")
                )
                .arg(
                    Arg::with_name("path_contains")
                        .value_name("path_contains")
                        .multiple(true)
                        .short("n")
                        .long("path_contains")
                        .help("The path to a file has to contain all of the passed strings to be considered a match")
                )
                .arg(
                    Arg::with_name("path_exact")
                        .value_name("path_exact")
                        .multiple(true)
                        .short("t")
                        .long("path_exact")
                        .help("A file's path has to match this path exactly to be considered a match")
                )
                .arg(
                    Arg::with_name("filename_regex")
                        .value_name("filename_regex")
                        .multiple(true)
                        .short("x")
                        .long("filename_regex")
                        .help("A filename has to match all of the passed regexes to be considered a match")
                )
                .arg(
                    Arg::with_name("filename_regex_ignore")
                        .value_name("filename_regex_ignore")
                        .multiple(true)
                        .short("g")
                        .long("filename_regex_ignore")
                        .help("A filename has to NOT match the passed regexes to be considered a match")
                )
                .arg(
                    Arg::with_name("filesize_exact")
                        .value_name("filesize_exact")
                        .conflicts_with_all(&["filesize_over", "filesize_under"])  
                        .validator(|input| {
                            input.parse::<u64>().map_err(|_| "filesize_exact has to be a valid positive number".to_string())?;
                            Ok(())
                        })
                        .short("s")
                        .long("filesize_exact")
                        .help("A file has to have exactly the number of bytes that were passed")
                )
                .arg(
                    Arg::with_name("filesize_over")
                        .value_name("filesize_over")
                        .validator(|input| {
                            input.parse::<u64>().map_err(|_| "filesize_over has to be a valid positive number".to_string())?;
                            Ok(())
                        })
                        .short("o")
                        .long("filesize_over")
                        .help("A file has to have more bytes than the amount that was passed")
                )
                .arg(
                    Arg::with_name("filesize_under")
                        .value_name("filesize_under")
                        .validator(|input| {
                            input.parse::<u64>().map_err(|_| "filesize_under has to be a valid positive number".to_string())?;
                            Ok(())
                        })
                        .short("u")
                        .long("filesize_under")
                        .help("A file has to have less bytes than the amount that was passed")
                )
                .arg(
                    Arg::with_name("modified_at")
                        .value_name("modified_at")
                        .conflicts_with_all(&["modified_before", "modified_after"])
                        .validator(|input| {
                            input.parse::<i64>().map_err(|_| "modified_at has to be a valid number".to_string())?;
                            Ok(())
                        })
                        // I'm running out of characters and don't want to use random ones that have nothing
                        // to do with the name of this option. Not sure what to do
                        // .short("")
                        .long("modified_at")
                        .help("The time the file was last modified at. Value should be in seconds relative to the unix epoch")
                )
                .arg(
                    Arg::with_name("modified_before")
                        .value_name("modified_before")
                        .validator(|input| {
                            input.parse::<i64>().map_err(|_| "modified_before has to be a valid number".to_string())?;
                            Ok(())
                        })
                        // .short("")
                        .long("modified_before")
                        .help("The file has to be last modified before this time. Value should be in seconds relative to the unix epoch")
                )
                .arg(
                    Arg::with_name("modified_after")
                        .value_name("modified_after")
                        .validator(|input| {
                            input.parse::<i64>().map_err(|_| "modified_after has to be a valid number".to_string())?;
                            Ok(())
                        })
                        // .short("")
                        .long("modified_after")
                        .help("The file has to be last modified after this time. Value should be in seconds relative to the unix epoch")
                )
                .arg(
                    Arg::with_name("accessed_at")
                        .value_name("accessed_at")
                        .conflicts_with_all(&["accessed_before", "accessed_after"])
                        .validator(|input| {
                            input.parse::<i64>().map_err(|_| "accessed_at has to be a valid number".to_string())?;
                            Ok(())
                        })
                        // .short("")
                        .long("accessed_at")
                        .help("The time the file was last accessed at. Value should be in seconds relative to the unix epoch")
                )
                .arg(
                    Arg::with_name("accessed_before")
                        .value_name("accessed_before")
                        .validator(|input| {
                            input.parse::<i64>().map_err(|_| "accessed_before has to be a valid number".to_string())?;
                            Ok(())
                        })
                        // .short("")
                        .long("accessed_before")
                        .help("The file has to be last accessed before this time. Value should be in seconds relative to the unix epoch")
                )
                .arg(
                    Arg::with_name("accessed_after")
                        .value_name("accessed_after")
                        .validator(|input| {
                            input.parse::<i64>().map_err(|_| "accessed_after has to be a valid number".to_string())?;
                            Ok(())
                        })
                        // .short("")
                        .long("accessed_after")
                        .help("The file has to be last accessed after this time. Value should be in seconds relative to the unix epoch")
                )
                .arg(
                    Arg::with_name("created_at")
                        .value_name("created_at")
                        .conflicts_with_all(&["created_before", "created_after"])
                        .validator(|input| {
                            input.parse::<i64>().map_err(|_| "created_at has to be a valid number".to_string())?;
                            Ok(())
                        })
                        // .short("")
                        .long("created_at")
                        .help("The time the file was created at. Value should be in seconds relative to the unix epoch. Note: Not all Unix platforms have this field available which results in an error")
                )
                .arg(
                    Arg::with_name("created_before")
                        .value_name("created_before")
                        .validator(|input| {
                            input.parse::<i64>().map_err(|_| "created_before has to be a valid number".to_string())?;
                            Ok(())
                        })
                        // .short("")
                        .long("created_before")
                        .help("The file has to be created before this time. Value should be in seconds relative to the unix epoch. Note: Not all Unix platforms have this field available which results in an error")
                )
                .arg(
                    Arg::with_name("created_after")
                        .value_name("created_after")
                        .validator(|input| {
                            input.parse::<i64>().map_err(|_| "created_after has to be a valid number".to_string())?;
                            Ok(())
                        })
                        // .short("")
                        .long("created_after")
                        .help("The file has to be created after this time. Value should be in seconds relative to the unix epoch. Note: Not all Unix platforms have this field available which results in an error")
                )
                .arg(
                    Arg::with_name("max_num_results")
                        .value_name("max_num_results")
                        .validator(|input| {
                            input.parse::<usize>().map_err(|_| "max_num_results has to be a valid positive number".to_string())?;
                            Ok(())
                        })
                        .short("r")
                        .long("max_num_results")
                        .help("Limits the amount of files returned. Default is unlimited")
                )
                .arg(
                    Arg::with_name("max_search_depth")
                        .value_name("max_search_depth")
                        .validator(|input| {
                            input.parse::<usize>().map_err(|_| "max_search_depth has to be a valid positive number".to_string())?;
                            Ok(())
                        })
                        .short("d")
                        .long("max_search_depth")
                        .help("Limits how many subfolders deep the search goes. Default is unlimited")
                )
                .arg(
                    Arg::with_name("min_depth_from_start")
                        .value_name("min_depth_from_start")
                        .default_value("0")
                        .hide_default_value(true)
                        .validator(|input| {
                            input.parse::<usize>().map_err(|_| "min_depth_from_start has to be a valid positive number".to_string())?;
                            Ok(())
                        })
                        .short("m")
                        .long("min_depth_from_start")
                        .help("All folders that are (starting from the start_path) less than this number deep are ignored. Default is 0 (nothing is ignored)")
                )
                .arg(
                    Arg::with_name("ignore")
                        .value_name("ignore")
                        .possible_values(&["files", "folders"])
                        .short("i")
                        .long("ignore")
                        .help("Ignores either all files or folders")
                )
                .arg(
                    Arg::with_name("ignore_hidden_files")
                        .short("h")
                        .long("ignore_hidden_files")
                        .help("If this flag is set all files that start with a '.' (a dot) will be ignored")
                )
                .arg(
                    Arg::with_name("follow_symlinks")
                        .short("f")
                        .long("follow_symlinks")
                        .help("If this flag is set any symlinks will be followed")
                )
                .arg(
                    Arg::with_name("output_separator")
                        .value_name("output_separator")
                        .default_value("\n")
                        .hide_default_value(true)
                        .env("FILY_OUTPUT_SEPARATOR")
                        .short("a")
                        .long("output_separator")
                        .help("Sets what is used to separate the paths to files that were found. Defaults to \\n")
                )
        )
        .subcommand(
            SubCommand::with_name("rename")
                .about("Renames files and folders. New name is produced with a template you provide")
                .setting(AppSettings::ArgRequiredElseHelp)
                .setting(AppSettings::DeriveDisplayOrder)
                .setting(AppSettings::WaitOnError)
                .setting(AppSettings::UnifiedHelpMessage)
                .arg(
                    Arg::with_name("new_filename_template")
                        .required(true)
                        .value_name("new_filename_template")
                        .short("t")
                        .long("template")
                        .help("Template which will be used to rename the files")
                )
        )
        .subcommand(
            SubCommand::with_name("duplicates")
                .about("Finds duplicate files and prints the paths to them in pairs")
                .setting(AppSettings::DeriveDisplayOrder)
                .setting(AppSettings::WaitOnError)
                .setting(AppSettings::UnifiedHelpMessage)
                .arg(
                    Arg::with_name("use_hash_version")
                        .short("h")
                        .long("use_hash_version")
                        .help("Hashes the contents to a crc32 and compares the hashes instead of comparing the bytes directly. This can reduce the required amount of RAM significantly")
                )
        )
        .subcommand(
            SubCommand::with_name("move")
                .about("Moves files and folders")
                .setting(AppSettings::ArgRequiredElseHelp)
                .setting(AppSettings::DeriveDisplayOrder)
                .setting(AppSettings::WaitOnError)
                .setting(AppSettings::UnifiedHelpMessage)
                .arg(
                    Arg::with_name("move_to")
                        .value_name("move_to")
                        .required(true)
                        .short("t")
                        .long("move_to")
                        .help("A path to which the files get moved to. Has to point to a folder")
                )
        )
        .subcommand(
            SubCommand::with_name("similar_images")
                .about("Finds similar images")
                .setting(AppSettings::ArgRequiredElseHelp)
                .setting(AppSettings::DeriveDisplayOrder)
                .setting(AppSettings::WaitOnError)
                .setting(AppSettings::UnifiedHelpMessage)
                .arg(
                    Arg::with_name("hash_alg")
                        .value_name("hash_alg")
                        .default_value("gradient")
                        .possible_values(&["mean", "gradient", "vertgradient", "doublegradient", "blockhash"])
                        .short("a")
                        .long("hash_alg")
                        .help("Sets the hashing algorithm")
                )
                .arg(
                    Arg::with_name("resize_filter")
                        .value_name("resize_filter")
                        .default_value("lanczos3")
                        .possible_values(&["nearest", "triangle", "catmullrom", "gaussian", "lanczos3"])
                        .short("r")
                        .long("resize_filter")
                        .help("Sets the filter used to resize images during hashing. This has no effect if the blockhash algorithm is being used")
                )
                .arg(
                    Arg::with_name("hash_width")
                        .value_name("hash_width")
                        .validator(|input| {
                            input.parse::<u32>().map_err(|_| "hash_width has to be a valid positive number".to_string())?;
                            Ok(())
                        })
                        .default_value("8")
                        .requires("hash_height")
                        .short("w")
                        .long("hash_width")
                        .help("Sets the hash width. Higher numbers create diminishing returns")
                )
                .arg(
                    Arg::with_name("hash_height")
                        .value_name("hash_height")
                        .validator(|input| {
                            input.parse::<u32>().map_err(|_| "hash_height has to be a valid positive number".to_string())?;
                            Ok(())
                        })
                        .default_value("8")
                        .requires("hash_width")
                        .short("h")
                        .long("hash_height")
                        .help("Sets the hash height. Higher numbers create diminishing returns")
                )
                .arg(
                    Arg::with_name("threshold")
                        .required(true)
                        .value_name("threshold")
                        .validator(|input| {
                            input.parse::<u32>().map_err(|_| "threshold has to be a valid positive number".to_string())?;
                            Ok(())
                        })
                        .short("t")
                        .long("threshold")
                        .help("Sets how close the images have to be to another")
                )
        )
        .subcommand(
            SubCommand::with_name("check_image_formats")
                .about("Finds images which extension do not match their actual format. This can produce false positives on files that aren't images, be sure to check the output")
                .setting(AppSettings::DeriveDisplayOrder)
                .setting(AppSettings::WaitOnError)
                .setting(AppSettings::UnifiedHelpMessage)
        ).get_matches();

    if let Err(e) = setup_logger(app.value_of("log_level").unwrap()) {
        eprintln!("Error while setting up logging {}", e);

        if app.is_present("fail_on_no_logging") {
            return Err(Box::from("Failed to set up logging"));
        }
    }

    match app.subcommand() {
        ("find", Some(args)) => {
            let paths_to_search_in: Vec<&OsStr> = args.values_of_os("paths_to_search_in").unwrap().collect();

            let mut find_options_builder = FindOptionsBuilder::new();

            if args.is_present("filename_exact") {
                find_options_builder.add_condition(
                    Condition::Value(SearchCriteria::Filename(Filename::Exact(args.value_of("filename_exact").unwrap().into())))
                );
            } else if args.is_present("filename_contains") {
                let criteria: Vec<SearchCriteria> =
                    args.values_of("filename_contains").unwrap().map(|substring| SearchCriteria::Filename(Filename::Contains(substring.into()))).collect();

                find_options_builder.add_all_of_condition(criteria);
            }

            if args.is_present("filesize_over") && args.is_present("filesize_under") {
                let over_this_size = args.value_of("filesize_over").unwrap().parse().unwrap();
                let under_this_size = args.value_of("filesize_under").unwrap().parse().unwrap();

                if over_this_size >= under_this_size {
                    return Err(Box::from("filesize_over has to be less than filesize_under"));
                }

                find_options_builder.add_condition(
                    Condition::And(
                        Box::from(Condition::Value(SearchCriteria::Filesize(Filesize::Over(over_this_size)))),
                        Box::from(Condition::Value(SearchCriteria::Filesize(Filesize::Under(under_this_size))))
                    )
                );
            } else if args.is_present("filesize_exact") {
                find_options_builder.add_condition(
                    Condition::Value(SearchCriteria::Filesize(Filesize::Exact(args.value_of("filesize_exact").unwrap().parse().unwrap())))
                );
            } else if args.is_present("filesize_over") {
                find_options_builder.add_condition(
                    Condition::Value(SearchCriteria::Filesize(Filesize::Over(args.value_of("filesize_over").unwrap().parse().unwrap())))
                );
            } else if args.is_present("filesize_under") {
                find_options_builder.add_condition(
                    Condition::Value(SearchCriteria::Filesize(Filesize::Under(args.value_of("filesize_under").unwrap().parse().unwrap())))
                );
            }

            if args.is_present("modified_before") && args.is_present("modified_after") {
                let before_this_time = args.value_of("modified_before").unwrap().parse().unwrap();
                let after_this_time = args.value_of("modified_after").unwrap().parse().unwrap();

                if after_this_time >= before_this_time {
                    return Err(Box::from("modified_after has to be less than modified_before"));
                }

                find_options_builder.add_condition(
                    Condition::And(
                        Box::from(Condition::Value(SearchCriteria::Modified(Modified::After(after_this_time)))),
                        Box::from(Condition::Value(SearchCriteria::Modified(Modified::Before(before_this_time))))
                    )
                );
            } else if args.is_present("modified_at") {
                find_options_builder.add_condition(
                    Condition::Value(SearchCriteria::Modified(Modified::At(args.value_of("modified_at").unwrap().parse().unwrap())))
                );
            } else if args.is_present("modified_before") {
                find_options_builder.add_condition(
                    Condition::Value(SearchCriteria::Modified(Modified::Before(args.value_of("modified_before").unwrap().parse().unwrap())))
                );
            } else if args.is_present("modified_after") {
                find_options_builder.add_condition(
                    Condition::Value(SearchCriteria::Modified(Modified::After(args.value_of("modified_after").unwrap().parse().unwrap())))
                );
            }

            if args.is_present("accessed_before") && args.is_present("accessed_after") {
                let before_this_time = args.value_of("accessed_before").unwrap().parse().unwrap();
                let after_this_time = args.value_of("accessed_after").unwrap().parse().unwrap();

                if after_this_time >= before_this_time {
                    return Err(Box::from("accessed_after has to be less than accessed_before"));
                }

                find_options_builder.add_condition(
                    Condition::And(
                        Box::from(Condition::Value(SearchCriteria::Accessed(Accessed::After(after_this_time)))),
                        Box::from(Condition::Value(SearchCriteria::Accessed(Accessed::Before(before_this_time))))
                    )
                );
            } else if args.is_present("accessed_at") {
                find_options_builder.add_condition(
                    Condition::Value(SearchCriteria::Accessed(Accessed::At(args.value_of("accessed_at").unwrap().parse().unwrap())))
                );
            } else if args.is_present("accessed_before") {
                find_options_builder.add_condition(
                    Condition::Value(SearchCriteria::Accessed(Accessed::Before(args.value_of("accessed_before").unwrap().parse().unwrap())))
                );
            } else if args.is_present("accessed_after") {
                find_options_builder.add_condition(
                    Condition::Value(SearchCriteria::Accessed(Accessed::After(args.value_of("accessed_after").unwrap().parse().unwrap())))
                );
            }

            if args.is_present("created_before") && args.is_present("created_after") {
                let before_this_time = args.value_of("created_before").unwrap().parse().unwrap();
                let after_this_time = args.value_of("created_after").unwrap().parse().unwrap();

                if after_this_time >= before_this_time {
                    return Err(Box::from("created_after has to be less than created_before"));
                }

                find_options_builder.add_condition(
                    Condition::And(
                        Box::from(Condition::Value(SearchCriteria::Created(Created::After(after_this_time)))),
                        Box::from(Condition::Value(SearchCriteria::Created(Created::Before(before_this_time))))
                    )
                );
            } else if args.is_present("created_at") {
                find_options_builder.add_condition(
                    Condition::Value(SearchCriteria::Created(Created::At(args.value_of("created_at").unwrap().parse().unwrap())))
                );
            } else if args.is_present("created_before") {
                find_options_builder.add_condition(
                    Condition::Value(SearchCriteria::Created(Created::Before(args.value_of("created_before").unwrap().parse().unwrap())))
                );
            } else if args.is_present("created_after") {
                find_options_builder.add_condition(
                    Condition::Value(SearchCriteria::Created(Created::After(args.value_of("created_after").unwrap().parse().unwrap())))
                );
            }

            let path_exact: Vec<SearchCriteria> =
                args.values_of("path_exact").unwrap_or_default().map(|path| SearchCriteria::FilePath(FilePath::Exact(path.into()))).collect();
            if !path_exact.is_empty() {
                find_options_builder.add_all_of_condition(path_exact);
            }

            let path_contains: Vec<SearchCriteria> =
                args.values_of("path_contains").unwrap_or_default().map(|substring| SearchCriteria::FilePath(FilePath::Contains(substring.into()))).collect();
            if !path_contains.is_empty() {
                find_options_builder.add_all_of_condition(path_contains);
            }

            let regex_match: Vec<Regex> =
                args.values_of("filename_regex").unwrap_or_default().map(|regex_str| Regex::new(regex_str).expect("invalid regex")).collect();
            if !regex_match.is_empty() {
                let regex_match_criteria = regex_match.into_iter().map(|regex| SearchCriteria::FilenameRegex(regex)).collect();

                find_options_builder.add_all_of_condition(regex_match_criteria);
            }

            let regex_ignore: Vec<Regex> =
                args.values_of("filename_regex_ignore").unwrap_or_default().map(|regex_str| Regex::new(regex_str).expect("invalid regex")).collect();
            if !regex_ignore.is_empty() {
                let regex_ignore_criteria = regex_ignore.into_iter().map(|regex| SearchCriteria::FilenameRegex(regex)).collect();

                find_options_builder.add_nothing_of_condition(regex_ignore_criteria);
            }

            if args.is_present("max_num_results") {
                find_options_builder.set_max_num_results(args.value_of("max_num_results").unwrap().parse().unwrap());
            }

            if args.is_present("max_search_depth") {
                find_options_builder.set_max_search_depth(args.value_of("max_search_depth").unwrap().parse().unwrap());
            }

            find_options_builder.set_min_depth_from_start(args.value_of("min_depth_from_start").unwrap().parse().unwrap());

            if args.is_present("ignore") {
                find_options_builder.set_ignored_files(
                    Some(match args.value_of("ignore").unwrap() {
                        "files" => Ignore::Files,
                        "folders" => Ignore::Folders,
                        _ => unreachable!("Someone messed with the possible values"),
                    })
                );
            }

            find_options_builder.set_ignore_hidden_files(args.is_present("ignore_hidden_files"));

            find_options_builder.set_follow_symlinks(args.is_present("follow_symlinks"));

            let find_options = find_options_builder.build();
            let output_separator = args.value_of("output_separator").unwrap();

            let results = find(&paths_to_search_in, &find_options);

            for (path, err) in results.1 {
                info!("{:?} {}", path.display(), err);
            }

            println!("{}", results.0
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<String>>()
                .join(output_separator)
            );
        }
        ("rename", Some(args)) => {
            let files_to_rename = if app.is_present("input_path_separator") {
                get_stdin_split(app.value_of("input_path_separator").unwrap())?
            } else {
                get_stdin_as_lines()?
            };
            let new_filename_template = args.value_of("new_filename_template").unwrap();

            rename_files(&files_to_rename, new_filename_template)?;
        }
        ("duplicates", Some(args)) => {
            let files_to_check = if app.is_present("input_path_separator") {
                get_stdin_split(app.value_of("input_path_separator").unwrap())?
            } else {
                get_stdin_as_lines()?
            };
            let use_hash_version = args.is_present("use_hash_version");

            let result;

            if use_hash_version {
                result = find_duplicate_files_hash(&files_to_check);
            } else {
                result = find_duplicate_files(&files_to_check);
            }

            println!("{}", result.0
                .iter()
                .map(|duplicates| format!("{}, {}", duplicates.0.display(), duplicates.1.display()))
                .collect::<Vec<String>>()
                .join("\n")
            );

            for (path, err) in result.1 {
                let (err, err_msg) = err.destructure();

                info!("{:?} {} {}", path.display(), err, err_msg);
            }
        }
        ("move", Some(args)) => {
            let files_to_move = if app.is_present("input_path_separator") {
                get_stdin_split(app.value_of("input_path_separator").unwrap())?
            } else {
                get_stdin_as_lines()?
            };
            let move_to = args.value_of("move_to").unwrap();

            let paths_with_errors = move_files(move_to, &files_to_move)?;

            for (path, err) in paths_with_errors {
                info!("Failed to move {:?} {}", path.display(), err);
            }
        }
        ("similar_images", Some(args)) => {
            let images_to_check = if app.is_present("input_path_separator") {
                get_stdin_split(app.value_of("input_path_separator").unwrap())?
            } else {
                get_stdin_as_lines()?
            };
            let mut similar_images_options = SimilarImagesOptions::default();

            similar_images_options.hash_alg = match args.value_of("hash_alg").unwrap() {
                "mean" => HashAlg::Mean,
                "gradient" => HashAlg::Gradient,
                "vertgradient" => HashAlg::VertGradient,
                "doublegradient" => HashAlg::DoubleGradient,
                "blockhash" => HashAlg::Blockhash,
                _ => unreachable!("Someone messed with the possible values")
            };

            similar_images_options.filter_type = match args.value_of("resize_filter").unwrap() {
                "nearest" => FilterType::Nearest,
                "triangle" => FilterType::Triangle,
                "catmullrom" => FilterType::CatmullRom,
                "gaussian" => FilterType::Gaussian,
                "lanczos3" => FilterType::Lanczos3,
                _ => unreachable!("Someone messed with the possible values")
            };

            if args.is_present("hash_width") {
                let hash_width = args.value_of("hash_width").unwrap().parse().unwrap();
                let hash_height = args.value_of("hash_height").unwrap().parse().unwrap();

                similar_images_options.hash_size = (hash_width, hash_height);
            }

            similar_images_options.threshold = args.value_of("threshold").unwrap().parse().unwrap();

            println!("{}", find_similar_images(&images_to_check, similar_images_options)
                .iter()
                .map(|similar_images| format!("{}, {}", similar_images.0.display(), similar_images.1.display()))
                .collect::<Vec<String>>()
                .join("\n")
            );
        }
        ("check_image_formats", _) => {
            let images_to_check = if app.is_present("input_path_separator") {
                get_stdin_split(app.value_of("input_path_separator").unwrap())?
            } else {
                get_stdin_as_lines()?
            };

            let results = check_image_formats(&images_to_check);

            for (path, err) in results.1 {
                let err_msg = match err {
                    CheckImageFormatsError::ContentGuessError(fily_err) => fily_err.destructure().1,
                    CheckImageFormatsError::UnknownPathExtension => "The paths extension is not known".to_string(),
                    CheckImageFormatsError::NoPathExtension => "The path has no extension".to_string(),
                };

                info!("{:?} {}", path.display(), err_msg);
            }

            println!("{}", results.0
                .iter()
                .map(|wrong_format_image| format!("{}, {}, {}", wrong_format_image.0.display(), wrong_format_image.1, wrong_format_image.2))
                .collect::<Vec<String>>()
                .join("\n")
            );
        }
        _ => eprintln!("Unknown subcommand"),
    };

    Ok(())
}

/// Sets up the logger backend for `log`
///
/// Sends all logs to a file called `fily.log`
///
/// Possible log levels:
/// * off
/// * trace
/// * debug
/// * info
/// * warn
/// * error
fn setup_logger(log_level: &str) -> Result<(), Box<dyn Error>> {
    let log_level = match log_level {
        "off" => log::LevelFilter::Off,
        "error" => log::LevelFilter::Error,
        "warn" => log::LevelFilter::Warn,
        "info" => log::LevelFilter::Info,
        "debug" => log::LevelFilter::Debug,
        "trace" => log::LevelFilter::Trace,
        _ => return Err(Box::from("Unknown loglevel")),
    };

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
