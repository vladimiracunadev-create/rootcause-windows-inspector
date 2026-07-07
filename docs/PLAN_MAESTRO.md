# Plan Maestro — RootCause Windows Inspector

**Versión base:** v0.13.0 · **Actualizado:** 2026-07-07
**Propósito:** hoja de ruta completa del producto — qué mejorar, en qué orden, con qué ediciones y hacia dónde escalar. Diseñado para retomar el trabajo en cualquier sesión sin perder contexto.

> **Al iniciar sesión:** leer este documento antes de cualquier acción.
> Estado del entorno: CI debe estar verde — v0.13.0 completada. Próximo objetivo: v1.0 (tray icon, firma digital, distribución pública).

---

## I. Posicionamiento actual del producto

RootCause ocupa un nicho específico que ninguna herramienta gratuita cubre bien:

```
Process Monitor   → ¿qué está pasando exactamente? (nivel syscall)
Task Manager      → ¿qué proceso usa más CPU? (básico)
RootCause         → ¿cuál es la CAUSA RAÍZ? (diagnóstico interpretado + acción)
```

### Ventajas reales sobre la competencia

| Ventaja | Herramientas que no lo tienen |
|---|---|
| Scoring heurístico automático — dice por qué, no solo qué | Todas |
| CLI nativa completa (`status`, `block-ip`, `wpr`, `kill`…) | Todas (gratuitas) |
| ETL/WPR integrado — sin abrir cmd ni instalar WPA | Todas |
| Historial SQLite + comparación A/B | Todas (gratuitas) |
| Cero telemetría verificable en Cargo.toml | PC Manager, GlassWire |
| Código auditable | PC Manager, GlassWire, Task Manager |

### Desventajas honestas a resolver

| Desventaja | Plan de ataque |
|---|---|
| ~~No monitorea entradas de autostart/registro~~ ✅ **Resuelto** — Tab Autostart + tareas programadas (v0.11) + detección de cambios contra baseline (v0.12) | Entregado |
| Sin firma digital → SmartScreen en primera ejecución | → Fase 6: firma digital |
| Solo 1 test unitario | → Fase 3: tests para funciones críticas |
| Sin configuración de umbrales por el usuario | → Fase 4: archivo `rootcause.toml` |
| Binario único — no hay opción ligera para sysadmins | → Fase 2: edición CLI-only |

---

## II. Estado técnico — hito v0.7 (snapshot histórico)

> Nota: esta sección es un registro del hito v0.7. El estado actual del producto es
> v0.13.0 (ver sección IV "Mapa de versiones" y `docs/ROADMAP.md` para lo entregado
> hasta hoy: detección de anomalías, baseline de autoarranque y de servicios).

### ✅ Completado en v0.7
- Feature flags GUI/CLI-only en `Cargo.toml` (eframe/egui opcionales)
- `#[cfg(feature = "gui")]` en `main.rs` — compilación limpia en ambas ediciones
- SQLite retención automática (últimas 1000 filas)
- Backup JSON automático al exportar snapshot
- Módulo PowerShell `RootCause.psm1` (9 cmdlets)
- Manifests Scoop, Winget (`VladimirAcuna.RootCause`), Chocolatey
- VS Code Extension completa (`package.json` + `extension.ts`)
- Skeleton tray icon documentado (`src/services/tray.rs`)
- Skeleton Windows Service (`src/bin/rootcause-service.rs`)
- Documentación actualizada en todos los docs clave

### Metadatos pendientes de confirmar

| Campo | Archivo | Estado |
|---|---|---|
| `EMAIL` | `src/meta.rs:20` | Vacío — pendiente confirmar con usuario |
| `GITLAB` URL | `src/meta.rs:27` | Pendiente verificar URL correcta |

### Problema de entorno local (vigente)
bash/MSYS2 usa `link.exe` incorrecto. Siempre usar `run_fmt.ps1` para formato.
`cargo check` local puede fallar por linker — CI en GitHub Actions siempre funciona.

