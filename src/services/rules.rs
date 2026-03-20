//! Reglas y correlación ligera para RootCause.
//!
//! Separamos esta lógica de `inspector.rs` para que la colección de datos no
//! quede mezclada con la clasificación, la priorización y el armado de
//! incidentes.

use crate::config::ProcessThresholds;
use crate::models::{
    Alert, AnomalyEvent, ConnectionInsight, IncidentEvidence, IncidentSummary, PrecisionStatus,
    ProcessInsight, ServiceState, Severity, SystemOverview, SystemSnapshot, TempEntry,
};

pub struct AlertBuildInputs<'a> {
    pub processes: &'a [ProcessInsight],
    pub connections: &'a [ConnectionInsight],
    pub temp_entries: &'a [TempEntry],
    pub services: &'a [ServiceState],
    pub anomalies: &'a [AnomalyEvent],
    pub precision: &'a PrecisionStatus,
}

pub fn classify_process(
    name: &str,
    exe_path: &str,
    cpu_percent: f32,
    memory_mb: f32,
    write_delta_mb: f32,
    thresholds: &ProcessThresholds,
) -> (Severity, u8, Vec<String>, String) {
    let mut score = 0_u8;
    let mut reasons = Vec::new();
    let lower_name = name.to_ascii_lowercase();
    let lower_path = exe_path.to_ascii_lowercase();

    if cpu_percent >= thresholds.cpu_critical_percent {
        score = score.saturating_add(35);
        reasons.push(format!("CPU alto ({cpu_percent:.1}%)"));
    } else if cpu_percent >= thresholds.cpu_warning_percent {
        score = score.saturating_add(18);
        reasons.push(format!("CPU sostenido ({cpu_percent:.1}%)"));
    }

    if memory_mb >= thresholds.memory_critical_mb {
        score = score.saturating_add(28);
        reasons.push(format!("Memoria elevada ({memory_mb:.0} MB)"));
    } else if memory_mb >= thresholds.memory_warning_mb {
        score = score.saturating_add(14);
        reasons.push(format!("Memoria moderada-alta ({memory_mb:.0} MB)"));
    }

    if write_delta_mb >= thresholds.io_write_critical_mb {
        score = score.saturating_add(40);
        reasons.push(format!(
            "Escritura intensa ({write_delta_mb:.1} MB en el intervalo)"
        ));
    } else if write_delta_mb >= thresholds.io_write_warning_mb {
        score = score.saturating_add(20);
        reasons.push(format!("Escritura perceptible ({write_delta_mb:.1} MB)"));
    }

    if lower_path.contains("\\temp\\") || lower_path.contains("\\appdata\\local\\temp\\") {
        score = score.saturating_add(24);
        reasons.push("Ejecutable lanzado desde carpeta temporal".to_owned());
    }

    if [
        "update",
        "installer",
        "setup",
        "msiexec",
        "trustedinstaller",
        "dism",
    ]
    .iter()
    .any(|needle| lower_name.contains(needle))
    {
        score = score.saturating_add(12);
        reasons.push("Patrón de actualización/instalación detectado".to_owned());
    }

    let severity = match score {
        0..=24 => Severity::Healthy,
        25..=54 => Severity::Warning,
        _ => Severity::Critical,
    };

    if reasons.is_empty() {
        reasons.push("Sin presión relevante en esta muestra".to_owned());
    }

    let category = if lower_path.contains("\\temp\\") {
        "Temporal / instalador".to_owned()
    } else if lower_path.contains("\\windows\\softwaredistribution")
        || lower_name.contains("update")
    {
        "Actualización / mantenimiento".to_owned()
    } else if lower_path.starts_with(r"c:\windows") {
        "Sistema operativo".to_owned()
    } else {
        "Aplicación de usuario".to_owned()
    };

    (severity, score, reasons, category)
}

