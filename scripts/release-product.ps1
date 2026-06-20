[CmdletBinding()]
param(
    [string]$Version,
    [switch]$VerifyEnvironment,
    [switch]$Publish,
    [switch]$SkipPublicLanding
)

$ErrorActionPreference = 'Stop'

$root = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$buildDir = Join-Path $root 'build'
$privateRepo = 'vladimiracunadev-create/rootcause-windows-inspector'
$publicRepo = 'vladimiracunadev-create/rootcause-windows-inspector'

function Write-Step([string]$Message) {
    Write-Host "`n==> $Message" -ForegroundColor Cyan
}

function Ensure-Command([string]$Name, [string]$Hint) {
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "$Name no está disponible. $Hint"
    }
}

function Invoke-RepoScript([string]$RelativePath) {
    $scriptPath = Join-Path $root $RelativePath
    if (-not (Test-Path $scriptPath)) {
        throw "No existe el script requerido: $scriptPath"
    }
    & $scriptPath
}

function Get-ProjectVersion {
    $cargoToml = Join-Path $root 'Cargo.toml'
    $match = Select-String -Path $cargoToml -Pattern '^version = "([^"]+)"' | Select-Object -First 1
    if (-not $match) {
        throw 'No fue posible detectar la versión en Cargo.toml'
    }
    return $match.Matches[0].Groups[1].Value
}

function Get-CurrentBranch {
    return (& git -C $root branch --show-current).Trim()
}

function Assert-RequiredArtifacts {
    $required = @(
        (Join-Path $buildDir 'RootCause-Portable.zip'),
        (Join-Path $buildDir 'RootCause-CLI-Portable.zip'),
        (Join-Path $buildDir 'RootCause.psm1'),
        (Join-Path $buildDir 'RootCause-VSCode-Extension.vsix'),
        (Join-Path $buildDir 'installer\RootCause-Setup.exe'),
        (Join-Path $buildDir 'SHA256SUMS.txt')
    )

    foreach ($path in $required) {
        if (-not (Test-Path $path)) {
            throw "Artefacto faltante: $path"
        }
    }
}

function Write-PrivateReleaseNotes([string]$Tag) {
    $path = Join-Path $buildDir "release-notes-$Tag-private.md"
    @"
## RootCause - Windows Inspector $Tag

Software de diagnostico para Windows escrito en Rust.
Detecta procesos problematicos, archivos temporales, conexiones sospechosas y actividad de actualizacion en segundo plano.

## Novedades de esta version

- Deteccion heuristica V1 de comportamiento anomalo y posible actividad maliciosa.
- Correlacion simple de senales para incidentes con severidad, evidencia y posible hipotesis de causa raiz.
- Cobertura inicial sobre rutas sospechosas, consumo anomalo, persistencia basica, reaparicion de procesos y actividad saliente inusual.
- Nueva linea base de resiliencia del agente: heartbeat local, deteccion de cierre abrupto previo e integridad basica de configuracion.
- Estado del agente visible en GUI, snapshot exportado, `status --json` y `config show`.
- Recomendacion de backoff ante reinicios o cierres abruptos repetidos para evitar bucles de recuperacion torpes.
- RootCause complementa observabilidad y diagnostico del endpoint, pero no reemplaza un antivirus o EDR dedicado.

---

## Que hay en este release

| Archivo | Descripcion |
|---|---|
| `RootCause-Setup.exe` | Instalador principal GUI + CLI |
| `RootCause-Portable.zip` | Version portable del build principal GUI + CLI |
| `RootCause-CLI-Portable.zip` | Version portable CLI-only (~4 MB, sin GUI) |
| `RootCause.psm1` | Modulo PowerShell (requiere `rootcause.exe`) |
| `RootCause-VSCode-Extension.vsix` | Extension VS Code (requiere `rootcause.exe`) |
| `SHA256SUMS.txt` | Hashes SHA-256 para verificar integridad antes de ejecutar |

---

## Instalacion rapida

### Opcion A - Instalador
1. Descarga `RootCause-Setup.exe`
2. Verifica el hash
3. Ejecuta el instalador
4. Abre RootCause o usa `rootcause` desde consola

### Opcion B - Portable GUI
1. Descarga `RootCause-Portable.zip`
2. Verifica el hash
3. Extrae en cualquier carpeta
4. Ejecuta `rootcause.exe` como administrador

### Opcion C - CLI-only / integraciones
- `RootCause-CLI-Portable.zip`: scripts, Server Core, CI
- `RootCause.psm1`: integracion PowerShell; requiere `rootcause.exe`
- `RootCause-VSCode-Extension.vsix`: integracion VS Code; requiere `rootcause.exe`

---

## Verificar integridad

```powershell
Get-FileHash .\RootCause-Portable.zip -Algorithm SHA256
Get-FileHash .\RootCause-CLI-Portable.zip -Algorithm SHA256
Get-FileHash .\RootCause-Setup.exe -Algorithm SHA256
Get-FileHash .\RootCause.psm1 -Algorithm SHA256
Get-FileHash .\RootCause-VSCode-Extension.vsix -Algorithm SHA256
```

Compara con `SHA256SUMS.txt`.
"@ | Set-Content -Path $path -Encoding UTF8
    return $path
}

