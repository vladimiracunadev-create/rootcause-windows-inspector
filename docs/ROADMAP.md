# Roadmap

## v0.5 Entregado — primer release publicado (consolida el trabajo previo)
- integracion WPR desde la UI
- estado de precision dentro del snapshot
- stop temporal de servicios permitidos
- documentacion de build y empaquetado endurecida
- scripts de hash y apertura WPA
- abrir carpeta de trazas desde la UI
- presets de captura mas claros
- relacion snapshot <-> ETL mas visible
- mejoras de whitelist local
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

## v0.11 Entregado (incluye el trabajo previo del tab Autostart)
- tab Autostart integrado en la GUI (Ctrl+7): entradas de registro HKCU/HKLM Run + carpetas Startup
- tabla con severidad heuristica, tipo de origen, comando completo con tooltip, indicador de existencia en disco
- chips de resumen por tipo de riesgo (sospechosas / a revisar)
- nota informativa al pie para entradas de Sistema vs Usuario
- atajos de teclado extendidos a Ctrl+1..9 (nuevo tab en posicion 7)
- mejoras profesionales a la interfaz: barra RAM normalizada a RAM real, tooltips Ctrl+N en tabs
- tareas programadas no-Microsoft integradas en el tab Autostart
- notas contextuales por tipo de entrada (RunOnce, HKLM, Scheduled Task, Startup)
- CLI `rootcause autostart [--json]` — lista entradas de persistencia desde consola
- panel Configuracion en tab Acerca con edicion inline de umbrales (CPU/RAM/IO, anomalias, refresco)
- boton Guardar persiste cambios a `rootcause-config.json` sin reiniciar via `save_config()` en InspectorService
- manifests Scoop/Winget/Chocolatey actualizados a 0.11.0
- version bump a 0.11.0

## v0.12 Entregado
- deteccion de cambios de autoarranque contra baseline conocida: da control explicito para saber si cambian los puntos de autoarranque de Windows
- baseline persistida en SQLite (tabla `persistence_baseline`) de las entradas de autoarranque (Registro Run/RunOnce HKCU/HKLM, carpetas Startup, tareas programadas no-Microsoft)
- clasificacion de cada entrada como NUEVA / MODIFICADA / ELIMINADA contra la baseline; primera foto = estado bueno conocido (silenciosa), cambios pegajosos hasta aceptar
- alertas kind `persistence-change` — Alta para nuevas/modificadas, Media para eliminadas
- aceptacion de baseline via UI (boton "✓ Aceptar estado actual como baseline" en el tab Autostart) y CLI (`rootcause autostart --accept`)
- `rootcause autostart --json` incluye el campo `change_status` por entrada
- manifests Scoop/Winget/Chocolatey actualizados a 0.12.0
- version bump a 0.12.0

## v0.13 Entregado (version actual)
- deteccion de cambios en servicios de Windows contra baseline conocida: da control explicito para saber si cambian los servicios instalados en el equipo
- motor generico de baseline reutilizable: generaliza el patron de autostart de v0.12 en un mecanismo comun; futuras superficies (hosts, registro, tareas) se anaden barato
- vigila todos los servicios y clasifica cada uno como NUEVO / MODIFICADO / ELIMINADO contra la baseline; el valor vigilado es StartMode + ruta del binario (cambio de modo de arranque o de binario)
- alertas kind `service-change` para servicios nuevos/modificados/eliminados
- aceptacion de baseline via CLI `rootcause services --accept`
- listado `rootcause services` (solo cambios) y `rootcause services --json` (incluye `change_status` por servicio)
- version bump a 0.13.0

## v1.0 Objetivo de distribucion formal
- tray icon activo (monitor proactivo en bandeja del sistema)
- firma digital (elimina alerta SmartScreen)
- publicacion en Scoop/Winget/Chocolatey con releases reales
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
