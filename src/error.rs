use std::fmt;

/// Helps displaying errors
pub trait DisplayError: std::error::Error + 'static {
    fn join_sources<'a>(&'a self, separator: &'a str) -> JoinErrorSources<'a>;
}

impl<T: std::error::Error + 'static + Sized> DisplayError for T {
    fn join_sources<'a>(&'a self, separator: &'a str) -> JoinErrorSources<'a> {
        JoinErrorSources {
            error: self,
            separator,
        }
    }
}

impl DisplayError for dyn std::error::Error {
    fn join_sources<'a>(&'a self, separator: &'a str) -> JoinErrorSources<'a> {
        JoinErrorSources {
            error: self,
            separator,
        }
    }
}

/// See `DisplayError::join_sources()`
pub struct JoinErrorSources<'a> {
    error: &'a (dyn std::error::Error + 'static),
    separator: &'a str,
}

impl<'a> fmt::Display for JoinErrorSources<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.error, f)?;
        let mut source = self.error.source();
        while let Some(error) = source {
            fmt::Display::fmt(self.separator, f)?;
            fmt::Display::fmt(error, f)?;
            source = error.source();
        }
        Ok(())
    }
}

/// Error type that should be returned from main() to display nice error messages
pub struct TerminatingError<T: TerminationInfo, E: 'static + std::error::Error> {
    _phantom: std::marker::PhantomData<T>,
    error: E,
}

impl<T: TerminationInfo, E: 'static + std::error::Error> fmt::Debug for TerminatingError<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        T::write_prefix(&mut *f)?;
        std::fmt::Display::fmt(&self.error.join_sources(T::error_separator()), f)
    }
}

pub trait TerminationInfo {
    fn write_prefix<W: std::fmt::Write>(writer: W) -> std::fmt::Result;
    fn error_separator() -> &'static str;
}

/// Prints application name and formats error sources one per line
pub enum MultilineTerminator {}

impl TerminationInfo for MultilineTerminator {
    fn write_prefix<W: std::fmt::Write>(mut writer: W) -> std::fmt::Result {
        match std::env::args_os().next().map(std::path::PathBuf::from) {
            Some(path) => write!(writer, "Application {} failed: ", path.display()),
            None => write!(writer, "Application failed: "),
        }
    }

    fn error_separator() -> &'static str {
        "\n\tcaused by: "
    }
}

/// Newtype around Box<dyn std::error::Error> to implement std::error::Error.
#[derive(Debug)]
pub struct BoxedError(Box<dyn 'static + std::error::Error>);

impl BoxedError {
    pub fn new<E: 'static + std::error::Error>(error: E) -> Self {
        BoxedError(Box::new(error))
    }
}

impl fmt::Display for BoxedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&*self.0, f)
    }
}

impl std::error::Error for BoxedError {
    fn source(&self) -> Option<&(dyn 'static + std::error::Error)> {
        self.0.source()
    }
}

impl<T, E> From<E> for TerminatingError<T, BoxedError> where T: TerminationInfo, E: 'static + std::error::Error {
    fn from(value: E) -> Self {
        TerminatingError {
            _phantom: Default::default(),
            error: BoxedError::new(value),
        }
    }
}
