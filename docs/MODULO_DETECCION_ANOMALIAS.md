# Modulo de deteccion de comportamiento anomalo (V1)

## Proposito

Este modulo extiende RootCause con una primera capacidad local de deteccion de senales compatibles con actividad no autorizada o potencialmente maliciosa, manteniendo el enfoque del producto: observabilidad, diagnostico, correlacion de eventos, causa raiz, alertas y sugerencias de accion.

RootCause complementa observabilidad y diagnostico del endpoint, pudiendo detectar senales compatibles con actividad maliciosa o no autorizada, pero no reemplaza una solucion antivirus o EDR dedicada.

## Estado actual

- Estado: `V1 inicial implementada en el repositorio`
- Alcance actual: heuristicas locales, correlacion basica, scoring, evidencia, incidentes resumidos, persistencia JSON/SQLite, salida GUI y CLI.
- Alcance pendiente: baseline mas madura para otras senales, hashes opcionales, supervisores de larga duracion y hardening del propio agente.

## Que hace hoy

- Monitorea procesos activos y reutiliza la captura ya existente de CPU, RAM, I/O y conexiones.
- Enriqeuce la muestra con `command line` para procesos de mayor interes operativo o de riesgo.
- Observa puntos de persistencia basicos de Windows:
  - `HKCU/HKLM\...\Run`
  - `HKCU/HKLM\...\RunOnce`
  - carpeta `Startup` del usuario y global
  - tareas programadas no-Microsoft
- Mantiene una baseline conocida de autoarranque en SQLite (`persistence_baseline`). La primera foto siembra la baseline en silencio; a partir de ahi cada scan compara contra ella y clasifica cada entrada como NUEVA, MODIFICADA (cambio de `command`), ELIMINADA o sin cambios. Los cambios quedan pegajosos hasta que el usuario los acepta (UI o `rootcause autostart --accept`).
- Revisa servicios relevantes de operacion y seguridad:
  - `wuauserv`, `BITS`, `DoSvc`, `TrustedInstaller`, `SysMain`
  - `WinDefend`, `WdNisSvc`, `MpsSvc`, `wscsvc`, `Sense`
- Genera eventos de anomalia con:
  - severidad `Low`, `Medium`, `High`, `Critical`
  - score interno
  - hipotesis de causa raiz
  - evidencia resumida
  - recomendacion sugerida
- Persiste incidentes resumidos y deja la anomalia visible en:
  - `Overview` de la GUI
  - `rootcause status`
  - `rootcause incidents`
  - exportacion JSON del snapshot

## Heuristicas V1 implementadas

1. CPU sostenido anormal.
2. Crecimiento anomalo de memoria entre muestras.
3. Escritura agresiva en disco.
4. Multiples destinos salientes publicos en una misma ventana.
5. Ejecucion desde rutas sospechosas como `Temp`, `Downloads` o rutas de usuario no confiables.
6. Proceso fuera de linea confiable local.
7. Persistencia sospechosa en Run/RunOnce/Startup (`suspicious-persistence`, heuristica de "parece sospechoso ahora").
8. Reaparicion rapida de proceso.
9. Relacion padre-hijo sospechosa.
10. Ejecucion repetitiva de scripts o comandos.
11. Comandos compatibles con alteracion de seguridad local.
12. Patron de exploracion agresiva en red local.
13. Correlacion basica de multiples senales sobre el mismo proceso o contexto.
14. Cambio de autoarranque contra baseline conocida, emitido como `persistence-change` (severidad `High` para NUEVA/MODIFICADA, `Medium` para ELIMINADA). Coexiste con `suspicious-persistence`: uno compara contra la baseline, el otro evalua si la entrada parece sospechosa ahora.

## Flujo aplicado

1. `InspectorService` recopila procesos, red, temporales, servicios, eventos y persistencia (`persistence_entries()`).
2. `InspectorService::detect_persistence_changes()` compara la persistencia observada contra `persistence_baseline` y anota los cambios; `persistence_change_event()` en `anomaly.rs` crea el `AnomalyEvent` correspondiente.
3. `services/anomaly.rs` evalua reglas heuristicas locales con un estado incremental ligero.
4. Cada hallazgo genera un `AnomalyEvent`.
5. `services/rules.rs` traduce las anomalias a alertas visibles y a un `IncidentSummary`.
6. El snapshot conserva:
   - `anomalies`
   - `incident`
   - `persistence_entries`
7. La GUI muestra riesgo, hipotesis, evidencia y anomalias destacadas en `Overview`.
8. CLI y exportacion JSON exponen el mismo resultado sin depender de IA remota.

## Arquitectura aplicada

### Modulos principales

- `src/services/anomaly.rs`
  - motor heuristico local
  - correlacion basica
  - estado incremental de CPU, memoria, respawn y scripts
  - `persistence_change_event()` construye el `AnomalyEvent` de tipo `persistence-change`
- `src/services/inspector.rs`
  - orquestacion del snapshot
  - integracion con persistencia, UI y CLI
  - `detect_persistence_changes()` compara la persistencia observada contra la baseline y anota los cambios
- `src/services/windows.rs`
  - servicios relevantes
  - persistencia observable en Windows (`persistence_entries()`)
  - captura de `command line`
- `src/services/persistence.rs`
  - baseline de autoarranque en SQLite (`persistence_baseline`)
  - `load_persistence_baseline()` / `replace_persistence_baseline()`
- `src/services/rules.rs`
  - traduccion a alertas e incidentes
- `src/models.rs`
  - `RiskLevel`
  - `AnomalyEvent`
  - `PersistenceEntry`
  - campos adicionales en `IncidentSummary` y `SystemSnapshot`