function Write-PublicReleaseNotes([string]$Tag) {
    $path = Join-Path $buildDir "release-notes-$Tag-public.md"
    @"
## RootCause Windows Inspector $Tag

Monitor forense ligero para Windows. Detecta que proceso, servicio o conexion esta degradando tu equipo.

## Novedades de esta version

- Nueva deteccion heuristica de actividad anomala compatible con problemas de seguridad.
- Nueva correlacion basica de senales para priorizar severidad, evidencia y recomendaciones.
- Nueva linea base de resiliencia del agente con heartbeat local, deteccion de cierre abrupto previo e integridad basica de configuracion.
- Estado del agente visible en GUI, snapshot exportado, `status --json` y `config show`.
- Recomendacion de backoff ante reinicios o cierres abruptos repetidos para evitar bucles de recuperacion torpes.
- RootCause no sustituye antivirus ni EDR especializados; complementa observabilidad y diagnostico local.

---

## Archivos en este release

| Archivo | Que es |
|---|---|
| `RootCause-Setup.exe` | Instalador principal GUI + CLI |
| `RootCause-Portable.zip` | Portable del build principal GUI + CLI |
| `RootCause-CLI-Portable.zip` | Portable CLI-only (~4 MB) |
| `RootCause.psm1` | Modulo PowerShell (requiere `rootcause.exe`) |
| `RootCause-VSCode-Extension.vsix` | Extension VS Code (requiere `rootcause.exe`) |
| `SHA256SUMS.txt` | Hashes SHA-256 para verificar integridad |

---

## Instalacion rapida

**Instalador**
1. Descarga `RootCause-Setup.exe`
2. Ejecuta y sigue los pasos
3. `rootcause` queda disponible en el PATH del sistema

**Portable GUI**
1. Descarga `RootCause-Portable.zip`
2. Extrae en cualquier carpeta
3. Ejecuta `rootcause.exe`

**Integraciones y modo CLI-only**
- `RootCause-CLI-Portable.zip` para Server Core, scripts y CI
- `RootCause.psm1` para automatizacion PowerShell
- `RootCause-VSCode-Extension.vsix` para usar RootCause desde VS Code

---

Landing: https://vladimiracunadev-create.github.io/rootcause-windows-inspector/
"@ | Set-Content -Path $path -Encoding UTF8
    return $path
}

function Ensure-Release([string]$Repo, [string]$Tag, [string]$Title, [string]$NotesFile, [string[]]$Assets) {
    $null = & gh release view $Tag --repo $Repo 2>$null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Release existente en ${Repo}: $Tag" -ForegroundColor Yellow
        return
    }

    & gh release create $Tag `
        --repo $Repo `
        --title $Title `
        --notes-file $NotesFile `
        @Assets
}

