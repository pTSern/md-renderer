@echo off
setlocal

echo ===================================================
echo     MDViewer - Unregister Windows File Association
echo ===================================================
echo.

reg delete "HKCU\Software\Classes\*\shell\MDViewer" /f >nul 2>&1
reg delete "HKCU\Software\Classes\Applications\mdviewer.exe" /f >nul 2>&1
reg delete "HKCU\Software\Classes\MDViewer.Markdown" /f >nul 2>&1
reg delete "HKCU\Software\Classes\.md" /ve /f >nul 2>&1

echo [SUCCESS] MDViewer associations removed.
echo.
pause
