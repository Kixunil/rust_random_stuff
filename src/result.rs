use crate::error::DisplayError;

/// Helper making implementations shorter
pub trait IntoResult: Sized {
    type Value;
    type Error;

    /// No-op, just a hack to make impl shorter
    fn internal_into_result(self) -> Result<Self::Value, Self::Error>;

    /// A version of map_err that doesn't consume error
    fn with_err<F: FnOnce(&Self::Error)>(self, fun: F) -> Result<Self::Value, Self::Error> {
        self.internal_into_result().map_err(|error| { fun(&error); error })
    }
}

/// ResultExt that provides nicer error messages than unwrap/expect
///
/// Exits with exit code 2 to allow grep-like behavior
pub trait UnwrapOrExit: IntoResult {
    /// Another trick to shorten impl
    ///
    /// But you may find it useful too
    fn unwrap_or_exit_custom<F: FnOnce(Self::Error)>(self, printer: F) -> Self::Value {
        self.internal_into_result().unwrap_or_else(|error| {
            printer(error);
            std::process::exit(2);
        })
    }

    /// Formatting using std::error::Error
    ///
    /// Note that Error trait is special, this displays sources separated with `: `
    fn unwrap_or_exit(self) -> Self::Value where Self::Error: 'static + std::error::Error {
        self.unwrap_or_exit_custom(|error| {
            eprintln!("Error: {}", error.join_sources(": "));
        })
    }

    /// Formatting using Display
    fn unwrap_or_exit_display(self) -> Self::Value where Self::Error: std::fmt::Display {
        self.unwrap_or_exit_custom(|error| eprintln!("Error: {}", error))
    }

    /// Formatting using Debug
    fn unwrap_or_exit_debug(self) -> Self::Value where Self::Error: std::fmt::Debug {
        self.unwrap_or_exit_custom(|error| eprintln!("Error: {:?}", error))
    }

    /// Log error and exit
    fn unwrap_or_exit_log<L: LogOwned>(self, mut logger: L) -> Self::Value where Self::Error: 'static + std::error::Error {
        self.unwrap_or_exit_custom(|error| logger.log_error_owned("Error", error))
    }
}

impl<T, E> IntoResult for Result<T, E> {
    type Value = T;
    type Error = E;

    fn internal_into_result(self) -> Self {
        self
    }
}

impl<T, E> UnwrapOrExit for Result<T, E> {}

