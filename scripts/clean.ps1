$ErrorActionPreference = 'Stop'

$root = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$buildDir = Join-Path $root 'build'

Write-Host '==> Limpiando artefactos Cargo' -ForegroundColor Cyan
cargo clean

if (Test-Path $buildDir) {
    Write-Host '==> Eliminando carpeta build' -ForegroundColor Cyan
    Remove-Item $buildDir -Recurse -Force
}

Write-Host 'Limpieza finalizada.' -ForegroundColor Green
