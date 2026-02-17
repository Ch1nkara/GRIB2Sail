use chrono::Local;
use colored::*;
use env_logger::Builder;
use log::{Level, LevelFilter};
use std::io::Write;

pub fn init(filter_level: LevelFilter) {
    Builder::new()
        .format(|buf, record| {
            let ts = Local::now().format("%Y-%m-%d %H:%M:%S");
            let level = match record.level() {
                Level::Error => record.level().to_string().red(),
                Level::Warn => record.level().to_string().truecolor(255, 165, 0), // orange
                Level::Info => record.level().to_string().green(),
                Level::Debug => record.level().to_string().yellow(),
                Level::Trace => record.level().to_string().normal(),
            };
            writeln!(buf, "{} [{}] {}", ts, level, record.args())
        })
        .filter_level(filter_level)
        .init();
}
