//! Integración con utilidades nativas de Windows.
//!
//! Esta capa se apoya en comandos estándar del sistema para evitar más peso de
//! dependencias y llegar rápido a datos útiles: eventos, servicios, netstat,
//! acciones administrativas puntuales, control del modo de precisión con WPR y
//! exportación de ETL vía tracerpt.

use crate::models::{EventRecord, PersistenceEntry, ServiceState};
use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

/// Ejecuta un script de PowerShell y devuelve stdout como String.
pub fn powershell(script: &str) -> Result<String> {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                script,
            ])
            .output()
            .context("No se pudo invocar PowerShell")?;

        if !output.status.success() {
            bail!(merge_output(&output));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = script;
        Ok(String::new())
    }
}

/// Indica si un comando está disponible en el sistema.
pub fn command_exists(command: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        Command::new("where")
            .arg(command)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = command;
        false
    }
}

/// Ejecuta `netstat` de Windows.
pub fn netstat() -> Result<String> {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("netstat")
            .args(["-ano", "-n"])
            .output()
            .context("No se pudo ejecutar netstat")?;

        if !output.status.success() {
            bail!(merge_output(&output));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(String::new())
    }
}

/// Devuelve eventos recientes de Warning/Error desde el log System.
pub fn recent_system_events(limit: usize) -> Result<Vec<EventRecord>> {
    let script = format!(
        r#"
        $items = Get-WinEvent -LogName System -MaxEvents 200 |
            Where-Object {{ $_.LevelDisplayName -in @('Warning','Error') }} |
            Select-Object -First {limit} TimeCreated, ProviderName, Id, LevelDisplayName, Message;
        $items | ConvertTo-Json -Depth 4
        "#
    );

    let json = powershell(&script)?;
    if json.trim().is_empty() {
        return Ok(Vec::new());
    }

    let value: Value = serde_json::from_str(&json)?;
    let mut records = Vec::new();
    match value {
        Value::Array(items) => {
            for item in items {
                records.push(map_event(&item));
            }
        }
        object => records.push(map_event(&object)),
    }
    Ok(records)
}

/// Devuelve servicios que ayudan a explicar actividad de update/instalación.
pub fn relevant_services() -> Result<Vec<ServiceState>> {
    let script = r#"
        Get-Service -Name wuauserv,bits,DoSvc,TrustedInstaller,SysMain,WinDefend,WdNisSvc,MpsSvc,wscsvc,Sense -ErrorAction SilentlyContinue |
            Select-Object Name, DisplayName, Status, StartType |
            ConvertTo-Json -Depth 4
    "#;

    let json = powershell(script)?;
    if json.trim().is_empty() {
        return Ok(Vec::new());
    }

    let value: Value = serde_json::from_str(&json)?;
    let mut services = Vec::new();
    match value {
        Value::Array(items) => {
            for item in items {
                services.push(map_service(&item));
            }
        }
        object => services.push(map_service(&object)),
    }
    Ok(services)
}

