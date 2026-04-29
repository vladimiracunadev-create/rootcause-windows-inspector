# Roadmap

## v0.3 Entregado
- integracion WPR desde la UI
- estado de precision dentro del snapshot
- stop temporal de servicios permitidos
- documentacion de build y empaquetado endurecida
- scripts de hash y apertura WPA

## v0.4 Entregado
- abrir carpeta de trazas desde la UI
- presets de captura mas claros
- relacion snapshot <-> ETL mas visible
- mejoras de whitelist local

## v0.5 Entregado
- parser resumido de ETL/metadata (`tracerpt` + `dumpfile.xml`)
- resumen del ultimo ETL desde la propia app
- exportacion `trace-analysis.json`
- CI en GitHub Actions y pipeline de release con ZIP/Inno/hashes

## v0.6 Entregado
- sparklines de CPU / RAM / I/O con `VecDeque`
- tab Historial con SQLite y comparacion A vs B
- filtro de severidad por procesos
- notificaciones toast de Windows
- command line por proceso via PowerShell batch
- tab Acerca con version, autor, stack, atajos y hardware
- landing publica inicial y hardening basico de acciones Windows

## v0.7 Entregado
- feature flags GUI / CLI-only
- retencion automatica en SQLite y backup JSON
- modulo PowerShell y manifests Scoop/Winget/Chocolatey
- extension VS Code
- skeletons documentados de tray y Windows Service
- configuracion operativa en `rootcause-config.json`
- auditoria local de acciones y CLI ampliada

## v0.8 Entregado
- modulo V1 de deteccion de comportamiento anomalo con heuristicas locales
- incidentes resumidos persistidos con evidencia correlacionada
- adaptador IA opcional desacoplado y apagado por defecto
- trazabilidad documental formal para REQ-SEC-001 y REQ-SEC-002

## v0.9 Entregado
- estado explicito de salud del agente con `Healthy / Recovered / Degraded`
- heartbeat local persistido para el propio agente
- deteccion de cierre abrupto previo y recuperacion visible en la siguiente sesion
- evidencia basica de integridad de configuracion mediante huella local
- recomendacion de backoff ante reinicios/cierres abruptos repetidos
- visibilidad del estado del agente en GUI, `status --json`, `config show` y snapshot exportado
- documentacion, landing y manifests alineados con la version 0.9.0

## v0.10 Entregado
- tab Autostart integrado en la GUI (Ctrl+7): entradas de registro HKCU/HKLM Run + carpetas Startup
- tabla con severidad heuristica, tipo de origen, comando completo con tooltip, indicador de existencia en disco
- chips de resumen por tipo de riesgo (sospechosas / a revisar)
- nota informativa al pie para entradas de Sistema vs Usuario
- atajos de teclado extendidos a Ctrl+1..9 (nuevo tab en posicion 7)
- mejoras profesionales a la interfaz: barra RAM normalizada a RAM real, tooltips Ctrl+N en tabs
- version bump a 0.10.0

## v0.11 Entregado (version actual)
- tareas programadas no-Microsoft integradas en el tab Autostart
- notas contextuales por tipo de entrada (RunOnce, HKLM, Scheduled Task, Startup)
- CLI `rootcause autostart [--json]` — lista entradas de persistencia desde consola
- panel Configuracion en tab Acerca: umbrales CPU/RAM/IO, anomalias, refresco, boton abrir config en Notepad
- version bump a 0.11.0

## v1.0 Objetivo de distribucion formal
- tray icon activo
- alertas editables desde la UI (no solo visibles)
- firma digital
- documentacion madura de instalacion y operacion

## Lineas estrategicas documentadas
- Implementacion actual en el repositorio: V1 inicial del modulo de deteccion de comportamiento anomalo con heuristicas locales, correlacion simple, evidencia tecnica, configuracion y exposicion en GUI/CLI.
- Referencia tecnica: `docs/MODULO_DETECCION_ANOMALIAS.md`
- Estado actualizado:
- `REQ-SEC-001` queda en `phase-1-implemented`.
- `REQ-SEC-002` queda en `phase-2-initial`.
- `REQ-SEC-001`: deteccion heuristica y correlacion de senales compatibles con actividad no autorizada, sin posicionar RootCause como antivirus o EDR.
- `REQ-SEC-002`: resiliencia inicial del agente con heartbeat, recuperacion tras cierre abrupto, integridad basica de configuracion y evidencia local, sin prometer invulnerabilidad ni un supervisor persistente de nivel servicio.
- Registro permanente: `docs/requirements/README.md`

## v2.0+ Largo plazo
- Windows Service activo (captura 24/7, named pipe, diagnostico nocturno)
- edicion Seguridad orientada a SOC
- edicion Enterprise con multi-equipo y observabilidad centralizada
- MSIX / Microsoft Store
