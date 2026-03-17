# Plan Maestro de Desarrollo — RootCause Windows Inspector

**Creado:** 2026-03-17
**Versión de referencia:** v0.6.0
**Estado CI en creación:** ❌ 2 errores clippy bloqueantes
**Propósito:** documento de continuidad — contiene TODO el contexto del producto, análisis técnico completo, prioridades ordenadas y visión de expansión. Diseñado para retomar el trabajo en cualquier sesión sin perder contexto.

---

## 1. Qué es el producto

**RootCause Windows Inspector** es un monitor forense ligero para Windows escrito en Rust + egui. Su propósito es mostrar con claridad cuál proceso, servicio o conexión es la causa raíz de la lentitud del sistema — sin convertirse en otra herramienta pesada.

### Filosofía del producto

- **Diagnóstico primero, intervención después** — la UI guía, no actúa sola
- **Cero telemetría** — sin red saliente, sin analytics, sin licencias en línea, auditable en Cargo.toml
- **Sin dependencias en runtime** — el `.exe` funciona solo, sin instalar .NET ni runtimes adicionales
- **Ligero por diseño** — cada crate agregado debe justificarse; la app no puede ser otra carga para el sistema que monitorea

### Repos del proyecto

| Repo | Visibilidad | Propósito |
|---|---|---|
| `rootcause-windows-inspector` | **Privado** | Código fuente completo |
| `rootcause-landing` | **Público** | Landing page + binarios de descarga |

**Landing pública:** `https://vladimiracunadev-create.github.io/rootcause-landing/`

---

## 2. Estado técnico actual (v0.6.0)

### Métricas del código

| Métrica | Valor |
|---|---|
| Líneas de código totales | ~6,368 LOC |
| Archivo más grande | `src/app.rs` — 2,973 líneas |
| Dependencias directas | 11 crates |
| Binario estimado (release) | ~18–20 MB |
| Perfil build | `opt-level=3` + `lto=true` + `strip=true` + `panic=abort` |
| Tests unitarios | 1 (en `inspector.rs`) |
| Cobertura CI | fmt + clippy + test + build |

### Arquitectura de módulos

```
src/
├── main.rs              — entrada: detecta args CLI → cli::run() o GUI
├── meta.rs              — constantes: VERSION, AUTHOR, GITHUB, GITLAB, EMAIL, LICENSE
├── cli.rs               — CLI completa: --help, status, snapshot, history, export, wpr, kill, block-ip, stop-service
├── app.rs               — UI: 8 tabs, sparklines, atajos de teclado, hardware info, tab Acerca
├── models.rs            — structs: SystemSnapshot, ProcessInsight, SnapshotRow, HardwareInfo, etc.
└── services/
    ├── inspector.rs     — orquestador: métricas, deltas I/O, clasificación, get_hardware_info()
    ├── persistence.rs   — SQLite: persist_snapshot(), load_recent()
    ├── windows.rs       — PowerShell, WPR, netstat, taskkill, firewall, toasts, cmdlines
    ├── network.rs       — parsea netstat, clasifica conexiones por severidad
    ├── temp_scan.rs     — escanea %TEMP%, SoftwareDistribution, DeliveryOptimization
    └── etl.rs           — analiza dumpfile.xml de tracerpt, genera trace-analysis.json
```

### Tabs de la UI

| Tab | Ícono | Qué hace |
|---|---|---|
| Resumen | ◈ | Semáforo global, sparklines CPU/RAM/IO, alertas, características del equipo |
| Procesos | ⚙ | Tabla con severidad heurística, filtro, cmdline para críticos, finalizar proceso |
| Conexiones | ◎ | Netstat enriquecido, filtro IPs públicas, bloquear IP |
| Temporales | ▤ | Carpetas temp por tamaño, sugerencia limpieza |
| ETW / WPR | ◉ | Iniciar/detener/analizar captura ETL desde UI |
| Servicios | ◧ | Estado de BITS, WUpdate, SysMain, DoSvc — detener desde UI |
| Historial | ◑ | Últimas 60 capturas SQLite, comparación A vs B |
| Acerca | ℹ | Versión, autor, links, atajos de teclado, tech stack |

### Comandos CLI disponibles

