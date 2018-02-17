#![deny(warnings)]
#![deny(missing_docs)]

//! A logger configured via an environment variable which writes to standard
//! error with nice colored output for log levels.
//!
//! ## Example
//!
//! ```
//! extern crate pretty_env_logger;
//! #[macro_use] extern crate log;
//!
//! fn main() {
//!     pretty_env_logger::init();
//!
//!     trace!("a trace example");
//!     debug!("deboogging");
//!     info!("such information");
//!     warn!("o_O");
//!     error!("boom");
//! }
//! ```

extern crate ansi_term;
extern crate env_logger;
extern crate log;
extern crate dogstatsd;

use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};

use ansi_term::{Color, Style};
use env_logger::Builder;
use log::Level;
use dogstatsd::{Client, Options};

struct ColorLevel(Level);

impl fmt::Display for ColorLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            Level::Trace => Color::Purple.paint("TRC"),
            Level::Debug => Color::Blue.paint("DBG"),
            Level::Info => Color::Green.paint("LOG"),
            Level::Warn => Color::Yellow.paint("WRN"),
            Level::Error => Color::Red.paint("ERR")
        }.fmt(f)
    }
}

struct DogLevel(Level);
impl fmt::Display for DogLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            Level::Trace => "trace",
            Level::Debug => "debug",
            Level::Info => "info",
            Level::Warn => "warning",
            Level::Error => "error",
        }.fmt(f)
    }
}

static MAX_MODULE_WIDTH: AtomicUsize = ATOMIC_USIZE_INIT;

/// Initializes the global logger with a pretty env logger.
///
/// This should be called early in the execution of a Rust program, and the
/// global logger may only be initialized once. Future initialization attempts
/// will return an error.
///
/// # Panics
///
/// This function fails to set the global logger if one has already been set.
#[inline]
pub fn init() {
    try_init().unwrap();
}

/// Initializes the global logger with a pretty env logger.
///
/// This should be called early in the execution of a Rust program, and the
/// global logger may only be initialized once. Future initialization attempts
/// will return an error.
///
/// # Errors
///
/// This function fails to set the global logger if one has already been set.
pub fn try_init() -> Result<(), log::SetLoggerError> {
    try_init_custom_env("RUST_LOG")
}

/// Initialized the global logger with a pretty env logger, with a custom variable name.
///
/// This should be called early in the execution of a Rust program, and the
/// global logger may only be initialized once. Future initialization attempts
/// will return an error.
///
/// # Panics
///
/// This function fails to set the global logger if one has already been set.
pub fn init_custom_env(environment_variable_name: &str) {
    try_init_custom_env(environment_variable_name).unwrap();
}

/// Initialized the global logger with a pretty env logger, with a custom variable name.
///
/// This should be called early in the execution of a Rust program, and the
/// global logger may only be initialized once. Future initialization attempts
/// will return an error.
///
/// # Errors
///
/// This function fails to set the global logger if one has already been set.
pub fn try_init_custom_env(environment_variable_name: &str) -> Result<(), log::SetLoggerError> {
    let mut builder = formatted_builder()?;

    if let Ok(s) = ::std::env::var(environment_variable_name) {
        builder.parse(&s);
    }

    builder.try_init()
}

/// Returns a `env_logger::Builder` for further customization.
///
/// This method will return a colored and formatted) `env_logger::Builder`
/// for further customization. Tefer to env_logger::Build crate documentation
/// for further details and usage.
///
/// This should be called early in the execution of a Rust program, and the
/// global logger may only be initialized once. Future initialization attempts
/// will return an error.
///
/// # Errors
///
/// This function fails to set the global logger if one has already been set.
pub fn formatted_builder() -> Result<Builder, log::SetLoggerError> {
    let mut builder = Builder::new();

    let dog = Client::new(Options::default()).unwrap();

    builder.format(move |f, record| {
        use std::io::Write;
        if let Some(module_path) = record.module_path() {
            let mut max_width = MAX_MODULE_WIDTH.load(Ordering::Relaxed);
            if max_width < module_path.len() {
                MAX_MODULE_WIDTH.store(module_path.len(), Ordering::Relaxed);
                max_width = module_path.len();
            }

            // our dirty datadog hack, maybe we shouldn't do it here
            let tags = vec![
                format!("level:{}", DogLevel(record.level())),
                format!("module:{}", module_path),
            ];
            dog.event(module_path, &format!("{}", record.args()), tags).unwrap();

            writeln!(f, " {} {} > {}",
                     ColorLevel(record.level()),
                     Style::new().bold().paint(format!("{: <width$}", module_path, width=max_width)),
                     record.args())
        } else {
            writeln!(f, " {} > {}",
                     ColorLevel(record.level()),
                     record.args())
        }
    });

    Ok(builder)
}
