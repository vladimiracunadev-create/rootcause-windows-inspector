```
╔═══════════════════════════════════════════════════════════════════════════════════╗
║                                                                                   ║
║  ██████╗  ██████╗  ██████╗ ████████╗ ██████╗  █████╗ ██╗   ██╗███████╗███████╗    ║
║  ██╔══██╗██╔═══██╗██╔═══██╗╚══██╔══╝██╔════╝ ██╔══██╗██║   ██║██╔════╝██╔════╝    ║
║  ██████╔╝██║   ██║██║   ██║   ██║   ██║      ███████║██║   ██║███████╗█████╗      ║
║  ██╔══██╗██║   ██║██║   ██║   ██║   ██║      ██╔══██║██║   ██║╚════██║██╔══╝      ║
║  ██║  ██║╚██████╔╝╚██████╔╝   ██║   ╚██████╗ ██║  ██║╚██████╔╝███████║███████╗    ║
║  ╚═╝  ╚═╝ ╚═════╝  ╚═════╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝ ╚═════╝╚══════╝╚══════╝      ║
║                                                                                   ║
║                     W I N D O W S   I N S P E C T O R                             ║
║               Forensic diagnostics · Built in Rust · v0.18.0                      ║
╚═══════════════════════════════════════════════════════════════════════════════════╝
```