pub fn build_alerts(
    inputs: AlertBuildInputs<'_>,
    overview: &mut SystemOverview,
    max_alerts: usize,
) -> Vec<Alert> {
    let AlertBuildInputs {
        processes,
        connections,
        temp_entries,
        services,
        anomalies,
        precision,
    } = inputs;
    let mut alerts = Vec::new();

    if let Some(anomaly) = anomalies.first() {
        overview.primary_severity = anomaly.severity.to_severity();
        overview.primary_reason = anomaly.root_cause_hypothesis.clone();
        for anomaly in anomalies.iter().take(3) {
            alerts.push(Alert {
                severity: anomaly.severity.to_severity(),
                title: format!("{} [{}]", anomaly.title, anomaly.severity.label()),
                detail: anomaly.summary.clone(),
                pid: anomaly.pid,
                path: anomaly.exe_path.clone(),
                hint: anomaly.recommended_action.clone(),
            });
        }
    }

    if let Some(process) = processes
        .iter()
        .find(|process| process.severity == Severity::Critical)
    {
        overview.primary_severity = Severity::Critical;
        overview.primary_reason = format!(
            "Proceso crítico detectado: {} (PID {})",
            process.name, process.pid
        );
        alerts.push(Alert {
            severity: Severity::Critical,
            title: "Proceso dominante con presión alta".to_owned(),
            detail: format!(
                "{} usa {:.1}% CPU, {:.0} MB RAM y escribe {:.1} MB por intervalo.",
                process.name, process.cpu_percent, process.memory_mb, process.io_write_mb_delta
            ),
            pid: Some(process.pid),
            path: Some(process.exe_path.clone()),
            hint: "Revísalo primero; si confirmas que no es esencial, puedes finalizarlo"
                .to_owned(),
        });
    }

    if let Some(connection) = connections
        .iter()
        .find(|item| item.severity == Severity::Critical)
    {
        if overview.primary_severity < Severity::Critical {
            overview.primary_severity = Severity::Critical;
            overview.primary_reason = format!(
                "Conexión pública sospechosa: {} -> {}",
                connection.process_name, connection.remote_address
            );
        }
        alerts.push(Alert {
            severity: Severity::Critical,
            title: "Conexión remota a revisar".to_owned(),
            detail: format!(
                "{} (PID {}) mantiene conexión con {}.",
                connection.process_name, connection.pid, connection.remote_address
            ),
            pid: Some(connection.pid),
            path: Some(connection.exe_path.clone()),
            hint: "Valida la ruta del ejecutable y bloquea la IP si no corresponde".to_owned(),
        });
    }

    if let Some(temp_entry) = temp_entries
        .iter()
        .find(|entry| entry.severity == Severity::Critical)
    {
        if overview.primary_severity < Severity::Warning {
            overview.primary_severity = Severity::Warning;
            overview.primary_reason = format!("Crecimiento temporal alto en {}", temp_entry.path);
        }
        alerts.push(Alert {
            severity: temp_entry.severity,
            title: "Acumulación temporal relevante".to_owned(),
            detail: format!(
                "{} pesa {:.1} MB y contiene {} archivos.",
                temp_entry.path, temp_entry.size_mb, temp_entry.file_count
            ),
            pid: None,
            path: Some(temp_entry.path.clone()),
            hint: temp_entry.note.clone(),
        });
    }

    for service in services {
        let lower_name = service.name.to_ascii_lowercase();
        if (lower_name == "wuauserv" || lower_name == "bits" || lower_name == "dosvc")
            && service.status.eq_ignore_ascii_case("Running")
        {
            alerts.push(Alert {
                severity: Severity::Warning,
                title: "Servicio de actualización activo".to_owned(),
                detail: format!(
                    "{} está {} y puede explicar actividad en segundo plano.",
                    service.display_name, service.status
                ),
                pid: None,
                path: None,
                hint: "Correlaciónalo con SoftwareDistribution, Delivery Optimization y procesos de instalación/descarga".to_owned(),
            });
        }
    }

    if !precision.wpr_available {
        alerts.push(Alert {
            severity: Severity::Healthy,
            title: "Modo de precisión disponible bajo instalación adicional".to_owned(),
            detail: "La app puede operar sin WPR, pero para correlación fina conviene instalar Windows Performance Toolkit.".to_owned(),
            pid: None,
            path: None,
            hint: "Úsalo cuando necesites saber con más exactitud qué archivo o actividad disparó la lentitud.".to_owned(),
        });
    }

    if alerts.is_empty() {
        overview.primary_severity = Severity::Healthy;
        overview.primary_reason = "No se observaron conflictos claros en esta muestra".to_owned();
        alerts.push(Alert {
            severity: Severity::Healthy,
            title: "Estado estable".to_owned(),
            detail: "No aparecieron procesos o conexiones anómalas dominantes en esta captura."
                .to_owned(),
            pid: None,
            path: None,
            hint: "Mantén el monitoreo unos minutos cuando aparezca la lentitud real".to_owned(),
        });
    }

    alerts.truncate(max_alerts);
    alerts
}

