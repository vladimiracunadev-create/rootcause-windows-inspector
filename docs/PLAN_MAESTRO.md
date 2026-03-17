# Plan Maestro — RootCause Windows Inspector

**Versión base:** v0.6.0 · **Actualizado:** 2026-03-17
**Propósito:** hoja de ruta completa del producto — qué mejorar, en qué orden, con qué ediciones y hacia dónde escalar. Diseñado para retomar el trabajo en cualquier sesión sin perder contexto.

> **Al iniciar sesión:** leer este documento antes de cualquier acción.
> Estado del entorno: CI tiene **2 errores clippy bloqueantes** — resolver FASE 0 primero.

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

## II. Estado técnico — lo que hay que resolver ya

### Bloqueantes CI (resolver antes de cualquier otra cosa)

| # | Error | Archivo | Fix exacto |
|---|---|---|---|
| 1 | `collapsible_if` | `src/app.rs:402` | `if let Some(idx) = tab_switch && let Some(&(tab,_,_)) = Tab::ALL.get(idx) { ... }` |
| 2 | `print_literal` | `src/cli.rs:218` | Mover literal `"Proceso dominante"` dentro del string de formato |

Después de corregir: `run_fmt.ps1` → push → verificar CI verde.

### Código pendiente de v0.6 (ya diseñado, falta conectar)

`get_hardware_info()` existe en `inspector.rs`. Falta:
- Campo `hardware_info: HardwareInfo` en `RootCauseApp`
- Poblar en `new()` con `insp.get_hardware_info()`
- Sección hardware en tab Overview
- Atajos de teclado en `update()` (patrón collect-then-execute para evitar borrow conflict)
- Sección atajos en `draw_tab_about()`

### Metadatos pendientes de confirmar (bloquean la distribución)

| Campo | Archivo | Estado |
|---|---|---|
| `EMAIL` | `src/meta.rs:20` | Vacío — pendiente confirmar |
| `GITLAB` URL | `src/meta.rs:27` | Pendiente verificar |

---

## III. Cómo mejorar el producto — por prioridad

---

### PRIORIDAD 1 — Completar lo que está a medio hacer

**Por qué primero:** hay features diseñadas y parcialmente implementadas. Terminarlas antes de empezar nuevas es más eficiente y cierra v0.6 limpiamente.

1. **Fix CI** (2 errores clippy) → CI verde
2. **Hardware info en UI** → tab Overview muestra OS, CPU, RAM, arquitectura
3. **Atajos de teclado activos** → F5, Ctrl+E, Ctrl+1…8
4. **Tab Acerca completo** → atajos documentados + info del producto
5. **Telemetría en landing corregida** → "Sin telemetría activa" → "Telemetría: cero"
6. **EMAIL y GitLab confirmados** → `src/meta.rs` completo

---

### PRIORIDAD 2 — Tres ediciones del producto

**Por qué:** el producto actual es bueno pero solo existe en una forma. Tres ediciones multiplican el alcance sin reescribir nada.

#### Edición 1 — GUI completa (ya existe)
`rootcause.exe` · ~18 MB · ventana gráfica + CLI combinadas.

#### Edición 2 — CLI-only (~4 MB)
**Impacto real:** sysadmins, scripts de automatización, Windows Server sin escritorio, integración en pipelines CI, distribución por gestores de paquetes.

Implementación con **feature flags de Rust** — un solo cambio en Cargo.toml:

```toml
[features]
default = ["gui"]
gui = ["eframe", "egui"]

[dependencies]
eframe = { version = "0.27", optional = true }
egui   = { version = "0.27", optional = true }
```

```bash
cargo build --release --no-default-features   # → ~4 MB, sin egui
```

Ahorro real: ~13 MB (egui + fuentes eliminados). Riesgo: ninguno — `cli.rs` ya es independiente de `app.rs`.

#### Edición 3 — Módulo PowerShell
**Impacto real:** los sysadmins de empresas viven en PowerShell. Un módulo que se integra en sus scripts vale más que una app nueva.

```powershell
Import-Module RootCause
Get-RootCauseStatus                              # objeto PS con Severity, CPU, RAM
Get-RootCauseProcesses | Where-Object Severity -eq "Critical"
Invoke-RootCauseExport -Path "C:\diag\snap.json"
```

Implementación: archivo `RootCause.psm1` que llama a `rootcause.exe` y convierte el JSON de salida en objetos PowerShell. **Cero cambios en Rust.**

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

#### 5.1 Auto-publicar releases en landing (paso crítico)
Actualmente los binarios no se publican automáticamente en el repo público. El flujo correcto:

```
CI privado (build .exe) → gh release create --repo rootcause-landing → descarga pública
```

Requiere: secret `LANDING_RELEASE_TOKEN` (PAT con permisos al repo landing) en el repo privado.
Resultado: los botones de descarga de la landing funcionan con cada nuevo release.

#### 5.2 Gestores de paquetes Windows

| Canal | Comando de instalación | Esfuerzo | Audiencia |
|---|---|---|---|
| **Scoop** | `scoop install rootcause` | Bajo — un JSON | Desarrolladores |
| **Winget** | `winget install rootcause` | Bajo — un YAML | Usuarios técnicos |
| **Chocolatey** | `choco install rootcause` | Bajo — `.nuspec` | Sysadmins enterprise |

Prerequisito: tener al menos un release público disponible.

#### 5.3 Firma digital
Elimina la alerta SmartScreen en primera ejecución. Opción gratuita: self-signed cert (reduce el alerta pero no lo elimina). Opción real: CodeSigning cert comercial (~$70–200/año).

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
v0.6  →  v0.7              →  v1.0                →  v2.0+
──────    ─────────────────    ─────────────────────    ──────────────────
Actual    CLI-only binary      Tab Autostart            Windows Service
          PowerShell module    Tray icon                VS Code Extension
          Fix CI + deuda       Scoop / Winget           Edición Seguridad
          Tab Acerca completo  Firma digital            Enterprise / Store
          Hardware en UI       Alertas configurables
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
