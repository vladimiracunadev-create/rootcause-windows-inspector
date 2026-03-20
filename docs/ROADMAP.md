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

## v0.6 ✅ Entregado
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

## v0.7 ✅ Entregado (versión actual)
- ✅ Feature flags Cargo.toml: edición GUI (`--features gui`, default) y CLI-only (`--no-default-features`, ~4 MB sin egui)
- ✅ `#[cfg(feature = "gui")]` en `main.rs` — compilación limpia sin GUI
- ✅ SQLite retención automática (últimas 1000 filas) — evita crecimiento indefinido de la BD
- ✅ Backup JSON automático al exportar (`rootcause-history-backup.json`) — recuperación ante fallo SQLite
- ✅ Módulo PowerShell `RootCause.psm1` — 9 cmdlets nativos (`Get-RootCauseStatus`, `Stop-RootCauseProcess`, etc.)
- ✅ Manifests de distribución: Scoop (`packaging/distribution/scoop/rootcause.json`), Winget (`VladimirAcuna.RootCause`), Chocolatey
- ✅ Extensión VS Code: status bar con CPU/RAM/severidad en tiempo real, alertas Critical, panel de diagnóstico, 3 comandos (`rootcause.refresh`, `rootcause.export`, `rootcause.openPanel`)
- ✅ Skeleton documentado de tray icon (`src/services/tray.rs`) — arquitectura con menú contextual y cambio de color por severidad
- ✅ Skeleton Windows Service (`src/bin/rootcause-service.rs`) — arquitectura con SCM, named pipe y loop de captura documentados
- ✅ Configuración operativa en `rootcause-config.json` con defaults seguros para captura, retención, umbrales y acciones
- ✅ Motor ligero de reglas/correlación con incidentes resumidos persistidos en SQLite y evidencia asociada
- ✅ Auditoría de acciones locales (`kill`, `block-ip`, `stop-service`, ETW/WPR, IA opcional) sin depender de servicios externos
- ✅ CLI ampliada con `status --json`, `history --json`, `incidents`, `config show/init`, `snapshot --output` y `ai explain-latest`
- ✅ Adaptador IA opcional por API, desacoplado y apagado por defecto: si falla, RootCause sigue detectando, alertando y guardando evidencia
- ✅ Release engineering endurecido: branding principal corregido y workflow `release-windows` alineado con el catálogo real (GUI, CLI-only, PowerShell y VS Code)
- ✅ Documentación estratégica formalizada para REQ-SEC-001 y REQ-SEC-002 con trazabilidad en README, índice documental y landing pública

## v1.0 — Objetivo de distribución formal
- Tab Autostart (HKCU\...\Run + carpeta Startup + tareas programadas)
- Tray icon activo (activar feature `tray`, `tray-icon = "0.14"`)
- Alertas y umbrales configurables (`rootcause.toml` en AppData)
- `rootcause snapshot --output <ruta>` en CLI
- Scoop / Winget publicados con primer release público
- Firma digital (CodeSigning cert — elimina SmartScreen)
- Documentación madura de instalación y operación

## Lineas estrategicas documentadas (post-v1.0)
- `REQ-SEC-001` — `planned` · prioridad alta estrategica: deteccion heuristica y correlacion de senales compatibles con actividad no autorizada o potencialmente maliciosa, sin posicionar RootCause como antivirus o EDR.
- `REQ-SEC-002` — `planned` · prioridad alta: resiliencia del agente mediante watchdog, reinicio, integridad, proteccion de configuracion y alertas ante manipulacion, sin prometer invulnerabilidad.
- Registro permanente: `docs/requirements/README.md`

## v2.0+ — Largo plazo
- Windows Service activo (captura 24/7, named pipe, diagnóstico de problemas nocturnos)
- Edición Seguridad (solo procesos sospechosos + conexiones + bloqueo, orientada a SOC)
- Edición Enterprise (Prometheus/Grafana, multi-equipo, GPO, CSV/Excel)
- MSIX / Microsoft Store
