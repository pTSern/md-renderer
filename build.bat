@echo off
setlocal enabledelayedexpansion

title MDViewer Auto Builder
echo =======================================================
echo               MDViewer - Auto Build Script
echo =======================================================
echo.

:: 1. Check if Cargo is installed
where cargo >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] Cargo/Rust is not found in your system PATH!
    echo Please ensure Rust is installed or run:
    echo   rustup default stable
    echo.
    pause
    exit /b 1
)

:: 2. Check if mdviewer is currently running and close it to prevent file lock
tasklist /fi "imagename eq mdviewer.exe" 2>nul | find /i "mdviewer.exe" >nul
if %errorlevel% equ 0 (
    echo [INFO] Found running mdviewer.exe process. Closing it for build...
    taskkill /im mdviewer.exe /f >nul 2>&1
    timeout /t 1 /nobreak >nul
)

:: 3. Build release binary
echo [BUILD] Compiling optimized release binary...
cargo build --release
if %errorlevel% neq 0 (
    echo.
    echo [ERROR] Build failed! Please check compiler errors above.
    pause
    exit /b %errorlevel%
)

:: 4. Verify output executable
set "TARGET_EXE=%~dp0target\release\mdviewer.exe"
if not exist "!TARGET_EXE!" (
    echo.
    echo [ERROR] Compiled binary not found at:
    echo "!TARGET_EXE!"
    pause
    exit /b 1
)

for %%F in ("!TARGET_EXE!") do set "FILESIZE=%%~zF"
set /a FILESIZE_KB=!FILESIZE! / 1024

echo.
echo =======================================================
echo   [SUCCESS] MDViewer built successfully!
echo   Location: !TARGET_EXE!
echo   Size:     !FILESIZE_KB! KB
echo =======================================================
echo.

:: 5. Quick action menu
echo What would you like to do?
echo [1] Run MDViewer with test document
echo [2] Register Windows .md file association
echo [3] Open release folder in Windows Explorer
echo [4] Exit
echo.

choice /c 1234 /n /m "Select an option [1-4]: "
if errorlevel 4 goto :done
if errorlevel 3 goto :open_folder
if errorlevel 2 goto :register
if errorlevel 1 goto :run_app

:run_app
echo Launching MDViewer with test_document.md...
start "" "!TARGET_EXE!" "%~dp0test_document.md"
goto :done

:register
if exist "%~dp0register_md_association.bat" (
    call "%~dp0register_md_association.bat"
) else (
    echo register_md_association.bat not found.
)
goto :done

:open_folder
explorer.exe "%~dp0target\release"
goto :done

:done
echo.
echo Done!
endlocal
