@echo off
setlocal
cd /d "%~dp0"
chcp 65001 >nul
if not exist ".venv\Scripts\python.exe" (
  py -3 -m venv .venv
  if errorlevel 1 exit /b 1
)
call ".venv\Scripts\python.exe" -m pip install -r requirements.txt
if errorlevel 1 exit /b 1
set "PYTHONPATH=%CD%\src"
call ".venv\Scripts\python.exe" -m duo %*