/// Enumeracion ligera de puntos de persistencia comunes en Windows.
pub fn persistence_entries() -> Result<Vec<PersistenceEntry>> {
    let script = r#"
        $items = @()

        $registryLocations = @(
            @{ Path = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'; Kind = 'Registry Run (HKCU)' },
            @{ Path = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\RunOnce'; Kind = 'Registry RunOnce (HKCU)' },
            @{ Path = 'HKLM:\Software\Microsoft\Windows\CurrentVersion\Run'; Kind = 'Registry Run (HKLM)' },
            @{ Path = 'HKLM:\Software\Microsoft\Windows\CurrentVersion\RunOnce'; Kind = 'Registry RunOnce (HKLM)' }
        )

        foreach ($entry in $registryLocations) {
            if (-not (Test-Path $entry.Path)) { continue }
            $props = Get-ItemProperty -Path $entry.Path
            foreach ($prop in $props.PSObject.Properties) {
                if ($prop.Name -in @('PSPath', 'PSParentPath', 'PSChildName', 'PSDrive', 'PSProvider')) { continue }
                $items += [pscustomobject]@{
                    EntryKind = $entry.Kind
                    Location = $entry.Path
                    Name = $prop.Name
                    Command = [string]$prop.Value
                }
            }
        }

        $startupFolders = @(
            @{ Path = [Environment]::GetFolderPath('Startup'); Kind = 'Startup Folder (Current User)' },
            @{ Path = "$env:ProgramData\Microsoft\Windows\Start Menu\Programs\Startup"; Kind = 'Startup Folder (All Users)' }
        )

        foreach ($entry in $startupFolders) {
            if (-not $entry.Path -or -not (Test-Path $entry.Path)) { continue }
            Get-ChildItem -Path $entry.Path -File -Force -ErrorAction SilentlyContinue | ForEach-Object {
                $items += [pscustomobject]@{
                    EntryKind = $entry.Kind
                    Location = $entry.Path
                    Name = $_.Name
                    Command = $_.FullName
                }
            }
        }

        $items | ConvertTo-Json -Depth 4
    "#;

    let json = powershell(script)?;
    if json.trim().is_empty() {
        return Ok(Vec::new());
    }

    let value: Value = serde_json::from_str(&json)?;
    let mut entries = Vec::new();
    match value {
        Value::Array(items) => {
            for item in items {
                entries.push(map_persistence_entry(&item));
            }
        }
        object => entries.push(map_persistence_entry(&object)),
    }
    Ok(entries)
}

/// Finaliza un proceso usando taskkill para maximizar compatibilidad con Windows.
pub fn terminate_process(pid: u32) -> Result<String> {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .output()
            .context("No se pudo ejecutar taskkill")?;

        if !output.status.success() {
            bail!(merge_output(&output));
        }

        Ok(merge_output(&output))
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = pid;
        Ok("Acción disponible solo en Windows".to_owned())
    }
}

/// Crea una regla outbound para bloquear una IP remota sospechosa.
///
/// Valida estrictamente el formato de la IP antes de usarla en el script
/// PowerShell para evitar inyección de comandos.
pub fn block_remote_ip(ip: &str) -> Result<String> {
    let safe_ip = ip.trim();
    if !is_valid_firewall_ip(safe_ip) {
        bail!(
            "Dirección IP no válida: '{safe_ip}'. \
             Solo se aceptan IPv4 (ej. 1.2.3.4) o IPv6 (ej. 2001:db8::1)."
        );
    }
    let script = format!(
        "New-NetFirewallRule \
         -DisplayName 'RootCause block {safe_ip}' \
         -Direction Outbound \
         -RemoteAddress {safe_ip} \
         -Action Block | Out-Null; \
         Write-Output 'Regla creada para bloquear {safe_ip}'"
    );
    powershell(&script)
}

/// Detiene temporalmente un servicio permitido.
///
/// Valida que el nombre solo contenga caracteres alfanuméricos, guiones y
/// guiones bajos (defensa en profundidad; la lista permitida está en
/// `InspectorService::stop_service`).
pub fn stop_service(service_name: &str) -> Result<String> {
    let safe = service_name.trim();
    if safe.is_empty()
        || !safe
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        bail!("Nombre de servicio no válido: '{safe}'.");
    }
    let script = format!(
        "Stop-Service -Name '{safe}' -Force -ErrorAction Stop; \
         Write-Output 'Servicio {safe} detenido temporalmente'"
    );
    powershell(&script)
}

/// Valida que una cadena sea una dirección IPv4 o IPv6 segura para usar
/// en scripts PowerShell. Solo permite dígitos hexadecimales, puntos y
/// dos puntos — sin espacios, comillas ni metacaracteres de shell.
fn is_valid_firewall_ip(ip: &str) -> bool {
    if ip.is_empty() || ip.len() > 45 {
        return false;
    }
    ip.chars()
        .all(|c| c.is_ascii_hexdigit() || c == '.' || c == ':')
}

/// Indica si WPR está disponible en el sistema.
pub fn wpr_available() -> bool {
    command_exists("wpr")
}

/// Indica si WPA está disponible en el sistema.
pub fn wpa_available() -> bool {
    command_exists("wpa")
}

/// Indica si tracerpt está disponible en el sistema.
pub fn tracerpt_available() -> bool {
    command_exists("tracerpt")
}

/// Devuelve el estado textual del grabador WPR.
pub fn wpr_status() -> Result<String> {
    #[cfg(target_os = "windows")]
    {
        if !wpr_available() {
            bail!("wpr.exe no está disponible");
        }

        let output = Command::new("wpr")
            .arg("-status")
            .output()
            .context("No se pudo consultar el estado de WPR")?;

        let merged = merge_output(&output);
        if merged.is_empty() {
            return Ok("WPR no devolvió salida para -status".to_owned());
        }
        Ok(merged)
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok("Solo disponible en Windows".to_owned())
    }
}

/// Determina si actualmente hay una captura WPR activa.
pub fn wpr_is_recording() -> Result<bool> {
    let status = wpr_status()?;
    let lower = status.to_ascii_lowercase();
    Ok(!(lower.contains("there are no trace profiles running")
        || lower.contains("no hay perfiles de seguimiento en ejecución")
        || lower.contains("there are no trace profiles")
        || lower.contains("no trace profiles running")
        || lower.contains("error code: 0xc5583000")))
}

