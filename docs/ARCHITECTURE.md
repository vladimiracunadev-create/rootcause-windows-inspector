# Arquitectura

Este documento explica la estructura técnica del proyecto y cómo se relacionan sus capas.

---

## 1) Objetivo arquitectónico

La arquitectura busca equilibrar cuatro cosas al mismo tiempo:

1. **bajo consumo**,
2. **claridad de código**,
3. **capacidad de diagnóstico real**,
4. **ruta profesional a mayor precisión**.

No se eligió una base web pesada ni Electron porque el propio monitor no debe convertirse en una fuente adicional de consumo.

---

## 2) Stack principal

### Lenguaje
- Rust

### GUI
- `eframe/egui`

### Persistencia
- SQLite vía `rusqlite`

### Métricas locales
- `sysinfo`

### Integración Windows
- PowerShell
- `netstat`
- `taskkill`
- `wpr`
- `wpa`
- `tracerpt`

---

## 3) Capas del código

```text
src/
├── config.rs
├── main.rs
├── app.rs
├── cli.rs
├── meta.rs
├── models.rs
└── services/
    ├── mod.rs
    ├── ai.rs
    ├── rules.rs
    ├── inspector.rs
    ├── network.rs
    ├── persistence.rs
    ├── baseline.rs
    ├── temp_scan.rs
    ├── etl.rs
    └── windows.rs
```

---

## 4) Responsabilidad por archivo

### `main.rs`
Punto de entrada. Detecta si hay argumentos CLI; si los hay, despacha a `cli::run()` y termina el proceso. Si no, levanta la ventana GUI.

### `meta.rs`
Constantes del producto: `VERSION`, `DISPLAY_NAME`, `AUTHOR`, `EMAIL`, `GITHUB`, `GITLAB`, `LICENSE`, `DESCRIPTION`. Único lugar de verdad; se usan en CLI y en el tab Acerca.

### `cli.rs`
Interfaz de línea de comandos completa.

Responsabilidades:
- `run(&[String]) -> i32` con dispatch de comandos,
- `--help` con ASCII art del producto,
- `--version`, `status [--json]`, `snapshot [--output PATH]`, `history [N] [--json]`, `incidents [N] [--json]`, `export`,
- `config show/init`,
- `ai explain-latest [--json]`,
- `wpr start/stop/cancel/analyze [--note NOTE]`,
- `autostart [--json] [--accept]`, `services [--json] [--accept]`,
- `kill <PID>`, `block-ip <IP>`, `stop-service <name>`.

### `config.rs`
Gestión de configuración operativa local.

Responsabilidades:
- cargar `rootcause-config.json` desde AppData,
- exponer defaults seguros para umbrales, retención y acciones,
- mantener IA opcional desactivada por defecto.

### `app.rs`
Capa de interfaz.

Responsabilidades:
- layout general con 10 tabs (Resumen, Procesos, Conexiones, Temporales, ETW/WPR, Servicios, Autostart, Historial, Manual, Acerca),
- atajos de teclado: `F5` = actualizar, `Ctrl+E` = exportar, `Ctrl+1…9` y `Ctrl+0` = cambio de tab,
- semáforo,
- sparklines de CPU / RAM / I/O (ring buffer `VecDeque<MetricSample>`, max 60 muestras),
- sección "Características del equipo" en tab Resumen (datos de `HardwareInfo`),
- filtro de severidad por tab de procesos,
- tab Autostart: tabla de entradas de registro Run (HKCU/HKLM), carpetas Startup y tareas programadas no-Microsoft con severidad heurística, comparadas contra la baseline conocida (`persistence_baseline`) para señalar cambios (`persistence-change`) NUEVA / MODIFICADA / ELIMINADA,
- tab de Historial con tabla SQLite y comparación A vs B,
- tab Temporales con escaneo de cachés y botón de limpieza segura de `%TEMP%` (>24h, no en uso),
- tab Manual: guía integrada que explica cada pestaña, la detección por baseline y las acciones seguras,
- notificaciones toast vía PowerShell (non-blocking),
- tab Acerca con versión, autor, links, atajos y hardware del equipo,
- control del modo de precisión,
- vista de resumen ETL con barra de proveedores.