```
rootcause --help                          Ayuda completa
rootcause --version                       Versión del producto
rootcause status                          Estado actual (severidad, CPU, RAM, I/O, alertas)
rootcause snapshot                        Captura completa en JSON (stdout)
rootcause history [N]                     Últimas N capturas del historial (default 10)
rootcause export                          Exporta snapshot a JSON en Descargas/Documentos
rootcause wpr start [--note NOTA]         Inicia captura ETW con WPR
rootcause wpr stop  [--note NOTA]         Detiene la captura y guarda ETL
rootcause wpr cancel                      Cancela captura activa
rootcause wpr analyze                     Resume último ETL con tracerpt
rootcause kill <PID>                      Finaliza proceso (respeta política de protección)
rootcause block-ip <IP>                   Bloquea IP vía firewall de Windows
rootcause stop-service <nombre>           Detiene servicio permitido (bits, wuauserv, etc.)
```

### Atajos de teclado

| Atajo | Acción |
|---|---|
| `F5` | Refrescar datos ahora |
| `Ctrl+E` | Exportar snapshot a JSON |
| `Ctrl+1` | Ir a tab Resumen |
| `Ctrl+2` | Ir a tab Procesos |
| `Ctrl+3` | Ir a tab Conexiones |
| `Ctrl+4` | Ir a tab Temporales |
| `Ctrl+5` | Ir a tab ETW/WPR |
| `Ctrl+6` | Ir a tab Servicios |
| `Ctrl+7` | Ir a tab Historial |
| `Ctrl+8` | Ir a tab Acerca |

---

## 3. Análisis técnico — fortalezas y deudas

### Fortalezas reales del código

- **Seguridad en acciones privilegiadas**: `is_valid_firewall_ip()` valida formato IP estrictamente antes de insertar en scripts PowerShell. Allowlist de servicios con doble validación. No hay vectores de inyección identificados.
- **Sin telemetría verificable**: 0 crates HTTP, 0 sockets salientes, 0 URLs hardcodeadas a servidores externos.
- **Clasificación inteligente de procesos**: scoring ponderado (CPU +35, RAM +28, I/O +40, ruta temporal +24, patrón instalador +12). Umbrales: Healthy 0–24, Warning 25–54, Critical 55+.
- **Binario autónomo**: `rusqlite bundled` incluye SQLite compilado. Sin `.dll` externas necesarias.
- **Perfil de release óptimo**: LTO + strip + codegen-units=1 + panic=abort → binario mínimo.
- **Graceful degradation**: fallos en métricas individuales no crashean la app; se reportan como alertas.

### Deudas técnicas conocidas

| Deuda | Archivo | Severidad | Descripción |
|---|---|---|---|
| `collapsible_if` | `app.rs:402` | 🔴 BLOQUEANTE CI | `if let` anidado debe colapsarse con let-chain |
| `print_literal` | `cli.rs:218` | 🔴 BLOQUEANTE CI | Literal en argumento de format!, debe ir en la cadena |
| `is_public_ip()` duplicada | `network.rs` + `etl.rs` | 🟡 Media | Misma función en dos lugares; riesgo de divergencia futura |
| Umbrales mágicos | `inspector.rs` | 🟡 Media | `65.0`, `2500.0`, `200.0` sin nombre de constante |
| `.expect("regex válida")` | `etl.rs:372` | 🟡 Media | Único punto de panic potencial; reemplazar con `lazy_static` |
| `app.rs` monolítico | `app.rs` | 🟢 Baja | 2,973 líneas; split en submódulos mejoraría mantenimiento |
| SQLite sin retención | `persistence.rs` | 🟢 Baja | Crece indefinidamente; agregar limpieza de rows antiguas |
| EMAIL vacío | `meta.rs:20` | 🟠 Alta | TODO pendiente — se muestra en tab Acerca y CLI --help |
| GITLAB sin confirmar | `meta.rs:27` | 🟠 Alta | URL puede ser incorrecta; verificar antes de distribución |
| Baselines I/O no se limpian | `inspector.rs` | 🟢 Muy baja | HashMap de PIDs nunca elimina entradas viejas (trivial) |

### Análisis de seguridad — veredicto

El código **no se presta para comportamiento de malware** por las siguientes razones verificables:
1. No hay crates de red (sin `reqwest`, `hyper`, `ureq`, ni similares en `Cargo.toml`)
2. No hay sockets TCP/UDP salientes en el código
3. No hay URLs de servidores externos hardcodeadas
4. Las únicas llamadas de red son lecturas locales del sistema (netstat, WMI)
5. Todo se almacena localmente en SQLite y JSON
6. El código es auditable en su totalidad

