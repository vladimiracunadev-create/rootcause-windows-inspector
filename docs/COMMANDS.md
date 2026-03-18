# Comandos y scripts

Este documento centraliza la referencia de comandos del proyecto.

---

## 1) Verificación de entorno

### PowerShell

```powershell
.\scripts\verify-environment.ps1
```

### Batch

```bat
scripts\verify-environment.bat
```

Verifica, idealmente:

- `cargo`
- `rustup`
- `cl`
- `powershell`
- `wpr`
- `tracerpt`
- `wpa`
- `iscc`

---

## 2) Build

### Edición GUI completa (por defecto, ~18 MB)

```powershell
cargo build --release
```

### Edición CLI-only (~4 MB, sin egui, sin interfaz gráfica)

```powershell
cargo build --release --no-default-features
# Produce: target\release\rootcause.exe — ideal para scripts, Server Core, pipelines CI
```

### Build release recomendado (con scripts)

```powershell
.\scripts\build-release.ps1
```

### Build release batch

```bat
scripts\build-release.bat
```

### Ejecutar en desarrollo

```powershell
cargo run
```

---

## 3) Calidad

```powershell
cargo check --all-targets
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features -- --nocapture
.\scripts\quality-gates.ps1
.\scripts\ci-local.ps1
```

---

## 4) Limpieza

```powershell
.\scripts\clean.ps1
```

---

## 5) Exportación y artefactos

### Portable ZIP

```powershell
.\scripts\package-portable.ps1
```

### Instalador Inno Setup

```powershell
.\scripts\package-inno.ps1
```

### Hash de artefactos

```powershell
.\scripts\hash-artifacts.ps1
```

---

## 6) Modo de precisión WPR

### Inicio rápido por script

```powershell
.\scripts\wpr-start-general.ps1 -ProblemDescription "Disco al 100% durante actualización"
```

### Detención y guardado

```powershell
.\scripts\wpr-stop-general.ps1 -ProblemDescription "Disco al 100% durante actualización"
```

### Abrir el ETL más reciente en WPA

```powershell
.\scripts\wpa-open-latest.ps1
```

---

## 7) Resumen ETL automatizado

### Exportar último ETL a XML + summary

```powershell
.\scripts\analyze-last-etl.ps1
```

### Exportar un ETL específico

```powershell
.\scripts\analyze-last-etl.ps1 -EtlPath "C:\ruta\problema.etl"
```

### En batch

```bat
scripts\analyze-last-etl.bat
```

---

## 8) WPR manual

### Ver perfiles

```powershell
wpr -profiles
```

### Estado actual

```powershell
wpr -status
```

### Iniciar captura general

```powershell
wpr -start GeneralProfile -filemode
```

### Detener y guardar ETL

```powershell
wpr -stop C:\ruta\archivo.etl "Descripción del problema" -skipPdbGen -compress
```

### Cancelar captura

```powershell
wpr -cancel
```

---

## 9) tracerpt manual

### Exportar ETL a XML y summary

```powershell
tracerpt C:\ruta\archivo.etl -o C:\ruta\dumpfile.xml -of XML -lr -summary C:\ruta\summary.txt
```

### Exportar en CSV cuando quieras inspección plana

```powershell
tracerpt C:\ruta\archivo.etl -o C:\ruta\dumpfile.csv -of CSV -lr -summary C:\ruta\summary.txt
```

---

## 10) Inno Setup manual

```powershell
iscc .\packaging\windows\RootCause.iss
```

---

## 11) Secuencia recomendada de release

```powershell
.\scripts\verify-environment.ps1
.\scripts\quality-gates.ps1
.\scripts\build-release.ps1
.\scripts\package-portable.ps1
.\scripts\package-inno.ps1
.\scripts\hash-artifacts.ps1
```


## 12) GitHub Actions

### Validación continua

```text
.github/workflows/ci.yml
```

### Empaquetado de release

```text
.github/workflows/release-windows.yml
```


## Branding y recursos

### Verificar recursos visuales
```powershell
dir .\assets
```

### Compilar release con icono embebido
```powershell
cargo build --release
```

### Empaquetar instalador con nombre y accesos RootCause
```powershell
.\scripts\package-inno.ps1
```


## CLI del producto (rootcause.exe)

El binario instalado funciona como herramienta de consola. Todos los comandos funcionan sin GUI.

### Ayuda y versión

```
rootcause --help
rootcause --version
```

### Estado del sistema

```
rootcause status
```
Imprime severidad, CPU, RAM, I/O y alertas activas.

### Snapshot JSON

```
rootcause snapshot
```
Vuelca la instantánea completa a stdout en formato JSON.

### Historial

```
rootcause history
rootcause history 20
```
Muestra las últimas N filas del historial SQLite (default: 10).

### Exportar

```
rootcause export
```
Guarda snapshot a JSON en la carpeta Descargas o Documentos.

### Captura ETL

```
rootcause wpr start [--note "descripción"]
rootcause wpr stop  [--note "descripción"]
rootcause wpr cancel
rootcause wpr analyze
```

### Acciones de intervención

```
rootcause kill <PID>
rootcause block-ip <IP>
rootcause stop-service <nombre>
```

### Forzar modo GUI

```
rootcause --gui
```

---

## Módulo PowerShell

```powershell
# Importar el módulo
Import-Module .\packaging\powershell\RootCause.psm1

# Ver estado del sistema como objeto PowerShell
Get-RootCauseStatus

# Procesos con filtro de severidad
Get-RootCauseProcesses -MinSeverity "Warning"

# Historial de capturas
Get-RootCauseHistory -Count 20

# Exportar snapshot a archivo
Invoke-RootCauseExport -Path "C:\diag\snapshot.json"

# Terminar proceso por PID
Stop-RootCauseProcess -Pid 1234

# Bloquear IP en firewall
Block-RootCauseIp -IpAddress "1.2.3.4"

# Detener servicio
Stop-RootCauseService -ServiceName "bits"

# Captura WPR desde PowerShell
Start-RootCauseCapture -Note "Disco al 100%"
Stop-RootCauseCapture -Note "Disco al 100%"
```

---

## VS Code Extension

```powershell
# Empaquetar la extensión (requiere Node.js)
cd vscode-extension
npm install
npx vsce package

# Instalar en VS Code
code --install-extension rootcause-inspector-0.1.0.vsix
```

Comandos disponibles desde la paleta de comandos (`Ctrl+Shift+P`):
- `RootCause: Actualizar estado del sistema`
- `RootCause: Exportar snapshot a JSON`
- `RootCause: Abrir panel de diagnóstico`

---

## Gestores de paquetes Windows

```powershell
# Scoop
scoop install rootcause

# Winget
winget install VladimirAcuna.RootCause

# Chocolatey
choco install rootcause-windows-inspector
```

---

## Empaquetado de la demo pública

### Compilar instalador DEMO con Inno Setup

```powershell
.\scripts\package-inno-demo.ps1
```

### Compilar manualmente el script DEMO

```powershell
iscc .\packaging\windows\RootCause-Demo.iss
```

### Archivos que debe revisar antes de publicar la demo

- `docs/DEMO_PUBLICA.md`
- `docs/GUIA_DE_USO_PREVIA.md`
- `docs/LIMITACIONES_DEMO.md`
- `docs/POLITICA_DE_PRIVACIDAD_LOCAL.md`
- `docs/INSTALACION_TRANSPARENTE_DEMO.md`
