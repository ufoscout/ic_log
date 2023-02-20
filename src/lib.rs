use std::{cell::RefCell, io};

use log::{LevelFilter, Log, Metadata, Record, SetLoggerError};

pub mod filter;
pub mod fmt;

pub use self::fmt::glob::*;

use self::filter::Filter;
use self::fmt::writer::{self, Writer};
use self::fmt::{FormatFn, Formatter};

/// The env logger.
///
/// This struct implements the `Log` trait from the [`log` crate][log-crate-url],
/// which allows it to act as a logger.
///
/// The [`init()`], [`try_init()`], [`Builder::init()`] and [`Builder::try_init()`]
/// methods will each construct a `Logger` and immediately initialize it as the
/// default global logger.
///
/// If you'd instead need access to the constructed `Logger`, you can use
/// the associated [`Builder`] and install it with the
/// [`log` crate][log-crate-url] directly.
///
/// [log-crate-url]: https://docs.rs/log/
/// [`init()`]: fn.init.html
/// [`try_init()`]: fn.try_init.html
/// [`Builder::init()`]: struct.Builder.html#method.init
/// [`Builder::try_init()`]: struct.Builder.html#method.try_init
/// [`Builder`]: struct.Builder.html
pub struct Logger {
    writer: Writer,
    filter: Filter,
    format: FormatFn,
}

/// `Builder` acts as builder for initializing a `Logger`.
///
/// It can be used to customize the log format, change the environment variable used
/// to provide the logging directives and also set the default log level filter.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate log;
/// # use std::io::Write;
/// use env_logger::Builder;
/// use log::LevelFilter;
///
/// let mut builder = Builder::from_default_env();
///
/// builder
///     .format(|buf, record| writeln!(buf, "{} - {}", record.level(), record.args()))
///     .filter(None, LevelFilter::Info)
///     .init();
///
/// error!("error message");
/// info!("info message");
/// ```
#[derive(Default)]
pub struct Builder {
    filter: filter::Builder,
    writer: writer::Builder,
    format: fmt::Builder,
    built: bool,
}

impl Builder {
    /// Initializes the log builder with defaults.
    ///
    /// **NOTE:** This method won't read from any environment variables.
    /// Use the [`filter`] and [`write_style`] methods to configure the builder
    /// or use [`from_env`] or [`from_default_env`] instead.
    ///
    /// # Examples
    ///
    /// Create a new builder and configure filters and style:
    ///
    /// ```
    /// use log::LevelFilter;
    /// use env_logger::{Builder, WriteStyle};
    ///
    /// let mut builder = Builder::new();
    ///
    /// builder
    ///     .filter(None, LevelFilter::Info)
    ///     .write_style(WriteStyle::Always)
    ///     .init();
    /// ```
    ///
    /// [`filter`]: #method.filter
    /// [`write_style`]: #method.write_style
    pub fn new() -> Builder {
        Default::default()
    }

    /// Sets the format function for formatting the log output.
    ///
    /// This function is called on each record logged and should format the
    /// log record and output it to the given [`Formatter`].
    ///
    /// The format function is expected to output the string directly to the
    /// `Formatter` so that implementations can use the [`std::fmt`] macros
    /// to format and output without intermediate heap allocations. The default
    /// `env_logger` formatter takes advantage of this.
    ///
    /// # Examples
    ///
    /// Use a custom format to write only the log message:
    ///
    /// ```
    /// use std::io::Write;
    /// use env_logger::Builder;
    ///
    /// let mut builder = Builder::new();
    ///
    /// builder.format(|buf, record| writeln!(buf, "{}", record.args()));
    /// ```
    ///
    /// [`Formatter`]: fmt/struct.Formatter.html
    /// [`String`]: https://doc.rust-lang.org/stable/std/string/struct.String.html
    /// [`std::fmt`]: https://doc.rust-lang.org/std/fmt/index.html
    pub fn format<F: 'static>(&mut self, format: F) -> &mut Self
    where
        F: Fn(&mut Formatter, &Record) -> io::Result<()> + Sync + Send,
    {
        self.format.custom_format = Some(Box::new(format));
        self
    }

    /// Use the default format.
    ///
    /// This method will clear any custom format set on the builder.
    pub fn default_format(&mut self) -> &mut Self {
        self.format = Default::default();
        self
    }

    /// Whether or not to write the level in the default format.
    pub fn format_level(&mut self, write: bool) -> &mut Self {
        self.format.format_level = write;
        self
    }

    /// Whether or not to write the module path in the default format.
    pub fn format_module_path(&mut self, write: bool) -> &mut Self {
        self.format.format_module_path = write;
        self
    }

    /// Whether or not to write the target in the default format.
    pub fn format_target(&mut self, write: bool) -> &mut Self {
        self.format.format_target = write;
        self
    }

