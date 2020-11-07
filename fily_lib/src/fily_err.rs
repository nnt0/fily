use std::{error::Error, fmt};

/// A wrapper around any type that implements error with the ability to add a message
/// that explains it
///
/// This can be created with the `new` function, calling `into` on any type that
/// implements `Error` or with the `context` and `with_context` functions on
/// a `Result`.
///
/// Note that creating this with the `From` implementation results in it having no context.
///
/// You can't change the existing context of a `FilyError` but you can always add to
/// it using the `add_to_context` function.
///
/// You can painlessly convert a `Result` containing any error to a `FilyError` with
/// context added by calling the `context` or `with_context` functions on it. They will,
/// if the `Result` is an `Err`, build a `FilyError` with the containing error and your
/// context from it.
///
/// If you call either the `context` or `with_context` functions on a `Result<T, FilyError<E>>`
/// it will not wrap the already existing `FilyError` in another one but rather add
/// your context to it, assuming the `Result` is the `Err` variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilyError<T: Error> {
    err: T,
    context: String,
}

impl<T: Error> Error for FilyError<T> {}

impl<T: Error> fmt::Display for FilyError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n{}", self.context, self.err)
    }
}

impl<T: Error> From<T> for  FilyError<T> {
    fn from(err: T) -> Self {
        FilyError::new(err, "")
    }
}

impl<T: Error> FilyError<T> {
    /// Creates a new `FilyError`
    pub fn new(err: T, context: impl Into<String>) -> Self {
        FilyError {
            err,
            context: context.into(),
        }
    }

    /// Creates a new `FilyError` and computes its context from a function
    pub fn new_with_context<U: Into<String>, F: Fn() -> U>(err: T, context: F) -> Self {
        FilyError {
            err,
            context: context().into(),
        }
    }

    /// Returns a reference to the underlying error
    ///
    /// Note that this may not necessarily be the root cause
    pub fn get_error(&self) -> &T {
        &self.err
    }

    /// Gets a string slice of the full context string
    pub fn get_context(&self) -> &str {
        self.context.as_str()
    }

    /// Adds additional context to the already existing context
    ///
    /// It's usually a good idea to insert a newline at the beginning if you
    /// add something to a context you haven't created or you'll possibly
    /// get error messages that may look a bit ugly
    pub fn add_to_context(mut self, context: impl AsRef<str>) -> Self {
        self.context.push_str(context.as_ref());
        self
    }

    /// Consumes the struct and returns the underlying error and context
    ///
    /// This is inteded to be used when you only want to have the error
    /// or the context string but don't want to reallocate them after getting
    /// a reference to them through `get_error` or `get_context`
    pub fn destructure(self) -> (T, String) {
        (self.err, self.context)
    }
}

pub trait Context<T, E: Error> {
    #[allow(clippy::missing_errors_doc)]
    fn context(self, context: impl Into<String>) -> Result<T, FilyError<E>>;
    #[allow(clippy::missing_errors_doc)]
    fn with_context<U: Into<String>, F: Fn() -> U>(self, context: F) -> Result<T, FilyError<E>>;
}

impl<T, E: Error> Context<T, E> for Result<T, E> {
    /// Converts a `Result<T, E>` to a `Result<T, FilyError<E>>` and adds context
    fn context(self, context: impl Into<String>) -> Result<T, FilyError<E>> {
        self.map_err(|err| FilyError::new(err, context))
    }

    /// Converts a `Result<T, E>` to a `Result<T, FilyError<E>>` and computes its context from a function
    fn with_context<U: Into<String>, F: Fn() -> U>(self, context: F) -> Result<T, FilyError<E>> {
        self.map_err(|err| FilyError::new(err, context()))
    }
}

impl<T, E: Error> Context<T, E> for Result<T, FilyError<E>> {
    /// Adds context to an already existing `FilyError` if the `Result` is an `Err`
    fn context(self, context: impl Into<String>) -> Result<T, FilyError<E>> {
        self.map_err(|err| err.add_to_context(context.into()))
    }

    /// Computes context from a function and adds it to an already existing `FilyError`
    /// if the `Result` is an `Err`
    fn with_context<U: Into<String>, F: Fn() -> U>(self, context: F) -> Result<T, FilyError<E>> {
        self.map_err(|err| err.add_to_context(context().into()))
    }
}
