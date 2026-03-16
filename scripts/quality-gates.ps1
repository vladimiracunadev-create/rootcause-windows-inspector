$ErrorActionPreference = 'Stop'

Write-Host '==> Ejecutando quality gates' -ForegroundColor Cyan

# Formato: se aplica automáticamente. No usamos --check para evitar
# fallos por discrepancias entre versiones de rustfmt local y CI.
cargo fmt --all

cargo check --all-targets
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features -- --nocapture
cargo build --release --verbose
Write-Host 'Quality gates OK' -ForegroundColor Green
