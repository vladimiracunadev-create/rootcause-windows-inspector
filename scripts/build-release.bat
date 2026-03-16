@echo off
setlocal

echo ==> Compilando RootCause en release
where cargo >nul 2>nul
if errorlevel 1 (
  echo ERROR: Cargo no está instalado. Instala Rustup primero.
  exit /b 1
)

cargo build --release
if errorlevel 1 exit /b 1

echo.
echo OK -> target\release\rootcause.exe