**Riesgos residuales de bajo nivel** (todos mitigados):
- PowerShell con `-ExecutionPolicy Bypass`: necesario para compatibilidad; los scripts son generados internamente, no cargados de disco
- cmdline de procesos: limitado a top 6, no se envía a ningún lugar
- XML de ETL no validado con schema: mitigado con `.trim()` y límite de caracteres

---

## 4. Comparación con software similar

### Posicionamiento en el mercado

| Herramienta | Diagnóstico automático | CLI | ETL desde UI | Historial | Sin telemetría | Open source |
|---|---|---|---|---|---|---|
| **RootCause** | ✅ Scoring heurístico | ✅ Completa | ✅ Nativo | ✅ SQLite | ✅ Cero | ✅ Apache 2.0 |
| Process Monitor (Sysinternals) | ❌ Solo datos | ❌ | ❌ | ❌ | ✅ | ❌ Freeware |
| Process Explorer (Sysinternals) | ❌ Solo datos | ❌ | ❌ | ❌ | ✅ | ❌ Freeware |
| Task Manager (Windows) | ❌ Básico | ❌ | ❌ | ❌ | Parcial | ❌ |
| PC Manager (Microsoft) | Parcial | ❌ | ❌ | ❌ | ❌ Tiene telemetría | ❌ |
| GlassWire | Solo red | ❌ | ❌ | Parcial | ❌ Freemium | ❌ |
| Resource Monitor | ❌ Solo datos | ❌ | ❌ | ❌ | ✅ | ❌ |

### Ventajas reales vs competencia

1. **Interpretación automática** — Process Monitor muestra todo; RootCause dice qué es relevante y por qué
2. **CLI nativa** — ningún competidor gratuito tiene `rootcause status` ni `rootcause block-ip`
3. **ETL integrado** — iniciar/detener/analizar WPR sin abrir cmd es único en este segmento
4. **Comparación histórica A/B** — ninguna herramienta gratuita persiste y compara snapshots
5. **Auditable y sin telemetría** — diferenciador fuerte para usuarios técnicos y empresas

### Desventajas honestas

1. **No es un event tracer** — Process Monitor captura eventos individuales en tiempo real; RootCause trabaja con snapshots cada N segundos
2. **Tamaño del binario** — 18 MB vs 2.5 MB de ProcMon (overhead de egui inevitable)
3. **Sin análisis de registro** — no monitorea HKEY_RUN ni autostart (pendiente en roadmap)
4. **Sin firma digital** — activa SmartScreen en primera ejecución

---

## 5. Composición del binario y opciones de reducción

### De dónde viene el peso (~18–20 MB)

```
Tu código Rust (6,368 LOC)        →   ~0.8 MB   (4% del total)
egui + eframe (widgets + ventana)  →  ~10.0 MB  (53% del total)
Fuentes tipográficas (en egui)     →   ~3.5 MB  (18% del total)
rusqlite (SQLite compilado)        →   ~1.0 MB   (5% del total)
sysinfo (métricas de sistema)      →   ~0.5 MB   (3% del total)
serde + serde_json                 →   ~0.4 MB   (2% del total)
resto de dependencias              →   ~0.8 MB   (4% del total)
────────────────────────────────────────────────
Total estimado                     →  ~17 MB
```

### Opciones de reducción reales

| Acción | Ahorro | Riesgo | Estado |
|---|---|---|---|
| Eliminar `rusqlite` → migrar a JSON | ~1 MB | Bajo — pierde SQL queries nativas | **Pendiente decisión del usuario** |
| Eliminar `regex` → parsing manual | ~0.3 MB | Bajo — etl.rs ya tiene helpers | Factible |
| Quitar `egui`/`eframe` | ~13 MB | **Destruye la interfaz gráfica** | ❌ Jamás |

**Conclusión**: con acciones seguras se puede llegar a ~17 MB. El peso real es `egui` — el precio inevitable de tener GUI nativa en Rust.

**Solución alternativa**: crear una segunda edición **CLI-only** que pesa ~3–5 MB y no lleva ninguna GUI.

---

## 6. Ediciones del producto — visión completa

### Ediciones planificadas