/// Inicia una captura WPR con GeneralProfile en file mode.
pub fn start_wpr_general_profile(record_temp_dir: &Path, marker_text: &str) -> Result<String> {
    #[cfg(target_os = "windows")]
    {
        if !wpr_available() {
            bail!(
                "wpr.exe no está disponible. Instala Windows Performance Toolkit antes de usar modo precisión."
            );
        }
        if wpr_is_recording().unwrap_or(false) {
            bail!(
                "Ya existe una captura WPR activa. Detén o cancela la sesión actual antes de iniciar otra."
            );
        }

        fs::create_dir_all(record_temp_dir)?;
        let temp_dir = record_temp_dir.display().to_string();
        let output = Command::new("wpr")
            .args([
                "-start",
                "GeneralProfile",
                "-filemode",
                "-recordtempto",
                &temp_dir,
            ])
            .output()
            .context("No se pudo iniciar WPR")?;

        if !output.status.success() {
            bail!(merge_output(&output));
        }

        let trimmed = marker_text.trim();
        if !trimmed.is_empty() {
            let marker = format!("RootCause precision: {trimmed}");
            let _ = Command::new("wpr").args(["-marker", &marker]).output();
        }

        Ok(format!(
            "Captura WPR iniciada con GeneralProfile en file mode. Carpeta temporal de traza: {temp_dir}"
        ))
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (record_temp_dir, marker_text);
        Ok("Solo disponible en Windows".to_owned())
    }
}

/// Detiene la captura WPR activa y guarda el ETL comprimido.
pub fn stop_wpr_capture(output_path: &Path, problem_description: &str) -> Result<String> {
    #[cfg(target_os = "windows")]
    {
        if !wpr_available() {
            bail!(
                "wpr.exe no está disponible. Instala Windows Performance Toolkit antes de usar modo precisión."
            );
        }

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let output_file = output_path.display().to_string();
        let problem = if problem_description.trim().is_empty() {
            "RootCause precision capture"
        } else {
            problem_description.trim()
        };

        let output = Command::new("wpr")
            .args(["-stop", &output_file, problem, "-skipPdbGen", "-compress"])
            .output()
            .context("No se pudo detener WPR")?;

        if !output.status.success() {
            bail!(merge_output(&output));
        }

        Ok(format!(
            "Captura WPR detenida y guardada en {output_file}. Puedes resumirla desde la propia app o abrirla en WPA."
        ))
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (output_path, problem_description);
        Ok("Solo disponible en Windows".to_owned())
    }
}

/// Cancela la captura WPR actual sin guardar ETL.
pub fn cancel_wpr_capture() -> Result<String> {
    #[cfg(target_os = "windows")]
    {
        if !wpr_available() {
            bail!(
                "wpr.exe no está disponible. Instala Windows Performance Toolkit antes de usar modo precisión."
            );
        }

        let output = Command::new("wpr")
            .arg("-cancel")
            .output()
            .context("No se pudo cancelar WPR")?;

        if !output.status.success() {
            bail!(merge_output(&output));
        }

        Ok("Captura WPR cancelada".to_owned())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok("Solo disponible en Windows".to_owned())
    }
}

/// Exporta un ETL a XML y summary.txt usando tracerpt para análisis automatizable.
pub fn export_etl_with_tracerpt(
    etl_path: &Path,
    xml_path: &Path,
    summary_path: &Path,
) -> Result<String> {
    #[cfg(target_os = "windows")]
    {
        if !tracerpt_available() {
            bail!(
                "tracerpt.exe no está disponible. Instala Windows Performance Toolkit o usa el componente nativo presente en tu Windows."
            );
        }
        if !etl_path.exists() {
            bail!("No existe el ETL {}", etl_path.display());
        }
        if let Some(parent) = xml_path.parent() {
            fs::create_dir_all(parent)?;
        }
        if let Some(parent) = summary_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let output = Command::new("tracerpt")
            .arg(etl_path)
            .args([
                "-o",
                &xml_path.display().to_string(),
                "-of",
                "XML",
                "-lr",
                "-summary",
                &summary_path.display().to_string(),
            ])
            .output()
            .context("No se pudo ejecutar tracerpt")?;

        if !output.status.success() {
            bail!(merge_output(&output));
        }

        Ok(format!(
            "ETL exportado a XML y summary.txt en {}",
            xml_path
                .parent()
                .map(|p| p.display().to_string())
                .unwrap_or_default()
        ))
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (etl_path, xml_path, summary_path);
        Ok("Solo disponible en Windows".to_owned())
    }
}

fn merge_output(output: &Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    format!("{}{}", stdout, stderr).trim().to_owned()
}

fn map_event(value: &Value) -> EventRecord {
    EventRecord {
        timestamp: value
            .get("TimeCreated")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        provider: value
            .get("ProviderName")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        id: value.get("Id").and_then(Value::as_u64).unwrap_or_default() as u32,
        level: value
            .get("LevelDisplayName")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        message: value
            .get("Message")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .replace(['\n', '\r'], " "),
    }
}

