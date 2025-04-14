use std::io::Write;
use std::sync::atomic::Ordering;

use super::SUPPRESS_MESSAGES;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

#[allow(dead_code)]
pub(crate) enum LogLevel {
    Info,
    Warning,
    Error,
    Debug,
}

pub(crate) fn log(level: LogLevel, message: impl Into<String>) {
    if SUPPRESS_MESSAGES.load(Ordering::Relaxed) {
        return;
    }
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let mut color_spec = ColorSpec::new();
    match level {
        LogLevel::Info => color_spec.set_fg(Some(Color::Green)),
        LogLevel::Warning => color_spec.set_fg(Some(Color::Yellow)),
        LogLevel::Error => color_spec.set_fg(Some(Color::Red)),
        LogLevel::Debug => color_spec.set_fg(Some(Color::Cyan)),
    };

    stdout.set_color(&color_spec).expect("Unable to set color");
    let cargo_name = env!("CARGO_PKG_NAME");
    write!(&mut stdout, "{} *{}* ", cargo_name, match level {
        LogLevel::Info => "INFO",
        LogLevel::Warning => "WARNING",
        LogLevel::Error => "ERROR",
        LogLevel::Debug => "DEBUG",
    })
    .expect("Could not write log message");
    stdout.reset().expect("Could not reset color");
    writeln!(&mut stdout, "{}", message.into()).expect("Unable to write to stdout");
    stdout.flush().expect("Unable to flush stdout");
}
