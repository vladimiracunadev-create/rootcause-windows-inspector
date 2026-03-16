# RootCause — Windows Inspector

[![CI Windows](https://github.com/vladimiracunadev-create/rootcause-windows-inspector/actions/workflows/ci.yml/badge.svg)](https://github.com/vladimiracunadev-create/rootcause-windows-inspector/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-edition%202024-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Windows%2010%2F11-lightgrey.svg)](docs/REQUIREMENTS.md)
[![Version](https://img.shields.io/badge/version-0.5.0-green.svg)](CHANGELOG.md)

**RootCause** es un software de escritorio para **Windows** escrito en **Rust** y orientado a un problema real: descubrir con claridad **qué proceso, carpeta, servicio, actualización, conexión o traza ETL** está degradando el equipo cuando aparecen síntomas como:

- disco al 100%,
- crecimiento anormal de `%TEMP%`,
- lentitud general del sistema,
- consumo alto de memoria,
- actividad de red no explicada,
- actualizaciones de Windows en segundo plano,
- instaladores o binarios sospechosos lanzados desde rutas temporales.

El proyecto sigue una idea simple:

> **diagnóstico primero, intervención después**.

No intenta ser un “limpiador mágico”. Busca **explicar la causa dominante**, dejar evidencia y dar una ruta profesional hacia una precisión mayor cuando la observación liviana no basta.

---

## Inicio rápido

```powershell
# 1. Verificar entorno (Rust, WPR, Inno Setup)
.\scripts\verify-environment.ps1

# 2. Compilar
cargo build --release

# 3. Ejecutar (requiere privilegios de administrador para algunas funciones)
.\target\release\rootcause.exe
```

> Para empaquetado como ZIP portable o instalador, ver [`docs/BUILD_WINDOWS.md`](docs/BUILD_WINDOWS.md) y [`docs/PACKAGING_WINDOWS.md`](docs/PACKAGING_WINDOWS.md).

---

## Estado de entrega de este repositorio

### Este repositorio incluye

- código fuente completo en Rust,
- GUI nativa ligera con `eframe/egui`,
- scripts de verificación, build, empaquetado y análisis ETL,
- documentación profunda de arquitectura, requisitos, operación, build y packaging,
- integración de **modo de precisión WPR/ETW** desde la interfaz,
- resumen asistido del **último ETL** usando `tracerpt` y heurísticas propias.

### Este repositorio NO incluye

- `.exe` precompilado,
- instalador `.exe` listo,
- firma digital,
- servicio residente de fondo,
- driver,
- promesas de reemplazo total de WPA.

### Por qué no se entrega el `.exe`

Se deja explícitamente fuera del repositorio por razones profesionales:

- facilita auditoría del código,
- evita binarios opacos sin trazabilidad,
- reduce desalineación entre fuente y binario,
- permite compilar según tu entorno Windows real,
- deja una ruta limpia a firma digital futura y empaquetado reproducible.

La construcción del ejecutable se documenta en [`docs/BUILD_WINDOWS.md`](docs/BUILD_WINDOWS.md).

---

## Distribución pública de la demo

Para publicación en tu sitio oficial, la variante recomendada es **RootCause Demo**:

- nombre visible al usuario: `RootCause Demo`,
- instalador sugerido: `RootCause-Demo-Setup.exe`,
- binario interno esperado: `rootcause.exe`,
- experiencia de instalación transparente con textos previos, lectura posterior y accesos claros.

Documentos clave para esta etapa:

- [Distribución pública controlada](docs/DEMO_PUBLICA.md)
- [Guía de uso previa](docs/GUIA_DE_USO_PREVIA.md)
- [Limitaciones de la demo](docs/LIMITACIONES_DEMO.md)
- [Política de privacidad local](docs/POLITICA_DE_PRIVACIDAD_LOCAL.md)
- [Instalación transparente de la demo](docs/INSTALACION_TRANSPARENTE_DEMO.md)

---


## Nombre actual y criterio de marca

Por ahora, **RootCause** debe entenderse como **nombre de trabajo** del repositorio.

Se recomienda evaluar un cambio antes de publicación formal, porque algunos nombres cercanos como `RootCause`, `WinPulse` y fórmulas genéricas basadas en “One Click” ya aparecen usados en otros productos o contextos públicos.

Revisión preliminar y alternativas en [`docs/NOMBRES_PRODUCTO.md`](docs/NOMBRES_PRODUCTO.md). La ruta estricta para implementar y registrar la marca actual está en [`docs/MARCA_Y_BRANDING_ROOTCAUSE.md`](docs/MARCA_Y_BRANDING_ROOTCAUSE.md).

---

## Rutas de lectura recomendadas

### Si quieres entender el repositorio completo
- [Análisis, descripción y documentación del repositorio](docs/REPOSITORIO_ANALISIS.md)

### Si nunca has trabajado con Rust
- [Mini manual de Rust orientado a este repositorio](docs/RUST_PARA_ROOTCAUSE.md)

### Si no sabes nada de software y quieres entender el proyecto
- [Manual para novatos](docs/MANUAL_PARA_NOVATOS.md)

### Si quieres mostrar el valor del proyecto a reclutadores
- [Documento para reclutadores](docs/RECLUTADORES.md)

### Si te importa la arquitectura y cómo puede escalar
- [Arquitectura y escalabilidad del proyecto](docs/ARQUITECTURA_ESCALABILIDAD.md)

### Si quieres ver toda la documentación ordenada por perfil
- [Índice maestro de documentación](docs/INDEX.md)

### Si quieres registrar, documentar e implementar la marca RootCause
- [Marca, naming y branding técnico de RootCause](docs/MARCA_Y_BRANDING_ROOTCAUSE.md)

---

## Qué problema resuelve de verdad

RootCause intenta responder preguntas concretas que Windows no suele responder bien en una sola vista:

- **¿Qué proceso está cargando disco ahora mismo?**
- **¿Qué carpeta temporal o caché creció y cuánto?**
- **¿El problema viene de Windows Update, Delivery Optimization, BITS o SysMain?**
- **¿Qué ejecutable mantiene conexiones a IP públicas?**
- **¿Hay una ruta sospechosa en `%TEMP%` o `AppData\Local\Temp`?**
- **¿Cómo capturo evidencia más precisa sin transformar el monitor en otra carga pesada?**
- **¿Cómo puedo resumir un ETL sin abrir WPA para todo?**

---

## Capas de funcionamiento

### 1) Modo operativo liviano
Es el modo principal.

Entrega:

- semáforo general,
- top de procesos dominantes,
- presión de CPU / RAM / escritura / lectura,
- escaneo de TEMP y cachés de Windows Update,
- conexiones activas por proceso,
- eventos recientes del sistema,
- servicios relevantes,
- exportación JSON,
- historial SQLite,
- acciones controladas sobre procesos, IP y algunos servicios.

Este modo debe mantener un consumo razonable y ser útil para observación frecuente.

### 2) Modo de precisión ETW/WPR
Cuando la observación liviana no basta, el software habilita una ruta más seria:

- iniciar captura WPR desde la interfaz,
- detener y guardar ETL desde la interfaz,
- cancelar captura si fue innecesaria,
- registrar el contexto humano del problema,
- mostrar el último ETL disponible,
- permitir **resumir el último ETL** desde la propia app cuando `tracerpt` está disponible,
- dejar clara la ruta de análisis profundo posterior en WPA.

Más detalle en [`docs/PRECISION_MODE_ETW.md`](docs/PRECISION_MODE_ETW.md) y [`docs/TRACE_SUMMARY_ETL.md`](docs/TRACE_SUMMARY_ETL.md).

---

## Qué lo diferencia de un “PC Manager” genérico

RootCause no intenta vender aceleración abstracta.

Busca hacer mejor estas cosas:

- **mostrar la causa dominante antes de actuar**,
- relacionar **proceso + ruta + red + servicios + temporales**,
- marcar con **semáforo** qué merece atención inmediata,
- dejar una ruta a **captura ETW real** cuando necesitas precisión,
- entregar un **primer resumen ETL** sin depender todo el tiempo de WPA,
- permitir acciones controladas con bajo ruido visual,
- mantener una interfaz moderna pero no sobrecargada,
- consumir menos recursos que una app web pesada o un monitor invasivo.

---

## Funcionalidades implementadas

### Observación de sistema
- CPU global,
- memoria usada / total,
- red entre intervalos,
- I/O agregado entre intervalos,
- semáforo general.

### Procesos dominantes
- nombre,
- PID,
- ruta del ejecutable,
- CPU,
- RAM,
- lectura / escritura del intervalo,
- severidad y puntaje,
- categoría,
- explicación breve.

### Temporales y cachés críticas
- `%TEMP%`,
- `C:\Windows\Temp`,
- `C:\Windows\SoftwareDistribution\Download`,
- `C:\ProgramData\Microsoft\Windows\DeliveryOptimization\Cache`.

### Red
- conexiones activas por `netstat`,
- correlación con PID,
- foco en IP pública,
- bloqueo controlado por firewall.

### Contexto Windows
- `wuauserv`,
- `BITS`,
- `DoSvc`,
- `TrustedInstaller`,
- `SysMain`,
- eventos recientes warning/error.

### Intervención controlada
- finalizar procesos no protegidos,
- bloquear IP remota,
- detener temporalmente servicios permitidos (`BITS`, `DoSvc`, `SysMain`, `wuauserv`),
- iniciar / detener / cancelar captura WPR,
- resumir el último ETL capturado.

### Evidencia
- historial SQLite,
- exportación JSON,
- carpeta de trazas ETL,
- carpeta de análisis ETL (`dumpfile.xml`, `summary.txt`, `trace-analysis.json`).

---

## Interfaz

La interfaz fue pensada con cinco principios:

1. **lectura rápida**,
2. **poca fricción visual**,
3. **semáforo entendible**,
4. **acciones visibles solo donde importan**,
5. **resumen de ETL entendible sin sobrecargar la pantalla**.

### Semáforo
- **Verde**: no hay una causa fuerte dominante en esa muestra.
- **Amarillo**: hay patrón relevante o presión moderada / sostenida.
- **Rojo**: hay señal dominante o combinación crítica.

### Secciones principales
1. resumen global,
2. modo de precisión,
3. resumen del último ETL procesado,
4. dónde mirar primero,
5. procesos dominantes,
6. temporales / cachés,
7. conexiones activas,
8. servicios y eventos.

---

## Estructura principal del repositorio

```text
rootcause-windows-inspector/
├── Cargo.toml
├── README.md
├── LICENSE
├── SECURITY.md
├── docs/
├── packaging/
├── scripts/
└── src/
```

Explicación profunda en [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).

Plan de pruebas manuales en [`docs/TESTING_WINDOWS.md`](docs/TESTING_WINDOWS.md).

---

## Requisitos

Consulta el detalle profesional en [`docs/REQUIREMENTS.md`](docs/REQUIREMENTS.md).

Incluye:

- requisitos mínimos,
- requisitos recomendados,
- requisitos máximos razonables,
- requisitos de build,
- empaquetado,
- permisos,
- espacio en disco,
- límites operativos,
- perfil de uso recomendado.

---


## Validación automática y despliegue

Este repositorio ahora incluye validación visible en GitHub Actions para evitar depender de intuición o “fe” respecto a compilación y empaquetado.

Incluye:

- `ci.yml` para formato, lint, tests y build release en `windows-latest`,
- `release-windows.yml` para generar ZIP portable, instalador Inno y hashes,
- réplica local de CI con `scripts/ci-local.ps1`,
- guía completa en [`docs/CI_GITHUB.md`](docs/CI_GITHUB.md).

> Importante: la CI aumenta mucho la confianza, pero no reemplaza la prueba manual en un Windows real con WPR/WPA instalados y permisos adecuados.

---

## Cómo construir el `.exe` después

### Comando mínimo

```powershell
cargo build --release
```

### Salida esperada

```text
target\release\rootcause.exe
```

### Script recomendado

```powershell
.\scripts\build-release.ps1
```

Guía completa en [`docs/BUILD_WINDOWS.md`](docs/BUILD_WINDOWS.md).

---

## Cómo generar paquetes de instalación

### Portable ZIP

```powershell
.\scripts\package-portable.ps1
```

### Instalador Inno Setup

```powershell
.\scripts\package-inno.ps1
```

Detalle completo en [`docs/PACKAGING_WINDOWS.md`](docs/PACKAGING_WINDOWS.md).

---

## Cómo usar la precisión ETW

### Captura desde la interfaz
1. escribe una descripción corta del problema,
2. pulsa **Iniciar captura WPR**,
3. reproduce el síntoma,
4. pulsa **Detener y guardar ETL**,
5. pulsa **Resumir último ETL** si `tracerpt` está disponible.

### Captura por scripts

```powershell
.\scripts\wpr-start-general.ps1 -ProblemDescription "Disco al 100% durante actualización"
.\scripts\wpr-stop-general.ps1 -ProblemDescription "Disco al 100% durante actualización"
.\scripts\analyze-last-etl.ps1
```

---

## Limitaciones honestas

- No se entrega binario precompilado.
- El resumen automático de ETL **no sustituye WPA** para pivoteo temporal fino, stacks o análisis profundo por símbolo.
- El escaneo TEMP es deliberadamente acotado; no indexa el disco completo.
- `netstat` no equivale a un IDS ni a una forénsica de red completa.
- Detener servicios o procesos es una intervención controlada, no una solución universal.

---

## Ruta profesional recomendada

1. verificar entorno,
2. compilar localmente en Windows,
3. correr quality gates,
4. usar modo liviano para ubicar sospechosos,
5. activar WPR solo si hace falta,
6. guardar ETL,
7. resumir ETL desde la app,
8. abrir WPA si el caso exige precisión más fina,
9. empaquetar ZIP portable o instalador,
10. firmar digitalmente en una fase posterior si el proyecto pasa a distribución formal.

---

## Contribuir

1. Lee [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) antes de tocar el código.
2. Corre los quality gates locales antes de proponer cambios:
   ```powershell
   .\scripts\quality-gates.ps1
   ```
3. Abre un issue describiendo el problema o mejora antes de un PR.
4. El CI en GitHub Actions valida formato, lint, tests y build release en `windows-latest`.

---

## Licencia

MIT — ver [`LICENSE`](LICENSE).

---

## Autor

Vladimir Acuña · [@vladimiracunadev-create](https://github.com/vladimiracunadev-create)
