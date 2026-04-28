@echo off
setlocal
cd /d "%~dp0"

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\start-tauri-windows.ps1"
if errorlevel 1 (
  echo.
  echo Tauri launch failed. Please read the error above.
  pause
  exit /b 1
)

endlocal
