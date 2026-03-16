$ErrorActionPreference = 'Stop'

if (-not (Get-Command wpa -ErrorAction SilentlyContinue)) {
    throw 'wpa.exe no está disponible. Instala Windows Performance Analyzer.'
}

$roots = @(
    (Join-Path $env:USERPROFILE 'Documents\RootCause\traces'),
    (Join-Path $env:USERPROFILE 'Downloads\RootCause\traces'),
    ([System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\build\traces')))
) | Select-Object -Unique

$latest = $null
foreach ($root in $roots) {
    if (-not (Test-Path $root)) {
        continue
    }

    $candidate = Get-ChildItem -Path $root -Filter *.etl -File -ErrorAction SilentlyContinue |
        Sort-Object LastWriteTime -Descending |
        Select-Object -First 1

    if ($candidate) {
        if (-not $latest -or $candidate.LastWriteTime -gt $latest.LastWriteTime) {
            $latest = $candidate
        }
    }
}

if (-not $latest) {
    throw 'No se encontró ningún ETL reciente ni en Documents/Downloads ni en build\traces.'
}

Write-Host "==> Abriendo $($latest.FullName) en WPA" -ForegroundColor Cyan
Start-Process wpa $latest.FullName
