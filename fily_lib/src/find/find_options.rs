use std::{error::Error, convert::TryInto};
use super::{Condition, SearchCriteria, Ignore};

/// Stores options for `find`
///
/// You can instantiate this struct directly but I recommend
/// using `FindOptionsBuilder`. It provides a couple nice helper functions
/// for creating `Condition`s
///
/// If you want to instantiate this directly you can use `Default::default()` which
/// provides a config that matches anything. So you only have to change the options
/// you care about
#[derive(Debug, Clone)]
pub struct FindOptions {
    /// Contains a vec of `Condition<SearchCriteria>` with which you
    /// you can define what criteria a file should or should not match.
    /// I recommend reading the docs on `Condition` and `SearchCriteria`
    pub options: Vec<Condition<SearchCriteria>>,

    /// Maximum amount of paths returned
    pub max_num_results: usize,

    /// Limits how many subfolders deep it searches
    pub max_search_depth: usize,

    /// Limits how many subfolders deep it should start searching. I recommend
    /// setting this lower than or equals to `max_search_depth`
    pub min_depth_from_start: usize,

    /// Used to either ignore all files or all folders
    pub ignore: Option<Ignore>,

    /// Ignore all files that start with a dot
    pub ignore_hidden_files: bool,

    /// If it should follow symlinks and search in there too. If this is false
    /// it will check the conditions against the symlink itself, not the file it
    /// points to
    pub follow_symlinks: bool,
}

impl Default for FindOptions {
    fn default() -> Self {
        FindOptions {
            options: Vec::new(),
            max_num_results: usize::MAX,
            max_search_depth: usize::MAX,
            min_depth_from_start: 0,
            ignore: None,
            ignore_hidden_files: false,
            follow_symlinks: false,
        }
    }
}

/// Builder for `FindOptions`
#[derive(Debug, Clone, Default)]
pub struct FindOptionsBuilder {
    find_options: FindOptions
}

impl FindOptionsBuilder {
    /// Creates and returns a new `FindOptionsBuilder`. The containing `FindOptions`
    /// is instantiated with its default implementation
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        FindOptionsBuilder::default()
    }

    /// Builds the `FindOptions` and returns it
    #[inline]
    #[must_use]
    pub fn build(self) -> FindOptions {
        self.find_options
    }

    /// Adds a single `condition` to the list of conditions
    #[inline]
    pub fn add_condition(&mut self, condition: Condition<SearchCriteria>) -> &mut Self {
        self.find_options.options.push(condition);
        self
    }

    /// Adds all `Condition`s in `conditions` to the list of conditions
    #[inline]
    pub fn add_conditions(&mut self, mut conditions: Vec<Condition<SearchCriteria>>) -> &mut Self {
        self.find_options.options.append(&mut conditions);
        self
    }

    /// Adds a condition that requires all of the search criterias in `search_criterias` to match
    pub fn add_all_of_condition(&mut self, search_criterias: Vec<SearchCriteria>) -> &mut Self {
        if search_criterias.is_empty() {
            return self;
        }

        self.find_options.options.push(Condition::build_all_of_condition(search_criterias));

        self
    }

    /// Adds a condition that requires any (at least one) of the search criterias in `search_criterias` to match
    pub fn add_any_of_condition(&mut self, search_criterias: Vec<SearchCriteria>) -> &mut Self {
        if search_criterias.is_empty() {
            return self;
        }

        self.find_options.options.push(Condition::build_any_of_condition(search_criterias));

        self
    }

    /// Adds a condition that requires none of the search criterias in `search_criterias` to match
    pub fn add_none_of_condition(&mut self, search_criterias: Vec<SearchCriteria>) -> &mut Self {
        if search_criterias.is_empty() {
            return self;
        }

        self.find_options.options.push(Condition::build_none_of_condition(search_criterias));

        self
    }

    /// Adds a condition from a `&str`. This can fail.
    /// Also this isn't actually implemented right now so it'll just panic if you call this
    #[inline]
    pub fn add_condition_from_str(&mut self, condition_str: &str) -> Result<&mut Self, Box<dyn Error>> {
        self.find_options.options.push(condition_str.try_into()?);

        Ok(self)
    }

    /// Sets the maximum number of paths returned
    ///
    /// Default is unlimited
    #[inline]
    pub fn set_max_num_results(&mut self, max_num_results: usize) -> &mut Self {
        self.find_options.max_num_results = max_num_results;
        self
    }

    /// Sets how many subfolders deep it'll search at max
    ///
    /// Default is unlimited
    #[inline]
    pub fn set_max_search_depth(&mut self, max_search_depth: usize) -> &mut Self {
        self.find_options.max_search_depth = max_search_depth;
        self
    }

    /// Sets how many subfolders deep it should start searching. This should be less or equals
    /// to the amount of subfolders deep it'll go at max or otherwise it'll always return
    /// exactly zero results
    ///
    /// Default is 0
    #[inline]
    pub fn set_min_depth_from_start(&mut self, min_depth_from_start: usize) -> &mut Self {
        self.find_options.min_depth_from_start = min_depth_from_start;
        self
    }

    /// Sets if it should either ignore all files or all folders
    ///
    /// `None` resets it to ignoring nothing
    #[inline]
    pub fn set_ignored_files(&mut self, ignored_files: Option<Ignore>) -> &mut Self {
        self.find_options.ignore = ignored_files;
        self
    }

    /// Sets if it should ignore files or folders that start with a `.`
    #[inline]
    pub fn set_ignore_hidden_files(&mut self, ignore_hidden_files: bool) -> &mut Self {
        self.find_options.ignore_hidden_files = ignore_hidden_files;
        self
    }

    /// Sets if it should follow symlinks. If this is `false` it will check any criteria against the
    /// symlink file and see if it matches, not against the file the symlink points to
    #[inline]
    pub fn set_follow_symlinks(&mut self, follow_symlinks: bool) -> &mut Self {
        self.find_options.follow_symlinks = follow_symlinks;
        self
    }
}