### Decisiones de diseño

- Sin reescritura completa de arquitectura.
- Sin dependencias nuevas para un motor AV o analisis remoto obligatorio.
- Sin acciones destructivas automaticas por defecto.
- Con estado incremental acotado para no elevar demasiado el costo por muestra.

## Estructura de evento

Cada `AnomalyEvent` intenta incluir, cuando la captura lo permite:

- `detected_at`
- `severity`
- `score`
- `kind` (por ejemplo `correlated-anomaly`, `suspicious-persistence` o `persistence-change`)
- `title`
- `process_name`
- `pid`
- `parent_pid`
- `parent_name`
- `exe_path`
- `cpu_percent`
- `memory_mb`
- `io_write_mb_delta`
- `unique_public_remotes`
- `unique_private_remotes`
- `summary`
- `root_cause_hypothesis`
- `recommended_action`
- `evidence`

## Ejemplo resumido de salida

```json
{
  "kind": "correlated-anomaly",
  "title": "Correlacion de senales anomalas",
  "severity": "Critical",
  "score": 100,
  "process_name": "powershell.exe",
  "pid": 4242,
  "summary": "Se correlacionaron 3 senales en el mismo proceso/contexto.",
  "root_cause_hypothesis": "riesgo critical por combinacion de ruta de ejecucion sospechosa + trafico saliente + persistencia",
  "recommended_action": "Priorizar revision manual, preservar evidencia y considerar aislamiento de red o escaneo con antivirus/EDR si no corresponde al contexto."
}
```

Ejemplo de cambio de autoarranque contra baseline:

```json
{
  "kind": "persistence-change",
  "title": "Nueva entrada de autoarranque no vista en la baseline",
  "severity": "High",
  "summary": "Aparecio una entrada Run (HKCU) que no estaba en la baseline conocida.",
  "root_cause_hypothesis": "persistencia NUEVA respecto a la baseline sembrada en el primer scan",
  "recommended_action": "Revisar el origen de la entrada y aceptarla en la baseline si es legitima (UI o `rootcause autostart --accept`)."
}
```

## Configuracion base

La configuracion vive en `rootcause-config.json`, dentro del bloque `anomaly`.

Claves principales:

- `enabled`
- `cpu_sustained_percent`
- `cpu_sustained_samples`
- `memory_growth_mb`
- `memory_growth_samples`
- `aggressive_write_mb`
- `aggressive_write_samples`
- `public_destination_count`
- `local_scan_destination_count`
- `respawn_window_secs`
- `respawn_count`
- `repetitive_script_count`
- `suspicious_path_keywords`
- `trusted_process_names`
- `trusted_path_prefixes`
- `suspicious_parent_names`
- `security_service_names`
- `watch_persistence`

`watch_persistence` (bool, `true` por defecto) habilita la observacion de persistencia, la comparacion contra la baseline `persistence_baseline` y la emision de eventos `persistence-change`. Con `false` no se siembra ni compara baseline.

Ejemplo orientativo:

```json
{
  "anomaly": {
    "enabled": true,
    "cpu_sustained_percent": 65.0,
    "cpu_sustained_samples": 2,
    "memory_growth_mb": 120.0,
    "aggressive_write_mb": 45.0,
    "public_destination_count": 3,
    "local_scan_destination_count": 8,
    "respawn_window_secs": 120,
    "respawn_count": 2,
    "watch_persistence": true
  }
}
```

## Que no hace todavia

- No es un motor de firmas.
- No confirma malware por si solo.
- No hace sandbox, hooking o driver kernel.
- No cubre todos los mecanismos de persistencia de Windows.
- No hace forense profunda de memoria.
- No reemplaza consola centralizada tipo EDR.
- No depende de IA remota para funcionar.

## Limitaciones honestas

- Los falsos positivos son posibles en software legitimo con comportamiento intenso o atipico.
- La cobertura de persistencia V1 es deliberadamente acotada.
- La ausencia de hash en V1 evita costo extra, pero reduce capacidad de comparacion binaria.
- El score de riesgo es heuristico y sirve para priorizacion, no para certeza.
- Si un atacante obtiene privilegios altos, una herramienta local aislada puede ser alterada o desactivada.

## UX aplicada

En la GUI, el `Overview` muestra:

- titulo del incidente dominante
- riesgo y score
- hipotesis de causa
- proceso principal involucrado
- evidencia resumida
- recomendacion sugerida
- tarjetas cortas para las anomalias destacadas

En CLI:

- `rootcause status`
- `rootcause incidents`
- `rootcause config show`

## Seguridad y responsabilidad

- La recomendacion se mantiene separada de la accion.
- RootCause no ejecuta mitigaciones destructivas automaticas en esta V1.
- Si el contexto no es claro, la salida privilegia revisar, preservar evidencia y escanear con herramientas dedicadas.

## Roadmap sugerido

### V2

- baseline local mas madura por host
- cobertura ampliada de persistencia
- hash opcional de binarios relevantes
- mejores exclusiones y listas confiables
- mas contexto de usuario/sesion
- correlacion temporal mas precisa

### V3

- soporte mas consistente para ejecucion continua mediante tray/service
- telemetria local mas rica para respawn y watchdog
- mayor cobertura de sabotaje al agente
- exportes tecnicos mas orientados a respuesta a incidentes

## Referencias

- [README principal](../README.md)
- [REQ-SEC-001](requirements/REQ-SEC-001-deteccion-comportamiento-anomalo.md)
- [Registro de requerimientos](requirements/README.md)
- [Roadmap](ROADMAP.md)
