# Arquitectura Evolutiva de RootCause

Documento técnico en español para orientar la evolución de RootCause sin tratarlo como greenfield.

## 1. Estado real hoy

### Stack confirmado
- Rust 2024
- GUI nativa con `eframe/egui`
- Persistencia local con SQLite (`rusqlite`)
- Recolección local con `sysinfo`
- Integración Windows vía PowerShell, `netstat`, `taskkill`, `wpr`, `tracerpt`
- Extensión VS Code en TypeScript
- Módulo PowerShell para automatización

### Flujo real actual
1. `main.rs` despacha a GUI o CLI.
2. `InspectorService` recolecta procesos, red, temporales, eventos y estado WPR.
3. Las reglas locales clasifican severidad y generan alertas.
4. El snapshot se persiste en SQLite.
5. Cuando hay ETL, se resume con `tracerpt` + heurísticas propias.
6. La UI, la CLI, la extensión VS Code y el módulo PowerShell consumen el mismo motor.

### Qué conviene conservar
- Un solo motor compartido entre GUI y CLI.
- Dependencia mínima del sistema operativo usando herramientas nativas.
- SQLite como almacenamiento liviano y auditable.
- WPR/ETW como capa opcional de precisión, no obligatoria.
- Exportes JSON como evidencia portable.

## 2. Debilidades detectadas en la auditoría

### Diseño
- `app.rs` sigue siendo demasiado grande y concentra mucha UI.
- `InspectorService` estaba mezclando colección, reglas y persistencia.
- Los umbrales de severidad estaban hardcodeados.
- La correlación existía, pero implícita y dispersa.

### Producto
- La extensión VS Code y el módulo PowerShell esperaban `status --json` / `history --json`, pero el binario no los ofrecía.
- Había historial de snapshots, pero no incidentes normalizados.
- Había acciones manuales protegidas, pero sin auditoría persistente.
- No existía un punto claro para integrar IA opcional sin contaminar el motor principal.

### Riesgo técnico
- Duplicación de heurísticas de IP pública entre `network.rs` y `etl.rs`.
- Baselines de I/O podían crecer con PIDs muertos.
- Configuración ausente para adaptar umbrales a equipos distintos.

## 3. Arquitectura objetivo evolutiva

No se propone reescritura. Se propone separar responsabilidades usando el mismo stack.

### Componentes actuales y evolución
- `collector / agent`
  - Hoy: `InspectorService` + `sysinfo` + `windows.rs`
  - Evolución: mantenerlo y evitar meter reglas de negocio dentro del recolector

- `rule engine`
  - Hoy: heurísticas embebidas
  - Evolución: `services/rules.rs` centraliza clasificación de procesos, alertas e incidentes

- `correlator`
  - Hoy: correlación básica entre procesos, temporales, red y ETL
  - Evolución: `rules.rs` genera `IncidentSummary` con causas probables, evidencia y acciones sugeridas

- `evidence store`
  - Hoy: snapshots + export JSON + artefactos ETL
  - Evolución: SQLite guarda snapshots, incidentes resumidos y auditoría de acciones

- `notifier`
  - Hoy: toast notification en GUI
  - Evolución: mantiene toast como canal local; otros canales pueden agregarse después sin mover el core

- `remediation executor`
  - Hoy: `kill`, `block-ip`, `stop-service`
  - Evolución: siguen siendo manuales, con allowlist y auditoría; automatización futura debe seguir deshabilitada por defecto

- `config manager`
  - Hoy: no existía
  - Evolución: `src/config.rs` carga `rootcause-config.json` sin dependencias nuevas

- `audit trail`
  - Hoy: no existía
  - Evolución: tabla `audit_log` en SQLite para acciones manuales y llamadas de IA

- `optional AI adapter`
  - Hoy: inexistente
  - Evolución: `services/ai.rs` es opcional, explícito y desacoplado; opera sobre un incidente ya resumido

## 4. Decisiones de implementación introducidas

### Módulos nuevos
- `src/config.rs`
  - Configuración operativa con defaults seguros
- `src/services/rules.rs`
  - Clasificación, alertas e incidentes
- `src/services/ai.rs`
  - Adaptador IA compatible con endpoint estilo OpenAI, apagado por defecto

### Persistencia ampliada
- `snapshots`
  - tendencia y comparación histórica
- `incidents`
  - incidentes resumidos, deduplicados por fingerprint inmediato
- `audit_log`
  - registro de acciones y enriquecimientos IA

### CLI reforzada
- `rootcause status --json`
- `rootcause history [N] --json`
- `rootcause incidents [N] --json`
- `rootcause config show [--json]`
- `rootcause config init`
- `rootcause ai explain-latest [--json]`
- `rootcause snapshot --output <ruta>`

## 5. Backlog evolutivo por fases

### Fase 1
- cerrar contratos JSON del CLI
- introducir config manager
- centralizar reglas
- persistir incidentes y auditoría

### Fase 2
- dividir `app.rs` por tabs o widgets
- incorporar vista UI de incidentes y auditoría
- enriquecer configuración desde la propia interfaz

### Fase 3
- correlación temporal con baseline por ventana
- reglas específicas para deploy reciente, colas y degradación de servicios
- severidad basada en tendencia, no solo en muestra instantánea

### Fase 4
- remediación segura opt-in por política
- catálogo de acciones permitidas con simulación previa
- más trazabilidad en auditoría

### Fase 5
- IA opcional por API
- ejecución explícita o por política opt-in
- límites de coste, timeout y fallback registrados

## 6. Cómo ejecutar y probar

### Ejecución
```powershell
.\scripts\verify-environment.ps1
cargo build --release
.\target\release\rootcause.exe
```

### Validación local
```powershell
.\run_fmt.ps1
.\run_check.ps1
```

Si `run_check.ps1` falla por `link.exe`, el problema es del entorno MSVC local, no del diseño funcional de RootCause. La CI de Windows sigue siendo la referencia para build formal.

## 7. IA opcional: activación y fallback

### Por defecto
- `ai.enabled = false`
- RootCause detecta, alerta, persiste y exporta sin IA

### Activación
1. Ejecutar `rootcause config init`
2. Editar `rootcause-config.json`
3. Definir:
   - `ai.enabled = true`
   - `ai.endpoint`
   - `ai.model`
   - `ai.api_key_env_var`
4. Exportar la variable de entorno de API key
5. Invocar `rootcause ai explain-latest`

### Fallback
- si la IA falla, el incidente sigue persistido
- la captura y las alertas siguen funcionando
- el fallo queda auditado

## 8. Qué no se hizo a propósito

- no se cambió Rust por otra tecnología
- no se añadió una base nueva
- no se introdujo un backend remoto obligatorio
- no se volvió obligatorio ETW
- no se volvió obligatoria la IA

La meta sigue siendo producto comercializable, bajo consumo y comportamiento predecible en Windows real.
