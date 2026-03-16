@echo off
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0analyze-last-etl.ps1" %*