| Edición | Binario | Peso estimado | Audiencia |
|---|---|---|---|
| **GUI completa** (actual) | `rootcause.exe` | ~18 MB | Usuarios de escritorio |
| **Instalador** (ya existe) | `rootcause-setup.exe` | ~19 MB | Instalación con PATH automático |
| **CLI-only** (v0.7) | `rootcause-cli.exe` | ~3–5 MB | Sysadmins, scripts, automatización |
| **Módulo PowerShell** (v0.7) | `RootCause.psm1` | ~5 KB | Integración en scripts PS existentes |
| **Tray icon** (v0.8) | `rootcause-tray.exe` | ~18 MB | Monitor silencioso permanente |

### Cómo se implementa CLI-only (feature flags Rust)

```toml
# Cargo.toml
[features]
default = ["gui"]
gui = ["eframe", "egui"]

[dependencies]
eframe = { version = "0.27", optional = true }
egui   = { version = "0.27", optional = true }
```

```
cargo build --release                        # GUI completa
cargo build --release --no-default-features  # CLI pura (~4 MB)
```

### Distribución futura

| Canal | Comando | Audiencia |
|---|---|---|
| **Winget** | `winget install rootcause` | Windows 10/11 usuarios técnicos |
| **Scoop** | `scoop install rootcause` | Desarrolladores |
| **Chocolatey** | `choco install rootcause` | Sysadmins enterprise |
| **GitHub Releases** | Descarga directa | Todos |

---

## 7. Plan de prioridades — 8 fases ordenadas

---

### 🚨 FASE 0 — Bloqueante inmediato

**CI está roto. Nada más avanza hasta resolver esto.**

| # | Tarea | Archivo | Acción exacta |
|---|---|---|---|
| 0.1 | Fix clippy `collapsible_if` | `src/app.rs:402` | Cambiar `if let Some(idx) = tab_switch { if let Some(&(tab,_,_)) = Tab::ALL.get(idx) { ... } }` por `if let Some(idx) = tab_switch && let Some(&(tab,_,_)) = Tab::ALL.get(idx) { ... }` |
| 0.2 | Fix clippy `print_literal` | `src/cli.rs:218` | Mover el literal `"Proceso dominante"` dentro del string de formato: `"... Proceso dominante"` — quitar del argumento |
| 0.3 | `run_fmt.ps1` | raíz | `powershell.exe -ExecutionPolicy Bypass -File run_fmt.ps1` |
| 0.4 | Push a master | GitHub | CI debe pasar verde antes de continuar |

---

### 🔧 FASE 1 — Completar v0.6 (código pendiente)

`get_hardware_info()` en inspector.rs ya fue implementado. Falta conectarlo a la UI.

| # | Tarea | Archivo | Detalle |
|---|---|---|---|
| 1.1 | Campo `hardware_info: HardwareInfo` en struct | `src/app.rs` | Agregar al struct `RootCauseApp` |
| 1.2 | Poblar `hardware_info` en `new()` | `src/app.rs` | Si inspector OK: `hardware_info: insp.get_hardware_info()`. Si falla: `HardwareInfo::default()` |
| 1.3 | Sección hardware en tab Overview | `src/app.rs` | Mostrar: OS, versión, hostname, CPU marca+núcleos+MHz, RAM total, arquitectura |
| 1.4 | Atajos de teclado en `update()` | `src/app.rs` | Patrón collect-then-execute: `let mut should_refresh = false; ctx.input(|i| { if i.key_pressed(Key::F5) { should_refresh = true; } }); if should_refresh { self.refresh_now(); }` |
| 1.5 | Sección atajos en `draw_tab_about()` | `src/app.rs` | Tabla visual con todos los atajos |
| 1.6 | `run_fmt.ps1` + verificar clippy | — | Sin warnings nuevos |
| 1.7 | Commit y push | master | `feat: completar v0.6 — hardware info, atajos de teclado, tab Acerca` |
| 1.8 | Reubicar tag v0.6.0 | GitHub | `git tag -d v0.6.0 && git tag -a v0.6.0 -m "v0.6.0 completo" && git push origin v0.6.0 --force` |

---

### 🌐 FASE 2 — Landing page y metadatos del producto

