use std::fs::{File, OpenOptions};
use std::io::{Write, BufWriter};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

pub struct FileLogger {
    writer: Arc<Mutex<BufWriter<File>>>,
}

impl FileLogger {
    pub fn new(log_file_path: PathBuf) -> Result<Self, std::io::Error> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file_path)?;
        
        let writer = Arc::new(Mutex::new(BufWriter::new(file)));
        
        Ok(FileLogger { writer })
    }
    
    pub fn log(&self, level: &str, message: &str) {
        let timestamp = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(duration) => {
                let secs = duration.as_secs();
                let dt = chrono::DateTime::from_timestamp(secs as i64, 0)
                    .unwrap_or_else(|| chrono::Utc::now());
                dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
            }
            Err(_) => "UNKNOWN_TIME".to_string(),
        };
        
        let log_entry = format!("[{}] {}: {}\n", timestamp, level, message);
        
        if let Ok(mut writer) = self.writer.lock() {
            if let Err(e) = writer.write_all(log_entry.as_bytes()) {
                eprintln!("Failed to write to log file: {}", e);
            } else if let Err(e) = writer.flush() {
                eprintln!("Failed to flush log file: {}", e);
            }
        }
    }
    
    pub fn info(&self, message: &str) {
        self.log("INFO", message);
    }
    
    pub fn error(&self, message: &str) {
        self.log("ERROR", message);
    }
    
    pub fn warn(&self, message: &str) {
        self.log("WARN", message);
    }
}

// Global logger instance
static mut GLOBAL_LOGGER: Option<FileLogger> = None;
static INIT: std::sync::Once = std::sync::Once::new();

pub fn init_file_logger(log_file_path: PathBuf) -> Result<(), std::io::Error> {
    INIT.call_once(|| {
        match FileLogger::new(log_file_path) {
            Ok(logger) => {
                unsafe {
                    GLOBAL_LOGGER = Some(logger);
                }
            }
            Err(e) => {
                eprintln!("Failed to initialize file logger: {}", e);
            }
        }
    });
    Ok(())
}

pub fn log_info(message: &str) {
    unsafe {
        if let Some(ref logger) = GLOBAL_LOGGER {
            logger.info(message);
        } else {
            eprintln!("INFO: {}", message);
        }
    }
}

pub fn log_error(message: &str) {
    unsafe {
        if let Some(ref logger) = GLOBAL_LOGGER {
            logger.error(message);
        } else {
            eprintln!("ERROR: {}", message);
        }
    }
}

pub fn log_warn(message: &str) {
    unsafe {
        if let Some(ref logger) = GLOBAL_LOGGER {
            logger.warn(message);
        } else {
            eprintln!("WARN: {}", message);
        }
    }
}