---

## III. Cómo mejorar el producto — por prioridad

---

### PRIORIDAD 1 — Completar lo pendiente (v0.6 + v0.7 resumen)

✅ **Completado:**
- Fix CI (2 errores clippy) → CI verde
- Hardware info en UI → tab Overview muestra OS, CPU, RAM, arquitectura
- Atajos de teclado activos → F5, Ctrl+E, Ctrl+1…8
- Tab Acerca completo → atajos + info del producto
- Feature flags CLI-only → `--no-default-features` produce ~4 MB
- SQLite retención + backup JSON automático
- PowerShell module (9 cmdlets), Scoop/Winget/Chocolatey manifests
- VS Code Extension completa
- Skeletons tray + Windows Service

⏳ **Pendiente:**
1. **EMAIL y GitLab confirmados** → `src/meta.rs` completo (bloquea distribución)
2. **Telemetría en landing corregida** → "Sin telemetría activa" → "Telemetría: cero"

---

### PRIORIDAD 2 — Ediciones del producto ✅ Completada en v0.7

**Estado actual — todas las ediciones implementadas:**

| Edición | Estado | Tamaño | Artefacto |
|---|---|---|---|
| GUI .exe | ✅ Producción | ~18 MB | `cargo build --release` |
| CLI-only .exe | ✅ Producción | ~4 MB | `cargo build --release --no-default-features` |
| PowerShell module | ✅ Producción | ~1 KB | `packaging/powershell/RootCause.psm1` |
| VS Code Extension | ✅ Producción | TypeScript | `vscode-extension/` |
| Tray icon | ⚙ Skeleton | — | `src/services/tray.rs` (activar feature `tray`) |
| Windows Service | ⚙ Skeleton | — | `src/bin/rootcause-service.rs` (activar feature `service`) |

**Para activar Tray icon** (próxima versión):
```toml
# Cargo.toml
tray = ["dep:tray-icon"]
[dependencies]
tray-icon = { version = "0.14", optional = true }
```

**Para activar Windows Service** (próxima versión):
```toml
service = ["dep:windows-service"]
[dependencies]
windows-service = { version = "0.6", optional = true }
```

---

### PRIORIDAD 3 — Calidad de código (deuda técnica)

**Por qué:** resolver la deuda ahora evita que se acumule y complique las fases siguientes.

| Tarea | Impacto | Esfuerzo |
|---|---|---|
| Consolidar `is_public_ip()` duplicada en `network.rs`+`etl.rs` | Elimina riesgo de divergencia | Bajo |
| Constantes para umbrales (`CPU_HIGH_PCT`, `RAM_HIGH_MB`, etc.) | Código legible, cambios en un lugar | Bajo |
| Fix `.expect("regex válida")` en `etl.rs:372` → `OnceLock` | Elimina único panic potencial | Bajo |
| Limpiar baselines I/O de PIDs muertos | Evita crecimiento de HashMap | Bajo |
| Tests unitarios para funciones críticas | Detectar regresiones en releases | Medio |
| Dividir `app.rs` (2,973 líneas) en submódulos | Mantenibilidad a largo plazo | Medio |
| Retención en SQLite (mantener solo últimas 1000 filas) | Evita crecimiento indefinido de la DB | Bajo |

#### Decisión pendiente: SQLite vs JSON

| Criterio | SQLite | JSON |
|---|---|---|
| Ahorro en binario | — | ~1 MB (elimina rusqlite) |
| Consultas historial | ✅ SQL nativo | ❌ Carga todo en memoria |
| Comparación A/B | ✅ Trivial | ⚠️ Requiere iterar |
| Integridad ante crash | ✅ ACID | ⚠️ Posible truncado |

**Recomendación:** si se mantiene la comparación A/B → SQLite. Si se simplifica → JSON ahorra 1 MB.
**Decisión: pendiente confirmación del usuario.**

---

