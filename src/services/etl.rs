//! Resumen asistido de capturas ETL.
//!
//! La meta no es reemplazar a WPA. La meta es reducir el tiempo hasta el primer
//! hallazgo útil: detectar procesos, rutas, servicios, actualizaciones o IPs que
//! aparezcan repetidamente en la traza exportada por herramientas oficiales.

use crate::models::{
    Severity, TraceAnalysisSummary, TraceFinding, TracePathSummary, TraceProcessSummary,
};
use anyhow::{Context, Result, bail};
use chrono::Utc;
use regex::Regex;
use std::cmp::Reverse;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

#[derive(Default)]
struct WorkingEvent {
    provider: String,
    event_id: Option<u32>,
    event_name: String,
    values: Vec<(String, String)>,
}

#[derive(Default)]
struct WorkingSummary {
    total_events: u64,
    provider_counts: HashMap<String, u64>,
    process_counts: HashMap<String, u64>,
    process_reasons: HashMap<String, String>,
    path_counts: HashMap<String, u64>,
    path_category: HashMap<String, String>,
    path_severity: HashMap<String, Severity>,
    ip_counts: HashMap<String, u64>,
    indicators: HashMap<String, u64>,
    findings: Vec<TraceFinding>,
}

/// Analiza un ETL ya transformado a XML/SUMMARY por `tracerpt` y genera un JSON resumido.
pub fn summarize_exported_etl(
    etl_path: &Path,
    xml_path: &Path,
    summary_txt_path: &Path,
    output_directory: &Path,
) -> Result<TraceAnalysisSummary> {
    if !xml_path.exists() {
        bail!("No existe el XML exportado {}", xml_path.display());
    }

    let mut state = WorkingSummary::default();
    parse_xml_events(xml_path, &mut state)?;
    let summary_txt = read_summary_excerpt(summary_txt_path);

    let mut providers: Vec<(String, u64)> = state.provider_counts.into_iter().collect();
    providers.sort_by_key(|item| Reverse(item.1));
    providers.truncate(8);

    let mut hot_processes: Vec<TraceProcessSummary> = state
        .process_counts
        .into_iter()
        .map(|(name, occurrences)| TraceProcessSummary {
            reason: state
                .process_reasons
                .get(&name)
                .cloned()
                .unwrap_or_else(|| "Proceso repetido dentro de la traza".to_owned()),
            severity: infer_process_severity(&name),
            name,
            occurrences,
        })
        .collect();
    hot_processes.sort_by_key(|item| Reverse(item.occurrences));
    hot_processes.truncate(8);

    let mut hot_paths: Vec<TracePathSummary> = state
        .path_counts
        .into_iter()
        .map(|(path, occurrences)| TracePathSummary {
            category: state
                .path_category
                .get(&path)
                .cloned()
                .unwrap_or_else(|| "ruta".to_owned()),
            severity: state
                .path_severity
                .get(&path)
                .copied()
                .unwrap_or(Severity::Healthy),
            path,
            occurrences,
        })
        .collect();
    hot_paths.sort_by_key(|item| Reverse(item.occurrences));
    hot_paths.truncate(10);

    let mut public_ips: Vec<String> = state.ip_counts.into_iter().map(|(ip, _)| ip).collect();
    public_ips.sort();
    public_ips.truncate(10);

    let mut indicators: Vec<(String, u64)> = state.indicators.into_iter().collect();
    indicators.sort_by_key(|item| Reverse(item.1));
    let indicator_texts: Vec<String> = indicators
        .iter()
        .map(|(text, count)| format!("{text} ({count})"))
        .take(12)
        .collect();

    if state.findings.is_empty() {
        state.findings.push(TraceFinding {
            severity: Severity::Healthy,
            title: "Sin hallazgo dominante en el resumen automático".to_owned(),
            detail: "La traza fue procesada, pero el análisis automático no encontró una señal claramente dominante. Usa WPA para profundizar en el intervalo exacto.".to_owned(),
            evidence: format!("Eventos procesados: {}", state.total_events),
        });
    }

    let headline = build_headline(
        &state.findings,
        &hot_processes,
        &hot_paths,
        &indicator_texts,
    );
    let confidence = build_confidence(&summary_txt, state.total_events, &hot_paths, &hot_processes);

    let mut limitations = vec![
        "Este resumen usa exportación por tracerpt y heurísticas propias; no sustituye la exploración pivoteada de WPA.".to_owned(),
        "La exactitud depende de que la captura WPR cubra el momento exacto del problema.".to_owned(),
        "Si quieres relación temporal fina por milisegundo o pilas, abre el ETL en WPA y carga símbolos.".to_owned(),
    ];
    if !summary_txt.is_empty() {
        limitations.push(format!("Extracto de summary.txt: {summary_txt}"));
    }

    let analysis = TraceAnalysisSummary {
        engine: "tracerpt + heurísticas RootCause".to_owned(),
        analyzed_at: Utc::now().to_rfc3339(),
        etl_path: etl_path.display().to_string(),
        output_directory: output_directory.display().to_string(),
        raw_xml_path: Some(xml_path.display().to_string()),
        raw_summary_path: Some(summary_txt_path.display().to_string()),
        total_events: state.total_events,
        headline,
        confidence,
        findings: state.findings,
        hot_processes,
        hot_paths,
        public_ips,
        providers,
        indicators: indicator_texts,
        limitations,
    };

    fs::create_dir_all(output_directory)?;
    let output_json = output_directory.join("trace-analysis.json");
    fs::write(&output_json, serde_json::to_string_pretty(&analysis)?)
        .with_context(|| format!("No se pudo escribir {}", output_json.display()))?;
    Ok(analysis)
}

