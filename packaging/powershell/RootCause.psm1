#Requires -Version 5.1
<#
.SYNOPSIS
    Módulo PowerShell para RootCause Windows Inspector.

.DESCRIPTION
    Wrapper que expone los comandos CLI de RootCause como cmdlets nativos de
    PowerShell. Permite integrar el diagnóstico del sistema en scripts existentes
    y pipelines de automatización.

.NOTES
    Requiere que rootcause.exe esté en el PATH o en la misma carpeta que este módulo.

.EXAMPLE
    Import-Module .\RootCause.psm1
    Get-RootCauseStatus
    Get-RootCauseProcesses | Where-Object { $_.Severity -eq "Critical" }
    Invoke-RootCauseExport -Path "C:\diag\snapshot.json"
#>

# ── Resolución del ejecutable ────────────────────────────────────────────────────

function Resolve-RootCauseExe {
    $local = Join-Path $PSScriptRoot "rootcause.exe"
    if (Test-Path $local) { return $local }
    $inPath = Get-Command "rootcause" -ErrorAction SilentlyContinue
    if ($inPath) { return $inPath.Source }
    throw "rootcause.exe no encontrado. Asegúrate de que esté en el PATH o en la misma carpeta que el módulo."
}

# ── Cmdlets públicos ─────────────────────────────────────────────────────────────

function Get-RootCauseStatus {
    <#
    .SYNOPSIS
        Devuelve el estado actual del sistema como objeto PowerShell.
    .DESCRIPTION
        Ejecuta 'rootcause status --json' y convierte la salida en un PSCustomObject
        con las propiedades Severity, CpuPercent, RamUsedGb, RamTotalGb,
        IoWriteMb, NetworkRxMb, AlertCount y PrimaryReason.
    .EXAMPLE
        Get-RootCauseStatus
    .EXAMPLE
        $s = Get-RootCauseStatus
        if ($s.Severity -eq "Critical") { Send-MailMessage ... }
    #>
    [CmdletBinding()]
    [OutputType([PSCustomObject])]
    param()

    $exe  = Resolve-RootCauseExe
    $json = & $exe status --json 2>$null

    if (-not $json) {
        # Fallback: parsear salida de texto plano
        $text = & $exe status 2>&1 | Out-String
        return [PSCustomObject]@{
            Severity      = if ($text -match "CRITICAL") { "Critical" } elseif ($text -match "WARNING") { "Warning" } else { "Healthy" }
            RawOutput     = $text
        }
    }

    try {
        $data = $json | ConvertFrom-Json
        [PSCustomObject]@{
            Severity      = $data.severity
            CpuPercent    = $data.cpu_percent
            RamUsedGb     = $data.ram_used_gb
            RamTotalGb    = $data.ram_total_gb
            IoWriteMb     = $data.io_write_mb
            NetworkRxMb   = $data.network_rx_mb
            AlertCount    = $data.alert_count
            PrimaryReason = $data.primary_reason
        }
    }
    catch {
        Write-Warning "No se pudo parsear la salida JSON: $_"
        [PSCustomObject]@{ RawOutput = $json }
    }
}

function Get-RootCauseProcesses {
    <#
    .SYNOPSIS
        Devuelve la lista de procesos con su clasificación de severidad.
    .DESCRIPTION
        Ejecuta 'rootcause snapshot' y extrae los procesos del JSON resultante.
        Cada proceso se devuelve como PSCustomObject con PID, Name, Severity,
        CpuPercent, MemoryMb, IoWriteMb y ExePath.
    .PARAMETER MinSeverity
        Filtra por severidad mínima: Healthy, Warning o Critical.
    .EXAMPLE
        Get-RootCauseProcesses
    .EXAMPLE
        Get-RootCauseProcesses -MinSeverity Warning | Select-Object Name, CpuPercent
    .EXAMPLE
        Get-RootCauseProcesses | Where-Object { $_.Severity -eq "Critical" } | ForEach-Object { Write-Warning "PID $($_.Pid): $($_.Name)" }
    #>
    [CmdletBinding()]
    [OutputType([PSCustomObject[]])]
    param(
        [ValidateSet("Healthy", "Warning", "Critical")]
        [string]$MinSeverity = "Healthy"
    )

    $exe  = Resolve-RootCauseExe
    $json = & $exe snapshot 2>&1

    try {
        $snap = $json | ConvertFrom-Json
        $severityOrder = @{ "Healthy" = 0; "Warning" = 1; "Critical" = 2 }
        $minLevel = $severityOrder[$MinSeverity]

        $snap.processes | ForEach-Object {
            $level = $severityOrder[$_.severity]
            if ($level -ge $minLevel) {
                [PSCustomObject]@{
                    Pid        = $_.pid
                    Name       = $_.name
                    Severity   = $_.severity
                    CpuPercent = $_.cpu_percent
                    MemoryMb   = $_.memory_mb
                    IoWriteMb  = $_.io_write_mb_delta
                    ExePath    = $_.exe_path
                    Score      = $_.score
                }
            }
        }
    }
    catch {
        Write-Error "No se pudo obtener la lista de procesos: $_"
    }
}

