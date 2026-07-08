# 🗺️ Roadmap

> Historial de versiones entregadas y objetivos futuros de RootCause. El detalle de cada versión se conserva más abajo; esta tabla es solo un resumen de navegación.

## Resumen de versiones

| Versión | Estado | Hito principal |
|---|---|---|
| [v0.5](#v05-entregado--primer-release-publicado-consolida-el-trabajo-previo) | ✅ Entregado | Primer release publicado: integración WPR, CI y pipeline de release |
| [v0.6](#v06-entregado) | ✅ Entregado | Sparklines, tab Historial con SQLite y comparación A/B |
| [v0.7](#v07-entregado) | ✅ Entregado | Feature flags GUI/CLI, módulo PowerShell y manifests |
| [v0.8](#v08-entregado) | ✅ Entregado | Módulo V1 de detección de comportamiento anómalo |
| [v0.9](#v09-entregado) | ✅ Entregado | Salud del agente: heartbeat, recuperación y backoff |
| [v0.11](#v011-entregado-incluye-el-trabajo-previo-del-tab-autostart) | ✅ Entregado | Tab Autostart + umbrales editables inline |
| [v0.12](#v012-entregado) | ✅ Entregado | Detección de cambios de autoarranque contra baseline |
| [v0.13](#v013-entregado) | ✅ Entregado | Detección de cambios en servicios + motor genérico de baseline |
| [v0.14](#v014-entregado) | ✅ Entregado | Overhaul de UI (ventana, scroll, glifos) + fix de colección PowerShell |
| [v0.15](#v015-entregado) | ✅ Entregado | Idioma ES/EN + tab Configuración, sección Docker, banner de veredicto y Manual profundo |
| [v0.16](#v016-entregado) | ✅ Entregado | Icono de bandeja con color por severidad, tooltip de veredicto y menú de acciones |
| [v0.17](#v017-entregado-version-actual) | ✅ **Actual** | Rediseño Fluent/Win11: barra lateral, iconos de línea, Segoe UI, logo del radar, modos de tema |
| [v1.0](#v10-objetivo-de-distribucion-formal) | 🎯 Objetivo | Distribución formal: firma digital, publicación, cerrar-a-bandeja |
| [v2.0+](#v20-largo-plazo) | 🔭 Largo plazo | Windows Service 24/7, ediciones Seguridad y Enterprise |

---

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

## v0.13 Entregado
- deteccion de cambios en servicios de Windows contra baseline conocida: da control explicito para saber si cambian los servicios instalados en el equipo
- motor generico de baseline reutilizable: generaliza el patron de autostart de v0.12 en un mecanismo comun; futuras superficies (hosts, registro, tareas) se anaden barato
- vigila todos los servicios y clasifica cada uno como NUEVO / MODIFICADO / ELIMINADO contra la baseline; el valor vigilado es StartMode + ruta del binario (cambio de modo de arranque o de binario)
- alertas kind `service-change` para servicios nuevos/modificados/eliminados
- aceptacion de baseline via CLI `rootcause services --accept`
- listado `rootcause services` (solo cambios) y `rootcause services --json` (incluye `change_status` por servicio)
- version bump a 0.13.0

## v0.14 Entregado
- UI: la ventana se dimensiona al area de trabajo real del monitor al arrancar (antes no se ajustaba y la barra de tareas recortaba el fondo)
- UI: barras de scroll solidas y siempre visibles en todos los tabs (antes la barra flotante era casi invisible y parecia que el Resumen no tenia scroll)
- UI: barrido de glifos que la fuente incluida no renderiza (salian como "cuadrado") en chips de severidad, iconos de estado, marcas de verificacion y botones
- UI: nuevo tab Manual (guia integrada de que hace cada pestana, la deteccion por baseline y las acciones seguras); ahora 10 tabs (Manual = Ctrl+9, Acerca = Ctrl+0)
- UI: limpieza segura de %TEMP% desde el tab Temporales (boton con confirmacion de 2 pasos) y por CLI `rootcause clean-temp [--yes]` (solo tu %TEMP%, >24h, salta lo bloqueado)
- UI: tarjetas de anomalias y sparklines del Resumen dejan de desbordar el texto
- FIX de coleccion: los tabs Servicios y Eventos recientes salian siempre vacios porque PowerShell devuelve exit code distinto de cero ante errores no-terminantes (p. ej. un servicio inexistente) aunque emita datos validos; ahora se usa la salida util
- FIX de codificacion: la salida de PowerShell se fuerza a UTF-8 (antes los acentos salian como "cuadrado" en nombres de servicios, eventos y rutas)
- version bump a 0.14.0

## v0.17 Entregado (version actual)
- Rediseño de interfaz estilo **Windows 11 / Fluent**, inspirado en la calidez de PC Manager sin clonarlo, conservando la densidad de datos
- Barra lateral (NavigationView) con navegacion agrupada (Actividad / Sistema / Analisis) en vez de las 11 pestañas superiores; Config/Manual/Acerca anclados abajo
- Iconos de linea dibujados con el Painter (sin emoji ni fuente externa) en la navegacion
- Tipografia nativa: Segoe UI (proporcional) y Consolas (monoespaciada) del sistema
- Logo de la marca: radar de circulos concentricos (igual que el .ico) en header e icono de ventana
- Modos de tema **Claro / Oscuro / Windows** (sigue el tema del sistema) seleccionables en Configuracion; persistido en `config.ui.theme`
- Colores en runtime (Palette/tokens) con acento = azul del icono (#1f6feb) en todos los modos
- Banner de veredicto tipo hero en el Resumen (de v0.15) integrado en el nuevo lenguaje visual
- version bump a 0.17.0

## v0.16 Entregado
- Tray: icono de bandeja del sistema activo en la edicion GUI (feature `gui` arrastra la crate `tray-icon`)
- Tray: el icono es un punto de color segun la salud global (verde = saludable, ambar = advertencia, rojo = critico)
- Tray: tooltip con el veredicto actual y el score de salud
- Tray: menu contextual con Mostrar ventana / Actualizar ahora / Exportar snapshot / Salir (drenado cada frame)
- Tray: creado en el hilo del event-loop de winit; creacion no fatal (si el SO lo rechaza, la app sigue sin bandeja)
- El modulo `src/services/tray.rs` deja de ser skeleton y pasa a implementacion real
- Pendiente para v1.0: cerrar-a-bandeja (mantener el proceso al cerrar la ventana)
- version bump a 0.16.0

## v0.15 Entregado
- UI: interfaz bilingue espanol / ingles con selector persistente; motor i18n con helper tr(es,en) e idioma guardado en la config (`ui.language`)
- UI: nuevo tab Configuracion (Ctrl+9) que reune idioma, umbrales de deteccion, anomalias e intervalo de refresco (movidos desde Acerca)
- UI: banner de veredicto tipo hero en el Resumen (aro de salud + titular + causa dominante en una linea)
- Almacenamiento: seccion Docker en el tab Temporales (docker system df / images / volume ls) con barra segmentada por categoria, tablas de imagenes y volumenes, y purga guiada segura de 2 pasos (solo imagenes dangling + cache de build; los volumenes nunca se autoborran porque contienen datos)
- CLI: nuevo comando `rootcause docker [--json | --prune-images | --prune-cache]`
- UI: Manual reescrito con el porque de cada parte (bilingue) + secciones nuevas (leelo en 30 segundos, Docker); ahora 11 tabs (Configuracion = Ctrl+9, Manual = Ctrl+0, Acerca solo por clic)
- Inspirado en la UX de Microsoft PC Manager, sin clonarlo: se toma su lenguaje visual amigable y se resuelve mejor lo que insinua (baseline, causa raiz)
- version bump a 0.15.0

## v1.0 Objetivo de distribucion formal
- tray icon: cerrar-a-bandeja (el icono base ya se entrego en v0.16)
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
