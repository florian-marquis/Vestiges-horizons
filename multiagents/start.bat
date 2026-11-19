@echo off
setlocal
cd /d "%~dp0"
chcp 65001 >nul
".venv\Scripts\python.exe" -c "import sys" >nul 2>&1
if errorlevel 1 (
  python -m venv --clear .venv >nul 2>&1
  if errorlevel 1 (
    py -3 -m venv --clear .venv
    if errorlevel 1 exit /b 1
  )
)
call ".venv\Scripts\python.exe" -m pip install -r requirements.txt
if errorlevel 1 exit /b 1
set "PYTHONPATH=%CD%\src"
call ".venv\Scripts\python.exe" -m duo %*
