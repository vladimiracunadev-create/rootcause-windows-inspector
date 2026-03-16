$ErrorActionPreference = 'Stop'

$root = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$buildDir = Join-Path $root 'build'
$hashFile = Join-Path $buildDir 'SHA256SUMS.txt'

if (-not (Test-Path $buildDir)) {
    throw 'No existe la carpeta build. Genera primero algún artefacto.'
}

$artifacts = Get-ChildItem $buildDir -Recurse -File | Where-Object {
    $_.Extension -in '.exe', '.zip', '.msi', '.etl'
}

if (-not $artifacts) {
    throw 'No se encontraron artefactos para hashear.'
}

$lines = foreach ($artifact in $artifacts) {
    $hash = Get-FileHash $artifact.FullName -Algorithm SHA256
    '{0} *{1}' -f $hash.Hash, $artifact.FullName.Substring($root.Length + 1)
}

$lines | Set-Content -Path $hashFile -Encoding UTF8
Write-Host "Hashes escritos en: $hashFile" -ForegroundColor Green
