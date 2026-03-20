# Plan Maestro — RootCause Windows Inspector

**Versión base:** v0.8.0 · **Actualizado:** 2026-03-20
**Propósito:** hoja de ruta completa del producto — qué mejorar, en qué orden, con qué ediciones y hacia dónde escalar. Diseñado para retomar el trabajo en cualquier sesión sin perder contexto.

> **Al iniciar sesión:** leer este documento antes de cualquier acción.
> Estado del entorno: CI debe estar verde — v0.7 completada con múltiples ediciones del producto.

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
| No monitorea entradas de autostart/registro | → Fase 4: Tab Autostart |
| Sin firma digital → SmartScreen en primera ejecución | → Fase 6: firma digital |
| Solo 1 test unitario | → Fase 3: tests para funciones críticas |
| Sin configuración de umbrales por el usuario | → Fase 4: archivo `rootcause.toml` |
| Binario único — no hay opción ligera para sysadmins | → Fase 2: edición CLI-only |

---

## II. Estado técnico — v0.7 entregada

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

#### 4.1 Tab Autostart (mayor diferenciador vs competencia)
Qué hace: muestra qué arranca con Windows — entradas de registro `HKEY_CURRENT_USER\...\Run`, carpeta Startup, tareas programadas.
Por qué primero: ninguna herramienta gratuita equivalente lo tiene integrado con diagnóstico. Diferenciador fuerte.
Implementación: PowerShell via `Command::spawn()` (ya existe el patrón) + nuevo tab. Cero crates nuevos.

#### 4.2 Tray icon (monitor proactivo)
Qué hace: ícono en bandeja del sistema. Cambia de color (verde/amarillo/rojo) según severidad. Click abre la ventana completa. El programa corre en segundo plano sin que el usuario lo vea.
Por qué importa: transforma RootCause de herramienta reactiva (abro cuando hay problema) a monitor proactivo (me avisa cuando hay problema).
Implementación: requiere actualizar eframe a 0.28+ que incluye soporte de tray nativo.

#### 4.3 Alertas y umbrales configurables
Qué hace: panel de configuración donde el usuario ajusta los umbrales de CPU/RAM/IO, el intervalo de refresco y las notificaciones. Guardar en `rootcause.toml` en AppData.
Por qué importa: distintos equipos tienen distintas líneas base. Un servidor con 80% CPU es normal; un desktop con 80% es alerta.

#### 4.4 `--output` en CLI
Qué hace: `rootcause snapshot --output diagnostico.json` además de stdout.
Implementación: cambio mínimo en `cli.rs`. Una línea.

---

### PRIORIDAD 5 — Distribución pública

**Por qué:** el producto puede ser descubierto y usado si está en los canales correctos.

#### 5.1 Auto-publicar releases en landing (paso crítico, pendiente)
Los manifests están listos (Scoop, Winget, Chocolatey). Falta conectar el CI:

```
CI privado (build .exe) → gh release create --repo rootcause-landing → descarga pública
```

Requiere: secret `LANDING_RELEASE_TOKEN` (PAT con permisos al repo landing) en el repo privado.
Resultado: los botones de descarga de la landing funcionan con cada nuevo release.
Los campos `UPDATE_SHA256_ON_RELEASE` en los manifests se rellenan con el hash real del binario.

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

```
v0.6  ✅   v0.7 ✅               v1.0 ⏳               v2.0+ ⏳
──────────  ────────────────────  ──────────────────── ──────────────────
GUI+CLI+    CLI-only binary ✅    Tab Autostart         Windows Service
SQLite+     PowerShell module ✅  Tray icon activo      Edición Seguridad
historial   Scoop/Winget/Choco✅  Scoop/Winget publis.  Edición Enterprise
            VS Code Extension ✅  Firma digital         MSIX / Store
            Tray skeleton ✅      Alertas config.
            Service skeleton ✅   EMAIL+GitLab meta.rs
            SQLite retención ✅
            JSON backup ✅
```

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
5. git commit (mensaje en español + Co-Authored-By Claude Sonnet 4.6)
6. git push origin master
7. Si cambia algo visible para el usuario → actualizar landing
```

### Flujo para actualizar landing

```
1. Editar C:\dev\rootcause-landing\index.html
2. git add index.html && git commit -m "chore: actualizar landing vX.Y.Z"
3. git push origin main  → GitHub Pages redespliega en ~60s
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
☐ rootcause-landing/index.html — versión + features
☐ run_fmt.ps1 sin errores · CI verde
☐ git tag -a vX.Y.Z && git push origin vX.Y.Z
☐ Publicar binarios en rootcause-landing/releases
☐ Verificar landing en browser
```

---

*Al modificar el producto de forma significativa, actualizar las secciones afectadas de este documento y hacer commit.*
