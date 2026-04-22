@echo off
setlocal

rem 2026-04-22 CST: Added a single root launcher so customer delivery checks
rem can start from one obvious file while the implementation stays inside src.

set "ROOT_DIR=%~dp0"
set "PS_SCRIPT=%ROOT_DIR%src\delivery_check.ps1"
set "REPORTS_DIR=%ROOT_DIR%src\reports"
set "VERSION_ERROR_FILE=%REPORTS_DIR%\powershell_version_error.txt"

if not exist "%PS_SCRIPT%" (
  echo Missing src\delivery_check.ps1
  pause
  exit /b 1
)

where powershell >nul 2>nul
if errorlevel 1 (
  if not exist "%REPORTS_DIR%" mkdir "%REPORTS_DIR%"
  >"%VERSION_ERROR_FILE%" (
    echo PowerShell was not found on this machine.
    echo The delivery check requires Windows PowerShell 5.1 or newer.
  )
  echo.
  echo This machine does not have PowerShell available.
  echo Please send the file "src\reports\powershell_version_error.txt" back to us.
  pause
  exit /b 90
)

powershell -NoProfile -Command "$v=$Host.Version; if (($v.Major -lt 5) -or (($v.Major -eq 5) -and ($v.Minor -lt 1))) { exit 51 }"
set "PS_VERSION_EXIT=%ERRORLEVEL%"
if "%PS_VERSION_EXIT%"=="51" (
  if not exist "%REPORTS_DIR%" mkdir "%REPORTS_DIR%"
  >"%VERSION_ERROR_FILE%" (
    echo Current PowerShell version is too old.
    echo Required version: Windows PowerShell 5.1 or newer.
  )
  >>"%VERSION_ERROR_FILE%" powershell -NoProfile -Command "$Host.Version.ToString()"
  echo.
  echo This machine has an older PowerShell version.
  echo Windows PowerShell 5.1 or newer is required.
  echo Please send the file "src\reports\powershell_version_error.txt" back to us.
  pause
  exit /b 51
)

if not "%PS_VERSION_EXIT%"=="0" (
  if not exist "%REPORTS_DIR%" mkdir "%REPORTS_DIR%"
  >"%VERSION_ERROR_FILE%" (
    echo PowerShell version check failed.
    echo Please send this file back to us.
  )
  echo.
  echo PowerShell version check failed.
  echo Please send the file "src\reports\powershell_version_error.txt" back to us.
  pause
  exit /b %PS_VERSION_EXIT%
)

powershell -ExecutionPolicy Bypass -File "%PS_SCRIPT%"
set "EXIT_CODE=%ERRORLEVEL%"

if not "%EXIT_CODE%"=="0" (
  echo.
  echo Delivery check failed with exit code %EXIT_CODE%.
  echo Please contact us and send the newest folder under src\reports back.
  pause
)

exit /b %EXIT_CODE%
