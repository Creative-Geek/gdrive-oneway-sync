use std::sync::mpsc;
use std::time::Duration;

use windows_service::service_dispatcher;
use windows_service::service::{
    ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus, ServiceType,
};
use windows_service::service_control_handler::{self, ServiceControlHandlerResult};

// Import your sync logic module
mod gdrive_sync;
mod file_logger;

const SERVICE_NAME: &str = "GdriveStealthSync";

fn main() -> Result<(), windows_service::Error> {
    // Register the service with the SCM.
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;
    Ok(())
}

extern "system" fn ffi_service_main(_argc: u32, _argv: *mut *mut u16) {
    // The mpsc channel is used to send a stop signal to the service loop.
    let (shutdown_tx, shutdown_rx) = mpsc::channel();

    // Initialize file-based logger
    let exe_path = match std::env::current_exe() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Failed to get current executable path: {}", e);
            return;
        }
    };
    let base_dir = exe_path.parent().expect("Failed to get parent dir");
    let log_file_path = base_dir.join("gdrive_sync.log");

    // Alternative: Use current working directory instead
    // let log_file_path = std::env::current_dir().unwrap_or_else(|_| base_dir.to_path_buf()).join("gdrive_sync.log");

    eprintln!("Attempting to create log file at: {:?}", log_file_path);
    match file_logger::init_file_logger(log_file_path.clone()) {
        Ok(_) => {
            eprintln!("Successfully initialized file logger: {:?}", log_file_path);
            eprintln!("Log file should be created at: {}", log_file_path.display());
        },
        Err(e) => {
            eprintln!("Failed to initialize file logger: {}", e);
            eprintln!("Attempted location was: {}", log_file_path.display());
        }
    }
    
    // Define the service control handler
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop => {
                file_logger::log_info("Received stop control event. Shutting down.");
                shutdown_tx.send(()).unwrap();
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    // Register the handler and get the status handle
    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler).unwrap();

    // Tell the SCM that the service is starting
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    }).unwrap();

    // Test that logging is working
    file_logger::log_info("GdriveStealthSync service is initializing...");

    // --- YOUR CORE LOGIC GOES HERE ---
    let _service_thread = std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            gdrive_sync::run_sync_loop().await;
        });
    });

    file_logger::log_info("Service started successfully.");

    // Wait for the stop signal
    shutdown_rx.recv().unwrap();

    // Tell the SCM that the service is stopped
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    }).unwrap();
}

// Helper functions for logging that can be used by other modules
pub fn log_info(message: &str) {
    file_logger::log_info(message);
}

pub fn log_error(message: &str) {
    file_logger::log_error(message);
}