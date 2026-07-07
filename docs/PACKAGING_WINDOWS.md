# Empaquetado en Windows

Este documento define la ruta profesional para generar artefactos de distribución sin incluir binarios precompilados en el repositorio.

---

## 1) Objetivos de empaquetado

- entregar los artefactos del runtime principal,
- entregar las integraciones publicables que ya existen,
- dejar hashes verificables,
- mantener trazabilidad entre fuente y artefacto.

---

## 2) Catálogo de artefactos

La referencia canónica del producto vive en [`CATALOGO_PRODUCTO.md`](CATALOGO_PRODUCTO.md).

Para empaquetado conviene separar:

- **núcleo**: GUI principal y CLI-only,
- **adaptadores**: PowerShell y VS Code,
- **perfil alternativo**: RootCause Demo.

### Portable ZIP principal (`RootCause-Portable.zip`)
Útil para:
- pruebas rápidas,
- distribución controlada,
- validaciones internas,
- revisión por reclutadores técnicos.

Este portable corresponde al build principal con GUI activa; no debe anunciarse como `CLI-only`.

### Instalador Inno Setup (`RootCause-Setup.exe`)
Útil para:
- experiencia más profesional,
- accesos directos,
- desinstalación,
- entrega más formal.

### CLI-only portable (`RootCause-CLI-Portable.zip`)
Build sin egui ni eframe. Para:
- sysadmins y scripts de automatización,
- Windows Server Core (sin escritorio),
- integración en pipelines CI,
- distribución por gestores de paquetes.

```powershell
cargo build --release --no-default-features --target-dir target/cli
# Produce: target\cli\release\rootcause.exe (~4 MB, sin GUI)
```

### Módulo PowerShell (`RootCause.psm1`)
`packaging/powershell/RootCause.psm1` — 9 cmdlets que envuelven la CLI.
No es standalone: requiere `rootcause.exe` en PATH o junto al módulo.

```powershell
Import-Module .\packaging\powershell\RootCause.psm1
Get-RootCauseStatus
Get-RootCauseProcesses | Where-Object Severity -eq "Critical"
```

### VS Code Extension (`RootCause-VSCode-Extension.vsix`)
`vscode-extension/` — extensión TypeScript empaquetable con `vsce package`.
No es standalone: requiere `rootcause.exe` disponible para consultar estado y exportar snapshots.

```powershell
cd vscode-extension
npm install
npx @vscode/vsce package --out ..\build\RootCause-VSCode-Extension.vsix
code --install-extension RootCause-VSCode-Extension.vsix
```

---

## 3) Flujo recomendado

1. `verify-environment`
2. `quality-gates`
3. `build-release`
4. `package-portable`
5. `package-inno`
6. `cargo build --release --no-default-features --target-dir target/cli`
7. `package-cli-portable`
8. `package-powershell-module`
9. `package-vscode-extension`
10. `hash-artifacts`

### Comando unico recomendado

```powershell
.\scripts\release-product.ps1 -VerifyEnvironment
```

Publicacion completa con push, tag y verificacion del release:

```powershell
.\scripts\release-product.ps1 -VerifyEnvironment -Publish
```

Wrapper para Git Bash / shell compatible:

```sh
./scripts/release-product.sh -VerifyEnvironment
./scripts/release-product.sh -VerifyEnvironment -Publish
```

---

## 4) Comandos

### Release completo

```powershell
.\scripts\release-product.ps1 -VerifyEnvironment
```

### Release completo + publicacion

```powershell
.\scripts\release-product.ps1 -VerifyEnvironment -Publish
```

### Portable

```powershell
.\scripts\package-portable.ps1
```

### CLI-only portable

```powershell
cargo build --release --no-default-features --target-dir target/cli
.\scripts\package-cli-portable.ps1
```

### Instalador

```powershell
.\scripts\package-inno.ps1
```

### Módulo PowerShell

```powershell
.\scripts\package-powershell-module.ps1
```

### VS Code Extension

```powershell
.\scripts\package-vscode-extension.ps1
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

### Para extensión VS Code
- Node.js
- `npm`

---

## 6) Política de binarios

- el repositorio no debe almacenar el `.exe` final,
- el instalador debe generarse localmente,
- todo artefacto debe ser reconstruible desde el código fuente y la documentación.


## 7) Empaquetado en GitHub Actions

El flujo `release-windows.yml` automatiza esta secuencia en `windows-latest`:

1. quality gates,
2. build release GUI,
3. ZIP portable GUI,
4. instalación de Inno Setup,
5. compilación de instalador,
6. build CLI-only,
7. ZIP CLI-only,
8. módulo PowerShell,
9. extensión VS Code,
10. generación de hashes.

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

> **Prerequisito para los tres gestores:** tener al menos un release público con los artefactos en las releases de este repo (ya disponible desde v0.12.0). Actualizar los campos `UPDATE_SHA256_ON_RELEASE` con los hashes SHA-256 reales de cada release.

---

## Ruta recomendada para la demo pública

Para distribución pública de evaluación separada del perfil principal se recomienda usar `packaging/windows/RootCause-Demo.iss`, no el empaquetado principal.

Objetivos de este instalador:

- mostrar un mensaje previo honesto,
- dejar claro que se trata de una demo,
- crear accesos con el nombre `RootCause Demo`,
- instalar documentación útil junto al binario,
- ofrecer abrir `LEEME-DEMO.txt` al finalizar.
