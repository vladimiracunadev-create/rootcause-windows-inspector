$ErrorActionPreference = 'Stop'

$root = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$iss = Join-Path $root 'packaging\windows\RootCause.iss'
$exe = Join-Path $root 'target\release\rootcause.exe'

if (-not (Test-Path $exe)) {
    throw 'No existe el ejecutable release. Ejecuta antes scripts\build-release.ps1'
}

$possibleIscc = @(
    (Get-Command iscc -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source -ErrorAction SilentlyContinue),
    'C:\Program Files (x86)\Inno Setup 6\ISCC.exe',
    'C:\Program Files\Inno Setup 6\ISCC.exe'
) | Where-Object { $_ }

$iscc = $possibleIscc | Where-Object { Test-Path $_ } | Select-Object -First 1

if (-not $iscc) {
    throw 'No se encontró ISCC.exe. Instala Inno Setup o ajusta la ruta en este script.'
}

Write-Host "==> Compilando instalador con $iscc" -ForegroundColor Cyan
& $iscc $iss

$installerDir = Join-Path $root 'build\installer'
if (Test-Path $installerDir) {
    Write-Host "Instalador generado en: $installerDir" -ForegroundColor Green
}
else {
    throw 'ISCC terminó, pero no se encontró la carpeta build\installer esperada.'
}
