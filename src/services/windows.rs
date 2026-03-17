//! Integración con utilidades nativas de Windows.
//!
//! Esta capa se apoya en comandos estándar del sistema para evitar más peso de
//! dependencias y llegar rápido a datos útiles: eventos, servicios, netstat,
//! acciones administrativas puntuales, control del modo de precisión con WPR y
//! exportación de ETL vía tracerpt.

use crate::models::{EventRecord, ServiceState};
use anyhow::{Context, Result, bail};
use serde_json::Value;
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

        return Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned());
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
        return Command::new("where")
            .arg(command)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);
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

        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
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
        Get-Service -Name wuauserv,bits,DoSvc,TrustedInstaller,SysMain -ErrorAction SilentlyContinue |
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

        return Ok(merge_output(&output));
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = pid;
        Ok("Acción disponible solo en Windows".to_owned())
    }
}

/// Crea una regla outbound para bloquear una IP remota sospechosa.
pub fn block_remote_ip(ip: &str) -> Result<String> {
    let safe_ip = ip.trim();
    let script = format!(
        "New-NetFirewallRule -DisplayName 'RootCause block {safe_ip}' -Direction Outbound -RemoteAddress {safe_ip} -Action Block | Out-Null; Write-Output 'Regla creada para bloquear {safe_ip}'"
    );
    powershell(&script)
}

/// Detiene temporalmente un servicio permitido.
pub fn stop_service(service_name: &str) -> Result<String> {
    let safe = service_name.trim();
    let script = format!(
        "Stop-Service -Name '{safe}' -Force -ErrorAction Stop; Write-Output 'Servicio {safe} detenido temporalmente'"
    );
    powershell(&script)
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
        return Ok(merged);
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

        return Ok(format!(
            "Captura WPR iniciada con GeneralProfile en file mode. Carpeta temporal de traza: {temp_dir}"
        ));
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

        return Ok(format!(
            "Captura WPR detenida y guardada en {output_file}. Puedes resumirla desde la propia app o abrirla en WPA."
        ));
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

        return Ok("Captura WPR cancelada".to_owned());
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

        return Ok(format!(
            "ETL exportado a XML y summary.txt en {}",
            xml_path
                .parent()
                .map(|p| p.display().to_string())
                .unwrap_or_default()
        ));
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
            .replace('\n', " ")
            .replace('\r', " "),
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
