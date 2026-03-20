$ErrorActionPreference = 'Stop'

$root = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$source = Join-Path $root 'packaging\powershell\RootCause.psm1'
$destination = Join-Path $root 'build\RootCause.psm1'

if (-not (Test-Path $source)) {
    throw 'No se encontró packaging\powershell\RootCause.psm1'
}

Copy-Item $source $destination -Force
Write-Host "Módulo PowerShell listo: $destination" -ForegroundColor Green