/// Result extension trait providing easy logging of errors
///
/// Supports `log` and `slog` (consuming only) crates - use appropriate features.
///
/// I'm too lazy to document each method, so read this:
///
/// * `log_${loglevel}()` - logs massage with error on error without changing the result
/// * `log_${loglevel}_and_replace() - log message and consume error, replacing it with parameter
/// * `log_${loglevel}_and_replace_with()` - log message and consume error but construct
///                                          the replacement using a reference to error before
///                                          consuming it.
///
/// If you're not sure what's the usefulness of the last one (or two), here's a real-life use case:
/// A HTTP server/application needs to log errors and also return more general responses (e.g. 404,
/// 500...). So we use `log_error_and_replace_with` to determine what kind of error response is
/// needed for given error. This converts all errors to a common error type, which itself can be
/// converted to HTTP response. (e.g. using `unwrap_or_else`)
///
/// This neatly separates business logic from logging and HTTP response handling.
pub trait LogResult: IntoResult where <Self as IntoResult>::Error: 'static + std::error::Error {
    /// Internal, helps with implementation
    fn convert_and_consume_err<E, ConvF, ConsF>(self, convert: ConvF, consume: ConsF) -> Result<Self::Value, E> where ConvF: FnOnce(&Self::Error) -> E, ConsF: FnOnce(Self::Error) {
        self.internal_into_result().map_err(|error| {
            let converted = convert(&error);
            consume(error);
            converted
        })
    }

    fn log_error<L: Log>(self, mut logger: L, message: &str) -> Result<Self::Value, Self::Error> {
        self.with_err(|error| logger.log_error(message, error))
    }

    fn log_warning<L: Log>(self, mut logger: L, message: &str) -> Result<Self::Value, Self::Error> {
        self.with_err(|error| logger.log_warning(message, error))
    }

    fn log_info<L: Log>(self, mut logger: L, message: &str) -> Result<Self::Value, Self::Error> {
        self.with_err(|error| logger.log_info(message, error))
    }

    fn log_debug<L: Log>(self, mut logger: L, message: &str) -> Result<Self::Value, Self::Error> {
        self.with_err(|error| logger.log_debug(message, error))
    }

    fn log_trace<L: Log>(self, mut logger: L, message: &str) -> Result<Self::Value, Self::Error> {
        self.with_err(|error| logger.log_trace(message, error))
    }

    fn log_error_and_replace<E, L: LogOwned>(self, logger: L, message: &str, replacement: E) -> Result<Self::Value, E> {
        self.log_error_and_replace_with(logger, message, move |_| replacement)
    }

    fn log_warning_and_replace<E, L: LogOwned>(self, logger: L, message: &str, replacement: E) -> Result<Self::Value, E> {
        self.log_warning_and_replace_with(logger, message, move |_| replacement)
    }

    fn log_info_and_replace<E, L: LogOwned>(self, logger: L, message: &str, replacement: E) -> Result<Self::Value, E> {
        self.log_info_and_replace_with(logger, message, move |_| replacement)
    }

    fn log_debug_and_replace<E, L: LogOwned>(self, logger: L, message: &str, replacement: E) -> Result<Self::Value, E> {
        self.log_debug_and_replace_with(logger, message, move |_| replacement)
    }

    fn log_trace_and_replace<E, L: LogOwned>(self, logger: L, message: &str, replacement: E) -> Result<Self::Value, E> {
        self.log_trace_and_replace_with(logger, message, move |_| replacement)
    }

    fn log_error_and_replace_with<E, F, L: LogOwned>(self, mut logger: L, message: &str, convert: F) -> Result<Self::Value, E> where F: FnOnce(&Self::Error) -> E {
        self.convert_and_consume_err(convert, |error| logger.log_error_owned(message, error))
    }

    fn log_warning_and_replace_with<E, F, L: LogOwned>(self, mut logger: L, message: &str, convert: F) -> Result<Self::Value, E> where F: FnOnce(&Self::Error) -> E {
        self.convert_and_consume_err(convert, |error| logger.log_warning_owned(message, error))
    }

    fn log_info_and_replace_with<E, F, L: LogOwned>(self, mut logger: L, message: &str, convert: F) -> Result<Self::Value, E> where F: FnOnce(&Self::Error) -> E {
        self.convert_and_consume_err(convert, |error| logger.log_info_owned(message, error))
    }

    fn log_debug_and_replace_with<E, F, L: LogOwned>(self, mut logger: L, message: &str, convert: F) -> Result<Self::Value, E> where F: FnOnce(&Self::Error) -> E {
        self.convert_and_consume_err(convert, |error| logger.log_debug_owned(message, error))
    }

    fn log_trace_and_replace_with<E, F, L: LogOwned>(self, mut logger: L, message: &str, convert: F) -> Result<Self::Value, E> where F: FnOnce(&Self::Error) -> E {
        self.convert_and_consume_err(convert, |error| logger.log_trace_owned(message, error))
    }
}

impl<T, E: 'static + std::error::Error> LogResult for Result<T, E> {}

/// Abstraction over loggers
///
/// This is for loggers that have to consume errors (e.g. because of sending them to another thread).
/// Loggers that don't have to consume errors must also implement this one anyway.
/// It is recommended to implement `Log` in such case and then use `impl_log_owned!(YourLogger)`.
/// In general, `log_${loglevel}` and `log_${loglevel}_owned` should have the same behavior when
/// observed by a user.
pub trait LogOwned {
    fn log_error_owned<E: 'static + std::error::Error>(&mut self, message: &str, error: E);
    fn log_warning_owned<E: 'static + std::error::Error>(&mut self, message: &str, error: E);
    fn log_info_owned<E: 'static + std::error::Error>(&mut self, message: &str, error: E);
    fn log_debug_owned<E: 'static + std::error::Error>(&mut self, message: &str, error: E);
    fn log_trace_owned<E: 'static + std::error::Error>(&mut self, message: &str, error: E);
}

