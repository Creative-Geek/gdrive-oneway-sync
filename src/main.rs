use std::sync::mpsc;
use std::time::Duration;
use std::sync::Mutex;
use std::sync::Arc;
use std::fs::OpenOptions;
use std::io::Write;

use windows_service::service_dispatcher;
use windows_service::service::{
    ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus, ServiceType,
};
use windows_service::service_control_handler::{self, ServiceControlHandlerResult};

// Import logging macros
use log::{info, error};
use simplelog::*;

// Import your sync logic module
mod gdrive_sync;

const SERVICE_NAME: &str = "GdriveStealthSync";
const MAX_LOG_SIZE: u64 = 2 * 1024 * 1024; // 2MB
const MAX_LOG_FILES: usize = 5;

// Global logger for custom rotation
static ROTATING_LOGGER: once_cell::sync::Lazy<Arc<Mutex<RotatingFileLogger>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(RotatingFileLogger::new())));

fn main() -> Result<(), windows_service::Error> {
    // Register the service with the SCM.
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;
    Ok(())
}

extern "system" fn ffi_service_main(_argc: u32, _argv: *mut *mut u16) {
    // Initialize file-based logging
    if let Err(e) = initialize_file_logging() {
        eprintln!("Failed to initialize logging: {}", e);
        return;
    }

    // The mpsc channel is used to send a stop signal to the service loop.
    let (shutdown_tx, shutdown_rx) = mpsc::channel();
    
    // Define the service control handler
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop => {
                info!("Received stop control event. Shutting down.");
                if let Err(e) = shutdown_tx.send(()) {
                    error!("Failed to send shutdown signal: {}", e);
                }
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    // Register the handler and get the status handle
    let status_handle = match service_control_handler::register(SERVICE_NAME, event_handler) {
        Ok(handle) => handle,
        Err(e) => {
            error!("Failed to register service control handler: {}", e);
            return;
        }
    };

    // Tell the SCM that the service is starting
    if let Err(e) = status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    }) {
        error!("Failed to set service status to Running: {}", e);
        return;
    }

    // Test that logging is working
    info!("GdriveStealthSync service is initializing...");

    // --- YOUR CORE LOGIC GOES HERE ---
    let _service_thread = std::thread::spawn(move || {
        match tokio::runtime::Runtime::new() {
            Ok(rt) => {
                rt.block_on(async {
                    gdrive_sync::run_sync_loop().await;
                });
            }
            Err(e) => {
                error!("Failed to create tokio runtime: {}", e);
            }
        }
    });

    info!("Service started successfully.");

    // Wait for the stop signal
    if let Err(e) = shutdown_rx.recv() {
        error!("Failed to receive shutdown signal: {}", e);
    }

    // Tell the SCM that the service is stopped
    if let Err(e) = status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    }) {
        error!("Failed to set service status to Stopped: {}", e);
    }
}

struct RotatingFileLogger {
    log_dir: std::path::PathBuf,
    current_file: Option<std::fs::File>,
    current_size: u64,
}

impl RotatingFileLogger {
    fn new() -> Self {
        Self {
            log_dir: std::path::PathBuf::new(),
            current_file: None,
            current_size: 0,
        }
    }

    fn initialize(&mut self, log_dir: std::path::PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        self.log_dir = log_dir;
        std::fs::create_dir_all(&self.log_dir)?;
        self.cleanup_old_log_files();
        self.rotate_if_needed()?;
        Ok(())
    }

    fn write_log(&mut self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        let message_bytes = message.as_bytes();
        
        // Check if we need to rotate
        if self.current_size + message_bytes.len() as u64 > MAX_LOG_SIZE {
            self.rotate_if_needed()?;
        }

        // Write to current file
        if let Some(ref mut file) = self.current_file {
            file.write_all(message_bytes)?;
            file.write_all(b"\n")?;
            file.flush()?;
            self.current_size += message_bytes.len() as u64 + 1;
        }

        Ok(())
    }

    fn rotate_if_needed(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Close current file
        self.current_file = None;
        
        // Clean up old files
        self.cleanup_old_log_files();
        
        // Create new log file
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let log_file_path = self.log_dir.join(format!("gdrive_sync_{}.log", timestamp));
        
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file_path)?;
            
        self.current_file = Some(file);
        self.current_size = 0;
        
        Ok(())
    }

    fn cleanup_old_log_files(&self) {
        let mut log_files = Vec::new();
        
        // Find all log files
        if let Ok(entries) = std::fs::read_dir(&self.log_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("gdrive_sync_") && name.ends_with(".log") {
                        if let Ok(metadata) = entry.metadata() {
                            log_files.push((path, metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH)));
                        }
                    }
                }
            }
        }
        
        // Sort by modification time (oldest first)
        log_files.sort_by_key(|(_, time)| *time);
        
        // Remove excess files
        while log_files.len() >= MAX_LOG_FILES {
            let (path, _) = log_files.remove(0);
            let _ = std::fs::remove_file(&path);
        }
    }
}

impl Write for RotatingFileLogger {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let message = String::from_utf8_lossy(buf);
        if let Err(e) = self.write_log(&message) {
            eprintln!("Failed to write to log: {}", e);
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if let Some(ref mut file) = self.current_file {
            file.flush()?;
        }
        Ok(())
    }
}

fn initialize_file_logging() -> Result<(), Box<dyn std::error::Error>> {
    // Get the directory where the executable is located
    let exe_path = std::env::current_exe()?;
    let log_dir = exe_path.parent().unwrap().join("logs");
    
    // Initialize the rotating logger
    ROTATING_LOGGER.lock().unwrap().initialize(log_dir)?;

    // Create a custom logger that writes through our rotating file logger
    struct CustomLogger;
    
    impl SharedLogger for CustomLogger {
        fn level(&self) -> LevelFilter {
            LevelFilter::Info
        }

        fn config(&self) -> Option<&Config> {
            None
        }

        fn as_log(self: Box<Self>) -> Box<dyn log::Log> {
            Box::new(*self)
        }
    }

    impl log::Log for CustomLogger {
        fn enabled(&self, metadata: &log::Metadata) -> bool {
            metadata.level() <= log::Level::Info
        }

        fn log(&self, record: &log::Record) {
            if self.enabled(record.metadata()) {
                let formatted = format!(
                    "{} [{}] {}:{} - {}",
                    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC"),
                    record.level(),
                    record.file().unwrap_or("unknown"),
                    record.line().unwrap_or(0),
                    record.args()
                );
                
                if let Ok(mut logger) = ROTATING_LOGGER.lock() {
                    let _ = logger.write_log(&formatted);
                }
            }
        }

        fn flush(&self) {
            if let Ok(mut logger) = ROTATING_LOGGER.lock() {
                let _ = logger.flush();
            }
        }
    }

    // Initialize the logger
    CombinedLogger::init(vec![Box::new(CustomLogger)])?;

    Ok(())
}

// Helper functions for logging that can be used by other modules
pub fn log_info(message: &str) {
    info!("{}", message);
}

pub fn log_error(message: &str) {
    error!("{}", message);
}