use std::ffi::OsString;
use regex::Regex;
use clap::{crate_name, crate_version, App, AppSettings, Arg, SubCommand};

use fily_lib::{
    find::{Filename, FilePath, Filesize, Modified, Accessed, Created, Ignore, Condition, SearchCriteria},
    similar_images::{HashAlg, FilterType},
};

#[derive(Debug, Clone)]
pub enum Subcommand {
    CheckImageFormats,

    Duplicates {
        use_hash_version: bool,
    },

    Find {
        paths_to_search_in: Vec<OsString>,
        conditions: Vec<Condition<SearchCriteria>>,
        max_num_results: usize,
        max_search_depth: usize,
        min_depth_from_start: usize,
        ignore: Option<Ignore>,
        ignore_hidden_files: bool,
        follow_symlinks: bool,
        output_separator: String,
    },

    Move {
        move_to: OsString,
    },

    Rename {
        template: String,
    },

    SimilarImages {
        hash_alg: HashAlg,
        resize_filter: FilterType,
        hash_width: u32,
        hash_height: u32,
        threshold: u32,
    },
}

#[derive(Debug, Clone)]
pub struct CLIOptions {
    pub subcommand: Subcommand,
    pub log_level: log::LevelFilter,
    pub strict_logging: bool,
    pub input_path_separator: Option<String>,
}