[![CI Windows](https://github.com/vladimiracunadev-create/rootcause-windows-inspector/actions/workflows/ci.yml/badge.svg)](https://github.com/vladimiracunadev-create/rootcause-windows-inspector/actions/workflows/ci.yml)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-edition%202024-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Windows%2010%2F11-lightgrey.svg)](docs/REQUIREMENTS.md)
[![Version](https://img.shields.io/badge/version-0.18.0-green.svg)](docs/ROADMAP.md)

🌐 **[Página del producto →](https://vladimiracunadev-create.github.io/rootcause-windows-inspector/)**  ·  📘 **[Manual de usuario →](docs/MANUAL_USUARIO.md)** (qué es cada cosa, en claro)

---

**RootCause es un software forense de ciberseguridad para Windows**, escrito en **Rust**.

Nace de una idea que es su razón de existir: **cualquier distorsión anómala de los recursos de un equipo —CPU, disco, memoria, red, procesos, autoarranque o servicios— puede ser el primer indicio de que algo está ocurriendo.** No solo lentitud: también malware activo, persistencia, exfiltración, un binario secuestrado o un control de seguridad que se apagó. RootCause **vigila esas distorsiones de forma agnóstica** (no necesita saber *qué* amenaza es para notar que "algo se comporta distinto"), **correlaciona las señales** en incidentes y **explica la causa raíz con evidencia** — el indicio temprano que te dice **dónde mirar**.

> **Diagnóstico primero. Intervención después.**

Es un **sensor forense y de apoyo a la decisión**, no un antivirus ni un EDR: no elimina malware ni bloquea por firma — **detecta indicios de comportamiento** (persistencia, tráfico saliente inusual, ejecución en rutas sospechosas, escritura agresiva tipo ransomware, cryptojacking…), deja registro y **complementa** a tu antivirus/EDR. También responde el lado "rendimiento" del mismo problema: qué proceso, carpeta, servicio o conexión está degradando el equipo.

> 🛡️ **Qué detecta hoy, amenaza por amenaza (con honestidad):** [`docs/DETECCION_AMENAZAS.md`](docs/DETECCION_AMENAZAS.md)

---

## ⚡ Inicio rápido

```powershell
# 1. Verificar entorno (Rust, WPR, Inno Setup)
.\scripts\verify-environment.ps1

# 2. Compilar
cargo build --release

# 3. Ejecutar (requiere administrador para algunas funciones)
.\target\release\rootcause.exe
```

> Para empaquetado como ZIP portable o instalador → [`docs/BUILD_WINDOWS.md`](docs/BUILD_WINDOWS.md) y [`docs/PACKAGING_WINDOWS.md`](docs/PACKAGING_WINDOWS.md).

---

## 🔍 Qué problema resuelve

Preguntas concretas que Windows no responde bien en una sola vista:

| Pregunta | Cómo RootCause ayuda |
|---|---|
| ¿Qué proceso carga el disco ahora mismo? | Top de procesos con I/O delta + sparkline de tendencia |
| ¿Qué carpeta temporal creció y cuánto? | Escaneo de `%TEMP%`, `SoftwareDistribution`, `DeliveryOptimization` |
| ¿Viene de Windows Update, BITS o SysMain? | Correlación proceso + servicios activos |
| ¿Qué ejecutable mantiene conexiones a IPs públicas? | Tabla red + bloqueo de IP desde la UI |
| ¿Cómo capturo evidencia sin añadir otra carga? | WPR/ETW integrado, resumen ETL sin salir de la app |

---

## 🛡️ Capas de funcionamiento

### 1 · Modo operativo liviano
El modo principal. Bajo consumo, útil para observación frecuente.

- 🟢🟡🔴 Semáforo general de estado del sistema
- 📊 Sparklines de CPU · RAM · I/O en tiempo real
- 🔝 Top de procesos dominantes con severidad y puntaje
- 🌡️ Presión CPU / RAM / escritura / lectura entre intervalos
- 🗂️ Escaneo de `TEMP` y cachés de Windows Update
- 🌐 Conexiones activas por proceso con correlación a IP pública
- 📜 Eventos recientes del sistema y servicios relevantes
- 🗃️ Exportación JSON · Historial SQLite con comparación A vs B
- ⚡ Filtro de severidad por proceso (Critical / Warning / Normal)
- 🔔 Notificaciones toast cuando aparece proceso Critical
- ⌨️ Atajos de teclado: `F5` actualizar · `Ctrl+E` exportar · `Ctrl+1…8` primeras 8 tabs · `Ctrl+9` Configuración · `Ctrl+0` Manual (Acerca solo por clic)
- 🖥️ Info de hardware del equipo: OS, CPU, núcleos, frecuencia, RAM
- 🚀 Tab Autostart: registro Run (HKCU/HKLM) y carpetas Startup con severidad y verificación en disco — detecta cambios contra una baseline conocida (NUEVA/MODIFICADA/ELIMINADA) y genera alertas `persistence-change`
- ⚙️ Detección de cambios en servicios de Windows (vía CLI `rootcause services` + alertas `service-change` en incidentes; no es un tab de la GUI): vigila `StartMode` + ruta del binario de cada servicio contra una baseline conocida (NUEVA/MODIFICADA/ELIMINADA) — captura servicios nuevos, secuestro del binario o cambios de modo de arranque (ej. deshabilitar Defender)
- 🗑️ Limpieza segura de temporales: botón **Limpiar %TEMP%** en el tab Temporales (y `rootcause clean-temp`) — borra solo tu `%TEMP%`, solo lo no usado en 24 h, saltando lo que esté en uso; confirmación de dos pasos y `--yes` en CLI
- 📖 Tab Manual: guía integrada que explica qué hace cada pestaña, la detección por baseline y las acciones seguras
- 🐳 Espacio de Docker (tab Temporales): imágenes, volúmenes y espacio recuperable vía `docker system df`, con purga guiada segura de 2 pasos (solo imágenes *dangling* + caché de build; los volúmenes no se autoborran) y `rootcause docker [--json|--prune-images|--prune-cache]`
- 🌍 Interfaz bilingüe **español / inglés** con selector persistente en el tab Configuración
- 🎨 Modos de tema **Claro / Oscuro / Windows** (este último sigue el tema del sistema); acento derivado del icono
- 🪟 Interfaz estilo **Windows 11 / Fluent**: barra lateral de navegación (NavigationView), tipografía nativa Segoe UI e iconos de línea
- 💻 CLI completa: `rootcause --help` con todos los comandos desde consola

### 2 · Modo de precisión ETW/WPR
Para cuando la observación liviana no basta.

- Iniciar / detener / cancelar captura WPR desde la UI
- Registrar contexto humano del problema
- Resumir el último ETL con `tracerpt` sin salir de la app
- Barra visual de proveedores ETW activos
- Ruta clara a WPA para análisis profundo posterior

---

## Seguridad, resiliencia y evolucion

RootCause evoluciona no solo para diagnosticar degradacion y fallos, sino tambien para detectar señales anómalas compatibles con actividad no autorizada y fortalecer la resiliencia del propio agente.

Estas lineas ya quedaron formalizadas como documentacion viva del producto, no como notas sueltas:

- [`REQ-SEC-001 - Deteccion de comportamiento anomalo y posible actividad maliciosa`](docs/requirements/REQ-SEC-001-deteccion-comportamiento-anomalo.md): define una evolucion basada en heuristicas, correlacion de señales, evidencia tecnica y sugerencias de mitigacion para procesos sospechosos, consumo anomalo, conexiones salientes inusuales, persistencia y rutas de ejecucion sospechosas.
- [`REQ-SEC-002 - Autoproteccion y resiliencia del agente RootCause`](docs/requirements/REQ-SEC-002-autoproteccion-y-resiliencia.md): ya cuenta con una base inicial de heartbeat local, deteccion de cierre abrupto, evidencia de integridad de configuracion, backoff recomendado y exposicion visible en GUI/CLI.
- [`Registro permanente de requerimientos`](docs/requirements/README.md): concentra estado, prioridad y trazabilidad con el roadmap tecnico.
- [`Qué hace RootCause frente a cada amenaza (hoy)`](docs/DETECCION_AMENAZAS.md): mapa **honesto** de cada familia de amenaza de ciberseguridad → qué detecta RootCause hoy (o por qué queda fuera de alcance), partiendo de que toda distorsión de recursos puede ser el primer indicio.
- [`Comparativa con el open source de seguridad/DFIR`](docs/COMPARATIVA_OSS.md): RootCause frente a osquery, Velociraptor, Wazuh, Sysmon, Sigma, Hayabusa/Chainsaw…, con **qué tomar / qué evitar** (y compatibilidad de licencias) y las **oportunidades priorizadas en Rust** para mejorar el producto antes de lanzarlo.
- [`Guía sencilla: cómo tomar cosas de la competencia`](docs/GUIA_COMPETENCIA.md): la versión **corta y clara** — semáforo de licencias y 5 pasos, caso a caso, para decidir cuándo se puede tomar código y cuándo solo la idea.

- [`Modulo de deteccion de comportamiento anomalo (V1)`](docs/MODULO_DETECCION_ANOMALIAS.md): describe la implementacion inicial ya integrada en el repositorio, con heuristicas locales, correlacion simple, incidentes resumidos, configuracion y salida visible en GUI/CLI.

Implementacion actual del repo:

- REQ-SEC-001 ya cuenta con una V1 inicial integrada para detectar CPU sostenido anormal, crecimiento de memoria, escritura agresiva, trafico saliente inusual, rutas sospechosas, persistencia basica, respawn rapido, scripts repetitivos y correlacion de senales.
- La salida actual muestra severidad, score, proceso involucrado, hipotesis de causa, evidencia resumida y recomendacion sugerida.
- REQ-SEC-002 pasa a una implementacion inicial y honesta: heartbeat local, recuperacion tras cierre abrupto, vigilancia basica de configuracion y estado del agente visible; aun no promete invulnerabilidad ni un supervisor persistente de nivel servicio.

Posicionamiento honesto:

- RootCause puede evolucionar para detectar señales compatibles con actividad maliciosa o no autorizada, sin reemplazar una solucion antivirus o EDR especializada.
- RootCause tambien debe contemplar la resiliencia de su propio agente, porque una herramienta de diagnostico puede convertirse en objetivo de manipulacion en escenarios reales.

---

## 🗂️ Ediciones del producto

| Modalidad | Tipo | Estado | Cómo se usa | ¿Sale en `release-windows`? |
|---|---|---|---|---|
| **GUI Desktop** | Núcleo principal | Producción | Instalador / portable | Sí |
| **CLI-only** | Núcleo alternativo | Producción | `--no-default-features` | Sí |
| **PowerShell module** | Adaptador | Producción | `Import-Module RootCause` | Sí |
| **VS Code Extension** | Adaptador | Producción | `code --install-extension` | Sí |
| **Tray icon** | Extensión del runtime | Producción | Incluido en la edición GUI | Sí |
| **Windows Service** | Extensión del runtime | Skeleton | Feature `service` futura | No |
| **RootCause Demo** | Perfil de distribución | Opcional | Instalador demo separado | No por defecto |

`PowerShell module` y `VS Code Extension` reutilizan `rootcause.exe`; no son motores alternativos del producto.

```powershell
# Edición CLI-only (~4 MB, sin interfaz gráfica)
cargo build --release --no-default-features

# Edición GUI completa (~18 MB, por defecto)
cargo build --release
```

### Artefactos oficiales del release principal

| Archivo | Contenido |
|---|---|
| `RootCause-Setup.exe` | Instalador principal de la app |
| `RootCause-Portable.zip` | Portable del build principal GUI + CLI |
| `RootCause-CLI-Portable.zip` | Portable de la edición CLI-only |
| `RootCause.psm1` | Módulo PowerShell |
| `RootCause-VSCode-Extension.vsix` | Extensión VS Code |
| `SHA256SUMS.txt` | Hashes de integridad |

Instalación rápida por modalidad:

- `RootCause-Setup.exe`: instalar y luego usar GUI o `rootcause` desde consola.
- `RootCause-Portable.zip`: extraer y ejecutar `rootcause.exe` (build principal con GUI + CLI).
- `RootCause-CLI-Portable.zip`: extraer y ejecutar `rootcause.exe` desde consola.
- `RootCause.psm1`: requiere `rootcause.exe` ya instalado o disponible en `PATH`.
- `RootCause-VSCode-Extension.vsix`: requiere `rootcause.exe` ya instalado o configurable en `rootcause.executablePath`.

Fuente de verdad del catálogo: [`docs/CATALOGO_PRODUCTO.md`](docs/CATALOGO_PRODUCTO.md).

---

## 📦 Gestores de paquetes Windows

```powershell
# Scoop
scoop install rootcause

# Winget
winget install VladimirAcuna.RootCause

# Chocolatey
choco install rootcause-windows-inspector
```

Manifests en `packaging/distribution/` · Módulo PowerShell en `packaging/powershell/`.

---

## 📁 Estado de entrega del repositorio

### ✅ Incluye
- Código fuente completo en Rust
- GUI nativa con `eframe/egui` (feature `gui`, por defecto)
- Edición CLI-only mediante feature flags (`--no-default-features`)
- Módulo PowerShell (`RootCause.psm1`) — 9 cmdlets nativos
- Manifests de distribución: Scoop, Winget, Chocolatey
- Extensión VS Code con status bar, alertas y panel de diagnóstico
- Tray icon activo (color por severidad); skeleton documentado: Windows Service
- Scripts de verificación, build, empaquetado y análisis ETL
- Documentación profunda de arquitectura, requisitos, operación y CI
- Modo de precisión WPR/ETW integrado en la interfaz
- Historial SQLite (últimas 1000 filas) + backup automático a JSON
- Configuración operativa en JSON (`rootcause-config.json`) con defaults seguros
- Salud del agente con heartbeat local, detección de cierre abrupto previo e integridad básica de configuración
- Registro de incidentes resumidos + auditoría de acciones en SQLite
- Adaptador IA opcional por API, desacoplado y apagado por defecto
- Instalador silencioso compatible con despliegue corporativo (`/VERYSILENT /SUPPRESSMSGBOXES`)

### ❌ No incluye
- `.exe` precompilado
- Firma digital
- Driver de kernel
- Parser completo equivalente a WPA

> **Por qué no se entrega el `.exe`:** facilita auditoría del código, evita binarios opacos, permite compilar según tu entorno Windows real y deja ruta limpia a firma digital futura.

---

## 🗂️ Secciones de la interfaz

Las **11 secciones** viven en una **barra lateral** (NavigationView estilo Windows 11), agrupadas por tema:

| Tab | Descripción |
|---|---|
| **Resumen** | Banner de veredicto + semáforo global + sparklines + características del equipo |
| **Procesos** | Tabla con filtro de severidad + command line de proceso |
| **Conexiones** | Conexiones activas por proceso + bloqueo de IP |
| **Temporales** | Cachés de Windows (TEMP, SoftwareDistribution, etc.) + botón **Limpiar %TEMP%** + **gestión de Docker** (imágenes, volúmenes, purga segura) |
| **ETW / WPR** | Captura WPR + resumen de traza ETL |
| **Servicios** | wuauserv, BITS, DoSvc, SysMain + eventos recientes |
| **Autostart** | Registro Run/RunOnce, carpetas Startup y tareas programadas + detección de cambios vs baseline |
| **Historial** | Snapshots SQLite + comparación A vs B con deltas |
| **Configuración** | Apariencia (modos Claro/Oscuro/Windows), idioma ES/EN y umbrales de detección |
| **Manual** | Guía integrada: qué hace cada tab, detección por baseline y acciones seguras |
| **Acerca** | Versión, autor, GitHub, atajos de teclado, hardware |

---

## 🔬 Funcionalidades implementadas

<details>
<summary><strong>Observación de sistema</strong></summary>

- CPU global · Memoria usada / total
- Red entre intervalos · I/O agregado entre intervalos
- Semáforo general · Sparklines de tendencia (ring buffer 60 muestras)

</details>

<details>
<summary><strong>Procesos dominantes</strong></summary>

- Nombre · PID · Ruta del ejecutable · Command line
- CPU · RAM · Lectura / escritura del intervalo
- Severidad y puntaje · Categoría · Explicación breve
- Filtro interactivo por severidad

</details>

<details>
<summary><strong>Temporales y cachés críticas</strong></summary>

- `%TEMP%` · `C:\Windows\Temp`
- `C:\Windows\SoftwareDistribution\Download`
- `C:\ProgramData\Microsoft\Windows\DeliveryOptimization\Cache`

</details>

<details>
<summary><strong>Red y conexiones</strong></summary>

- Conexiones activas vía `netstat` · Correlación con PID
- Foco en IP pública · Bloqueo controlado por firewall

</details>

<details>
<summary><strong>Historial y evidencia</strong></summary>

- Historial SQLite: retención de hasta 1000 filas; el tab Historial muestra y compara las últimas 60 capturas
- Comparación A vs B con deltas de CPU / RAM / I/O / Alertas
- Incidentes resumidos persistidos con causas probables y evidencia correlacionada
- Auditoría local de acciones (`kill`, `block-ip`, `stop-service`, `accept-persistence-baseline`, `accept-service-baseline`, WPR, IA opcional)
- Exportación JSON · Carpeta trazas ETL y análisis

</details>

<details>
<summary><strong>Intervención controlada</strong></summary>

- Finalizar procesos no protegidos · Bloquear IP remota
- Detener temporalmente servicios (`BITS`, `DoSvc`, `SysMain`, `wuauserv`)
- Iniciar / detener / cancelar captura WPR
- Resumir el último ETL capturado
- Acciones manuales gobernadas por configuración local y auditadas en SQLite

</details>

<details>
<summary><strong>Configuración e IA opcional</strong></summary>

- `rootcause config show` · `rootcause config init`
- Umbrales de procesos y temporales en `rootcause-config.json`
- `rootcause status --json` y `rootcause history --json` para integraciones
- `rootcause incidents` para revisar degradaciones persistidas
- `rootcause ai explain-latest` para enriquecer el último incidente solo si IA está habilitada
- Si la IA falla o no está configurada, RootCause sigue funcionando normal

</details>

<details>
<summary><strong>Deteccion de comportamiento anomalo (V1)</strong></summary>

- Heuristicas locales para CPU sostenido, crecimiento de memoria, escritura agresiva y trafico saliente inusual
- Rutas sospechosas, baseline confiable configurable y relacion padre-hijo sospechosa
- Persistencia basica en Run/RunOnce/Startup y servicios de seguridad relevantes, con comparacion contra una baseline conocida y clasificacion de cambios (NUEVA/MODIFICADA/ELIMINADA) que emite anomalias `persistence-change`
- Vigilancia de servicios de Windows (`StartMode` + ruta del binario) contra una baseline conocida, con la misma clasificacion (NUEVA/MODIFICADA/ELIMINADA) que emite anomalias `service-change`
- Correlacion simple de senales con score, severidad, hipotesis de causa y recomendacion sugerida
- Exposicion del incidente dominante en GUI, CLI, export JSON e historial persistido
- Posicionamiento honesto: complementa observabilidad y diagnostico; no reemplaza antivirus o EDR

</details>

---

## 🚀 Validación automática

Este repositorio incluye validación visible en GitHub Actions:

- **`ci.yml`** — formato, lint, tests y build release en `windows-latest`
- **`release-windows.yml`** — ZIP portable, instalador Inno y hashes SHA-256
- Réplica local de CI con `scripts/ci-local.ps1`
- Guía completa en [`docs/CI_GITHUB.md`](docs/CI_GITHUB.md)

> La CI aumenta la confianza, pero no reemplaza prueba manual en Windows real con WPR/WPA y permisos adecuados.

---

## 📦 Empaquetado

```powershell
# Flujo completo de release y artefactos
.\scripts\release-product.ps1 -VerifyEnvironment

# Flujo completo + push/tag/release
.\scripts\release-product.ps1 -VerifyEnvironment -Publish

# Instalación silenciosa (corporativo)
RootCause-Setup.exe /VERYSILENT /SUPPRESSMSGBOXES /NORESTART
```

También disponible desde shell:

```sh
./scripts/release-product.sh -VerifyEnvironment
./scripts/release-product.sh -VerifyEnvironment -Publish
```

Detalle completo → [`docs/PACKAGING_WINDOWS.md`](docs/PACKAGING_WINDOWS.md)

---

## 📐 Estructura del repositorio

```text
rootcause-windows-inspector/
├── Cargo.toml            ← versión, features (gui / cli-only), dependencias
├── README.md
├── LICENSE               ← Apache 2.0
├── SECURITY.md
├── docs/                 ← 25+ documentos de arquitectura, operación y producto
├── landing/              ← Landing page (servida por GitHub Pages desde este repo)
├── packaging/
│   ├── windows/          ← Inno Setup .iss (instalador GUI)
│   ├── powershell/       ← RootCause.psm1 (módulo PowerShell, 9 cmdlets)
│   ├── chocolatey/       ← rootcause.nuspec + chocolateyInstall.ps1
│   └── distribution/
│       ├── scoop/        ← rootcause.json (manifest Scoop)
│       └── winget/       ← rootcause.yaml (manifest Winget)
├── vscode-extension/     ← Extensión VS Code (TypeScript, status bar, alertas)
│   ├── package.json
│   ├── tsconfig.json
│   └── src/extension.ts
├── scripts/              ← build, verify, package, wpr, etl
└── src/
    ├── main.rs           ← entrada: despacha CLI o GUI según args + feature guards
    ├── cli.rs            ← CLI completa (--help, status, snapshot, wpr, kill…)
    ├── config.rs         ← configuración operativa y defaults seguros
    ├── meta.rs           ← constantes del producto (versión, autor, links)
    ├── app.rs            ← UI completa (tabs, sparklines, historial, filtros)
    ├── models.rs         ← structs compartidos + incidentes + auditoría
    ├── bin/
    │   └── rootcause-service.rs  ← skeleton Windows Service
    └── services/
        ├── ai.rs         ← adaptador IA opcional por API
        ├── inspector.rs  ← orquestador principal + get_hardware_info()
        ├── persistence.rs← SQLite + snapshots + incidentes + audit trail
        ├── rules.rs      ← rule engine ligero y correlación de incidentes
        ├── tray.rs       ← icono de bandeja (edición GUI; color por severidad)
        ├── windows.rs    ← PowerShell, WPR, toast, cmdlines
        ├── network.rs    ← netstat + clasificación
        ├── temp_scan.rs  ← temporales y cachés
        └── etl.rs        ← análisis dumpfile.xml
```

---

## 📚 Rutas de lectura recomendadas

| Perfil | Documento |
|---|---|
| 🧑‍💻 Desarrollador | [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) · [`docs/RUST_PARA_ROOTCAUSE.md`](docs/RUST_PARA_ROOTCAUSE.md) |
| 👤 Usuario final | **[`docs/MANUAL_USUARIO.md`](docs/MANUAL_USUARIO.md)** (manual claro y completo) · [`docs/MANUAL_PARA_NOVATOS.md`](docs/MANUAL_PARA_NOVATOS.md) · [`docs/OPERACION.md`](docs/OPERACION.md) |
| 🏢 Reclutador | [`docs/RECLUTADORES.md`](docs/RECLUTADORES.md) · [`docs/REPOSITORIO_ANALISIS.md`](docs/REPOSITORIO_ANALISIS.md) |
| 🔬 Arquitectura | [`docs/ARQUITECTURA_ESCALABILIDAD.md`](docs/ARQUITECTURA_ESCALABILIDAD.md) · [`docs/ARQUITECTURA_EVOLUTIVA.md`](docs/ARQUITECTURA_EVOLUTIVA.md) |
| 🛡️ Evolucion y resiliencia | [`docs/requirements/README.md`](docs/requirements/README.md) · [`docs/ROADMAP.md`](docs/ROADMAP.md) |
| 📋 Release | [`docs/RELEASE_CHECKLIST.md`](docs/RELEASE_CHECKLIST.md) · [`docs/ROADMAP.md`](docs/ROADMAP.md) |
| 📑 Todo | [`docs/INDEX.md`](docs/INDEX.md) |

---

## 🤖 IA opcional por API

RootCause no necesita IA para detectar lentitud, persistir evidencia, notificar ni operar con CLI/GUI.

Activación mínima:

```powershell
rootcause config init
$env:ROOTCAUSE_AI_API_KEY="tu_api_key"
rootcause config show
rootcause ai explain-latest
```

La configuración efectiva vive en `rootcause-config.json`. Para habilitar IA debes poner `ai.enabled = true` y definir `ai.endpoint`.

Si el proveedor IA falla:

- la captura sigue funcionando
- las alertas no se pierden
- el incidente ya persistido se conserva
- el error queda auditado

---

## 🔗 Distribución pública (demo)

Si quieres una distribución pública de evaluación separada del perfil principal, usa **RootCause Demo** como perfil alternativo de distribución:

- [`docs/DEMO_PUBLICA.md`](docs/DEMO_PUBLICA.md)
- [`docs/GUIA_DE_USO_PREVIA.md`](docs/GUIA_DE_USO_PREVIA.md)
- [`docs/LIMITACIONES_DEMO.md`](docs/LIMITACIONES_DEMO.md)
- [`docs/POLITICA_DE_PRIVACIDAD_LOCAL.md`](docs/POLITICA_DE_PRIVACIDAD_LOCAL.md)

---

## ⚠️ Limitaciones honestas

- No se entrega binario precompilado ni firmado
- El resumen ETL **no sustituye WPA** para pivoteo temporal fino o análisis de símbolos
- El escaneo TEMP es deliberadamente acotado; no indexa el disco completo
- `netstat` no equivale a un IDS ni a forense de red completa
- Detener servicios o procesos es una mitigación puntual, no una solución universal

---

## 🗺️ Nombre y marca

**RootCause** es el nombre de trabajo actual del repositorio. Se recomienda evaluar alternativas antes de publicación formal. Revisión preliminar en [`docs/NOMBRES_PRODUCTO.md`](docs/NOMBRES_PRODUCTO.md).

---

## 📄 Licencia

Apache 2.0 — ver [`LICENSE`](LICENSE) y [`docs/LICENCIA_Y_DECISION.md`](docs/LICENCIA_Y_DECISION.md).

---

## ✍️ Autor

Vladimir Acuña · [@vladimiracunadev-create](https://github.com/vladimiracunadev-create)