pub fn derive_incident(snapshot: &SystemSnapshot) -> Option<IncidentSummary> {
    if let Some(top) = snapshot.anomalies.first() {
        let probable_causes = dedupe_strings(
            snapshot
                .anomalies
                .iter()
                .take(5)
                .map(|event| event.root_cause_hypothesis.clone())
                .collect(),
        );
        let mut recommended_actions = snapshot
            .anomalies
            .iter()
            .take(5)
            .map(|event| event.recommended_action.clone())
            .collect::<Vec<_>>();
        recommended_actions
            .push("Exportar snapshot JSON para preservar evidencia tecnica".to_owned());
        recommended_actions.push(
            "Escanear con antivirus o EDR especializado si la actividad no corresponde al contexto esperado.".to_owned(),
        );
        recommended_actions.push(
            "Correlacionar la alerta con historial reciente, servicios y persistencia local."
                .to_owned(),
        );

        let evidence = snapshot
            .anomalies
            .iter()
            .take(3)
            .flat_map(|event| event.evidence.iter().cloned())
            .take(8)
            .collect::<Vec<_>>();
        let anomaly_types = dedupe_strings(
            snapshot
                .anomalies
                .iter()
                .map(|event| event.kind.clone())
                .collect(),
        );

        return Some(IncidentSummary {
            incident_id: format!("incident-{}", snapshot.collected_at.timestamp_millis()),
            fingerprint: format!(
                "{:?}|{}|{}|{}",
                top.severity.to_severity(),
                top.kind,
                top.title,
                snapshot.anomalies.len()
            )
            .to_ascii_lowercase(),
            collected_at: snapshot.collected_at.to_owned(),
            severity: top.severity.to_severity(),
            kind: top.kind.clone(),
            title: top.title.clone(),
            summary: top.summary.clone(),
            root_cause_hypothesis: top.root_cause_hypothesis.clone(),
            probable_causes,
            recommended_actions: dedupe_strings(recommended_actions),
            evidence,
            risk_level: Some(top.severity),
            risk_score: top.score,
            anomaly_count: snapshot.anomalies.len(),
            anomaly_types,
            anomaly_events: snapshot.anomalies.iter().take(5).cloned().collect(),
            ai_advice: None,
        });
    }

    if snapshot.overview.primary_severity == Severity::Healthy && snapshot.trace_analysis.is_none()
    {
        return None;
    }

    let kind = incident_kind(snapshot);
    let title = incident_title(snapshot);
    let summary = snapshot.overview.primary_reason.clone();
    let evidence = incident_evidence(snapshot);
    let probable_causes = probable_causes(snapshot);
    let recommended_actions = recommended_actions(snapshot);

    Some(IncidentSummary {
        incident_id: format!("incident-{}", snapshot.collected_at.timestamp_millis()),
        fingerprint: format!(
            "{:?}|{}|{}",
            snapshot.overview.primary_severity, kind, title
        )
        .to_ascii_lowercase(),
        collected_at: snapshot.collected_at.to_owned(),
        severity: snapshot.overview.primary_severity,
        kind,
        title,
        summary,
        root_cause_hypothesis: snapshot.overview.primary_reason.clone(),
        probable_causes,
        recommended_actions,
        evidence,
        risk_level: None,
        risk_score: 0,
        anomaly_count: 0,
        anomaly_types: Vec::new(),
        anomaly_events: Vec::new(),
        ai_advice: None,
    })
}