impl CLIOptions {
    /// Creates a new `CLIOptions` from the cli arguments
    // TODO: Not sure about the error type. Maybe I'll have to change that later
    pub fn new() -> Result<Self, &'static str> {
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
                Arg::with_name("strict_logging")
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
                            .help("A filename has to NOT match all of the passed regexes to be considered a match")
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
                    .about("Finds images which extension do not match their actual format. This can produce false positives on files that aren't images, make sure to check the output")
                    .setting(AppSettings::DeriveDisplayOrder)
                    .setting(AppSettings::WaitOnError)
                    .setting(AppSettings::UnifiedHelpMessage)
            )
            .get_matches();

        let subcommand = match app.subcommand() {
            ("find", Some(args)) => {
                let paths_to_search_in: Vec<OsString> = args.values_of_os("paths_to_search_in")
                    .expect("paths_to_search_in didn't exist")
                    .map(|os_str| os_str.to_os_string())
                    .collect();

                let mut conditions = Vec::new();

                if args.is_present("filename_exact") {
                    let filename_exact = args.value_of("filename_exact")
                        .expect("filename_exact didn't exist")
                        .into();

                    conditions.push(Condition::Value(SearchCriteria::Filename(Filename::Exact(filename_exact))));
                } else if args.is_present("filename_contains") {
                    let criterias: Vec<SearchCriteria> = args.values_of("filename_contains")
                        .expect("filename_contains didn't exist")
                        .map(|substr| SearchCriteria::Filename(Filename::Contains(substr.into())))
                        .collect();

                    conditions.push(Condition::build_all_of_condition(criterias));
                }

                if args.is_present("filesize_over") && args.is_present("filesize_under") {
                    let over_this_size = args.value_of("filesize_over")
                        .expect("filesize_over didn't exist")
                        .parse()
                        .expect("filesize_over parse failed");

                    let under_this_size = args.value_of("filesize_under")
                        .expect("filesize_under didn't exist")
                        .parse()
                        .expect("filesize_under parse failed");

                    if over_this_size >= under_this_size {
                        return Err("filesize_over has to be less than filesize_under");
                    }

                    conditions.push(
                        Condition::And(
                            Box::from(Condition::Value(SearchCriteria::Filesize(Filesize::Over(over_this_size)))),
                            Box::from(Condition::Value(SearchCriteria::Filesize(Filesize::Under(under_this_size))))
                        )
                    );
                } else if args.is_present("filesize_exact") {
                    let filesize_exact = args.value_of("filesize_exact")
                        .expect("filesize_exact didn't exist")
                        .parse()
                        .expect("filesize_exact parse failed");

                    conditions.push(Condition::Value(SearchCriteria::Filesize(Filesize::Exact(filesize_exact))));
                } else if args.is_present("filesize_over") {
                    let filesize_over = args.value_of("filesize_over")
                        .expect("filesize_over didn't exist")
                        .parse()
                        .expect("filesize_over parse failed");

                    conditions.push(Condition::Value(SearchCriteria::Filesize(Filesize::Over(filesize_over))));
                } else if args.is_present("filesize_under") {
                    let filesize_under = args.value_of("filesize_under")
                        .expect("filesize_under didn't exist")
                        .parse()
                        .expect("filesize_under parse failed");

                    conditions.push(Condition::Value(SearchCriteria::Filesize(Filesize::Under(filesize_under))));
                }

                if args.is_present("modified_before") && args.is_present("modified_after") {
                    let before_this_time = args.value_of("modified_before")
                        .expect("modified_before didn't exist")
                        .parse()
                        .expect("modified_before parse failed");

                    let after_this_time = args.value_of("modified_after")
                        .expect("modified_after didn't exist")
                        .parse()
                        .expect("modified_after parse failed");

                    if after_this_time >= before_this_time {
                        return Err("modified_after has to be less than modified_before");
                    }

                    conditions.push(
                        Condition::And(
                            Box::from(Condition::Value(SearchCriteria::Modified(Modified::After(after_this_time)))),
                            Box::from(Condition::Value(SearchCriteria::Modified(Modified::Before(before_this_time))))
                        )
                    );
                } else if args.is_present("modified_at") {
                    let modified_at = args.value_of("modified_at")
                        .expect("modified_at didn't exist")
                        .parse()
                        .expect("modified_at parse failed");

                    conditions.push(Condition::Value(SearchCriteria::Modified(Modified::At(modified_at))));
                } else if args.is_present("modified_before") {
                    let modified_before = args.value_of("modified_before")
                        .expect("modified_before didn't exist")
                        .parse()
                        .expect("modified_before parse failed");

                    conditions.push(Condition::Value(SearchCriteria::Modified(Modified::Before(modified_before))));
                } else if args.is_present("modified_after") {
                    let modified_after = args.value_of("modified_after")
                        .expect("modified_after didn't exist")
                        .parse()
                        .expect("modified_after parse failed");

                    conditions.push(Condition::Value(SearchCriteria::Modified(Modified::After(modified_after))));
                }

                if args.is_present("accessed_before") && args.is_present("accessed_after") {
                    let before_this_time = args.value_of("accessed_before")
                        .expect("accessed_before didn't exist")
                        .parse()
                        .expect("accessed_before parse failed");

                    let after_this_time = args.value_of("accessed_after")
                        .expect("accessed_after didn't exist")
                        .parse()
                        .expect("accessed_after parse failed");

                    if after_this_time >= before_this_time {
                        return Err("accessed_after has to be less than accessed_before");
                    }

                    conditions.push(
                        Condition::And(
                            Box::from(Condition::Value(SearchCriteria::Accessed(Accessed::After(after_this_time)))),
                            Box::from(Condition::Value(SearchCriteria::Accessed(Accessed::Before(before_this_time))))
                        )
                    );
                } else if args.is_present("accessed_at") {
                    let accessed_at = args.value_of("accessed_at")
                        .expect("accessed_at didn't exist")
                        .parse()
                        .expect("accessed_at parse failed");

                    conditions.push(Condition::Value(SearchCriteria::Accessed(Accessed::At(accessed_at))));
                } else if args.is_present("accessed_before") {
                    let accessed_before = args.value_of("accessed_before")
                        .expect("accessed_before didn't exist")
                        .parse()
                        .expect("accessed_before parse failed");

                    conditions.push(Condition::Value(SearchCriteria::Accessed(Accessed::Before(accessed_before))));
                } else if args.is_present("accessed_after") {
                    let accessed_after = args.value_of("accessed_after")
                        .expect("accessed_after didn't exist")
                        .parse()
                        .expect("accessed_after parse failed");

                    conditions.push(Condition::Value(SearchCriteria::Accessed(Accessed::After(accessed_after))));
                }

                if args.is_present("created_before") && args.is_present("created_after") {
                    let before_this_time = args.value_of("created_before")
                        .expect("created_before didn't exist")
                        .parse()
                        .expect("created_before parse failed");

                    let after_this_time = args.value_of("created_after")
                        .expect("created_after didn't exist")
                        .parse()
                        .expect("created_after parse failed");

                    if after_this_time >= before_this_time {
                        return Err("created_after has to be less than created_before");
                    }

                    conditions.push(
                        Condition::And(
                            Box::from(Condition::Value(SearchCriteria::Created(Created::After(after_this_time)))),
                            Box::from(Condition::Value(SearchCriteria::Created(Created::Before(before_this_time))))
                        )
                    );
                } else if args.is_present("created_at") {
                    let created_at = args.value_of("created_at")
                        .expect("created_at didn't exist")
                        .parse()
                        .expect("created_at parse failed");

                    conditions.push(Condition::Value(SearchCriteria::Created(Created::At(created_at))));
                } else if args.is_present("created_before") {
                    let created_before = args.value_of("created_before")
                        .expect("created_before didn't exist")
                        .parse()
                        .expect("created_before parse failed");

                    conditions.push(Condition::Value(SearchCriteria::Created(Created::Before(created_before))));
                } else if args.is_present("created_after") {
                    let created_after = args.value_of("created_after")
                        .expect("created_after didn't exist")
                        .parse()
                        .expect("created_after parse failed");

                    conditions.push(Condition::Value(SearchCriteria::Created(Created::After(created_after))));
                }

                let path_contains: Vec<SearchCriteria> = args.values_of("path_contains")
                    .unwrap_or_default()
                    .map(|substring| SearchCriteria::FilePath(FilePath::Contains(substring.into())))
                    .collect();

                if !path_contains.is_empty() {
                    conditions.push(Condition::build_all_of_condition(path_contains));
                }

                let regex_match_criterias: Vec<SearchCriteria> = args.values_of("filename_regex")
                    .unwrap_or_default()
                    .map(|regex_str| Regex::new(regex_str).expect("invalid regex"))
                    .map(|regex| SearchCriteria::FilenameRegex(regex))
                    .collect();

                if !regex_match_criterias.is_empty() {
                    conditions.push(Condition::build_all_of_condition(regex_match_criterias));
                }

                let regex_ignore_criterias: Vec<SearchCriteria> = args.values_of("filename_regex_ignore")
                    .unwrap_or_default()
                    .map(|regex_str| Regex::new(regex_str).expect("invalid regex"))
                    .map(|regex| SearchCriteria::FilenameRegex(regex))
                    .collect();

                if !regex_ignore_criterias.is_empty() {
                    conditions.push(Condition::build_none_of_condition(regex_ignore_criterias));
                }

                let max_num_results = if args.is_present("max_num_results") {
                    args.value_of("max_num_results")
                        .expect("max_num_results didn't exist")
                        .parse()
                        .expect("max_num_results parse failed")
                } else {
                    usize::MAX
                };

                let max_search_depth = if args.is_present("max_search_depth") {
                    args.value_of("max_search_depth")
                        .expect("max_search_depth didn't exist")
                        .parse()
                        .expect("max_search_depth parse failed")
                } else {
                    usize::MAX
                };

                let min_depth_from_start = if args.is_present("min_depth_from_start") {
                    args.value_of("min_depth_from_start")
                        .expect("min_depth_from_start didn't exist")
                        .parse()
                        .expect("min_depth_from_start parse failed")
                } else {
                    0
                };

                let ignore = args.value_of("ignore")
                    .map(|ignore_str| match ignore_str {
                        "files" => Ignore::Files,
                        "folders" => Ignore::Folders,
                        _ => unreachable!("Someone messed with the possible values ignore"),
                    });

                let ignore_hidden_files = args.is_present("ignore_hidden_files");

                let follow_symlinks = args.is_present("follow_symlinks");

                let output_separator = args.value_of("output_separator")
                    .expect("output_separator didn't exist")
                    .to_string();

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
                }
            }
            ("rename", Some(args)) => {
                let template = args.value_of("new_filename_template")
                    .expect("new_filename_template didn't exist")
                    .to_string();

                Subcommand::Rename {
                    template,
                }
            }
            ("duplicates", Some(args)) => {
                let use_hash_version = args.is_present("use_hash_version");

                Subcommand::Duplicates {
                    use_hash_version,
                }
            }
            ("move", Some(args)) => {
                let move_to = args.value_of_os("move_to")
                    .expect("move_to didn't exist")
                    .to_os_string();

                Subcommand::Move {
                    move_to,
                }
            }
            ("similar_images", Some(args)) => {
                let hash_alg = match args.value_of("hash_alg").expect("hash_alg didn't exist") {
                    "mean" => HashAlg::Mean,
                    "gradient" => HashAlg::Gradient,
                    "vertgradient" => HashAlg::VertGradient,
                    "doublegradient" => HashAlg::DoubleGradient,
                    "blockhash" => HashAlg::Blockhash,
                    _ => unreachable!("Someone messed with the possible values hash_alg")
                };

                let resize_filter = match args.value_of("resize_filter").expect("resize_filter didn't exist") {
                    "nearest" => FilterType::Nearest,
                    "triangle" => FilterType::Triangle,
                    "catmullrom" => FilterType::CatmullRom,
                    "gaussian" => FilterType::Gaussian,
                    "lanczos3" => FilterType::Lanczos3,
                    _ => unreachable!("Someone messed with the possible values resize_filter")
                };

                let hash_width = args.value_of("hash_width")
                    .expect("hash_width didn't exist")
                    .parse()
                    .expect("hash_width parse failed");

                let hash_height = args.value_of("hash_height")
                    .expect("hash_height didn't exist")
                    .parse()
                    .expect("hash_height parse failed");

                let threshold = args.value_of("threshold")
                    .expect("threshold didn't exist")
                    .parse()
                    .expect("threshold parse failed");

                Subcommand::SimilarImages {
                    hash_alg,
                    resize_filter,
                    hash_width,
                    hash_height,
                    threshold,
                }
            }
            ("check_image_formats", _) => Subcommand::CheckImageFormats,
            _ => return Err("Unknown Subcommand"),
        };

        let log_level = match app.value_of("log_level").expect("log_level doesn't exist") {
            "off" => log::LevelFilter::Off,
            "error" => log::LevelFilter::Error,
            "warn" => log::LevelFilter::Warn,
            "info" => log::LevelFilter::Info,
            "debug" => log::LevelFilter::Debug,
            "trace" => log::LevelFilter::Trace,
            _ => unreachable!("Someone messed with the possible values log_level"),
        };

        let strict_logging = app.is_present("strict_logging");

        let input_path_separator = app.value_of("input_path_separator")
            .map(|separator_str| separator_str.to_string());

        let options = CLIOptions {
            subcommand,
            log_level,
            strict_logging,
            input_path_separator,
        };

        Ok(options)
    }
}
