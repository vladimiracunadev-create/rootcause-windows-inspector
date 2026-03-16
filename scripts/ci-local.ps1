$ErrorActionPreference = 'Stop'

Write-Host '==> Replica local del flujo CI de GitHub Actions' -ForegroundColor Cyan

& "$PSScriptRoot\verify-environment.ps1"
if (-not (Test-Path (Join-Path $PSScriptRoot '..\Cargo.lock'))) {
    Write-Host '==> Cargo.lock no existe aún; se generará durante el primer build' -ForegroundColor Yellow
}

cargo check --all-targets
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features -- --nocapture
cargo build --release --verbose

Write-Host 'CI local completado correctamente.' -ForegroundColor Green
