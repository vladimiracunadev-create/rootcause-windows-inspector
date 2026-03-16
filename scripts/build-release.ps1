$ErrorActionPreference = 'Stop'

Write-Host '==> Compilando RootCause en release' -ForegroundColor Cyan

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw 'Cargo no está instalado. Instala Rustup primero.'
}

cargo build --release

$exe = Join-Path $PSScriptRoot '..\target\release\rootcause.exe'
$exe = [System.IO.Path]::GetFullPath($exe)

if (-not (Test-Path $exe)) {
    throw "No se encontró el ejecutable esperado: $exe"
}

Write-Host "OK -> $exe" -ForegroundColor Green
