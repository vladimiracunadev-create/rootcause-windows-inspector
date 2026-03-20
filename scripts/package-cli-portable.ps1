$ErrorActionPreference = 'Stop'

$root = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$releaseExe = Join-Path $root 'target\cli\release\rootcause.exe'
$buildDir = Join-Path $root 'build\RootCause-CLI-Portable'
$zipPath = Join-Path $root 'build\RootCause-CLI-Portable.zip'

if (-not (Test-Path $releaseExe)) {
    throw 'No existe el ejecutable CLI-only. Ejecuta antes cargo build --release --no-default-features --target-dir target/cli'
}

if (Test-Path $buildDir) {
    Remove-Item $buildDir -Recurse -Force
}
if (Test-Path $zipPath) {
    Remove-Item $zipPath -Force
}

New-Item -ItemType Directory -Path $buildDir | Out-Null
Copy-Item $releaseExe $buildDir
Copy-Item (Join-Path $root 'README.md') $buildDir
Copy-Item (Join-Path $root 'LICENSE') $buildDir
Copy-Item (Join-Path $root 'SECURITY.md') $buildDir
Copy-Item (Join-Path $root 'docs') $buildDir -Recurse
Copy-Item (Join-Path $root 'scripts') $buildDir -Recurse

Compress-Archive -Path (Join-Path $buildDir '*') -DestinationPath $zipPath

Write-Host "CLI-only portable listo: $zipPath" -ForegroundColor Green