function Wait-ReleaseWorkflow([string]$Tag) {
    $runId = ''
    for ($i = 0; $i -lt 30; $i++) {
        $runId = (& gh run list --workflow release-windows.yml --repo $privateRepo --json databaseId,headBranch --jq ".[] | select(.headBranch == `"$Tag`") | .databaseId" --limit 20).Trim()
        if ($runId) {
            break
        }
        Start-Sleep -Seconds 5
    }

    if (-not $runId) {
        throw "No apareció una corrida de release-windows para $Tag"
    }

    Write-Step "Esperando workflow release-windows ($runId)"
    & gh run watch $runId --repo $privateRepo --interval 10
    if ($LASTEXITCODE -ne 0) {
        throw "La corrida $runId terminó con error"
    }
}

$resolvedVersion = if ($Version) { $Version } else { Get-ProjectVersion }
$tag = if ($resolvedVersion.StartsWith('v')) { $resolvedVersion } else { "v$resolvedVersion" }
$currentBranch = Get-CurrentBranch

Write-Host "Release target: $tag" -ForegroundColor Green
Write-Host "Branch actual : $currentBranch" -ForegroundColor Green

Ensure-Command cargo 'Instala Rustup.'
Ensure-Command git 'Instala Git.'

if ($VerifyEnvironment) {
    Write-Step 'Verificando entorno'
    Invoke-RepoScript 'scripts\verify-environment.ps1'
}

Write-Step 'Quality gates + build release'
Invoke-RepoScript 'scripts\quality-gates.ps1'

Write-Step 'Portable GUI'
Invoke-RepoScript 'scripts\package-portable.ps1'

Write-Step 'Instalador Inno'
Invoke-RepoScript 'scripts\package-inno.ps1'

Write-Step 'Compilando edicion CLI-only'
cargo build --release --no-default-features --target-dir target/cli

Write-Step 'Portable CLI-only'
Invoke-RepoScript 'scripts\package-cli-portable.ps1'

Write-Step 'Modulo PowerShell'
Invoke-RepoScript 'scripts\package-powershell-module.ps1'

Write-Step 'Extension VS Code'
Invoke-RepoScript 'scripts\package-vscode-extension.ps1'

Write-Step 'Hashes SHA-256'
Invoke-RepoScript 'scripts\hash-artifacts.ps1'

Write-Step 'Validando artefactos generados'
Assert-RequiredArtifacts

if (-not $Publish) {
    Write-Host "`nArtefactos locales listos en $buildDir" -ForegroundColor Green
    exit 0
}

Ensure-Command gh 'Instala y autentica GitHub CLI (`gh auth login`).'

Write-Step 'Verificando autenticacion GitHub CLI'
& gh auth status

$privateNotes = Write-PrivateReleaseNotes $tag
$publicNotes = Write-PublicReleaseNotes $tag
$localAssets = @(
    (Join-Path $buildDir 'RootCause-Portable.zip'),
    (Join-Path $buildDir 'RootCause-CLI-Portable.zip'),
    (Join-Path $buildDir 'RootCause.psm1'),
    (Join-Path $buildDir 'RootCause-VSCode-Extension.vsix'),
    (Join-Path $buildDir 'installer\RootCause-Setup.exe'),
    (Join-Path $buildDir 'SHA256SUMS.txt')
)

Write-Step "Publicando branch $currentBranch"
& git -C $root push origin $currentBranch
if ($LASTEXITCODE -ne 0) {
    throw "No se pudo hacer push de $currentBranch"
}

Write-Step "Creando tag $tag"
& git -C $root rev-parse $tag 2>$null | Out-Null
if ($LASTEXITCODE -ne 0) {
    & git -C $root tag $tag
    if ($LASTEXITCODE -ne 0) {
        throw "No se pudo crear el tag $tag"
    }
}
else {
    Write-Host "Tag local ya existe: $tag" -ForegroundColor Yellow
}

Write-Step "Publicando tag $tag"
& git -C $root push origin $tag
if ($LASTEXITCODE -ne 0) {
    throw "No se pudo hacer push del tag $tag"
}

Wait-ReleaseWorkflow $tag

Write-Step 'Verificando release privado'
& gh release view $tag --repo $privateRepo
if ($LASTEXITCODE -ne 0) {
    Ensure-Release `
        -Repo $privateRepo `
        -Tag $tag `
        -Title "RootCause $tag" `
        -NotesFile $privateNotes `
        -Assets $localAssets
}

if (-not $SkipPublicLanding) {
    Write-Step 'Verificando release publico'
    & gh release view $tag --repo $publicRepo
    if ($LASTEXITCODE -ne 0) {
        Ensure-Release `
            -Repo $publicRepo `
            -Tag $tag `
            -Title "RootCause Windows Inspector $tag" `
            -NotesFile $publicNotes `
            -Assets $localAssets
    }
}

Write-Host "`nRelease completo OK: $tag" -ForegroundColor Green
