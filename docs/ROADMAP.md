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

## v0.5 ✅ Entregado (versión actual)
- ✅ parser resumido de ETL/metadata (`tracerpt` + `dumpfile.xml`)
- ✅ resumen del último ETL desde la propia app
- ✅ exportación `trace-analysis.json`
- ✅ CI en GitHub Actions (formato, lint, tests, build release, artefactos)
- ✅ pipeline de release con ZIP portable, instalador Inno y hashes SHA-256
- ✅ repositorio publicado en GitHub
- ✅ release de GitHub con instrucciones completas de instalación y verificación
- ✅ estabilización de CI: toolchain fijo, `rustfmt.toml` con `max_width = 100`
- ✅ corrección de patrones Rust que rompían CI bajo `-D warnings`:
  - `Ok(format!(...));` con `;` sobrante en bloques `#[cfg]`
  - `collapsible_if` con let-chains Rust 2024
  - `needless_return` en colas de funciones
  - `collapsible_str_replace` con caracteres múltiples
- ✅ corrección de tests: escape de backslash en strings y orden de condiciones

## v0.6 — Próxima iteración
- exportación de evidencia más rica
- notas de caso dentro del historial
- empaquetado más corporativo
- comparación temporal entre capturas
- timeline básica de síntomas

## v1.0 — Objetivo de distribución formal
- producto estable para distribución más formal
- documentación madura de instalación y operación
- firma digital si el presupuesto y proceso lo permiten
- posibles rutas futuras a MSIX y distribución más empresarial