### `models.rs`
Modelos serializables.

Responsabilidades:
- snapshot de sistema,
- procesos (incluye `command_line: Option<String>` con `#[serde(default)]`),
- conexiones,
- temporales,
- servicios,
- estado de precisión,
- resumen ETL,
- `SnapshotRow` para filas del historial SQLite,
- `IncidentSummary` para correlación persistida,
- `AuditRecord` para trazabilidad de acciones,
- `AiIncidentAdvice` para enriquecimiento opcional,
- `HardwareInfo` para datos estáticos del hardware (OS, CPU, RAM, arquitectura).

### `services/rules.rs`
Motor ligero de reglas y correlación.

Responsabilidades:
- clasificar procesos con umbrales configurables,
- construir alertas a partir de procesos, red, temporales y servicios,
- derivar incidentes resumidos con evidencia y acciones sugeridas.

### `services/inspector.rs`
Orquestador principal.

Responsabilidades:
- refrescar métricas,
- calcular deltas,
- ensamblar el snapshot completo,
- aplicar reglas y correlación,
- persistir snapshots, incidentes y auditoría,
- `get_hardware_info()` — recopila datos de hardware una sola vez al iniciar,
- exponer acciones de UI,
- coordinar ETL + resumen,
- invocar IA opcional solo sobre incidentes ya persistidos.

### `services/network.rs`
Parsea `netstat` y clasifica conexiones.

### `services/temp_scan.rs`
Escanea rutas temporales y cachés relevantes.

### `services/windows.rs`
Adaptador de utilidades nativas de Windows.

Responsabilidades:
- PowerShell,
- `netstat`,
- `taskkill`,
- firewall,
- servicios,
- WPR,
- `tracerpt`,
- `show_toast_notification()` vía WinRT/PowerShell (non-blocking),
- `batch_process_cmdlines()` vía `Get-CimInstance Win32_Process` en batch,
- `persistence_entries()` — recopila entradas de registro Run/RunOnce (HKCU/HKLM), carpetas Startup y tareas programadas no-Microsoft vía PowerShell; `InspectorService::detect_persistence_changes()` las compara contra la baseline y clasifica cada entrada como NUEVA / MODIFICADA / ELIMINADA o sin cambios,
- `services_baseline_items()` — enumera `Win32_Service` (Name, DisplayName, StartMode, PathName) como `WatchedItem` para el motor genérico de baseline; `InspectorService::detect_service_changes()` compara el valor vigilado `StartMode|PathName` contra la baseline de la superficie Servicios y emite el kind `service-change`,
- `is_valid_firewall_ip()` — validación estricta de IPv4/IPv6 antes de construir scripts PowerShell (defensa contra command injection).

### `services/etl.rs`
Resumen asistido del ETL.

Responsabilidades:
- analizar `dumpfile.xml`,
- extraer rutas, imágenes, IPs e indicadores,
- generar `trace-analysis.json`,
- producir un resumen consumible por la UI.

### `services/persistence.rs`
Persistencia local SQLite.

Responsabilidades:
- guardar snapshots,
- guardar incidentes resumidos,
- registrar auditoría de acciones y de IA,
- mantener el esquema SQLite: `snapshots`, `incidents`, `audit_log`, `persistence_baseline` (baseline conocida de autoarranque con `entry_key`, `entry_kind`, `location`, `name`, `command`, `target_path` y `first_seen`) y la tabla genérica `baseline(surface, entry_key, value, label, detail, first_seen)` con PK compuesta `(surface, entry_key)` que respalda el motor genérico de detección de cambios por superficie,
- `load_recent(limit)` — devuelve últimas N filas como `Vec<SnapshotRow>`,
- `load_persistence_baseline()` / `replace_persistence_baseline()` — leen y siembran/reemplazan la baseline de autoarranque para la detección de cambios,
- `load_baseline(surface)` / `replace_baseline(surface, items)` — leen y siembran/reemplazan la baseline genérica de cualquier superficie sobre la tabla `baseline`,
- parsea `alerts_json` para derivar `alerts_count` y `has_critical`.

### `services/baseline.rs`
Motor genérico de detección de cambios contra baseline. Generaliza el patrón introducido por el autostart para poder aplicarlo a cualquier superficie observable.

