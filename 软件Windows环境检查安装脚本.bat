@echo off
setlocal
cd /d "%~dp0"

echo Checking and installing the development environment for this software on Windows...
echo.

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\check-env.ps1" -Install
if errorlevel 1 (
  echo.
  echo Environment installation failed. Please read the error above, then run this file again after fixing it.
  echo Common fix: make sure winget can access the internet, or run winget source update in PowerShell.
  pause
  exit /b 1
)

echo.
echo Finished. If Rust, fnm, or Visual Studio Build Tools were installed, close this window and open a new terminal before running the Tauri app.
pause
