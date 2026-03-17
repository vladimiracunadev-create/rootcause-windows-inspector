# Plan Maestro — RootCause Windows Inspector

**Fecha:** 2026-03-17
**Versión de referencia:** v0.6.0
**Propósito:** documento de continuidad total. Recoge el análisis completo del producto, todas las decisiones tomadas, comparación con la competencia, análisis técnico profundo, visión de expansión y todas las prioridades ordenadas. Diseñado para retomar el trabajo en cualquier sesión sin perder ningún contexto.

> **Instrucción para Claude al iniciar sesión:** leer este documento antes de cualquier acción. Contiene todo lo que se ha discutido y decidido sobre el producto.

---

## Índice

1. [Qué es el producto y su filosofía](#1-qué-es-el-producto-y-su-filosofía)
2. [Estado técnico actual](#2-estado-técnico-actual-v060)
3. [Análisis del producto — fortalezas, debilidades y deudas](#3-análisis-del-producto)
4. [Comparación con software de la misma índole](#4-comparación-con-software-similar)
5. [Análisis del binario — tamaño y reducción](#5-análisis-del-binario)
6. [Telemetría — qué es y cuál es el estado real](#6-telemetría)
7. [Decisiones pendientes del usuario](#7-decisiones-pendientes-del-usuario)
8. [Ediciones del producto — visión completa](#8-ediciones-del-producto)
9. [Landing page y estrategia de distribución pública](#9-landing-page-y-distribución)
10. [Skills del proyecto](#10-skills-del-proyecto)
11. [Prioridades ordenadas — 8 fases](#11-prioridades-ordenadas)
12. [Reglas de trabajo por sesión](#12-reglas-de-trabajo-por-sesión)

---

## 1. Qué es el producto y su filosofía

**RootCause Windows Inspector** es un monitor forense ligero para Windows escrito en Rust + egui. Su propósito central es mostrar con claridad cuál proceso, servicio o conexión es la causa raíz de la lentitud del sistema — y hacerlo sin convertirse en otra herramienta pesada que contribuya al problema.

### Filosofía del producto

- **Diagnóstico primero, intervención después.** La UI guía al usuario hacia la causa; las acciones (matar proceso, bloquear IP, detener servicio) son una consecuencia, no el centro.
- **Cero telemetría.** Sin red saliente, sin analytics, sin licencias en línea. Todo corre localmente. Verificable en Cargo.toml: no existe ningún crate HTTP.
- **Sin dependencias en runtime.** El `.exe` funciona solo. Sin instalar .NET, sin DLLs adicionales, sin runtimes.
- **Ligero por diseño.** Cada crate agregado debe justificarse. La app no puede ser otra carga para el sistema que monitorea.
- **Auditable.** El código fuente es revisable por cualquier persona técnica. Ninguna parte del comportamiento está oculta.

### Repositorios del proyecto

| Repo | Visibilidad | Propósito | Rama |
|---|---|---|---|
| `rootcause-windows-inspector` | **Privado** | Código fuente completo | `master` |
| `rootcause-landing` | **Público** | Landing page + releases públicos | `main` |

**Landing pública:** `https://vladimiracunadev-create.github.io/rootcause-landing/`

---

## 2. Estado técnico actual (v0.6.0)

### Arquitectura de módulos

```
src/
├── main.rs              — entrada: detecta args → cli::run() o launch_gui()
├── meta.rs              — constantes: VERSION, AUTHOR, GITHUB, GITLAB, EMAIL, LICENSE
├── cli.rs               — CLI completa: --help, status, snapshot, history, export,
│                          wpr (start/stop/cancel/analyze), kill, block-ip, stop-service
├── app.rs               — UI completa: 8 tabs, sparklines, atajos, hardware info, Acerca
├── models.rs            — structs: SystemSnapshot, ProcessInsight, SnapshotRow,
│                          HardwareInfo, Alert, Severity, etc.
└── services/
    ├── inspector.rs     — orquestador: métricas, deltas I/O, get_hardware_info()
    ├── persistence.rs   — SQLite: persist_snapshot(), load_recent()
    ├── windows.rs       — PowerShell, WPR, netstat, taskkill, firewall, toasts
    ├── network.rs       — parsea netstat, clasifica IPs y conexiones
    ├── temp_scan.rs     — escanea %TEMP%, SoftwareDistribution, DeliveryOptimization
    └── etl.rs           — analiza dumpfile.xml de tracerpt, genera trace-analysis.json
```

### Tabs de la UI (v0.6)

| Tab | Ícono | Contenido | Acciones |
|---|---|---|---|
| Resumen | ◈ | Semáforo global, sparklines CPU/RAM/IO, alertas, hardware del equipo | — |
| Procesos | ⚙ | Tabla con scoring heurístico, filtro severidad, cmdline para críticos | Finalizar proceso |
| Conexiones | ◎ | Netstat enriquecido, filtro IPs públicas | Bloquear IP |
| Temporales | ▤ | Carpetas temp por tamaño y recuento | — |
| ETW / WPR | ◉ | Estado de captura, última traza, resumen ETL | Iniciar · Detener · Cancelar · Analizar |
| Servicios | ◧ | BITS, WUpdate, SysMain, DoSvc + eventos | Detener servicio |
| Historial | ◑ | Últimas 60 capturas SQLite | Comparar A vs B |
| Acerca | ℹ | Versión, autor, links, atajos, tech stack | — |

### Comandos CLI (v0.6)

```
rootcause --help                       Ayuda completa con ASCII art
rootcause --version                    Versión del producto
rootcause status                       Estado del sistema (severidad, CPU, RAM, I/O, alertas)
rootcause snapshot                     Captura completa en JSON (stdout)
rootcause history [N]                  Últimas N capturas del historial (default 10)
rootcause export                       Exporta snapshot a JSON en Descargas/Documentos
rootcause wpr start [--note NOTA]      Inicia captura ETW con WPR
rootcause wpr stop  [--note NOTA]      Detiene la captura y guarda el ETL
rootcause wpr cancel                   Cancela captura activa
rootcause wpr analyze                  Resume último ETL con tracerpt
rootcause kill <PID>                   Finaliza proceso (respeta política de protección)
rootcause block-ip <IP>               Bloquea IP vía firewall de Windows
rootcause stop-service <nombre>        Detiene servicio permitido (bits, wuauserv, etc.)
```

### Atajos de teclado (v0.6)

| Atajo | Acción |
|---|---|
| `F5` | Refrescar datos |
| `Ctrl+E` | Exportar snapshot a JSON |
| `Ctrl+1` … `Ctrl+8` | Navegar a cada tab en orden |

### Métricas del código

| Métrica | Valor |
|---|---|
| LOC totales | ~6,368 |
| Archivo más grande | `app.rs` — 2,973 líneas |
| Dependencias directas | 11 crates |
| Binario estimado | ~18–20 MB |
| Perfil release | `opt-level=3` + `lto=true` + `strip=true` + `panic=abort` + `codegen-units=1` |
| Tests | 1 test unitario (`inspector.rs`) |
| CI gates | fmt + clippy -D warnings + test + build release |

---

## 3. Análisis del producto

### Fortalezas reales

**Arquitectura**
- Separación limpia: `models.rs` como contrato central, servicios desacoplados de la UI.
- `cli.rs` completamente independiente de `app.rs` — los mismos servicios sirven a ambos.
- `meta.rs` centraliza todas las constantes del producto — una sola fuente de verdad.

**Seguridad en acciones privilegiadas**
- `is_valid_firewall_ip()` valida formato IP estrictamente antes de construir cualquier script PowerShell. Sin esta validación habría riesgo de inyección de comandos.
- Allowlist de servicios (`stoppable_services: HashSet`) con doble validación — solo bits, dosvc, sysmain, wuauserv pueden detenerse desde la UI.
- Procesos del sistema protegidos por lista explícita (`protected_names`) y validación de rutas.
- No hay vectores de inyección identificados en el código actual.

**Clasificación inteligente de procesos**
- Scoring ponderado: CPU alto +35, CPU sostenido +18, RAM elevada +28, RAM moderada +14, escritura intensa +40, escritura perceptible +20, ruta temporal +24, patrón instalador +12.
- Umbrales: Healthy 0–24, Warning 25–54, Critical 55+.
- Razones legibles por humano acompañan cada clasificación.

**Robustez operativa**
- Graceful degradation: fallos en subsistemas individuales (netstat, eventos, servicios) generan alertas pero no crashean la app.
- Binario autónomo: rusqlite bundled compila SQLite dentro del exe — sin DLLs externas.

### Deudas técnicas por resolver

| Deuda | Archivo | Severidad | Descripción |
|---|---|---|---|
| `collapsible_if` | `app.rs:402` | 🔴 **BLOQUEANTE CI** | `if let` anidado — colapsar con let-chain de Rust 2024 |
| `print_literal` | `cli.rs:218` | 🔴 **BLOQUEANTE CI** | Literal `"Proceso dominante"` debe ir dentro del string de formato |
| `is_public_ip()` duplicada | `network.rs` + `etl.rs` | 🟡 Media | Misma función en dos módulos — riesgo de divergencia |
| Umbrales como números mágicos | `inspector.rs` | 🟡 Media | `65.0`, `2500.0`, `200.0` sin nombre de constante |
| `.expect("regex válida")` | `etl.rs:372` | 🟡 Media | Único punto de panic potencial del código |
| `app.rs` monolítico | `app.rs` | 🟢 Baja | 2,973 líneas — split en submódulos mejoraría mantenimiento |
| Baselines I/O no se limpian | `inspector.rs` | 🟢 Baja | HashMap de PIDs nunca elimina entradas de procesos muertos |
| SQLite sin retención | `persistence.rs` | 🟢 Baja | La base de datos crece indefinidamente |
| `EMAIL` vacío | `meta.rs` | 🟠 Alta | Se muestra en tab Acerca y en `--help` — pendiente confirmar |
| `GITLAB` sin verificar | `meta.rs` | 🟠 Alta | URL puede ser incorrecta — verificar antes de distribución |
| Solo 1 test | general | 🟡 Media | Código crítico sin cobertura (classify_process, parse_netstat, is_public_ip) |

### Análisis de seguridad — veredicto

**El código no representa ni puede representar malware.** Evidencia verificable:

1. `Cargo.toml` no contiene ningún crate HTTP (`reqwest`, `hyper`, `ureq`, `attohttpc`, `curl` — ninguno).
2. No existe ningún `TcpStream::connect()` ni `UdpSocket::send_to()` saliente.
3. No hay URLs de servidores externos hardcodeadas en ningún módulo.
4. Las únicas "conexiones de red" que toca el código son lecturas del estado local del sistema via netstat (leer, no escribir).
5. Todo almacenamiento es local: SQLite en AppData, JSON en Descargas/Documentos.
6. Los scripts PowerShell se generan internamente con parámetros validados — no se cargan de disco.

**Riesgos residuales de nivel bajo (todos mitigados):**
- PowerShell con `-ExecutionPolicy Bypass`: necesario para compatibilidad de entornos; los comandos son predefinidos, no construidos desde input externo.
- cmdline de procesos: limitado a top 6 críticos, se muestra en UI y CLI, no se envía a ningún lugar.
- XML de ETL: se parsea sin validación de schema; mitigado con límite de longitud y sanitización de strings.

---

## 4. Comparación con software similar

### Tabla comparativa

| Herramienta | Diagnóstico automático | CLI | ETL desde UI | Historial | Telemetría | Código abierto | Precio |
|---|---|---|---|---|---|---|---|
| **RootCause** | ✅ Scoring heurístico | ✅ Completa | ✅ Nativo | ✅ SQLite + A/B | ✅ **Cero** | ✅ Apache 2.0 | Gratis |
| Process Monitor (Sysinternals) | ❌ Solo datos crudos | ❌ | ❌ | ❌ | ✅ | ❌ Freeware | Gratis |
| Process Explorer (Sysinternals) | ❌ Solo datos crudos | ❌ | ❌ | ❌ | ✅ | ❌ Freeware | Gratis |
| Task Manager (Windows) | ❌ Muy básico | ❌ | ❌ | ❌ | Parcial | ❌ | Incluido |
| PC Manager (Microsoft) | Parcial | ❌ | ❌ | ❌ | ❌ Tiene telemetría | ❌ | Gratis |
| GlassWire | Solo red | ❌ | ❌ | Parcial | ❌ Freemium | ❌ | Freemium |
| Resource Monitor | ❌ Solo datos | ❌ | ❌ | ❌ | ✅ | ❌ | Incluido |
| Perfmon | ❌ Solo métricas | Parcial | ❌ | Parcial | ✅ | ❌ | Incluido |

### Ventajas reales de RootCause vs la competencia

1. **Interpretación automática** — Process Monitor muestra miles de eventos; RootCause dice cuál es el dominante y por qué. El scoring heurístico hace el trabajo del analista.
2. **CLI nativa completa** — ninguna herramienta gratuita tiene `rootcause status`, `rootcause block-ip` ni `rootcause wpr`. Único en el segmento.
3. **ETL integrado** — iniciar, detener y analizar trazas WPR sin abrir cmd ni instalar WPA. Único en herramientas gratuitas.
4. **Historial + comparación A/B** — ninguna herramienta gratuita persiste capturas y permite comparar dos momentos. Diferenciador real.
5. **Cero telemetría verificable** — diferenciador fuerte para usuarios técnicos, empresas con políticas de privacidad y entornos corporativos.
6. **Código auditable** — para entornos de seguridad, poder revisar el código es un requisito no negociable.

### Desventajas honestas vs la competencia

1. **No es un event tracer en tiempo real** — Process Monitor captura cada syscall individualmente; RootCause trabaja con snapshots cada N segundos. Para debugging a nivel de syscall, ProcMon sigue siendo la herramienta.
2. **Tamaño del binario** — 18 MB vs 2.5 MB de ProcMon. El overhead de egui es el precio de la interfaz moderna.
3. **Sin análisis de registro de Windows** — no monitorea HKEY_RUN ni entradas de autostart (está en el roadmap como tab Autostart).
4. **Sin firma digital** — activa SmartScreen en primera ejecución. Pendiente para v1.0.
5. **Un solo desarrollador** — menor velocidad de respuesta a bugs críticos comparado con herramientas corporativas.

### Posicionamiento correcto

RootCause **no compite con Process Monitor** — son herramientas distintas para distintos momentos:
- Process Monitor: debugging profundo, ya sé qué buscar.
- **RootCause: primer diagnóstico, no sé qué está pasando.** RootCause te dice dónde mirar.

El competidor más directo real es **PC Manager de Microsoft** — pero PC Manager tiene telemetría, no tiene CLI, no tiene historial y no es auditable.

---

## 5. Análisis del binario

### De dónde viene el peso (~18–20 MB)

```
Componente                              Peso estimado    % del total
─────────────────────────────────────────────────────────────────────
egui + eframe (ventana + widgets)          ~10.0 MB          53%
Fuentes tipográficas (baked en egui)        ~3.5 MB          18%
Tu código Rust (6,368 LOC)                  ~0.8 MB           4%
rusqlite (SQLite compilado)                 ~1.0 MB           5%
sysinfo (métricas del sistema)              ~0.5 MB           3%
serde + serde_json                          ~0.4 MB           2%
Resto de dependencias                       ~0.8 MB           4%
Overhead del ejecutable Windows             ~0.5 MB           3%
─────────────────────────────────────────────────────────────────────
Total                                      ~17.5 MB         100%
```

**Conclusión clave:** tus 6,368 líneas de código representan el 4% del binario. El 96% son dependencias, principalmente egui. Reducir código no reduce el binario de forma significativa.

### Opciones de reducción reales

| Acción | Ahorro estimado | Riesgo | Recomendación |
|---|---|---|---|
| Eliminar `rusqlite` → JSON | ~1.0 MB | Bajo — pierde SQL nativo | **Pendiente decisión** |
| Eliminar `regex` → parsing manual | ~0.3 MB | Bajo | ✅ Factible |
| Quitar egui/eframe | ~13.5 MB | **Destruye la interfaz** | ❌ Jamás |

**Con acciones seguras: ~18 MB → ~16.7 MB.** No es reducción dramática.

**La solución real para binario pequeño:** crear la edición CLI-only (~3–5 MB) con feature flags — misma base de código, sin egui. Ver sección 8.

### SQLite vs JSON — análisis completo (decisión pendiente)

| Criterio | SQLite actual | JSON plano |
|---|---|---|
| Ahorro en binario | — | ~1 MB (elimina rusqlite) |
| Consultas (filtro, orden, límite) | ✅ SQL nativo | ❌ Carga todo en memoria |
| Comparación A/B historial | ✅ Trivial con SELECT | ⚠️ Requiere iterar array |
| Integridad ante crash | ✅ ACID | ⚠️ Posible JSON truncado |
| Crecimiento del archivo | ⚠️ Crece sin límite | ⚠️ Igual, más fácil de purgar |
| Complejidad de mantenimiento | Media | Baja |

**Recomendación**: si el historial A/B y las consultas son features que se mantienen → **quedarse con SQLite** (la funcionalidad justifica el crate). Si se simplifica el historial a "ver últimas N capturas" sin comparación → **migrar a JSON**.

---

## 6. Telemetría

### Qué es telemetría (explicación simple)

Telemetría es cuando un programa que instalas en tu computadora le manda información a alguien más — sin que tú lo pidas y muchas veces sin que lo notes.

Por ejemplo: enciendes un televisor y sin avisarte, el televisor le manda un mensaje a la fábrica diciendo "el usuario lo encendió a las 8pm, vio Netflix 2 horas, está en Santiago de Chile". Eso es telemetría.

Windows lo hace. Chrome lo hace. PC Manager de Microsoft lo hace. La mayoría de herramientas "gratuitas" lo hacen — el negocio detrás es vender esos datos o usarlos para analytics.

### Estado real en RootCause

**Telemetría: CERO. Ninguna. Sin adjetivos.**

Evidencia técnica directa en `Cargo.toml`:
- No existe `reqwest` — el crate HTTP más común en Rust
- No existe `hyper`, `ureq`, `attohttpc`, `curl` ni ningún otro crate de red
- No existe `sentry`, `datadog`, `mixpanel` ni ningún SDK de analytics

Evidencia en el código:
- No hay `TcpStream::connect()` saliente en ningún módulo
- No hay URLs de servidores externos hardcodeadas
- No hay `Command::new("curl")` ni `Invoke-WebRequest` en scripts generados

**Por qué decir "cero" sin adjetivos:** durante esta sesión se cambió a "Sin telemetría activa" en la landing. Eso fue un error — implica que podría haber telemetría pasiva. La realidad es que no hay ninguna. El texto correcto es "Telemetría: cero" o "Sin telemetría".

**Acción pendiente:** corregir en landing el texto "Sin telemetría activa" → "Telemetría: cero" en la feature card y footer.

---

## 7. Decisiones pendientes del usuario

Estas decisiones afectan el código y no pueden tomarse sin confirmación:

| # | Decisión | Opciones | Impacto |
|---|---|---|---|
| 7.1 | **¿EMAIL de contacto?** | Confirmar email para `src/meta.rs` | Se muestra en `--help` y tab Acerca |
| 7.2 | **¿URL GitLab correcta?** | Verificar `https://gitlab.com/vladimiracunadev-create` | Se muestra en tab Acerca y footer landing |
| 7.3 | **¿SQLite o JSON para historial?** | Ver análisis en sección 5 | Afecta ~1 MB del binario y la implementación del historial A/B |
| 7.4 | **¿Texto telemetría en landing?** | "Sin telemetría activa" (actual) vs "Telemetría: cero" (correcto) | Solo cosmético pero semánticamente importante |

---

## 8. Ediciones del producto

### Tres ediciones inmediatas (v0.7)

| Edición | Binario | Peso | Audiencia | Estado |
|---|---|---|---|---|
| **GUI completa** | `rootcause.exe` | ~18 MB | Usuarios con escritorio | ✅ Existe |
| **Instalador** | `rootcause-setup.exe` | ~19 MB | Instalación con PATH automático | ✅ Existe |
| **CLI-only** | `rootcause-cli.exe` | ~3–5 MB | Sysadmins, scripts, automatización | 🔲 Planificado |

### Cómo se implementa CLI-only — Feature Flags de Rust

```toml
# Cargo.toml
[features]
default = ["gui"]
gui = ["eframe", "egui"]

[dependencies]
eframe = { version = "0.27", optional = true }
egui   = { version = "0.27", optional = true }
```

```rust
// main.rs
#[cfg(feature = "gui")]
mod app;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        std::process::exit(cli::run(&args[1..]));
    }
    #[cfg(feature = "gui")]
    launch_gui();
    #[cfg(not(feature = "gui"))]
    { eprintln!("Versión CLI. Usa: rootcause --help"); std::process::exit(0); }
}
```

```bash
cargo build --release                       # GUI completa (~18 MB)
cargo build --release --no-default-features # CLI pura (~3–5 MB)
```

### Otras versiones discutidas (horizonte v0.7–v2.0)

| Versión | Descripción | Esfuerzo | Prioridad |
|---|---|---|---|
| **Módulo PowerShell** | `RootCause.psm1` — `Get-RootCauseStatus`, `Get-RootCauseProcesses`. Wrapper que llama a `rootcause.exe` y convierte JSON en objetos PS. **Cero cambios en Rust.** | Bajo | Alta |
| **Tray icon** | Ícono en bandeja del sistema. Cambia color según severidad. Click abre ventana. Requiere actualizar eframe a 0.28+. | Medio | Alta |
| **Scoop bucket** | `bucket/rootcause.json` en repo público. `scoop install rootcause`. | Bajo | Media |
| **Winget manifest** | YAML para `winget-pkgs`. `winget install rootcause`. | Bajo | Media |
| **Chocolatey** | `.nuspec` + script PS. `choco install rootcause`. | Bajo | Media |
| **Firma digital** | Self-signed cert o CodeSigning comercial. Elimina alerta SmartScreen. | Medio | Media |
| **Tab Autostart** | Leer HKEY_CURRENT_USER\...\Run, carpeta Startup, tareas programadas. | Medio | Media |
| **VS Code Extension** | Barra de estado con estado del sistema. TypeScript wrapper sobre `rootcause status --json`. | Medio | Baja |
| **Windows Service** | `rootcause-service.exe` corre sin usuario logueado. Historial continuo 24/7. GUI se conecta via named pipes. | Muy alta | Largo plazo |
| **Edición Seguridad** | Solo procesos sospechosos + conexiones + bloqueo. Orientada a SOC. Feature flags. | Medio | Largo plazo |
| **Edición Enterprise** | Prometheus/Grafana, multi-equipo, GPO, CSV/Excel. Modelo B2B. | Muy alta | v2.0 |
| **MSIX / Microsoft Store** | Empaquetado para Store. Requiere cuenta desarrollador + firma. | Alta | v2.0 |

---

## 9. Landing page y distribución

### Arquitectura de repos públicos

GitHub Pages permite **un repo especial** (`<usuario>.github.io`) y **repos de proyecto ilimitados** (`usuario.github.io/nombre-repo`). El repo de código puede ser privado; la landing es un repo separado público.

```
repo privado: rootcause-windows-inspector  (código, nunca expuesto)
repo público: rootcause-landing            (landing + releases de binarios)
URL:          https://vladimiracunadev-create.github.io/rootcause-landing/
```

### Flujo de releases públicos

```
CI en repo privado                    Repo público landing
┌─────────────────────────┐           ┌──────────────────────────────┐
│ release-windows.yml     │           │ rootcause-landing            │
│  1. cargo build release │           │  ├── index.html              │
│  2. crea release privado│ ────────▶ │  └── releases/               │
│  3. publica binarios    │           │       ├── rootcause.exe       │
│     en landing via      │           │       ├── rootcause-setup.exe │
│     LANDING_RELEASE_    │           │       └── SHA256SUMS.txt      │
│     TOKEN (PAT secret)  │           └──────────────────────────────┘
└─────────────────────────┘
```

- Código fuente: **nunca expuesto**
- Binarios compilados: **públicamente descargables** desde `rootcause-landing/releases`
- Los botones de descarga en la landing ya apuntan a `rootcause-landing/releases/latest`

### Errores corregidos en la landing (esta sesión)

| Error | Estado |
|---|---|
| Badge CI apuntaba al repo privado (daba 404) | ✅ Reemplazado por badge estático |
| Sección "🦀 Desde fuente" mostraba link al repo privado | ✅ Eliminada, reemplazada por "🔐 Licencia" |
| "Opción B — Desde fuente" en instalación mostraba `git clone` del repo privado | ✅ Eliminada |
| Links del footer a docs del repo privado | ✅ Reemplazados por links internos de la landing |
| Descargas apuntaban a `rootcause-windows-inspector/releases` | ✅ Ahora apuntan a `rootcause-landing/releases` |
| "Sin telemetría activa" implica telemetría pasiva | ⚠️ **Pendiente** — corregir a "Telemetría: cero" |

### Cuándo actualizar la landing

| Evento | Qué actualizar en la landing |
|---|---|
| Bump de versión | Badge versión, número en hero, sección descarga, og:description |
| Nueva funcionalidad visible | Agregar feature card en sección Características |
| Nuevo comando CLI | Tabla de comandos CLI |
| Cambio de requisitos del sistema | Sección Requisitos |
| Release con tag `v*` | Binarios se publican automáticamente (cuando se configure el PAT) |
| Cambio de nombre del producto | Todo (usar skill `rootcause-rename`) |

---

## 10. Skills del proyecto

### Skills disponibles

| Skill | Archivo | Propósito | Versión |
|---|---|---|---|
| `rootcause-improve` | `~/.claude/plugins/.../rootcause-improve/SKILL.md` | Desarrollo, features, CI, docs, release, landing | 1.2.0 |
| `rootcause-rename` | Por crear | Cambiar el nombre del producto en todos los archivos | Pendiente |

### rootcause-rename — especificación

Cuando se ejecute este skill con un nombre nuevo, debe:

1. Buscar todas las ocurrencias del nombre actual con `grep -r "RootCause" --include="*.rs" --include="*.md" --include="*.toml" --include="*.html" --include="*.json"`
2. Listar todos los archivos afectados agrupados por tipo
3. Mostrar al usuario la lista completa y pedir confirmación explícita antes de modificar
4. Si se confirma, actualizar en todos los archivos:
   - `src/meta.rs` — `DISPLAY_NAME`, `DESCRIPTION`
   - `Cargo.toml` — `name`, `description`
   - Todos los `.md` en `docs/` y raíz
   - `rootcause-landing/index.html` — title, meta, headers, badges, footer
   - `.github/workflows/*.yml` — nombres de jobs y artefactos
   - Los propios skills (`SKILL.md`, `README.md` del skill)
   - `packaging/windows/installer.iss` — nombre del instalador
5. Ejecutar `run_fmt.ps1` tras los cambios en Rust
6. Hacer commit con mensaje `chore: renombrar producto a <NombreNuevo>`
7. Push a ambos repos

---

## 11. Prioridades ordenadas

---

### 🚨 FASE 0 — Bloqueante inmediato (hacer PRIMERO)

**CI está roto. Nada más avanza hasta que esto esté verde.**

| # | Tarea | Archivo | Cambio exacto |
|---|---|---|---|
| 0.1 | Fix `collapsible_if` | `src/app.rs:402` | Cambiar `if let Some(idx) = tab_switch { if let Some(&(tab,_,_)) = Tab::ALL.get(idx) { ... } }` → `if let Some(idx) = tab_switch && let Some(&(tab,_,_)) = Tab::ALL.get(idx) { ... }` |
| 0.2 | Fix `print_literal` | `src/cli.rs:218` | Mover el literal `"Proceso dominante"` dentro del string de formato, eliminarlo del argumento |
| 0.3 | `run_fmt.ps1` | raíz | `powershell.exe -ExecutionPolicy Bypass -File run_fmt.ps1` |
| 0.4 | Push y verificar CI verde | GitHub Actions | Confirmar que los 4 gates pasan |

---

### 🔧 FASE 1 — Completar v0.6 (código pendiente)

`get_hardware_info()` ya existe en `inspector.rs`. Falta conectarlo a la UI.

| # | Tarea | Archivo | Detalle |
|---|---|---|---|
| 1.1 | Campo `hardware_info: HardwareInfo` en struct | `src/app.rs` | Agregar al struct `RootCauseApp` |
| 1.2 | Poblar en `new()` | `src/app.rs` | Si inspector OK: `insp.get_hardware_info()`. Si falla: `HardwareInfo::default()` |
| 1.3 | Sección hardware en tab Overview | `src/app.rs` | OS, versión, hostname, CPU (marca + núcleos + MHz), RAM total GB, arquitectura |
| 1.4 | Atajos de teclado en `update()` | `src/app.rs` | Patrón collect-then-execute para evitar borrow conflict con egui closures |
| 1.5 | Sección atajos en `draw_tab_about()` | `src/app.rs` | Tabla visual F5, Ctrl+E, Ctrl+1..8 |
| 1.6 | `run_fmt.ps1` + verificar | — | Sin warnings nuevos |
| 1.7 | Commit y push | master | `feat: completar v0.6 — hardware, atajos, tab Acerca` |
| 1.8 | Reubicar tag v0.6.0 | GitHub | Apuntar al commit final correcto |

---

### 🌐 FASE 2 — Metadatos y landing

| # | Tarea | Archivo | Detalle |
|---|---|---|---|
| 2.1 | Confirmar EMAIL | `src/meta.rs:20` | Usuario debe proveer su email de contacto |
| 2.2 | Verificar URL GitLab | `src/meta.rs:27` | Confirmar o corregir URL |
| 2.3 | Corregir telemetría en landing | `rootcause-landing/index.html` | "Sin telemetría activa" → "Telemetría: cero" en feature card y footer |
| 2.4 | Push landing | rama `main` | GitHub Pages redespliega en ~60s |

---

### 📚 FASE 3 — Documentación completa (barrido)

| # | Tarea | Detalle |
|---|---|---|
| 3.1 | `docs/ARCHITECTURE.md` | Añadir `meta.rs` y `cli.rs`. Actualizar `inspector.rs` con `get_hardware_info()`. Actualizar `models.rs` con `HardwareInfo`. |
| 3.2 | `docs/OPERACION.md` | Agregar sección "Uso desde consola (CLI)" con tabla completa de comandos y ejemplos. Añadir atajos de teclado. |
| 3.3 | `docs/COMMANDS.md` | Crear (si no existe) documento completo de comandos CLI con ejemplos y códigos de salida. |
| 3.4 | `docs/ROADMAP.md` | Marcar v0.6 100% completado con lista completa de deliverables. Agregar v0.7 con las features planificadas. |
| 3.5 | `docs/RECLUTADORES.md` | Añadir CLI completa, atajos, hardware info, landing page y análisis de competencia como capacidades. |
| 3.6 | `docs/INDEX.md` | Enlazar `COMMANDS.md`, `PLAN_MAESTRO.md` y cualquier doc nuevo. |
| 3.7 | `README.md` (raíz) | Verificar que no diga "ROOTCAL" — debe decir "ROOTCAUSE" en todas las ocurrencias. |
| 3.8 | Crear skill `rootcause-rename` | Ver especificación en sección 10. |
| 3.9 | Push `rootcause-windows-inspector` | `docs: barrido completo v0.6 — CLI, atajos, hardware, skills, plan maestro` |

---

### ⚙️ FASE 4 — Calidad de código

| # | Tarea | Archivo | Impacto |
|---|---|---|---|
| 4.1 | Consolidar `is_public_ip()` | `network.rs` + `etl.rs` | Eliminar duplicación — una sola función en `network.rs`, importar en `etl.rs` |
| 4.2 | Constantes para umbrales de clasificación | `inspector.rs` | `const CPU_HIGH_PCT: f32 = 65.0` etc. — sin números mágicos |
| 4.3 | Fix `.expect()` | `etl.rs:372` | Reemplazar con `OnceLock` o `LazyLock` (Rust 1.80+) — eliminar único panic potencial |
| 4.4 | Limpieza de baselines I/O | `inspector.rs` | Eliminar entradas de PIDs que ya no existen en `self.system.processes()` |
| 4.5 | Retención SQLite | `persistence.rs` | `DELETE FROM snapshots WHERE id NOT IN (SELECT id FROM snapshots ORDER BY collected_at DESC LIMIT 1000)` |
| 4.6 | **Migrar SQLite → JSON** | `persistence.rs` | **Solo si usuario confirma en decisión 7.3.** Elimina `rusqlite`, ahorra ~1 MB |
| 4.7 | Dividir `app.rs` en submódulos | `src/app/` | `overview.rs`, `processes.rs`, `connections.rs`, etc. No afecta binario ni funcionalidad |
| 4.8 | Tests unitarios | `tests/` o inline | Mínimo: `classify_process()`, `is_public_ip()`, `is_valid_firewall_ip()`, `parse_netstat_output()` |

---

### 🚀 FASE 5 — Tres ediciones del producto (v0.7)

| # | Edición | Implementación | Esfuerzo |
|---|---|---|---|
| 5.1 | **CLI-only** | Feature flags en Cargo.toml. `eframe`/`egui` como `optional`. `#[cfg(feature = "gui")]` en main.rs. Build con `--no-default-features` en CI. | Bajo |
| 5.2 | **Módulo PowerShell** | `RootCause.psm1` que llama a `rootcause.exe` y convierte JSON en objetos PS. `Get-RootCauseStatus`, `Get-RootCauseProcesses`, `Invoke-RootCauseExport`. Cero cambios en Rust. | Bajo |
| 5.3 | **Tray icon** | Ícono en bandeja. Cambia color según severidad. Click abre GUI. Requiere actualizar eframe a 0.28+. | Medio |

---

### 📦 FASE 6 — Distribución pública (v0.7 → v1.0)

| # | Tarea | Descripción | Prerequisito |
|---|---|---|---|
| 6.1 | **Auto-publish releases a landing** | En `release-windows.yml`: `gh release create --repo rootcause-landing` con los binarios. Requiere secret `LANDING_RELEASE_TOKEN` (PAT con permisos al repo landing). | PAT configurado |
| 6.2 | **Scoop bucket** | `bucket/rootcause.json` con SHA256, URL y descripción. `scoop install rootcause`. | Release público |
| 6.3 | **Winget manifest** | YAML en `winget-pkgs`. `winget install rootcause`. | Release público |
| 6.4 | **Firma digital** | Self-signed o CodeSigning cert comercial. Elimina alerta SmartScreen. | Opcional pero recomendado |
| 6.5 | **Chocolatey** | `.nuspec` + PS install script. `choco install rootcause`. | Release público |

---

### 🔬 FASE 7 — Features de producto nuevas (v0.7 → v1.0)

| # | Feature | Descripción | Impacto | Cero crates nuevos |
|---|---|---|---|---|
| 7.1 | **Tab Autostart** | Leer HKEY_CURRENT_USER\...\Run, carpeta Startup, tareas programadas via PowerShell. Mostrar ejecutable, ruta y estado. | Alto — diferenciador vs Sysinternals | ✅ PowerShell |
| 7.2 | **Alertas configurables** | Panel de configuración: umbrales CPU/RAM/IO, intervalo de refresco, notificaciones. Guardar en `rootcause.toml` en AppData. | Medio | ✅ serde_json o toml |
| 7.3 | **`--output` en CLI** | `rootcause snapshot --output diag.json` además de stdout. | Bajo | ✅ |
| 7.4 | **Tests unitarios** | Cubrir `classify_process()`, `parse_netstat_output()`, `is_public_ip()`, `is_valid_firewall_ip()`. | Alto para confianza en releases | ✅ |
| 7.5 | **Archivo de configuración** | `rootcause.toml` para personalización de la app. | Bajo | ✅ |

---

### 🏗️ FASE 8 — Largo plazo (v2.0+)

| # | Versión | Descripción | Complejidad |
|---|---|---|---|
| 8.1 | **Windows Service** | `rootcause-service.exe` corre sin usuario. Historial continuo 24/7. GUI conecta via named pipes. | Muy alta |
| 8.2 | **VS Code Extension** | Barra de estado con estado del sistema. TypeScript wrapper sobre `rootcause status --json`. | Media |
| 8.3 | **Edición Seguridad** | Solo procesos sospechosos + conexiones + bloqueo. UI orientada a SOC. Feature flags. | Media |
| 8.4 | **Edición Enterprise** | Prometheus/Grafana, multi-equipo, GPO, CSV/Excel. Modelo B2B. | Muy alta |
| 8.5 | **MSIX / Microsoft Store** | Requiere cuenta desarrollador + firma digital. | Alta |

---

## 12. Reglas de trabajo por sesión

### Al iniciar cualquier sesión

```
1. Leer este documento completo (PLAN_MAESTRO.md)
2. git log --oneline -3  →  ¿en qué commit estamos?
3. Verificar CI en GitHub Actions → ¿verde o rojo?
4. Si CI rojo → resolver FASE 0 antes de cualquier otra cosa
```

### Flujo obligatorio para cambios Rust

```
1. Leer el archivo con Read tool ANTES de editar (obligatorio)
2. Editar con Edit tool (no reescribir archivos completos salvo necesidad real)
3. powershell.exe -ExecutionPolicy Bypass -File run_fmt.ps1
4. Verificar clippy (run_check.ps1 o esperar CI)
5. git add <archivos específicos>  (nunca git add -A)
6. git commit (en español + Co-Authored-By Claude Sonnet 4.6)
7. git push origin master
8. Si es release: tag + push tag + actualizar landing
```

### Flujo obligatorio para actualizar la landing

```
1. cd C:\dev\rootcause-landing
2. Editar index.html con los cambios
3. git add index.html
4. git commit -m "chore: actualizar landing para vX.Y.Z"
5. git push origin main
   → GitHub Pages redespliega en ~60 segundos
```

### Notas de entorno local (CRÍTICO)

| Nota | Detalle |
|---|---|
| **Conflicto MSVC/MSYS2** | bash usa el `link.exe` de MSYS2. Cualquier `cargo check/build/clippy` puede fallar con error de linker. **Siempre usar PowerShell.** |
| **cargo fmt** | `powershell.exe -ExecutionPolicy Bypass -File run_fmt.ps1` |
| **cargo check/clippy** | `powershell.exe -ExecutionPolicy Bypass -File run_check.ps1` |
| **CI en GitHub** | Usa `windows-latest` con toolchain correcto — siempre compila bien aunque falle local |
| **Ruta producto** | `C:\dev\rootcause-windows-inspector` — rama `master` |
| **Ruta landing** | `C:\dev\rootcause-landing` — rama `main` |
| **Toolchain Rust** | `C:\Users\vbav\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin\` |

### Convenciones de commits

| Prefijo | Cuándo |
|---|---|
| `feat:` | Nueva funcionalidad |
| `fix:` | Corrección de bug |
| `style:` | Solo cargo fmt |
| `docs:` | Solo documentación |
| `refactor:` | Reestructuración sin cambio funcional |
| `test:` | Tests |
| `chore:` | Bump versión, CI, mantenimiento |

Siempre en **español**. Siempre con:
```
Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
```

### Checklist de release (resumen rápido)

```
☐ Bump versión en Cargo.toml
☐ Badge versión en README.md actualizado
☐ ROADMAP.md — ítem ✅
☐ ARCHITECTURE.md — si hay módulos nuevos
☐ OPERACION.md — si hay acciones nuevas
☐ COMMANDS.md — si hay comandos CLI nuevos
☐ RECLUTADORES.md — features nuevas
☐ INDEX.md — docs nuevos enlazados
☐ rootcause-landing/index.html — versión + features
☐ run_fmt.ps1 sin errores
☐ CI verde (4 gates)
☐ git tag -a vX.Y.Z
☐ git push origin master && git push origin vX.Y.Z
☐ Verificar que CI genera ZIP + Setup.exe + SHA256SUMS
☐ Publicar binarios en rootcause-landing/releases
☐ Verificar landing en browser
```

---

*Última actualización: 2026-03-17. Al modificar el producto de forma significativa, actualizar las secciones afectadas y hacer commit de este documento.*