### PRIORIDAD 4 — Features nuevas de producto

Ordenadas por impacto real para el usuario:

#### 4.1 Tab Autostart ✅ Completado en v0.11.0
Qué hace: muestra qué arranca con Windows — entradas de registro `HKCU/HKLM\...\Run` y carpetas Startup.
Dónde: tab "Autostart" (Ctrl+7), 9 tabs totales ahora.
Implementación: `windows::persistence_entries()` (PowerShell) → `snap.persistence_entries` → `draw_tab_autostart()`.
Datos: nombre, tipo de origen (pill diferenciado), comando completo con tooltip, indicador de existencia en disco, severidad heurística.

#### 4.2 Tareas programadas en Autostart ✅ Completado en v0.11.0
`Get-ScheduledTask` (excluye `\Microsoft\*` y deshabilitadas) → tipo "Scheduled Task" en `persistence_entries`.
Pill amarillo en UI, CLI muestra "Tarea programada". Nota contextual al pie por tipo.
Próximo: filtrar también por `TaskPath` raíz no-Microsoft para reducir ruido.

#### 4.2b Detección de cambios de autoarranque (baseline SQLite) ✅ Completado en v0.12.0
Qué hace: da control explícito para saber si cambian los puntos de autoarranque de Windows (Registro Run/RunOnce HKCU/HKLM, carpetas Startup, tareas programadas no-Microsoft).
Cómo: baseline persistida en SQLite (tabla `persistence_baseline`). La primera foto se toma como estado bueno conocido (silenciosa, sin alertas). En escaneos posteriores cada entrada se clasifica **NUEVA / MODIFICADA / ELIMINADA** contra la baseline; los cambios son pegajosos hasta que el usuario los acepte.
Alertas: genera alertas kind `persistence-change` — **Alta** para entradas nuevas/modificadas, **Media** para eliminadas.
Aceptación: botón en el tab Autostart ("✓ Aceptar estado actual como baseline") o CLI `rootcause autostart --accept`. `rootcause autostart --json` incluye el campo `change_status` por entrada.

#### 4.2c Detección de cambios en servicios (motor genérico de baseline) ✅ Completado en v0.13.0
Qué hace: vigila todos los servicios de Windows y detecta si cambian contra una baseline conocida — servicio **NUEVO / MODIFICADO / ELIMINADO**. El valor vigilado es **StartMode + ruta del binario** (un cambio de modo de arranque o de binario cuenta como MODIFICADO).
Motor genérico: v0.13 generaliza el patrón de autostart de v0.12 en un **motor de baseline reutilizable**. La lógica de "primera foto = estado bueno conocido → clasificar cambios → aceptar baseline" deja de estar acoplada a autoarranque y pasa a ser un mecanismo común. Futuras superficies (archivo `hosts`, claves de registro, tareas) se añaden barato sobre este motor.
Alertas: genera alertas kind `service-change` para servicios nuevos/modificados/eliminados.
Aceptación y listado: CLI `rootcause services --accept` fija la baseline. `rootcause services` lista solo los cambios; `rootcause services --json` incluye el campo `change_status` por servicio.

#### 4.3 Tray icon (monitor proactivo)
Qué hace: ícono en bandeja del sistema. Cambia de color (verde/amarillo/rojo) según severidad. Click abre la ventana completa. El programa corre en segundo plano sin que el usuario lo vea.
Por qué importa: transforma RootCause de herramienta reactiva (abro cuando hay problema) a monitor proactivo (me avisa cuando hay problema).
Implementación: requiere actualizar eframe a 0.28+ que incluye soporte de tray nativo.

#### 4.4 Alertas y umbrales configurables ✅ v0.11.0
Panel de configuración en tab Acerca con edición inline de umbrales (CPU, RAM, I/O, anomalías, refresco) y botón Guardar que persiste a `rootcause-config.json` sin reiniciar.
`save_config(&mut self, config)` implementado en `InspectorService` y `ConfigManager::save_to_path()` en `config.rs`.

