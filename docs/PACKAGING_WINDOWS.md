# Empaquetado en Windows

Este documento define la ruta profesional para generar artefactos de distribución sin incluir binarios precompilados en el repositorio.

---

## 1) Objetivos de empaquetado

- entregar un ZIP portable,
- entregar un instalador `.exe` con Inno Setup,
- dejar hashes verificables,
- mantener trazabilidad entre fuente y artefacto.

---

## 2) Artefactos posibles

### Portable ZIP
Útil para:
- pruebas rápidas,
- distribución controlada,
- validaciones internas,
- revisión por reclutadores técnicos.

### Instalador Inno Setup
Útil para:
- experiencia más profesional,
- accesos directos,
- desinstalación,
- entrega más formal.

### CLI-only binary (~4 MB)
Build sin egui ni eframe. Para:
- sysadmins y scripts de automatización,
- Windows Server Core (sin escritorio),
- integración en pipelines CI,
- distribución por gestores de paquetes.

```powershell
cargo build --release --no-default-features
# Produce: target\release\rootcause.exe (~4 MB, sin GUI)
```

### Módulo PowerShell
`packaging/powershell/RootCause.psm1` — 9 cmdlets que envuelven la CLI.
Distribución: copiar el `.psm1` a cualquier directorio en `$PSModulePath`.

```powershell
Import-Module .\packaging\powershell\RootCause.psm1
Get-RootCauseStatus
Get-RootCauseProcesses | Where-Object Severity -eq "Critical"
```

### VS Code Extension
`vscode-extension/` — extensión TypeScript empaquetable con `vsce package`.
Requiere: `npm install -g @vscode/vsce` y `npm install` en `vscode-extension/`.

```powershell
cd vscode-extension
npm install
npx vsce package   # genera rootcause-inspector-0.7.0.vsix
code --install-extension rootcause-inspector-0.7.0.vsix
```

---

## 3) Flujo recomendado

1. `verify-environment`
2. `quality-gates`
3. `build-release`
4. `package-portable`
5. `package-inno`
6. `hash-artifacts`

---

## 4) Comandos

### Portable

```powershell
.\scripts\package-portable.ps1
```

### Instalador

```powershell
.\scripts\package-inno.ps1
```

### Hashes

```powershell
.\scripts\hash-artifacts.ps1
```

---

## 5) Requisitos

### Obligatorios
- build release exitoso

### Para instalador
- Inno Setup instalado
- `ISCC.exe` en PATH o ruta conocida por el script

---

## 6) Política de binarios

- el repositorio no debe almacenar el `.exe` final,
- el instalador debe generarse localmente,
- todo artefacto debe ser reconstruible desde el código fuente y la documentación.


## 7) Empaquetado en GitHub Actions

El flujo `release-windows.yml` automatiza esta secuencia en `windows-latest`:

1. quality gates,
2. build release,
3. ZIP portable,
4. instalación de Inno Setup,
5. compilación de instalador,
6. generación de hashes.

Esto no elimina la necesidad de validar el instalador en una máquina Windows real antes de distribuirlo fuera de un entorno controlado.


## Identidad visual del paquete

El instalador y los accesos directos deben usar el icono de marca definido en `assets/rootcause.ico`.

Esto ayuda a que:

- el instalador se vea consistente con el producto,
- el acceso del escritorio muestre `RootCause`,
- el usuario pueda fijar la app en Windows 11 con una identidad visual coherente.


## Distribución por gestores de paquetes Windows

### Scoop
Manifest: `packaging/distribution/scoop/rootcause.json`

```powershell
# Publicar en bucket Scoop propio
scoop bucket add rootcause https://github.com/vladimiracunadev-create/rootcause-scoop-bucket
scoop install rootcause

# Una vez en el bucket oficial:
scoop install rootcause
```

### Winget
Manifest: `packaging/distribution/winget/rootcause.yaml`
PackageIdentifier: `VladimirAcuna.RootCause`

```powershell
# Validar manifest localmente
winget validate --manifest packaging\distribution\winget\

# Subir a winget-pkgs: PR en https://github.com/microsoft/winget-pkgs
# Instalación final del usuario:
winget install VladimirAcuna.RootCause
```

### Chocolatey
Manifests: `packaging/chocolatey/rootcause.nuspec` + `tools/chocolateyInstall.ps1`

```powershell
# Empaquetar localmente
cd packaging\chocolatey
choco pack

# Publicar en Chocolatey Community (requiere cuenta):
choco push rootcause-windows-inspector.X.Y.Z.nupkg --source https://push.chocolatey.org

# Instalación final del usuario:
choco install rootcause-windows-inspector
```

> **Prerequisito para los tres gestores:** tener al menos un release público con los artefactos en `rootcause-landing/releases`. Actualizar los campos `UPDATE_SHA256_ON_RELEASE` con los hashes SHA-256 reales de cada release.

---

## Ruta recomendada para la demo pública

Para distribución pública se recomienda usar `packaging/windows/RootCause-Demo.iss`, no el empaquetado interno de trabajo.

Objetivos de este instalador:

- mostrar un mensaje previo honesto,
- dejar claro que se trata de una demo,
- crear accesos con el nombre `RootCause Demo`,
- instalar documentación útil junto al binario,
- ofrecer abrir `LEEME-DEMO.txt` al finalizar.
