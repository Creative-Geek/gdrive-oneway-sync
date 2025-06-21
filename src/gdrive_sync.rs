use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use notify_debouncer_full::{new_debouncer, notify::*};
use google_drive3::{api, DriveHub, hyper_util::client::legacy::connect::HttpConnector, yup_oauth2::ServiceAccountKey};
use hyper_rustls::HttpsConnector;

// Import the logging functions from main.rs
use crate::{log_info, log_error}; 

#[derive(Deserialize)]
struct Config {
    local_folder_path: String,
    gdrive_folder_id: String,
}

pub async fn run_sync_loop() {
    let exe_path = match std::env::current_exe() {
        Ok(path) => path,
        Err(e) => {
            log_error(&format!("Failed to get current executable path: {}", e));
            return;
        }
    };
    let base_dir = exe_path.parent().expect("Failed to get parent dir");

    let config_path = base_dir.join("config.json");
    let config: Config = match fs::read_to_string(&config_path) {
        Ok(s) => match serde_json::from_str(&s) {
            Ok(cfg) => cfg,
            Err(e) => {
                log_error(&format!("Failed to parse config.json: {}", e));
                return;
            }
        },
        Err(e) => {
            log_error(&format!("Failed to read config.json: {}", e));
            return;
        }
    };

    let creds_path = base_dir.join("credentials.json");
    let secret_json = match fs::read_to_string(&creds_path) {
        Ok(content) => content,
        Err(e) => {
            log_error(&format!("Failed to read credentials.json: {}", e));
            return;
        }
    };

    let secret: ServiceAccountKey = match serde_json::from_str(&secret_json) {
        Ok(key) => key,
        Err(e) => {
            log_error(&format!("Failed to parse service account key: {}", e));
            return;
        }
    };

    let auth = match google_drive3::yup_oauth2::ServiceAccountAuthenticator::builder(secret)
        .build()
        .await
    {
        Ok(auth) => auth,
        Err(e) => {
            log_error(&format!("Failed to create authenticator: {}", e));
            return;
        }
    };

    // Create HTTP client using the newer API structure
    let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
        .build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .unwrap()
                .https_or_http()
                .enable_http1()
                .build(),
        );

    let hub = DriveHub::new(client, auth);

    log_info(&format!("Google Drive connection established successfully"));
    log_info(&format!("Initial sync starting for folder '{}'", &config.local_folder_path));
    // TODO: Implement an initial sync for existing files.

    log_info(&format!("Now watching for new files in: {}", &config.local_folder_path));
    log_info(&format!("Target Google Drive folder ID: {}", &config.gdrive_folder_id));

    let (tx, rx) = std::sync::mpsc::channel();
    let mut debouncer = new_debouncer(Duration::from_secs(5), None, tx).unwrap();

    debouncer.watcher()
        .watch(Path::new(&config.local_folder_path), RecursiveMode::NonRecursive)
        .unwrap();

    for res in rx {
        match res {
            Ok(events) => {
                for event in events {
                    if let EventKind::Create(_) = event.kind {
                        for path in &event.paths {
                            log_info(&format!("New file detected: {:?}", &path));
                            tokio::time::sleep(Duration::from_secs(2)).await; // Wait for write to finish
                            upload_file(&hub, path, &config.gdrive_folder_id).await;
                        }
                    }
                }
            },
            Err(e) => log_error(&format!("File watch error: {:?}", e)),
        }
    }
}

async fn upload_file(hub: &DriveHub<HttpsConnector<HttpConnector>>, file_path: &PathBuf, parent_folder_id: &str) {
    if !file_path.is_file() {
        return;
    }
    let file_name = file_path.file_name().unwrap().to_str().unwrap();

    let mut remote_file = api::File::default();
    remote_file.name = Some(file_name.to_string());
    remote_file.parents = Some(vec![parent_folder_id.to_string()]);

    let file_content = match fs::File::open(file_path) {
        Ok(f) => f,
        Err(e) => {
            log_error(&format!("Failed to open file {:?}: {}", file_path, e));
            return;
        }
    };

    log_info(&format!("Uploading '{}' to Google Drive", file_name));

    let result = hub
        .files()
        .create(remote_file)
        .upload(file_content, "application/octet-stream".parse().unwrap())
        .await;

    match result {
        Ok((_, file)) => log_info(&format!("Successfully uploaded '{}' with ID: {}", file_name, file.id.unwrap_or_default())),
        Err(e) => log_error(&format!("Failed to upload '{}'. Error: {}", file_name, e)),
    }
}