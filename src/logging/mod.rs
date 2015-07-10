use std::io::prelude::*;
use std::fs::OpenOptions;
use std::fs;
use std::ops::Deref;
use std::path::Path;
use std::io::{Error, ErrorKind};
use std::sync::{Arc, Mutex};

use log::{self, LogRecord, LogLevel, LogMetadata, LogLevelFilter, SetLoggerError};
use time;

struct StdoutLogger;

impl log::Log for StdoutLogger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= LogLevel::Info
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            println!("{} - {} - {}",
                     record.level(),
                     time::now().to_timespec().sec,
                     record.args());
        }
    }
}

struct FileLogger {
    file: Arc<Mutex<fs::File>>,
}

impl FileLogger {
    pub fn new(path: &str) -> Result<FileLogger, Error> {
        let ospath = Path::new(path).parent();
        if ospath.is_none() {
            return Err(Error::new(
                ErrorKind::Other, format!("Failed to use log directory: {}", path)
            ));
        }

        match fs::create_dir_all(&ospath.unwrap()) {
            Err(e) => return Err(Error::new(
                ErrorKind::Other, format!("Failed to create log directory: {}", e)
            )),
            Ok(_) => (),
        }

        OpenOptions::new().append(true).create(true).open(path).map( |file| {
            FileLogger{
                file: Arc::new(Mutex::new(file))
            }
        })
    }
}

impl log::Log for FileLogger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= LogLevel::Info
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            let mut logfile = self.file.clone();
            // TODO(tyler) this doesn't actually work yet.
            logfile.lock()
                .unwrap()
                .write_all(format!("{} - {} - {}",
                                   record.level(),
                                   time::now().to_timespec().sec,
                                   record.args()).as_bytes());
        }
    }
}

pub fn init_logger(path: Option<String>) -> Result<(), SetLoggerError> {
    match path {
        Some(p) => {
            let logger = FileLogger::new(p.trim_left()).unwrap();
            log::set_logger(|max_log_level| {
                max_log_level.set(LogLevelFilter::Info);
                Box::new(logger)
            })
        },
        None => {
            log::set_logger(|max_log_level| {
                max_log_level.set(LogLevelFilter::Info);
                Box::new(StdoutLogger)
            })
        }
    }
}