    /// Configures the amount of spaces to use to indent multiline log records.
    /// A value of `None` disables any kind of indentation.
    pub fn format_indent(&mut self, indent: Option<usize>) -> &mut Self {
        self.format.format_indent = indent;
        self
    }

    /// Configures if timestamp should be included and in what precision.
    pub fn format_timestamp(&mut self, timestamp: Option<fmt::TimestampPrecision>) -> &mut Self {
        self.format.format_timestamp = timestamp;
        self
    }

    /// Configures the timestamp to use second precision.
    pub fn format_timestamp_secs(&mut self) -> &mut Self {
        self.format_timestamp(Some(fmt::TimestampPrecision::Seconds))
    }

    /// Configures the timestamp to use millisecond precision.
    pub fn format_timestamp_millis(&mut self) -> &mut Self {
        self.format_timestamp(Some(fmt::TimestampPrecision::Millis))
    }

    /// Configures the timestamp to use microsecond precision.
    pub fn format_timestamp_micros(&mut self) -> &mut Self {
        self.format_timestamp(Some(fmt::TimestampPrecision::Micros))
    }

    /// Configures the timestamp to use nanosecond precision.
    pub fn format_timestamp_nanos(&mut self) -> &mut Self {
        self.format_timestamp(Some(fmt::TimestampPrecision::Nanos))
    }

    /// Configures the end of line suffix.
    pub fn format_suffix(&mut self, suffix: &'static str) -> &mut Self {
        self.format.format_suffix = suffix;
        self
    }

    /// Adds a directive to the filter for a specific module.
    ///
    /// # Examples
    ///
    /// Only include messages for info and above for logs in `path::to::module`:
    ///
    /// ```
    /// use env_logger::Builder;
    /// use log::LevelFilter;
    ///
    /// let mut builder = Builder::new();
    ///
    /// builder.filter_module("path::to::module", LevelFilter::Info);
    /// ```
    pub fn filter_module(&mut self, module: &str, level: LevelFilter) -> &mut Self {
        self.filter.filter_module(module, level);
        self
    }

    /// Adds a directive to the filter for all modules.
    ///
    /// # Examples
    ///
    /// Only include messages for info and above for logs globally:
    ///
    /// ```
    /// use env_logger::Builder;
    /// use log::LevelFilter;
    ///
    /// let mut builder = Builder::new();
    ///
    /// builder.filter_level(LevelFilter::Info);
    /// ```
    pub fn filter_level(&mut self, level: LevelFilter) -> &mut Self {
        self.filter.filter_level(level);
        self
    }

    /// Adds filters to the logger.
    ///
    /// The given module (if any) will log at most the specified level provided.
    /// If no module is provided then the filter will apply to all log messages.
    ///
    /// # Examples
    ///
    /// Only include messages for info and above for logs in `path::to::module`:
    ///
    /// ```
    /// use env_logger::Builder;
    /// use log::LevelFilter;
    ///
    /// let mut builder = Builder::new();
    ///
    /// builder.filter(Some("path::to::module"), LevelFilter::Info);
    /// ```
    pub fn filter(&mut self, module: Option<&str>, level: LevelFilter) -> &mut Self {
        self.filter.filter(module, level);
        self
    }

    /// Parses the directives string in the same form as the `RUST_LOG`
    /// environment variable.
    ///
    /// See the module documentation for more details.
    pub fn parse_filters(&mut self, filters: &str) -> &mut Self {
        self.filter.parse(filters);
        self
    }

    /// Sets the target for the log output.
    ///
    /// Env logger can log to either stdout, stderr or a custom pipe. The default is stderr.
    ///
    /// The custom pipe can be used to send the log messages to a custom sink (for example a file).
    /// Do note that direct writes to a file can become a bottleneck due to IO operation times.
    ///
    /// # Examples
    ///
    /// Write log message to `stdout`:
    ///
    /// ```
    /// use env_logger::{Builder, Target};
    ///
    /// let mut builder = Builder::new();
    ///
    /// builder.target(Target::Stdout);
    /// ```
    pub fn target(&mut self, target: fmt::Target) -> &mut Self {
        self.writer.target(target);
        self
    }

    /// Sets whether or not styles will be written.
    ///
    /// This can be useful in environments that don't support control characters
    /// for setting colors.
    ///
    /// # Examples
    ///
    /// Never attempt to write styles:
    ///
    /// ```
    /// use env_logger::{Builder, WriteStyle};
    ///
    /// let mut builder = Builder::new();
    ///
    /// builder.write_style(WriteStyle::Never);
    /// ```
    pub fn write_style(&mut self, write_style: fmt::WriteStyle) -> &mut Self {
        self.writer.write_style(write_style);
        self
    }

