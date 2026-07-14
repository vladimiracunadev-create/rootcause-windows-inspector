$ErrorActionPreference = 'Stop'

$root = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$buildDir = Join-Path $root 'build'
$hashFile = Join-Path $buildDir 'SHA256SUMS.txt'

if (-not (Test-Path $buildDir)) {
    throw 'No existe la carpeta build. Genera primero algún artefacto.'
}

# Solo los artefactos PUBLICABLES viven directamente en build\ o build\installer\.
# Las subcarpetas (p.ej. build\RootCause-Portable\) son staging para armar los ZIP
# y NO deben hashearse: contienen un rootcause.exe interno que no es un asset.
$allowedDirs = @($buildDir, (Join-Path $buildDir 'installer'))
$artifacts = Get-ChildItem $buildDir -Recurse -File | Where-Object {
    $_.Extension -in '.exe', '.zip', '.msi', '.etl', '.psm1', '.vsix' -and
    $allowedDirs -contains $_.DirectoryName
} | Sort-Object Name

if (-not $artifacts) {
    throw 'No se encontraron artefactos para hashear.'
}

# Se emiten BASENAMES (no rutas internas de CI) para que `sha256sum -c SHA256SUMS.txt`
# funcione en la carpeta donde el usuario descarga los assets planos del release.
$lines = foreach ($artifact in $artifacts) {
    $hash = Get-FileHash $artifact.FullName -Algorithm SHA256
    '{0} *{1}' -f $hash.Hash, $artifact.Name
}

# Sin BOM: un BOM al inicio rompe la verificación de la primera línea con sha256sum.
[System.IO.File]::WriteAllText($hashFile, (($lines -join "`n") + "`n"), (New-Object System.Text.UTF8Encoding($false)))
Write-Host "Hashes escritos en: $hashFile" -ForegroundColor Green
