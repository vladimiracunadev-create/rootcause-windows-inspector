//! Generación de reportes forenses de actividad (Markdown).
//!
//! Un reporte toma una captura del sistema y la vuelca en un documento legible y
//! archivable: veredicto de salud, incidentes/anomalías (indicios de seguridad),
//! alertas, cambios de línea base (persistencia), procesos de mayor riesgo,
//! conexiones salientes públicas y temporales. Coherente con la identidad forense:
//! son **indicios con evidencia**, no veredictos; complementa al antivirus/EDR.
//!
//! Se usa desde la GUI (botón "Reporte forense"), desde la CLI (`rootcause report`)
//! y de forma automática al final del día (la app lo genera al detectar el cambio
//! de fecha, si está habilitado en Configuración).

use crate::meta;
use crate::models::{HardwareInfo, Severity, SystemSnapshot};
use chrono::Local;
use std::fmt::Write as _;
use std::path::PathBuf;

/// Directorio donde se guardan los reportes.
pub fn reports_dir() -> PathBuf {
    dirs::data_local_dir()
        .or_else(dirs::data_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("RootCauseInspector")
        .join("reports")
}

/// Etiqueta humana de severidad para el reporte.
fn sev_word(sev: Severity) -> &'static str {
    match sev {
        Severity::Healthy => "Saludable",
        Severity::Warning => "Advertencia",
        Severity::Critical => "Crítico",
    }
}