| # | Tarea | Archivo | Detalle |
|---|---|---|---|
| 2.1 | "Sin telemetría activa" → "Telemetría: cero" | `rootcause-landing/index.html` | En badge hero, feature card, footer, og:description y meta description |
| 2.2 | Completar `EMAIL` | `src/meta.rs:20` | Pendiente que el usuario confirme su email de contacto |
| 2.3 | Verificar URL GITLAB | `src/meta.rs:27` | Confirmar `https://gitlab.com/vladimiracunadev-create` o corregir |
| 2.4 | Push landing | `rootcause-landing` rama `main` | GitHub Pages redespliegue automático en ~60s |

---

### 🧠 FASE 3 — Skills y documentación completa

| # | Tarea | Detalle |
|---|---|---|
| 3.1 | Crear skill `rootcause-rename` | Skill que al cambiar el nombre del producto actualiza: todas las ocurrencias en `src/`, todos los `.md` de `docs/`, `Cargo.toml`, `index.html` de la landing, nombre del binario en CI, y los propios skills. Debe hacer un barrido con grep, listar todos los archivos afectados y preguntar confirmación antes de actuar. |
| 3.2 | Actualizar `ARCHITECTURE.md` | Añadir `meta.rs` y `cli.rs` en la tabla de módulos. Actualizar responsabilidades de `inspector.rs` con `get_hardware_info()`. Actualizar `models.rs` con `HardwareInfo`. |
| 3.3 | Actualizar `OPERACION.md` | Agregar sección "Uso desde consola (CLI)" con tabla de comandos y ejemplos. Añadir los atajos de teclado. |
| 3.4 | Crear o actualizar `COMMANDS.md` | Documento completo de todos los comandos CLI con ejemplos y códigos de salida. |
| 3.5 | Actualizar `ROADMAP.md` | Marcar v0.6 como 100% completado. Agregar columna v0.7 con las features planificadas. |
| 3.6 | Actualizar `RECLUTADORES.md` | Añadir CLI completa, atajos de teclado, hardware info y landing page como capacidades demostrables. |
| 3.7 | Actualizar `INDEX.md` | Enlazar `COMMANDS.md` y este `PLAN_MAESTRO.md`. |
| 3.8 | Push `rootcause-windows-inspector` | `docs: barrido v0.6 completo — skills, CLI, hardware, atajos` |
| 3.9 | Actualizar skill `rootcause-improve` SKILL.md | Confirmar que incluye la landing, las tres ediciones planificadas y el skill rootcause-rename en tabla de skills relacionados. |

---

### ⚙️ FASE 4 — Calidad de código (deuda técnica)

| # | Tarea | Archivo | Impacto | Decisión previa requerida |
|---|---|---|---|---|
| 4.1 | Consolidar `is_public_ip()` | `network.rs` + `etl.rs` | Eliminar duplicación — mover a `network.rs`, importar en `etl.rs` | No |
| 4.2 | Constantes para umbrales | `inspector.rs` | `const CPU_HIGH_PCT: f32 = 65.0` etc. al inicio del archivo | No |
| 4.3 | Fix `.expect()` con `OnceLock` o `lazy_static` | `etl.rs:372` | Eliminar único panic potencial del código | No |
| 4.4 | **Migrar SQLite → JSON** | `persistence.rs` | Ahorra ~1 MB. Eliminar `rusqlite` de `Cargo.toml`. Trade-off: la comparación A/B debe reimplementarse cargando el JSON completo | **PENDIENTE: usuario debe confirmar si acepta perder queries SQL nativas** |
| 4.5 | Retención SQLite (si se mantiene) | `persistence.rs` | `DELETE FROM snapshots WHERE id NOT IN (SELECT id FROM snapshots ORDER BY collected_at DESC LIMIT 1000)` en `persist_snapshot()` | Solo si se mantiene SQLite |
| 4.6 | Dividir `app.rs` en submódulos | `src/app/` | `overview.rs`, `processes.rs`, `connections.rs`, `precision.rs`, `services.rs`, `history.rs`, `about.rs`. No afecta binario ni funcionalidad. | No |

---

### 🚀 FASE 5 — Tres ediciones del producto (v0.7)

