param(
    [string]$ProblemDescription = 'RootCause precision capture'
)

$ErrorActionPreference = 'Stop'

if (-not (Get-Command wpr -ErrorAction SilentlyContinue)) {
    throw 'wpr.exe no está disponible. Instala Windows Performance Toolkit antes de usar este script.'
}

$root = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$tracesDir = Join-Path $root 'build\traces'
New-Item -ItemType Directory -Path $tracesDir -Force | Out-Null

Write-Host '==> Iniciando captura WPR con GeneralProfile en filemode' -ForegroundColor Cyan
wpr -start GeneralProfile -filemode -recordtempto $tracesDir
wpr -marker "RootCause precision: $ProblemDescription"
Write-Host 'Captura iniciada. Reproduce el problema y luego ejecuta scripts\wpr-stop-general.ps1' -ForegroundColor Green
