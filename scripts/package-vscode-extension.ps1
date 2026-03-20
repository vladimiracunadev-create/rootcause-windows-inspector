$ErrorActionPreference = 'Stop'

$root = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$extensionDir = Join-Path $root 'vscode-extension'
$artifact = Join-Path $root 'build\RootCause-VSCode-Extension.vsix'

if (-not (Test-Path $extensionDir)) {
    throw 'No existe la carpeta vscode-extension'
}

if (-not (Get-Command npm -ErrorAction SilentlyContinue)) {
    throw 'npm no está disponible. Instala Node.js para empaquetar la extensión VS Code.'
}

if (Test-Path $artifact) {
    Remove-Item $artifact -Force
}

Push-Location $extensionDir
try {
    npm install --no-fund --no-audit
    npx @vscode/vsce package --out $artifact
}
finally {
    Pop-Location
}

Write-Host "VS Code extension lista: $artifact" -ForegroundColor Green
