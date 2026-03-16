$ErrorActionPreference = 'Stop'

Write-Host '==> Ejecutando quality gates' -ForegroundColor Cyan
cargo check --all-targets
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features -- --nocapture
cargo build --release --verbose
Write-Host 'Quality gates OK' -ForegroundColor Green
