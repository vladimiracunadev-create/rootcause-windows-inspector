# Roadmap

## v0.3 ✅ Entregado
- ✅ integración WPR desde la UI
- ✅ estado de precisión dentro del snapshot
- ✅ stop temporal de servicios permitidos
- ✅ documentación de build y empaquetado endurecida
- ✅ scripts de hash y apertura WPA

## v0.4 ✅ Entregado
- ✅ abrir carpeta de trazas desde la UI
- ✅ presets de captura más claros
- ✅ relación snapshot ↔ ETL más visible
- ✅ mejoras de whitelist local

## v0.5 ✅ Entregado
- ✅ parser resumido de ETL/metadata (`tracerpt` + `dumpfile.xml`)
- ✅ resumen del último ETL desde la propia app
- ✅ exportación `trace-analysis.json`
- ✅ CI en GitHub Actions (formato, lint, tests, build release, artefactos)
- ✅ pipeline de release con ZIP portable, instalador Inno y hashes SHA-256
- ✅ repositorio publicado en GitHub
- ✅ release de GitHub con instrucciones completas de instalación y verificación
- ✅ estabilización de CI: toolchain fijo, `rustfmt.toml` con `max_width = 100`
- ✅ corrección de patrones Rust que rompían CI bajo `-D warnings`
- ✅ corrección de tests: escape de backslash en strings y orden de condiciones

## v0.6 ✅ Entregado (versión actual)
- ✅ sparklines de CPU / RAM / I/O con ring buffer (`VecDeque`) — sin crates extra
- ✅ tab **Historial** con tabla SQLite y comparación A vs B con deltas
- ✅ filtro de severidad por tab de procesos (Critical / Warning / Normal / todos)
- ✅ notificaciones toast de Windows cuando aparece proceso Critical (PowerShell, non-blocking)
- ✅ correlación proceso ↔ command line via `Get-CimInstance Win32_Process` en batch
- ✅ ETL summary enriquecido: barra de proveedores ETW e indicadores en la UI
- ✅ instalador silencioso: soporte `/VERYSILENT /SUPPRESSMSGBOXES /NORESTART` en Inno Setup
- ✅ cero crates nuevos: todo con primitivos egui, PowerShell y SQLite existente

## v1.0 — Objetivo de distribución formal
- producto estable para distribución más formal
- documentación madura de instalación y operación
- firma digital si el presupuesto y proceso lo permiten
- posibles rutas futuras a MSIX y distribución más empresarial