#### 4.5 `--output` en CLI ✅ Implementado
Qué hace: `rootcause snapshot --output diagnostico.json` además de stdout.
Implementación: cambio mínimo en `cli.rs`. Una línea.

---

### PRIORIDAD 5 — Distribución pública

**Por qué:** el producto puede ser descubierto y usado si está en los canales correctos.

#### 5.1 Auto-publicar releases ✅ Resuelto
El repo es público y el pipeline ya publica releases automáticamente: al pushear un tag `vX.Y.Z`,
`release-windows.yml` compila los 6 artefactos y crea el GitHub Release en este mismo repo.

```
git tag vX.Y.Z → release-windows.yml (build + package) → GitHub Release público
```

La landing vive en `landing/` dentro de este repo (GitHub Pages vía `deploy-landing.yml`) y sus
botones de descarga apuntan a `releases/latest/download/`, así que funcionan con cada nuevo release
sin pasos manuales. El antiguo repo separado `rootcause-landing` y el secret `LANDING_RELEASE_TOKEN`
quedaron obsoletos (ver nota histórica en `docs/CI_GITHUB.md`).
Pendiente real: rellenar los campos `UPDATE_SHA256_ON_RELEASE` de los manifests con el hash real al
publicar en cada gestor (Scoop/Winget/Chocolatey).

#### 5.2 Gestores de paquetes Windows ✅ Manifests creados

| Canal | Manifest | Pendiente |
|---|---|---|
| **Scoop** | `packaging/distribution/scoop/rootcause.json` | PR en bucket scoop-extras o propio |
| **Winget** | `packaging/distribution/winget/rootcause.yaml` | PR en microsoft/winget-pkgs |
| **Chocolatey** | `packaging/chocolatey/rootcause.nuspec` | Cuenta Chocolatey + push |

Prerequisito para los tres: primer release público con binarios disponibles.

#### 5.3 Firma digital
Elimina la alerta SmartScreen. Opciones:
- Self-signed (gratis, reduce pero no elimina el alerta)
- CodeSigning cert comercial (~$70–200/año) — elimina completamente el alerta

---

### PRIORIDAD 6 — Largo plazo (v2.0+)

En orden de impacto potencial:

| Versión | Descripción | Complejidad |
|---|---|---|
| **Windows Service** | Corre sin usuario logueado. Historial continuo 24/7. GUI se conecta via named pipes. Permite diagnosticar problemas nocturnos. | Muy alta |
| **VS Code Extension** | Barra de estado con severidad del sistema mientras programas. Click abre panel de alertas. TypeScript wrapper sobre `rootcause status --json`. | Media |
| **Edición Seguridad** | Solo procesos sospechosos + conexiones + bloqueo. Orientada a SOC y respuesta a incidentes. Feature flags — mismo código. | Media |
| **Edición Enterprise** | Prometheus/Grafana, multi-equipo, GPO, CSV/Excel. Modelo B2B. | Muy alta |
| **MSIX / Microsoft Store** | Distribución oficial Microsoft. Requiere cuenta de desarrollador + firma. | Alta |

---

## IV. Mapa de versiones del producto

Entregado (coincide con los tags/releases publicados en GitHub):

