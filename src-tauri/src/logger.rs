use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

use log::{Level, LevelFilter, Metadata, Record};

struct FileLogger {
    file: Mutex<File>,
    level: LevelFilter,
}

impl log::Log for FileLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let line = format!(
            "{} [{:>5}] {}: {}\n",
            timestamp,
            record.level(),
            record.target(),
            record.args()
        );
        if let Ok(mut file) = self.file.lock() {
            let _ = file.write_all(line.as_bytes());
            let _ = file.flush();
        }
        if record.level() <= Level::Warn {
            eprint!("{line}");
        } else {
            println!("{line}");
        }
    }

    fn flush(&self) {
        if let Ok(mut file) = self.file.lock() {
            let _ = file.flush();
        }
    }
}

/// Initializes the application logger. Writes to `<log_dir>/game-space.log`.
pub fn init(log_dir: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(log_dir)?;
    let path = log_dir.join("game-space.log");
    let file = OpenOptions::new().create(true).append(true).open(&path)?;

    let logger = FileLogger {
        file: Mutex::new(file),
        level: LevelFilter::Info,
    };
    log::set_boxed_logger(Box::new(logger))?;
    log::set_max_level(LevelFilter::Info);
    log::info!("=== Game Space logger started ===");
    Ok(())
}
