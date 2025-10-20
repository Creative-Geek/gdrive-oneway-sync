# Google Drive One-Way Sync

A lightweight Windows service that automatically syncs files from a local folder to Google Drive in real-time. Perfect for automatic backups, continuous uploads, or keeping cloud storage in sync with local work.

## Features

- ✨ **Real-time Sync**: Automatically uploads new files as they appear in the watched folder
- 🔄 **Windows Service**: Runs in the background without user interaction
- 📝 **File-based Logging**: Detailed logs with automatic rotation (2MB per file, max 5 files)
- 🔐 **Service Account Authentication**: Secure Google Drive API integration
- 🎯 **One-Way Upload**: Simple, focused functionality - local to cloud only
- 💾 **Lightweight**: Optimized for minimal resource usage

## Prerequisites

Before you begin, ensure you have:

1. **Windows OS** (Windows 10 or later recommended)
2. **Administrator privileges** (required for Windows Service installation)
3. **Google Cloud Project** with Drive API enabled
4. **Service Account credentials** from Google Cloud Console

## Google Drive Setup

### Step 1: Create a Google Cloud Project

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Click "Select a project" → "New Project"
3. Enter a project name and click "Create"

### Step 2: Enable Google Drive API

1. In your project, go to "APIs & Services" → "Library"
2. Search for "Google Drive API"
3. Click on it and press "Enable"

### Step 3: Create Service Account

1. Go to "APIs & Services" → "Credentials"
2. Click "Create Credentials" → "Service Account"
3. Enter a name (e.g., "gdrive-sync-service")
4. Click "Create and Continue"
5. Skip the optional steps and click "Done"

### Step 4: Generate Service Account Key

1. Click on the service account you just created
2. Go to the "Keys" tab
3. Click "Add Key" → "Create new key"
4. Choose "JSON" format
5. Click "Create" - a JSON file will download
6. **Save this file as `credentials.json` in the same folder as the executable**

### Step 5: Share Google Drive Folder

1. Create or open the Google Drive folder you want to sync to
2. Click "Share"
3. Paste the service account email (found in the JSON file as `client_email`)
4. Give it "Editor" permissions
5. Click "Share"
6. Copy the folder ID from the URL (the part after `/folders/`)

## Installation

### Step 1: Download/Build the Application

**Option A: Download Release**

- Download the latest release from the [Releases page](https://github.com/Creative-Geek/gdrive-oneway-sync/releases)
- Extract the files to a location of your choice

**Option B: Build from Source**

```bash
git clone https://github.com/Creative-Geek/gdrive-oneway-sync.git
cd gdrive-oneway-sync
cargo build --release
```

The executable will be in `target/release/gdrive-stealth-sync.exe`

### Step 2: Configure

1. Copy `config.json.template` to `config.json`
2. Edit `config.json`:

   ```json
   {
     "local_folder_path": "C:\\Users\\YourName\\Documents\\ToSync",
     "gdrive_folder_id": "1a2b3c4d5e6f7g8h9i0j"
   }
   ```

   - `local_folder_path`: Full path to the folder you want to watch
   - `gdrive_folder_id`: The ID from the Google Drive folder URL

3. Place your `credentials.json` (from Google Cloud) in the same directory

### Step 3: Install as Windows Service

1. Open **Command Prompt or PowerShell as Administrator**
2. Navigate to the folder containing the files
3. Run the installation script:
   ```cmd
   install.bat
   ```

The service will be created and started automatically.

## Usage

### Managing the Service

**Start the service:**

```cmd
sc start GdriveStealthSync
```

**Stop the service:**

```cmd
sc stop GdriveStealthSync
```

**Check service status:**

```cmd
sc query GdriveStealthSync
```

**View logs:**
Logs are stored in the `logs/` folder next to the executable:

- Files are named `gdrive_sync_YYYYMMDD_HHMMSS.log`
- Automatically rotated when they reach 2MB
- Maximum of 5 log files kept (oldest deleted automatically)

### Uninstalling

1. Open Command Prompt or PowerShell as Administrator
2. Stop and delete the service:
   ```cmd
   sc stop GdriveStealthSync
   sc delete GdriveStealthSync
   ```
3. Delete the application folder

## Configuration Files

### config.json

```json
{
  "local_folder_path": "C:\\Path\\To\\Local\\Folder",
  "gdrive_folder_id": "YOUR_FOLDER_ID"
}
```

### credentials.json

This is the service account key file downloaded from Google Cloud. Do not share this file or commit it to version control.

## Troubleshooting

### Service won't start

- Check that `config.json` and `credentials.json` exist in the correct location
- Verify the paths in `config.json` use double backslashes (`\\`)
- Check the logs in the `logs/` folder for specific errors

### Files not uploading

- Verify the service account email has been granted access to the Google Drive folder
- Check that the `gdrive_folder_id` is correct
- Ensure the local folder path exists and is accessible
- Review logs for authentication or permission errors

### "Access Denied" errors

- Make sure the service account has "Editor" permissions on the Google Drive folder
- Verify the `credentials.json` file is valid and not corrupted

### Logs not appearing

- Check that the service has write permissions to its installation directory
- The `logs/` folder is created automatically on first run
- Verify there's sufficient disk space

## GUI Configurator

A graphical configurator tool is available in the `gdrive_configurator/` directory to help create the `config.json` file without editing JSON manually. This is especially useful for non-technical users.

To use it:

1. Build the configurator: `cd gdrive_configurator && cargo build --release`
2. Run the executable to generate `config.json` with a user-friendly interface

## How It Works

1. The service monitors the specified local folder for new files
2. When a new file is detected, it waits 5 seconds for the file to finish writing
3. The file is uploaded to the specified Google Drive folder
4. All operations are logged with timestamps
5. The service continues running in the background

**Note**: This is a one-way sync only. Files are uploaded to Google Drive but not downloaded. Changes in Google Drive do not affect local files.

## Security Considerations

- Keep `credentials.json` secure - it provides access to your Google Drive
- The service account should only have access to the specific folder you want to sync
- Use the principle of least privilege - don't grant unnecessary permissions
- Regularly review the service account's access in Google Drive

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

Built with:

- [Rust](https://www.rust-lang.org/)
- [google-drive3](https://crates.io/crates/google-drive3) - Google Drive API client
- [notify](https://crates.io/crates/notify) - File system notifications
- [windows-service](https://crates.io/crates/windows-service) - Windows service integration

## Support

If you encounter issues or have questions:

1. Check the [Troubleshooting](#troubleshooting) section
2. Review the logs in the `logs/` directory
3. Open an issue on [GitHub](https://github.com/Creative-Geek/gdrive-oneway-sync/issues)