- **v0.5** — primer release: GUI + CLI, SQLite + historial, integración WPR/ETW, parser resumido de ETL, CI en GitHub Actions + pipeline de release (ZIP/Inno/hashes)
- **v0.6** — sparklines CPU/RAM/IO, tab Historial con comparación A vs B, filtro de severidad, notificaciones toast, tab Acerca, landing inicial
- **v0.7** — feature flags GUI/CLI-only, módulo PowerShell, manifests Scoop/Winget/Chocolatey, extensión VS Code, skeletons tray/service, retención SQLite + backup JSON
- **v0.8** — módulo V1 de detección de comportamiento anómalo, incidentes correlacionados con evidencia, adaptador IA opcional desacoplado
- **v0.9** — salud del agente (heartbeat, recuperación tras cierre abrupto, backoff, integridad básica de configuración)
- **v0.11** — tab Autostart (Ctrl+7: Run HKCU/HKLM + Startup) y tareas programadas, CLI `autostart`, umbrales editables inline (`save_config`), UI profesional (RAM pbar real, Ctrl+1..9)
- **v0.12** — detección de cambios de autoarranque vs baseline (NUEVA/MODIFICADA/ELIMINADA), alertas `persistence-change`, aceptar baseline (botón UI + `rootcause autostart --accept`)
- **v0.13** — detección de cambios en servicios de Windows vs baseline (NUEVO/MODIFICADO/ELIMINADO por StartMode + ruta del binario) sobre un motor genérico de baseline reutilizable, alertas `service-change`, `rootcause services [--json] [--accept]`

Pendiente:

- **v1.0** ⏳ — tray icon activo, firma digital, publicación real en Scoop/Winget/Chocolatey, EMAIL + GitLab en `meta.rs`
- **v2.0+** ⏳ — Windows Service 24/7, edición Seguridad (SOC), edición Enterprise, MSIX / Microsoft Store

---

## V. Skills del proyecto

| Skill | Propósito |
|---|---|
| `rootcause-improve` | Todo el desarrollo — features, CI, docs, releases, landing |
| `rootcause-rename` | Cambiar el nombre del producto en **todos** los archivos (src, docs, Cargo.toml, landing, CI, skills) — con listado de archivos afectados y confirmación antes de actuar |

---

## VI. Reglas de trabajo por sesión

### Al iniciar

```
1. git log --oneline -3       → ¿en qué commit estamos?
2. CI en GitHub Actions       → ¿verde o rojo?
3. Si rojo → Sección II (bloqueantes CI) antes que nada
4. Revisar sección "Decisiones pendientes del usuario" (sección II)
```

### Flujo para cambios Rust

```
1. Read tool antes de editar — siempre
2. Edit tool para cambios
3. powershell.exe -ExecutionPolicy Bypass -File run_fmt.ps1
4. git add <archivos específicos>
5. git commit (mensaje en español + Co-Authored-By Claude)
6. git push origin master
7. Si cambia algo visible para el usuario → actualizar landing
```

### Flujo para actualizar landing

```
1. Editar landing/index.html (en este repo)
2. git add landing/index.html && git commit -m "chore: actualizar landing vX.Y.Z"
3. git push origin master  → deploy-landing.yml redespliega GitHub Pages en ~60s
```

### Problema de entorno local (CRÍTICO)

bash/MSYS2 usa el `link.exe` incorrecto. **Nunca correr `cargo` directo en bash.**
Siempre usar PowerShell:
- fmt: `powershell.exe -ExecutionPolicy Bypass -File run_fmt.ps1`
- check/clippy: `powershell.exe -ExecutionPolicy Bypass -File run_check.ps1`

CI en GitHub usa `windows-latest` con el toolchain correcto — siempre compila bien aunque falle local.

### Checklist de release

```
☐ Bump versión en Cargo.toml
☐ README.md — badge versión
☐ ROADMAP.md — ítem ✅
☐ ARCHITECTURE.md — módulos nuevos
☐ OPERACION.md — acciones nuevas para el usuario
☐ COMMANDS.md — comandos CLI nuevos
☐ RECLUTADORES.md — features nuevas
☐ INDEX.md — docs nuevos enlazados
☐ landing/index.html — versión + features
☐ run_fmt.ps1 sin errores · CI verde
☐ git tag -a vX.Y.Z && git push origin vX.Y.Z  → release-windows.yml publica los binarios
☐ Verificar release: gh release view vX.Y.Z --json assets (6 artefactos, KB > 0)
☐ Verificar landing en browser
```

---

*Al modificar el producto de forma significativa, actualizar las secciones afectadas de este documento y hacer commit.*
