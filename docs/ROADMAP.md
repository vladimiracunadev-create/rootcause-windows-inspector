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
- ✅ CLI completa: `rootcause <comando>` desde consola de Windows con `--help` y todos los comandos
- ✅ tab **Acerca**: versión, autor, email, GitHub, GitLab, stack técnico, atajos, hardware
- ✅ atajos de teclado: `F5` actualizar, `Ctrl+E` exportar, `Ctrl+1…8` cambio de tab
- ✅ características del equipo: sección en tab Resumen y tab Acerca (OS, CPU, núcleos, RAM)
- ✅ módulo `meta.rs`: constantes del producto en un único lugar
- ✅ cero crates nuevos: todo con primitivos egui, PowerShell, SQLite y sysinfo existente
- ✅ landing page pública: `rootcause-landing` en GitHub Pages con releases públicos
- ✅ hardening de seguridad: validación estricta de IP y nombre de servicio antes de invocar PowerShell (`is_valid_firewall_ip` en `windows.rs`), defensa en profundidad contra command injection

## v1.0 — Objetivo de distribución formal
- producto estable para distribución más formal
- documentación madura de instalación y operación
- firma digital si el presupuesto y proceso lo permiten
- posibles rutas futuras a MSIX y distribución más empresarial