fn parse_xml_events(xml_path: &Path, state: &mut WorkingSummary) -> Result<()> {
    let file =
        File::open(xml_path).with_context(|| format!("No se pudo abrir {}", xml_path.display()))?;
    let reader = BufReader::new(file);
    let mut event = None::<WorkingEvent>;

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.starts_with("<Event") {
            event = Some(WorkingEvent::default());
            continue;
        }
        if trimmed.starts_with("</Event>") {
            if let Some(completed) = event.take() {
                apply_event(completed, state);
            }
            continue;
        }

        let Some(current) = event.as_mut() else {
            continue;
        };

        if trimmed.starts_with("<Provider") {
            if let Some(name) = extract_attr(trimmed, "Name") {
                current.provider = name;
            } else if let Some(guid) = extract_attr(trimmed, "Guid") {
                current.provider = guid;
            }
        } else if trimmed.starts_with("<EventID>") {
            current.event_id = extract_text(trimmed).parse::<u32>().ok();
        } else if trimmed.starts_with("<EventName") {
            current.event_name = extract_text(trimmed);
        } else if trimmed.starts_with("<Data") {
            let name = extract_attr(trimmed, "Name").unwrap_or_else(|| "value".to_owned());
            let value = cleanup_value(&extract_text(trimmed));
            if !value.is_empty() {
                current.values.push((name, value));
            }
        }
    }

    Ok(())
}

fn apply_event(event: WorkingEvent, state: &mut WorkingSummary) {
    state.total_events = state.total_events.saturating_add(1);
    if !event.provider.is_empty() {
        *state
            .provider_counts
            .entry(event.provider.clone())
            .or_insert(0) += 1;
    }

    let mut local_paths = Vec::new();
    let mut local_processes = Vec::new();
    let mut local_ips = Vec::new();
    let mut local_text = format!("{} {}", event.provider, event.event_name).to_ascii_lowercase();

    for (name, value) in &event.values {
        let lowered = value.to_ascii_lowercase();
        local_text.push(' ');
        local_text.push_str(name);
        local_text.push(' ');
        local_text.push_str(&lowered);

        if looks_like_windows_path(value) {
            local_paths.push(value.clone());
        }
        if let Some(proc_name) = detect_process_name(name, value) {
            local_processes.push(proc_name);
        }
        local_ips.extend(extract_public_ips(value));
    }

    for path in local_paths {
        *state.path_counts.entry(path.clone()).or_insert(0) += 1;
        state
            .path_category
            .entry(path.clone())
            .or_insert_with(|| categorize_path(&path));
        state
            .path_severity
            .entry(path.clone())
            .or_insert_with(|| severity_for_path(&path));

        if is_temp_executable_path(&path) {
            push_finding_once(
                &mut state.findings,
                TraceFinding {
                    severity: Severity::Critical,
                    title: "Ejecutable observado desde carpeta temporal".to_owned(),
                    detail: "La traza contiene referencias repetidas a un binario ejecutado desde una ruta temporal. Esto merece validación inmediata.".to_owned(),
                    evidence: path.clone(),
                },
            );
        }
        if is_windows_update_path(&path) {
            *state
                .indicators
                .entry("Actividad compatible con Windows Update / servicing".to_owned())
                .or_insert(0) += 1;
        }
        if is_delivery_optimization_path(&path) {
            *state
                .indicators
                .entry("Actividad compatible con Delivery Optimization".to_owned())
                .or_insert(0) += 1;
        }
    }

    for proc_name in local_processes {
        *state.process_counts.entry(proc_name.clone()).or_insert(0) += 1;
        state
            .process_reasons
            .entry(proc_name.clone())
            .or_insert_with(|| reason_for_process(&proc_name));
        if looks_like_update_process(&proc_name) {
            *state
                .indicators
                .entry("Procesos de actualización/instalación dentro de la traza".to_owned())
                .or_insert(0) += 1;
        }
    }

    for ip in local_ips {
        *state.ip_counts.entry(ip.clone()).or_insert(0) += 1;
        push_finding_once(
            &mut state.findings,
            TraceFinding {
                severity: Severity::Warning,
                title: "IP pública observada dentro del ETL".to_owned(),
                detail: "La traza contiene al menos una IP pública. Debes validar si corresponde al software esperado o a actualizaciones legítimas.".to_owned(),
                evidence: ip,
            },
        );
    }

    if local_text.contains("trustedinstaller")
        || local_text.contains("wuauserv")
        || local_text.contains("musnotification")
    {
        *state
            .indicators
            .entry("Señales de servicing o componentes del sistema".to_owned())
            .or_insert(0) += 1;
    }
    if local_text.contains("bits") || local_text.contains("background intelligent transfer service")
    {
        *state
            .indicators
            .entry("BITS involucrado en la captura".to_owned())
            .or_insert(0) += 1;
    }
}

