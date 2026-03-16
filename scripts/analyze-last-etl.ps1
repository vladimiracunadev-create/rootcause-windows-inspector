param(
    [string]$TraceRoot = "$env:USERPROFILE\Documents\RootCause\traces",
    [string]$EtlPath,
    [string]$OutputRoot = "$env:USERPROFILE\Documents\RootCause\traces\analysis"
)

$ErrorActionPreference = "Stop"

if (-not (Get-Command tracerpt -ErrorAction SilentlyContinue)) {
    throw "tracerpt.exe no está disponible en PATH. Instala Windows Performance Toolkit o usa un Windows que ya lo incluya."
}

if (-not $EtlPath) {
    $last = Get-ChildItem -Path $TraceRoot -Filter *.etl -File -ErrorAction Stop |
        Sort-Object LastWriteTime -Descending |
        Select-Object -First 1

    if (-not $last) {
        throw "No se encontró ningún .etl en $TraceRoot"
    }

    $EtlPath = $last.FullName
}

if (-not (Test-Path $EtlPath)) {
    throw "No existe el ETL: $EtlPath"
}

$stem = [System.IO.Path]::GetFileNameWithoutExtension($EtlPath)
$outDir = Join-Path $OutputRoot $stem
$xmlPath = Join-Path $outDir "dumpfile.xml"
$summaryPath = Join-Path $outDir "summary.txt"

New-Item -ItemType Directory -Path $outDir -Force | Out-Null

Write-Host "[RootCause] ETL seleccionado: $EtlPath"
Write-Host "[RootCause] Carpeta de salida: $outDir"

tracerpt $EtlPath -o $xmlPath -of XML -lr -summary $summaryPath

Write-Host "[RootCause] Exportación completada"
Write-Host "XML: $xmlPath"
Write-Host "Summary: $summaryPath"
Write-Host "Siguiente paso: abre la app y pulsa 'Resumir último ETL' o revisa estos artefactos manualmente."
