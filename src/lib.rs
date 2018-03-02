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
            hours, mins, secs, d.subsec_nanos() / 1_000_000
        );

        let color = match record.level() {
            Level::Trace => Color::Purple,
            Level::Debug => Color::Blue,
            Level::Info => Color::Green,
            Level::Warn => Color::Yellow,
            Level::Error => Color::Red,
        };

        let l = match record.level() {
            Level::Trace => "TRC",
            Level::Debug => "DBG",
            Level::Info => "LOG",
            Level::Warn => "WRN",
            Level::Error => "ERR",
        };

        if let Some(module_path) = record.module_path() {

            // our dirty datadog hack, maybe we shouldn't do it here
            let tags = vec![
                format!("level:{}", DogLevel(record.level())),
                format!("module:{}", module_path),
            ];
            dog.event(format!("[{} {}] {}", l, time, module_path), format!("{}", record.args()), tags).unwrap();
            
            let header = format!("[{} {} {}]", l, time, module_path);
            writeln!(f, "{} {}", 
                Style::new().fg(color).bold().paint(header.clone()),
                format!("{}",record.args()).replace("\n", &format!("\n{: <width$} ",  " ", width=header.len())))
        } else {
            let header = format!("[{} {}]", l, time);
            writeln!(f, "{} {}", 
                Style::new().fg(color).bold().paint(header.clone()),
                format!("{}",record.args()).replace("\n", &format!("\n{: <width$} ",  " ", width=header.len())))
        }
    });

    Ok(builder)
}

#[cfg(test)]
mod tests {
    #[test]
    fn log_types() {
        use init;
        use std::thread::sleep_ms;
        init();
        trace!("We are tracing now!");
        sleep_ms(20);
        debug!("Debugging works fine");
        sleep_ms(131);
        info!("This is an info level message: {}", 3 + 5);
        sleep_ms(540);
        warn!("Warnings should be yellowish\nOh and by the way this is multiline");
        sleep_ms(543);
        error!("Oofie owie this is serious");
    }
}