fn extract_attr(line: &str, attr: &str) -> Option<String> {
    let pattern = format!("{attr}=\"");
    let start = line.find(&pattern)? + pattern.len();
    let tail = &line[start..];
    let end = tail.find('"')?;
    Some(cleanup_value(&tail[..end]))
}

fn extract_text(line: &str) -> String {
    let start = match line.find('>') {
        Some(value) => value + 1,
        None => return String::new(),
    };
    let tail = &line[start..];
    let end = tail.find('<').unwrap_or(tail.len());
    cleanup_value(&tail[..end])
}

fn cleanup_value(value: &str) -> String {
    value
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .trim()
        .to_owned()
}

fn looks_like_windows_path(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.contains(":\\") || trimmed.starts_with("\\\\")
}

fn detect_process_name(field_name: &str, value: &str) -> Option<String> {
    let lower_field = field_name.to_ascii_lowercase();
    let lower_value = value.to_ascii_lowercase();
    if lower_field.contains("image")
        || lower_field.contains("process")
        || lower_field.contains("command")
    {
        if let Some(last) = lower_value.rsplit(['\\', '/']).next() {
            if last.ends_with(".exe") || last.ends_with(".dll") || last.ends_with(".sys") {
                return Some(last.to_owned());
            }
        }
        if lower_value.ends_with(".exe") {
            return Some(lower_value);
        }
    }
    None
}

fn extract_public_ips(value: &str) -> Vec<String> {
    let regex = Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").expect("regex válida");
    regex
        .find_iter(value)
        .filter_map(|capture| {
            let ip = capture.as_str();
            if is_public_ip(ip) {
                Some(ip.to_owned())
            } else {
                None
            }
        })
        .collect()
}

fn is_public_ip(ip: &str) -> bool {
    let ip = ip.trim();
    !(ip.starts_with("10.")
        || ip.starts_with("127.")
        || ip.starts_with("192.168.")
        || ip.starts_with("169.254.")
        || ip == "0.0.0.0"
        || ip.is_empty()
        || private_172(ip))
}

fn private_172(ip: &str) -> bool {
    let Some(rest) = ip.strip_prefix("172.") else {
        return false;
    };
    let second = rest
        .split('.')
        .next()
        .and_then(|value| value.parse::<u8>().ok());
    matches!(second, Some(value) if (16..=31).contains(&value))
}

fn categorize_path(path: &str) -> String {
    let lower = path.to_ascii_lowercase();
    if is_windows_update_path(path) {
        "windows-update".to_owned()
    } else if is_delivery_optimization_path(path) {
        "delivery-optimization".to_owned()
    } else if lower.contains("\\temp\\") || lower.contains("\\appdata\\local\\temp\\") {
        "temporal".to_owned()
    } else if lower.starts_with(r"c:\windows") {
        "sistema".to_owned()
    } else {
        "usuario".to_owned()
    }
}

fn severity_for_path(path: &str) -> Severity {
    if is_temp_executable_path(path) {
        Severity::Critical
    } else if is_windows_update_path(path) || is_delivery_optimization_path(path) {
        Severity::Warning
    } else {
        Severity::Healthy
    }
}

