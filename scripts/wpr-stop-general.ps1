param(
    [string]$OutputPath = '',
    [string]$ProblemDescription = 'RootCause precision capture'
)

$ErrorActionPreference = 'Stop'

if (-not (Get-Command wpr -ErrorAction SilentlyContinue)) {
    throw 'wpr.exe no está disponible. Instala Windows Performance Toolkit antes de usar este script.'
}

$root = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$tracesDir = Join-Path $root 'build\traces'
New-Item -ItemType Directory -Path $tracesDir -Force | Out-Null

if ([string]::IsNullOrWhiteSpace($OutputPath)) {
    $timestamp = Get-Date -Format 'yyyyMMdd-HHmmss'
    $OutputPath = Join-Path $tracesDir ("rootcause-general-$timestamp.etl")
}

Write-Host "==> Deteniendo captura WPR y guardando ETL en $OutputPath" -ForegroundColor Cyan
wpr -stop $OutputPath $ProblemDescription -skipPdbGen -compress
Write-Host 'ETL generado correctamente.' -ForegroundColor Green
