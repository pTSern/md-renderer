@echo off
setlocal enabledelayedexpansion

echo ===============================================================================
echo  MDViewer Release Packager
echo ===============================================================================
echo.

:: 1. Check Cargo
where cargo >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo [ERROR] 'cargo' is not found in PATH. Please install Rust and try again.
    pause
    exit /b 1
)

:: 2. Stop running instances of mdviewer to avoid file lock
echo [*] Checking for running instances of mdviewer.exe...
powershell -NoProfile -Command "Get-Process mdviewer -ErrorAction SilentlyContinue | Stop-Process -Force" >nul 2>&1

:: 3. Build optimized release binary
echo [*] Compiling release binary (cargo build --release)...
cargo build --release
if %ERRORLEVEL% neq 0 (
    echo.
    echo [ERROR] Release build failed! Please check compiler errors above.
    pause
    exit /b 1
)

if not exist "target\release\mdviewer.exe" (
    echo [ERROR] Expected executable target\release\mdviewer.exe not found!
    pause
    exit /b 1
)

:: 4. Prepare clean dist directory
echo [*] Preparing distribution directory (dist\mdviewer)...
if exist "dist" rmdir /s /q "dist"
mkdir "dist\mdviewer"

:: 5. Copy release assets
echo [*] Copying release files...
copy /y "target\release\mdviewer.exe" "dist\mdviewer\mdviewer.exe" >nul
copy /y "keybindings.json" "dist\mdviewer\keybindings.json" >nul
copy /y "logo.png" "dist\mdviewer\logo.png" >nul

if exist "RELEASE_README.txt" (
    copy /y "RELEASE_README.txt" "dist\mdviewer\README.txt" >nul
)
if exist "test_document.md" (
    copy /y "test_document.md" "dist\mdviewer\sample.md" >nul
)

:: 6. Create Zip Archive
echo [*] Compressing into mdviewer-windows-x64.zip...
if exist "mdviewer-windows-x64.zip" del /f /q "mdviewer-windows-x64.zip"
powershell -NoProfile -Command "Compress-Archive -Path 'dist\mdviewer' -DestinationPath 'mdviewer-windows-x64.zip' -Force"

if not exist "mdviewer-windows-x64.zip" (
    echo [ERROR] Failed to create mdviewer-windows-x64.zip!
    pause
    exit /b 1
)

:: 7. Success Summary
echo.
echo ===============================================================================
echo  SUCCESS! Package created: mdviewer-windows-x64.zip
echo ===============================================================================
powershell -NoProfile -Command "$z = Get-Item 'mdviewer-windows-x64.zip'; Write-Host (' Archive Size : ' + [math]::Round($z.Length / 1MB, 2) + ' MB (' + $z.Length + ' bytes)') -ForegroundColor Green; Write-Host (' Output Path  : ' + $z.FullName) -ForegroundColor Cyan"
echo.
echo Ready for distribution! Unzip anywhere and double-click mdviewer.exe.
echo ===============================================================================
echo.
pause