function Get-RootCauseHistory {
    <#
    .SYNOPSIS
        Devuelve el historial de capturas almacenado en SQLite.
    .PARAMETER Count
        Número de capturas a recuperar (por defecto 10, máximo 100).
    .EXAMPLE
        Get-RootCauseHistory
    .EXAMPLE
        Get-RootCauseHistory -Count 30 | Where-Object HasCritical | Select-Object CollectedAt, DominantProcess
    #>
    [CmdletBinding()]
    [OutputType([PSCustomObject[]])]
    param(
        [ValidateRange(1, 100)]
        [int]$Count = 10
    )

    $exe  = Resolve-RootCauseExe
    $json = & $exe history $Count --json 2>&1

    try {
        $rows = $json | ConvertFrom-Json
        $rows | ForEach-Object {
            [PSCustomObject]@{
                CollectedAt      = $_.collected_at
                CpuUsage         = $_.cpu_usage
                MemoryUsedGb     = $_.memory_used_gb
                IoWriteMb        = $_.io_write_mb_delta
                DominantProcess  = $_.dominant_process
                AlertCount       = $_.alerts_count
                HasCritical      = $_.has_critical
            }
        }
    }
    catch {
        Write-Warning "Salida no es JSON válido — mostrando texto plano"
        & $exe history $Count 2>&1
    }
}

function Invoke-RootCauseExport {
    <#
    .SYNOPSIS
        Exporta un snapshot completo a JSON.
    .PARAMETER Path
        Ruta de destino. Si no se especifica, se usa Descargas o Documentos.
    .EXAMPLE
        Invoke-RootCauseExport
    .EXAMPLE
        Invoke-RootCauseExport -Path "C:\soporte\diagnostico.json"
    #>
    [CmdletBinding()]
    param(
        [string]$Path
    )

    $exe = Resolve-RootCauseExe

    if ($Path) {
        $json = & $exe snapshot 2>&1
        $json | Set-Content -Path $Path -Encoding UTF8
        Write-Host "Exportado → $Path"
    }
    else {
        & $exe export 2>&1
    }
}

function Stop-RootCauseProcess {
    <#
    .SYNOPSIS
        Finaliza un proceso por PID usando la política de protección de RootCause.
    .PARAMETER Pid
        PID del proceso a finalizar.
    .EXAMPLE
        Stop-RootCauseProcess -Pid 1234
    #>
    [CmdletBinding(SupportsShouldProcess)]
    param(
        [Parameter(Mandatory)]
        [int]$Pid
    )

    if ($PSCmdlet.ShouldProcess("PID $Pid", "Finalizar proceso")) {
        $exe = Resolve-RootCauseExe
        & $exe kill $Pid 2>&1
    }
}

function Block-RootCauseIp {
    <#
    .SYNOPSIS
        Bloquea una IP remota vía firewall de Windows.
    .PARAMETER IpAddress
        Dirección IP a bloquear.
    .EXAMPLE
        Block-RootCauseIp -IpAddress "192.0.2.100"
    #>
    [CmdletBinding(SupportsShouldProcess)]
    param(
        [Parameter(Mandatory)]
        [string]$IpAddress
    )

    if ($PSCmdlet.ShouldProcess($IpAddress, "Bloquear IP en firewall")) {
        $exe = Resolve-RootCauseExe
        & $exe block-ip $IpAddress 2>&1
    }
}

function Stop-RootCauseService {
    <#
    .SYNOPSIS
        Detiene un servicio permitido (bits, wuauserv, sysmain, dosvc).
    .PARAMETER ServiceName
        Nombre del servicio a detener.
    .EXAMPLE
        Stop-RootCauseService -ServiceName "wuauserv"
    #>
    [CmdletBinding(SupportsShouldProcess)]
    param(
        [Parameter(Mandatory)]
        [ValidateSet("bits", "wuauserv", "sysmain", "dosvc")]
        [string]$ServiceName
    )

    if ($PSCmdlet.ShouldProcess($ServiceName, "Detener servicio")) {
        $exe = Resolve-RootCauseExe
        & $exe stop-service $ServiceName 2>&1
    }
}

function Start-RootCauseCapture {
    <#
    .SYNOPSIS
        Inicia una captura ETW con WPR.
    .PARAMETER Note
        Nota descriptiva del problema que se está diagnosticando.
    .EXAMPLE
        Start-RootCauseCapture -Note "Disco lento durante backup nocturno"
    #>
    [CmdletBinding()]
    param(
        [string]$Note = "Captura iniciada desde PowerShell"
    )

    $exe = Resolve-RootCauseExe
    & $exe wpr start --note $Note 2>&1
}

function Stop-RootCauseCapture {
    <#
    .SYNOPSIS
        Detiene la captura ETW activa y guarda el ETL.
    .PARAMETER Note
        Descripción del problema reproducido durante la captura.
    #>
    [CmdletBinding()]
    param(
        [string]$Note = "Captura detenida desde PowerShell"
    )

    $exe = Resolve-RootCauseExe
    & $exe wpr stop --note $Note 2>&1
}

# ── Exportar cmdlets ─────────────────────────────────────────────────────────────

Export-ModuleMember -Function @(
    'Get-RootCauseStatus',
    'Get-RootCauseProcesses',
    'Get-RootCauseHistory',
    'Invoke-RootCauseExport',
    'Stop-RootCauseProcess',
    'Block-RootCauseIp',
    'Stop-RootCauseService',
    'Start-RootCauseCapture',
    'Stop-RootCauseCapture'
)
