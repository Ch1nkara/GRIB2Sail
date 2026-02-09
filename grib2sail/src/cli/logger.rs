use env_logger::Builder;
use chrono::Local;
use std::io::Write;
use colored::*;
use log::{LevelFilter,Level};


pub fn init(level: i32) {
    let filter_level = match level {
        1 => LevelFilter::Error,
        2 => LevelFilter::Warn,
        3 => LevelFilter::Info,
        4 => LevelFilter::Debug,
        _ => LevelFilter::Off,
    };
    Builder::new()
        .format(|buf, record| {
            let ts = Local::now().format("%Y-%m-%d %H:%M:%S");
            let level = match record.level() {
                Level::Error => record.level().to_string().red(),
                Level::Warn => record.level().to_string().truecolor(255,165,0), // orange
                Level::Info => record.level().to_string().green(),
                Level::Debug => record.level().to_string().yellow(),
                Level::Trace => record.level().to_string().normal(),
            };
            writeln!(
                buf,
                "{} [{}] {}",
                ts,
                level,
                record.args()
            )
        })
        .filter_level(filter_level)
        .init();
}