Responsabilidades:
- `WatchedItem { key, value, label, detail, change_status }` — unidad genérica vigilada por superficie,
- `diff_surface(store, surface_id, items)` — compara los `WatchedItem` observados contra la baseline de esa superficie y clasifica cada uno como NUEVA / MODIFICADA / ELIMINADA (la primera foto siembra en silencio; los cambios quedan pegajosos),
- `surface_change_event()` — genera un `AnomalyEvent` con kind `<surface>-change` a partir de los cambios detectados.

La primera superficie construida sobre este motor es **Servicios**: `windows::services_baseline_items()` enumera `Win32_Service` (Name, DisplayName, StartMode, PathName) y el valor vigilado es `StartMode|PathName` (no el estado en ejecución). En `InspectorService`, `detect_service_changes()`, `accept_service_baseline()` y `service_entries_with_changes()` (con la const `SERVICE_SURFACE`) coordinan la detección, que emite el kind de anomalía `service-change`.

El autostart de v0.12 sigue en su ruta dedicada `persistence_baseline` sin cambios; su migración al motor genérico es futura.

### `services/ai.rs`
Adaptador opcional de IA.

Responsabilidades:
- recibir un incidente ya resumido,
- invocar un endpoint compatible por API,
- devolver resumen, causas probables, acciones y confianza,
- fallar de forma aislada sin afectar la detección principal.

---

## 5) Flujo de datos

### Observación liviana
1. `app.rs` pide refresh,
2. `inspector.rs` consulta procesos, red, temporales, eventos y servicios,
3. `rules.rs` clasifica y correlaciona señales,
4. arma `SystemSnapshot`,
5. persiste snapshot + incidente resumido en SQLite,
6. devuelve datos a la UI.

### Enriquecimiento IA opcional
1. el motor ya detectó y persistió un incidente local,
2. CLI o futuras vistas piden enriquecimiento,
3. `services/ai.rs` envía un paquete resumido al proveedor configurado,
4. si responde bien, se actualiza el incidente persistido,
5. si falla, RootCause sigue funcionando y registra auditoría.

### Modo de precisión
1. `app.rs` dispara captura WPR,
2. `windows.rs` ejecuta `wpr`,
3. se guarda `.etl`,
4. `app.rs` dispara resumen ETL,
5. `windows.rs` ejecuta `tracerpt`,
6. `etl.rs` analiza `dumpfile.xml`,
7. se genera `trace-analysis.json`,
8. `inspector.rs` lo reincorpora al snapshot y la UI lo muestra.

---

## 6) Decisiones de diseño clave

### GUI nativa ligera
Para minimizar consumo y complejidad.

### Integración con herramientas nativas
Para aprovechar lo que Windows ya expone, en vez de introducir dependencias más pesadas desde el inicio.

### Persistencia pequeña
SQLite guarda solo lo útil para comparar tendencias, revisar incidentes y auditar acciones, no un universo de datos crudos.

### ETL como capa progresiva
No se obliga al usuario a entrar a ETW desde el primer minuto. Primero se observa; luego, si hace falta, se sube de nivel.

### Artefactos auditables
El resumen ETL produce archivos intermedios visibles y revisables.

---

## 7) Límites de la arquitectura actual

- no hay servicio de fondo persistente,
- no hay driver,
- no hay parser completo de WPA,
- no hay carga de símbolos dentro de la app,
- no hay correlación temporal fina por intervalos seleccionables dentro de la UI,
- no hay integración directa con ETWAnalyzer u otros motores externos.

---

## 8) Expansiones previstas

### Corto plazo (v0.7+)
- notas de caso dentro del historial,
- exportación de evidencia más rica,
- timeline básica de síntomas,
- mejorar heurísticas ETL,
- completar campos `EMAIL` y `GITLAB` en `meta.rs`.

### Mediano plazo
- perfiles WPR más específicos,
- resumen más granular por proveedor/evento,
- correlación con intervalos de tiempo del síntoma.

### Largo plazo
- análisis ETW más profundo,
- mayor soporte a símbolos,
- packaging más corporativo,
- firma digital y pipeline de liberación formal.