fn map_service(value: &Value) -> ServiceState {
    ServiceState {
        name: value
            .get("Name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        display_name: value
            .get("DisplayName")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        status: value
            .get("Status")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        start_type: value
            .get("StartType")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
    }
}

fn map_persistence_entry(value: &Value) -> PersistenceEntry {
    let command = value
        .get("Command")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_owned();
    let target_path = extract_target_path(&command);
    let exists_on_disk = target_path
        .as_ref()
        .map(|path| Path::new(path).exists())
        .unwrap_or(false);

    PersistenceEntry {
        entry_kind: value
            .get("EntryKind")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        location: value
            .get("Location")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        name: value
            .get("Name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        command,
        target_path,
        exists_on_disk,
        note: "Persistencia observable desde Run/RunOnce o carpeta Startup".to_owned(),
        ..Default::default()
    }
}

fn extract_target_path(command: &str) -> Option<String> {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return None;
    }

    let candidate = if let Some(rest) = trimmed.strip_prefix('"') {
        rest.split('"').next().unwrap_or_default().trim().to_owned()
    } else if let Some(rest) = trimmed.strip_prefix("'") {
        rest.split("'").next().unwrap_or_default().trim().to_owned()
    } else {
        let lower = trimmed.to_ascii_lowercase();
        let extensions = [
            ".exe", ".cmd", ".bat", ".ps1", ".vbs", ".js", ".hta", ".lnk",
        ];
        let mut extracted = None;
        for extension in extensions {
            if let Some(index) = lower.find(extension) {
                extracted = Some(trimmed[..index + extension.len()].trim().to_owned());
                break;
            }
        }
        extracted.unwrap_or_else(|| {
            trimmed
                .split_whitespace()
                .next()
                .unwrap_or_default()
                .trim_matches('"')
                .trim_matches(char::from(39_u8))
                .to_owned()
        })
    };

    if candidate.is_empty() {
        None
    } else {
        Some(candidate)
    }
}

/// Muestra una notificación toast de Windows (no bloqueante, fire-and-forget).
pub fn show_toast_notification(title: &str, body: &str) {
    #[cfg(target_os = "windows")]
    {
        let safe_title = title.replace('\'', "\\'");
        let safe_body = body
            .replace('\'', "\\'")
            .chars()
            .take(200)
            .collect::<String>();
        let script = format!(
            r#"try {{
  $null = [Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime]
  $t = [Windows.UI.Notifications.ToastTemplateType]::ToastText02
  $xml = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent($t)
  $xml.GetElementsByTagName('text')[0].InnerText = '{safe_title}'
  $xml.GetElementsByTagName('text')[1].InnerText = '{safe_body}'
  $n = [Windows.UI.Notifications.ToastNotification]::new($xml)
  [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('RootCause Inspector').Show($n)
}} catch {{ }}"#
        );
        let _ = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-WindowStyle",
                "Hidden",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                &script,
            ])
            .spawn();
    }
    #[cfg(not(target_os = "windows"))]
    let _ = (title, body);
}

/// Obtiene la línea de comandos completa de los PIDs indicados en una sola llamada a WMI.
/// Devuelve un mapa PID → CommandLine. Los procesos sin cmdline visible se omiten.
pub fn batch_process_cmdlines(pids: &[u32]) -> HashMap<u32, String> {
    #[cfg(target_os = "windows")]
    {
        if pids.is_empty() {
            return HashMap::new();
        }
        let pid_list = pids
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let script = format!(
            "Get-CimInstance Win32_Process -Filter 'ProcessId IN ({pid_list})' \
             | Select-Object ProcessId,CommandLine \
             | ConvertTo-Json -Compress"
        );
        let Ok(raw) = powershell(&script) else {
            return HashMap::new();
        };
        parse_cmdline_json(&raw)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = pids;
        HashMap::new()
    }
}

fn parse_cmdline_json(raw: &str) -> HashMap<u32, String> {
    let mut map = HashMap::new();
    let Ok(value) = serde_json::from_str::<Value>(raw) else {
        return map;
    };
    // ConvertTo-Json devuelve un objeto si hay un solo resultado, array si hay varios.
    let entries = match &value {
        Value::Array(arr) => arr.as_slice().to_vec(),
        single => vec![single.clone()],
    };
    for entry in entries {
        let pid = entry
            .get("ProcessId")
            .and_then(Value::as_u64)
            .map(|v| v as u32);
        let cmdline = entry
            .get("CommandLine")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .map(str::to_owned);
        if let (Some(pid), Some(cmdline)) = (pid, cmdline) {
            map.insert(pid, cmdline);
        }
    }
    map
}