| # | Edición | Implementación | Esfuerzo |
|---|---|---|---|
| 5.1 | **CLI-only binary** | Feature flags en `Cargo.toml`. `eframe`/`egui` como `optional`. Build con `--no-default-features`. Actualizar `main.rs` con `#[cfg(feature = "gui")]`. Resultado: ~3–5 MB. | Bajo — 90% ya existe |
| 5.2 | **Módulo PowerShell** | Archivo `RootCause.psm1` que llama a `rootcause.exe` y convierte JSON output en objetos PS. Comandos: `Get-RootCauseStatus`, `Get-RootCauseProcesses`, `Invoke-RootCauseExport`. **Cero cambios en Rust.** | Bajo |
| 5.3 | **Tray icon** | Ícono en bandeja del sistema. Cambia color según severidad (verde/amarillo/rojo). Click abre ventana principal. Requiere actualizar `eframe` a 0.28+ que incluye tray support. | Medio |

---

### 📦 FASE 6 — Distribución pública (v0.7 → v1.0)

| # | Tarea | Descripción | Esfuerzo |
|---|---|---|---|
| 6.1 | **Auto-publish releases a landing** | En `release-windows.yml`: después de build, usar `gh release create --repo vladimiracunadev-create/rootcause-landing` para publicar binarios en repo público. Requiere crear secret `LANDING_RELEASE_TOKEN` (PAT con permisos al repo landing). Los botones de descarga de la landing ya apuntan a `rootcause-landing/releases/latest`. | Medio |
| 6.2 | **Scoop bucket** | Crear `bucket/rootcause.json` en repo público con hash SHA256, URL de descarga y comando de instalación. `scoop install rootcause`. | Bajo |
| 6.3 | **Winget manifest** | Crear YAML de manifiesto para el repositorio `winget-pkgs`. `winget install rootcause`. | Bajo |
| 6.4 | **Firma digital del binario** | Self-signed certificate o CodeSigning cert comercial. Elimina alerta SmartScreen en primera ejecución. | Medio |
| 6.5 | **Chocolatey** | `.nuspec` + script PowerShell de instalación. `choco install rootcause`. | Bajo |

---

### 🔬 FASE 7 — Features de producto nuevas (v0.7 → v1.0)

| # | Feature | Descripción | Prioridad | Cero crates nuevos |
|---|---|---|---|---|
| 7.1 | **Tab Autostart** | Leer HKEY_CURRENT_USER\SOFTWARE\Microsoft\Windows\CurrentVersion\Run, carpeta Startup, tareas programadas via PowerShell. Mostrar qué arranca con Windows y su ruta. Mayor diferenciador vs Sysinternals. | Alta | ✅ PowerShell + winreg (ya existe windows-rs) |
| 7.2 | **Alertas configurables** | Usuario puede ajustar umbrales de CPU/RAM/IO desde un panel de configuración. Guardar en `rootcause.toml` en AppData. | Media | ✅ toml (o serde_json) |
| 7.3 | **`--output` en CLI** | `rootcause snapshot --output diag.json` además de stdout. | Baja | ✅ |
| 7.4 | **Tests unitarios** | Para `classify_process()`, `parse_netstat_output()`, `is_public_ip()`, `is_valid_firewall_ip()`. Actualmente solo 1 test. | Media | ✅ |
| 7.5 | **Archivo de configuración** | `rootcause.toml` para: intervalo de refresco, umbrales de alerta, retención SQLite, notificaciones on/off. | Baja | ✅ |

---

### 🏗️ FASE 8 — Largo plazo (v2.0+)

| # | Versión | Descripción | Complejidad |
|---|---|---|---|
| 8.1 | **Windows Service** | `rootcause-service.exe` corre sin usuario logueado. Recopila datos 24/7. La GUI se conecta via named pipes o socket local. Permite diagnosticar problemas nocturnos y tener historial continuo. | Muy alta |
| 8.2 | **VS Code Extension** | Barra de estado en VS Code con estado del sistema en tiempo real. Click abre panel de alertas. TypeScript wrapper que llama a `rootcause status --json`. | Media |
| 8.3 | **Edición Seguridad** | Versión stripped: solo procesos con rutas sospechosas, conexiones a IPs públicas, servicios anómalos. UI orientada a SOC / respuesta a incidentes. Feature flags en Cargo.toml. | Media |
| 8.4 | **Edición Enterprise** | Exportación Prometheus/Grafana, gestión multi-equipo, configuración via GPO, reportes CSV/Excel. Modelo de negocio B2B. | Muy alta |
| 8.5 | **MSIX / Microsoft Store** | Empaquetado MSIX para distribución en Microsoft Store. Requiere cuenta de desarrollador y firma digital. | Alta |