fn incident_kind(snapshot: &SystemSnapshot) -> String {
    if snapshot
        .processes
        .iter()
        .any(|process| process.severity == Severity::Critical)
    {
        "process-pressure".to_owned()
    } else if snapshot
        .connections
        .iter()
        .any(|connection| connection.severity >= Severity::Warning)
    {
        "network-anomaly".to_owned()
    } else if snapshot
        .temp
        .top_entries
        .iter()
        .any(|entry| entry.severity >= Severity::Warning)
    {
        "temp-growth".to_owned()
    } else if snapshot.trace_analysis.is_some() {
        "trace-correlation".to_owned()
    } else {
        "system-degradation".to_owned()
    }
}

fn incident_title(snapshot: &SystemSnapshot) -> String {
    if let Some(alert) = snapshot.alerts.first() {
        return alert.title.clone();
    }
    snapshot.overview.primary_reason.clone()
}

fn incident_evidence(snapshot: &SystemSnapshot) -> Vec<IncidentEvidence> {
    let mut evidence = Vec::new();

    if let Some(process) = snapshot.processes.first() {
        evidence.push(IncidentEvidence {
            kind: "process".to_owned(),
            label: format!("Proceso dominante {}", process.pid),
            value: format!(
                "{} | CPU {:.1}% | RAM {:.0} MB | I/O W {:.1} MB",
                process.name, process.cpu_percent, process.memory_mb, process.io_write_mb_delta
            ),
        });
    }

    if let Some(connection) = snapshot
        .connections
        .iter()
        .find(|item| item.severity >= Severity::Warning)
    {
        evidence.push(IncidentEvidence {
            kind: "connection".to_owned(),
            label: format!("Red {}", connection.pid),
            value: format!(
                "{} -> {} ({})",
                connection.process_name, connection.remote_address, connection.reason
            ),
        });
    }

    if let Some(temp_entry) = snapshot.temp.top_entries.first() {
        evidence.push(IncidentEvidence {
            kind: "temp".to_owned(),
            label: "Temporal destacado".to_owned(),
            value: format!(
                "{} | {:.1} MB | {} archivos",
                temp_entry.path, temp_entry.size_mb, temp_entry.file_count
            ),
        });
    }

    if let Some(trace) = snapshot.trace_analysis.as_ref() {
        evidence.push(IncidentEvidence {
            kind: "trace".to_owned(),
            label: "Resumen ETL".to_owned(),
            value: format!("{} | {}", trace.headline, trace.etl_path),
        });
    }

    evidence
}

fn probable_causes(snapshot: &SystemSnapshot) -> Vec<String> {
    let mut causes = Vec::new();

    if let Some(process) = snapshot
        .processes
        .iter()
        .find(|process| process.severity >= Severity::Warning)
    {
        causes.extend(process.reasons.iter().take(3).cloned());
    }

    if let Some(connection) = snapshot
        .connections
        .iter()
        .find(|connection| connection.severity >= Severity::Warning)
    {
        causes.push(connection.reason.clone());
    }

    if let Some(temp_entry) = snapshot
        .temp
        .top_entries
        .iter()
        .find(|entry| entry.severity >= Severity::Warning)
    {
        causes.push(temp_entry.note.clone());
    }

    if let Some(trace) = snapshot.trace_analysis.as_ref() {
        causes.push(trace.headline.clone());
    }

    if causes.is_empty() {
        causes.push(snapshot.overview.primary_reason.clone());
    }

    dedupe_strings(causes)
}