    /// Parses whether or not to write styles in the same form as the `RUST_LOG_STYLE`
    /// environment variable.
    ///
    /// See the module documentation for more details.
    pub fn parse_write_style(&mut self, write_style: &str) -> &mut Self {
        self.writer.parse_write_style(write_style);
        self
    }

    /// Sets whether or not the logger will be used in unit tests.
    ///
    /// If `is_test` is `true` then the logger will allow the testing framework to
    /// capture log records rather than printing them to the terminal directly.
    pub fn is_test(&mut self, is_test: bool) -> &mut Self {
        self.writer.is_test(is_test);
        self
    }

    /// Initializes the global logger with the built env logger.
    ///
    /// This should be called early in the execution of a Rust program. Any log
    /// events that occur before initialization will be ignored.
    ///
    /// # Errors
    ///
    /// This function will fail if it is called more than once, or if another
    /// library has already initialized a global logger.
    pub fn try_init(&mut self) -> Result<(), SetLoggerError> {
        let logger = self.build();

        let max_level = logger.filter();
        let r = log::set_boxed_logger(Box::new(logger));

        if r.is_ok() {
            log::set_max_level(max_level);
        }

        r
    }

    /// Initializes the global logger with the built env logger.
    ///
    /// This should be called early in the execution of a Rust program. Any log
    /// events that occur before initialization will be ignored.
    ///
    /// # Panics
    ///
    /// This function will panic if it is called more than once, or if another
    /// library has already initialized a global logger.
    pub fn init(&mut self) {
        self.try_init()
            .expect("Builder::init should not be called after logger initialized");
    }

    /// Build an env logger.
    ///
    /// The returned logger implements the `Log` trait and can be installed manually
    /// or nested within another logger.
    pub fn build(&mut self) -> Logger {
        assert!(!self.built, "attempt to re-use consumed builder");
        self.built = true;

        Logger {
            writer: self.writer.build(),
            filter: self.filter.build(),
            format: self.format.build(),
        }
    }
}

impl Logger {

    /// Returns the maximum `LevelFilter` that this env logger instance is
    /// configured to output.
    pub fn filter(&self) -> LevelFilter {
        self.filter.filter()
    }

    /// Checks if this record matches the configured filter.
    pub fn matches(&self, record: &Record) -> bool {
        self.filter.matches(record)
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.filter.enabled(metadata)
    }

    fn log(&self, record: &Record) {
        if self.matches(record) {
            // Log records are written to a thread-local buffer before being printed
            // to the terminal. We clear these buffers afterwards, but they aren't shrunk
            // so will always at least have capacity for the largest log record formatted
            // on that thread.
            //
            // If multiple `Logger`s are used by the same threads then the thread-local
            // formatter might have different color support. If this is the case the
            // formatter and its buffer are discarded and recreated.

            thread_local! {
                static FORMATTER: RefCell<Option<Formatter>> = RefCell::new(None);
            }

            let print = |formatter: &mut Formatter, record: &Record| {
                let _ =
                    (self.format)(formatter, record).and_then(|_| formatter.print(&self.writer));

                // Always clear the buffer afterwards
                formatter.clear();
            };

            let printed = FORMATTER
                .try_with(|tl_buf| {
                    match tl_buf.try_borrow_mut() {
                        // There are no active borrows of the buffer
                        Ok(mut tl_buf) => match *tl_buf {
                            // We have a previously set formatter
                            Some(ref mut formatter) => {
                                // Check the buffer style. If it's different from the logger's
                                // style then drop the buffer and recreate it.
                                if formatter.write_style() != self.writer.write_style() {
                                    *formatter = Formatter::new(&self.writer);
                                }

                                print(formatter, record);
                            }
                            // We don't have a previously set formatter
                            None => {
                                let mut formatter = Formatter::new(&self.writer);
                                print(&mut formatter, record);

                                *tl_buf = Some(formatter);
                            }
                        },
                        // There's already an active borrow of the buffer (due to re-entrancy)
                        Err(_) => {
                            print(&mut Formatter::new(&self.writer), record);
                        }
                    }
                })
                .is_ok();

            if !printed {
                // The thread-local storage was not available (because its
                // destructor has already run). Create a new single-use
                // Formatter on the stack for this call.
                print(&mut Formatter::new(&self.writer), record);
            }
        }
    }

    fn flush(&self) {}
}


mod std_fmt_impls {
    use super::*;
    use std::fmt;

    impl fmt::Debug for Logger {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.debug_struct("Logger")
                .field("filter", &self.filter)
                .finish()
        }
    }

    impl fmt::Debug for Builder {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            if self.built {
                f.debug_struct("Logger").field("built", &true).finish()
            } else {
                f.debug_struct("Logger")
                    .field("filter", &self.filter)
                    .field("writer", &self.writer)
                    .finish()
            }
        }
    }
}

