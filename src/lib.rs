#[macro_use]
extern crate log;
extern crate ansi_term;
extern crate env_logger;
extern crate dogstatsd;

use std::fmt;
use std::time::SystemTime;

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

#[inline]
pub fn init() {
    try_init().unwrap();
}

pub fn try_init() -> Result<(), log::SetLoggerError> {
    try_init_custom_env("RUST_LOG")
}

pub fn init_custom_env(environment_variable_name: &str) {
    try_init_custom_env(environment_variable_name).unwrap();
}

pub fn try_init_custom_env(environment_variable_name: &str) -> Result<(), log::SetLoggerError> {
    let mut builder = formatted_builder()?;

    if let Ok(s) = ::std::env::var(environment_variable_name) {
        builder.parse(&s);
    }

    builder.try_init()
}

pub fn formatted_builder() -> Result<Builder, log::SetLoggerError> {
    let mut builder = Builder::new();

    let mut opts = Options::default();
    opts.namespace = "".to_string();
    let dog = Client::new(opts).unwrap();

    let start = SystemTime::now();

    builder.format(move |f, record| {
        use std::io::Write;

        let now = SystemTime::now();
        let d = match now.duration_since(start) {
            Ok(d) => d,
            Err(e) => e.duration(),
        };

        let secs = d.as_secs() % 60;
        let mins = d.as_secs() / 60 % 60;
        let hours = d.as_secs() / 3600;
        let time = format!("{}:{:02}:{:02}.{:03}",
            hours, mins, secs, 0
        );
        let l = ColorLevel(record.level());

        if let Some(module_path) = record.module_path() {

            // our dirty datadog hack, maybe we shouldn't do it here
            let tags = vec![
                format!("level:{}", DogLevel(record.level())),
                format!("module:{}", module_path),
            ];
            dog.event(module_path, &format!("{}", record.args()), tags).unwrap();
            

            writeln!(f, "{} {}",
                    l,
                    Style::new().bold().paint(format!("[{} {}]",
                        time,
                        module_path)));
            writeln!(f, "{} {}", 
                l,
                format!("{}",record.args()).replace("\n", &format!("\n{} ", l)))
        } else {
            writeln!(f, "{} {}",
                    l,
                    Style::new().bold().paint(format!("[{}]",
                        time)));
            writeln!(f, "{} {}", 
                l,
                format!("{}",record.args()).replace("\n", &format!("\n{} ", l)))
        }
    });

    Ok(builder)
}

#[cfg(test)]
mod tests {
    #[test]
    fn log_types() {
        use init;
        init();
        trace!("We are tracing now!");
        debug!("Debugging works fine");
        info!("This is an info level message: {}", 3 + 5);
        warn!("Warnings should be yellowish\nOh and by the way this is multiline");
        error!("Oofie owie this is serious");
    }
}