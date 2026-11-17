use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Mutex, Once, OnceLock};

use crate::log_stamp::{header_line, role_of_log_name, stamped_line, unix_millis_now};

const FALLBACK_LOG_PATH: &str = "niertogether.log";
const UNKNOWN_BUILD_COMMIT: &str = "unknown";

static WRITE_LOCK: Mutex<()> = Mutex::new(());
static HEADER_WRITTEN: Once = Once::new();
static SETTINGS: OnceLock<LogSettings> = OnceLock::new();
static LOG_PATH: OnceLock<String> = OnceLock::new();

#[derive(Debug, Clone, Copy)]
pub struct LogSettings {
    pub build_commit: &'static str,
    pub resolve_path: fn() -> String,
}

pub fn configure(settings: LogSettings) {
    let _ = SETTINGS.set(settings);
}

pub fn log_path() -> &'static str {
    LOG_PATH.get_or_init(|| match SETTINGS.get() {
        Some(settings) => (settings.resolve_path)(),
        None => FALLBACK_LOG_PATH.to_owned(),
    })
}

pub fn stamp(message: &str) -> String {
    stamped_line(unix_millis_now(), message)
}

pub fn append_line(path: &str, line: &str) {
    append_lines(path, &[line]);
}

pub fn log_line(message: &str) {
    let path = log_path();
    let stamped = stamp(message);
    let mut lines: Vec<String> = Vec::with_capacity(2);
    let _guard = WRITE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    HEADER_WRITTEN.call_once(|| {
        lines.push(header_line(
            unix_millis_now(),
            SETTINGS
                .get()
                .map_or(UNKNOWN_BUILD_COMMIT, |settings| settings.build_commit),
            role_of_log_name(path),
        ));
    });
    lines.push(stamped);
    write_lines_locked(path, &lines);
}

fn append_lines(path: &str, lines: &[&str]) {
    let _guard = WRITE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    write_lines_locked(path, lines);
}

fn write_lines_locked<T: AsRef<str>>(path: &str, lines: &[T]) {
    let mut text = String::new();
    for line in lines {
        text.push_str(line.as_ref());
        text.push('\n');
    }
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = file.write_all(text.as_bytes());
    }
}