/// Abstraction over loggers
///
/// This is for loggers that don't have to consume errors.
pub trait Log: LogOwned {
    fn log_error(&mut self, message: &str, error: &(dyn 'static + std::error::Error));
    fn log_warning(&mut self, message: &str, error: &(dyn 'static + std::error::Error));
    fn log_info(&mut self, message: &str, error: &(dyn 'static + std::error::Error));
    fn log_debug(&mut self, message: &str, error: &(dyn 'static + std::error::Error));
    fn log_trace(&mut self, message: &str, error: &(dyn 'static + std::error::Error));
}

impl<T: LogOwned> LogOwned for &mut T {
    fn log_error_owned<E: 'static + std::error::Error>(&mut self, message: &str, error: E) {
        (*self).log_error_owned(message, error);
    }

    fn log_warning_owned<E: 'static + std::error::Error>(&mut self, message: &str, error: E) {
        (*self).log_warning_owned(message, error);
    }

    fn log_info_owned<E: 'static + std::error::Error>(&mut self, message: &str, error: E) {
        (*self).log_info_owned(message, error);
    }

    fn log_debug_owned<E: 'static + std::error::Error>(&mut self, message: &str, error: E) {
        (*self).log_debug_owned(message, error);
    }

    fn log_trace_owned<E: 'static + std::error::Error>(&mut self, message: &str, error: E) {
        (*self).log_trace_owned(message, error);
    }
}

/// Implements LogOwned if you implemented Log manually
#[macro_export]
macro_rules! impl_log_owned {
    ($type:ty) => {
        impl LogOwned for $type {
            fn log_error_owned<E: 'static + std::error::Error>(&mut self, message: &str, error: E) {
                $crate::result::Log::log_error(self, message, &error);
            }

            fn log_warning_owned<E: 'static + std::error::Error>(&mut self, message: &str, error: E) {
                $crate::result::Log::log_warning(self, message, &error);
            }

            fn log_info_owned<E: 'static + std::error::Error>(&mut self, message: &str, error: E) {
                $crate::result::Log::log_info(self, message, &error);
            }

            fn log_debug_owned<E: 'static + std::error::Error>(&mut self, message: &str, error: E) {
                $crate::result::Log::log_debug(self, message, &error);
            }

            fn log_trace_owned<E: 'static + std::error::Error>(&mut self, message: &str, error: E) {
                $crate::result::Log::log_trace(self, message, &error);
            }
        }
    }
}

impl<T: Log> Log for &mut T {
    fn log_error(&mut self, message: &str, error: &(dyn 'static + std::error::Error)) {
        (*self).log_error(message, error);
    }

    fn log_warning(&mut self, message: &str, error: &(dyn 'static + std::error::Error)) {
        (*self).log_warning(message, error);
    }
    fn log_info(&mut self, message: &str, error: &(dyn 'static + std::error::Error)) {
        (*self).log_info(message, error);
    }
    fn log_debug(&mut self, message: &str, error: &(dyn 'static + std::error::Error)) {
        (*self).log_debug(message, error);
    }
    fn log_trace(&mut self, message: &str, error: &(dyn 'static + std::error::Error)) {
        (*self).log_trace(message, error);
    }
}

/// Marker that uses global logger provided by `log` crate to log
#[cfg(feature = "log")]
#[derive(Copy, Clone)]
pub struct GlobalLogger;


/// Generates `{message}: {error}` with sources separated by `: `.
#[cfg(feature = "log")]
impl Log for GlobalLogger {
    fn log_error(&mut self, message: &str, error: &(dyn 'static + std::error::Error)) {
        log::error!("{}: {}", message, error.join_sources(": "));
    }

    fn log_warning(&mut self, message: &str, error: &(dyn 'static + std::error::Error)) {
        log::warn!("{}: {}", message, error.join_sources(": "));
    }
    fn log_info(&mut self, message: &str, error: &(dyn 'static + std::error::Error)) {
        log::info!("{}: {}", message, error.join_sources(": "));
    }
    fn log_debug(&mut self, message: &str, error: &(dyn 'static + std::error::Error)) {
        log::debug!("{}: {}", message, error.join_sources(": "));
    }
    fn log_trace(&mut self, message: &str, error: &(dyn 'static + std::error::Error)) {
        log::trace!("{}: {}", message, error.join_sources(": "));
    }
}

#[cfg(feature = "log")]
impl_log_owned!(GlobalLogger);

/// Uses native Error logging with `errorr` as the key.
#[cfg(feature = "slog")]
impl LogOwned for &slog::Logger {
    fn log_error_owned<E: 'static + std::error::Error>(&mut self, message: &str, error: E) {
        slog::error!(self, "{}", message; "error" => #error);
    }

    fn log_warning_owned<E: 'static + std::error::Error>(&mut self, message: &str, error: E) {
        slog::warn!(self, "{}", message; "error" => #error);
    }
    fn log_info_owned<E: 'static + std::error::Error>(&mut self, message: &str, error: E) {
        slog::info!(self, "{}", message; "error" => #error);
    }
    fn log_debug_owned<E: 'static + std::error::Error>(&mut self, message: &str, error: E) {
        slog::debug!(self, "{}", message; "error" => #error);
    }
    fn log_trace_owned<E: 'static + std::error::Error>(&mut self, message: &str, error: E) {
        slog::trace!(self, "{}", message; "error" => #error);
    }
}

/// Prints nice error message when returned from `main()`
///
/// Errors are ususally formatted using `Debug` when returned from `main()`.
/// This sucks because `Debug` is not intended for users.
/// This alias solves it by using special error type which formats the error using `Error` trait.
/// It contains a `Box` so it trades one allocation for convenience (just write `?` anywhere).
///
/// Using this for anything else is not recommended!
pub type MultilineTerminator = Result<(), crate::error::TerminatingError<crate::error::MultilineTerminator, crate::error::BoxedError>>;
