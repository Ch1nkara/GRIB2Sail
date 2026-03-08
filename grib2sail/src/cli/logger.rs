use grib2sail as g2s;

use chrono::Local;
use colored::*;
use indicatif::ProgressBar;
use log::{Level, LevelFilter, Metadata, Record, set_logger, set_max_level};
use once_cell::sync::OnceCell;
use std::io::{Write, stderr, stdout};
use std::sync::Mutex;

struct GribLogger {
    level_filter: LevelFilter,
}

static PROGRESS_BAR: OnceCell<Mutex<Option<ProgressBar>>> = OnceCell::new();
static LOGGER: OnceCell<GribLogger> = OnceCell::new();

impl log::Log for GribLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level_filter
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let ts = Local::now().format("%Y-%m-%d %H:%M:%S");
        let level = match record.level() {
            Level::Error => record.level().to_string().red(),
            Level::Warn => record.level().to_string().yellow(),
            Level::Info => record.level().to_string().green(),
            Level::Debug => record.level().to_string().magenta(),
            Level::Trace => record.level().to_string().normal(),
        };
        let msg = format!("{} [{}] {}", ts, level, record.args());

        match record.level() {
            Level::Error => write_err(msg),
            _ => {
                if let Some(mutex) = PROGRESS_BAR.get() {
                    match mutex.lock() {
                        Ok(guard) => match &*guard {
                            Some(pb) => pb.println(msg),
                            None => write_std(msg),
                        },
                        Err(_) => write_std(msg),
                    }
                } else {
                    write_std(msg);
                }
            }
        }
    }

    fn flush(&self) {}
}

fn write_err(msg: String) {
    let _ = writeln!(&mut stderr(), "{}", msg);
}

fn write_std(msg: String) {
    let _ = writeln!(&mut stdout(), "{}", msg);
}

pub fn set_progress_bar(len: usize) -> Result<(), g2s::GribError> {
    let pb = ProgressBar::new(len as u64);
    let mutex = PROGRESS_BAR.get_or_init(|| Mutex::new(None));
    let mut guard = mutex.lock().map_err(|e| {
        g2s::GribError::Generic(format!("Mutex poisoned: {}", e))
    })?;
    *guard = Some(pb);
    Ok(())
}

pub fn increment_progress_bar(inc: u64) -> Result<(), g2s::GribError> {
    let mutex = PROGRESS_BAR.get().ok_or_else(|| {
        g2s::GribError::Generic("progress bar not initialized".to_string())
    })?;
    let guard = mutex.lock().map_err(|e| {
        g2s::GribError::Generic(format!("Mutex poisoned: {}", e))
    })?;
    let pb = guard.as_ref().ok_or_else(|| {
        g2s::GribError::Generic("progress bar inside mutex is None".to_string())
    })?;
    pb.inc(inc);
    Ok(())
}

pub fn clear_progress_bar() -> Result<(), g2s::GribError> {
    let mutex = PROGRESS_BAR.get().ok_or_else(|| {
        g2s::GribError::Generic("progress bar not initialized".to_string())
    })?;
    let mut guard = mutex.lock().map_err(|e| {
        g2s::GribError::Generic(format!("Mutex poisoned: {}", e))
    })?;
    *guard = None;
    Ok(())
}

pub fn init(level_filter: LevelFilter) -> Result<(), g2s::GribError> {
    let logger = GribLogger { level_filter };
    LOGGER
        .set(logger)
        .map_err(|_| g2s::GribError::Generic("setting failed".to_string()))?;
    let logger_ref = LOGGER
        .get()
        .ok_or_else(|| g2s::GribError::Generic("getting failed".to_string()))?;
    set_logger(logger_ref)?;
    set_max_level(level_filter);
    Ok(())
}