fn is_temp_executable_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    (lower.contains("\\temp\\") || lower.contains("\\appdata\\local\\temp\\"))
        && (lower.ends_with(".exe") || lower.ends_with(".dll") || lower.ends_with(".msi"))
}

fn is_windows_update_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.contains("\\softwaredistribution\\")
        || lower.contains("catroot")
        || lower.contains("winsxs")
}

fn is_delivery_optimization_path(path: &str) -> bool {
    path.to_ascii_lowercase().contains("deliveryoptimization")
}

fn looks_like_update_process(name: &str) -> bool {
    [
        "trustedinstaller",
        "tiworker",
        "msiexec",
        "setup",
        "dism",
        "mouso",
        "usoclient",
        "wuauclt",
    ]
    .iter()
    .any(|needle| name.contains(needle))
}

fn reason_for_process(name: &str) -> String {
    if looks_like_update_process(name) {
        "Patrón de actualización/instalación repetido dentro de la traza".to_owned()
    } else if name.contains("svchost") {
        "Proceso del sistema muy presente; correlaciónalo con servicios y rutas".to_owned()
    } else {
        "Imagen repetida dentro del ETL".to_owned()
    }
}

fn infer_process_severity(name: &str) -> Severity {
    if looks_like_update_process(name) {
        Severity::Warning
    } else if name.contains("temp") {
        Severity::Critical
    } else {
        Severity::Healthy
    }
}

fn push_finding_once(target: &mut Vec<TraceFinding>, finding: TraceFinding) {
    if target
        .iter()
        .any(|item| item.title == finding.title && item.evidence == finding.evidence)
    {
        return;
    }
    target.push(finding);
}

fn build_headline(
    findings: &[TraceFinding],
    hot_processes: &[TraceProcessSummary],
    hot_paths: &[TracePathSummary],
    indicators: &[String],
) -> String {
    if let Some(finding) = findings
        .iter()
        .find(|item| item.severity == Severity::Critical)
    {
        return finding.title.clone();
    }
    if hot_paths
        .iter()
        .any(|item| item.category == "windows-update")
    {
        return "La traza sugiere actividad relacionada con Windows Update o servicing".to_owned();
    }
    if let Some(process) = hot_processes.first() {
        return format!("Proceso más repetido en la traza: {}", process.name);
    }
    if let Some(indicator) = indicators.first() {
        return format!("Indicador dominante: {indicator}");
    }
    "Traza procesada sin hallazgo dominante automático".to_owned()
}

fn build_confidence(
    summary_excerpt: &str,
    total_events: u64,
    hot_paths: &[TracePathSummary],
    hot_processes: &[TraceProcessSummary],
) -> String {
    if total_events >= 500 && (!hot_paths.is_empty() || !hot_processes.is_empty()) {
        return "Media-alta: la captura tiene suficiente volumen para orientar la investigación"
            .to_owned();
    }
    if !summary_excerpt.is_empty() {
        return "Media: hay señal, pero conviene abrir WPA para ver el intervalo exacto".to_owned();
    }
    "Baja-media: la captura fue procesada, pero la evidencia resumida es limitada".to_owned()
}

fn read_summary_excerpt(summary_txt_path: &Path) -> String {
    if !summary_txt_path.exists() {
        return String::new();
    }
    let text = fs::read_to_string(summary_txt_path).unwrap_or_default();
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take(3)
        .collect::<Vec<_>>()
        .join(" | ")
}

/// Dado un ETL, devuelve las rutas donde deben quedar sus artefactos de resumen.
pub fn analysis_layout(base_output_dir: &Path, etl_path: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let stem = etl_path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("trace")
        .to_owned();
    let output_dir = base_output_dir.join(stem);
    let xml_path = output_dir.join("dumpfile.xml");
    let summary_path = output_dir.join("summary.txt");
    let json_path = output_dir.join("trace-analysis.json");
    (xml_path, summary_path, json_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconoce_ruta_temporal_ejecutable() {
        assert!(is_temp_executable_path(
            r"C:\Users\vbav\AppData\Local\Temp\setup.exe"
        ));
        assert!(!is_temp_executable_path(r"C:\Windows\System32\notepad.exe"));
    }

    #[test]
    fn clasifica_actualizacion() {
        assert!(is_windows_update_path(
            r"C:\Windows\SoftwareDistribution\Download\a.cab"
        ));
        assert!(looks_like_update_process("trustedinstaller.exe"));
    }
}
