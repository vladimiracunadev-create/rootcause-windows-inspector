@echo off
setlocal

echo ==> Verificando entorno base

where cargo >nul 2>nul
if errorlevel 1 (
  echo [FAIL] cargo no encontrado. Instala Rustup.
  exit /b 1
) else (
  echo [OK] cargo
)

where rustup >nul 2>nul
if errorlevel 1 (
  echo [FAIL] rustup no encontrado. Instala Rustup.
  exit /b 1
) else (
  echo [OK] rustup
)

where cl >nul 2>nul
if errorlevel 1 (
  echo [WARN] cl no encontrado. Instala Visual Studio Build Tools con Desktop development with C++.
) else (
  echo [OK] cl
)

where iscc >nul 2>nul
if errorlevel 1 (
  echo [WARN] iscc no encontrado. Instala Inno Setup si empaquetaras instalador.
) else (
  echo [OK] iscc
)


where tracerpt >nul 2>nul
if errorlevel 1 (
  echo [WARN] tracerpt no encontrado. Instala o habilita tracerpt si usaras resumen ETL local.
) else (
  echo [OK] tracerpt
)

where wpr >nul 2>nul
if errorlevel 1 (
  echo [WARN] wpr no encontrado. Instala Windows Performance Toolkit si usaras modo precision.
) else (
  echo [OK] wpr
)

where wpa >nul 2>nul
if errorlevel 1 (
  echo [WARN] wpa no encontrado. Instala Windows Performance Analyzer si abriras ETL.
) else (
  echo [OK] wpa
)

echo.
echo Entorno base verificado.