---

## 8. Mapa visual de fases

```
HOY                CORTO PLAZO         MEDIANO PLAZO       LARGO PLAZO
(v0.6)             (v0.7)              (v1.0)              (v2.0+)
───────────────    ─────────────────   ─────────────────   ──────────────────
FASE 0             FASE 4              FASE 6              FASE 8
Fix CI             Deuda técnica       Distribución        Windows Service
                   (código limpio)     Scoop/Winget        VS Code Extension
FASE 1             FASE 5              Firma digital       Enterprise
Completar v0.6     CLI-only binary                         Security Edition
                   PowerShell mod      FASE 7              MSIX Store
FASE 2             Tray icon           Tab Autostart
Landing fix                            Alertas config
                   FASE 3              Tests unitarios
FASE 3             (si no se hizo      Config file
Docs + Skills      antes)
```

---

## 9. Reglas de trabajo para cada sesión

### Antes de empezar cualquier tarea

```
1. git log --oneline -3           → ¿en qué commit estamos?
2. git status                     → ¿hay cambios sin commitear?
3. Verificar CI en GitHub Actions → ¿verde o rojo?
4. Leer src/meta.rs               → ¿EMAIL y GITLAB completos?
```

### Flujo de trabajo obligatorio para cambios Rust

```
1. Leer el archivo con Read tool antes de editar
2. Editar con Edit tool (nunca reescribir archivos completos salvo necesidad)
3. powershell.exe -ExecutionPolicy Bypass -File run_fmt.ps1
4. Verificar clippy (ver CI o run_check.ps1)
5. git add <archivos específicos> (nunca git add -A)
6. git commit con mensaje en español + Co-Authored-By
7. git push origin master
8. Si es release: tag + push tag + actualizar landing
```

### Notas de entorno local (CRÍTICO)

- **Conflicto MSVC/MSYS2**: bash en este equipo usa `link.exe` de MSYS2 en vez de MSVC. Cualquier comando `cargo` que compile (check, build, clippy) puede fallar con error de linker. **Solución**: siempre usar `powershell.exe -ExecutionPolicy Bypass -File run_fmt.ps1` para fmt. Para check/clippy usar `run_check.ps1` con paths explícitos al toolchain MSVC.
- **CI en GitHub Actions** usa `windows-latest` con toolchain correcto → compilación siempre funciona en CI aunque falle local.
- **Ruta del proyecto**: `C:\dev\rootcause-windows-inspector`
- **Ruta de la landing**: `C:\dev\rootcause-landing`
- **Rama principal**: `master` (producto), `main` (landing)
- **Toolchain Rust**: `C:\Users\vbav\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin\`

### Convenciones de commits

| Prefijo | Cuándo |
|---|---|
| `feat:` | Nueva funcionalidad |
| `fix:` | Corrección de bug |
| `style:` | Solo cargo fmt |
| `docs:` | Solo documentación |
| `refactor:` | Reestructuración sin cambio funcional |
| `test:` | Tests |
| `chore:` | Bump versión, CI, tareas de mantenimiento |

Siempre en **español**. Siempre incluir:
```
Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
```

---

## 10. Checklist de release (resumen rápido)

```
☐ Bump versión en Cargo.toml
☐ Actualizar badge de versión en README.md
☐ Actualizar ROADMAP.md (marcar ítems ✅)
☐ Actualizar ARCHITECTURE.md si hay módulos nuevos
☐ Actualizar OPERACION.md si hay acciones nuevas para el usuario
☐ Actualizar COMMANDS.md si hay comandos CLI nuevos
☐ Actualizar RECLUTADORES.md con features nuevas
☐ Actualizar INDEX.md si hay docs nuevos
☐ Actualizar rootcause-landing/index.html (versión + features)
☐ run_fmt.ps1 → sin errores
☐ CI verde (fmt + clippy + test + build)
☐ git tag -a vX.Y.Z
☐ git push origin master && git push origin vX.Y.Z
☐ Verificar que GitHub Actions genera ZIP + Setup.exe + SHA256SUMS
☐ Publicar binarios en rootcause-landing/releases (manual o automático)
☐ Verificar landing page actualizada en browser
```

---

*Este documento es el punto de partida de cada sesión de trabajo. Si algo cambia, actualizar la sección correspondiente y hacer commit.*
