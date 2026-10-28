use chrono::Local;
use std::fmt;
use std::io::Write;
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Severity {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}
impl Severity {
    fn as_str(self) -> &'static str {
        match self {
            Severity::Debug => "DEBUG",
            Severity::Info => "INFO",
            Severity::Warn => "WARN",
            Severity::Error => "ERROR",
        }
    }
}
impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

static MIN_LEVEL: AtomicU8 = AtomicU8::new(Severity::Debug as u8);

/// Set the minimum severity that gets logged.
pub fn set_level(level: Severity) {
    MIN_LEVEL.store(level as u8, Ordering::Relaxed);
}

/// Core logging function. Prefer the `debug!`, `info!`, `warn!`, `error!` macros,
/// which fill in the file name and line number automatically.
pub fn log(severity: Severity, file: &str, line: u32, args: fmt::Arguments<'_>) {
    if (severity as u8) < MIN_LEVEL.load(Ordering::Relaxed) {
        return;
    }

    // `file!()` yields a path like "src/main.rs"; keep only the file name.
    let filename = file
        .rsplit(|c| c == '/' || c == '\\')
        .next()
        .unwrap_or(file);

    let timestamp = Local::now().format("%d-%m-%Y %H:%M:%S");

    let msg = format!("{timestamp} <{filename}>({line}) [{severity}]: {args}");

    // Locking the handle keeps each log line intact across threads.
    // Warnings and errors go to stderr; everything else to stdout.
    if severity >= Severity::Warn {
        let _ = writeln!(std::io::stderr().lock(), "{msg}");
    } else {
        let _ = writeln!(std::io::stdout().lock(), "{msg}");
    }
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::logger::log($crate::logger::Severity::Debug, file!(), line!(), format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::logger::log($crate::logger::Severity::Info, file!(), line!(), format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::logger::log($crate::logger::Severity::Warn, file!(), line!(), format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::logger::log($crate::logger::Severity::Error, file!(), line!(), format_args!($($arg)*))
    };
}