/// Construye un reporte forense en Markdown a partir de una captura del sistema.
pub fn build_report(snap: &SystemSnapshot, hw: &HardwareInfo) -> String {
    let ov = &snap.overview;
    let now = Local::now();
    let captured = snap.collected_at.with_timezone(&Local);

    let crit = snap
        .alerts
        .iter()
        .filter(|a| matches!(a.severity, Severity::Critical))
        .count();
    let warn = snap
        .alerts
        .iter()
        .filter(|a| matches!(a.severity, Severity::Warning))
        .count();

    let mut s = String::new();

    // ── Encabezado ──────────────────────────────────────────────────────────
    let _ = writeln!(s, "# Reporte forense — RootCause Windows Inspector");
    let _ = writeln!(s);
    let _ = writeln!(s, "- **Generado:** {}", now.format("%Y-%m-%d %H:%M:%S"));
    let _ = writeln!(
        s,
        "- **Captura analizada:** {}",
        captured.format("%Y-%m-%d %H:%M:%S")
    );
    let _ = writeln!(s, "- **Versión:** RootCause v{}", meta::VERSION);
    if !hw.host_name.is_empty() {
        let _ = writeln!(
            s,
            "- **Equipo:** {} · {} {} · {} · {} núcleos · {:.1} GB RAM",
            hw.host_name, hw.os_name, hw.os_version, hw.cpu_brand, hw.cpu_cores, hw.total_ram_gb
        );
    }
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "> Reporte de **indicios** con evidencia (no veredictos). RootCause es un software \
         forense de ciberseguridad; complementa a tu antivirus/EDR, no lo reemplaza."
    );
    let _ = writeln!(s);

    // ── Veredicto ───────────────────────────────────────────────────────────
    let _ = writeln!(s, "## Veredicto");
    let _ = writeln!(s, "- **Estado general:** {}", sev_word(ov.primary_severity));
    if !ov.primary_reason.is_empty() {
        let _ = writeln!(s, "- **Motivo dominante:** {}", ov.primary_reason);
    }
    let _ = writeln!(s, "- **Alertas:** {} crítica(s) · {} aviso(s)", crit, warn);
    let _ = writeln!(s);

    // ── Métricas ────────────────────────────────────────────────────────────
    let ram_pct = ov.memory_used_gb / ov.memory_total_gb.max(0.1) * 100.0;
    let _ = writeln!(s, "## Métricas en la captura");
    let _ = writeln!(s, "| Recurso | Valor |");
    let _ = writeln!(s, "|---|---|");
    let _ = writeln!(s, "| CPU | {:.1}% |", ov.cpu_usage_percent);
    let _ = writeln!(
        s,
        "| RAM | {:.1} / {:.1} GB ({:.0}%) |",
        ov.memory_used_gb, ov.memory_total_gb, ram_pct
    );
    let _ = writeln!(
        s,
        "| Disco I/O | escritura {:.1} MB · lectura {:.1} MB |",
        ov.io_write_mb_delta, ov.io_read_mb_delta
    );
    let _ = writeln!(
        s,
        "| Red | Rx {:.1} MB · Tx {:.1} MB |",
        ov.network_rx_mb_delta, ov.network_tx_mb_delta
    );
    let _ = writeln!(s, "| Temporales vigilados | {:.0} MB |", ov.temp_total_mb);
    let _ = writeln!(s);

    // ── Incidente / causa raíz ──────────────────────────────────────────────
    if let Some(inc) = snap.incident.as_ref() {
        let _ = writeln!(s, "## Incidente / causa raíz");
        let _ = writeln!(s, "- **{}**", inc.title);
        if let Some(risk) = inc.risk_level {
            let _ = writeln!(s, "- Riesgo: {} (score {})", risk.label(), inc.risk_score);
        }
        if !inc.root_cause_hypothesis.is_empty() {
            let _ = writeln!(s, "- Hipótesis: {}", inc.root_cause_hypothesis);
        }
        if !inc.anomaly_types.is_empty() {
            let _ = writeln!(s, "- Señales: {}", inc.anomaly_types.join(", "));
        }
        for a in inc.recommended_actions.iter().take(4) {
            let _ = writeln!(s, "  - Acción sugerida: {a}");
        }
        let _ = writeln!(s);
    }

    // ── Anomalías (indicios de comportamiento) ──────────────────────────────
    if !snap.anomalies.is_empty() {
        let _ = writeln!(s, "## Indicios de comportamiento (anomalías)");
        for a in snap.anomalies.iter().take(15) {
            let who = a.process_name.as_deref().unwrap_or("—");
            let _ = writeln!(
                s,
                "- **{}** `{}` · {} (score {}) — {}",
                a.title, a.kind, who, a.score, a.summary
            );
        }
        let _ = writeln!(s);
    }

    // ── Dónde mirar primero (alertas) ───────────────────────────────────────
    if !snap.alerts.is_empty() {
        let _ = writeln!(s, "## Dónde mirar primero");
        for al in snap.alerts.iter().take(10) {
            let pid = al.pid.map(|p| format!(" (PID {p})")).unwrap_or_default();
            let _ = writeln!(
                s,
                "- [{}] **{}**{pid} — {}",
                sev_word(al.severity),
                al.title,
                al.detail
            );
            if let Some(path) = &al.path {
                let _ = writeln!(s, "  - `{path}`");
            }
        }
        let _ = writeln!(s);
    }

    // ── Cambios de línea base (persistencia / autoarranque) ─────────────────
    let persistence_changes: Vec<_> = snap
        .persistence_entries
        .iter()
        .filter(|e| e.change_status.is_change())
        .collect();
    if !persistence_changes.is_empty() {
        let _ = writeln!(s, "## Cambios de autoarranque vs línea base (persistencia)");
        for e in persistence_changes.iter().take(20) {
            let _ = writeln!(
                s,
                "- **{}** · {} — {} `{}`",
                e.change_status.label(),
                e.entry_kind,
                e.name,
                e.command
            );
        }
        let _ = writeln!(s);
    }

    // ── Procesos de mayor riesgo ────────────────────────────────────────────
    if !snap.processes.is_empty() {
        let mut procs: Vec<_> = snap.processes.iter().collect();
        procs.sort_by_key(|p| std::cmp::Reverse(p.score));
        let _ = writeln!(s, "## Procesos de mayor riesgo");
        let _ = writeln!(s, "| Proceso | PID | CPU | RAM | Escr. disco | Riesgo |");
        let _ = writeln!(s, "|---|---:|---:|---:|---:|---:|");
        for p in procs.iter().take(10) {
            let _ = writeln!(
                s,
                "| {} | {} | {:.1}% | {:.0} MB | {:.1} MB | {} |",
                p.name, p.pid, p.cpu_percent, p.memory_mb, p.io_write_mb_delta, p.score
            );
        }
        let _ = writeln!(s);
    }

    // ── Conexiones salientes públicas (posible C2 / exfiltración) ────────────
    let public_conns: Vec<_> = snap
        .connections
        .iter()
        .filter(|c| c.is_public_remote)
        .collect();
    if !public_conns.is_empty() {
        let _ = writeln!(s, "## Conexiones salientes a IP pública");
        let _ = writeln!(s, "| Proceso | PID | Destino | Estado |");
        let _ = writeln!(s, "|---|---:|---|---|");
        for c in public_conns.iter().take(15) {
            let _ = writeln!(
                s,
                "| {} | {} | {} | {} |",
                c.process_name, c.pid, c.remote_address, c.state
            );
        }
        let _ = writeln!(s);
    }

    // ── Temporales que crecen ───────────────────────────────────────────────
    if !snap.temp.top_entries.is_empty() {
        let _ = writeln!(s, "## Temporales de mayor tamaño");
        for t in snap.temp.top_entries.iter().take(8) {
            let _ = writeln!(s, "- {:.1} MB — `{}`", t.size_mb, t.path);
        }
        let _ = writeln!(s);
    }

    // ── Salud del agente ────────────────────────────────────────────────────
    let _ = writeln!(s, "## Salud del agente RootCause");
    let _ = writeln!(s, "- Estado: {}", snap.agent_health.status.label());
    if !snap.agent_health.summary.is_empty() {
        let _ = writeln!(s, "- {}", snap.agent_health.summary);
    }
    let _ = writeln!(s);

    let _ = writeln!(
        s,
        "---\n_Generado por RootCause v{}. Los hallazgos son indicios para investigar, \
         no confirmaciones de amenaza._",
        meta::VERSION
    );

    s
}

/// Guarda el reporte en el directorio de reportes con nombre por marca de tiempo.
/// Devuelve la ruta del archivo escrito.
pub fn save_report(content: &str) -> std::io::Result<PathBuf> {
    let dir = reports_dir();
    std::fs::create_dir_all(&dir)?;
    let name = format!("rootcause-{}.md", Local::now().format("%Y%m%d-%H%M%S"));
    let path = dir.join(name);
    std::fs::write(&path, content)?;
    Ok(path)
}
