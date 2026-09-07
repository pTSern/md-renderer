@echo off
setlocal enabledelayedexpansion

echo ===================================================
echo     MDViewer - Windows File Association Setup
echo ===================================================
echo.

set "EXE_PATH=%~dp0target\release\mdviewer.exe"

if not exist "!EXE_PATH!" (
    echo [ERROR] mdviewer.exe not found at:
    echo "!EXE_PATH!"
    echo Please build the project first with: cargo build --release
    pause
    exit /b 1
)

echo Registering MDViewer for current user...
echo Target: "!EXE_PATH!"
echo.

:: 1. Add "Open with MDViewer" to right-click context menu for all files
reg add "HKCU\Software\Classes\*\shell\MDViewer" /ve /d "Open with MDViewer" /f >nul
reg add "HKCU\Software\Classes\*\shell\MDViewer\command" /ve /d "\"!EXE_PATH!\" \"%%1\"" /f >nul

:: 2. Register MDViewer application
reg add "HKCU\Software\Classes\Applications\mdviewer.exe\shell\open\command" /ve /d "\"!EXE_PATH!\" \"%%1\"" /f >nul
reg add "HKCU\Software\Classes\Applications\mdviewer.exe\SupportedTypes" /v ".md" /t REG_SZ /d "" /f >nul
reg add "HKCU\Software\Classes\Applications\mdviewer.exe\SupportedTypes" /v ".markdown" /t REG_SZ /d "" /f >nul

:: 3. Set as default / open handler for .md files
reg add "HKCU\Software\Classes\.md" /ve /d "MDViewer.Markdown" /f >nul
reg add "HKCU\Software\Classes\MDViewer.Markdown" /ve /d "Markdown Document" /f >nul
reg add "HKCU\Software\Classes\MDViewer.Markdown\shell\open\command" /ve /d "\"!EXE_PATH!\" \"%%1\"" /f >nul

echo [SUCCESS] MDViewer successfully registered!
echo - Right-click any file and select "Open with MDViewer"
echo - Double-clicking .md files will open MDViewer directly
echo.
pause