fn recommended_actions(snapshot: &SystemSnapshot) -> Vec<String> {
    let mut actions = vec![
        "Exportar snapshot JSON para preservar evidencia".to_owned(),
        "Correlacionar el hallazgo con historial reciente y cambios del sistema".to_owned(),
    ];

    if snapshot.overview.primary_severity >= Severity::Warning {
        actions.push("Capturar ETW/WPR si la degradación sigue activa".to_owned());
    }

    if snapshot
        .connections
        .iter()
        .any(|connection| connection.severity >= Severity::Warning)
    {
        actions.push(
            "Validar la IP remota y bloquearla solo si confirmas que no corresponde".to_owned(),
        );
    }

    if snapshot
        .processes
        .iter()
        .any(|process| process.severity == Severity::Critical)
    {
        actions.push(
            "Revisar command line y ruta del proceso antes de finalizarlo manualmente".to_owned(),
        );
    }

    if snapshot.trace_analysis.is_none() {
        actions
            .push("Si no basta la observación liviana, subir a modo de precisión ETW".to_owned());
    }

    dedupe_strings(actions)
}

fn dedupe_strings(items: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    for item in items {
        if !out.iter().any(|existing| existing == &item) {
            out.push(item);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AnomalyEvent, RiskLevel};
    use chrono::Utc;

    #[test]
    fn proceso_critico_produce_incidente() {
        let snapshot = SystemSnapshot {
            overview: SystemOverview {
                primary_severity: Severity::Critical,
                primary_reason: "Proceso crítico detectado".to_owned(),
                ..Default::default()
            },
            alerts: vec![Alert {
                severity: Severity::Critical,
                title: "Proceso dominante con presión alta".to_owned(),
                detail: "demo".to_owned(),
                pid: Some(42),
                path: None,
                hint: "demo".to_owned(),
            }],
            processes: vec![ProcessInsight {
                pid: 42,
                name: "setup.exe".to_owned(),
                cpu_percent: 88.0,
                memory_mb: 1500.0,
                io_write_mb_delta: 220.0,
                severity: Severity::Critical,
                reasons: vec!["CPU alto".to_owned()],
                ..Default::default()
            }],
            ..Default::default()
        };

        let incident = derive_incident(&snapshot).expect("debe existir");
        assert_eq!(incident.kind, "process-pressure");
        assert_eq!(incident.severity, Severity::Critical);
        assert!(!incident.evidence.is_empty());
    }

    #[test]
    fn anomalia_prioriza_incidente_de_riesgo() {
        let snapshot = SystemSnapshot {
            anomalies: vec![AnomalyEvent {
                event_id: "anom-demo".to_owned(),
                detected_at: Utc::now(),
                severity: RiskLevel::High,
                score: 78,
                kind: "suspicious-execution-path".to_owned(),
                title: "Ejecucion desde ruta sospechosa".to_owned(),
                process_name: Some("powershell.exe".to_owned()),
                pid: Some(4242),
                summary: "Proceso ejecutado desde carpeta temporal.".to_owned(),
                root_cause_hypothesis:
                    "degradacion compatible con actividad persistente no autorizada".to_owned(),
                recommended_action: "Revisar manualmente".to_owned(),
                ..Default::default()
            }],
            ..Default::default()
        };

        let incident = derive_incident(&snapshot).expect("debe existir");
        assert_eq!(incident.kind, "suspicious-execution-path");
        assert_eq!(incident.risk_score, 78);
        assert_eq!(incident.anomaly_count, 1);
        assert_eq!(incident.risk_level, Some(RiskLevel::High));
    }
}
