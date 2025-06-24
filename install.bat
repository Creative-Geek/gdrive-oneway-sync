@echo off
setlocal

:: --- Configuration ---
SET "ServiceName=GdriveStealthSync"
SET "ServiceDisplayName=Google Drive Stealth Sync Service"
SET "InstallDir=C:\Program Files\%ServiceName%"
SET "ExeName=gdrive-stealth-sync.exe"
SET "ConfigName=config.json"
SET "CredsName=credentials.json"
:: --- End Configuration ---

echo.
echo === Stealth Sync Service Installer ===
echo.

echo [+] Stopping and deleting any existing service...
sc.exe stop %ServiceName% > nul 2>&1
sc.exe delete %ServiceName% > nul 2>&1
timeout /t 2 /nobreak > nul

echo [+] Creating installation directory: %InstallDir%
mkdir "%InstallDir%"

echo [+] Copying application files...
copy "%ExeName%" "%InstallDir%\"
copy "%ConfigName%" "%InstallDir%\"
copy "%CredsName%" "%InstallDir%\"

echo [+] Creating the Windows Service...
sc.exe create %ServiceName% binPath= "%InstallDir%\%ExeName%" start= auto DisplayName= "%ServiceDisplayName%"

IF %ERRORLEVEL% NEQ 0 (
    echo [!!!] FAILED to create the service. Please run this script as an Administrator.
    pause
    exit /b 1
)

echo [+] Setting service to recover on failure...
sc.exe failure %ServiceName% reset= 86400 actions= restart/60000

echo [+] Service created successfully.
echo [+] Starting the service...
sc.exe start %ServiceName%

echo.
echo === Installation Complete ===
echo.
echo You can check the Windows Event Log (under 'Windows Logs' -> 'Application')
echo for messages from '%ServiceName%' to verify it is working.
echo.
pause